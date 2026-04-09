# Launcher Studio

**Launcher Studio** is a GTK4 desktop application for creating and managing `.desktop` files on Linux. This intuitive tool lets you easily create custom launchers for your favorite applications, ensuring seamless integration with your desktop environment.

## What Is Implemented

- Unified action flow between menu and toolbar for `New`, `Open`, and `Save`
- Unsaved changes protection before destructive navigation (`New`, `Open`, `Quit`)
- `Save As` action with `.desktop` extension enforcement
- Keyboard shortcuts for common actions
- File chooser filter for `.desktop` files
- Backup creation on overwrite (`<file>.desktop.bak`)
- Centralized `.desktop` parsing in `DesktopEntry`
- Parser and writer unit tests for round-trip and sanitization
- Icon service caching to reduce repeated theme scans

## Features

- User-friendly GUI for creating `.desktop` files.
- Supports all major desktop entry fields, including advanced options.
- Localized fields for multi-language support.
- Automatically saves launchers to `~/.local/share/applications`.

## Keyboard Shortcuts

- `Ctrl+N`: New
- `Ctrl+O`: Open
- `Ctrl+S`: Save
- `Ctrl+Shift+S`: Save As
- `F5`: Refresh
- `Ctrl+Q`: Quit
- `F11`: Toggle fullscreen


## Getting Started

### Prerequisites

Before using **Launcher Studio**, ensure the following requirements are installed:

- Rust (latest stable version)
- GTK4 development libraries

### Build and Run

Clone the repository, compile the application, and run it:

```shell
git clone https://github.com/MrArnaudMichel/launcher_studio.git
cd launcher_studio
cargo build --release
./target/release/launcher_studio
```

### Development Run

```shell
cargo run
```

### Quality Checks

```shell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Roadmap

### Quick wins

- Improve inline validation feedback in form fields
- Add explicit overwrite confirmation in `Save As`
- Display current file state in window title (`*` when dirty)

### Mid-term

- Asynchronous file loading/saving to keep UI responsive
- Add parser test fixtures for complex real-world `.desktop` files
- Add CI pipeline for formatting, linting, and tests

### Long-term

- Extended source browser for user and system application directories
- Advanced preview with grouped sections and validation markers
- Packaging matrix (Deb + Flatpak)

## License

This application is licensed under the [**MIT License**](LICENSE), allowing free use for personal or commercial purposes.