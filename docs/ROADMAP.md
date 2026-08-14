# Roadmap

## M0 — Framework bootstrap

- [x] Cargo workspace
- [x] Apache-2.0 licensing
- [x] core domain types
- [x] capability traits
- [x] async runtime/transport
- [x] streaming upload primitive
- [x] codegen IR
- [x] custom YAML spec frontend
- [x] generated demo raw client
- [x] mock provider adapter
- [x] xtask codegen
- [x] generated-code verification
- [x] CI
- [x] architecture documentation

Exit criterion:

A mock provider can generate a raw client from a schema, perform an async request through the common runtime, map the raw response into `oxisport-core`, and compile/test cleanly.

## M1 — First real provider

- [ ] select Strava or Intervals.icu
- [ ] document API/schema source
- [ ] auth/configuration
- [ ] ActivitySource vertical slice
- [ ] raw -> normalized mappings
- [ ] fixtures
- [ ] mapping tests
- [ ] example application

Exit criterion:

A real service can return normalized activities through a provider implementation without leaking raw provider models into application code.

## M2 — Second provider and intersection validation

- [ ] implement another provider
- [ ] map its Activity model
- [ ] compare normalized semantics
- [ ] revise core only where both implementations justify it
- [ ] add pagination stream abstraction

Exit criterion:

Two unrelated services implement ActivitySource using the same normalized Activity model without provider-specific hacks in core.

## M3 — Routes and streaming

- [ ] RouteSource
- [ ] RouteSink
- [ ] GPX transfer
- [ ] streaming downloads
- [ ] streaming uploads
- [ ] retryable/spooled body support
- [ ] progress reporting

## M4 — Workouts/devices

- [ ] WorkoutSource
- [ ] WorkoutSink
- [ ] DeviceSource
- [ ] DeviceCourseSink
- [ ] Garmin integration research/implementation

## M5 — Cross-provider workflows

- [ ] lossless/native transfer selection
- [ ] capability negotiation
- [ ] activity copy
- [ ] route copy
- [ ] workout copy
- [ ] conflict/idempotency model

## Later

Candidate providers:

- RideWithGPS
- Komoot
- TrainingPeaks
- Polar
- COROS
- Suunto
- Wahoo
- Runalyze

Provider additions should primarily arrive as isolated provider crates and should not cause continuous expansion of the core model.
