//! Timeline document save/load (W4) — a **sidecar** file for the timeline alone.
//!
//! There is no unified project save today (only the vector scene persists), and
//! the ECS scene has no stable per-entity id (its snapshots are index-based and
//! reallocate entity bits on load). So a binding cannot be reconnected by entity
//! bits across a relaunch. Instead we identify a bound object by its **name** —
//! unique (enforced by `name_unique`) and stable across a reload of the same
//! scene — hashing it into the timeline's `WireId`.
//!
//! This makes the sidecar genuinely useful (the demo scene re-spawns the same
//! names every launch, so tracks reconnect) and forward-compatible: when a real
//! project save + stable-id scheme lands (the B5 follow-up, owned outside this
//! line), only the `wire_of`/`entity_of` closures change.

use ph2d_ecs::{Entity, Name, World};
use ph2d_timeline::{TimelineState, WireId, resolve_entities, stamp_wire_ids};

/// FNV-1a of a name → a stable non-null `WireId`. Names are unique, so this is a
/// stable per-object id; `NULL` is reserved, so a hash that lands on 0 is nudged.
fn wire_id_for_name(name: &str) -> WireId {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut h = OFFSET;
    for b in name.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(PRIME);
    }
    WireId(if h == 0 { 1 } else { h })
}

/// The bound entity's name-derived wire id, or `NULL` when it has no `Name`
/// (a transient object that would not survive a reload anyway).
fn wire_of(world: &World, entity_bits: u64) -> WireId {
    match world.get::<Name>(Entity::from_bits(entity_bits)) {
        Some(name) => wire_id_for_name(name.as_str()),
        None => WireId::NULL,
    }
}

/// Where the sidecar is written. Env-overridable like the vector save.
pub(crate) fn save_path() -> String {
    std::env::var("PH2D_TIMELINE_SAVE_PATH")
        .unwrap_or_else(|_| "ph2d_timeline.postcard".to_string())
}

/// Stamp each binding's wire id from its object's name, then write the document.
/// Returns the number of bound tracks saved.
pub(crate) fn save(timeline: &mut TimelineState, world: &World) -> Result<usize, String> {
    let bytes = serialize(timeline, world)?;
    let n = timeline.doc.bindings().len();
    std::fs::write(save_path(), &bytes).map_err(|e| e.to_string())?;
    Ok(n)
}

/// Read the document back and reconnect its bindings by matching each saved wire
/// id against the live named entities. Returns `(reconnected, total)`.
pub(crate) fn load(
    timeline: &mut TimelineState,
    world: &mut World,
) -> Result<(usize, usize), String> {
    let bytes = std::fs::read(save_path()).map_err(|e| e.to_string())?;
    deserialize(timeline, world, &bytes)
}

/// Stamp wire ids from names and serialize — the pure half of [`save`], testable
/// without touching the filesystem.
fn serialize(timeline: &mut TimelineState, world: &World) -> Result<Vec<u8>, String> {
    stamp_wire_ids(&mut timeline.doc, |bits| wire_of(world, bits));
    timeline.doc.to_bytes()
}

/// Deserialize and reconnect bindings to live entities by name — the pure half of
/// [`load`]. Returns `(reconnected, total)`.
///
/// Selection / history / clipboard are panel state, not persisted — they reset to
/// a clean slate around the loaded document, so an undo after load cannot reach
/// into the previous session.
fn deserialize(
    timeline: &mut TimelineState,
    world: &mut World,
    bytes: &[u8],
) -> Result<(usize, usize), String> {
    let doc = ph2d_timeline::TimelineDoc::from_bytes(bytes)?;

    // Build the name-hash → live-entity map once, then resolve every binding.
    let mut by_wire: std::collections::BTreeMap<u64, u64> = std::collections::BTreeMap::new();
    let mut q = world.query::<(Entity, &Name)>();
    for (e, name) in q.iter(world) {
        by_wire.insert(wire_id_for_name(name.as_str()).0, e.to_bits());
    }

    *timeline = TimelineState::new();
    timeline.doc = doc;
    let total = timeline.doc.bindings().len();
    let resolved = resolve_entities(&mut timeline.doc, |w| by_wire.get(&w.0).copied());
    Ok((resolved, total))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::SimWorld;
    use ph2d_timeline::{PropKind, TimelineIntent as I, apply_intent};

    /// A world with two named sprites; returns their live entity bits by name.
    fn world_with(names: &[&str]) -> (SimWorld, Vec<u64>) {
        let mut sim = SimWorld::new();
        let bits = names
            .iter()
            .map(|n| sim.world_mut().spawn(Name::new(*n)).id().to_bits())
            .collect();
        (sim, bits)
    }

    fn key(timeline: &mut TimelineState, entity: u64) {
        let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
        apply_intent(
            timeline,
            &mut ph,
            I::AddKey {
                entity,
                prop: PropKind::TranslationX,
                t: ph2d_anim::RationalTime::from_seconds(0.0),
                value: ph2d_anim::AnimValue::Float(1.0),
                interp: ph2d_anim::Interp::Linear,
            },
        );
    }

    #[test]
    fn a_track_reconnects_by_name_after_the_entities_respawn() {
        // Session 1: two named sprites, one track each.
        let (save_world, save_bits) = world_with(&["sprite_001", "sprite_002"]);
        let mut timeline = TimelineState::new();
        key(&mut timeline, save_bits[0]);
        key(&mut timeline, save_bits[1]);
        let bytes = serialize(&mut timeline, save_world.world()).unwrap();

        // Session 2: the SAME names respawn with DIFFERENT entity bits (a few
        // throwaway entities offset the allocator so the bits genuinely differ).
        let mut sim2 = SimWorld::new();
        for _ in 0..3 {
            sim2.world_mut().spawn(());
        }
        let load_bits: Vec<u64> = ["sprite_001", "sprite_002"]
            .iter()
            .map(|n| sim2.world_mut().spawn(Name::new(*n)).id().to_bits())
            .collect();
        let mut load_world = sim2;
        assert_ne!(save_bits, load_bits, "a respawn hands out fresh bits");
        let mut loaded = TimelineState::new();
        let (n, total) = deserialize(&mut loaded, load_world.world_mut(), &bytes).unwrap();
        assert_eq!((n, total), (2, 2), "both tracks reconnected by name");
        // Each binding now points at THIS session's entity for its name.
        for (b, want) in loaded.doc.bindings().iter().zip(&load_bits) {
            assert_eq!(b.entity, *want);
            assert!(!b.missing);
        }
    }

    #[test]
    fn a_track_whose_object_is_gone_loads_missing_not_dropped() {
        let (save_world, save_bits) = world_with(&["sprite_001", "sprite_002"]);
        let mut timeline = TimelineState::new();
        key(&mut timeline, save_bits[0]);
        key(&mut timeline, save_bits[1]);
        let bytes = serialize(&mut timeline, save_world.world()).unwrap();

        // Session 2: only the first sprite is back.
        let (mut load_world, _) = world_with(&["sprite_001"]);
        let mut loaded = TimelineState::new();
        let (n, total) = deserialize(&mut loaded, load_world.world_mut(), &bytes).unwrap();
        assert_eq!(
            (n, total),
            (1, 2),
            "one reconnected, the other survives as missing"
        );
        assert!(!loaded.doc.bindings()[0].missing);
        assert!(loaded.doc.bindings()[1].missing);
    }

    #[test]
    fn load_resets_panel_state_so_undo_cannot_cross_sessions() {
        let (world, bits) = world_with(&["sprite_001"]);
        let mut timeline = TimelineState::new();
        key(&mut timeline, bits[0]);
        let bytes = serialize(&mut timeline, world.world()).unwrap();
        // A dirty session: something selected + an undo step banked.
        let (mut world2, _) = world_with(&["sprite_001"]);
        let mut loaded = TimelineState::new();
        key(&mut loaded, bits[0]); // bank a history entry
        deserialize(&mut loaded, world2.world_mut(), &bytes).unwrap();
        assert!(!loaded.history.can_undo(), "history reset around the load");
        assert!(
            loaded.selection.is_empty(),
            "selection reset around the load"
        );
    }
}
