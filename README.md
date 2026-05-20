# ModelParams Rust Workspace

This directory contains the Rust workspace for `modelparams`.

## Crates

- `modelparams`: main library, examples, and integration tests.
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

Examples live in `modelparams/examples`.

Run one example with:

```powershell
cargo run -p modelparams --example beps_model
```

Other available examples:

```powershell
cargo run -p modelparams --example pml_model
cargo run -p modelparams --example soil_model
cargo run -p modelparams --example van_genuchten
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
