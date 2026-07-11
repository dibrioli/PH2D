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
    fn a_binding_that_never_got_a_wire_id_resolves_to_missing() {
        // stamp with NULL (a transient object) → load can't resolve it.
        let mut st = two_bound();
        stamp_wire_ids(&mut st.doc, |_| WireId::NULL);
        let n = resolve_entities(&mut st.doc, |_| Some(99));
        assert_eq!(n, 0, "a null wire id never resolves, even to a live entity");
        assert!(st.doc.bindings().iter().all(|b| b.missing));
    }
}
