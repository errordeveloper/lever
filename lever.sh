#!/bin/bash -x
source ./cargo.sh
$CARGO build \
  && env RUST_LOG=lever=debug target/lever
