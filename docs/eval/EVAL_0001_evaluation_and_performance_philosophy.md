# EVAL-0001: Evaluation-First Design and Performance Philosophy

## Status

Accepted

## Context

This project is designed not only to be *played*, but to be *evaluated*.

Many games optimize implicitly; this project exposes:

- performance budgets,
- degradation strategies,
- and evaluation tooling directly to the developer and player.

## Evaluation Pillars

### 1. AI Autopilot

- A player-like AI can control the ship.
- Used for:
  - automated soak tests
  - balance iteration
  - benchmarking under load
- AI is constrained to avoid omniscience or perfect aim.

### 2. HUD Instrumentation

The HUD exposes:

- entity counts
- collision pair counts
- fragmentation events
- AI decision cadence
- per-frame timing (sim/render)

This allows immediate feedback when toggles change behavior.

### 3. Soak Tests

- CLI-driven soak mode runs the game unattended.
- Used to detect:
  - memory growth
  - entity runaway
  - performance degradation over time

### 4. Performance Guard

- Monitors FPS and simulation cost.
- Applies bounded degradation strategies:
  - reduce debris TTL
  - reduce fragmentation caps
  - downgrade collision policy (optional)

The goal is graceful degradation, not sudden failure.

## Explicit Budgets

The game enforces explicit budgets:

- maximum active bodies
- maximum fragments per event
- maximum polygon complexity
- collision policy constraints

Budgets are surfaced via presets and advanced settings.

## Player-Facing Honesty

Modes that are expensive (e.g. Full collisions) are:

- clearly labeled
- optionally warned
- optionally auto-downgraded

This avoids misleading the player about performance expectations.

## Consequences

- The game remains playable across a wide device range.
- Performance tradeoffs are visible and intentional.
- The project demonstrates real-world system engineering practices.
