# FigX - Pragmatic Design Asset Management

FigX is a no-nonsense CLI tool for importing design assets from Figma into your codebase. Built for developers who value explicit control and reproducibility.

[**See full documentation**](https://tonykolomeytsev.github.io/figx)

<img src=".github/preview.gif" width=100%>

# Features

At a high level, FigX is a straightforward and reliable CLI tool designed for:
- 🔁 Deterministic and reproducible imports
- 🔧 Seamless integration into CI/CD pipelines
- 💻 Cross-platform support: macOS, Windows, and Linux

FigX comes with built-in import profiles for various formats, enabling immediate use without additional setup:

| Profile | Description |
| --- | --- |
| `adroid-webp` | 1. Downloads PNG variants for themes (`night`/`light`) and screen densities (`hdpi`, `xhdpi`, etc.)<br> 2. Converts all variants to WebP using [libwebp](https://developers.google.com/speed/webp)<br> 3. Places the resulting images into the appropriate `drawable-*` directories for Android |
| `compose` | 1. Downloads SVG from Figma<br> 2. Simplifies SVG using [usvg](https://github.com/linebender/resvg/tree/main/crates/usvg)<br> 3. Converts to `ImageVector` for Jetpack Compose |
| `webp` | 1. Downloads PNG from Figma<br> 2. Converts PNG to WebP using [libwebp](https://developers.google.com/speed/webp) |
| `png` | Downloads PNG assets directly from Figma |
| `svg` | Downloads SVG assets directly from Figma | 
| `pdf` | Downloads ЗВА assets directly from Figma | 

> Profiles `png`, `svg`, `pdf` and `compose` support matrix-like import configurations — multiple variants (e.g. `light`/`night`, sizes `16`/`20`/`24`) for a single resource, similar to GitHub Actions matrices.

# Quick Start

## Install

### For MacOS

The easiest way to install on macOS is via Homebrew:

```bash
brew tap tonykolomeytsev/figx
brew install figx
```

### For Windows

Download the latest `.msi` installer from the [releases page](https://github.com/tonykolomeytsev/figx/releases/latest), then run the installer to complete the setup.

Download installer latest version MSI installer and then install.
Releases latest release: 

### For Linux

Follow the detailed installation instructions available in the [documentation](https://tonykolomeytsev.github.io/figx/user_guide/1-installation.html).

## Run your first import

- TODO

# Documentation
Full documentation available at: [tonykolomeytsev.github.io/figx](https://tonykolomeytsev.github.io/figx)

# License

GPL-3.0 license © 2025 Anton Kolomeytsev
