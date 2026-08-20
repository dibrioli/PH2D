//! **A VARIANTE POR PONTO** (doc 89 folha 08 — the P0).
//!
//! The cartesian product was the only thing this node could say: every shape at
//! every point. Four references ship a variant mode and we shipped none — Blender
//! `Pick Instances` + `Instance Index` (default `id`, wrapping both ways), Houdini
//! `Piece Attribute`, Cavalry `Auto Id`/`Shape Id`, C4D `Modify Clone` — and the
//! sheet's own attempt at a workaround failed for a structural reason: a
//! `motion.cull` on the `shape` port picks a GLOBAL subset, never one shape per
//! point.

use super::*;

/// `n` shapes, each carrying a `size` that names it (`10, 20, 30 …`) — the
/// appearance column is what proves WHICH shape landed, because `P` is summed and
/// would read the same whichever one it was.
fn shapes(n: usize) -> Stream {
    #[expect(clippy::cast_precision_loss, reason = "a tiny fixture index")]
    let sizes: Vec<f32> = (0..n).map(|i| (i + 1) as f32 * 10.0).collect();
    Stream::new(n)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; n]))
        .with("size", Column::Scalar(sizes))
}

/// `n` points on a line, with the `Index` column a real producer would give them.
fn points(n: usize) -> Stream {
    #[expect(clippy::cast_precision_loss, reason = "a tiny fixture index")]
    let p: Vec<[f32; 2]> = (0..n).map(|i| [i as f32, 0.0]).collect();
    #[expect(clippy::cast_precision_loss, reason = "a tiny fixture index")]
    let idx: Vec<f32> = (0..n).map(|i| i as f32).collect();
    Stream::new(n)
        .with("P", Column::Vec2(p))
        .with("Index", Column::Scalar(idx))
}

/// The `size` of every stamp — i.e. which shape each one wears.
fn worn(s: &Stream) -> Vec<f32> {
    match s.get("size") {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("the appearance column has to survive the stamp"),
    }
}

/// **Off is the product, and the seven gates next door already say so** — this
/// one pins that the mode SELECTOR itself cannot change it, which is the part
/// that is new.
#[test]
fn off_is_the_cartesian_product_this_node_always_emitted() {
    let out = duplicate(&shapes(3), &points(4), 4, Pick::Off, 0, 0.0);
    assert_eq!(out.count(), 12, "3 shapes at 4 points is 12 stamps");
    // Shape-major: the first four wear shape 0, and so on.
    assert_eq!(
        worn(&out),
        vec![
            10.0, 10.0, 10.0, 10.0, 20.0, 20.0, 20.0, 20.0, 30.0, 30.0, 30.0, 30.0
        ]
    );
    // And a seed cannot reach a mode that does not consult one.
    for seed in [0u32, 7, 12345] {
        assert_eq!(
            worn(&duplicate(&shapes(3), &points(4), 4, Pick::Off, seed, 0.0)),
            worn(&out),
            "the seed is Random's alone"
        );
    }
}

/// **Cycle deals the shapes around the points** — one stamp per point, wearing
/// `id mod shapes`. This is the reference default in all four tools.
#[test]
fn cycle_gives_each_point_one_shape_and_wraps() {
    let out = duplicate(&shapes(3), &points(7), 7, Pick::Cycle, 0, 0.0);
    assert_eq!(
        out.count(),
        7,
        "a variant mode stamps once per POINT, not the product"
    );
    assert_eq!(worn(&out), vec![10.0, 20.0, 30.0, 10.0, 20.0, 30.0, 10.0]);
}

/// **The pick is a property of the POINT, not of its row.**
///
/// Blender's `Instance Index` defaults to `id` and Houdini's `Piece Attribute` is
/// an attribute for exactly this reason: reorder the points and each keeps the
/// shape it had. Reading the position in the list instead would slide every shape
/// along the moment anything upstream sorted.
#[test]
fn the_shape_travels_with_the_point_through_a_reorder() {
    let straight = points(4);
    // The same four points, rows reversed — `Index` travels with the row, which
    // is what a permuting node (`motion.sort`) leaves behind.
    let reversed = Stream::new(4)
        .with(
            "P",
            Column::Vec2(vec![[3.0, 0.0], [2.0, 0.0], [1.0, 0.0], [0.0, 0.0]]),
        )
        .with("Index", Column::Scalar(vec![3.0, 2.0, 1.0, 0.0]));

    let a = worn(&duplicate(&shapes(3), &straight, 4, Pick::Cycle, 0, 0.0));
    let mut b = worn(&duplicate(&shapes(3), &reversed, 4, Pick::Cycle, 0, 0.0));
    b.reverse();
    assert_eq!(a, b, "reordering the points does not re-deal the shapes");
}

/// **A stream with no `Index` still works** — the position is the id, which is
/// the only answer available and the one a hand-built stream expects.
#[test]
fn a_point_stream_without_an_index_column_falls_back_to_its_position() {
    let bare = Stream::new(5).with("P", Column::Vec2(vec![[0.0, 0.0]; 5]));
    assert_eq!(
        worn(&duplicate(&shapes(2), &bare, 5, Pick::Cycle, 0, 0.0)),
        vec![10.0, 20.0, 10.0, 20.0, 10.0]
    );
}

/// **A NEGATIVE id wraps too, and does not panic.**
///
/// The reference wraps in both directions, and `rem_euclid` is why: a bare `%`
/// on `-1` gives `-1`, which as an index is a panic on the very stream an
/// upstream node could hand us.
#[test]
fn a_negative_id_wraps_the_other_way_instead_of_panicking() {
    let neg = Stream::new(4)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; 4]))
        .with("Index", Column::Scalar(vec![-1.0, -2.0, -3.0, -4.0]));
    // -1 mod 3 = 2, -2 -> 1, -3 -> 0, -4 -> 2.
    assert_eq!(
        worn(&duplicate(&shapes(3), &neg, 4, Pick::Cycle, 0, 0.0)),
        vec![30.0, 20.0, 10.0, 30.0]
    );
}

/// **Random scatters the shapes, stays in range, and answers to the seed.**
///
/// The in-range half is the one that matters most: `hash01` is strictly `< 1`, so
/// `floor(h · ns)` can never reach `ns` — and the gate checks it over enough
/// points that an off-by-one at the top would show rather than hide.
#[test]
fn random_scatters_within_range_and_the_seed_chooses_which_scatter() {
    let out = duplicate(&shapes(3), &points(200), 200, Pick::Random, 0, 0.0);
    assert_eq!(out.count(), 200, "still one stamp per point");
    let w = worn(&out);
    assert!(
        w.iter().all(|s| [10.0, 20.0, 30.0].contains(s)),
        "every stamp wears one of the three shapes"
    );
    // All three actually appear — a hash that collapsed to one shape would pass
    // the range check above and be useless.
    for s in [10.0f32, 20.0, 30.0] {
        assert!(w.contains(&s), "shape {s} never came up in 200 draws");
    }
    // A different seed is a different scatter, and the same seed repeats.
    let other = worn(&duplicate(
        &shapes(3),
        &points(200),
        200,
        Pick::Random,
        9,
        0.0,
    ));
    assert_ne!(w, other, "the seed selects the assignment");
    assert_eq!(
        w,
        worn(&duplicate(
            &shapes(3),
            &points(200),
            200,
            Pick::Random,
            0,
            0.0
        )),
        "and the same seed is the same scatter, every cook"
    );
}

/// **The premise the Random range rests on: `hash01` never returns 1.**
///
/// ⚠️ The clamp inside `pick_shape` is defence in depth and provably unreachable
/// — a mutation that loosens it survives every gate here, and correctly so. So
/// what is pinned is the fact that makes it unnecessary: the hash divides a value
/// capped at `2^24 - 1` by `2^24`, so it is strictly below one, so `floor(h · ns)`
/// is at most `ns - 1`. Break THAT and the mode starts wearing a shape that does
/// not exist.
#[test]
fn the_hash_is_strictly_below_one_which_is_why_the_floor_is_in_range() {
    let mut worst = 0.0f32;
    for id in 0..50_000u32 {
        for seed in [0u32, 1, 7, 9999] {
            let h = hash01(id, seed);
            assert!((0.0..1.0).contains(&h), "hash01({id}, {seed}) = {h}");
            worst = worst.max(h);
        }
    }
    assert!(
        worst > 0.99,
        "and it does get close, so the bound is tight: {worst}"
    );
}

/// **The budget divisor follows the MODE.**
///
/// `Off` builds `ns · np` stamps and has to share the ceiling; a variant mode
/// builds exactly `np`, so capping it by `max / ns` there would throw away points
/// for a product that is never built — a silent loss of three quarters of a
/// scatter, with nothing on screen to say why.
#[test]
fn the_variant_modes_do_not_pay_for_a_product_they_never_build() {
    assert_eq!(
        points_within_budget(Pick::Off, 4, 100, 100),
        25,
        "the product shares the ceiling four ways"
    );
    for mode in [Pick::Cycle, Pick::Random] {
        assert_eq!(
            points_within_budget(mode, 4, 100, 100),
            100,
            "{mode:?} stamps once per point, so all 100 fit"
        );
        assert_eq!(
            points_within_budget(mode, 4, 500, 100),
            100,
            "and the ceiling still holds when the points alone exceed it"
        );
    }
}

/// **`Index`/`Count` are renumbered over what was actually stamped**, so a
/// downstream ramp spans the variant set rather than the product that was not
/// built.
#[test]
fn the_renumbering_counts_the_stamps_that_exist() {
    let out = duplicate(&shapes(3), &points(4), 4, Pick::Cycle, 0, 0.0);
    let Some(Column::Scalar(idx)) = out.get("Index") else {
        panic!("Index")
    };
    assert_eq!(idx, &vec![0.0, 1.0, 2.0, 3.0]);
    let Some(Column::Scalar(cnt)) = out.get("Count") else {
        panic!("Count")
    };
    assert_eq!(cnt, &vec![4.0; 4], "Count is the stamped total, not 3 x 4");
}
