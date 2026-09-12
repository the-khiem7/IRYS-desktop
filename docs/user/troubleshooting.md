---
title: Troubleshooting
description: Recover when the tray icon, countdown, reminder, settings, or startup behaviour is not working.
slug: user/troubleshooting
editUrl: https://github.com/the-khiem7/IRYS-desktop/edit/main/docs/user/troubleshooting.md
---

## I cannot find IRYS

Check the system tray’s hidden-icons area. If the eye icon is absent, launch IRYS again from the Start menu or application launcher. Opening it twice should focus or restore the existing app rather than create two independent schedules.

## The reminder did not appear

Open Settings and check whether the routine is paused. On Windows, also check whether away reset or full-screen deferral applies. Use **Break now** to distinguish a reminder-display problem from a scheduling delay.

## Settings do not save

Keep the settings window open, correct any field-level validation message, and retry. If IRYS reports a persistence or connection error, restart the app; your unsaved values may need to be entered again after a full restart.

## A reminder will not close

Try **Escape**, **Skip**, or **Snooze**. If those fail, use the tray menu to quit IRYS. This is a safety bug—please [open an issue](https://github.com/the-khiem7/IRYS-desktop/issues) with your operating system, IRYS version, reminder style, and the steps that led to it.

## Linux behaviour differs

Linux packages are published, but away detection and full-screen detection are Windows-only today. Their toggles do not suppress or defer reminders on Linux.
