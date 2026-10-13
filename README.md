# release-plz-action

Action for [release-plz](https://github.com/release-plz/release-plz).

## Docs

Learn how to use this action in the [docs](https://release-plz.dev/).

## Caching

The action caches Cargo's registry index, crate archives, and Git dependencies
(not credentials, installed tools, or build artifacts). Set the action input
`cache: false` to disable it, for example if your workflow already caches
`~/.cargo`.

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
