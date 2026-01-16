# macOS Desktop Application

This document explains how to build and run Artist Oil Paints as a standalone macOS desktop application.

## Overview

The macOS app bundle (`ArtistOilPaints.app`) wraps the Leptos server in a clickable application that:
- Starts the web server automatically
- Opens your browser to the app
- Runs locally on `http://127.0.0.1:3000`

## Prerequisites

- Rust toolchain installed
- `cargo-leptos` installed: `cargo install cargo-leptos`
- The project must have a valid `data.db` SQLite database

## Building the App

From the project root directory:

```bash
./build-app.sh
```

This will:
1. Build the release binary with `cargo leptos build --release`
2. Create/update the `ArtistOilPaints.app` bundle
3. Copy the binary, database, site assets, and .env to the bundle

## App Bundle Structure

```
ArtistOilPaints.app/
└── Contents/
    ├── Info.plist          # macOS app metadata
    ├── MacOS/
    │   └── launch          # Startup script
    └── Resources/
        ├── aop             # Server binary
        ├── data.db         # SQLite database
        ├── site/           # Frontend assets (CSS, JS, WASM)
        ├── .env            # Environment config (optional)
        └── server.log      # Runtime log file
```

## Running the App

### Option 1: Double-click
Simply double-click `ArtistOilPaints.app` in Finder.

### Option 2: From Terminal
```bash
open ArtistOilPaints.app
```

### Option 3: Install to Applications
```bash
cp -r ArtistOilPaints.app /Applications/
```
Then launch from Launchpad or Applications folder.

## What Happens on Launch

1. The launcher script starts the server
2. Waits for the server to be ready (up to 15 seconds)
3. Opens your default browser to `http://127.0.0.1:3000`
4. Server continues running until you quit the app

## Stopping the App

- Close the terminal window that opened with the app
- Or kill the process: `lsof -ti:3000 | xargs kill`

## Troubleshooting

### "Server failed to start"
Check the log file at `ArtistOilPaints.app/Contents/Resources/server.log`

Common issues:
- Missing `data.db` - ensure database was copied
- Port 3000 already in use - kill existing process
- Missing `.env` with required variables

### "Connection refused" in browser
The server didn't start in time. Check `server.log` for errors.

### App won't open (macOS security)
If macOS blocks the app:
1. Go to System Preferences > Security & Privacy
2. Click "Open Anyway" for ArtistOilPaints

Or run from terminal first to bypass Gatekeeper:
```bash
xattr -cr ArtistOilPaints.app
```

## Adding a Custom Icon

1. Create a 512x512 or 1024x1024 PNG image
2. Convert to .icns format:
   ```bash
   sips -s format icns your-icon.png --out ArtistOilPaints.app/Contents/Resources/AppIcon.icns
   ```
3. The icon will appear after rebuilding or copying to a new location

## Environment Variables

The app reads from `.env` in the Resources folder. Key variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | SQLite path | `sqlite:./data.db` |
| `RESEND_API_KEY` | Email API key | (required for email features) |
| `EMAIL_FROM` | Sender address | (required for email features) |
| `BASE_URL` | App URL for emails | `http://127.0.0.1:3000` |

## Development vs Production

For local use, the default settings work fine. For sharing the app:

1. Remove sensitive keys from `.env`
2. Consider disabling email verification for offline use
3. The database contains paint spectral data but no user data initially

## Updating the App

After making code changes:

```bash
./build-app.sh
```

If the app is in Applications folder:
```bash
./build-app.sh && cp -r ArtistOilPaints.app /Applications/
```
