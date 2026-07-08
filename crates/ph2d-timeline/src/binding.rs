//! [`TargetBinding`] — the document-level link from an animation target to a
//! concrete scene object + property.
//!
//! A binding says "the track named by `target` drives *this* entity's *this*
//! property". The `target` is an **opaque handle allocated by the document**
//! ([`crate::TimelineDoc::bind`]) — truly meaningless (HR-8), so two entities
//! animating the same [`PropKind`] get distinct tracks in one clip (the reason
//! the doc allocates rather than deriving the target from the prop).
//!
//! A binding carries two identities for its object: the live ECS `entity` bits
//! (runtime, rebuilt each session) and a stable `wire_id` that survives
//! save/load (resolved back to an entity on open — W1.T8). A binding whose
//! entity no longer resolves is flagged `missing` rather than silently
//! no-oping (P6).

use ph2d_anim::AnimTarget;
use serde::{Deserialize, Serialize};

use crate::prop::PropKind;

/// A stable, save-surviving identity for a bound scene object. Resolved back to
/// a live ECS entity when the project opens (W1.T8 maps this to the SceneDoc
/// wire id); `0` is the null id (no object yet resolved).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WireId(pub u64);

impl WireId {
    /// The null wire id (unresolved).
    pub const NULL: WireId = WireId(0);

    /// `true` if this is the null (unresolved) id.
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }
}

/// One document binding: which entity + property an allocated target drives. A
/// binding is unique per `(entity, prop)` within a clip.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TargetBinding {
    /// The opaque animation target the clip's track is keyed by (allocated by
    /// the document; stable for the doc's lifetime + across save/load).
    pub target: AnimTarget,
    /// The property this binding drives.
    pub prop: PropKind,
    /// Stable save-time identity of the bound object (serialized).
    pub wire_id: WireId,
    /// Live ECS entity bits for this session (runtime only — rebuilt from
    /// `wire_id` on load, so never serialized).
    #[serde(skip)]
    pub entity: u64,
    /// `true` when `entity` could not be resolved this session (dead / not yet
    /// loaded). The panel shows a "missing" badge; apply skips it (never a
    /// silent no-op — P6).
    #[serde(skip)]
    pub missing: bool,
}

impl TargetBinding {
    /// A binding of an allocated `target` to a live entity + property. Callers
    /// go through [`crate::TimelineDoc::bind`], which allocates the target and
    /// keeps `(entity, prop)` unique.
    #[must_use]
    pub fn new(target: AnimTarget, entity: u64, prop: PropKind) -> Self {
        Self {
            target,
            prop,
            wire_id: WireId::NULL,
            entity,
            missing: false,
        }
    }

    /// Builder: set the stable save-time id.
    #[must_use]
    pub fn with_wire_id(mut self, wire_id: WireId) -> Self {
        self.wire_id = wire_id;
        self
    }
}
