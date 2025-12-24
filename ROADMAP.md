# LoomWM Roadmap

> Estat actual: **Fase 0 - Infraestructura** ✅

## Què és LoomWM?

Un compositor Wayland de nova generació que substitueix el paradigma de finestres per un **canvas infinit** amb **nodes** interconnectats, impulsat per IA.

---

## Estat Actual del Projecte

### ✅ Completat

#### Infraestructura Base
- [x] Workspace Cargo amb 5 crates
- [x] Configuració Rust 2024 edition
- [x] Sistema de dependències (Smithay 0.7, wayland-server 0.31)
- [x] Compila sense errors ni warnings

#### Seguretat (Security by Default)
- [x] Protecció contra path traversal en configuració
- [x] Redacció automàtica de claus API en logs
- [x] Límits de recursos (nodes, connexions, coordenades)
- [x] Execució segura de comandes (allowlist d'aplicacions)
- [x] Validació de noms de scripts
- [x] Truncat UTF-8 segur (sense panics)
- [x] cargo-audit sense vulnerabilitats
- [x] deny.toml configurat

#### CI/CD
- [x] GitHub Actions per CI (check, test, fmt, clippy)
- [x] GitHub Actions per seguretat (audit, deny, secrets, SAST)
- [x] .gitignore complet

#### Documentació
- [x] README.md amb visió del projecte
- [x] Llicència GPL-3.0-or-later
- [x] Aquest ROADMAP.md

---

## Arquitectura de Crates

```
loom-wm (binary)
├── loom-core      # Compositor principal, backends, estat global
├── loom-canvas    # Canvas infinit, nodes, connexions, viewport
├── loom-protocol  # Extensions Wayland personalitzades
├── loom-ai        # Integració IA, parsing d'intent, UI generativa
└── loom-config    # Configuració, keybindings, temes
```

### Estat per Crate

| Crate | Estat | Funcionalitat |
|-------|-------|---------------|
| `loom-core` | 🟡 Esquelet | Estructures bàsiques, sense funcionalitat real |
| `loom-canvas` | 🟡 Esquelet | Estructures Node/Canvas, sense renderitzat |
| `loom-protocol` | 🟡 Esquelet | Només definicions, sense protocol Wayland |
| `loom-ai` | 🟡 Esquelet | Mock d'intent parsing, sense IA real |
| `loom-config` | 🟢 Funcional | Càrrega de config, keybindings, validació segura |

**Llegenda:** 🔴 No començat | 🟡 Esquelet/WIP | 🟢 Funcional | ✅ Complet

---

## Fases de Desenvolupament

### Fase 0: Infraestructura ✅
- [x] Estructura del projecte
- [x] Dependències
- [x] Seguretat base
- [x] CI/CD

### Fase 1: Compositor Mínim Viable 🔴
> Objectiu: Un compositor que pugui mostrar una finestra

- [ ] **Backend DRM/KMS**
  - [ ] Inicialització de dispositiu GPU
  - [ ] Mode setting (resolució, refresh rate)
  - [ ] Gestió de buffers (GBM)
  - [ ] VSync i page flipping

- [ ] **Backend Winit** (per desenvolupament)
  - [ ] Finestra de debug dins X11/Wayland existent
  - [ ] Renderitzat bàsic

- [ ] **Integració Smithay**
  - [ ] CompositorState
  - [ ] XdgShellState
  - [ ] SeatState (input)
  - [ ] OutputState

- [ ] **Event Loop**
  - [ ] Calloop integration
  - [ ] Wayland socket
  - [ ] Input events (libinput)
  - [ ] Timer events

- [ ] **Renderitzat Bàsic**
  - [ ] Clear screen amb color
  - [ ] Renderitzar surface d'un client
  - [ ] Damage tracking bàsic

### Fase 2: Canvas Infinit 🔴
> Objectiu: Substituir finestres per nodes en un canvas

- [ ] **Sistema de Nodes**
  - [ ] Crear/destruir nodes
  - [ ] Associar surfaces Wayland a nodes
  - [ ] Transformacions (posició, escala, rotació)

- [ ] **Viewport**
  - [ ] Pan (arrossegar canvas)
  - [ ] Zoom (scroll + tecla)
  - [ ] Límits i bounds

- [ ] **Connexions entre Nodes**
  - [ ] Model de dades per connexions
  - [ ] Renderitzat de línies/corbes
  - [ ] Interacció (crear/eliminar connexions)

- [ ] **Navegació**
  - [ ] Minimap
  - [ ] Go-to node
  - [ ] Historial de posicions

### Fase 3: Protocol Wayland Estès 🔴
> Objectiu: Clients poden interactuar amb el canvas

- [ ] **Protocol loom_canvas_v1**
  - [ ] Definició XML del protocol
  - [ ] Generació de codi amb wayland-scanner
  - [ ] Implementació servidor

- [ ] **Operacions del Protocol**
  - [ ] get_node_info
  - [ ] set_node_position
  - [ ] create_connection
  - [ ] subscribe_to_events

### Fase 4: Integració IA 🔴
> Objectiu: La IA entén i executa intents de l'usuari

- [ ] **Intent Parsing**
  - [ ] Integració amb LLM (local o API)
  - [ ] Parsing de comandes naturals
  - [ ] Context awareness (nodes actius, historial)

- [ ] **Accions IA**
  - [ ] Organitzar nodes automàticament
  - [ ] Suggerir connexions
  - [ ] Cercar en el canvas

- [ ] **UI Generativa**
  - [ ] Generar layouts basats en intent
  - [ ] Adaptar UI al context

### Fase 5: Poliment i Features Avançades 🔴
- [ ] Temes i personalització visual
- [ ] Animacions fluides
- [ ] Gestures multi-touch
- [ ] Plugins/extensions
- [ ] Persistència d'estat (guardar/carregar canvas)
- [ ] Multi-monitor
- [ ] XWayland (compatibilitat X11)

---

## Pròxims Passos Immediats

### Sprint Actual: Compositor Mínim

1. **Configurar backend Winit** - Per poder desenvolupar sense reiniciar sessió
2. **Implementar event loop amb Calloop** - Core del compositor
3. **Integrar Smithay delegates** - CompositorHandler, XdgShellHandler
4. **Renderitzar primer frame** - Clear screen amb un color

---

## Com Contribuir

```bash
# Clonar
git clone https://github.com/USER/loomWM.git
cd loomWM

# Dependències (Fedora)
sudo dnf install libxkbcommon-devel libudev-devel libseat-devel \
    libinput-devel libgbm-devel libdrm-devel wayland-devel \
    mesa-libEGL-devel mesa-libGL-devel

# Compilar
cargo build

# Tests
cargo test

# Executar (quan estigui funcional)
cargo run
```

---

## Notes Tècniques

### Per què Smithay?
- Biblioteca Rust pura per compositors Wayland
- Modular: només uses el que necessites
- Ben mantinguda i documentada
- Usada per projectes reals (cosmic-comp, etc.)

### Per què Canvas Infinit?
- Les finestres són una metàfora dels 80s
- El canvas permet relacions espacials entre contingut
- Més natural per fluxos de treball moderns
- Permet zoom semàntic (veure més o menys detall)

### Per què IA?
- Interfícies basades en intent > interfícies basades en accions
- La IA pot entendre "vull veure el codi i la documentació junts"
- Automatització intel·ligent de tasques repetitives

---

*Última actualització: 2025-12-24*
