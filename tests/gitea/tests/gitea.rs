use std::{
    env, fs,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, anyhow, bail, ensure};
use reqwest::{
    Method,
    blocking::{Client, Response},
};
use serde_json::{Value, json};

const USER: &str = "release-test";
const DIRECTORY: &str = env!("CARGO_MANIFEST_DIR");

#[test]
fn action_on_gitea() -> Result<()> {
    let mut compose = Compose::up()?;
    let address = capture(compose.command().args(["port", "gitea", "3000"]))?;
    run(compose.admin_user().args([
        "create",
        "--username",
        USER,
        "--password",
        "integration-test-password-not-a-secret",
        "--email",
        "release-test@example.com",
        "--must-change-password=false",
    ]))?;
    let token = capture(compose.admin_user().args([
        "generate-access-token",
        "--username",
        USER,
        "--token-name",
        "integration",
        "--scopes",
        "all",
        "--raw",
    ]))?;
    if env::var("GITHUB_ACTIONS").as_deref() == Ok("true") {
        println!("::add-mask::{token}");
    }
    let gitea = Gitea {
        address,
        token,
        client: Client::builder().timeout(Duration::from_secs(30)).build()?,
    };
    test_release_pr(&gitea).context("Gitea release PR test failed")?;
    compose.passed = true;
    Ok(())
}

fn test_release_pr(gitea: &Gitea) -> Result<()> {
    let repo = format!("{USER}/forge");
    let path = format!("/repos/{repo}");
    gitea.send(
        Method::POST,
        "/user/repos",
        Some(&json!({"name": "forge", "default_branch": "main"})),
    )?;
    gitea.send(Method::PATCH, &path, Some(&json!({"has_actions": true})))?;
    gitea.send(
        Method::PUT,
        &format!("{path}/actions/secrets/RELEASE_PLZ_TOKEN"),
        Some(&json!({"data": gitea.token})),
    )?;

    let temp = tempfile::tempdir()?;
    let fixture = temp.path().join("fixture");
    let directory = Path::new(DIRECTORY);
    copy_dir(&directory.join("fixture"), &fixture)?;
    fs::copy(
        directory.join("../../action.yml"),
        fixture.join("action.yml"),
    )?;
    let git = || {
        let mut command = Command::new("git");
        command.arg("-C").arg(&fixture);
        command
    };
    run(git().args(["init", "--initial-branch=main"]))?;
    run(git().args(["config", "user.name", "Release test"]))?;
    run(git().args(["config", "user.email", "release-test@example.com"]))?;
    run(git().args(["config", "commit.gpgsign", "false"]))?;
    run(git().args(["config", "tag.gpgsign", "false"]))?;
    run(git().args(["add", "."]))?;
    run(git().args(["commit", "-m", "feat: initial release"]))?;
    run(git().args(["tag", "v0.1.0"]))?;
    let source = fixture.join("src/lib.rs");
    fs::write(
        &source,
        fs::read_to_string(&source)?.replace("Hello, Gitea!", "Hello from Gitea!"),
    )?;
    run(git().args(["add", "."]))?;
    run(git().args(["commit", "-m", "fix: improve greeting"]))?;
    let sha = capture(git().args(["rev-parse", "HEAD"]))?;
    // No remote is configured: the workflow clones the repository from Gitea.
    run(git()
        .args(["push", "--atomic"])
        .arg(format!(
            "http://{USER}:{}@{}/{repo}.git",
            gitea.token, gitea.address
        ))
        .args(["main", "refs/tags/v0.1.0"])
        .env("GIT_TERMINAL_PROMPT", "0"))?;

    wait_for_workflow(gitea, &repo, &sha)?;
    let prs = gitea.get(&format!("{path}/pulls?state=open"))?;
    let prs = prs
        .as_array()
        .context("expected an array of pull requests")?;
    ensure!(
        prs.len() == 1
            && prs[0]["title"] == "chore: release v0.1.1"
            && prs[0]["base"]["ref"] == "main"
            && prs[0]["head"]["ref"]
                .as_str()
                .is_some_and(|branch| branch.starts_with("release-plz-")),
        "expected one release PR for v0.1.1 from a release-plz branch onto main, got {prs:?}"
    );
    let number = prs[0]["number"].as_u64().context("missing PR number")?;
    let files = gitea.get(&format!("{path}/pulls/{number}/files"))?;
    let files = files
        .as_array()
        .context("expected an array of changed files")?;
    for expected in ["Cargo.toml", "CHANGELOG.md"] {
        ensure!(
            files.iter().any(|file| file["filename"] == expected),
            "release PR did not change {expected}"
        );
    }
    println!("PASS: created a Gitea release PR and returned matching outputs");
    Ok(())
}

fn wait_for_workflow(gitea: &Gitea, repo: &str, sha: &str) -> Result<()> {
    println!("Waiting for Gitea workflow: {repo}");
    let deadline = Instant::now() + Duration::from_secs(900);
    let mut previous_status = Value::Null;
    let mut run_id = None;
    let failure = loop {
        if Instant::now() >= deadline {
            break anyhow!("Gitea workflow did not succeed within 15 minutes");
        }
        let runs = gitea.get(&format!("/repos/{repo}/actions/runs"))?;
        let runs = runs["workflow_runs"]
            .as_array()
            .context("missing workflow_runs array")?;
        if let Some(run) = runs
            .iter()
            .find(|run| run["head_sha"] == sha && run["event"] == "push")
        {
            run_id = run["id"].as_u64();
            let status = json!([run["status"], run["conclusion"]]);
            if status != previous_status {
                println!("{repo}: {status}");
                previous_status = status;
            }
            if run["status"] == "completed" {
                if run["conclusion"] == "success" {
                    return Ok(());
                }
                break anyhow!("workflow failed: {run}");
            }
        }
        thread::sleep(Duration::from_secs(3));
    };
    match gitea.job_logs(repo, run_id) {
        Ok(logs) => println!("{logs}"),
        Err(error) => eprintln!("Could not retrieve workflow logs: {error:#}"),
    }
    Err(failure)
}

struct Gitea {
    address: String,
    token: String,
    client: Client,
}

impl Gitea {
    fn send(&self, method: Method, path: &str, body: Option<&Value>) -> Result<Response> {
        let description = format!("{method} {path}");
        let mut request = self
            .client
            .request(method, format!("http://{}/api/v1{path}", self.address))
            .header("Authorization", format!("token {}", self.token));
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request.send().with_context(|| description.clone())?;
        let status = response.status();
        if !status.is_success() {
            bail!("{description}: {status} {}", response.text()?);
        }
        Ok(response)
    }

    fn get(&self, path: &str) -> Result<Value> {
        self.send(Method::GET, path, None)?
            .json()
            .with_context(|| format!("invalid JSON from {path}"))
    }

    fn job_logs(&self, repo: &str, run_id: Option<u64>) -> Result<String> {
        let run_id = run_id.context("no workflow run was created")?;
        let jobs = self.get(&format!("/repos/{repo}/actions/runs/{run_id}/jobs"))?;
        let mut logs = String::new();
        for job in jobs["jobs"].as_array().context("missing jobs array")? {
            let id = job["id"].as_u64().context("missing job ID")?;
            let path = format!("/repos/{repo}/actions/jobs/{id}/logs");
            logs.push_str(&self.send(Method::GET, &path, None)?.text()?);
        }
        Ok(logs)
    }
}

struct Compose {
    passed: bool,
}

impl Compose {
    /// Removes what a killed previous run left behind (`Drop` never ran for it),
    /// then starts the stack.
    fn up() -> Result<Self> {
        let compose = Self { passed: false };
        compose
            .down()
            .context("could not clean up a previous run")?;
        run(compose.command().args([
            "up",
            "--build",
            "--detach",
            "--wait",
            "--wait-timeout",
            "180",
        ]))?;
        Ok(compose)
    }

    fn command(&self) -> Command {
        let mut command = Command::new("docker");
        command
            .args(["compose", "-f"])
            .arg(Path::new(DIRECTORY).join("compose.yml"))
            // Both can override the top-level `name` in compose.yml, which `down` relies on.
            .env_remove("COMPOSE_PROJECT_NAME")
            .env_remove("COMPOSE_ENV_FILES");
        command
    }

    fn down(&self) -> Result<()> {
        run(self.command().args([
            "down",
            "--volumes",
            "--rmi",
            "local",
            "--remove-orphans",
            "--timeout",
            "5",
        ]))
    }

    fn admin_user(&self) -> Command {
        let mut command = self.command();
        command.args(["exec", "-T", "gitea", "gitea", "admin", "user"]);
        command
    }
}

impl Drop for Compose {
    fn drop(&mut self) {
        if !self.passed {
            let _ = run(self.command().args(["logs", "--no-color", "--tail", "200"]));
        }
        if let Err(error) = self.down() {
            eprintln!("Could not clean up Gitea test containers: {error:#}");
        }
    }
}

fn run(command: &mut Command) -> Result<()> {
    let program = command.get_program().to_string_lossy().into_owned();
    let status = command
        .status()
        .with_context(|| format!("could not start {program}"))?;
    ensure!(status.success(), "{program} exited with {status}");
    Ok(())
}

fn capture(command: &mut Command) -> Result<String> {
    let program = command.get_program().to_string_lossy().into_owned();
    let output = command
        .stderr(Stdio::inherit())
        .output()
        .with_context(|| format!("could not start {program}"))?;
    ensure!(
        output.status.success(),
        "{program} exited with {}",
        output.status
    );
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn copy_dir(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
