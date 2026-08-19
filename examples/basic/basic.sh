#!/usr/bin/env bash

# Equivalent to a systemd unit file with
# OpenFile specified

cargo build --example basic

readonly scriptDir="$(dirname -- "$(realpath -- "$0")")"

exec systemd-run \
  --wait --pty --collect \
  --user \
  -p OpenFile="$scriptDir/basic.txt:basic:read-only" \
  "$scriptDir/../../target/debug/examples/basic"

