#!/bin/sh -x
../cargo/target/cargo build
env RUST_LOG=handle=debug target/handle
