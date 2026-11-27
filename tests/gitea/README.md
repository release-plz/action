# Gitea integration test

Requires Docker with Compose v2 or newer, Git, Rust, and network access to
download images, actions, and tools. Run from the repository root:

```sh
cargo test --manifest-path tests/gitea/Cargo.toml --locked -- --nocapture
```

The test starts Gitea and an Actions runner in Docker, pushes a Rust fixture
containing the working tree's `action.yml`, and lets the fixture workflow check
out the repository with `actions/checkout` before running the real composite
action with `forge: gitea`. The workflow asserts the action's `pr`, `prs`, and
`prs_created` outputs describe pull request #1 for `release-plz-gitea-test`
0.1.1, and the harness asserts Gitea has exactly one open pull request, from a
`release-plz-` branch onto `main`, changing `Cargo.toml` and `CHANGELOG.md`.

The Compose project `release-plz-gitea-test` is torn down when the test exits,
and a run interrupted with Ctrl-C is cleaned up by the next run. To remove it by
hand, run `docker compose -f tests/gitea/compose.yml down --volumes`.
