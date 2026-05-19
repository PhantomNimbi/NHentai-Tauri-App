#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")" && pwd)"
log_dir="$root_dir/logs"
mkdir -p "$log_dir"

usage() {
  cat <<EOF
Usage: $0 [platform] [cargo-args]
Platforms:
  linux   Build Linux desktop app
  windows Build Windows desktop app
  macos   Build macOS desktop app
  android Build Android APKs
  ios     Build iOS app

If no platform is provided, the script will detect the current host platform.
EOF
  exit 1
}

platform="auto"
if [ "$#" -gt 0 ]; then
  case "$1" in
    -h|--help|help)
      usage
      ;;
    linux|windows|macos|android|ios)
      platform="$1"
      shift
      ;;
    *)
      platform="auto"
      ;;
  esac
fi

if [ "$platform" = "auto" ]; then
  case "$(uname -s)" in
    Linux*) platform=linux ;;
    Darwin*) platform=macos ;;
    MINGW*|MSYS*|CYGWIN*) platform=windows ;;
    *) echo "Unsupported platform: $(uname -s)"; exit 1 ;;
  esac
fi

timestamp="$(date '+%Y%m%d-%H%M%S')"
log_file="$log_dir/build-$platform-$timestamp.log"

cd "$root_dir/src-tauri"

echo "Building for platform: $platform"
echo "Logging output to: $log_file"
case "$platform" in
  linux)
    if cargo tauri build --target x86_64-unknown-linux-gnu "$@" 2>&1 | tee "$log_file"; then
      echo "Build succeeded. Log: $log_file"
    else
      echo "Build failed. See log: $log_file"
      exit 1
    fi
    ;;
  windows)
    if cargo tauri build "$@" 2>&1 | tee "$log_file"; then
      echo "Build succeeded. Log: $log_file"
    else
      echo "Build failed. See log: $log_file"
      exit 1
    fi
    ;;
  macos)
    if cargo tauri build "$@" 2>&1 | tee "$log_file"; then
      echo "Build succeeded. Log: $log_file"
    else
      echo "Build failed. See log: $log_file"
      exit 1
    fi
    ;;
  android)
    if cargo tauri android build --apk -t aarch64 armv7 i686 x86_64 "$@" 2>&1 | tee "$log_file"; then
      echo "Build succeeded. Log: $log_file"
    else
      echo "Build failed. See log: $log_file"
      exit 1
    fi
    ;;
  ios)
    if cargo tauri ios build "$@" 2>&1 | tee "$log_file"; then
      echo "Build succeeded. Log: $log_file"
    else
      echo "Build failed. See log: $log_file"
      exit 1
    fi
    ;;
  *)
    usage
    ;;
 esac
