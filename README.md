# ModelParams Rust Workspace

[![CI](https://github.com/jl-pkgs/ModelParams.rs/actions/workflows/CI.yml/badge.svg)](https://github.com/jl-pkgs/ModelParams.rs/actions/workflows/CI.yml)
[![Codecov](https://codecov.io/gh/jl-pkgs/ModelParams.rs/branch/master/graph/badge.svg)](https://app.codecov.io/gh/jl-pkgs/ModelParams.rs/tree/master)

This directory contains the Rust workspace for `modelparams`.

## Crates

- `modelparams`: main library and integration tests.
- `modelparams-examples`: top-level example package.
- `modelparams-derive`: procedural macros used by `modelparams`.

## Build

Run commands from this directory:

```powershell
cd rust
cargo build
```

For an optimized release build:

```powershell
cargo build --release
```

## Run Examples

Examples live in `examples`.

Run one example with:

```powershell
cargo run -p modelparams-examples --example beps_model
```

Other available examples:

```powershell
cargo run -p modelparams-examples --example pml_model
cargo run -p modelparams-examples --example soil_model
cargo run -p modelparams-examples --example van_genuchten
```

## Run Tests

Run all workspace tests:

```powershell
cargo test
```

Run tests for the main crate only:

```powershell
cargo test -p modelparams
```

Run a specific integration test:

```powershell
cargo test -p modelparams --test par_map
cargo test -p modelparams --test sceua_benchmarks
```

## Lints

Workspace lint settings are defined in `Cargo.toml`. In particular,
`unused_imports` is allowed at the workspace level so unused imports do not
produce warnings or fail strict warning-as-error builds.
