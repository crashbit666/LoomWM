# LoomWM - TODO

> Project task tracking system

## 🎯 Current Sprint: Minimum Viable Compositor

### In Progress 🔄

_No tasks currently in progress_

### Pending 📋

#### High Priority
- [ ] Implement Winit backend for development
- [ ] Configure event loop with Calloop
- [ ] Create CompositorHandler (Smithay delegate)
- [ ] Create XdgShellHandler for surface management
- [ ] Render first frame (clear screen)

#### Medium Priority
- [ ] Add SeatState for input
- [ ] Implement basic keyboard handling
- [ ] Add pointer/mouse handling
- [ ] Create OutputState for display management

#### Low Priority
- [ ] Document internal architecture
- [ ] Add more unit tests
- [ ] Configure benchmarks

---

## ✅ Completed

### 2024-12-24
- [x] Create Cargo workspace structure
- [x] Configure dependencies (Smithay 0.7)
- [x] Implement base security (path traversal, API keys)
- [x] Configure CI/CD (GitHub Actions)
- [x] Create README.md and documentation
- [x] Configure cargo-audit and cargo-deny
- [x] Clean warnings (clippy, fmt)
- [x] Create ROADMAP.md

---

## 🐛 Known Bugs

_No known bugs currently_

---

## 💡 Ideas and Future Improvements

- [ ] "Focus" mode that isolates a node and blurs the rest
- [ ] Typed connections (data, reference, temporal)
- [ ] Timeline to see canvas evolution
- [ ] Real-time collaboration (multi-user)
- [ ] Integration with development tools (LSP, debugger)
- [ ] Voice commands for navigation
- [ ] Haptic feedback for compatible devices

---

## 📊 Metrics

| Metric | Value |
|--------|-------|
| Lines of code (src) | 1,608 |
| Tests | 7 |
| Crates | 5 |
| Direct dependencies | ~15 |
| Vulnerabilities (cargo audit) | 0 |

---

## 🗓️ Sprint History

### Sprint 0: Infrastructure (Completed)
- Duration: 1 day
- Goal: Project structure and base security
- Result: ✅ Completed

### Sprint 1: Minimum Compositor (Current)
- Goal: Display a window and render a client
- Result: 🔄 In progress

---

*Updated: 2025-12-24*
