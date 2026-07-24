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

use crate::path::MotionPath;
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
    /// The value this property held when it was **first animated** — the pose the
    /// object was in before the timeline took it over.
    ///
    /// This is the base a clip lane fades in *from* when nothing is under it
    /// (ADR-0115 R5). It is not a nicety: without it, a lane at partial coverage
    /// has to blend toward *something*, and the only other candidate is the
    /// property's type default — which for `TranslationX` is 0, i.e. the parent's
    /// origin. A sprite easing in would fly across the canvas.
    ///
    /// Captured lazily by the apply, on the first frame the binding is live (the
    /// world still holds the authored pose then, because no track drives it yet),
    /// and persisted from there on. Rive shipped without this and had to add
    /// **Capture Base State**; Unreal calls it the **Base Pose**. Appended (v4).
    ///
    /// ⚠️ **It is in the TRACK's units, not the scene's.** For a
    /// [`PropKind::Position`] binding the track measures *distance along the path*,
    /// so `rest` is the distance of the path point nearest the authored pose
    /// ([`MotionPath::project`]) — not a point, and not zero. Zero would be the
    /// START of the trajectory, which is the same failure this field exists to
    /// prevent: a sprite easing in would fly there from wherever it stood.
    pub rest: Option<f32>,
    /// **The trajectory**, for a [`PropKind::Position`] binding and only for one
    /// ([ADR-0141]). `None` on every other kind — and on a Position binding that has
    /// no keys yet, which is a path with no anchors.
    ///
    /// It lives on the BINDING rather than in the clip because the trajectory is a
    /// property of *this object's movement*, not of one clip's take on it: two clips
    /// that both animate the object along it are two timings of the same journey,
    /// which is precisely what the arc-length track expresses.
    ///
    /// Anchor `i` pairs with key `i` of the track, and the number that key holds is
    /// [`MotionPath::arclen_at`]. Appended (v12).
    ///
    /// [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md
    pub path: Option<MotionPath>,
    /// **Auto-orient**: o objeto gira para a tangente do caminho enquanto o percorre
    /// (o *Orient Along Path* do AE). Só faz sentido num binding
    /// [`PropKind::Position`], e é **opt-in** — girar o objeto sem que ninguém peça
    /// reescreve uma pose que o artista autorou.
    ///
    /// ⚠️ Autorado, mas não é a palavra final: [`crate::TimelineDoc::auto_orient`]
    /// **RECUSA** quando a mesma entidade tem uma track de Rotation, porque aí seriam
    /// dois autores do mesmo ângulo e o de trás venceria em silêncio. Appended (v12).
    pub auto_orient: bool,
}

impl crate::doc::TimelineDoc {
    /// **The document forgets an object's track** — remove `target`'s binding and
    /// its track from **every** clip (Enio, 2026-07-22: *"a timeline precisa ser
    /// resetada ao deletar o objeto"*).
    ///
    /// Not [`Self::unbind`]: that is the panel's per-row verb and trims the
    /// ACTIVE clip only. Bindings are document-level — every clip keys the same
    /// objects — so forgetting an object that no longer exists has to sweep the
    /// clips it never shows. A track left behind in an inactive clip is exactly
    /// the stale state that made a deleted object's timeline "totalmente bugada"
    /// for the next object created.
    ///
    /// Returns `true` if the binding existed.
    pub fn purge_binding(&mut self, target: AnimTarget) -> bool {
        let (bindings, clips) = self.purge_parts();
        let Some(pos) = bindings.iter().position(|b| b.target == target) else {
            return false;
        };
        bindings.remove(pos);
        for named in clips {
            named.clip.remove_track(target);
        }
        true
    }
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
            rest: None,         // captured on the first live apply (see the field)
            path: None,         // a Position binding grows one with its first key
            auto_orient: false, // opt-in: girar sem pedido reescreve a pose autorada
        }
    }

    /// Builder: set the stable save-time id.
    #[must_use]
    pub fn with_wire_id(mut self, wire_id: WireId) -> Self {
        self.wire_id = wire_id;
        self
    }
}
