# Launcher Studio

**Launcher Studio** is a GTK4 desktop application for creating and managing `.desktop` files on Linux. This intuitive tool lets you easily create custom launchers for your favorite applications, ensuring seamless integration with your desktop environment.

---

## Features

- User-friendly GUI for creating `.desktop` files.
- Supports all major desktop entry fields, including advanced options.
- Localized fields for multi-language support.
- Automatically saves launchers to `~/.local/share/applications`.

---

## Getting Started

### Prerequisites

Before using **Launcher Studio**, ensure the following requirements are installed:

- Rust (latest stable version)
- GTK4 development libraries

### Build and Run

Clone the repository, compile the application, and run it:


```shell
bash git clone [https://github.com/MrArnaudMichel/launcher_studio.git](https://github.com/MrArnaudMichel/launcher_studio.git) 
cd launcher_studio 
cargo build --release ./target/release/launcher_studio
```

## License

This application is licensed under the [**MIT License**](LICENSE), allowing free use for personal or commercial purposes.

echo "deb [trusted=yes] https://github.com/MrArnaudMichel/launcher_studio ./" | sudo tee /etc/apt/sources.list.d/launcherstudio.list