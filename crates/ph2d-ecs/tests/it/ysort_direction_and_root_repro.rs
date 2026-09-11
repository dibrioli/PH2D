//! Reproduction harness for the 2026-05-31 sorting audit (Enio report:
//! "after using Y-Sort the hierarchy z-order broke and the sprite lower
//! on screen goes to the BACK"). Two independent claims under test:
//!
//! 1. DIRECTION — with the canonical `YSort::default().axis == (0,-1)`
//!    and a **Y-up** world (camera.rs §11.1: higher world Y = higher on
//!    screen), a sprite **lower on screen (smaller world Y)** must rank
//!    HIGHER (drawn in front). This asserts the axis flip is correct.
//!
//! 2. ROOT INERTNESS — `ysort_key` only consults a *strict ancestor*'s
//!    YSort. Editor sprites spawn as flat roots, so enabling Y-Sort on a
//!    root sprite (what the Inspector §7 toggle does) currently has NO
//!    effect on that sprite. This test documents the live behavior so the
//!    fix (self-inclusive Y-Sort) can flip the assertion.

use bevy_ecs::hierarchy::ChildOf;
use ph2d_core::Vec2;
use ph2d_ecs::sort_key::compute_sort_ranks;
use ph2d_ecs::{SortInput, World, YSort};

fn rank_of(ranks: &[(ph2d_ecs::Entity, u32)], e: ph2d_ecs::Entity) -> u32 {
    ranks.iter().find(|(x, _)| *x == e).unwrap().1
}

/// CLAIM 1: with a YSort *parent*, lower-on-screen (smaller world Y)
/// must draw in FRONT (higher rank).
#[test]
fn ysort_parent_lower_on_screen_draws_front() {
    let mut w = World::new();
    let parent = w.spawn((YSort::default(),)).id(); // axis (0,-1), enabled
    let high = w.spawn((ChildOf(parent),)).id(); // higher on screen
    let low = w.spawn((ChildOf(parent),)).id(); // lower on screen

    // world_pos comes from the extract; Y-up so larger Y = higher screen.
    let inputs = vec![
        SortInput {
            entity: high,
            world_pos: Vec2::new(0.0, 100.0),
        },
        SortInput {
            entity: low,
            world_pos: Vec2::new(0.0, -100.0),
        },
    ];
    let ranks = compute_sort_ranks(&w, &inputs);
    assert!(
        rank_of(&ranks, low) > rank_of(&ranks, high),
        "lower-on-screen sprite (y=-100) must rank in front of higher one \
         (y=100) under axis (0,-1); got low={} high={}",
        rank_of(&ranks, low),
        rank_of(&ranks, high),
    );
}

/// CLAIM 2 (FIXED — ADR-0073-amendment-2 self-inclusive Y-Sort): Y-Sort
/// enabled on flat ROOT sprites THEMSELVES (the editor's Inspector §7
/// toggle, since sprites import as roots) now sorts them by projected Y,
/// just like a YSort parent would. Lower-on-screen draws in front.
#[test]
fn ysort_on_root_self_sorts_by_y() {
    let mut w = World::new();
    // Two roots, each with YSort enabled on ITSELF (the Inspector toggle).
    let high = w.spawn((YSort::default(),)).id();
    let low = w.spawn((YSort::default(),)).id();

    let inputs = vec![
        SortInput {
            entity: high,
            world_pos: Vec2::new(0.0, 100.0),
        },
        SortInput {
            entity: low,
            world_pos: Vec2::new(0.0, -100.0),
        },
    ];
    let ranks = compute_sort_ranks(&w, &inputs);
    assert!(
        rank_of(&ranks, low) > rank_of(&ranks, high),
        "self-inclusive Y-Sort: a root sprite lower on screen (y=-100) must \
         rank in front of a higher one (y=100); got low={} high={}",
        rank_of(&ranks, low),
        rank_of(&ranks, high),
    );
}
