# AI-0002: Threat and Target Scoring Model

## Status

Accepted

## Context

To behave believably, the AI must prioritize **survival first**, then opportunistic offense.
This requires:

- Fast, local reasoning
- No global pathfinding
- No perfect prediction

Threat and target selection must be:

- Cheap (bounded cost)
- Reactive
- Degraded under clutter

---

## World Visibility Constraints

The AI does not see the full world state.

Limits:

- Sensor radius: ~1.2× screen width
- Update cadence: tied to reaction time (5–10 Hz)
- Attention cap: only top `N` objects considered (default N = 10)

---

## Threat Scoring (Avoidance)

Each nearby object contributes a **threat score** based on time-to-collision and size.

### Approximate Time-To-Collision (TTC)

For object at position `p` with velocity `v`:

```text
relative_pos = p - ship_pos
relative_vel = v - ship_vel
ttc ≈ -dot(relative_pos, relative_vel) / |relative_vel|²
```

If `ttc < 0`, object is diverging → ignore.

---

### Threat Score Formula

```text
threat =
    w_ttc * clamp(1 / ttc, 0, T_MAX)

- w_dist * clamp(1 / distance, 0, D_MAX)
- w_size * object_radius
```

Typical weights:

- `w_ttc`: high (primary driver)
- `w_dist`: medium
- `w_size`: low–medium

Only the top K threats (e.g. K = 3–5) are used for steering.

---

## Avoidance Vector

Each threat produces a repulsion vector:

```text
avoid_dir = normalize(ship_pos - object_pos)
avoidance += threat * avoid_dir
```

The summed avoidance vector is normalized and blended with goal-seeking vectors.

---

## Target Scoring (Offense)

Targets are asteroids or enemies that are *safe* to engage.

### Target Score Formula

```text
target_score =
    w_align * alignment

- w_dist * (1 / distance)
- w_size * target_radius

- w_threat * local_threat
```

Where:

- `alignment = dot(ship_forward, dir_to_target)`
- `local_threat` is the summed threat near the target

Only targets above a minimum alignment threshold are considered.

---

## Fire Gating (Anti-Aimbot Rule)

The AI fires only if:

- alignment ≥ threshold (varies by difficulty)
- aim error estimate < tolerance
- not in a high-avoidance maneuver
- weapon cooldown ready

This prevents continuous perfect fire.

---

## Consequences

- AI prioritizes survival naturally
- Behavior degrades gracefully under clutter
- Computation remains O(N) with small N
- Produces readable, human-like motion patterns
