#!/bin/bash

# MacOS Prereqs:
# https://dev.to/h_ajsf/cross-compiling-rust-for-raspberry-pi-4iai
# brew install arm-linux-gnueabihf-binutils
# rustup target add armv7-unknown-linux-musleabihf

# .cargo/config
# [build]
# target = "armv7-unknown-linux-musleabihf"
#
# [target.armv7-unknown-linux-musleabihf]
# linker = "arm-linux-gnueabihf-ld"

# Fedora Prereqs:
# sudo dnf install -y gcc-arm-linux-gnu
# sudo dnf install -y binutils-arm-linux-gnu
# rustup target add armv7-unknown-linux-musleabihf

set -o errexit

readonly TARGET_HOST=r2d2
readonly TARGET=armv7-unknown-linux-musleabihf
readonly TARGET_PATH=/home/pi/bin/rpi
readonly SOURCE_PATH=./target/$TARGET/release/rpi-simon-says

cargo build --release
scp ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
ssh -t ${TARGET_HOST} ${TARGET_PATH}