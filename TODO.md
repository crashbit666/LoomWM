# LoomWM - TODO

> Sistema de seguiment de tasques del projecte

## 🎯 Sprint Actual: Compositor Mínim Viable

### En Progrés 🔄

_Cap tasca en progrés actualment_

### Pendent 📋

#### Alta Prioritat
- [ ] Implementar backend Winit per desenvolupament
- [ ] Configurar event loop amb Calloop
- [ ] Crear CompositorHandler (Smithay delegate)
- [ ] Crear XdgShellHandler per gestionar surfaces
- [ ] Renderitzar primer frame (clear screen)

#### Mitjana Prioritat
- [ ] Afegir SeatState per input
- [ ] Implementar keyboard handling bàsic
- [ ] Afegir pointer/mouse handling
- [ ] Crear OutputState per gestió de pantalles

#### Baixa Prioritat
- [ ] Documentar arquitectura interna
- [ ] Afegir més tests unitaris
- [ ] Configurar benchmarks

---

## ✅ Completat

### 2024-12-24
- [x] Crear estructura de workspace Cargo
- [x] Configurar dependències (Smithay 0.7)
- [x] Implementar seguretat base (path traversal, API keys)
- [x] Configurar CI/CD (GitHub Actions)
- [x] Crear README.md i documentació
- [x] Configurar cargo-audit i cargo-deny
- [x] Netejar warnings (clippy, fmt)
- [x] Crear ROADMAP.md

---

## 🐛 Bugs Coneguts

_Cap bug conegut actualment_

---

## 💡 Idees i Millores Futures

- [ ] Mode "focus" que aïlla un node i difumina la resta
- [ ] Connexions amb tipus (dades, referència, temporal)
- [ ] Timeline per veure l'evolució del canvas
- [ ] Col·laboració en temps real (multi-usuari)
- [ ] Integració amb eines de desenvolupament (LSP, debugger)
- [ ] Voice commands per navegació
- [ ] Haptic feedback per dispositius compatibles

---

## 📊 Mètriques

| Mètrica | Valor |
|---------|-------|
| Línies de codi (src) | 1.608 |
| Tests | 7 |
| Crates | 5 |
| Dependències directes | ~15 |
| Vulnerabilitats (cargo audit) | 0 |

---

## 🗓️ Historial de Sprints

### Sprint 0: Infraestructura (Completat)
- Durada: 1 dia
- Objectiu: Estructura del projecte i seguretat base
- Resultat: ✅ Completat

### Sprint 1: Compositor Mínim (Actual)
- Objectiu: Mostrar una finestra i renderitzar un client
- Resultat: 🔄 En progrés

---

*Actualitzat: 2025-12-24*
