use std::process::Command;

use clap::Parser;

use crate::args::CliArgs;

mod args;
mod pr;
mod release;

const ACTION_YML_PATH: &str = "../action.yml";
const RELEASE_PLZ_REPO: &str = "release-plz/release-plz";

fn git_pull() {
    let repo = git_cmd::Repo::new(".").unwrap();
    let current_branch = repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]).unwrap();
    if current_branch != "main" {
        panic!("You are not on the main branch");
    }
    let status = repo.git(&["status", "--porcelain"]).unwrap();
    if !status.is_empty() {
        panic!("You have uncommitted changes");
    }
    repo.git(&["pull"]).unwrap();
}

pub fn latest_release(repo: &str) -> String {
    let last_tag = Command::new("gh")
        .args(["release", "list", "--limit", "1", "--repo", repo])
        .output()
        .expect("failed to execute process");
    let last_tag = String::from_utf8(last_tag.stdout).unwrap();
    let last_tag = last_tag.trim();
    println!("latest tag: {repo}: `{}`", last_tag);
    last_tag
        .split_whitespace()
        .next()
        .unwrap()
        .trim_start_matches("release-plz-v")
        .to_string()
}

fn verify_release_plz_tag(release_plz_tag: &str) {
    if !release_plz_tag.starts_with("0.3") {
        panic!("latest tag `{release_plz_tag}` is not a release-plz tag. Probably you just need to wait until the release is published");
    }
    let release_plz_tag = format!("release-plz-v{}", release_plz_tag);
    // run: gh release view {tag} --repo release-plz/release-plz --json assets --jq '.assets | length'
    let output = Command::new("gh")
        .args([
            "release",
            "view",
            &release_plz_tag,
            "--repo",
            RELEASE_PLZ_REPO,
            "--json",
            "assets",
            "--jq",
            ".assets | length",
        ])
        .output()
        .unwrap();
    let out = String::from_utf8(output.stdout).unwrap();
    let expected_assets = "10";
    let out = out.trim();
    if out != expected_assets {
        println!("release-plz tag `{release_plz_tag}` does not have {expected_assets} assets, it has {out} instead. Either:\n- you need to wait until the binaries are published\n- one binary failed to compile");
        println!(">>> Press enter to continue or Ctrl+C to abort <<<");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
    }
}

fn main() {
    let args = CliArgs::parse();
    match args.command {
        args::Command::Pr => {
            let release_plz_tag = latest_release(RELEASE_PLZ_REPO);
            verify_release_plz_tag(&release_plz_tag);
            pr::update_action_yml(&release_plz_tag);
            pr::create_pr(&release_plz_tag);
        }
        args::Command::Release => {
            git_pull();
            release::create_release();
        }
    }
}
