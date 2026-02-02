# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

S.EE Desktop is a collection of native desktop clients for the S.EE URL shortening, text sharing, and file hosting service. This is a monorepo with platform-specific implementations in separate directories.

## Repository Structure

```
desktop/
├── linux/          # GTK4 + libadwaita + Rust (available)
├── macos/          # Coming soon
├── windows/        # Coming soon
└── .github/
    └── workflows/
        └── release-linux.yml   # Automated Linux releases
```

## Platform-Specific Guidance

### Linux

See [linux/CLAUDE.md](./linux/CLAUDE.md) for detailed guidance including:
- Build commands (`cargo build`, `cargo run`)
- Architecture (async bridge pattern, local history storage)
- API reference and SDK usage
- UI patterns with GTK4/libadwaita

**Quick reference:**
```bash
cd linux
cargo build --release    # Build
cargo run --release      # Run
cargo check              # Check for errors
cargo clippy             # Lint
cargo fmt                # Format
```

## CI/CD

### Linux Release Workflow

Triggered by version tags (`v*`) or manual dispatch. Builds:
- Flatpak (x86_64 only - no ARM64 container available)
- deb packages (x86_64, ARM64)
- rpm packages (x86_64, ARM64)

**Testing releases:**
```bash
git tag v0.1.0-test1
git push origin v0.1.0-test1
```

**Creating official release:**
```bash
git tag v0.1.0
git push origin v0.1.0
```

## Packaging

Linux packaging files are in `linux/packaging/`:
- `PKGBUILD` - Arch Linux (build from source)
- `PKGBUILD-bin` - Arch Linux (prebuilt binary)
- `flatpak/ee.s.app.yml` - Flatpak manifest
- `deb/postinst` - Debian post-install script
- `rpm/post_install.sh` - RPM post-install script

Package metadata is in `linux/Cargo.toml` under `[package.metadata.deb]` and `[package.metadata.generate-rpm]`.

## AUR Packages

Published as `see-desktop` and `see-desktop-bin` on AUR (name `see` was taken).
