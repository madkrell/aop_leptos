# Artist Oil Paints

A web application for finding optimal paint mixtures to match target colors using Kubelka-Munk color theory.

## Features

- **Target Color Mixing** - Upload an image or use a color picker to select a target color, then find the best paint mixture
- **Test Mix** - Create custom paint mixtures and preview the result
- **User Settings** - Choose your paint brand and preferred mixing strategy
- **Authentication** - Email/password auth with verification and password reset

## Tech Stack

- **Leptos 0.8** - Full-stack Rust web framework
- **Axum** - Web server
- **SQLite + SQLx** - Database
- **Kubelka-Munk** - Physically accurate paint mixing algorithm

## Quick Start

1. Copy `.env.example` to `.env` and configure your settings
2. Ensure you have `data.db` with the paint spectral data
3. Run `cargo leptos watch` for development

See [CLAUDE.md](CLAUDE.md) for detailed documentation.
