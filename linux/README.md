# S.EE Desktop for Linux

A native GNOME desktop client for [S.EE](https://s.ee) - URL shortening, text sharing, and file hosting service.

![GTK4](https://img.shields.io/badge/GTK-4.14+-blue)
![libadwaita](https://img.shields.io/badge/libadwaita-1.5+-purple)
![Rust](https://img.shields.io/badge/Rust-2024-orange)
![License](https://img.shields.io/badge/License-MIT-green)

## Features

- **URL Shortening** - Create short links with custom aliases
- **Text Sharing** - Share code snippets, notes, and text content
- **File Uploads** - Upload and share files with drag & drop support
- **QR Code Generation** - Generate QR codes for any link (export as PNG, SVG, or PDF)
- **Local History** - Track all your created links, texts, and files
- **Multiple Domains** - Choose from available domains for each service
- **Native GNOME Experience** - Follows GNOME Human Interface Guidelines

## Screenshots

*Coming soon*

## Installation

### Arch Linux (AUR)

```bash
# Using yay (prebuilt binary, recommended)
yay -S see-desktop-bin

# Or build from source
yay -S see-desktop
```

- [see-desktop](https://aur.archlinux.org/packages/see-desktop) - Build from source
- [see-desktop-bin](https://aur.archlinux.org/packages/see-desktop-bin) - Prebuilt binary

### Ubuntu/Debian (.deb)

Download from [Releases](https://github.com/sdotee/desktop/releases):

```bash
# x86_64
wget https://github.com/sdotee/desktop/releases/download/v0.1.0/see_0.1.0_amd64.deb
sudo apt install ./see_0.1.0_amd64.deb

# ARM64
wget https://github.com/sdotee/desktop/releases/download/v0.1.0/see_0.1.0_arm64.deb
sudo apt install ./see_0.1.0_arm64.deb
```

### Fedora (.rpm)

Download from [Releases](https://github.com/sdotee/desktop/releases):

```bash
# x86_64
wget https://github.com/sdotee/desktop/releases/download/v0.1.0/see-0.1.0-1.x86_64.rpm
sudo dnf install ./see-0.1.0-1.x86_64.rpm

# ARM64
wget https://github.com/sdotee/desktop/releases/download/v0.1.0/see-0.1.0-1.aarch64.rpm
sudo dnf install ./see-0.1.0-1.aarch64.rpm
```

### Flatpak

Download from [Releases](https://github.com/sdotee/desktop/releases):

```bash
wget https://github.com/sdotee/desktop/releases/download/v0.1.0/see-0.1.0-x86_64.flatpak
flatpak install ./see-0.1.0-x86_64.flatpak
```

### Build from Source

#### Requirements

- GTK 4.14+
- libadwaita 1.5+
- Rust 1.85+ (2024 edition)

#### Dependencies

**Arch Linux:**
```bash
sudo pacman -S gtk4 libadwaita cairo pango gdk-pixbuf2
```

**Ubuntu/Debian:**
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev libcairo2-dev libpango1.0-dev libssl-dev pkg-config
```

**Fedora:**
```bash
sudo dnf install gtk4-devel libadwaita-devel cairo-devel pango-devel openssl-devel
```

#### Build

```bash
git clone https://github.com/sdotee/desktop.git
cd desktop/linux
cargo build --release
./target/release/see
```

## Configuration

### Config File

Configuration is stored at `~/.config/see/config.toml`:

```toml
api_key = "your-api-key-here"
base_url = "https://s.ee/api/v1"
timeout = 30

# Default domains for each service
default_link_domain = "s.ee"
default_text_domain = "ba.sh"
default_file_domain = "fs.to"
```

### Environment Variables

You can also configure the app using environment variables (takes precedence over config file):

```bash
export SEE_API_KEY="your-api-key"
export SEE_BASE_URL="https://s.ee/api/v1"
export SEE_TIMEOUT=30
```

### Getting an API Key

1. Visit [s.ee](https://s.ee) and create an account
2. Go to your dashboard
3. Generate an API key
4. Enter the key in Preferences (Ctrl+,)

## Usage

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+1` | Switch to Links view |
| `Ctrl+2` | Switch to Texts view |
| `Ctrl+3` | Switch to Files view |
| `Ctrl+,` | Open Preferences |
| `Ctrl+?` | Show keyboard shortcuts |
| `Ctrl+Q` | Quit |

### Links

1. Paste a URL in the "URL" field
2. Optionally select a domain and custom alias
3. Click "Shorten URL"
4. The shortened URL is automatically copied to clipboard

### Texts

1. Enter a title (optional, defaults to "Untitled")
2. Type or paste your text content
3. Click "Create Text"
4. Get both a share page URL and a raw text URL

### Files

1. Click "Choose File" or drag & drop a file
2. Wait for upload to complete
3. Get both a share page URL and a direct download URL

## Data Storage

- **Config**: `~/.config/see/config.toml`
- **History**: `~/.local/share/see/history.json`

History is stored locally and is not synced with the server. To delete items from S.EE servers, visit [s.ee/user/dashboard](https://s.ee/user/dashboard).

## Installing GSettings Schema (Optional)

For persistent window state, install the GSettings schema:

```bash
sudo install -Dm644 data/ee.s.app.gschema.xml /usr/share/glib-2.0/schemas/
sudo glib-compile-schemas /usr/share/glib-2.0/schemas/
```

## Development

### Project Structure

```
see-desktop/
├── src/
│   ├── main.rs           # Entry point
│   ├── application.rs    # App lifecycle
│   ├── config.rs         # Configuration
│   ├── api/              # API client
│   ├── storage/          # Local storage
│   ├── views/            # UI views
│   ├── widgets/          # Custom widgets
│   └── qrcode/           # QR generation
├── data/
│   └── resources/        # UI templates, CSS, icons
└── Cargo.toml
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check code
cargo clippy
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Links

- **Website**: [s.ee](https://s.ee)
- **Issues**: [GitHub Issues](https://github.com/sdotee/desktop/issues)
- **API Documentation**: [S.EE API Docs](https://s.ee/docs/developers/api/)

## Credits

- Built with [GTK4](https://gtk.org/) and [libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/)
- Uses [see-sdk](https://crates.io/crates/see-sdk) for API communication
- Icons from [Hugeicons](https://hugeicons.com/)
