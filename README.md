# Asteroids

Rust Asteroids — deterministic baseline + toggleable AI autopilot, designed as a portfolio-grade systems project.

This repo is built around **bounded-cost simulation**, **policy toggles** (physics/collision/fragmentation), and **repeatable evaluation** (AI autopilot + soak tests + HUD instrumentation).

## Quick start

### Prerequisites

- Rust stable (via rustup)
- A working GPU driver / graphics stack (macOS/Windows/Linux)

### Run

```bash
cargo run
```

### Run with a specific preset (recommended)

```bash
cargo run -- --preset classic
cargo run -- --preset ai_autopilot
```

### Run with explicit toggles (override preset)

```bash
cargo run -- \
  --preset classic \
  --player-controller ai \
  --ai-profile balanced \
  --leaderboard local \
  --physics arcade \
  --collision player_only
```

## Controls

### Human controller (default)

- Rotate left/right: `A` / `D` (or `←` / `→`)
- Thrust: `W` (or `↑`)
- Fire primary: `Space`
- Pause: `Esc`
- Options: `O`

### AI controller

- Enable in Options: `Player Controller = AI`
- Or via CLI: `--player-controller ai --ai-profile balanced`

## Presets (high-level)

- `classic` — faithful baseline, minimal policies, local leaderboard
- `arcade_upgrades` — baseline with upgrades enabled
- `ai_autopilot` — baseline with AI controlling the ship (evaluation bot)
- `fracture` — polygon slicing + lite physics
- `horde` — dense spawns, bounded fragmentation, BigOnly collisions
- `simulation` — warned mode (Full collisions), guard enabled
- `custom` — exposes all toggles and budget knobs

See [Settings & Presets Matrix](docs/design/settings_and_presets_matrix.md)

## Key toggles (user levers)

Toggles are treated as **policies**, not compile-time forks.

- Player controller: `Human | AI(profile)`
- AI profile: `Casual | Balanced | Veteran`
- Physics mode: `Off | Arcade | Lite`
- Collision policy: `PlayerOnly | BigOnly | Full`
- Fragmentation mode: `Off | ClassicSplit | SliceOnly | Explode | Full`
- Upgrades: enabled/disabled
- Performance Guard: enabled/disabled

See [Design Analysis](docs/design/design_analysis.md)

## Milestones (fast path)

For the early-stage target milestone, follow:

See [Getting Started](docs/getting_started.md)

The first goal is **Epic 1 + Epic 1A**:

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
