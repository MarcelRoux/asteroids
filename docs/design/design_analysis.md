# Design analysis

This document captures the core architectural direction of the project: a bounded-cost real-time simulation with user-configurable policies (toggles) that can be evaluated reproducibly (AI autopilot + soak tests).

## Design principles

1. **Bounded work per frame**: every system enforces caps and avoids unbounded recursion.
2. **Policies over branches**: feature differences are expressed as swappable strategies, not conditionals scattered through gameplay code.
3. **Controller abstraction**: player control (Human / AI / future network) is expressed via `ControlIntent`.
4. **Evaluation-first**: the project is instrumented from the beginning (HUD + soak tests).
5. **Engine-light**: macroquad is used for rendering/input; simulation logic is implemented in project code.

## Configuration spine

A single configuration structure selects policies at startup (and optionally mid-run where safe).

```rust
struct GameConfig {
    // Core gameplay
    difficulty: DifficultyPreset,              // Classic | Horde | Custom
    leaderboard_mode: LeaderboardMode,         // Off | LocalTop10

    // Player control
    player_controller: PlayerControllerMode,   // Human | AI(profile)

    // Systems toggles
    upgrades_enabled: bool,
    physics_mode: PhysicsMode,                 // Off | Arcade | Lite
    fragmentation_mode: FragmentationMode,     // Off | ClassicSplit | SliceOnly | Explode | Full
    collision_policy: CollisionPolicy,         // PlayerOnly | BigOnly | Full
}
```

Each optional feature is implemented as one of:

- a no-op system,
- a strategy behind a trait,
- a subsystem omitted from the update schedule.

## Roadmap summary

The project progresses from a baseline reference game to increasingly expressive policies, while maintaining bounded performance.

### Epic 0 — Engine scaffold

Deliver:

- fixed timestep loop
- entity store
- basic collisions
- HUD instrumentation

Expose early toggles:

- `physics_mode`
- `leaderboard_mode`
- `player_controller`

### Epic 1 — Classic Asteroids baseline

Deliver:

- ship controls, bullets
- asteroid spawning + classic split
- score + local leaderboard
- menu: Play / Options / Leaderboard

### Epic 1A — Toggleable AI autopilot

Deliver:

- `Controller` abstraction producing `ControlIntent`
- `WorldSnapshot` visibility limits
- AI constraints (reaction cadence, aim noise, commitment windows, fire gating)
- Options toggle: Human vs AI (profile)
- HUD instrumentation for AI

Rationale:

- enables automated soak tests and benchmarking immediately after the baseline

### Epic 2+ (later)

- Upgrades and progression
- Convex polygon slicing and bounded fragmentation
- Enemy archetypes + carriers + sentinels
- Physics ladder and collision policies

## Performance contracts

The architecture assumes explicit performance contracts that are enforced centrally via budgets:

- maximum active bodies (`MAX_BODIES`)
- maximum fragments per event (`FRAG_EVENT_CAP`)
- polygon complexity caps (`V_MAX`)
- collision policy constraints
- debris TTL as a degradation lever

Collision policy is the primary scaling lever:

- `PlayerOnly`: maximum scalability
- `BigOnly`: bounded fragment-fragment interactions
- `Full`: expensive; requires tighter caps and/or guardrails

## Progression model

Progression is content-driven and can be layered over the same core policies:

- unlock weapons (laser slice, drill explosion)
- unlock ship upgrades
- unlock advanced presets (Simulation-ish / Experimental) with clear warnings

Replace all instances of ‘^^^’ with ‘```’ for proper markdown rendering.
