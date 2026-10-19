use next_version::NextVersion;

use crate::{latest_release, run};

fn next_tag() -> String {
    let mut action_tag = latest_release("release-plz/action");
    println!("latest tag: {:?}", action_tag);
    // remove `v`
    action_tag.remove(0);
    let semver: semver::Version = action_tag.parse().unwrap();
    let selection = {
        let next_patch = semver.increment_patch();
        let next_minor = semver.increment_minor();
        let next_major = semver.increment_major();
        let next = inquire::Select::new("Version:", vec![next_patch, next_minor, next_major])
            .prompt()
            .unwrap();
        format!("v{next}")
    };
    println!("next tag: {:?}", selection);
    selection
}

pub fn create_release() {
    let tag = &next_tag();
    run("git", &["tag", "-a", tag, "-m", tag]);
    run("git", &["push", "origin", tag]);
    run("gh", &["release", "create", tag, "--generate-notes"]);
}
