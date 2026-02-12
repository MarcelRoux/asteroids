# AI-0004: Integration and Interfaces

## Status

Proposed

## Context

The AI autopilot must integrate into the game in a way that:

- preserves a single downstream control path (no AI-only gameplay code)
- constrains the AI to avoid omniscience and “aimbot” behavior
- remains performant under horde-like entity counts
- supports early evaluation (HUD + soak tests)

This document defines:

- module boundaries
- required types and interfaces
- data flow and update cadence
- wiring for toggles and presets
- required HUD instrumentation

---

## Decision

Integrate AI as a `Controller` policy producing `ControlIntent`, consuming a restricted `WorldSnapshot`.

The ship simulation consumes only `ControlIntent`; it never reads inputs directly.

---

## Required Modules

Note: The module/file split described below is the target design. The current codebase uses different file paths
post-refactor:

- `src/controllers/mod.rs` defines `ControlIntent` and the `Controller` trait.
- `src/controllers/human.rs` implements the human controller.
- `src/ai/mod.rs` implements the AI controller and defines `WorldSnapshot` (currently in one module).

### controllers/controller.rs

Defines the shared interface.

```rust
pub struct ControlIntent {
    pub thrust: f32,              // 0..1
    pub turn: f32,                // -1..1
    pub fire_primary: bool,
    pub fire_secondary: bool,
    pub deploy_sentinel: bool,
}

pub trait Controller {
    fn tick(&mut self, world: &WorldSnapshot, dt: f32) -> ControlIntent;
}
```

Note: `deploy_sentinel` is reserved for later implementation. The current code's `ControlIntent` does not include it yet.

### controllers/human.rs

Reads input devices and emits `ControlIntent`.

Non-goals:

- no gameplay logic
- no world access

### controllers/ai.rs

Computes `ControlIntent` from `WorldSnapshot` and internal state:

- reaction timing
- target selection + commitment windows
- aim error + fire gating

The AI must be deterministic given:

- world snapshot
- internal RNG seed

---

## WorldSnapshot Contract

### Purpose

Provide the AI enough information to behave player-like, while preventing omniscience and bounding CPU.

### Construction

`WorldSnapshot` is built once per sim tick (or once per AI decision cadence tick) from the authoritative world state.

### Data included (minimum for baseline)

- Ship state:
  - position, velocity
  - facing direction (unit vector) and angular velocity (optional)
- Nearby asteroids (bounded list):
  - id (stable per entity)
  - position, velocity
  - radius (approx)
- Global parameters:
  - dt
  - screen bounds / wrap rules

### Data excluded

- global entity store
- future positions
- hidden/internal flags unrelated to player perception

### Visibility constraints

- Sensor radius: configurable (default 1.2× screen width)
- Attention cap: configurable (default N=10 objects)
- Update cadence: tied to AI reaction timing (5–10 Hz)

### Types

```rust
pub struct WorldSnapshot {
    pub ship: ShipSnapshot,
    pub asteroids: Vec<AsteroidSnapshot>,   // already bounded
    pub screen: ScreenSnapshot,
    pub time: TimeSnapshot,
}

pub struct AsteroidSnapshot {
    pub id: EntityId,
    pub pos: Vec2,
    pub vel: Vec2,
    pub radius: f32,
}
```

Implementation note:

- prefer reusing buffers or smallvec to reduce allocations
- snapshot should be “cheap to build, cheap to read”

---

## Update Cadence and Scheduling

### Simulation tick (e.g. 60 Hz)

Each sim tick:

1) Build `WorldSnapshot` (or update cached snapshot)
2) `controller.tick(snapshot, dt)` -> `ControlIntent`
3) Apply intent to ship:
   - thrust / turn / fire flags
4) Run systems:
   - movement, collisions, scoring, spawning, etc.

### AI decision cadence (reaction model)

The AI internally updates “decisions” (target selection, threat ranking) at:

- 5–10 Hz depending on profile

Between decision updates:

- intent is still produced each tick, but based on committed plan
- aim noise and small steering adjustments continue

---

## Configuration and Presets Wiring

### GameConfig additions

```rust
pub enum PlayerControllerMode {
    Human,
    AI { profile: AiProfile },
}

pub enum AiProfile {
    Casual,
    Balanced,
    Veteran,
}
```

`GameConfig` must include:

- `player_controller`
- `ai_profile` (if AI selected)
- and any AI tuning parameters (optional, keep minimal at first)

### Preset mapping

Add a preset:

- `ai_autopilot` => controller = AI(Balanced), baseline policies otherwise

---

## Runtime Toggle Requirements

### Options menu

- Player Controller: Human | AI
- AI Profile: Casual | Balanced | Veteran (only visible when AI selected)

Toggle semantics:

- Minimum: effective on next run/start
- Optional: hot-swap controller at runtime (safe if Controller state resets cleanly)

---

## HUD Instrumentation Requirements

The HUD must expose:

Core:

- controller mode (Human vs AI(profile))
- entity count
- sim frame time

AI-specific:

- AI decision cadence (Hz) and next decision countdown
- current target id + score
- top threat TTC + score
- AI tick cost estimate (ms)

Soak test:

- periodic logs: average FPS, max entities, AI cost, deaths, score

---

## Acceptance Criteria (Integration)

- Switching controller modes does not affect downstream gameplay logic (only input source changes).
- AI never directly mutates world state; it only emits `ControlIntent`.
- AI visibility is bounded (sensor radius + attention cap).
- AI cost remains bounded under high entity counts.
- AI can be used for a 10-minute soak test without runaway entities or memory growth.

---

## Consequences

- Clean separation between control policy and game simulation
- AI becomes a first-class evaluation tool early
- Future controllers (assist mode, replays, demo playback) can reuse the same seam
