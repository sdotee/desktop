# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/claude-code) when working with this codebase.

## Project Overview

S.EE Desktop is a native GNOME desktop client for the S.EE URL shortening, text sharing, and file hosting service. Built with Rust, GTK4, and libadwaita for a modern Linux desktop experience.

- **App ID**: `ee.s.app`
- **Binary Name**: `see`
- **Language**: Rust (Edition 2024)
- **UI Framework**: GTK4 + libadwaita

## Build Commands

```bash
# Build (debug)
cargo build

# Build (release)
cargo build --release

# Run
cargo run --release

# Check for errors without building
cargo check

# Run tests
cargo test

# Generate documentation
cargo doc --open
```

## Architecture

### Key Design Decisions

1. **SDK is Blocking**: The `see-sdk` uses `reqwest::blocking`, so we bridge to GTK's async main loop using `gio::spawn_blocking` + `async_channel`.

2. **Local History Storage**: The SDK lacks list endpoints, so created items are stored locally in `~/.local/share/see/history.json`.

3. **Config Priority** (high to low):
   - Environment variables (`SEE_API_KEY`, `SEE_BASE_URL`, `SEE_TIMEOUT`)
   - Config file (`~/.config/see/config.toml`)
   - SDK defaults

### Project Structure

```
see-desktop/
├── Cargo.toml              # Dependencies
├── build.rs                # GResource compilation
├── data/
│   ├── ee.s.app.desktop    # Desktop entry
│   ├── ee.s.app.metainfo.xml
│   ├── ee.s.app.gschema.xml
│   ├── icons/              # App icons
│   └── resources/
│       ├── resources.gresource.xml
│       ├── style.css       # Custom CSS
│       ├── icons/          # Custom SVG icons
│       └── ui/*.ui         # UI templates
└── src/
    ├── main.rs             # Entry point
    ├── application.rs      # AdwApplication subclass
    ├── config.rs           # Config management
    ├── error.rs            # Error types
    ├── api/
    │   └── client.rs       # SDK wrapper + async bridge
    ├── storage/
    │   ├── history.rs      # Local history storage
    │   └── models.rs       # Data models (LinkEntry, TextEntry, FileEntry)
    ├── qrcode/
    │   └── generator.rs    # QR code generation (PNG/SVG/PDF)
    ├── widgets/
    │   ├── link_row.rs     # Link list item widget
    │   ├── text_row.rs     # Text list item widget
    │   └── file_row.rs     # File list item widget
    └── views/
        ├── window.rs       # Main window
        ├── links.rs        # Links view (URL shortening)
        ├── texts.rs        # Texts view (text sharing)
        ├── files.rs        # Files view (file uploads)
        ├── qr_dialog.rs    # QR code dialog
        └── preferences.rs  # Settings window
```

### Async Bridge Pattern

```rust
// Spawn blocking SDK call, receive result asynchronously
let receiver = spawn_api_call(config, ApiRequest::ShortenUrl { url, domain, slug });

glib::spawn_future_local(async move {
    if let Ok(ApiResponse::ShortenUrl(Ok(result))) = receiver.recv().await {
        // Update UI on main thread
    }
});
```

## Key Files

- **`src/api/client.rs`**: SDK wrapper with `async_bridge` module for non-blocking API calls
- **`src/storage/models.rs`**: Data structures for links, texts, and files
- **`src/views/*.rs`**: Main UI views with create forms and history lists
- **`data/resources/style.css`**: Custom GTK4 CSS styling
- **`data/resources/icons/`**: Custom SVG symbolic icons

## SDK Response Fields

### URL Shortening
- `result.data.short_url` - The shortened URL
- `result.data.slug` - URL slug/alias

### Text Sharing
- `result.data.short_url` - Direct link to text
- `result.data.slug` - Text slug
- Page URL format: `https://{domain}/{slug}`
- Raw URL format: `https://{domain}/{slug}/raw`

### File Upload
- `result.data.url` - Direct file URL
- `result.data.page` - Share page URL (optional)
- `result.data.hash` - File hash (used as slug for deletion)
- `result.data.filename`, `result.data.size`

## UI Patterns

- Use `adw::PreferencesGroup` for form sections
- Use `adw::EntryRow` / `adw::ComboRow` for form fields
- Use `adw::ActionRow` subclasses for list items
- Use `adw::ToastOverlay` for notifications
- Use `adw::AlertDialog` for confirmations
- Pagination: 10 items per page with Previous/Next buttons

## Custom Icons

Located in `data/resources/icons/`, prefixed with `see-`:
- `see-link-symbolic.svg` - Links tab
- `see-text-symbolic.svg` - Texts tab
- `see-file-symbolic.svg` - Files tab
- `see-qr-code-symbolic.svg` - QR code button
- `see-copy-page-symbolic.svg` - Copy share page
- `see-copy-link-symbolic.svg` - Copy direct link
- `see-delete-symbolic.svg` - Delete button

## Common Tasks

### Adding a New API Endpoint

1. Add request/response variants to `ApiRequest`/`ApiResponse` in `src/api/client.rs`
2. Implement the SDK call in `ApiClient`
3. Handle the response in `spawn_api_call` match arm

### Adding a New View

1. Create `src/views/new_view.rs`
2. Add to `src/views/mod.rs`
3. Add page to `window.ui` ViewStack
4. Initialize in `src/views/window.rs`

### Modifying Storage Models

1. Update struct in `src/storage/models.rs`
2. Add `#[serde(default)]` for new optional fields (backward compatibility)
3. Update corresponding view and widget code

## S.EE API Reference

OpenAPI Spec: https://raw.githubusercontent.com/sdotee/docs/refs/heads/main/openapi_swagger.yaml

### Authentication
All API requests require `Authorization: Bearer {api_key}` header.

### POST /api/v1/shorten - Create Short URL
```json
{
  "target_url": "https://example.com",  // required
  "domain": "s.ee",                      // required, default: "s.ee"
  "custom_slug": "my-link",              // optional
  "title": "My Link",                    // optional
  "password": "secret",                  // optional, 3-32 chars
  "expire_at": 1735689600,               // optional, unix timestamp
  "tag_ids": [1, 2]                      // optional
}
```

### POST /api/v1/text - Create Text Sharing
```json
{
  "content": "Hello World",              // required, max 2MB
  "title": "My Text",                    // required
  "domain": "fs.to",                     // optional, default: "fs.to"
  "text_type": "plain_text",             // optional: "plain_text" | "source_code" | "markdown"
  "custom_slug": "my-text",              // optional
  "password": "secret",                  // optional, 3-32 chars
  "expire_at": 1735689600,               // optional, unix timestamp
  "tag_ids": [1, 2]                      // optional, max 5
}
```

### POST /api/v1/file/upload - Upload File
- Multipart form data with `file` field
- Returns: `url` (direct), `page` (share page), `hash` (delete key), `filename`, `size`

### DELETE Endpoints
- `DELETE /api/v1/shorten` - Body: `{ "domain": "s.ee", "slug": "abc123" }`
- `DELETE /api/v1/text` - Body: `{ "domain": "fs.to", "slug": "abc123" }`
- `GET /api/v1/file/delete/{hash}` - Delete file by hash

### GET Domain Lists
- `GET /api/v1/domains` - URL shortening domains
- `GET /api/v1/text/domains` - Text sharing domains
- `GET /api/v1/file/domains` - File upload domains

### Response Format
```json
{
  "code": 200,
  "message": "Success",
  "data": { ... }
}
```
