#!/usr/bin/env bash
# Runs the Rust gates that cannot run on a Windows machine without MSVC.
#
# Mirrors the `rust` job in .github/workflows/ci.yml so a green run here means a
# green run there, with the documented exception of anything #[cfg(windows)].
set -uo pipefail

cd /app/src-tauri

failed=0
run() {
  local name="$1"
  shift
  printf '\n\033[1;36m==> %s\033[0m\n' "$name"
  if "$@"; then
    printf '\033[1;32m    PASS: %s\033[0m\n' "$name"
  else
    printf '\033[1;31m    FAIL: %s\033[0m\n' "$name"
    failed=1
  fi
}

# Every gate runs even if an earlier one fails: one round-trip should surface
# every problem, not just the first.
run "cargo fmt --all --check"  cargo fmt --all --check
run "cargo clippy -D warnings" cargo clippy --all-targets --all-features -- -D warnings
run "cargo test"               cargo test --all-features

printf '\n'
if [ "$failed" -eq 0 ]; then
  printf '\033[1;32mAll Rust gates passed.\033[0m\n'
  printf 'Still unverified here (Linux cannot reach it): platform/win.rs,\n'
  printf 'the Windows tray/overlay behaviour, and the MSI/NSIS bundles.\n'
else
  printf '\033[1;31mOne or more Rust gates failed.\033[0m\n'
fi

exit "$failed"
