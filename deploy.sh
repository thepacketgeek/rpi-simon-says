#!/bin/bash

# Prereqs:
# https://dev.to/h_ajsf/cross-compiling-rust-for-raspberry-pi-4iai
# brew install arm-linux-gnueabihf-binutils
# rustup target add armv7-unknown-linux-musleabihf

# .cargo/config
# [build]
# target = "armv7-unknown-linux-musleabihf"
#
# [target.armv7-unknown-linux-musleabihf]
# linker = "arm-linux-gnueabihf-ld"

set -o errexit

readonly TARGET_HOST=r2d2
readonly TARGET_PATH=/home/pi/bin/rpi
readonly SOURCE_PATH=./target/armv7-unknown-linux-musleabihf/release/rpi

cargo build --release -v
scp ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
ssh -t ${TARGET_HOST} ${TARGET_PATH}