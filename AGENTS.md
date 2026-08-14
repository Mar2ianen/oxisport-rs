# AGENTS.md

## Project

OxiSport is an Apache-2.0 Rust framework for interoperable sport and fitness service integrations.

The repository is a Cargo workspace containing shared domain/runtime crates and provider-specific crates.

## Architectural rules

1. `oxisport-core` must never depend on a provider crate.
2. Provider wire models must not leak into `oxisport-core`.
3. Shared abstractions are capability-based; do not create a giant provider interface.
4. A provider implements only capabilities it actually supports.
5. Provider-specific functionality must remain reachable through provider/raw APIs.
6. Raw code generation ends at the provider adapter boundary.
7. Generated raw models must not become the public normalized domain model.
8. New providers should normally not require changes to existing providers or `oxisport-core`.
9. Network-facing APIs are async-only and Tokio-native.
10. Never create a Tokio runtime inside a library crate.
11. Large payload APIs must be streaming-friendly; do not require whole-file buffering.
12. MCP/LLM integration is out of scope for this repository.

## Dependency direction

Allowed:

provider -> provider-raw
provider -> oxisport-core
provider -> oxisport-runtime
provider-raw -> oxisport-runtime
workflow -> providers
workflow -> oxisport-core

Forbidden:

oxisport-core -> provider
oxisport-runtime -> provider
provider A -> provider B unless it is explicitly a higher-level workflow crate

## Raw vs normalized types

Raw types mirror the provider API.

Normalized types represent durable domain concepts shared meaningfully across providers.

Do not add a field to the normalized model simply because one provider exposes it.

When uncertain, keep data provider-specific.

## Async

Use Tokio-compatible async I/O.

No blocking reqwest APIs.

Do not use blocking sleeps in async code.

Prefer streaming bodies for files/media.

Avoid reading an entire remote file into memory unless the API operation genuinely requires it.

## Code generation

Generated Rust source is committed to git.

Codegen must not run in `build.rs`.

Generated files are not edited manually.

Code generation targets only raw provider APIs.

Handwritten adapters convert raw models into normalized models.

## Changes

Before adding a new abstraction:
- demonstrate at least one concrete provider need;
- consider whether it belongs in a provider crate instead;
- avoid speculative core expansion.

Before adding a dependency:
- justify why an existing workspace dependency or std/Tokio primitive is insufficient;
- prefer maintained, focused crates;
- avoid unnecessary runtime/framework dependencies.

## Testing

Run before considering work complete:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace --all-features

Generated sources must also pass:

    cargo xtask check-generated

Mapping code should have unit tests using representative provider payloads.

Network tests must not require credentials in the normal test suite.

## Commits

Keep generated-code updates and their source spec changes in the same change.

Do not commit credentials, API tokens, cookies, sessions, or real private activity data.
