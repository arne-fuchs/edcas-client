#!/bin/bash
# shellcheck disable=SC2164
cd "$(dirname "$0")"
#cargo build
# Sleep to wait until elite starts
#sleep 60
./target/debug/EliteRustClient
