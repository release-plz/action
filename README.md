# release-plz-action

Action for [release-plz](https://github.com/release-plz/release-plz).

## Docs

Learn how to use this action in the [docs](https://release-plz.dev/).

## Gitea Actions

Set `forge: gitea` and provide a Gitea token through the `GITHUB_TOKEN`
environment variable. The action uses the runner's `GITHUB_SERVER_URL` and
`GITHUB_REPOSITORY` to locate the repository.

Automatic git user configuration from the token is only supported for GitHub.
For Gitea, configure the release author's name and email before running the
action, using the identity you want on release commits:

```yaml
- uses: https://github.com/actions/checkout@v4
  with:
    fetch-depth: 0
    token: ${{ secrets.RELEASE_TOKEN }}
- name: Configure release author
  run: |
    git config --global user.name "Release Bot"
    git config --global user.email "release-bot@example.com"
- uses: https://github.com/release-plz/action@v0.5
  env:
    GITHUB_TOKEN: ${{ secrets.RELEASE_TOKEN }}
    CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
  with:
    forge: gitea
```

Replace the example identity with your release account's name and email, and
set `RELEASE_TOKEN` to a Gitea token with permission to push commits and create
pull requests and releases. Passing it to checkout also authenticates Git pushes.
The runner needs the usual release-plz prerequisites
(including Rust, Git, Bash and jq); `gh` is not required for Gitea.

The deprecated `backend` input remains supported when `forge` is not specified.
An explicit `forge` takes precedence over `backend`.

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
