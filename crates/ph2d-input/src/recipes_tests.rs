use std::collections::BTreeSet;

use super::*;
use crate::map::InputMap;
use crate::readers::{PLATFORM_PLAYER_READS, TOP_DOWN_PLAYER_READS};

fn names(r: Recipe) -> BTreeSet<&'static str> {
    r.actions().iter().map(|a| a.name).collect()
}

#[test]
fn the_platformer_recipe_is_what_the_platform_player_reads() {
    let want: BTreeSet<&str> = PLATFORM_PLAYER_READS.into_iter().collect();
    assert_eq!(names(Recipe::Platformer), want);
}

#[test]
fn the_top_down_recipe_is_what_the_top_down_player_reads() {
    let want: BTreeSet<&str> = TOP_DOWN_PLAYER_READS.into_iter().collect();
    assert_eq!(names(Recipe::TopDown), want);
}

#[test]
fn the_shooter_walks_and_jumps_on_the_platform_names_and_adds_fire_and_reload() {
    let platform: BTreeSet<&str> = PLATFORM_PLAYER_READS.into_iter().collect();
    let n = names(Recipe::Shooter);
    let extra: BTreeSet<&str> = n.difference(&platform).copied().collect();
    assert_eq!(extra, BTreeSet::from([RECIPE_FIRE, RECIPE_RELOAD]));
    assert!(names(Recipe::Blank).is_empty());
}

#[test]
fn the_recipe_keyboard_is_the_factory_keyboard() {
    let factory = InputMap::with_player_defaults();
    for r in Recipe::ALL {
        for ra in r.actions() {
            let Some(f) = factory.id(ra.name).and_then(|id| factory.get(id)) else {
                continue;
            };
            let keys: Vec<Binding> = ra
                .bindings
                .iter()
                .copied()
                .filter(|b| matches!(b, Binding::Key(_)))
                .collect();
            assert_eq!(keys, f.bindings, "{r:?}/{}", ra.name);
        }
    }
}

#[test]
fn only_actions_with_a_stick_get_the_dead_zone() {
    for r in Recipe::ALL {
        for ra in r.actions() {
            let stick = ra.bindings.iter().any(|b| matches!(b, Binding::PadAxis { .. }));
            let want: f32 = if stick { 0.2 } else { 0.0 };
            assert_eq!(ra.dead_zone.to_bits(), want.to_bits(), "{r:?}/{}", ra.name);
        }
    }
}

#[test]
fn starting_from_a_recipe_twice_never_reuses_an_id() {
    let mut m = InputMap::with_player_defaults();
    let mut seen: BTreeSet<_> = m.actions().iter().map(|a| a.id).collect();
    for r in [Recipe::Platformer, Recipe::Shooter, Recipe::TopDown] {
        m.start_from(r);
        assert_eq!(m.len(), r.actions().len());
        for a in m.actions() {
            assert!(seen.insert(a.id), "{r:?} reutilizou {:?}", a.id);
        }
    }
    m.start_from(Recipe::Blank);
    assert!(m.is_empty());
}

#[test]
fn starting_from_a_recipe_keeps_the_controls_without_the_gone_ids() {
    let mut m = InputMap::with_player_defaults();
    let jump = m.id("jump").expect("fábrica");
    m.controls_mut().players_can_change_keys = true;
    m.controls_mut().set_shown_in_game(jump, false);
    m.start_from(Recipe::Platformer);
    assert!(m.controls().players_can_change_keys);
    assert!(m.controls().hidden_in_game.is_empty());
    let a = m.get(m.id("move_left").expect("receita")).expect("existe");
    assert!(a.is_analog());
    assert_eq!(a.dead_zone.to_bits(), 0.2_f32.to_bits());
}

#[test]
fn every_recipe_has_its_two_keys() {
    let keys: BTreeSet<_> = Recipe::ALL
        .iter()
        .flat_map(|r| [r.label_key(), r.blurb_key()])
        .collect();
    assert_eq!(keys.len(), 8);
    assert!(keys.iter().all(|k| k.starts_with("input_map_try.recipe.")));
}
