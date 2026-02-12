use macroquad::prelude::{Color, Vec2, draw_line};

fn rotate_vec(v: Vec2, cos_a: f32, sin_a: f32) -> Vec2 {
    Vec2::new(v.x * cos_a - v.y * sin_a, v.x * sin_a + v.y * cos_a)
}

fn transform_point(point: Vec2, pos: Vec2, angle: f32) -> Vec2 {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    rotate_vec(point, cos_a, sin_a) + pos
}

pub fn draw_shape(
    lines: &[(Vec2, Vec2)],
    position: Vec2,
    angle: f32,
    stroke_width: f32,
    color: Color,
    glow: bool,
) {
    if glow {
        let glow_color = Color::new(color.r, color.g, color.b, color.a * 0.25);
        for (a, b) in lines {
            let a_world = transform_point(*a, position, angle);
            let b_world = transform_point(*b, position, angle);
            draw_line(
                a_world.x,
                a_world.y,
                b_world.x,
                b_world.y,
                stroke_width * 1.6,
                glow_color,
            );
        }
    }

    for (a, b) in lines {
        let a_world = transform_point(*a, position, angle);
        let b_world = transform_point(*b, position, angle);
        draw_line(
            a_world.x,
            a_world.y,
            b_world.x,
            b_world.y,
            stroke_width,
            color,
        );
    }
}

fn scaled_point(x: f32, y: f32, scale: f32) -> Vec2 {
    Vec2::new(x * scale, y * scale)
}

/// Returns the line segments for the player ship outline in local space (nose = +y) and
/// labels the key points for clarity:
///
/// ```text
///    N
///   / \
///  /   \
/// L-----R
///  \___/
/// ```
///
/// - `N` is the nose point at (0, +1.0)
/// - `L` and `R` are the rear corners at (−0.7, −0.8) and (+0.7, −0.8)
/// - The base notch is formed by the segments to the center gap (`<- - ->`)
/// - `gap_center` sits at (0, −0.6) to close the rear with twin legs.
pub fn ship_lines(scale: f32) -> Vec<(Vec2, Vec2)> {
    let nose = scaled_point(0.0, 1.0, scale);
    let left_rear = scaled_point(-0.7, -0.8, scale);
    let right_rear = scaled_point(0.7, -0.8, scale);
    let notch_left = scaled_point(-0.2, -0.8, scale);
    let notch_right = scaled_point(0.2, -0.8, scale);
    let gap_center = scaled_point(0.0, -0.6, scale);

    vec![
        (left_rear, nose),
        (nose, right_rear),
        (left_rear, notch_left),
        (notch_right, right_rear),
        (notch_left, gap_center),
        (notch_right, gap_center),
    ]
}

pub fn saucer_large_lines(scale: f32) -> Vec<(Vec2, Vec2)> {
    let outline = [
        (-1.5, 0.18),
        (-1.1, 0.35),
        (-0.6, 0.5),
        (0.0, 0.6),
        (0.6, 0.5),
        (1.1, 0.35),
        (1.5, 0.18),
        (1.7, -0.05),
        (1.4, -0.35),
        (0.8, -0.55),
        (-0.8, -0.55),
        (-1.4, -0.35),
        (-1.7, -0.05),
        (-1.5, 0.18),
    ];
    let windows = [(-0.5, 0.0), (-0.25, 0.25), (0.25, 0.25), (0.5, 0.0)];

    let mut lines = outline
        .windows(2)
        .map(|window| {
            (
                scaled_point(window[0].0, window[0].1, scale),
                scaled_point(window[1].0, window[1].1, scale),
            )
        })
        .collect::<Vec<_>>();

    for chunk in windows.windows(2) {
        let a = scaled_point(chunk[0].0, chunk[0].1, scale * 0.9);
        let b = scaled_point(chunk[1].0, chunk[1].1, scale * 0.9);
        lines.push((a, b));
    }

    lines
}

pub fn saucer_small_lines(scale: f32) -> Vec<(Vec2, Vec2)> {
    let outline = [
        (-1.2, 0.2),
        (-0.8, 0.36),
        (-0.3, 0.45),
        (0.3, 0.45),
        (0.8, 0.36),
        (1.2, 0.2),
        (1.35, -0.03),
        (1.05, -0.25),
        (-1.05, -0.25),
        (-1.35, -0.03),
        (-1.2, 0.2),
    ];
    let windows = [(-0.35, 0.1), (-0.15, 0.27), (0.15, 0.27), (0.35, 0.1)];

    let mut lines = outline
        .windows(2)
        .map(|window| {
            (
                scaled_point(window[0].0, window[0].1, scale),
                scaled_point(window[1].0, window[1].1, scale),
            )
        })
        .collect::<Vec<_>>();

    for chunk in windows.windows(2) {
        let a = scaled_point(chunk[0].0, chunk[0].1, scale * 0.9);
        let b = scaled_point(chunk[1].0, chunk[1].1, scale * 0.9);
        lines.push((a, b));
    }

    lines
}
