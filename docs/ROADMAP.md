# LoomWM Roadmap

> Current Phase: **Phase 0 - Infrastructure** ✅

## What is LoomWM?

A next-generation Wayland compositor that replaces the window paradigm with an **infinite canvas** of interconnected **nodes**, powered by AI.

---

## Current Project Status

### ✅ Completed

#### Base Infrastructure
- [x] Cargo workspace with 5 crates
- [x] Rust 2024 edition configuration
- [x] Dependency system (Smithay 0.7, wayland-server 0.31)
- [x] Compiles without errors or warnings

#### Security (Security by Default)
- [x] Path traversal protection in configuration
- [x] Automatic API key redaction in logs
- [x] Resource limits (nodes, connections, coordinates)
- [x] Safe command execution (application allowlist)
- [x] Script name validation
- [x] Safe UTF-8 truncation (no panics)
- [x] cargo-audit with no vulnerabilities
- [x] deny.toml configured

#### CI/CD
- [x] GitHub Actions for CI (check, test, fmt, clippy)
- [x] GitHub Actions for security (audit, deny, secrets, SAST)
- [x] Complete .gitignore

#### Documentation
- [x] README.md with project vision
- [x] GPL-3.0-or-later license
- [x] This ROADMAP.md

---

## Crate Architecture

```
loom-wm (binary)
├── loom-core      # Main compositor, backends, global state
├── loom-canvas    # Infinite canvas, nodes, connections, viewport
├── loom-protocol  # Custom Wayland extensions
├── loom-ai        # AI integration, intent parsing, generative UI
└── loom-config    # Configuration, keybindings, themes
```

### Status by Crate

| Crate | Status | Functionality |
|-------|--------|---------------|
| `loom-core` | 🟡 Skeleton | Basic structures, no real functionality |
| `loom-canvas` | 🟡 Skeleton | Node/Canvas structures, no rendering |
| `loom-protocol` | 🟡 Skeleton | Definitions only, no Wayland protocol |
| `loom-ai` | 🟡 Skeleton | Intent parsing mock, no real AI |
| `loom-config` | 🟢 Functional | Config loading, keybindings, secure validation |

**Legend:** 🔴 Not started | 🟡 Skeleton/WIP | 🟢 Functional | ✅ Complete

---

## Development Phases

### Phase 0: Infrastructure ✅
- [x] Project structure
- [x] Dependencies
- [x] Base security
- [x] CI/CD

### Phase 1: Minimum Viable Compositor 🔴
> Goal: A compositor that can display a window

- [ ] **DRM/KMS Backend**
  - [ ] GPU device initialization
  - [ ] Mode setting (resolution, refresh rate)
  - [ ] Buffer management (GBM)
  - [ ] VSync and page flipping

- [ ] **Winit Backend** (for development)
  - [ ] Debug window inside existing X11/Wayland
  - [ ] Basic rendering

- [ ] **Smithay Integration**
  - [ ] CompositorState
  - [ ] XdgShellState
  - [ ] SeatState (input)
  - [ ] OutputState

- [ ] **Event Loop**
  - [ ] Calloop integration
  - [ ] Wayland socket
  - [ ] Input events (libinput)
  - [ ] Timer events

- [ ] **Basic Rendering**
  - [ ] Clear screen with color
  - [ ] Render client surface
  - [ ] Basic damage tracking

### Phase 2: Infinite Canvas 🔴
> Goal: Replace windows with nodes on a canvas

- [ ] **Node System**
  - [ ] Create/destroy nodes
  - [ ] Associate Wayland surfaces with nodes
  - [ ] Transformations (position, scale, rotation)

- [ ] **Viewport**
  - [ ] Pan (drag canvas)
  - [ ] Zoom (scroll + key)
  - [ ] Limits and bounds

- [ ] **Connections Between Nodes**
  - [ ] Data model for connections
  - [ ] Line/curve rendering
  - [ ] Interaction (create/delete connections)

- [ ] **Navigation**
  - [ ] Minimap
  - [ ] Go-to node
  - [ ] Position history

### Phase 3: Extended Wayland Protocol 🔴
> Goal: Clients can interact with the canvas

- [ ] **loom_canvas_v1 Protocol**
  - [ ] XML protocol definition
  - [ ] Code generation with wayland-scanner
  - [ ] Server implementation

- [ ] **Protocol Operations**
  - [ ] get_node_info
  - [ ] set_node_position
  - [ ] create_connection
  - [ ] subscribe_to_events

### Phase 4: AI Integration 🔴
> Goal: AI understands and executes user intents

- [ ] **Intent Parsing**
  - [ ] LLM integration (local or API)
  - [ ] Natural command parsing
  - [ ] Context awareness (active nodes, history)

- [ ] **AI Actions**
  - [ ] Automatically organize nodes
  - [ ] Suggest connections
  - [ ] Search the canvas

- [ ] **Generative UI**
  - [ ] Generate layouts based on intent
  - [ ] Adapt UI to context

### Phase 5: Polish and Advanced Features 🔴
- [ ] Themes and visual customization
- [ ] Smooth animations
- [ ] Multi-touch gestures
- [ ] Plugins/extensions
- [ ] State persistence (save/load canvas)
- [ ] Multi-monitor
- [ ] XWayland (X11 compatibility)

---

## Immediate Next Steps

### Current Sprint: Minimum Compositor

1. **Configure Winit backend** - To develop without restarting session
2. **Implement event loop with Calloop** - Compositor core
3. **Integrate Smithay delegates** - CompositorHandler, XdgShellHandler
4. **Render first frame** - Clear screen with a color

---

## Technical Notes

### Why Smithay?
- Pure Rust library for Wayland compositors
- Modular: only use what you need
- Well maintained and documented
- Used by real projects (cosmic-comp, etc.)

### Why Infinite Canvas?
- Windows are an 80s metaphor
- Canvas allows spatial relationships between content
- More natural for modern workflows
- Enables semantic zoom (see more or less detail)

### Why AI?
- Intent-based interfaces > action-based interfaces
- AI can understand "I want to see the code and documentation together"
- Intelligent automation of repetitive tasks

---

*Last updated: 2025-12-24*
