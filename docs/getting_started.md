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

### Run with a preset

```bash
cargo run -- --preset classic
cargo run -- --preset ai_autopilot
```

### Run with explicit policy overrides

```bash
cargo run -- \
  --preset classic \
  --player-controller ai \
  --ai-profile balanced \
  --leaderboard local \
  --physics arcade \
  --collision player_only
```

---

## Control modes

### Human controller (default)

- Rotate: `A` / `D` (or `←` / `→`)
- Thrust: `W` (or `↑`)
- Fire primary: `Space`
- Pause: `Esc`
- Options: `O`

### AI controller

- Selectable via Options: `Player Controller = AI`
- Or via CLI: `--player-controller ai --ai-profile balanced`

The AI controller is intentionally constrained (reaction delay, noise, limited awareness) to remain player-like and suitable for evaluation.

---

## Presets

Presets map gameplay intent to coherent policy configurations and performance budgets.

- `classic` — faithful baseline, minimal policies, local leaderboard
- `arcade_upgrades` — baseline with progression systems enabled
- `ai_autopilot` — baseline with AI controlling the ship (evaluation preset)
- `fracture` — convex polygon slicing + lite physics
- `horde` — dense spawns, bounded fragmentation, `BigOnly` collisions
- `simulation` — higher physical fidelity (warned, guarded)
- `custom` — exposes all toggles and budget knobs

See [Settings & Presets Matrix](design/settings_and_presets_matrix.md)

---

## Feature toggles (policies)

All optional features are implemented as runtime policies rather than compile-time forks.

- Player controller: `Human | AI(profile)`
- Physics mode: `Off | Arcade | Lite`
- Collision policy: `PlayerOnly | BigOnly | Full`
- Fragmentation mode: `Off | ClassicSplit | SliceOnly | Explode | Full`
- Upgrades: enabled / disabled
- Performance Guard: enabled / disabled

See [Design Analysis](design/design_analysis.md)

---

## Early milestone

The first delivery target is **Epic 1 + Epic 1A**:

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
