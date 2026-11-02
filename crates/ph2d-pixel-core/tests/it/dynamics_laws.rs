//! **As leis da dinâmica** que a fixtura do Trial não isola por coluna: o filtro no VECTOR, a pausa,
//! a rampa da pressão (Pixelorama 1.2.3, linear entre limiares) e o carimbo por ponto do traço.

use ph2d_pixel_core::dither::Dither;
use ph2d_pixel_core::dynamics::{
    Channel, Dab, Dynamics, Sense, Sensor, VELOCITY_FULL_PX, VELOCITY_ZERO_PX, Velocity,
};
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Paint, Sprite, Stroke};

const MS: u64 = 1_000_000;

fn passos(v: &mut Velocity, de: (f64, u64), passo: f64, n: usize, dt: u64) -> (f64, u64) {
    let (mut x, mut t) = de;
    for _ in 0..n {
        x += passo;
        t += dt;
        v.sample([x, 0.0], t);
    }
    (x, t)
}

fn px(v: &Velocity) -> f64 {
    v.value() * VELOCITY_FULL_PX - VELOCITY_ZERO_PX
}

#[test]
fn the_filter_converges_to_the_step_and_does_not_divide_by_dt() {
    for dt in [2, 8, 32, 128] {
        let mut v = Velocity::default();
        passos(&mut v, (0.0, 0), 16.0, 400, dt * MS);
        assert!((px(&v) - 16.0).abs() < 1e-3, "dt {dt} ms: {}", px(&v));
    }
}

#[test]
fn reversing_the_direction_starts_again_from_the_bottom() {
    let mut v = Velocity::default();
    let fim = passos(&mut v, (0.0, 0), 32.0, 60, 8 * MS);
    assert!(px(&v) > 31.0);
    let mut minimo = f64::MAX;
    let (mut x, mut t) = fim;
    for _ in 0..60 {
        x -= 32.0;
        t += 8 * MS;
        v.sample([x, 0.0], t);
        minimo = minimo.min(px(&v));
    }
    assert!(minimo < 6.0, "o vector passa pelo zero: mínimo {minimo}");
    assert!(px(&v) > 31.0, "e volta à velocidade do passo");
}

#[test]
fn a_pause_does_not_decay_and_the_first_event_after_it_weighs_almost_alone() {
    let mut v = Velocity::default();
    let (x, t) = passos(&mut v, (0.0, 0), 64.0, 50, 8 * MS);
    let antes = v.value();
    assert_eq!(v.value(), antes, "sem eventos nada muda");
    v.sample([x + 8.0, 0.0], t + 500 * MS);
    assert!((px(&v) - 8.0).abs() < 0.01, "e^(−500/50) ≈ 0: {}", px(&v));
}

#[test]
fn the_first_event_only_marks_the_start() {
    let mut v = Velocity::default();
    v.sample([100.0, 0.0], 32 * MS);
    assert!(px(&v).abs() < 1e-9);
}

#[test]
fn pressure_and_velocity_share_one_linear_ramp_between_the_handles() {
    let c = Channel {
        sensor: Sensor::Pressure,
        lo: 0.2,
        hi: 0.8,
        min: 1.0,
        max: 9.0,
    };
    let at = |p: f64| {
        c.map(Sense {
            pressure: p,
            velocity: 0.0,
        })
        .expect("ligado")
    };
    assert_eq!(at(0.0), 1.0);
    assert_eq!(at(0.2), 1.0);
    assert!((at(0.5) - 5.0).abs() < 1e-12);
    assert_eq!(at(0.8), 9.0);
    assert_eq!(at(1.0), 9.0);
    let v = Channel {
        sensor: Sensor::Velocity,
        ..c
    };
    assert_eq!(
        v.map(Sense {
            pressure: 0.0,
            velocity: 0.5
        }),
        Some(at(0.5))
    );
    assert_eq!(Channel::default().map(Sense::default()), None, "Off");
    let d = Dynamics {
        size: c,
        ..Dynamics::default()
    };
    let s = |p| Sense {
        pressure: p,
        velocity: 0.0,
    };
    assert_eq!(d.dab(s(0.5)).size, Some(5));
    assert_eq!(d.dab(s(0.49)).size, Some(4), "floor, como o Trial");
    assert_eq!(d.dab(s(0.5)).angle, None);
}

fn sprite() -> Sprite {
    Sprite::new(NewSprite {
        width: 32,
        height: 32,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .expect("sprite")
}

#[test]
fn a_dab_changes_the_size_of_the_points_after_it() {
    let s = sprite();
    let key = (s.layers()[0].id, s.frames()[0].id);
    let mut st = Stroke::begin(&s, key.0, key.1, 1, Paint::default(), (2, 10));
    st.to(&s, 8, 10);
    st.set_dab(Dab {
        size: Some(5),
        ..Dab::default()
    });
    st.to(&s, 20, 10);
    let col = |x: i32| st.cells().filter(|c| c.0 == x).count();
    assert_eq!(col(5), 1, "antes do carimbo: 1");
    assert_eq!(col(15), 5, "depois: 5");
}

#[test]
fn the_gradient_paints_bayer_cells_of_the_front_colour_over_the_back() {
    let s = sprite();
    let key = (s.layers()[0].id, s.frames()[0].id);
    let (fg, bg) = (
        s.value_of_rgba([0, 0, 0, 255]),
        s.value_of_rgba([255, 255, 255, 255]),
    );
    let paint = Paint {
        brush: ph2d_pixel_core::brush::Brush {
            size: 8,
            shape: ph2d_pixel_core::brush::BrushShape::Square,
            ..Default::default()
        },
        ..Paint::default()
    };
    let mut st = Stroke::new(key.0, key.1, fg, paint).with_gradient(bg, Dither::Bayer4);
    st.set_dab(Dab {
        gradient: Some(0.5),
        ..Dab::default()
    });
    st.start(&s, 8, 8);
    for (x, y) in st.cells().collect::<Vec<_>>() {
        let quer = if Dither::Bayer4.level(x, y) < 8 {
            fg
        } else {
            bg
        };
        assert_eq!(st.value_at(&s, x, y), Some(quer), "({x},{y})");
    }
    assert!(st.cells().count() >= 64);
}
