#!/bin/bash

# This script is set up to be run only from linux or apple silicon mac hosts; windows and intel mac
# are cross compiled from the respective hosts.
#
# USAGE: ./build_release_package.sh <platform>
#
# It takes one parameter to indicate which platform to build a package for: mac, windows or linux
#
# Cross compiling to x86_64 windows is support by the 'cross' utility
#       cargo install cross
#       sudo apt install podman
#       cross build --release --target x86_64-pc-windows-gnu
#
# Cross compiling to x86_64 mac is done just with the toolchain
#       rustup target add x86_64-apple-darwin
#       cargo build --release --target=x86_64-apple-darwin
#

# Set the release tag
RELEASE_TAG="v1.0.0"

# Set the output directory for archives
OUTPUT_DIR="target"

# Additional files to include
SUPPORT_FILES="README.md config.toml LICENSE"

# Set the name of the output archives
LINUX_ARCHIVE_NAME="kontour-linux_x86_64_$RELEASE_TAG.tar.gz"
WINDOWS_ARCHIVE_NAME="kontour-windows_x86_64_$RELEASE_TAG.zip"
MAC_ARM_ARCHIVE_NAME="kontour-mac_aarch64_$RELEASE_TAG.zip"
MAC_INTEL_ARCHIVE_NAME="kontour-mac_x86_64_$RELEASE_TAG.zip"

# Check if platform argument is provided
if [ $# -ne 1 ]; then
  echo "Please provide a platform argument: 'linux', 'windows' or 'mac'"
  exit 1
fi

# Read the platform argument
PLATFORM="$1"


if [ "$PLATFORM" == "linux" ]; then
    cargo build --release
    rm -f "$OUTPUT_DIR/$LINUX_ARCHIVE_NAME" 
    tar -czf "$OUTPUT_DIR/$LINUX_ARCHIVE_NAME" $SUPPORT_FILES -C target/release kontour 
elif [ "$PLATFORM" == "windows" ]; then
    cross build --release --target x86_64-pc-windows-gnu
    rm -f "$OUTPUT_DIR/$WINDOWS_ARCHIVE_NAME"
    zip -rj "$OUTPUT_DIR/$WINDOWS_ARCHIVE_NAME" "target/x86_64-pc-windows-gnu/release/kontour.exe" $SUPPORT_FILES
elif [ "$PLATFORM" == "mac" ]; then
    cargo build --release
    cargo build --release --target=x86_64-apple-darwin
    rm -f "$MAC_ARM_ARCHIVE_NAME" "$MAC_INTEL_ARCHIVE_NAME"
    tar -czf "$OUTPUT_DIR/$MAC_ARM_ARCHIVE_NAME" $SUPPORT_FILES -C target/release kontour 
    tar -czf "$OUTPUT_DIR/$MAC_INTEL_ARCHIVE_NAME" $SUPPORT_FILES -C target/x86_64-apple-darwin/release kontour 
else
  echo "Invalid platform argument: '$PLATFORM'"
  echo "Please provide a platform argument: 'linux', 'windows' or 'mac'"
  exit 1
fi




echo "Archives created successfully."
