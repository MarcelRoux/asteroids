# AI-0001: Player Controller Architecture (Human vs AI)

## Status

Accepted

## Context

The game supports both human-controlled and AI-controlled operation of the player ship.  
The AI must:

- Behave *player-like*, not as an aimbot
- Operate under human-like constraints (reaction time, noise, limited awareness)
- Be toggleable at runtime for evaluation, stress testing, and demo purposes
- Share the same downstream control path as human input

To ensure clean architecture and avoid gameplay divergence, all player control is modeled as a **controller policy** that emits intents, not actions.

---

## Decision

Introduce a unified `Controller` interface that produces a `ControlIntent` each simulation tick.  
Both human input and AI logic implement this interface.

The active controller is selected via configuration or runtime toggle.

---

## Core Types

### ControlIntent

Represents *intent*, not force or state mutation.

```rust
struct ControlIntent {
    thrust: f32,              // 0.0 .. 1.0
    turn: f32,                // -1.0 .. 1.0 (left/right)
    fire_primary: bool,
    fire_secondary: bool,
    deploy_sentinel: bool,
}
```

Note: `deploy_sentinel` is reserved for a later "sentinel" feature and is not implemented yet. The current code's
`ControlIntent` does not include this field.

### Controller Trait

```rust
trait Controller {
    fn tick(&mut self, world: &WorldSnapshot, dt: f32) -> ControlIntent;
}
```

## Implementation Notes

- Current file paths (post-refactor):
- `src/controllers/mod.rs` defines `ControlIntent` and the `Controller` trait.
- `src/controllers/human.rs` implements the human controller.
- `src/ai/mod.rs` implements the AI controller and defines `WorldSnapshot` (currently in one module).

---

## Implementations

### HumanController

- Reads keyboard/gamepad input
- Applies dead zones and smoothing
- Emits `ControlIntent` directly

### AiController

- Reads from a *restricted* `WorldSnapshot`
- Updates decisions at a limited cadence (reaction time)
- Applies noise and commitment windows
- Emits intents through the same interface

---

## Toggle and Configuration

The active controller is selected via configuration:

```rust
enum PlayerControllerMode {
    Human,
    AI { profile: AiProfile },
}
```

Runtime toggling is permitted for:

- Demo purposes
- Automated stress testing
- “Autopilot” evaluation of balance and performance

---

## Non-Goals

- No frame-perfect aiming
- No omniscient world access
- No pathfinding or global planning
- No ML or training component

The AI is a *heuristic autopilot*, not a solver.

---

## Consequences

- Human and AI players exercise identical downstream systems
- AI can be used early for performance and fragmentation evaluation
- Player-like imperfections are enforced structurally, not cosmetically
