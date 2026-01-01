# 👻 Phantimer

![Rust](https://img.shields.io/badge/Made_with-Rust-orange?style=for-the-badge&logo=rust)
![Hyprland](https://img.shields.io/badge/Hyprland-Native-00a4bd?style=for-the-badge&logo=archlinux)
![AUR](https://img.shields.io/aur/version/phantimer?style=for-the-badge&color=blue&label=AUR)
![License](https://img.shields.io/github/license/aman-sanin/phantimer?style=for-the-badge)

**Phantimer** is a lightweight, "ghost" timer specifically designed for the [Hyprland](https://hyprland.org/) compositor. It spawns a floating, pinned, and unobtrusive timer window that stays out of your way while keeping you on track.

> _"It floats. It fades. It haunts your workflow... productively."_

---

## ✨ Features

- **👻 Ghost Mode:** Automatically floats, pins, and removes borders from the timer window.
- **📍 Smart Positioning:** Snaps perfectly to the top-right corner of your active monitor (using `monitor_w` logic).
- **🌫️ Interactive Opacity:** High visibility when active (90%), fades into the background when inactive (20%).
- **🐚 Terminal Agnostic:** Automatically detects your default `$TERMINAL` (defaults to `foot` if not set).
- **🦀 Blazingly Fast:** Written in pure Rust for instant startup.

---

## 📸 Preview

![Phantimer Screenshot](./assets/phantimer_dashboard.png)

![Phantimer Screenshot](./assets/phantimer.png)

---

## 📦 Installation

### 🏹 Arch Linux (AUR)

The recommended installation method is via the AUR:

```bash
yay -S phantimer
# or
paru -S phantimer
```
