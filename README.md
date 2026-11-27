# release-plz-action

Action for [release-plz](https://github.com/release-plz/release-plz).

## Docs

Learn how to use this action in the [docs](https://release-plz.dev/).

## GitHub Enterprise Server

The runner downloads the action's `actions/cache` step while setting up the
job, even when caching is disabled. So on GitHub Enterprise Server, the action
needs:

- Runners that support Node 24 actions (version 2.327.1 or later). GHES 3.19
  and later require newer runners than that.
- The `actions/cache` version pinned in [`action.yml`](action.yml). Your
  instance's bundled copy may not include it: update it with
  [`actions-sync`](https://github.com/actions/actions-sync), or
  [use `actions/cache` from GitHub.com](https://docs.github.com/en/enterprise-server@latest/admin/managing-github-actions-for-your-enterprise/managing-access-to-actions-from-githubcom/using-the-latest-version-of-the-official-bundled-actions)
  through GitHub Connect.

## Caching

Set the `cache` input to `true` to cache Cargo's registry index, crate
archives, and Git dependencies between runs (not Cargo's credentials file,
installed tools, or build artifacts).

Caching is disabled by default because Cargo doesn't re-verify crate archives
it finds in its cache: an entry poisoned by another workflow on your default
branch (or the branch you release from) could run code in the release-plz job,
with access to its GitHub and registry tokens. Only enable it if you trust
every workflow (and the actions and dependencies they run) on those branches.

<br>

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version 2.0</a>
or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
