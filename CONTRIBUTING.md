# Contributing

Thanks for considering a contribution to OxiSport.

## License

This project is Apache-2.0 licensed. By contributing you agree that your
contribution is submitted under the terms of the Apache-2.0 license.

## Adding a provider

1. Add a spec under `specs/<provider>` if code generation applies.
2. Add `crates/providers/<provider>/oxisport-<provider>-raw`.
3. Add `crates/providers/<provider>/oxisport-<provider>`.
4. Implement authentication.
5. Implement only the capability traits the provider actually supports.
6. Add mapping/conversion tests with representative (non-private) fixtures.
7. Add integration tests where the testing infrastructure permits.
8. Open a pull request.

A new provider should not require modifying unrelated providers or
`oxisport-core`. If you believe the normalized model needs a change, start a
discussion first: core changes must be justified by more than one provider.

## API and spec documentation

Document the API source (official docs, schema files, versions) for every
provider. Do not invent endpoints. If a provider relies on undocumented or
reverse-engineered behavior, say so clearly in the provider crate and in its
documentation.

## Generated code policy

Generated raw clients are committed to git and are never edited by hand.

Generated-code updates and their source spec changes belong in the same
change.

Code generation must not run in `build.rs`; downstream users never need the
code generator to build a provider crate.

## Required checks

Run before opening a PR:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace --all-features
    cargo doc --workspace --all-features --no-deps
    cargo xtask check-generated

## Privacy

Do not commit credentials, API tokens, cookies, sessions, or real private
activity data. Fixtures must be synthetic or clearly non-sensitive.
