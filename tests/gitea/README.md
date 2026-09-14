# Gitea integration test

Run from the repository root with Docker (including Compose v2 or newer), Git,
and Rust installed:

```sh
cargo test --manifest-path tests/gitea/Cargo.toml --locked -- --nocapture
```

The test starts an isolated Gitea server and an Actions runner in Docker, then
pushes a Rust fixture containing the working tree's `action.yml`. Gitea runs the
real composite action, including tool installation, and checks its PR outputs
against the Gitea API. The harness also checks that the PR changes the manifest
and changelog for a patch release using `forge: gitea`.

The fixture uses release-plz's `git_only` mode, so no Cargo registry or publishing
credentials are needed. Network access is needed to download images, actions,
and tools. The runner executes jobs inside its own container, without mounting
the host Docker socket. Every run uses a separate Compose project, generated
credentials, and a dynamically assigned localhost port. Containers and volumes
are removed on success or failure; container logs are printed on failure.
