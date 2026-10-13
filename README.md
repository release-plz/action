# release-plz-action

Action for [release-plz](https://github.com/release-plz/release-plz).

## Docs

Learn how to use this action in the [docs](https://release-plz.dev/).

## Caching

Set the action input `cache: true` to cache Cargo's registry index, crate
archives, and Git dependencies between runs (not credentials, installed tools,
or build artifacts).

Caching is disabled by default because Cargo doesn't re-verify crate archives
it finds in its cache: an entry poisoned by another workflow on your default
branch could run code in the job that holds your publishing credentials. Only
enable it if you trust every workflow (and the actions and dependencies they
run) on your default branch.

The action uses `actions/cache`, so it needs a runner that supports Node 24
actions (GitHub Actions runner 2.327.1 or later) even when caching is disabled.

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
