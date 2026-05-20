#! /bin/env bash
args = "--release -p modelparams-examples"

cargo run $args --example beps_model
cargo run $args --example pml_model
cargo run $args --example soil_model
cargo run $args --example van_genuchten

# cargo run --example van_genuchten
