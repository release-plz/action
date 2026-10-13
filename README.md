# release-plz-action

Action for [release-plz](https://github.com/release-plz/release-plz).

## Docs

Learn how to use this action in the [docs](https://release-plz.dev/).

## Caching

The action caches Cargo's registry index, downloaded crate archives, and Git
dependency repositories by default. It uses `CARGO_HOME` when set, otherwise
`$HOME/.cargo`. Credentials, installed tools, extracted sources, and build artifacts
are not cached, so this cache is unaffected by Rust 1.100's build-directory layout
changes.

The cache key includes the runner OS and architecture and the repository's
`Cargo.toml` and `Cargo.lock` files. When those files change, the action restores
the most recent download cache for the same OS and architecture and saves a new
entry after a successful job. Cargo still checks the registry for updates.

Set the action input `cache: false` to disable caching, for example if your
workflow already manages Cargo's download cache.

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
