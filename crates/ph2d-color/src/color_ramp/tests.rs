//! Tests for the [`ColorRamp`] object: stop management, clamping at the ends, every interpolation
//! mode and color space, and the baked LUT.

use super::*;

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

fn approx(a: [f32; 4], b: [f32; 4], eps: f32) -> bool {
    (0..4).all(|i| (a[i] - b[i]).abs() <= eps)
}

#[test]
fn default_is_black_to_white_linear() {
    let r = ColorRamp::default();
    assert_eq!(r.len(), 2);
    assert_eq!(r.eval(0.0), BLACK);
    assert_eq!(r.eval(1.0), WHITE);
    assert!(
        approx(r.eval(0.5), [0.5, 0.5, 0.5, 1.0], 1e-6),
        "linear midpoint is mid-grey"
    );
}

#[test]
fn ends_are_clamped_no_extrapolation() {
    let r = ColorRamp::new(
        vec![RampStop::new(0.25, BLACK), RampStop::new(0.75, WHITE)],
        RampColorMode::Rgb,
        RampInterp::Linear,
    );
    assert_eq!(
        r.eval(-1.0),
        BLACK,
        "below first stop holds the first color"
    );
    assert_eq!(r.eval(0.1), BLACK);
    assert_eq!(r.eval(2.0), WHITE, "above last stop holds the last color");
    assert!(
        approx(r.eval(0.5), [0.5, 0.5, 0.5, 1.0], 1e-6),
        "midway between 0.25 and 0.75"
    );
}

#[test]
fn constant_holds_the_left_stop() {
    let r = ColorRamp::new(
        vec![RampStop::new(0.0, BLACK), RampStop::new(1.0, WHITE)],
        RampColorMode::Rgb,
        RampInterp::Constant,
    );
    assert_eq!(r.eval(0.0), BLACK);
    assert_eq!(r.eval(0.49), BLACK, "left of the next stop → left color");
    assert_eq!(r.eval(0.99), BLACK);
    assert_eq!(r.eval(1.0), WHITE, "exactly at the last stop");
}

#[test]
fn ease_is_smoothstep_symmetric() {
    let r = ColorRamp::new(
        vec![RampStop::new(0.0, BLACK), RampStop::new(1.0, WHITE)],
        RampColorMode::Rgb,
        RampInterp::Ease,
    );
    assert!(
        approx(r.eval(0.5), [0.5, 0.5, 0.5, 1.0], 1e-6),
        "smoothstep(0.5)=0.5"
    );
    // Ease pulls toward the ends: at 0.25 the value is below the linear 0.25.
    assert!(r.eval(0.25)[0] < 0.25, "ease-in below linear at t=0.25");
    assert!(r.eval(0.75)[0] > 0.75, "ease-out above linear at t=0.75");
}

#[test]
fn cardinal_passes_through_the_stops() {
    // Catmull–Rom interpolates (passes through) the control stops exactly at their positions.
    let r = ColorRamp::new(
        vec![
            RampStop::new(0.0, BLACK),
            RampStop::new(0.33, RED),
            RampStop::new(0.66, GREEN),
            RampStop::new(1.0, WHITE),
        ],
        RampColorMode::Rgb,
        RampInterp::Cardinal,
    );
    assert!(
        approx(r.eval(0.33), RED, 1e-3),
        "passes through the red stop: {:?}",
        r.eval(0.33)
    );
    assert!(
        approx(r.eval(0.66), GREEN, 1e-3),
        "passes through the green stop"
    );
}

#[test]
fn bspline_stays_in_gamut_and_smooth() {
    let r = ColorRamp::new(
        vec![
            RampStop::new(0.0, BLACK),
            RampStop::new(0.5, RED),
            RampStop::new(1.0, WHITE),
        ],
        RampColorMode::Rgb,
        RampInterp::BSpline,
    );
    for i in 0..=20 {
        let c = r.eval(i as f32 / 20.0);
        assert!(
            c.iter().all(|&x| (0.0..=1.0).contains(&x)),
            "B-spline stays in [0,1]: {c:?}"
        );
    }
}

#[test]
fn hsv_mode_takes_a_hue_path_not_through_grey() {
    // Red → Green in RGB passes through dark/olive; in HSV (Near) it sweeps hue through yellow,
    // keeping saturation high. The midpoints differ.
    let rgb = ColorRamp::new(
        vec![RampStop::new(0.0, RED), RampStop::new(1.0, GREEN)],
        RampColorMode::Rgb,
        RampInterp::Linear,
    );
    let hsv = ColorRamp::new(
        vec![RampStop::new(0.0, RED), RampStop::new(1.0, GREEN)],
        RampColorMode::Hsv,
        RampInterp::Linear,
    );
    let m_rgb = rgb.eval(0.5);
    let m_hsv = hsv.eval(0.5);
    assert_ne!(m_rgb, m_hsv, "HSV interpolation differs from RGB");
    // HSV midpoint of red→green (Near, +60° hue) is yellow-ish: high R and G, low B.
    assert!(
        m_hsv[0] > 0.5 && m_hsv[1] > 0.5 && m_hsv[2] < 0.2,
        "HSV mid is yellow-ish: {m_hsv:?}"
    );
}

#[test]
fn endpoints_are_exact_in_every_mode() {
    for mode in [RampColorMode::Rgb, RampColorMode::Hsv, RampColorMode::Hsl] {
        for interp in [
            RampInterp::Linear,
            RampInterp::Ease,
            RampInterp::Cardinal,
            RampInterp::BSpline,
            RampInterp::Constant,
        ] {
            let r = ColorRamp::new(
                vec![RampStop::new(0.0, RED), RampStop::new(1.0, GREEN)],
                mode,
                interp,
            );
            assert!(
                approx(r.eval(0.0), RED, 1e-4),
                "{mode:?}/{interp:?} start exact"
            );
            assert!(
                approx(r.eval(1.0), GREEN, 1e-4),
                "{mode:?}/{interp:?} end exact"
            );
        }
    }
}

#[test]
fn add_remove_keep_sorted_and_nonempty() {
    let mut r = ColorRamp::default();
    let i = r.add_stop(RampStop::new(0.5, RED));
    assert_eq!(i, 1, "0.5 inserts between 0.0 and 1.0");
    assert_eq!(r.len(), 3);
    assert!(
        r.stops().windows(2).all(|w| w[0].pos <= w[1].pos),
        "stays sorted"
    );
    r.remove_stop(1);
    assert_eq!(r.len(), 2);
    // Cannot empty the ramp.
    r.remove_stop(0);
    r.remove_stop(0);
    assert_eq!(r.len(), 1, "always keeps at least one stop");
}

#[test]
fn set_position_resorts() {
    let mut r = ColorRamp::new(
        vec![
            RampStop::new(0.0, BLACK),
            RampStop::new(0.5, RED),
            RampStop::new(1.0, WHITE),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    );
    // Drag the black stop (index 0, pos 0.0) past the red one (0.5) to 0.7.
    let new_i = r.set_position(0, 0.7);
    assert_eq!(new_i, 1, "0.7 lands between red (0.5) and white (1.0)");
    assert_eq!(r.stops()[1].color, BLACK, "the moved stop kept its color");
    assert!(
        r.stops().windows(2).all(|w| w[0].pos <= w[1].pos),
        "stays sorted"
    );
}

#[test]
fn bake_lut_matches_eval() {
    let r = ColorRamp::new(
        vec![
            RampStop::new(0.0, RED),
            RampStop::new(0.5, GREEN),
            RampStop::new(1.0, [0.0, 0.0, 1.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    );
    let mut lut = [[0.0f32; 4]; 256];
    r.bake_into(&mut lut);
    assert_eq!(lut[0], RED);
    assert_eq!(lut[255], [0.0, 0.0, 1.0, 1.0]);
    for i in (0..256).step_by(17) {
        let t = i as f32 / 255.0;
        assert!(
            approx(lut[i], r.eval(t), 1e-6),
            "lut[{i}] matches eval({t})"
        );
    }
}

#[test]
fn hue_paths_differ() {
    let mut near = ColorRamp::new(
        vec![RampStop::new(0.0, RED), RampStop::new(1.0, GREEN)],
        RampColorMode::Hsv,
        RampInterp::Linear,
    );
    near.hue = RampHue::Near;
    let mut far = near.clone();
    far.hue = RampHue::Far;
    // Near red→green goes via yellow (mid R+G high); Far goes the long way via magenta/blue.
    assert_ne!(
        near.eval(0.5),
        far.eval(0.5),
        "Near and Far hue arcs differ"
    );
}
