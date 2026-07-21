//! The contact marks: where they sit, and what their size means.

use super::*;
use ph2d_ecs::Entity;

fn window() -> WindowSize {
    WindowSize {
        width: 1000,
        height: 1000,
    }
}

fn camera() -> Camera2d {
    Camera2d {
        center: [0.0, 0.0],
        height_world: 10.0,
        ..Camera2d::default()
    }
}

fn contact(point: [f32; 2], impulse: f32) -> BodyContact {
    BodyContact {
        a: Entity::from_bits(1),
        b: Entity::from_bits(2),
        point,
        impulse,
    }
}

/// Half the width of a mark's horizontal arm, in screen px — the size the load maps
/// to. Read off the path so the test measures what is DRAWN, not what the constant
/// says.
fn arm_px(path: &BezPath) -> f64 {
    let pts: Vec<Point> = path
        .elements()
        .iter()
        .filter_map(|el| match el {
            ph2d_vector::PathEl::MoveTo(p) | ph2d_vector::PathEl::LineTo(p) => Some(*p),
            _ => None,
        })
        .collect();
    (pts[1].x - pts[0].x) / 2.0
}

#[test]
fn a_heavier_load_draws_a_bigger_mark() {
    // The size IS the reading. A stack shows a gradient because the bottom joint
    // carries everything above it, and if every mark were the same size the overlay
    // would be answering "are these touching" while pretending to answer "how hard".
    let light = &contact_marks(true, &[contact([0.0, 0.0], 0.005)], &camera(), window())[0];
    let heavy = &contact_marks(true, &[contact([0.0, 0.0], 0.045)], &camera(), window())[0];
    assert!(
        arm_px(heavy) > arm_px(light) * 1.5,
        "a 9x load should draw a visibly bigger mark ({} vs {})",
        arm_px(heavy),
        arm_px(light)
    );
    // And it saturates rather than growing without bound — past the ruler, "very
    // loaded" is the useful reading and a mark the size of the screen is not.
    let huge = &contact_marks(true, &[contact([0.0, 0.0], 500.0)], &camera(), window())[0];
    assert!(
        (arm_px(huge) - arm_px(heavy)).abs() < 1.0,
        "the mark must saturate at the ruler, got {} vs {}",
        arm_px(huge),
        arm_px(heavy)
    );
}

#[test]
fn the_mark_sits_on_the_contact_point_in_screen_space() {
    // A camera 10 world units tall over a 1000 px window is 100 px per unit, and the
    // centre of the world is the centre of the screen. A contact 2 units to the right
    // is therefore 200 px right of centre — the arithmetic a mark drawn at the body's
    // centre, or in world units, would get wrong.
    let marks = contact_marks(true, &[contact([2.0, 0.0], 0.01)], &camera(), window());
    let pts: Vec<Point> = marks[0]
        .elements()
        .iter()
        .filter_map(|el| match el {
            ph2d_vector::PathEl::MoveTo(p) | ph2d_vector::PathEl::LineTo(p) => Some(*p),
            _ => None,
        })
        .collect();
    let cx = (pts[0].x + pts[1].x) / 2.0;
    let cy = (pts[2].y + pts[3].y) / 2.0;
    assert!(
        (cx - 700.0).abs() < 1.0 && (cy - 500.0).abs() < 1.0,
        "the mark should be centred at (700, 500) px, got ({cx}, {cy})"
    );
}

#[test]
fn the_toggle_switches_the_marks_off() {
    assert!(
        contact_marks(false, &[contact([0.0, 0.0], 0.01)], &camera(), window()).is_empty(),
        "a painter or vector user must never see physics chrome over their artwork"
    );
    assert!(
        contact_marks(true, &[], &camera(), window()).is_empty(),
        "a scene where nothing touches draws nothing"
    );
}
