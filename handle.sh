#!/bin/bash -x
source ./cargo.sh
$CARGO build \
  && env RUST_LOG=handle=debug target/handle
