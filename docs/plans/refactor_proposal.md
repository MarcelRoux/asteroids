# Refactor Proposal: App/Simulation/UI Modularization

## Objective

Reduce cognitive load and improve iteration speed by separating orchestration, game simulation logic, and presentation concerns. The immediate targets are large modules:

- `src/main.rs`
- `src/simulation/mod.rs`
- `src/ui/mod.rs`

This proposal preserves current behavior while creating explicit seams for future features.

## Current Observations

### 1. `main.rs` mixes too many responsibilities

Current file includes:

- app state machine
- input handling
- simulation ticking
- rendering calls
- game-over name entry flow
- run/session lifecycle helpers
- autopilot profile switching

This creates high coupling and makes feature work (new states, input changes, replay modes) risky.

### 2. `simulation/mod.rs` is a god module

Current file includes:

- all domain data models (`Ship`, `Asteroid`, `Alien`, `Bullet`, etc.)
- state container (`Simulation`)
- update loop orchestration (`step`)
- multiple systems (movement/spawn/combat/collision)
- score/lives progression
- rendering logic (`draw_debug`, thruster visuals)
- geometry/math utilities
- policy/status types

This blocks focused testing and increases merge conflicts.

### 3. `ui/mod.rs` is a single large inline menu module

UI concerns are grouped together in one inline module:

- HUD overlays
- menu screens
- leaderboard view
- game-over view
- shared widget helpers

This is workable now, but scaling UI variants will quickly become noisy.

## Target Architecture

## Layer boundaries

Dependency direction should be:

- `app -> simulation`
- `app -> ui`
- `app -> config/controllers/eval/scoreboard`

`simulation` must not depend on `ui`.

### Proposed module layout

```text
src/
  main.rs
  app/
    mod.rs
    state.rs
    session.rs
  simulation/
    mod.rs
    model.rs
    math.rs
    render.rs
    systems/
      mod.rs
      movement.rs
      spawn.rs
      combat.rs
      collision.rs
  ui/
    mod.rs
    hud.rs
    screens.rs
    widgets.rs
```

## Responsibilities by module

### `src/main.rs`

- bootstrap only (window/runtime setup)
- construct `App`
- call `app.tick()` each frame

No feature logic should remain here.

### `src/app/*`

- owns app-level state machine and transitions
- coordinates simulation, UI, leaderboard, perf guard, options state
- owns run lifecycle and game-over entry flow
- handles per-screen input mapping

### `src/simulation/*`

- `model.rs`: core entities/enums/internal world representation
- `systems/*`: pure-ish logic grouped by concern
- `render.rs`: simulation visualization only
- `math.rs`: wrap/clamp/geometry helper functions
- `mod.rs`: public API surface (`Simulation`, `SimulationStatus`, `SimulationPolicy`)

### `src/ui/*`

- `hud.rs`: score/stats/autopilot overlays
- `screens.rs`: full-screen menu/state screens
- `widgets.rs`: generic drawing primitives (`draw_menu_box`, shared text formatting)
- `mod.rs`: re-export facade

## Phase Plan (low-risk order)

## Phase 1: Extract app/session shell

- add `src/app/state.rs` with `AppState`
- add `src/app/session.rs` with helpers currently at bottom of `main.rs`:
  - run reset/finish
  - controller selection for human/autopilot
  - autopilot profile cycling
  - run-name sanitize/default helpers
- add `src/app/mod.rs` with `App` struct and `tick()`
- reduce `main.rs` to thin bootstrap

Expected outcome: orchestration leaves `main.rs` without behavior changes.

## Phase 2: Split simulation data model

- move private types/enums from `simulation/mod.rs` into `simulation/model.rs`
- keep type visibility minimal (`pub(super)` where needed)
- keep `Simulation` behavior unchanged in this phase

Expected outcome: easier navigation and reduced surface area in `simulation/mod.rs`.

## Phase 3: Extract simulation systems

Move methods into focused files:

- movement: `update_ship`, `update_asteroids`, wrap/clamp interactions
- spawn: asteroid/alien spawning
- combat: firing, bullet/debris updates
- collision: hit detection, score/lives adjustments, fragmentation, debris spawn triggers

Keep `Simulation::step()` as a short orchestration method.

Expected outcome: core logic is testable per subsystem.

## Phase 4: Isolate rendering from simulation logic

- move `draw_debug`, `draw_thruster`, shape draw usage to `simulation/render.rs`
- keep rendering optional/side-effecting and separate from rules

Expected outcome: future headless simulation or benchmark mode is straightforward.

## Phase 5: Split UI into focused modules

- move overlays to `ui/hud.rs`
- move menu/game-over/leaderboard screens to `ui/screens.rs`
- keep shared widgets in `ui/widgets.rs`
- maintain stable call sites through re-exports in `ui/mod.rs`

Expected outcome: UI additions do not crowd one file.

## Guardrails

- No behavior changes during structural phases (1-5).
- Preserve save format compatibility for leaderboard data.
- Keep public API stable unless explicitly versioned.
- Prefer incremental PRs with compile-green checkpoints after each phase.

## Testing Strategy

Before refactor:

- run full test suite and capture baseline
- optionally record simple runtime smoke checks (menu -> play -> game over -> leaderboard)

During refactor:

- ensure each phase compiles and tests pass
- add focused unit tests around extracted collision/scoring logic
- avoid large mixed “move + behavior change” commits

After refactor:

- rerun baseline smoke path
- verify leaderboard read/write unchanged
- verify autopilot toggle/profile cycle behavior unchanged

## Suggested Initial Deliverable

First implementation slice should complete Phase 1 + Phase 2 only. This yields immediate file-size reductions in `main.rs` and `simulation/mod.rs` with minimal risk and creates clean seams for subsequent phases.
