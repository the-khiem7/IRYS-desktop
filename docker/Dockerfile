# Toolchain image for verifying Irys without a local MSVC install.
#
# Why this exists: Irys ships for Windows, but the development machine is a
# managed workstation where Visual Studio Build Tools cannot be installed. With
# no linker, nothing Rust-side runs there - not even `cargo check`, because
# tauri-build's build.rs must be linked and executed first.
#
# A Linux toolchain sidesteps that entirely. It compiles and runs everything
# that is not `#[cfg(windows)]`, which is most of the codebase and all 43 core
# tests. What it CANNOT cover, by construction:
#
#   - src-tauri/src/platform/win.rs  (cfg-gated out; never even parsed here)
#   - the Windows tray, transparent always-on-top window, HKCU autostart
#   - the MSI and NSIS bundles
#
# Those stay the Windows CI job's responsibility. This image is the fast local
# loop, not a replacement for that gate.

# --- frontend: build dist/ so tauri-build can find frontendDist -------------
FROM node:24-bookworm-slim AS web

WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci --no-fund --no-audit

COPY tsconfig.json vite.config.ts index.html break.html ./
COPY src-vue ./src-vue
RUN npx vue-tsc --noEmit && npx vite build

# --- rust toolchain ---------------------------------------------------------
FROM rust:1-bookworm

# Tauri's Linux system dependencies. webkit2gtk is the Linux webview,
# libayatana-appindicator3 backs the tray, and libxdo is pulled in by Tauri's
# input handling. None of this ships in the Windows build; it is only here so
# the crate can compile for a Linux target.
RUN apt-get update && apt-get install -y --no-install-recommends \
      libwebkit2gtk-4.1-dev \
      libgtk-3-dev \
      libayatana-appindicator3-dev \
      librsvg2-dev \
      libxdo-dev \
      libssl-dev \
      pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add rustfmt clippy

# Keep build artefacts off the bind-mounted source tree. The host's
# src-tauri/target holds Windows-MSVC artefacts, and letting Linux builds share
# it would mean the two constantly invalidate each other.
ENV CARGO_TARGET_DIR=/build/target
ENV CARGO_HOME=/build/cargo

WORKDIR /app

# The frontend bundle, so tauri-build's frontendDist check is satisfied even
# when only the Rust is being verified.
COPY --from=web /app/dist /app/dist

COPY docker/verify.sh /usr/local/bin/verify
RUN chmod +x /usr/local/bin/verify

CMD ["verify"]
