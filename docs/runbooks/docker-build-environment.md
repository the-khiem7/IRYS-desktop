---
title: Docker build environment and Rancher / WSL disk runbook
description: Keep container build storage bounded and recover the Windows Rancher Desktop and WSL environment safely.
slug: runbooks/docker-build-environment
editUrl: https://github.com/the-khiem7/IRYS-desktop/edit/main/docs/runbooks/docker-build-environment.md
---

## Purpose

Keep Docker storage bounded and build tools isolated from this Windows development machine. IRYS builds Rust only through Rancher Desktop's Linux container; a future Go project will use the same Docker environment and therefore shares its storage budget.

This is an operational runbook, not application source code. Commands that delete data are marked explicitly.

## Mandatory build-environment policy

**All project builds, verification, tests, and packaging run inside Docker.** Do not install project build toolchains or SDKs directly on the Windows host. The host is only the editor/orchestrator: source code, Git, Rancher Desktop / Docker, and commands that start the project's Compose services are allowed.

In particular, do not install or invoke these project compilers on the host:

- Rust: `rustup`, `rustc`, `cargo`, Cargo targets, or Rust build caches.
- Windows cross-build dependencies: MSVC toolchain, Windows SDK, or linker packages added solely to build IRYS.
- Go: a Go SDK, module cache, or Go build cache for the future Go project.

Use the container-backed project commands instead:

```powershell
npm run verify
npm run build:windows
npm run build:windows:release
npm run shell
```

These scripts delegate to `docker compose -f docker/compose.yml`; the Rust toolchain, xwin Windows SDK/linker, Cargo registry, and target artifacts remain inside the image or Docker named volumes. A command is compliant only when the compiler and SDK execute in the container, even if it is started from a host PowerShell prompt.

For another project, apply the same rule: use its Docker/Compose build command and keep the Go SDK and build caches inside its containers or Docker volumes. Do not solve a missing compiler by installing it on Windows. Add or repair the project's container image instead.

### Exception process

There is no standing local-toolchain exception. A temporary host installation requires explicit owner approval first, a stated reason why Docker cannot do the work, a cleanup plan, and confirmation once the toolchain/SDK and its caches have been removed. Document the exception in the relevant project runbook before taking that action.

## Current policy

`%USERPROFILE%\.wslconfig` is the WSL-wide configuration file. It currently contains:

```ini
[wsl2]
defaultVhdSize=20GB
```

`defaultVhdSize` caps the virtual disk of **new** WSL distributions. It is a maximum, not space preallocated on the host: VHD files remain sparse and grow as they are used. The setting is global, so any newly created WSL distribution is affected, including Rancher Desktop's distro.

Rancher Desktop does not expose a separate Windows disk-size control. Its `experimental.virtualMachine.diskSize` setting is documented for macOS/Linux, not Windows. See [Microsoft's WSL configuration reference](https://learn.microsoft.com/windows/wsl/wsl-config) and [Rancher Desktop non-GUI settings](https://docs.rancherdesktop.io/next/references/non-gui-settings/).

### Why 20 GB

It is a reasonable ceiling for this machine while still leaving room for the IRYS Docker image, Node packages, Cargo registry, Windows SDK, Rust target artifacts, and a Go build cache. It is **not** a per-project quota: IRYS and any Go project share it.

If either project routinely cannot build because of `no space left on device`, raise the ceiling before creating the next Rancher distro or prune stale Docker data. Do not expect an already-created VHD to shrink merely because this value is reduced.

## Normal workflow

Use the project's container commands; do not install or invoke Rust/Cargo on the host:

```powershell
npm run verify
npm run build:windows
npm run build:windows:release
```

The Compose file deliberately keeps Cargo/target caches in Docker named volumes rather than the repository. This improves rebuild speed but those caches count toward the shared Rancher/WSL storage limit.

To inspect Docker consumption while Rancher is running:

```powershell
docker system df
docker builder du
```

To check host free space and the Rancher VHD:

```powershell
Get-PSDrive C
Get-Item "$env:LOCALAPPDATA\rancher-desktop\distro-data\ext4.vhdx"
```

The VHD's logical `Length` can be larger than its physical disk allocation. Host free space is the practical signal to watch.

## Reclaim space

### 1. Safe-ish Docker cache cleanup

This removes stopped containers, unused images, unused networks, and build cache. It **does not** remove Docker named volumes, but images may need to be pulled or rebuilt later.

```powershell
docker system prune -af
```

Run it only when Rancher Desktop is healthy. Keep Kubernetes disabled in Rancher unless it is actually needed; its images add meaningful storage cost.

### 2. Rancher Kubernetes image cache

When Docker is unavailable but Rancher Desktop's CLI still responds, this clears cached Kubernetes images without deleting the VM or Docker named volumes:

```powershell
& "$env:ProgramFiles\Rancher Desktop\resources\resources\win32\bin\rdctl.exe" reset --cache
```

### 3. Stop, then compact

Stop Rancher cleanly before touching its VHD:

```powershell
& "$env:ProgramFiles\Rancher Desktop\resources\resources\win32\bin\rdctl.exe" shutdown
wsl --shutdown
```

Compaction can return filesystem blocks that are already free inside the VHD; it cannot delete live Docker images or volumes. If it does not recover space, prune data first. Do not delete `ext4.vhdx` manually while Rancher/WSL is running.

### 4. Factory reset — destructive last resort

```powershell
& "$env:ProgramFiles\Rancher Desktop\resources\resources\win32\bin\rdctl.exe" reset --factory
```

This deletes the Rancher VM, Docker containers, images, Kubernetes data, and **all Docker named volumes**. It is appropriate only after explicit approval. After reset, start Rancher Desktop again and complete first-run setup; the newly created distro receives the `defaultVhdSize` cap from `.wslconfig`.

## Incident record: 2026-08-10

- C: dropped to 0.68 GB free while Rancher `distro-data\ext4.vhdx` had grown to about 20 GB; Docker then failed with I/O/SIGBUS and Hyper-V socket errors.
- `rdctl reset --cache` recovered several GB but the WSL backend remained stuck.
- An approved `rdctl reset --factory` removed the Rancher distributions and recovered roughly 28 GB of host space. Docker volumes were intentionally discarded.
- The 20 GB `.wslconfig` limit was created *after* the reset, before Rancher creates its next distro.

## Recovery checklist

1. Check `Get-PSDrive C` before starting a large Docker build.
2. Run `docker system df` and `docker system prune -af` while Docker is healthy.
3. If Rancher is unresponsive after disk exhaustion, run `rdctl shutdown` then `wsl --shutdown`; restart Rancher.
4. If it still cannot start, use `rdctl reset --cache`.
5. Escalate to `rdctl reset --factory` only with approval that all Docker data may be lost.
