# Asteroids — Systems-Oriented Rust Game

This project is a **systems-focused Asteroids-style game** implemented in Rust, designed to demonstrate:

- bounded-cost real-time simulation,
- explicit performance ownership,
- policy-driven feature toggles (physics, collision, fragmentation), and
- evaluation-first tooling (AI autopilot, soak tests, HUD instrumentation).

The game is intentionally engine-light (macroquad) so that performance, correctness, and tradeoffs are visible in project code rather than hidden behind a framework.

---

## Quick start

### Prerequisites

- Rust (stable toolchain via rustup)
- Working GPU driver / graphics stack (macOS / Windows / Linux)

### Run

```bash
cargo run
```

### Planned: CLI presets and toggles

This runbook previously referenced `--preset` and other CLI flags. Those are planned but not implemented yet.
Today, presets and toggles are changed in-game from the Options screen (see Controls below).

---

## Control modes

### Gameplay (human or autopilot)

- Rotate: `A` / `D` (or `←` / `→`)
- Thrust: `W` (or `↑`)
- Fire primary: `Space`
- Fire secondary: `Shift`
- Pause / resume: `P`
- End run (back to main menu): `Esc`
- Toggle autopilot: `U`
- Toggle stats overlay: `T`
- Toggle invulnerability (debug): `I`

### AI controller

Autopilot can be toggled at runtime with `U`. When autopilot is enabled, `P` cycles AI profile.
Planned: wiring presets and CLI to configure AI controller selection directly.

The AI controller is intentionally constrained (reaction delay, noise, limited awareness) to remain player-like and suitable for evaluation.

---

## Presets

Presets map gameplay intent to coherent policy configurations and performance budgets.

- Implemented (in-game options via `Y`): `classic`, `arcade_upgrades`
- Planned / partial: `ai_autopilot` (config exists; autopilot engagement wiring is in-progress)
- Planned: `fracture`, `horde`, `simulation`, `custom`

See [Settings & Presets Matrix](design/settings_and_presets_matrix.md)

---

## Feature toggles (policies)

Optional features are intended to be implemented as runtime policies rather than compile-time forks. Some are still planned.

- Player controller: `Human | AI(profile)`
- Physics mode: `Off | Arcade | Lite`
- Collision policy: `PlayerOnly | BigOnly | Full`
- Fragmentation mode: `Off | ClassicSplit | SliceOnly | Explode | Full`
- Upgrades: enabled / disabled
- Performance Guard toggle: planned (guard exists; toggle is not exposed yet)

See [Design Analysis](design/design_analysis.md)

---

## Menus and hotkeys

- Main menu: `P` start, `O` options, `L` leaderboard, `Esc` quit
- Options: `Y` cycle presets, `C` collision, `K` physics, `F` fragmentation, `L` leaderboard mode, `G` upgrades, `Enter`/`Esc` back
- Game over: type name, `Backspace` delete, `Enter` submit, `Esc` cancel

---

## Early milestone

Epics 1, 1A, and 1B are considered complete:

- Classic Asteroids baseline (score + local leaderboard)
- Toggleable AI autopilot that plays in a player-like manner
- HUD instrumentation and a 10-minute unattended soak test

Development steps for this milestone are documented in this runbook.

---

## Documentation map

### Architectural decisions

- [ADR 0000: Engine choice](adrs/ADR_0000_engine_choice_macroquad_over_bevy.md)

### Core design

- [Design analysis](design/design_analysis.md)
- [Recommended epics](design/recommended_epics.md)
- [Settings & presets matrix](design/settings_and_presets_matrix.md)

### AI design

- [AI 0001: Player controller architecture (Human vs AI)](ai/AI_0001_player_controller_architecture_(human_vs_ai).md)
- [AI 0002: Threat and target scoring](ai/AI_0002_thread_and_target_scoring.md)
- [AI 0003: Profiles and behavior parameters](ai/AI_0003_profiles_and_behaviour_parameters.md)
- [AI 0004: Integration and interfaces](ai/AI_0004_integration_and_interfaces.md)

### Evaluation and simulation contracts

- [EVAL-0001: Evaluation-first design and performance philosophy](eval/EVAL_0001_evaluation_and_performance_philosophy.md)
- [SIM-0001: Simulation boundary and determinism contract](sim/SIM_0001_simulation_boundary.md)

---

## License

Apache-2.0
