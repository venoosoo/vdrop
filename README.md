# 💧 vdrop

[![GitHub license](https://img.shields.io/github/license/venoosoo/vdrop)](LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/venoosoo/vdrop?style=social)](https://github.com/venoosoo/vdrop/stargazers)

## Table of Contents
- [About](#about)
- [Features](#features)
- [Motivation](#motivation)
- [Installation](#installation)
- [Contributing & To-Do](#contributing--to-do)
- [License](#license)

---

## About

**vdrop** is a **fast, local network file transfer utility** built using **Rust** and **Tauri** for a lightweight, cross-platform experience. It provides a simple graphical interface for sending and receiving files between devices on the same network.

## Features

- **Local Network Transfer:** Facilitates quick file exchange between connected devices.
- **Intuitive GUI:** Simple, easy-to-use interface for drag-and-drop file transfers.
- **Transfer History:** Easily view a list of files you have received.
- **Cross-Platform:** Works seamlessly on both **Windows** and **Linux**.
- **Rust Performance:** Leverages the speed and efficiency of a Rust backend.

## Motivation

The idea behind **vdrop** stemmed from the frustration of using messaging apps, email, or cloud services just to move files between your own local devices. **vdrop** offers a dedicated, direct, and more efficient solution for sharing files instantly over your home or office network.

## Installation

### Prerequisites

You will need the standard development environment for a Tauri application, including **Rust** and the necessary system dependencies for building the native application.

### From Source

1. **Clone the repository:**
   ```bash
   git clone [https://github.com/venoosoo/vdrop.git](https://github.com/venoosoo/vdrop.git)
   cd vdrop

    Install dependencies and run (Tauri/Cargo):
    Bash

    cargo tauri dev

    (For production builds, use cargo tauri build)

    View History: Use the history tab to see a record of all received files.

Contributing & To-Do

Contributions, issues, and feature requests are welcome!
Planned Features / To-Do List

A dedicated Settings Tab is planned to introduce user customization, including:

    Language selection.

    Option to automatically accept incoming files.

    Implementation of a white/black list for trusted devices.

    Support for different UI themes (e.g., Light/Dark mode).

License

This project is licensed under the MIT License - see the LICENSE file for details.

Project Link: https://github.com/venoosoo/vdrop
