use crate::{run, ACTION_YML_PATH};

fn release_plz_version() -> String {
    let action_yml = std::fs::read_to_string(ACTION_YML_PATH).unwrap();
    let yml: serde_yaml::Value = serde_yaml::from_str(&action_yml).unwrap();
    yml["inputs"]["version"]["default"]
        .as_str()
        .unwrap()
        .to_string()
}

pub fn update_action_yml(release_plz_tag: &str) {
    let mut action_yml = std::fs::read_to_string(ACTION_YML_PATH).unwrap();
    let current_release_plz_version = release_plz_version();
    action_yml = action_yml.replace(&current_release_plz_version, release_plz_tag);
    // TODO update cargo-semver-checks
    std::fs::write(ACTION_YML_PATH, action_yml).unwrap();
}

pub fn create_pr(release_plz_tag: &str) {
    let commit_msg = format!("Update to {}", release_plz_tag);
    let branch = format!("update-{release_plz_tag}");
    run("git", &["checkout", "-b", &branch]);
    run("git", &["add", ACTION_YML_PATH]);
    run("git", &["commit", "-m", &commit_msg]);
    run("git", &["push", "origin", &branch]);

    let output = run(
        "gh",
        &["pr", "create", "--fill", "--repo", "release-plz/action"],
    );

    run("git", &["checkout", "-"]);

    println!("{output}");
}
