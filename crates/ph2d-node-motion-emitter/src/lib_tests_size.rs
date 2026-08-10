//! **How BIG a particle is** — the `size_random` family, split from `lib_tests.rs` at the
//! HR-18 LOC cap along the seam that file already carries: its siblings answer *where is a
//! particle born* and *which way it goes*, this one *what size it is*.
//!
//! A GRANDCHILD of `lib.rs`, so `use super::*` still reaches the fixtures (`spec`/`ids_of`)
//! instead of copying them — a second `spec()` would be a second answer to *what does an
//! untouched emitter look like?*.

use super::*;
use crate::{LANE_SIZE, MANIFEST, MotionEmitter, Spawn, emit, rand01};

fn sizes_of(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("size").unwrap() {
        Column::Vec2(v) => v.clone(),
        other => panic!("size is Vec2, got {other:?}"),
    }
}

/// **`size_random = 0` is the one size that always shipped, to the BIT.**
///
/// ⚠️ Compared by `to_bits`, not by `==`: the claim is that the column did not move at all, and
/// float equality would wave through a `-0.0` or a value one ulp away — exactly the divergence a
/// byte-identity claim exists to forbid. The factor is `1.0` exactly (`jz × 0.0 × 2.0` is a
/// signed zero, and `1.0 + (±0.0)` is `1.0`), and multiplying by `1.0` is exact in IEEE-754.
#[test]
fn a_size_random_of_zero_is_the_one_size_that_always_shipped() {
    let s = spec();
    let out = emit(&s, 3.0);
    let sizes = sizes_of(&out);
    assert!(sizes.len() >= 8, "a real sample: {}", sizes.len());
    for (i, v) in sizes.iter().enumerate() {
        for (k, axis) in v.iter().enumerate() {
            assert_eq!(
                axis.to_bits(),
                s.size.to_bits(),
                "particle {i} axis {k}: {axis} is not the authored {}",
                s.size
            );
        }
    }
}

/// **Turned up, the sizes fill the band `size × (1 ± r)`** — and the band is what makes this a
/// spray rather than a rule: without it every grain of a fountain is the same grain.
///
/// The oracle is the SHAPE of the distribution, not one particle: the extremes have to reach
/// most of the way to both ends, and the mean has to stay put — a draw that only grew (or only
/// shrank) would still spread while quietly re-scaling the whole emitter.
#[test]
fn size_random_fills_the_band_without_moving_the_mean() {
    let mut s = spec();
    s.spawn = Spawn::Continuous { rate: 400.0 };
    s.life = 2.0;
    s.size_random = 1.0;
    let sizes: Vec<f32> = sizes_of(&emit(&s, 3.0)).into_iter().map(|v| v[0]).collect();
    assert!(sizes.len() >= 200, "a real sample: {}", sizes.len());

    let (lo, hi) = sizes
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
    let mean = sizes.iter().sum::<f32>() / sizes.len() as f32;
    // `r = 1` opens the band to `0 .. 2×`; a uniform draw over a few hundred samples lands
    // comfortably inside the outer tenth at each end.
    assert!(lo < 0.1 * s.size, "the small end is reached: {lo}");
    assert!(hi > 1.9 * s.size, "the big end is reached: {hi}");
    assert!(
        (mean - s.size).abs() < 0.05 * s.size,
        "the band is centred on the authored size: mean {mean} vs {}",
        s.size
    );
}

/// **A particle keeps its size while the window slides** — the same identity property the speed
/// and the birthplace have, on the lane the size uses. An index-hashed size would make every
/// grain in the air re-scale the moment an older one died: a shimmer nobody authored.
///
/// ⚠️ The two premises this fixture has to satisfy are the ones its sibling paid for: the gap
/// must be SHORTER than the life (or there is nothing left to compare) and long enough to kill
/// somebody (or the window never slid).
#[test]
fn a_particle_keeps_its_size_while_the_window_slides() {
    let mut s = spec();
    s.life = 0.5;
    s.size_random = 0.9;
    let sample = |t: f32| -> Vec<(u32, [f32; 2])> {
        let out = emit(&s, t);
        ids_of(&out)
            .into_iter()
            .map(|i| i as u32)
            .zip(sizes_of(&out))
            .collect()
    };
    let (early, later) = (sample(1.0), sample(1.3));
    assert!(early[0].0 != later[0].0, "the window really slid");
    let mut shared = 0usize;
    for (id, sz) in &early {
        if let Some((_, other)) = later.iter().find(|(i, _)| i == id) {
            assert_eq!(sz, other, "id {id} kept its size");
            shared += 1;
        }
    }
    assert!(shared >= 3, "the frames overlap: {shared}");
}

/// **How big a particle is is not how fast it leaves.** Sharing one hash lane between the size
/// and the speed ties them together exactly — the big grains would be precisely the fast ones,
/// which reads as a rule the artist did not write rather than as variety.
///
/// ⚠️ The oracle is the emitted COLUMNS, never the hash: asserting `rand01(.., LANE_SIZE) !=
/// rand01(.., LANE_SPEED)` is a fact about the hash function that stays true whichever lane the
/// emitter actually passes, and the mutation that matters (`LANE_SIZE` → `LANE_SPEED` at the
/// call site) sails straight through it. This measures the correlation between each particle's
/// size and its speed, both read back off the stream: **exactly ±1 with the lanes shared.**
#[test]
fn how_big_a_particle_is_is_not_how_fast_it_leaves() {
    let mut s = spec();
    s.spawn = Spawn::Continuous { rate: 400.0 };
    s.life = 2.0;
    s.size_random = 1.0;
    s.speed_random = 1.0;
    let out = emit(&s, 3.0);
    let sizes: Vec<f64> = sizes_of(&out).into_iter().map(|v| v[0] as f64).collect();
    let speeds: Vec<f64> = vels_of(&out)
        .into_iter()
        .map(|v| ((v[0] * v[0] + v[1] * v[1]) as f64).sqrt())
        .collect();
    assert!(sizes.len() >= 200, "a real sample: {}", sizes.len());

    let n = sizes.len() as f64;
    let (ms, mv) = (
        sizes.iter().sum::<f64>() / n,
        speeds.iter().sum::<f64>() / n,
    );
    let (mut cov, mut vs, mut vv) = (0.0, 0.0, 0.0);
    for (a, b) in sizes.iter().zip(&speeds) {
        let (ea, eb) = (a - ms, b - mv);
        cov += ea * eb;
        vs += ea * ea;
        vv += eb * eb;
    }
    let corr = cov / (vs.sqrt() * vv.sqrt());
    assert!(
        corr.abs() < 0.2,
        "size and speed are separate draws: {corr}"
    );
}

/// **A size never goes negative, however far the knob is typed past its slider.**
///
/// ⚠️ This is where `size_random` DIFFERS from its twin `speed_random`, and the difference is
/// not a matter of taste: past `r = 1` the multiplier turns negative, and for a speed that is a
/// picture (the particle launches into the opposite half of the cone) while for a size it is
/// not — a quad of negative side is a winding flip or nothing at all. The vanishing particles
/// this produces are honest; a negative extent would not be.
#[test]
fn an_oversized_size_random_never_makes_a_negative_size() {
    let mut s = spec();
    s.spawn = Spawn::Continuous { rate: 400.0 };
    s.life = 2.0;
    s.size_random = 4.0;
    let sizes: Vec<f32> = sizes_of(&emit(&s, 3.0)).into_iter().map(|v| v[0]).collect();
    assert!(sizes.len() >= 200, "a real sample: {}", sizes.len());
    for (i, &v) in sizes.iter().enumerate() {
        assert!(v >= 0.0, "particle {i} has size {v}");
    }
    // And the floor is REACHED, so the assertion above is not vacuous: at `r = 4` the band would
    // open to `−3 .. 5×`, so most of the lower half is clamped away.
    let floored = sizes.iter().filter(|&&v| v == 0.0).count();
    assert!(floored > 20, "the floor is where the band went: {floored}");
}

/// **The authored `size_random` REACHES the column.** Every gate above builds a `Spec` by hand,
/// so all of them stay green with `ctx.param("size_random")` unread — this walks the seam from
/// `set_param` to a sized particle, and its first half never names the default it is testing.
#[test]
fn the_authored_size_random_reaches_the_column() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionEmitter as &dyn NodeOp)
        }
    }
    let cook_spread = |r: Option<f32>| -> f32 {
        let mut g = Graph::new();
        let em = g.add_node("motion.emitter");
        g.set_param(em, "rate", 400.0);
        g.set_param(em, "life", 2.0);
        g.set_param(em, "size", 0.5);
        if let Some(v) = r {
            g.set_param(em, "size_random", v);
        }
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, em, 3.0).unwrap();
        let sizes = sizes_of(out[0].as_stream());
        let (lo, hi) = sizes
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(v[0]), h.max(v[0])));
        hi - lo
    };
    assert_eq!(cook_spread(None), 0.0, "untouched: one size for everybody");
    assert!(cook_spread(Some(1.0)) > 0.8, "the band arrived");
}

/// The lane is the one the module documents, and it is not one of the four that shipped before
/// it. A silent collision would make the size a copy of another draw — the very thing
/// `how_big_a_particle_is_is_not_how_fast_it_leaves` measures, stated here where it is cheap.
#[test]
fn the_size_lane_is_its_own() {
    for other in [LANE_ANGLE, LANE_SPEED, LANE_SHAPE_U, LANE_SHAPE_V] {
        assert_ne!(LANE_SIZE, other, "the size lane collides with {other}");
    }
    // And it draws something: a constant lane would satisfy every inequality above.
    let draws: Vec<f32> = (0..64).map(|id| rand01(7, id, LANE_SIZE)).collect();
    let (lo, hi) = draws
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
    assert!(hi - lo > 0.8, "the lane varies: {lo}..{hi}");
}

/// The manifest carries the param, and it is the LAST one — appended, so a saved graph reads it
/// as absent and takes the default that reproduces the emitter that always shipped.
#[test]
fn size_random_is_appended_last() {
    let names: Vec<&str> = MANIFEST.params.iter().map(|p| p.name).collect();
    assert_eq!(names.last(), Some(&"size_random"), "params: {names:?}");
    let spec = MANIFEST
        .params
        .iter()
        .find(|p| p.name == "size_random")
        .expect("declared");
    assert_eq!(spec.default, 0.0, "the neutral is the shipped behaviour");
}

/// The GPU kernel is handed the param. Without this the mirror would read a uniform slot that
/// the cook never fills and every particle on the device would come back at the authored size,
/// while the CPU spread — a divergence only the parity gate could see, and only if its fixture
/// happened to vary the size.
#[test]
fn the_kernel_is_handed_the_size_random() {
    let k = crate::GPU_KERNEL;
    assert!(
        k.params.contains(&"size_random"),
        "kernel params: {:?}",
        k.params
    );
    assert!(
        k.wgsl.contains("params.size_random"),
        "the kernel body reads it"
    );
}
