Apiary
=============================

[<img alt="badge-github"    src="https://img.shields.io/badge/github.com-HyeonuPark/apiary-green">](https://github.com/HyeonuPark/apiary)
[<img alt="badge-crates.io" src="https://img.shields.io/crates/v/apiary.svg">](https://crates.io/crates/apiary)
[<img alt="badge-docs.rs"   src="https://docs.rs/apiary/badge.svg">](https://docs.rs/apiary)
[<img alt="badge-ci"        src="https://img.shields.io/github/workflow/status/HyeonuPark/apiary/CI/main">](https://github.com/HyeonuPark/apiary/actions?query=branch%3Amain)

HTTP API interface as a trait.

# Goals

- Generate HTTP routing code from the custom trait definition.
- Handles HTTP body as JSON or the plain text.
- Leverages the Tower Service as a middleware.

# Non-goals

- Non-textual parameters.
- Streaming body.

# Future goals

- OpenAPI spec generation.
- Generate code from the OpenAPI spec.

# License

Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
