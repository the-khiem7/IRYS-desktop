# Toolchain image for developing Irys without a local MSVC install.
#
# Why this exists: Irys ships for Windows, but the development machine is a
# managed workstation where Visual Studio Build Tools cannot be installed. With
# no linker, nothing Rust-side runs there - not even `cargo check`, because
# tauri-build's build.rs must be linked and executed first.
#
# This image does two jobs:
#
#   1. Verify   - fmt, clippy and the 43 core tests on a Linux target. Fast,
#                 because it needs no Windows machinery at all.
#   2. Build    - cross-compile a real Windows NSIS installer via cargo-xwin,
#                 which links with lld-link against Microsoft's SDK instead of
#                 needing MSVC installed on the host.
#
# What it still cannot do: RUN the app. There is no Windows and no desktop
# session here, so the tray, the overlay and WebView2 can only be checked by
# installing on an actual machine. MSI is also out of reach, since WiX is
# Windows-only; the per-user NSIS installer is the one worth having anyway.

FROM rust:1-bookworm

# --- Tauri's Linux system dependencies -------------------------------------
# Needed to compile the crate for a Linux target, which is what makes the fast
# clippy/test loop possible. webkit2gtk is the Linux webview,
# libayatana-appindicator3 backs the tray.
#
# --- plus the Windows cross-compilation toolchain --------------------------
# clang, lld and llvm give cargo-xwin its linker and resource compiler; nsis
# provides makensis, which Tauri shells out to for the installer.
RUN apt-get update && apt-get install -y --no-install-recommends \
      libwebkit2gtk-4.1-dev \
      libgtk-3-dev \
      libayatana-appindicator3-dev \
      librsvg2-dev \
      libxdo-dev \
      libssl-dev \
      pkg-config \
      clang \
      lld \
      llvm \
      nsis \
    && rm -rf /var/lib/apt/lists/*

# Node, copied from the official image rather than installed from Debian, whose
# nodejs is far too old for Vite 8. Both images are bookworm, so glibc matches.
COPY --from=node:24-bookworm-slim /usr/local/bin/node /usr/local/bin/node
COPY --from=node:24-bookworm-slim /usr/local/lib/node_modules /usr/local/lib/node_modules
RUN ln -s /usr/local/lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
    && ln -s /usr/local/lib/node_modules/npm/bin/npx-cli.js /usr/local/bin/npx

RUN rustup component add rustfmt clippy \
    && rustup target add x86_64-pc-windows-msvc

RUN cargo install --locked cargo-xwin

# Accepts the Microsoft Visual Studio / Windows SDK licence so cargo-xwin can
# fetch the headers and import libraries it links against. This was an explicit
# decision by the project owner, not a default: the SDK is downloaded from
# Microsoft's own servicing CDN and used solely to build this application.
ENV XWIN_ACCEPT_LICENSE=1

# Keep every cache off the bind-mounted source tree. The host's
# src-tauri/target holds Windows-MSVC artefacts from a different toolchain, and
# sharing it would mean the two constantly invalidate each other.
ENV CARGO_TARGET_DIR=/build/target \
    CARGO_HOME=/build/cargo \
    XWIN_CACHE_DIR=/build/xwin \
    XDG_CACHE_HOME=/build/cache

WORKDIR /app

COPY docker/verify.sh /usr/local/bin/verify
COPY docker/build-windows.sh /usr/local/bin/build-windows
RUN chmod +x /usr/local/bin/verify /usr/local/bin/build-windows

CMD ["verify"]
