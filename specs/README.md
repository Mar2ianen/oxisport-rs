# Specs

Machine-readable API specifications consumed by `oxisport-codegen`.

## Format

The current format is a small custom YAML specification (see
`examples/example.yaml`). It is intentionally minimal: it exists to prove the
codegen pipeline and will be extended (or replaced by an OpenAPI frontend)
as providers are implemented.

Supported field types:

| YAML type   | Rust type        |
|-------------|------------------|
| `string`    | `String`         |
| `bool`      | `bool`           |
| `i16/i32/i64` (or `int16/32/64`) | `i16/i32/i64` |
| `u16/u32/u64` (or `uint16/32/64`) | `u16/u32/u64` |
| `f32/f64`   | `f32/f64`        |
| `json`      | `serde_json::Value` |
| `list<T>`   | `Vec<T>`         |
| model name  | the generated struct |

Endpoint methods: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`.

Path parameters are declared as `path_params` and referenced in `path` with
`{name}` placeholders.

## Pipeline

    specs/*.yaml
        |
        v
    oxisport-codegen (YAML -> IR -> Rust source)
        |
        v
    crates/providers/*/oxisport-*-raw/src/generated.rs

Regenerate with:

    cargo xtask codegen

Verify committed generated sources are up to date with:

    cargo xtask check-generated

Generated sources are committed to git. Codegen never runs in `build.rs`.

## Plan

- add an OpenAPI frontend mapping into the same internal IR;
- support query parameters, headers and shared request/response models;
- provider-specific auth and pagination conventions stay in handwritten
  adapter code, not in generated clients.

Do not vendor third-party API descriptions until their redistribution and
license terms are understood.
