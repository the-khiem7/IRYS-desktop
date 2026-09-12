---
title: Privacy and safety
description: Learn what IRYS stores, what it can access, and how break reminders remain dismissible.
slug: user/privacy
editUrl: https://github.com/the-khiem7/IRYS-desktop/edit/main/docs/user/privacy.md
---

IRYS has no account, sign-in, telemetry, or analytics. It has no application networking code and does not send your routine or settings anywhere.

## Data on your device

Your preferences are stored in one small local JSON file. They contain routine and appearance settings, not credentials or personal content.

## Limited access

The app’s Tauri capabilities allow its windows to listen for IRYS events. File, shell, network, and dialog access are not granted to the frontend.

## A reminder must always be escapable

The full-screen reminder is transparent, frameless, and always on top, so dismissibility is a safety requirement. **Escape**, **Skip**, and **Snooze** remain available, and IRYS does not capture your keyboard. If a reminder cannot be dismissed, quit IRYS from the tray and [report the bug](https://github.com/the-khiem7/IRYS-desktop/issues).
