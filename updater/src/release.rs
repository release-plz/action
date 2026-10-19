use next_version::NextVersion;

use crate::{latest_release, run};

fn select_next_version() -> semver::Version {
    let mut action_tag = latest_release("release-plz/action");
    println!("latest tag: {:?}", action_tag);
    // remove `v`
    action_tag.remove(0);
    let semver: semver::Version = action_tag.parse().unwrap();
    let next_patch = semver.increment_patch();
    let next_minor = semver.increment_minor();
    let next_major = semver.increment_major();
    let next = inquire::Select::new("Version:", vec![next_patch, next_minor, next_major])
        .prompt()
        .unwrap();
    println!("next tag: \"v{next}\"");
    next
}

pub fn create_release() {
    let version = select_next_version();
    let tag = &format!("v{version}");
    run("git", &["tag", "-a", tag, "-m", tag]);
    run("git", &["push", "origin", tag]);
    run("gh", &["release", "create", tag, "--generate-notes"]);
    move_minor_tag(&version, tag);
}

/// Point the `vX.Y` tag to the release just created,
/// so that users of `release-plz/action@vX.Y` get the latest patch release.
fn move_minor_tag(version: &semver::Version, tag: &str) {
    let minor_tag = &format!("v{}.{}", version.major, version.minor);
    let message = format!("Release {tag}");
    run("git", &["tag", "--force", "-a", minor_tag, "-m", &message]);
    run("git", &["push", "--force", "origin", minor_tag]);
}
