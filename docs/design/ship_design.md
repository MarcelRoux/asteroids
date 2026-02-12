DESIGNS TO IMPLEMENT:

Implement a simple, readable “vector outline” art style for the player ship and alien ships (Asteroids-inspired). Use line segments only (no filled polygons), white lines on black background, with optional subtle glow via duplicate-draw (same geometry slightly thicker / faint alpha if supported). Keep shapes low-vertex for performance and clarity.

Core ASCII references (orientation: ship nose points up in local space, +Y):

Player ship:
   /\
  /  \
 /____\
/      \

Alien ship (large):
    ____________
 __/            \__
/__________________\
\__              __/
   \____________/

Alien ship (small):
     ________
  __/        \__
 /______________\
 \__          __/
    \________/

PLAYER SHIP (classic triangular ship):

- Geometry: an isosceles triangle outline with a small “notch” or gap at the rear to imply an engine.
- Orientation: ship_angle points along the triangle’s nose (forward).
- Suggested proportions (scale with a SHIP_SIZE constant):
  - Nose point at (0, +1.0)
  - Rear-left at (-0.7, -0.8)
  - Rear-right at (+0.7, -0.8)
  - Rear notch/gap: split the rear base into two short segments with a center gap, matching the ASCII “/____\\” line with a break in the middle.
- Draw as 5–7 line segments (outline + notch legs). No filled triangles.

THRUST FX:

- When thrusting, draw a flame as 2–3 short line segments emitted from the rear gap (a small V or fork).
- Flame length and jitter should vary slightly per frame (small RNG) but remain stable-looking.

ALIEN SHIPS (two variants: large saucer and small saucer):

- Both match the ASCII references: an upper dome (top arc), a wide mid-body, and a lower arc, all outline-only.
- LARGE SAUCER:
  - Wider, flatter silhouette; 12–18 segments total (including optional windows).
  - Top dome approximates: `__/            \\__` using a few straight segments.
  - Lower hull approximates: `\\__              __/` then `\\____________/`.
  - Optional windows: short ticks on the center band (as in current `shapes.rs`).
- SMALL SAUCER:
  - Same design language but more compact; 10–16 segments total.
  - Slightly taller dome relative to width to visually distinguish from large, matching the ASCII.
- Keep a consistent “stroke weight” and align geometry to the same forward direction convention (even if saucers move sideways).

IMPLEMENTATION DETAILS:

- Provide functions that return local-space line lists for each shape:
  - fn ship_lines(scale: f32) -> Vec<(Vec2, Vec2)>
  - fn saucer_large_lines(scale: f32) -> Vec<(Vec2, Vec2)>
  - fn saucer_small_lines(scale: f32) -> Vec<(Vec2, Vec2)>
- Rendering: transform each segment by rotation(ship_angle) + translation(position) and draw with macroquad draw_line().
- Keep all constants at top: SHIP_SIZE, SAUCER_LARGE_SIZE, SAUCER_SMALL_SIZE, STROKE_WIDTH.
- Ensure hitboxes can remain circles (radius ~ scale * 0.9) independent of exact outline.

Deliverables:

1) A Rust module `src/render/shapes.rs` containing the geometry builders and a `draw_shape(lines, pos, angle, stroke)` helper.
2) Update player/enemy rendering to use these outlines.
3) Add thrust flame drawing toggled by thrust input.
