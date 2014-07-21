#!/bin/sh -x
../cargo/target/cargo build
env RUST_LOG=lever=debug target/lever
