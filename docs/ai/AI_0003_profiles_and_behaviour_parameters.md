# AI-0003: AI Profiles, Reaction Model, and Imperfection

## Status

Accepted

## Context

To avoid “aimbot” behavior, the AI must be constrained by:

- Reaction delay
- Limited decision cadence
- Aiming noise
- Commitment windows

Difficulty and style are expressed by **profiles**, not by changing algorithms.

---

## Reaction Model

### Decision Cadence

High-level decisions update at:

- Casual: 5 Hz (200 ms)
- Normal: 7 Hz (~140 ms)
- Veteran: 10 Hz (100 ms)

Between updates, the AI commits to the current target and steering goal.

---

## Aim Error Model

Aim is represented as a **cone**, not a point.

```text
aim_angle = ideal_angle + Normal(0, σ)
```

Where σ depends on:

- base profile value
- distance to target
- angular velocity of ship
- local clutter (number of threats)

This ensures:

- misses under stress
- reduced accuracy while turning
- no frame-perfect corrections

---

## Commitment Window

Once a target is selected:

- Commit for `0.5–1.5s` (profile-dependent)
- Break commitment only if threat spikes

This prevents dithering and creates readable intent.

---

## AI Profiles

### Casual

- Reaction: slow
- Aim error: large
- Avoidance weight: high
- Aggression: low

Feels cautious, sometimes indecisive.

---

### Balanced

- Reaction: medium
- Aim error: moderate
- Avoidance/aggression balanced

Target behavior for demos and evaluation.

---

### Veteran

- Reaction: fast
- Aim error: small (but non-zero)
- Aggression: high
- More frequent commitment

Feels skilled but still fallible.

---

## Parameter Table (Indicative)

| Parameter | Casual | Balanced | Veteran |
| --------- | ------ | -------- | ------- |
| Reaction Hz | 5 | 7 | 10 |
| Base aim σ (deg) | 6–8 | 3–5 | 1.5–2.5 |
| Avoidance weight | High | Medium | Low |
| Aggression bias | Low | Medium | High |
| Commit duration | Short | Medium | Long |

All values are tunable and data-driven.

---

## Debug / Evaluation Mode (Optional)

A non-player-facing mode may enable:

- Zero aim noise
- Full world visibility
- Instant reaction

This mode is strictly for testing and must not be exposed in normal presets.

---

## Consequences

- AI remains believable under all physics modes
- Difficulty scales via parameters, not code changes
- Autopilot is suitable for stress testing and demos
- No visual “fading” or robotic perfection
