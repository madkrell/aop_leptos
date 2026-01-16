# Artist Oil Paints - Leptos Application

## Overview

Artist Oil Paints is a web application for finding optimal paint mixtures to match target colors.

**Tech Stack:**
- **Leptos 0.8 + Axum** - Full-stack Rust web framework
- **SQLite + SQLx** - Database with direct SQL queries
- **tower-sessions** - Cookie-based session authentication
- **Resend.com** - Email verification and password reset
- **rayon** - Parallel processing for paint combinations

**Deployment:** Designed for oxyde.cloud (Leptos native hosting)

---

## Guiding Principles

1. **Clean & Concise** - Remove verbosity, no unnecessary abstractions
2. **Simple > Clever** - Direct approaches over complex patterns
3. **Delete liberally** - If it doesn't add value, remove it
4. **SQLx over ORM** - Raw SQL is clearer and more efficient for this app

---

## Architecture

```
┌─────────────────────────────────────────┐
│        Leptos 0.8 + Axum Server         │
├─────────────────────────────────────────┤
│  Leptos SSR Pages   │  Server Functions │
│  (/, /login, etc.)  │  (auth, mixing)   │
├─────────────────────────────────────────┤
│      tower-sessions (cookie-based)      │
├─────────────────────────────────────────┤
│            SQLite (SQLx)                │
│    (users, settings, paint data)        │
└─────────────────────────────────────────┘
```

---

## Paint Mixing Algorithm

### Kubelka-Munk Theory

The application uses the Kubelka-Munk (K-M) model for physically accurate subtractive paint mixing.

**Key formulas:**
- **Reflectance → K/S:** `K/S = (1 - R)² / (2R)`
- **K/S → Reflectance:** `R = 1 + K/S - √(K/S² + 2·K/S)`

**How mixing works:**
1. Convert each paint's spectral reflectance to K/S ratios (absorption/scattering)
2. Mix K/S values using weighted average (pigments are additive in K/S space)
3. Convert mixed K/S back to reflectance
4. Calculate Delta E error in Lab color space (perceptually meaningful)

**Why K-M is better than linear mixing:**
- Linear reflectance mixing is physically incorrect for paints
- K-M accounts for how pigments absorb light
- Prevents nonsensical results like "81% black" for light colors

**Key files:**
- `src/services/optimization.rs` - K-M mixing functions
- `src/services/paint_mixing.rs` - Paint combination service with parallel processing
- `src/services/lhtss.rs` - LHTSS algorithm for RGB → spectral reflectance conversion

### LHTSS Algorithm

Converts sRGB colors to spectral reflectance curves:
- Uses Newton-Raphson iteration with 500 max iterations
- Tolerance: 1e-6, with best-solution fallback if error < 1.0
- Returns 31 wavelength values (400nm-700nm, 10nm steps)

### Parallelization

All paint combination searches use `rayon::par_iter()` for parallel processing:
- `find_black_white_n_colors` - White + Black + 2 or 3 colors
- `find_all_available_colors` - 3, 4, 5 paint combinations
- `find_neutral_greys` - Grey + 2 other colors
- `find_no_black` - 3-4 paint combinations without black

---

## Mix Strategy Options

Available in Settings page (`src/models/paint.rs` → `MixChoice`):

1. **black + white + 2 colours** - White, Black, + 2 chromatic paints
2. **black + white + 3 colours** - White, Black, + 3 chromatic paints (more combinations, slower)
3. **all available colours** - Try all 3, 4, 5 paint combinations
4. **neutral greys** - Use grey paints as base
5. **no black** - Exclude black from mixtures

---

## Project Structure

```
/
├── Cargo.toml                    # Workspace + leptos metadata
├── src/
│   ├── lib.rs                    # Crate root, feature gates
│   ├── app.rs                    # Main Leptos App component
│   ├── main.rs                   # Axum server setup (SSR only)
│   ├── state.rs                  # AppState definition
│   │
│   ├── pages/                    # Leptos page components
│   │   ├── home.rs               # Landing page
│   │   ├── login.rs              # Login form
│   │   ├── register.rs           # Registration form
│   │   ├── verify_email.rs       # Email verification
│   │   ├── forgot_password.rs    # Request password reset
│   │   ├── reset_password.rs     # Set new password
│   │   ├── settings.rs           # User paint preferences
│   │   ├── target_mix.rs         # Main color mixing tool
│   │   └── test_mix.rs           # Custom mix testing
│   │
│   ├── components/               # Reusable UI components
│   │   ├── nav.rs                # Navigation bar
│   │   └── auth_guard.rs         # Auth guard wrapper
│   │
│   ├── server_fns/               # Server functions
│   │   ├── auth.rs               # Auth server functions
│   │   └── paint.rs              # Paint/mixing server functions
│   │
│   ├── services/                 # Business logic
│   │   ├── auth.rs               # Password hashing, user management
│   │   ├── email.rs              # Resend API integration
│   │   ├── lhtss.rs              # LHTSS spectral algorithm
│   │   ├── optimization.rs       # Kubelka-Munk mixing + gradient descent
│   │   └── paint_mixing.rs       # Paint combination finder
│   │
│   ├── db/                       # Database layer (SQLx)
│   │   └── mod.rs                # Pool setup + query functions
│   │
│   └── models/                   # Shared data structures
│       └── paint.rs              # MixingResult, ColorError, MixChoice
│
├── style/
│   └── main.scss                 # Styles
│
├── public/                       # Static assets
├── data.db                       # SQLite database (not in git)
└── .env                          # Environment config (not in git)
```

---

## Database Schema

```sql
-- Users table
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    email_verified INTEGER DEFAULT 0,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    failed_attempts INTEGER DEFAULT 0,
    locked_until TEXT
);

-- Tokens (verification and reset)
CREATE TABLE tokens (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,  -- 'verify' or 'reset'
    hash TEXT NOT NULL,
    expires_at TEXT NOT NULL
);

-- User settings
CREATE TABLE user_settings (
    _id TEXT PRIMARY KEY,
    email TEXT,
    colour_mix_choice TEXT,
    selected_colors TEXT  -- JSON: {"brand_name": ["color1", "color2", ...]}
);

-- Paint brand tables (e.g., michael_harding, winsor_newton_artist_oil_colour, etc.)
-- Pre-populated with spectral data
CREATE TABLE <brand_name> (
    _id TEXT PRIMARY KEY,
    spectral_curve BLOB,     -- bincode Vec<f64>, 31 values
    d65_10deg_hex TEXT       -- Hex color for display
);
```

---

## Environment Configuration

**`.env`:**
```env
DATABASE_URL=sqlite:./data.db

# Resend.com
RESEND_API_KEY=re_xxxxxxxxxxxx
EMAIL_FROM=noreply@artistoilpaints.co.uk
BASE_URL=https://artistoilpaints.co.uk

# Production (set to any value to enable HTTPS cookies)
# PRODUCTION=1
```

---

## Commands

```bash
cargo leptos watch     # Dev server with hot reload
cargo leptos build -r  # Release build
cargo test             # Tests
cargo clippy           # Lint
```

---

## References

- [Kubelka-Munk theory - Wikipedia](https://en.wikipedia.org/wiki/Kubelka%E2%80%93Munk_theory)
- [Spectral.js - GitHub](https://github.com/rvanwijnen/spectral.js/)
- [Leptos Book](https://book.leptos.dev/)
