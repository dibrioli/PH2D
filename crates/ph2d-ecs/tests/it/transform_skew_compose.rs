//! Skew compose determinism golden: `blake3(postcard(skew_compose_cases()))` equals the committed
//! `transform_skew_compose.expected` — cross-OS proof that `libm::tanf`/`sincosf` in `compose`
//! are platform-independent (HR-5).

use ph2d_core::Vec2;
use ph2d_ecs::Transform;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn skew_compose_cases() -> [Transform; 3] {
    [
        Transform::compose(
            Transform {
                translation: Vec2::new(10.0, 20.0),
                rotation: std::f32::consts::FRAC_PI_4,
                scale: Vec2::new(2.0, 1.5),
                skew_x: 0.1,
                skew_y: 0.0,
            },
            Transform {
                translation: Vec2::new(5.0, -3.0),
                rotation: 0.2,
                scale: Vec2::new(0.5, 0.5),
                skew_x: 0.0,
                skew_y: 0.05,
            },
        ),
        Transform::compose(
            Transform {
                translation: Vec2::new(-7.0, 4.0),
                rotation: 0.0,
                scale: Vec2::new(3.0, 0.25),
                skew_x: -0.3,
                skew_y: 0.15,
            },
            Transform::IDENTITY,
        ),
        Transform::compose(
            Transform {
                translation: Vec2::new(1.25, -8.5),
                rotation: 1.2,
                scale: Vec2::new(1.0, 1.0),
                skew_x: 0.4,
                skew_y: -0.4,
            },
            Transform {
                translation: Vec2::new(2.0, 2.0),
                rotation: -0.6,
                scale: Vec2::new(1.5, 2.5),
                skew_x: 0.2,
                skew_y: 0.1,
            },
        ),
    ]
}

#[test]
fn transform_compose_skew_byte_identical_cross_os() {
    let bytes = postcard::to_allocvec(&skew_compose_cases()).unwrap();
    let hash = blake3::hash(&bytes);
    let expected = std::fs::read_to_string(fixtures_dir().join("transform_skew_compose.expected"))
        .expect("transform_skew_compose.expected missing — run the ignored generator");
    let expected = expected.trim();
    assert_eq!(
        hash.to_hex().as_str(),
        expected,
        "Transform compose-with-skew is not cross-OS bit-identical. Either the libm \
         migration is incomplete (a `f32::tan`/`sin_cos` slipped into the skew path), \
         the compose math changed, or the golden fixture is stale. DO NOT regenerate \
         without a cross-OS re-verify + ADR amendment (HR-5 contract)."
    );
}
