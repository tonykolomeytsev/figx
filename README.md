# Figmagic - Pragmatic Design Asset Management

Figmagic is a no-nonsense CLI tool for importing design assets from Figma into your codebase. Built for developers who value explicit control and reproducibility.

## Why Choose Figmagic?
### 🎯 **Explicit by Design**

- **No magic**: Every operation is clearly defined in your config

- **No surprises**: Assets are imported exactly as specified - nothing more, nothing less

- **No structural requirements**: Works with your existing project layout

### ⚡ **Performance Optimized**

- **Fast exports**: faster than figma-export

- **Smart caching**: Only fetches changed assets

### 🔍 Single Source of Truth

- **Config-as-documentation**: `.figmagic.toml` defines your asset pipeline

- **Reproducible results** across all environments

- Version control friendly

# Quick Start

## Install

### For MacOS

Easiest way for MacOS is to install from homebrew:

```bash
brew tap tonykolomeytsev/figmagic
brew install figmagic
```
### For other OSs

See instructions in [docs](https://tonykolomeytsev.github.io/figmagic/user_guide/1-installation.html).

### Build from source on any OS

```bash
cargo install --release --locked --path app
```

## Run your first import

Full explanation in the [docs](https://tonykolomeytsev.github.io/figmagic/user_guide/2.2.1-minimal-example.html).

```toml
# .figmagic.toml
[remotes.design]
file_key = "MhjeA23R15tAR3PO2JamCv"
container_node_ids = ["30788-66292"]
```

And:

```toml
# .fig.toml
[svg]
puzzle = "Environment / Puzzle"
```

Then just run:
```bash
figmagic import //...
```

# Philosophy

Figmagic follows these core principles:
1. **Explicit over implicit**: All behavior is defined in configuration
2. **Minimal assumptions**: Works with your project structure
3. **Deterministic outputs**: Same inputs → same outputs, every time. Now only the designer can screw things up.
4. **Developer experience**: Fast, cache-aware, and CI-friendly

# Documentation
Full documentation available at: [tonykolomeytsev.github.io/figmagic](https://tonykolomeytsev.github.io/figmagic)

# License

GPL-3.0 license © 2025 Anton Kolomeytsev
