use crate::config::AiProfile;
use crate::controllers::{ControlIntent, Controller};
use macroquad::prelude::{Vec2, screen_height, screen_width};
use std::f32::consts::{FRAC_PI_2, PI};

/// Snapshot that controllers can use to reason about nearby asteroids.
///
/// NOTE: Keep this allocation and size bounded upstream (sensor radius + attention cap).
#[derive(Clone, Debug)]
pub struct AsteroidSnapshot {
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
}

/// Read-only snapshot exposed to controllers.
///
/// Intentionally minimal: enough for player-like heuristics, not enough for omniscience.
#[derive(Clone, Debug)]
pub struct WorldSnapshot {
    pub ship_position: Vec2,
    pub ship_velocity: Vec2,
    pub ship_angle: f32,
    pub asteroids: Vec<AsteroidSnapshot>,
}

impl Default for WorldSnapshot {
    fn default() -> Self {
        Self {
            ship_position: Vec2::ZERO,
            ship_velocity: Vec2::ZERO,
            ship_angle: -FRAC_PI_2,
            asteroids: Vec::new(),
        }
    }
}

impl WorldSnapshot {
    /// Iterate over asteroid snapshots without cloning the backing storage.
    pub fn iter_asteroids(&self) -> impl Iterator<Item = &AsteroidSnapshot> {
        self.asteroids.iter()
    }
}

// -------------------------
// AI design summary
// -------------------------
// Goal: emulate a strong human strategy for this ruleset:
// - maintain a central operating region (margin-of-error maximization)
// - clear collision-probable threats proactively (lane-clearing)
// - evade only when TTC is critical (commit, then recover)
// - fire aggressively (no ammo downside), but gate by alignment to avoid aimbot feel
// - avoid edges/spawn lanes via explicit repulsion
//
// Constraints:
// - bounded CPU: consider only provided snapshot (already capped upstream)
// - player-like: reaction cadence, aim noise, commitment windows

// -------------------------
// Tunables
// -------------------------

/// Encapsulates all tuning parameters so they can be tweaked holistically per profile.
struct AiTuning {
    center_ring_min: f32,
    center_ring_max: f32,
    edge_margin: f32,
    edge_repulse_gain: f32,
    threat_range: f32,
    threat_ttc_max: f32,
    rel_speed_min_sq: f32,
    primary_range: f32,
    primary_arc: f32,
    secondary_cluster_range: f32,
    secondary_cluster_count: i32,
    target_max_range: f32,
    target_arc: f32,
    target_commit_min: f32,
    target_commit_max: f32,
    evade_hold: f32,
    recover_hold: f32,
    turn_smoothing: f32,
    evade_thrust: f32,
    engage_thrust: f32,
    recenter_thrust: f32,
    medium_radius: f32,
    large_radius: f32,
}

const TUNING: AiTuning = AiTuning {
    center_ring_min: 40.0,
    center_ring_max: 130.0,
    edge_margin: 110.0,
    edge_repulse_gain: 2.6,
    threat_range: 360.0,
    threat_ttc_max: 1.35,
    rel_speed_min_sq: 8.0 * 8.0,
    primary_range: 440.0,
    primary_arc: 0.72,
    secondary_cluster_range: 320.0,
    secondary_cluster_count: 2,
    target_max_range: 520.0,
    target_arc: 0.95,
    target_commit_min: 0.55,
    target_commit_max: 1.35,
    evade_hold: 0.48,
    recover_hold: 0.85,
    turn_smoothing: 0.35,
    evade_thrust: 0.45,
    engage_thrust: 0.78,
    recenter_thrust: 0.72,
    medium_radius: 16.0,
    large_radius: 24.0,
};

// -------------------------
// Helper math
// -------------------------

/// Normalize angles into [-π, π) using `rem_euclid` for clarity.
fn normalize_angle(angle: f32) -> f32 {
    (angle + PI).rem_euclid(2.0 * PI) - PI
}

fn clamp01(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

fn center() -> Vec2 {
    Vec2::new(screen_width() * 0.5, screen_height() * 0.5)
}

fn forward(angle: f32) -> Vec2 {
    Vec2::from_angle(angle)
}

// Small deterministic PRNG for aim noise and tie-breaking.
// (Avoids depending on global randomness and keeps behavior reproducible.)
#[derive(Clone, Debug)]
struct XorShift32 {
    state: u32,
}

impl XorShift32 {
    fn new(seed: u32) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    fn next_f32(&mut self) -> f32 {
        // [0,1)
        let u = self.next_u32();
        (u as f32) / (u32::MAX as f32 + 1.0)
    }

    fn normal_approx(&mut self) -> f32 {
        // Approximate N(0,1) via sum of uniforms (CLT).
        let mut s = 0.0;
        for _ in 0..6 {
            s += self.next_f32();
        }
        // mean=3, var=0.5; normalize to ~N(0,1)
        (s - 3.0) * 1.41421356
    }
}

// -------------------------
// AI state
// -------------------------

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Phase {
    Engage,
    Evade,
    Recover,
    Recenter,
}

#[derive(Clone, Debug)]
pub struct AiController {
    profile: AiProfile,

    // Reaction / decision cadence
    decision_timer: f32,

    // Target commitment
    target: Option<Vec2>,
    target_timer: f32,

    // Phase timing
    phase: Phase,
    phase_timer: f32,

    // Output smoothing
    last_turn: f32,

    // Cached evasion direction
    last_avoid_dir: Vec2,

    // Deterministic noise
    rng: XorShift32,
}

impl AiController {
    pub fn new(profile: AiProfile) -> Self {
        // Seed can later be plumbed from a run seed.
        Self {
            profile,
            decision_timer: 0.0,
            target: None,
            target_timer: 0.0,
            phase: Phase::Engage,
            phase_timer: 0.0,
            last_turn: 0.0,
            last_avoid_dir: Vec2::from_angle(-FRAC_PI_2),
            rng: XorShift32::new(0xC0FFEE_u32 ^ (profile as u32 + 1)),
        }
    }

    fn decision_interval(&self) -> f32 {
        match self.profile {
            AiProfile::Casual => 0.20,
            AiProfile::Balanced => 0.14,
            AiProfile::Veteran => 0.10,
        }
    }

    fn base_aim_sigma_deg(&self) -> f32 {
        match self.profile {
            AiProfile::Casual => 7.0,
            AiProfile::Balanced => 4.0,
            AiProfile::Veteran => 2.2,
        }
    }

    fn choose_commit_duration(&mut self) -> f32 {
        let u = self.rng.next_f32();
        TUNING.target_commit_min + u * (TUNING.target_commit_max - TUNING.target_commit_min)
    }
}

// -------------------------
// Threat model
// -------------------------

#[derive(Copy, Clone, Debug)]
struct Threat {
    avoid_dir: Vec2,
    severity: f32,
}

fn asteroid_size_weight(radius: f32) -> f32 {
    if radius >= TUNING.large_radius {
        1.65
    } else if radius >= TUNING.medium_radius {
        1.28
    } else {
        1.0
    }
}

fn detect_threat(world: &WorldSnapshot) -> Option<Threat> {
    let mut best: Option<Threat> = None;
    for ast in world.iter_asteroids() {
        let rel = ast.position - world.ship_position;
        let distance = rel.length();
        if distance > TUNING.threat_range {
            continue;
        }

        let rel_vel = ast.velocity - world.ship_velocity;
        let rel_speed_sq = rel_vel.length_squared();
        if rel_speed_sq < TUNING.rel_speed_min_sq {
            continue;
        }

        // Positive TTC means closing.
        let ttc = -rel.dot(rel_vel) / rel_speed_sq;
        if ttc <= 0.0 || ttc > TUNING.threat_ttc_max {
            continue;
        }

        // Severity ramps with smaller TTC and closer distance, weighted by size and relative speed.
        let size_w = asteroid_size_weight(ast.radius);
        let speed_w = 1.0 + (rel_vel.length() / 140.0).clamp(0.0, 1.4);
        let dist_w = (220.0 / distance.max(50.0)).clamp(0.25, 4.0);
        let ttc_w = (1.05 / (ttc + 0.08)).clamp(0.0, 9.0);
        let severity = (ttc_w * dist_w * size_w * speed_w).clamp(0.0, 10.0);
        if severity <= 0.0 {
            continue;
        }

        // Base evade direction: away from asteroid.
        let avoid_base = (-rel).normalize_or_zero();
        let avoid_dir = safe_direction(world, avoid_base);

        let candidate = Threat {
            avoid_dir,
            severity,
        };

        if best
            .as_ref()
            .map_or(true, |cur| candidate.severity > cur.severity)
        {
            best = Some(candidate);
        }
    }

    best
}

fn safe_direction(world: &WorldSnapshot, base: Vec2) -> Vec2 {
    // Evaluate a small set of candidate headings and choose the lowest risk.
    let offsets = [
        0.0,
        PI / 8.0,
        -PI / 8.0,
        PI / 5.0,
        -PI / 5.0,
        PI / 3.0,
        -PI / 3.0,
    ];
    let base_angle = base.to_angle();

    let mut best_dir = base;
    let mut best_score = direction_risk(world, base);

    for off in offsets.iter().skip(1) {
        let cand = Vec2::from_angle(base_angle + off);
        let score = direction_risk(world, cand);
        if score < best_score {
            best_score = score;
            best_dir = cand;
        }
    }

    best_dir
}

fn direction_risk(world: &WorldSnapshot, dir: Vec2) -> f32 {
    // Penalize headings that point into dense / aligned asteroid regions.
    // This stays cheap because the snapshot is bounded upstream.
    world
        .asteroids
        .iter()
        .map(|ast| {
            let rel = ast.position - world.ship_position;
            let distance = rel.length().max(1.0);
            let angle = normalize_angle(rel.to_angle() - dir.to_angle()).abs();

            let mut score = 0.0;

            // If the asteroid is near the direction we want to travel, penalize proximity.
            if angle < 0.85 {
                score += (0.85 - angle) * (TUNING.secondary_cluster_range / distance);
            }

            // If asteroid velocity aligns with that direction, add some risk.
            let vel_align = normalize_angle(ast.velocity.to_angle() - dir.to_angle()).abs();
            if vel_align < 0.9 {
                score += (0.9 - vel_align) * (ast.velocity.length() / 240.0);
            }

            // Larger asteroids increase risk.
            score * asteroid_size_weight(ast.radius)
        })
        .sum()
}

// -------------------------
// Target selection (lane clearing)
// -------------------------

fn select_target_lane_clearing(world: &WorldSnapshot) -> Option<Vec2> {
    // Prefer targets in the forward cone that are likely to become collision-probable soon.
    // If none, fall back to a near-ish target that does not drag us to edges.

    let ship_fwd = forward(world.ship_angle);
    let ship_pos = world.ship_position;

    let mut best: Option<(Vec2, f32)> = None;

    for ast in world.iter_asteroids() {
        let rel = ast.position - ship_pos;
        let distance = rel.length();
        if distance < 1.0 || distance > TUNING.target_max_range {
            continue;
        }

        let dir = rel / distance;
        let align = ship_fwd.dot(dir).clamp(-1.0, 1.0);
        let angle = align.acos();
        if angle > TUNING.target_arc {
            continue;
        }

        // TTC estimate for "is this on a collision trajectory soon".
        let rel_vel = ast.velocity - world.ship_velocity;
        let rel_speed_sq = rel_vel.length_squared();
        let mut ttc_bonus = 0.0;
        if rel_speed_sq >= TUNING.rel_speed_min_sq {
            let ttc = -rel.dot(rel_vel) / rel_speed_sq;
            if ttc > 0.0 {
                // Prefer smaller TTC (but avoid exploding to infinity).
                ttc_bonus = (1.0 / (ttc + 0.25)).clamp(0.0, 4.0);
            }
        }

        // Prefer larger asteroids to reduce future branching.
        let size_w = asteroid_size_weight(ast.radius);

        // Prefer closer.
        let dist_bonus = (260.0 / distance).clamp(0.2, 3.5);

        // Edge penalty: avoid targets that pull the ship toward edges/spawn lanes.
        let edge_penalty = edge_proximity(ast.position) * 1.8;

        // Alignment bonus (within cone).
        let align_bonus = (1.0 - (angle / TUNING.target_arc)).clamp(0.0, 1.0) * 2.2;

        let score = (align_bonus + dist_bonus + ttc_bonus) * size_w - edge_penalty;

        if best.as_ref().map_or(true, |(_, s)| score > *s) {
            best = Some((ast.position, score));
        }
    }

    best.map(|(p, _)| p)
}

fn edge_proximity(pos: Vec2) -> f32 {
    // Returns 0 in center-ish regions and increases near edges.
    let w = screen_width();
    let h = screen_height();

    let left = pos.x;
    let right = w - pos.x;
    let top = pos.y;
    let bottom = h - pos.y;

    let min_edge = left.min(right).min(top).min(bottom);
    if min_edge >= TUNING.edge_margin {
        0.0
    } else {
        ((TUNING.edge_margin - min_edge) / TUNING.edge_margin).clamp(0.0, 1.0)
    }
}

fn edge_repulsion(world: &WorldSnapshot) -> Vec2 {
    // Repel from edges strongly to avoid spawn lanes.
    let w = screen_width();
    let h = screen_height();
    let p = world.ship_position;

    let mut rep = Vec2::ZERO;

    let left = p.x;
    let right = w - p.x;
    let top = p.y;
    let bottom = h - p.y;

    if left < TUNING.edge_margin {
        rep.x += (TUNING.edge_margin - left) / TUNING.edge_margin;
    }
    if right < TUNING.edge_margin {
        rep.x -= (TUNING.edge_margin - right) / TUNING.edge_margin;
    }
    if top < TUNING.edge_margin {
        rep.y += (TUNING.edge_margin - top) / TUNING.edge_margin;
    }
    if bottom < TUNING.edge_margin {
        rep.y -= (TUNING.edge_margin - bottom) / TUNING.edge_margin;
    }

    if rep.length_squared() > 1e-6 {
        rep.normalize() * TUNING.edge_repulse_gain
    } else {
        Vec2::ZERO
    }
}

// -------------------------
// Fire policy
// -------------------------

fn compute_fire_policy(world: &WorldSnapshot) -> (bool, bool, i32, i32) {
    let ship_fwd = forward(world.ship_angle);

    let mut forward_hits = 0;
    let mut cluster_hits = 0;

    for ast in world.iter_asteroids() {
        let rel = ast.position - world.ship_position;
        let distance = rel.length();
        if distance > TUNING.primary_range {
            continue;
        }

        let dir = rel.normalize_or_zero();
        let align = ship_fwd.dot(dir).clamp(-1.0, 1.0);
        let angle = align.acos();

        if angle < TUNING.primary_arc {
            forward_hits += 1;
            if distance < TUNING.secondary_cluster_range {
                cluster_hits += 1;
            }
        }
    }

    let fire_primary = forward_hits > 0;
    let fire_secondary = cluster_hits >= TUNING.secondary_cluster_count;
    (fire_primary, fire_secondary, forward_hits, cluster_hits)
}

// -------------------------
// Controller implementation
// -------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::prelude::Vec2;
    use std::f32::consts::{FRAC_PI_2, PI};

    #[test]
    fn normalize_angle_ranges_minus_pi_to_pi() {
        assert!((normalize_angle(PI) + PI).abs() < 1e-6);
        assert!((normalize_angle(-PI) + PI).abs() < 1e-6);
        assert!((normalize_angle(3.5 * PI) + PI / 2.0).abs() < 1e-6);
    }

    #[test]
    fn clamp01_bounds_inputs() {
        assert_eq!(clamp01(-1.0), 0.0);
        assert_eq!(clamp01(0.5), 0.5);
        assert_eq!(clamp01(2.0), 1.0);
    }

    #[test]
    fn xor_shift32_deterministic_sequence() {
        let mut rng = XorShift32::new(1);
        assert_eq!(rng.next_u32(), 270369);
        assert_eq!(rng.next_u32(), 67634689);
        assert_eq!(rng.next_u32(), 2647435461);
    }

    #[test]
    fn xor_shift32_next_f32_range() {
        let mut rng = XorShift32::new(42);
        let value = rng.next_f32();
        assert!(value >= 0.0 && value < 1.0);
    }

    #[test]
    fn asteroid_size_weight_prefers_large() {
        let small = asteroid_size_weight(5.0);
        let medium = asteroid_size_weight(TUNING.medium_radius);
        let large = asteroid_size_weight(TUNING.large_radius + 1.0);
        assert!(medium > small);
        assert!(large > medium);
    }

    #[test]
    fn world_snapshot_iterates_asteroids() {
        let snapshot = WorldSnapshot {
            ship_position: Vec2::ZERO,
            ship_velocity: Vec2::ZERO,
            ship_angle: -FRAC_PI_2,
            asteroids: vec![
                AsteroidSnapshot {
                    position: Vec2::new(20.0, 0.0),
                    velocity: Vec2::ZERO,
                    radius: 3.0,
                },
                AsteroidSnapshot {
                    position: Vec2::new(-10.0, 5.0),
                    velocity: Vec2::ZERO,
                    radius: 5.0,
                },
            ],
        };
        assert_eq!(snapshot.iter_asteroids().count(), 2);
    }
}

impl Controller for AiController {
    fn tick(&mut self, world: &WorldSnapshot, dt: f32) -> ControlIntent {
        // Timers
        self.decision_timer = (self.decision_timer - dt).max(0.0);
        self.target_timer = (self.target_timer - dt).max(0.0);
        self.phase_timer = (self.phase_timer - dt).max(0.0);

        // Threat detection (continuous)
        let threat = detect_threat(world);

        // Phase selection with hysteresis
        if let Some(t) = threat {
            // Only enter evasion if threat is sufficiently severe.
            // This keeps behavior aligned with “dodge only when critical”.
            let enter = t.severity >= 1.1;
            if enter {
                self.phase = Phase::Evade;
                self.phase_timer = TUNING.evade_hold;
                self.last_avoid_dir = t.avoid_dir;
            }
        }

        if self.phase == Phase::Evade && self.phase_timer <= 0.0 {
            self.phase = Phase::Recover;
            self.phase_timer = TUNING.recover_hold;
        }

        // Recenter if far from center ring and not actively evading.
        let to_center = center() - world.ship_position;
        let center_dist = to_center.length();
        let outside_ring = center_dist > TUNING.center_ring_max;
        let inside_ring = center_dist < TUNING.center_ring_min;

        if self.phase != Phase::Evade {
            if outside_ring {
                self.phase = Phase::Recenter;
            } else if self.phase == Phase::Recenter && !outside_ring {
                self.phase = Phase::Engage;
            } else if self.phase == Phase::Recover && self.phase_timer <= 0.0 {
                self.phase = Phase::Engage;
            }
        }

        // Decision cadence: update target at limited frequency.
        if self.decision_timer <= 0.0 {
            self.decision_timer = self.decision_interval();

            // Refresh target if commitment expired or target is missing.
            if self.target.is_none() || self.target_timer <= 0.0 {
                self.target = select_target_lane_clearing(world);
                self.target_timer = self.choose_commit_duration();
            } else {
                // If we have a target, occasionally re-anchor to the nearest asteroid to that target
                // (prevents staleness when the target disappears).
                if let Some(tp) = self.target {
                    let mut best = None;
                    for ast in world.iter_asteroids() {
                        let d = (ast.position - tp).length_squared();
                        if best.as_ref().map_or(true, |(_, bd)| d < *bd) {
                            best = Some((ast.position, d));
                        }
                    }
                    if let Some((p, _)) = best {
                        self.target = Some(p);
                    } else {
                        self.target = None;
                        self.target_timer = 0.0;
                    }
                }
            }
        }

        // Compute high-level steering vectors
        let mut desired_heading = Vec2::ZERO;

        // Edge repulsion is always active to avoid spawn lanes.
        desired_heading += edge_repulsion(world);

        // Center control: maintain a ring, avoid jitter by only applying when outside the ring.
        if outside_ring {
            desired_heading += to_center.normalize_or_zero() * 1.9;
        } else if self.phase == Phase::Recover && !inside_ring {
            desired_heading += to_center.normalize_or_zero() * 1.1;
        }

        // Phase-specific behavior
        match self.phase {
            Phase::Evade => {
                desired_heading += self.last_avoid_dir * 3.0;
            }
            Phase::Recover => {
                // Recovery: bias toward center and away from edges; avoid aggressive chasing.
                desired_heading += self.last_avoid_dir * 0.6;
            }
            Phase::Recenter => {
                // Recenter: prioritize re-entering the operating region.
                desired_heading += to_center.normalize_or_zero() * 2.6;
            }
            Phase::Engage => {
                // Engage: lane-clearing toward committed target, but do not sacrifice edge safety.
                if let Some(tp) = self.target {
                    let dir = (tp - world.ship_position).normalize_or_zero();
                    desired_heading += dir * 1.35;
                }
            }
        }

        // If the heading is zero (rare), default to facing forward.
        if desired_heading.length_squared() < 1e-6 {
            desired_heading = forward(world.ship_angle);
        }

        // Convert heading into desired angle.
        let mut desired_angle = desired_heading.normalize_or_zero().to_angle();

        // Aim noise (player-like imperfections).
        // Noise increases with clutter and with large turn magnitudes.
        let clutter = world.asteroids.len().min(10) as f32;
        let clutter_scale = 1.0 + 0.06 * clutter;
        let base_sigma_deg = self.base_aim_sigma_deg();
        let sigma_deg = base_sigma_deg * clutter_scale;
        let sigma_rad = (sigma_deg.to_radians()).clamp(0.0, 0.35);

        // Apply noise only to aiming/engagement directions, not to pure evasion.
        if self.phase != Phase::Evade {
            desired_angle += self.rng.normal_approx() * sigma_rad;
        }

        // Turn command
        let delta = normalize_angle(desired_angle - world.ship_angle);
        let desired_turn = (delta / 1.0).clamp(-1.0, 1.0);
        let smooth_turn =
            desired_turn * (1.0 - TUNING.turn_smoothing) + self.last_turn * TUNING.turn_smoothing;
        self.last_turn = smooth_turn;

        // Thrust policy
        let mut thrust = match self.phase {
            Phase::Evade => TUNING.evade_thrust,
            Phase::Recover => 0.58,
            Phase::Recenter => TUNING.recenter_thrust,
            Phase::Engage => TUNING.engage_thrust,
        };

        // If we are near edges, reduce thrust slightly to maintain turn authority.
        let edge_p = edge_proximity(world.ship_position);
        if edge_p > 0.0 {
            thrust *= 1.0 - 0.22 * edge_p;
        }

        // Fire policy: aggressive lane clearing.
        let (mut fire_primary, mut fire_secondary, _forward_hits, cluster_hits) =
            compute_fire_policy(world);

        // In Evade, fire secondary more often to create space.
        if self.phase == Phase::Evade {
            fire_primary = true;
            if cluster_hits > 0 {
                fire_secondary = true;
            }
        }

        // Fire gating: require basic alignment for primary fire.
        // This prevents constant “laser pointer” behavior while still being aggressive.
        let ship_fwd = forward(world.ship_angle);
        let align_ok = if let Some(tp) = self.target {
            let rel = (tp - world.ship_position).normalize_or_zero();
            ship_fwd.dot(rel) > (TUNING.primary_arc.cos() * 0.98)
        } else {
            true
        };

        fire_primary = fire_primary && align_ok;

        let mut intent = ControlIntent::default();
        intent.turn = smooth_turn.clamp(-1.0, 1.0);
        intent.thrust = clamp01(thrust).clamp(0.15, 0.90);
        intent.fire_primary = fire_primary;
        intent.fire_secondary = fire_secondary;
        intent
    }
}
