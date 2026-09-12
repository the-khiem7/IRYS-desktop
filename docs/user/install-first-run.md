---
title: Install and first run
description: Install IRYS on Windows or Linux and confirm the tray timer is running.
slug: user/install-first-run
editUrl: https://github.com/the-khiem7/IRYS-desktop/edit/main/docs/user/install-first-run.md
---

## Download the right package

Download the newest package from [GitHub Releases](https://github.com/the-khiem7/IRYS-desktop/releases):

- **Windows:** use the NSIS `.exe` for a per-user install, or the `.msi` when your environment prefers MSI packages.
- **Arch Linux:** install the maintained binary package with `yay -S irys-bin`.
- **Other Linux distributions:** use the `.deb` package on Debian-based distributions or the `.AppImage` elsewhere.

Windows may show a SmartScreen warning because the installer is not code-signed. Confirm that the download came from the IRYS GitHub releases page before continuing.

## Confirm the first run

1. Launch IRYS from the Start menu or application launcher.
2. Find the eye icon in the system tray. Windows may place it in the hidden-icons overflow.
3. Open **Settings** from the tray menu and confirm that the next-break countdown is moving.
4. Use **Break now** if you want to test the reminder immediately.

Closing the settings window hides it; the tray routine continues in the background. Use **Quit IRYS** from the tray menu when you want the app to stop completely.
