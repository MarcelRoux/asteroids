# SIM-0001: Simulation Boundary and Determinism Contract

## Status

Accepted

## Purpose

Define a strict boundary between:

- **simulation** (authoritative game state), and
- **presentation / control** (input, AI, rendering).

This contract enables:

- AI autopilot
- reproducible soak tests
- future multiplayer or replay systems
- engine portability

## Core Principle

The simulation step is a pure function of:

- previous world state
- per-entity ControlIntent for the current tick
- deterministic RNG state
- immutable configuration (GameConfig)

Rendering, input, and UI must not mutate simulation state directly.

## Fixed Timestep

- Simulation runs at a fixed timestep (e.g. 60 Hz).
- Rendering may interpolate or run at variable frame rates.
- No simulation logic depends on wall-clock time.

## Control Path

- All player, AI, or future network control is expressed as `ControlIntent`.
- Simulation consumes intents, never raw input.

This ensures:

- Human, AI, and future networked players are equivalent.
- Control logic can be swapped without touching gameplay code.

## WorldSnapshot

- AI (and future replay/debug tools) consume a restricted `WorldSnapshot`.
- Snapshot visibility is bounded and intentionally lossy.
- Simulation state is never exposed directly.

## Determinism Goals

This project targets **practical determinism**:

- deterministic RNG streams
- consistent tick ordering
- bounded floating-point drift tolerated in arcade modes

Perfect lockstep determinism is not required initially, but the architecture does not preclude it.

## Non-Goals

- No simulation logic inside rendering code
- No hidden side effects in systems
- No per-frame allocations in hot simulation paths

## Consequences

- AI autopilot and soak tests are first-class citizens
- Performance issues are attributable and debuggable
- Multiplayer or replay systems can be added without redesign
