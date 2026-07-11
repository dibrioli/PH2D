//! Save/load bridging for the timeline document (W1.T8 / W4).
//!
//! A [`TargetBinding`](crate::TargetBinding) carries two identities for its
//! object: the live ECS `entity` bits (this session only, `#[serde(skip)]`) and a
//! stable [`WireId`](crate::WireId) that IS serialized. These two functions move
//! between them at the save/load boundary — the ONLY place the document touches a
//! scene identity.
//!
//! The scene mapping lives in the shell, so both take a closure rather than a
//! scene dependency: `ph2d-timeline` never learns what a `SceneDoc` is.

use crate::WireId;
use crate::doc::TimelineDoc;

/// Before saving, stamp each binding's `wire_id` from its live entity, so the
/// serialized document names its objects by a stable id. `wire_of` returns the
/// scene's stable id for a live entity's bits ([`WireId::NULL`] for an entity
/// that has none — e.g. a transient object that will not survive the save).
pub fn stamp_wire_ids(doc: &mut TimelineDoc, wire_of: impl Fn(u64) -> WireId) {
    for b in doc.bindings_mut() {
        b.wire_id = wire_of(b.entity);
    }
}

/// After loading, resolve each binding's live `entity` from its serialized
/// `wire_id`. `entity_of` maps a stable id back to this session's entity bits, or
/// `None` when the object is gone / not yet loaded — those bindings are flagged
/// `missing` (the panel badges them; apply skips them — never a silent no-op).
///
/// Returns how many bindings resolved, so the caller can report "N of M tracks
/// reconnected".
pub fn resolve_entities(doc: &mut TimelineDoc, entity_of: impl Fn(WireId) -> Option<u64>) -> usize {
    let mut resolved = 0;
    for b in doc.bindings_mut() {
        match (!b.wire_id.is_null())
            .then(|| entity_of(b.wire_id))
            .flatten()
        {
            Some(entity) => {
                b.entity = entity;
                b.missing = false;
                resolved += 1;
            }
            None => {
                b.entity = 0;
                b.missing = true;
            }
        }
    }
    resolved
}

/// Session-time identity upkeep — the same wire-id machinery, run per frame
/// instead of at the save/load boundary.
///
/// Two halves: every LIVE binding refreshes its `wire_id` from its object (so
/// the name-hash is already stored when the entity later dies), and every
/// MISSING binding with a known `wire_id` tries to reconnect. This is what lets
/// a track survive its object: deleting the object hides its rows (the snapshot
/// skips missing bindings), and when an object with the same name comes back —
/// the global editor undo restores the world by RESPAWNING, so the same object
/// returns under fresh entity bits — the binding heals and the rows return.
///
/// Returns how many bindings healed. Steady state (nothing missing, `entity_of`
/// never called) allocates nothing.
pub fn refresh_and_heal_bindings(
    doc: &mut TimelineDoc,
    wire_of: impl Fn(u64) -> WireId,
    entity_of: impl Fn(WireId) -> Option<u64>,
) -> usize {
    let mut healed = 0;
    for b in doc.bindings_mut() {
        if !b.missing {
            // Keep NULL from erasing a stored hash: an object that lost its
            // name (or a transient) keeps the last identity it had.
            let w = wire_of(b.entity);
            if !w.is_null() {
                b.wire_id = w;
            }
        } else if !b.wire_id.is_null()
            && let Some(entity) = entity_of(b.wire_id)
        {
            b.entity = entity;
            b.missing = false;
            healed += 1;
        }
    }
    healed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prop::PropKind;
    use crate::state::TimelineState;
    use crate::{TimelineIntent as I, apply_intent};
    use ph2d_anim::{AnimValue, Interp, RationalTime};
    use ph2d_core::Playhead;
    use std::collections::BTreeMap;

    /// A doc bound to two live entities (bits 10 and 20), each with a key.
    fn two_bound() -> TimelineState {
        let mut st = TimelineState::new();
        let mut ph = Playhead::new(1.0 / 60.0);
        for entity in [10u64, 20] {
            apply_intent(
                &mut st,
                &mut ph,
                I::AddKey {
                    entity,
                    prop: PropKind::TranslationX,
                    t: RationalTime::from_seconds(0.0),
                    value: AnimValue::Float(0.0),
                    interp: Interp::Linear,
                },
            );
        }
        st
    }

    #[test]
    fn wire_ids_round_trip_through_a_respawn() {
        // Save: entity 10 -> wire 100, entity 20 -> wire 200.
        let mut st = two_bound();
        let save_map: BTreeMap<u64, u64> = [(10, 100), (20, 200)].into();
        stamp_wire_ids(&mut st.doc, |e| WireId(save_map[&e]));
        assert_eq!(st.doc.bindings()[0].wire_id, WireId(100));
        assert_eq!(st.doc.bindings()[1].wire_id, WireId(200));

        // Serialize + deserialize (entity bits are dropped on the wire).
        let bytes = st.doc.to_bytes().unwrap();
        let mut loaded = TimelineDoc::from_bytes(&bytes).unwrap();
        assert_eq!(
            loaded.bindings()[0].entity,
            0,
            "entity bits are not serialized"
        );

        // Load into a fresh session where the SAME objects respawned with new
        // bits: wire 100 -> entity 11, wire 200 -> entity 21.
        let load_map: BTreeMap<u64, u64> = [(100, 11), (200, 21)].into();
        let n = resolve_entities(&mut loaded, |w| load_map.get(&w.0).copied());
        assert_eq!(n, 2, "both tracks reconnected");
        assert_eq!(loaded.bindings()[0].entity, 11);
        assert_eq!(loaded.bindings()[1].entity, 21);
        assert!(loaded.bindings().iter().all(|b| !b.missing));
    }

    #[test]
    fn an_unresolvable_binding_is_flagged_missing_not_dropped() {
        let mut st = two_bound();
        stamp_wire_ids(&mut st.doc, |e| WireId(e * 10));
        let bytes = st.doc.to_bytes().unwrap();
        let mut loaded = TimelineDoc::from_bytes(&bytes).unwrap();

        // Only the first object is back this session.
        let n = resolve_entities(&mut loaded, |w| (w == WireId(100)).then_some(11));
        assert_eq!(n, 1, "one of two reconnected");
        assert!(!loaded.bindings()[0].missing, "entity 10 came back");
        assert!(loaded.bindings()[1].missing, "entity 20 is gone");
        assert_eq!(loaded.bindings()[1].entity, 0);
        // The track and its keys survive — the binding is dormant, not deleted.
        assert_eq!(loaded.bindings().len(), 2);
    }

    #[test]
    fn a_missing_binding_heals_back_to_a_live_entity_with_its_wire_id() {
        // Session-time upkeep: entity 10's binding got its name-hash stamped
        // while alive (wire 100); the object died (missing) and came back under
        // fresh bits (11) — the binding must reconnect, and the live one (20)
        // must keep refreshing its stamp.
        let mut st = two_bound();
        let healed = refresh_and_heal_bindings(
            &mut st.doc,
            |e| WireId(e * 10), // live stamp: 10→100, 20→200
            |_| None,
        );
        assert_eq!(healed, 0, "nothing missing yet");
        assert_eq!(st.doc.bindings()[0].wire_id, WireId(100), "stamped live");

        // Entity 10 dies (the apply flags it), then respawns as 11 (same name).
        st.doc.bindings_mut()[0].missing = true;
        let healed = refresh_and_heal_bindings(
            &mut st.doc,
            |e| WireId(e * 10),
            |w| (w == WireId(100)).then_some(11),
        );
        assert_eq!(healed, 1, "the dead binding reconnected");
        assert_eq!(st.doc.bindings()[0].entity, 11, "to the fresh bits");
        assert!(!st.doc.bindings()[0].missing);
        // A missing binding whose object is still gone stays dormant, and a
        // NULL live stamp (object lost its name) never erases a stored hash.
        st.doc.bindings_mut()[1].missing = true;
        let healed = refresh_and_heal_bindings(&mut st.doc, |_| WireId::NULL, |_| None);
        assert_eq!(healed, 0);
        assert_eq!(
            st.doc.bindings()[0].wire_id,
            WireId(100),
            "a NULL stamp keeps the hash the binding already had"
        );
        assert!(st.doc.bindings()[1].missing, "still dormant, not dropped");
    }

    #[test]
    fn a_binding_that_never_got_a_wire_id_resolves_to_missing() {
        // stamp with NULL (a transient object) → load can't resolve it.
        let mut st = two_bound();
        stamp_wire_ids(&mut st.doc, |_| WireId::NULL);
        let n = resolve_entities(&mut st.doc, |_| Some(99));
        assert_eq!(n, 0, "a null wire id never resolves, even to a live entity");
        assert!(st.doc.bindings().iter().all(|b| b.missing));
    }
}
