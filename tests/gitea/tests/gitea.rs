use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail, ensure};
use reqwest::{Method, blocking::Client};
use serde_json::{Value, json};

const USER: &str = "release-test";

#[test]
fn action_on_gitea() -> Result<()> {
    let mut stack = Stack::new()?;
    run(stack.command().args([
        "up",
        "--build",
        "--detach",
        "--wait",
        "--wait-timeout",
        "180",
    ]))?;
    let address = capture(stack.command().args(["port", "gitea", "3000"]))?;
    run(stack.admin().args([
        "create",
        "--username",
        USER,
        "--password",
        "integration-test-password-not-a-secret",
        "--email",
        "release-test@example.com",
        "--must-change-password=false",
    ]))?;
    let token = capture(stack.admin().args([
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
        url: format!("http://{address}/api/v1"),
        token,
        client: Client::builder().timeout(Duration::from_secs(30)).build()?,
    };
    test_release_pr(&stack, &gitea, &address).context("Gitea release PR test failed")?;
    stack.passed = true;
    Ok(())
}

fn test_release_pr(stack: &Stack, gitea: &Gitea, address: &str) -> Result<()> {
    let repo = format!("{USER}/forge");
    let path = format!("/repos/{repo}");
    gitea.request(
        Method::POST,
        "/user/repos",
        json!({"name": "forge", "default_branch": "main"}),
    )?;
    gitea.request(Method::PATCH, &path, json!({"has_actions": true}))?;
    gitea.request(
        Method::PUT,
        &format!("{path}/actions/secrets/RELEASE_PLZ_TOKEN"),
        json!({"data": gitea.token}),
    )?;

    let temp = tempfile::tempdir()?;
    let fixture = temp.path().join("fixture");
    copy_dir(&stack.directory.join("fixture"), &fixture)?;
    fs::copy(
        stack.directory.join("../../action.yml"),
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
            "http://{USER}:{}@{address}/{repo}.git",
            gitea.token
        ))
        .args(["main", "refs/tags/v0.1.0"])
        .env("GIT_TERMINAL_PROMPT", "0"))?;

    wait_for_workflow(gitea, &path, &sha)?;
    let prs = gitea.get(&format!("{path}/pulls?state=open"))?;
    let prs = prs
        .as_array()
        .context("expected an array of pull requests")?;
    ensure!(
        prs.len() == 1 && prs[0]["title"] == "chore: release v0.1.1",
        "expected one release PR for v0.1.1, got {prs:?}"
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
    while Instant::now() < deadline {
        let runs = gitea.get(&format!("{repo}/actions/runs"))?;
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
                if run["conclusion"] != "success" {
                    gitea.print_job_logs(repo, run_id);
                    bail!("workflow failed: {run}");
                }
                return Ok(());
            }
        }
        thread::sleep(Duration::from_secs(3));
    }
    gitea.print_job_logs(repo, run_id);
    bail!("Gitea workflow did not succeed within 15 minutes")
}

struct Gitea {
    url: String,
    token: String,
    client: Client,
}

impl Gitea {
    fn text(&self, method: Method, path: &str, data: Value) -> Result<String> {
        let mut request = self
            .client
            .request(method.clone(), format!("{}{path}", self.url))
            .header("Authorization", format!("token {}", self.token));
        if !data.is_null() {
            request = request.json(&data);
        }
        let response = request.send().with_context(|| format!("{method} {path}"))?;
        let status = response.status();
        let body = response.text()?;
        ensure!(status.is_success(), "{method} {path}: {status} {body}");
        Ok(body)
    }

    fn request(&self, method: Method, path: &str, data: Value) -> Result<Value> {
        let body = self.text(method, path, data)?;
        if body.is_empty() {
            Ok(Value::Null)
        } else {
            serde_json::from_str(&body).with_context(|| format!("invalid JSON from {path}"))
        }
    }

    fn get(&self, path: &str) -> Result<Value> {
        self.request(Method::GET, path, Value::Null)
    }

    fn print_job_logs(&self, repo: &str, run_id: Option<u64>) {
        let result = (|| -> Result<()> {
            let run_id = run_id.context("no workflow run was created")?;
            let jobs = self.get(&format!("{repo}/actions/runs/{run_id}/jobs"))?;
            for job in jobs["jobs"].as_array().context("missing jobs array")? {
                let id = job["id"].as_u64().context("missing job ID")?;
                println!(
                    "{}",
                    self.text(
                        Method::GET,
                        &format!("{repo}/actions/jobs/{id}/logs"),
                        Value::Null
                    )?
                );
            }
            Ok(())
        })();
        if let Err(error) = result {
            eprintln!("Could not retrieve workflow logs: {error:#}");
        }
    }
}

struct Stack {
    directory: PathBuf,
    passed: bool,
}

impl Stack {
    /// Compose project name. Fixed so that the next run can clean up a stack
    /// leaked by an interrupted run, at the cost of one run per host at a time.
    const PROJECT: &str = "release-plz-gitea-test";
    /// Gitea requires registration tokens to be at least 32 characters long.
    const REGISTRATION_TOKEN: &str = "integration-test-runner-registration-token-not-a-secret";

    fn new() -> Result<Self> {
        let stack = Self {
            directory: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            passed: false,
        };
        // `Drop` does not run when the previous run was killed.
        stack.down().context("could not clean up a previous run")?;
        Ok(stack)
    }

    fn command(&self) -> Command {
        let mut command = Command::new("docker");
        command
            .args(["compose", "-f"])
            .arg(self.directory.join("compose.yml"))
            .args(["-p", Self::PROJECT])
            .env("GITEA_RUNNER_REGISTRATION_TOKEN", Self::REGISTRATION_TOKEN);
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

    fn admin(&self) -> Command {
        let mut command = self.command();
        command.args(["exec", "-T", "gitea", "gitea", "admin", "user"]);
        command
    }
}

impl Drop for Stack {
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
