# Asteroids

Rust Asteroids — determinism-targeting baseline + toggleable AI autopilot, designed as a portfolio-grade systems project.

This repo is built around **bounded-cost simulation**, **policy toggles** (physics/collision/fragmentation), and **repeatable evaluation** (AI autopilot + soak tests + HUD instrumentation).

## Quick start

### Prerequisites

- Rust stable (via rustup)
- A working GPU driver / graphics stack (macOS/Windows/Linux)

### Run

```bash
cargo run
```

### Planned: CLI presets and toggles

This README previously documented `--preset` and other CLI flags. Those are **planned** but not implemented yet.
Today, presets and toggles are changed in-game from the Options screen (see Controls).

## Controls

### Gameplay

- Rotate left/right: `A` / `D` (or `←` / `→`)
- Thrust: `W` (or `↑`)
- Fire primary: `Space`
- Fire secondary: `Shift`
- Pause / resume: `P`
- End run (back to main menu): `Esc`
- Toggle autopilot: `U`
- Toggle stats overlay: `T`
- Toggle invulnerability (debug): `I`

### Main menu

- Start run: `P`
- Options: `O`
- Leaderboard: `L`
- Quit: `Esc`

### Options

- Cycle presets: `Y`
- Cycle collision policy: `C`
- Cycle physics mode: `K`
- Cycle fragmentation mode: `F`
- Toggle leaderboard mode: `L`
- Toggle upgrades: `G`
- Back: `Enter` / `Esc`

### AI controller

AI autopilot can be toggled at runtime with `U`. When autopilot is enabled, `P` cycles AI profile.
Planned: wiring the "AI Autopilot" preset to auto-enable autopilot, and enabling CLI flags for presets/toggles.

## Presets (high-level)

- Implemented (in-game options via `Y`): `classic`, `arcade_upgrades`
- Planned / partial: `ai_autopilot` (config exists; autopilot engagement wiring is in-progress)
- Planned: `fracture`, `horde`, `simulation`, `custom`

See [Settings & Presets Matrix](docs/design/settings_and_presets_matrix.md)

## Key toggles (user levers)

Toggles are treated as **policies**, not compile-time forks.

- Player controller: `Human | AI(profile)`
- AI profile: `Casual | Balanced | Veteran`
- Physics mode: `Off | Arcade | Lite`
- Collision policy: `PlayerOnly | BigOnly | Full`
- Fragmentation mode: `Off | ClassicSplit | SliceOnly | Explode | Full`
- Upgrades: enabled/disabled
- Performance Guard toggle: planned (guard exists; toggle is not exposed yet)

See [Design Analysis](docs/design/design_analysis.md)

## Milestones (fast path)

For the early-stage target milestone, follow:

See [Getting Started](docs/getting_started.md)

Epics **1**, **1A**, and **1B** are considered complete:

- Classic Asteroids playable (score + leaderboard)
- AI autopilot toggle plays in a player-like way (no aimbot)
- HUD instrumentation + 10-minute soak test

See [Recommended Epics](docs/design/recommended_epics.md)

## Documentation map

- ADRs:
  - [ADR 0000: Engine choice](docs/adrs/ADR_0000_engine_choice_macroquad_over_bevy.md)
- Epics + acceptance criteria: [Recommended Epics](docs/design/recommended_epics.md)
- Settings/presets + budgets: [Settings & Presets Matrix](docs/design/settings_and_presets_matrix.md)
- AI design:
  - [AI 0001: Player controller architecture (human vs AI)](docs/ai/AI_0001_player_controller_architecture_(human_vs_ai).md)
  - [AI 0002: Thread and target scoring](docs/ai/AI_0002_thread_and_target_scoring.md)
  - [AI 0003: Profiles and behaviour parameters](docs/ai/AI_0003_profiles_and_behaviour_parameters.md)
  - [AI 0004: Integration and interfaces](docs/ai/AI_0004_integration_and_interfaces.md)
- Evaluation + instrumentation:
  - [EVAL-0001: Evaluation-first design and performance philosophy](docs/eval/EVAL_0001_evaluation_and_performance_philosophy.md)
- Simulation boundary:
  - [SIM-0001: Simulation boundary and determinism contract](docs/sim/SIM_0001_simulation_boundary.md)

## License

[Apache](LICENSE)
