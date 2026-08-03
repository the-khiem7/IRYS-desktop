#!/usr/bin/env bash
# Every gate that can run without a Windows machine or a desktop.
#
# Mirrors the frontend and rust jobs in .github/workflows/ci.yml, so a green run
# here means a green run there - with the documented exception of anything
# #[cfg(windows)], which a Linux compiler never parses.
set -uo pipefail

cd /app

failed=0
run() {
  local name="$1"
  shift
  printf '\n\033[1;36m==> %s\033[0m\n' "$name"
  if (cd "$WORKDIR" && "$@"); then
    printf '\033[1;32m    PASS: %s\033[0m\n' "$name"
  else
    printf '\033[1;31m    FAIL: %s\033[0m\n' "$name"
    failed=1
  fi
}

# node_modules and dist live in named volumes, so they start empty on a fresh
# checkout. The host's copies are deliberately not shared: node_modules holds
# Windows binaries for esbuild/rolldown that cannot execute here.
if [ ! -x node_modules/.bin/vite ]; then
  printf '\033[1;36m==> installing frontend dependencies\033[0m\n'
  npm ci --no-fund --no-audit || exit 1
fi

WORKDIR=/app
run "vue-tsc --noEmit" npx vue-tsc --noEmit
# Must succeed before the Rust gates: tauri-build errors out if frontendDist
# does not exist, so a missing dist/ would look like a Rust failure.
run "vite build"       npx vite build

WORKDIR=/app/src-tauri
# All three run even if an earlier one fails, so one round-trip surfaces every
# problem. Deliberately the opposite of CI's fail-fast, which had a formatting
# error hiding a clippy error, which in turn hid whether the tests passed.
run "cargo fmt --all --check"  cargo fmt --all --check
run "cargo clippy -D warnings" cargo clippy --all-targets --all-features -- -D warnings
run "cargo test"               cargo test --all-features

printf '\n'
if [ "$failed" -eq 0 ]; then
  printf '\033[1;32mAll local gates passed.\033[0m\n'
  printf 'Not covered here, by construction: platform/win.rs (cfg-gated out), the\n'
  printf 'Windows tray and overlay, WebView2, and the installers. Build one with\n'
  printf '`docker compose -f docker/compose.yml run --rm -T build-windows`.\n'
else
  printf '\033[1;31mOne or more gates failed.\033[0m\n'
fi

exit "$failed"
