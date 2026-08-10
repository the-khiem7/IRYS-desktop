#!/usr/bin/env bash
# Cross-compiles a Windows NSIS installer from Linux via cargo-xwin.
#
#   build-windows          fast, dev profile   <- use this while iterating
#   build-windows release  optimised profile   <- what actually ships
#
# Produces the same per-user installer the release CI does, without needing MSVC
# on the host or a push to GitHub. The result is copied to ./out on the host,
# because the build directory lives in a Docker volume the host cannot see.
set -euo pipefail

cd /app

mode="${1:-debug}"
target_dir="${CARGO_TARGET_DIR}/x86_64-pc-windows-msvc"

if [ "$mode" = "debug" ]; then
  # The release profile in Cargo.toml sets `lto = true` and `codegen-units = 1`
  # to keep the shipped binary small. Both are hostile to build speed: one
  # disables parallel codegen inside every crate, the other adds a largely
  # single-threaded whole-program pass at the end. None of that matters when the
  # question is "does Escape dismiss the overlay", so the local loop skips it.
  #
  # A debug build also keeps its console window, because main.rs only sets
  # windows_subsystem = "windows" under not(debug_assertions) - so eprintln!
  # diagnostics become visible, which is what you want on a first run.
  profile_args=(--debug)
  bundle_dir="${target_dir}/debug/bundle/nsis"
  printf '\033[1;36m==> building DEBUG (fast; bigger binary, GUI subsystem)\033[0m\n'
elif [ "$mode" = "release" ]; then
  profile_args=()
  bundle_dir="${target_dir}/release/bundle/nsis"
  printf '\033[1;36m==> building RELEASE (optimised; slow, lto + codegen-units=1)\033[0m\n'
else
  printf 'unknown mode: %s (expected "debug" or "release")\n' "$mode" >&2
  exit 2
fi

# node_modules lives in a named volume, so it starts empty on a fresh checkout.
# The host's own copy is deliberately not shared: it holds Windows binaries for
# esbuild/rolldown that cannot execute here.
if [ ! -x node_modules/.bin/vite ]; then
  printf '\033[1;36m==> installing frontend dependencies\033[0m\n'
  npm ci --no-fund --no-audit
fi

# --bundles nsis: MSI needs WiX, which is Windows-only. NSIS is also the build
# worth having, since it installs per-user and needs no admin.
npx tauri build \
  --runner cargo-xwin \
  --target x86_64-pc-windows-msvc \
  --bundles nsis \
  "${profile_args[@]}"

if ! compgen -G "${bundle_dir}/*.exe" > /dev/null; then
  printf '\n\033[1;31mBuild reported success but produced no installer in %s\033[0m\n' "$bundle_dir"
  ls -la "$bundle_dir" 2>/dev/null || printf '(directory does not exist)\n'
  exit 1
fi

mkdir -p /app/out
cp "${bundle_dir}"/*.exe /app/out/

printf '\n\033[1;32m==> installer ready on the host, in ./out\033[0m\n'
ls -lh /app/out/*.exe | sed 's|/app/|./|'
printf '\nUnsigned, so SmartScreen warns once: More info -> Run anyway.\n'
printf 'This build has never been executed. Work the manual checks in\n'
printf 'docs/baseline/irys-desktop-app/irys-desktop-app.useguide.md - escapability first,\n'
printf 'and have Ctrl+Shift+Esc ready before the first break fires.\n'
