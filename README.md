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

---

## Build, Package, and Publish

Once your changes are ready, you can build and publish a version of your app for others to use.

### 1. Generate the Package and APT Repository

To create a `.deb` package and an APT repository, run:

```shell
bash ./build_and_publish.sh
```

This script will:
- Compile the app in `release` mode.
- Generate a `.deb` file for distribution.
- Update the `debian/` folder and create a `Packages.gz` file.

### 2. Publish to GitHub Pages

To publish the APT repository on GitHub Pages (branch `gh-pages`), run:

```shell
bash ./publish_to_ghpages.sh
```


The script will:
- Upload the contents of the `debian/` folder to the `gh-pages` branch.
- Update the remote repository with your APT repository.

### 3. Configure GitHub Pages

In your repository settings:
- Set GitHub Pages to use the `gh-pages` branch.
- Choose the root directory as the publishing source.

### 4. User Updates

End users who have added your APT repository to their package manager will see updates automatically.

---

## Dependencies

The following tools are required for development, compilation, and publishing:

- **Rust**: Programming language (https://www.rust-lang.org/)
- **cargo-deb**: For creating Debian packages
- **dpkg-scanpackages**: For generating the `Packages.gz` file
- **git**: Version control system
- **rsync**: File synchronization

---

## License

This application is licensed under the [**MIT License**](LICENSE), allowing free use for personal or commercial purposes.

echo "deb [trusted=yes] https://launcherstudio.arnaudmichel.fr/ ./" | sudo tee /etc/apt/sources.list.d/launcherstudio.list