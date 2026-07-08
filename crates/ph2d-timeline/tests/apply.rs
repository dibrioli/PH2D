//! The binding model, headless: a `SpriteAnimation` sampled at a playhead time
//! writes the entity's `Transform` — the real proof that the general timeline
//! animates a scene object (GPU-free).

use ph2d_anim::{AnimValue, Clip, Interp, Key, RationalTime, Track};
use ph2d_core::Vec2;
use ph2d_ecs::{Transform, World};
use ph2d_timeline::{SpriteAnimation, SpriteProp, apply_sprite_animations};

fn ramp(a: f32, b: f32, secs: f64) -> Track {
    Track::new(vec![
        Key {
            t: RationalTime::from_seconds(0.0),
            value: AnimValue::Float(a),
            interp: Interp::Linear,
        },
        Key {
            t: RationalTime::from_seconds(secs),
            value: AnimValue::Float(b),
            interp: Interp::Hold,
        },
    ])
}

#[test]
fn apply_writes_bound_props_leaves_others() {
    let clip = Clip::new(RationalTime::from_seconds(2.0))
        .with_track(SpriteProp::TranslationX.target(), ramp(0.0, 10.0, 2.0))
        .with_track(SpriteProp::Rotation.target(), ramp(0.0, 2.0, 2.0));

    let mut w = World::new();
    // Authored: x=0, y=5. Only x + rotation are animated; y must survive.
    let e = w
        .spawn((
            Transform::from_translation(Vec2::new(0.0, 5.0)),
            SpriteAnimation::new(clip),
        ))
        .id();

    apply_sprite_animations(&mut w, 0.0);
    let xf = *w.get::<Transform>(e).unwrap();
    assert!((xf.translation.x - 0.0).abs() < 1e-5);
    assert!(
        (xf.translation.y - 5.0).abs() < 1e-5,
        "unbound Y must be kept"
    );

    // Midpoint of the 2 s ramps.
    apply_sprite_animations(&mut w, 1.0);
    let xf = *w.get::<Transform>(e).unwrap();
    assert!((xf.translation.x - 5.0).abs() < 1e-5, "X animated to 5.0");
    assert!((xf.rotation - 1.0).abs() < 1e-5, "rotation animated to 1.0");
    assert!((xf.translation.y - 5.0).abs() < 1e-5, "Y still untouched");
}

#[test]
fn looping_wraps_past_duration() {
    let clip = Clip::new(RationalTime::from_seconds(2.0))
        .with_track(SpriteProp::TranslationX.target(), ramp(0.0, 10.0, 2.0));
    let mut w = World::new();
    let e = w
        .spawn((
            Transform::from_translation(Vec2::ZERO),
            SpriteAnimation::new(clip), // looping
        ))
        .id();

    // t = 2.5 s wraps to 0.5 s → x = 2.5.
    apply_sprite_animations(&mut w, 2.5);
    let xf = *w.get::<Transform>(e).unwrap();
    assert!(
        (xf.translation.x - 2.5).abs() < 1e-5,
        "looped to 0.5s → 2.5"
    );
}

#[test]
fn non_looping_holds_past_end() {
    let clip = Clip::new(RationalTime::from_seconds(2.0))
        .with_track(SpriteProp::TranslationX.target(), ramp(0.0, 10.0, 2.0));
    let mut w = World::new();
    let e = w
        .spawn((
            Transform::from_translation(Vec2::ZERO),
            SpriteAnimation::once(clip), // one-shot
        ))
        .id();

    // t = 100 s clamps at the last key → x = 10.
    apply_sprite_animations(&mut w, 100.0);
    let xf = *w.get::<Transform>(e).unwrap();
    assert!((xf.translation.x - 10.0).abs() < 1e-5, "held at end → 10");
}

#[test]
fn empty_world_is_a_noop() {
    let mut w = World::new();
    apply_sprite_animations(&mut w, 1.0); // must not panic
}
