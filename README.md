# OxiSport

OxiSport is an async-first Rust framework for interoperable sport and fitness service integrations.

The goal is not to make every service look identical.

The goal is to provide:

- shared domain types where services genuinely overlap;
- capability-based provider abstractions;
- provider-specific escape hatches;
- generated raw API clients where practical;
- Tokio-native asynchronous I/O;
- streaming-friendly handling of routes, activity files and media;
- infrastructure that makes new provider integrations straightforward to contribute.

## Status

Early development. APIs are not stable yet.

The initial architecture is being built around services such as Strava, Intervals.icu and Garmin Connect.

## Architecture

    remote API
        |
        v
    provider raw client
        |
        v
    provider adapter
        |
        v
    OxiSport normalized model

For example, both Garmin and Strava may expose activities:

    GarminActivity ---\
                       +--> Activity
    StravaActivity ---/

They remain different providers with different capabilities.

OxiSport does not require Strava to emulate Garmin or Garmin to emulate Strava.

## Capabilities

Providers implement only the capabilities they support.

Examples include:

- ActivitySource
- ActivitySink
- RouteSource
- RouteSink
- WorkoutSource
- WorkoutSink
- AthleteSource
- GearSource
- DeviceSource
- DeviceCourseSink

There is intentionally no giant `Provider` interface.

## Async by default

OxiSport is Tokio-native.

All network-facing APIs are asynchronous.

Large payloads such as GPX, FIT, TCX and media are designed around streaming rather than mandatory whole-file buffering.

## Raw APIs

Provider raw clients expose service-specific functionality.

Code generation may be used for the mechanical HTTP layer:

    API schema
        |
        v
    generated raw client
        |
        v
    handwritten adapter
        |
        v
    stable domain types

Code generation never defines the shared OxiSport domain model.

## Repository

    crates/
      oxisport-core
      oxisport-runtime
      oxisport-files
      oxisport-codegen
      providers/
        ...

## Adding a provider

A new provider generally consists of:

1. a raw client;
2. authentication;
3. conversions to shared domain types;
4. implementations of the capabilities the provider actually supports;
5. tests.

Adding a provider should not require modifying unrelated providers.

## Non-goals

OxiSport is not:

- an MCP server;
- an AI framework;
- a blocking HTTP SDK;
- an attempt to expose only the lowest common denominator of every fitness service.

MCP support can be built externally using crates such as `rmcp`.

## License

Apache-2.0
