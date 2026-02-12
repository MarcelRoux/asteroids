# Epics with acceptance criteria

This roadmap is organized into shippable epics. Each epic has explicit acceptance criteria to prevent scope creep and to support incremental, measurable progress.

## Epic 0 — Engine scaffold and performance instrumentation

### Objective

Establish a stable simulation/render loop and the instrumentation needed to evolve the system safely.

### Deliverables

- Fixed timestep simulation (e.g. 60 Hz) with accumulator; render at vsync/unlocked.
- Input mapping, pause, restart, window scaling.
- Deterministic RNG seed (for repeatable tests).
- HUD: FPS, sim ms, render ms, entity count, collision pair count.

### Toggles (early)

- `physics_mode = Off | Arcade`
- `leaderboard_mode = Off`
- `player_controller = Human | AI` (profile selectable)

### Acceptance criteria

- Simulation remains stable under frame drops (e.g. 30–120 FPS) with no spiral-of-death.
- HUD shows entity count, collision pairs, and per-frame timings.
- Soak mode runs 10 minutes with no memory growth or entity runaway.

---

## Epic 1 — Classic Asteroids baseline (shippable)

### Objective

Ship a complete baseline game that is the reference point for all later feature work

### Deliverables

- Ship movement (thrust/rotate), screen wrap.
- Bullets, cooldown, asteroid spawning.
- Classic split rule (2 fragments) without polygon slicing.
- Score system + local Top 10 leaderboard (file-backed).
- Menu: Play / Options / Leaderboard.

### Acceptance criteria

- From fresh start, player can play indefinitely; no softlocks.
- Leaderboard persists across restarts and sorts correctly.
- Baseline sustains target FPS on low-end device class.

---

## Epic 1A — Toggleable AI autopilot controller (evaluation foundation)

### Objective

Enable a player-like AI controller via runtime toggle to support early evaluation (balance + performance) and automated soak testing

### Deliverables

- Controller abstraction emitting `ControlIntent` (HumanController + AiController).
- AI constraints to avoid aimbot behavior:
  - reaction delay + limited decision cadence (5–10 Hz),
  - aim error cone (distance/turn/clutter dependent),
  - commitment windows (avoid dithering),
  - fire gating (shoot only when alignment and safety thresholds are met).
- `WorldSnapshot` restrictions (sensor radius + attention cap) to bound CPU and prevent omniscience.
- Options toggle: Player Controller = Human | AI (profile: Casual | Balanced | Veteran).
- HUD: controller mode, AI cadence, current target/threat score, per-tick AI cost.

### Acceptance criteria

- With Player Controller = AI, the ship survives and scores in Classic mode without frame-perfect aim.
- AI behavior is purposeful (no visible “fading” or dithering) due to commitment windows.
- AI cost is bounded (e.g. <0.5 ms/frame at 1,000 entities under `collision_policy = PlayerOnly`).
- Human and AI controllers exercise identical downstream systems via `ControlIntent`.
- AI-enabled soak test runs 10 minutes with no memory growth or entity runaway.

---

## Epic 1B — Remaining Asteroids Game Mechanics

### Objective

Bring game closer to the classic game by implementing additional game mechanics.

### Deliverables

- Improved resolution of ship (akin to classic game).
- Indication of thrust (akin to classic game).
- Player (and AI) name capturing at game over (exit should not persist score).
- A life is gained for every 10_000 points.
- Basic small alien ship following horizonal path, firing at player with accuracy that is score-based:
  - Alien ship fires at player in a cone that starts wide and becomes narrower (more accurate) as the player gains points.
  - Alien ship bullet is equivalent to player bullet at this time.
  - Up to 2 small alien ships may spawn at a time.
  - Starts to spawn randomly after 40_000 points have been reached.
  - Small alien ship hit score: 1000.
- Basic large alien ship following horizontal path, firing at player with accuracy that is score-based:
  - Alien ship fires at player in a cone that starts wide and becomes narrowwer (more accurate) as the player gains points.
  - Alien ship bullet is equivalent to player bullet at this time.
  - Up to 1 large alien ship may spawn at a time.
  - Starts to spawn randomly after 40_000 points have been reached.
  - Large alien ship hit score: 200.
- Menu and leaderboard etc. polish to put the elements in the middle as opposed to top left of the screen.
- Leaderboard polish to right align the numerical scores for better perception of orders of magnitude.
- Stretch: Additional statistics to track.
  - Player shots fired.
  - Player shots hit.
  - Player accuracy - based on above values.
  - Per-entity hit breakdown:
    - Large asteroid.
    - Medium asteroid.
    - Small asteroid.
    - Large alien ship.
    - Small alien ship.
- Stretch: Display additional statistics in the debug overlay and store alongside leaderboard statistics.

### Acceptance criteria

-
- With Player Controller = AI, the ship survives and scores in Classic mode without frame-perfect aim.
- AI behavior is purposeful (no visible “fading” or dithering) due to commitment windows.
- AI cost is bounded (e.g. <0.5 ms/frame at 1,000 entities under `collision_policy = PlayerOnly`).
- Human and AI controllers exercise identical downstream systems via `ControlIntent`.
- AI-enabled soak test runs 10 minutes with no memory growth or entity runaway.

---

## Epic 2 — Progression and upgrades panel

### Objective

Add ship progression as optional, data-driven systems that do not compromise baseline behavior

### Deliverables

- Upgrade currency earned during run.
- Upgrade panel between waves (or pause).
- Upgrade categories:
  - mobility (accel, max speed, turn rate)
  - defense (shields, regen/cooldown)
  - weapon improvements (cooldown, energy)
  - utility (sentinel capacity later)

### Acceptance criteria

- With `upgrades_enabled=false`, game behaves exactly like Epic 1.
- Upgrades are data-driven (tuning via config or table).
- No per-tick allocations caused by upgrade checks (precompute modifiers).

---

## Epic 3 — Convex polygon asteroids and slicing weapon

### Objective

Introduce plausible fracture via bounded-cost computational geometry

### Deliverables

- Asteroids represented as convex polygons (≤ `V_MAX`, default 24 vertices).
- `split_convex_polygon_by_line()` with bias retries.
- Post-split simplification and degeneracy handling.
- Laser weapon: slice + “damage on failed split”.

### Acceptance criteria

- Split produces two valid convex polygons or no split; never invalid geometry.
- Budgets enforced: `V_MAX`, bounded retries, slivers rejected.
- Debug view can render polygon outline + cut line for visual verification.
- Slicing many asteroids in one frame stays within a fixed time budget (e.g. <2–4 ms on baseline machine).

---

## Epic 4 — Explosion / drill-to-center shatter model

### Objective

Provide “scientific-feeling” shatter behavior without heavy simulation

### Deliverables

- Drill rocket states: homing → attach → drill → detonate.
- Shatter algorithm: K cut attempts on largest fragment (bounded), keep N largest, rest → particles.
- Mass/area-weighted “pop” impulse; optional angular velocity under Lite physics.

### Acceptance criteria

- Fragmentation is bounded (per-event cap + global cap).
- Mass/area feels conserved (no sudden disappearance of material).
- Interior-origin blast yields visibly different shard distribution.

---

## Epic 5 — Enemies, carrier + drones, sentinels

### Objective

Add horde-mode content and systemic interactions

### Deliverables

- Enemy archetypes with cheap state machines/steering.
- Carrier spawns drones; carrier destruction changes drone behavior.
- Sentinel deployable: target selection, fire support, cooldown/energy.

### Acceptance criteria

- Enemy AI runs within a fixed per-frame budget (e.g. <1–2 ms at 500 enemies).
- Carrier-drone dependency behaves correctly.
- Sentinels do not introduce unexpected collision overhead.
- AI autopilot can be used to benchmark enemy-wave performance and difficulty presets (no special-casing).

---

## Epic 6 — Physics ladder and collision policies

### Objective

Expose “realism vs performance” as user-facing policies without destabilizing the simulation

### Physics modes

- Off: no angular velocity; simple kinematics
- Arcade: impulses affect linear velocity; minimal damping
- Lite: linear + angular; mass-weighted pop; mild damping

### Collision policies

- PlayerOnly: only player/projectiles collide with asteroids/fragments
- BigOnly: only large fragments collide with each other
- Full: all collide (expensive)

### Acceptance criteria

- Switching modes changes behavior predictably and is reflected in HUD/settings.
- PlayerOnly and BigOnly sustain horde-mode body counts.
- Full mode warns and/or triggers Performance Guard.
- No correctness regressions (no stuck bodies, no pathological tunneling spikes).

---

## Epic 7 — Presets, UX polish, and Performance Guard

### Objective

Make toggles usable and safe for a wide device range

### Deliverables

- Preset selector: Classic / Arcade Upgrades / Fracture / Horde / Simulation / Custom.
- Advanced panel: caps (max bodies, debris TTL, event fragment cap), thresholds, physics.
- Performance Guard: monitors frame time and applies degradation policy.

### Acceptance criteria

- Presets are coherent and one-click.
- Performance Guard prevents runaway entity growth and preserves playability.
- Settings persist and can be reset.
