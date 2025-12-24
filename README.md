# LoomWM

**Weaving your digital intent.**

> **Loom** (teler) + **W**ave (ona) + **M**anager = LoomWM
>
> The "W" stands for **Wave**, not Window — because information flows like waves across the canvas, not trapped in rigid rectangular boxes.

LoomWM is a next-generation Wayland compositor that breaks away from the traditional desktop paradigm. Instead of overlapping windows, it presents an **infinite canvas** where applications become **nodes** that can be arranged, connected, and manipulated spatially.

## Vision

Traditional desktop environments are based on concepts from the 1970s-80s: windows, folders, and icons. LoomWM reimagines the desktop for the AI era:

- **Fluid Workspace**: No rigid windows. Content flows on an infinite 2D canvas.
- **Node-Based Organization**: Applications are nodes. Connect them to show relationships.
- **Intent-Driven UI**: Express what you want, not how to get there.
- **Generative Interface**: The UI adapts and generates elements based on context.
- **Spatial Memory**: Leverage human spatial memory for better organization.

## Features (Planned)

- [ ] Infinite canvas with pan/zoom navigation
- [ ] Node-based window management
- [ ] Semantic connections between content
- [ ] AI-powered intent parsing
- [ ] Generative UI elements
- [ ] Custom Wayland protocol extensions for node-aware apps

## Architecture

```
loomWM/
├── src/main.rs              # Entry point
└── crates/
    ├── loom-core/           # Wayland compositor (Smithay)
    ├── loom-canvas/         # Infinite canvas & node system
    ├── loom-protocol/       # Custom Wayland extensions
    ├── loom-ai/             # Intent parsing & UI generation
    └── loom-config/         # Configuration management
```

## Building

### Dependencies (Fedora)

```bash
sudo dnf install \
    libdisplay-info-devel \
    systemd-devel \
    libseat-devel \
    libinput-devel \
    libxkbcommon-devel \
    mesa-libgbm-devel \
    libdrm-devel \
    wayland-devel \
    mesa-libEGL-devel \
    mesa-libGL-devel \
    pkg-config
```

### Dependencies (Debian/Ubuntu)

```bash
sudo apt install \
    libdisplay-info-dev \
    libudev-dev \
    libseat-dev \
    libinput-dev \
    libxkbcommon-dev \
    libgbm-dev \
    libdrm-dev \
    libwayland-dev \
    libegl1-mesa-dev \
    libgl1-mesa-dev \
    pkg-config
```

### Compile

```bash
cargo build --release
```

### Run

From a TTY (without any display server running):

```bash
./target/release/loom-wm
```

For development (nested in existing X11/Wayland session):

```bash
# Winit backend will be auto-detected
cargo run
```

## Configuration

Configuration is stored in `~/.config/loom-wm/config.toml`:

```toml
[general]
terminal = "foot"
debug = false

[canvas]
initial_zoom = 1.0
show_grid = true
grid_spacing = 50.0

[ai]
enabled = true
# API key can also be set via LOOM_AI_API_KEY environment variable
# api_key = "your-key-here"

[[keybindings]]
key = "Super+Return"
action = { type = "terminal" }

[[keybindings]]
key = "Super+Space"
action = { type = "ai_prompt" }

[[keybindings]]
key = "Super+Shift+Q"
action = { type = "quit" }
```

## Security

LoomWM is designed with security as a priority:

- **No arbitrary command execution**: Only allowlisted apps and validated scripts
- **Path traversal protection**: Config files are validated against allowed directories
- **API key protection**: Keys are redacted from logs, environment variables preferred
- **Resource limits**: Prevents DoS via node/connection limits
- **Input validation**: All user input is sanitized

## Contributing

Contributions are welcome! Please read the following before submitting:

1. This project uses **Rust 2024 edition**
2. Run `cargo fmt` and `cargo clippy` before committing
3. Add tests for security-sensitive code
4. Follow the existing code style

## License

LoomWM is free software: you can redistribute it and/or modify it under the terms of the **GNU General Public License** as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

See [LICENSE](LICENSE) for the full license text.

## Acknowledgments

- [Smithay](https://github.com/Smithay/smithay) - The Wayland compositor library that makes this possible
- The Wayland and Linux graphics communities
- Research from Nielsen Norman Group on future UI paradigms

---

*LoomWM - Teixint la teva intenció digital.*
