# Settings and presets matrix

This document defines:

- user-facing toggles (feature levers),
- performance and correctness budgets,
- presets that map “play modes” to coherent configurations.

The primary goal is to keep the system configurable while ensuring the simulation remains bounded under worst-case load.

## Core toggles

- `upgrades_enabled`: `true | false`
- `player_controller`: `Human | AI`
- `ai_profile`: `Casual | Balanced | Veteran` (only relevant when `player_controller = AI`)
- `fragmentation_mode`: `Off | ClassicSplit | SliceOnly | Explode | Full`
- `physics_mode`: `Off | Arcade | Lite`
- `collision_policy`: `PlayerOnly | BigOnly | Full`
- `enemies_enabled`: `true | false`
- `sentinels_enabled`: `true | false`
- `leaderboard_mode`: `Off | LocalTop10`
- `performance_guard`: `Off | On`

## Budget knobs

Budget knobs are exposed in an Advanced panel and/or derived from presets.

- `MAX_BODIES`: global cap on active physics entities (ship + bullets + asteroids + fragments + enemies)
- `FRAG_EVENT_CAP`: cap on *real* fragments created per fragmentation event
- `DEBRIS_TTL_MS`: time-to-live for small debris (used for graceful degradation)
- `BIG_COLLISION_RADIUS`: radius threshold used by `collision_policy = BigOnly`
- `V_MAX`: max vertices per asteroid polygon (fixed across modes; default `24`)

## Presets

Presets are intended to be one-click and internally consistent. “Custom” exposes all levers.

| Preset | Intended feel | Controller | Upgrades | Fragmentation | Physics | Collision policy | Enemies | Sentinels | Leaderboard | Performance guard | Suggested budgets |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Classic | Faithful baseline | Human | Off | ClassicSplit | Arcade | PlayerOnly | Off | Off | LocalTop10 | On | `MAX_BODIES=800`, `FRAG_EVENT_CAP=2–4`, `DEBRIS_TTL_MS=900` |
| Arcade Upgrades | Baseline + builds | Human | On | ClassicSplit | Arcade | PlayerOnly | Off | Off | LocalTop10 | On | `MAX_BODIES=900`, `FRAG_EVENT_CAP=2–4`, `DEBRIS_TTL_MS=900` |
| AI Autopilot | Baseline evaluation bot | AI (Balanced) | Off | ClassicSplit | Arcade | PlayerOnly | Off | Off | LocalTop10 | On | `MAX_BODIES=800`, `FRAG_EVENT_CAP=2–4`, `DEBRIS_TTL_MS=900` |
| Fracture Mode | Signature slicing | Human | On | SliceOnly | Lite | PlayerOnly | Optional | Optional | LocalTop10 | On | `MAX_BODIES=1200`, `FRAG_EVENT_CAP=8–10`, `DEBRIS_TTL_MS=800`, `V_MAX=24` |
| Horde Mode | Dense targets, bounded chaos | Human | On | Explode (or Full*) | Lite | BigOnly | On | On | LocalTop10 | On | `MAX_BODIES=1500–2500`, `FRAG_EVENT_CAP=10–12`, `DEBRIS_TTL_MS=600`, `BIG_COLLISION_RADIUS` tuned |
| Simulation-ish | “More physical” (warned) | Human | On | Full | Lite | Full | On | Optional | LocalTop10 | On (aggressive) | `MAX_BODIES=600–1200`, `FRAG_EVENT_CAP=6–10`, `DEBRIS_TTL_MS=500–700` |
| Custom | Expert panel | User | User | User | User | User | User | User | User | User | Budgets editable + reset |

\- In Horde, “Full” fragmentation can coexist with `collision_policy = BigOnly` to keep performance bounded.

## Performance Guard

The Performance Guard is a runtime policy that maintains playability by reducing expensive effects first.

### Guard inputs

- rolling FPS (or simulation frame time)
- current body count
- collision pair count
- controller mode (Human vs AI) for reproducible benchmarks

### Guard actions (in order)

1. Reduce `DEBRIS_TTL_MS` (least noticeable)
2. Reduce `FRAG_EVENT_CAP`
3. Raise `BIG_COLLISION_RADIUS` (reduces fragment-fragment collision workload)
4. Downgrade `collision_policy`: `Full → BigOnly → PlayerOnly` (optional “Allow auto downgrade” toggle)
5. Clamp spawns for subsequent waves until recovered

## Notes

The only configuration that can reliably collapse performance is the combination of:

- `collision_policy = Full`,
- high `MAX_BODIES`,
- high `FRAG_EVENT_CAP`.

Presets and the Performance Guard are designed to prevent accidental entry into this regime.
