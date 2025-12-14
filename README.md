That's a great idea for readability! Using a dedicated list format makes the "To-Do" section much clearer.

Here is the revised README.md focusing on improved formatting for the Contributing & To-Do section:
Markdown

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


Contributing & To-Do

We welcome contributions, bug reports, and suggestions! Please feel free to open an issue or submit a pull request.
🔨 Development Roadmap (To-Do)

A major focus is introducing a dedicated Settings Tab to give users control over the application's behavior and appearance:

    Customization:

        ✅ Implement support for Changing Themes (e.g., Light/Dark mode).

        ✅ Introduce Language Selection.

    Security & Control:

        ✅ Add an option for Automatically Accepting/Rejecting Files.

        ✅ Develop a White/Black List feature for trusted/blocked devices.

License

This project is licensed under the MIT License - see the LICENSE file for details.

Project Link: https://github.com/venoosoo/vdrop
