//! **The point gizmo** — the grabbable dots at every joint's two world anchors.
//!
//! The three `GizmoView` publishers ([`super::paint_sprite_gizmo`] and friends)
//! are all BOXES with scale/rotate handles, built from drawable geometry (a
//! sprite quad, a vector bbox). A physics joint is a *point*: it carries a
//! `Transform` (its authored anchor) but no geometry, so it publishes no box and
//! — until this — had no canvas handle at all. Its anchor was authorable only by
//! typing into the Inspector's Position fields.
//!
//! # Two ends, one vocabulary (W-J2)
//!
//! A joint binds **two** bodies and each end attaches somewhere on its own, so
//! one handle could only ever author half of it. The second dot is body B's
//! anchor — and it is drawn in the **same amber**, as a hollow ring rather than
//! a filled dot, because the two are the same kind of thing at two ends. That is
//! the vocabulary the joint overlay already speaks (W-J1 draws A's ownership line
//! solid and B's dashed): *solid is A, open is B*, said once and meant twice. Two
//! hues would have claimed they are different kinds of thing.
//!
//! ⚠️ **A Pin at rest has both anchors at the SAME point** — two bodies sharing a
//! place is what a pin is. So the marks are drawn concentric and the hit rects
//! are nested: A takes the inner square, B the band outside it. Nudging one dot
//! aside to make room would draw an anchor where it is not.
//!
//! # Every joint, not the selected one (W-J2b)
//!
//! The view carries a **list**. A joint has no sprite, so a canvas click could
//! never reach it through `pick_sprites_at_world` — which meant the only way to
//! get its handles on screen was to hunt for it in the Hierarchy first, and a
//! handle you must find somewhere else before you can grab it is a handle that
//! is not on the canvas at all (Enio, 2026-07-25).
//!
//! Several joints therefore register the same two kinds of handle in one frame,
//! and a hit id must say *which*. That question already has an answer here:
//! `gizmo::paint::keyed_handle_id` gives every EXTRA selection its own id space
//! by hashing the entity bits, and the shell resolves the hit through the map
//! the painter filled while painting. These dots do the same —
//! [`point_handle_id`] — for the same reason and with the same failure mode
//! avoided (a linear scrambler makes consecutive ids collide; see that
//! function's note).

use super::camera::world_to_screen_px;
use super::hit::ids;
use crate::interaction::HitIndex;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::Theme;
use ph2d_vector::{Affine, BezPath, Circle, Color as VelloColor, Point, Stroke, VectorScene};
use std::collections::BTreeMap;

/// What a point handle authors. Editor-core's own word for it — the gizmo layer
/// knows there are grabbable points on a joint and nothing else about physics;
/// the shell maps this onto the sides and the fields.
///
/// # The two families, and the difference is what a QUANTITY is (W-J3)
///
/// The **anchors** are places by nature: the dot is the value. The **parameter**
/// handles are numbers the §12 rows also edit — an angle (a limit wall) and a
/// distance (a rest / max length) — and they are here because *an angle and a
/// distance already have places*: dragging the wall to 30° needs no scale to
/// convert anything, and neither does dragging the ring to 2 m.
///
/// ⚠️ **That is exactly why the motor's speed is NOT in this list** — see the
/// note on `render_loop::point_gizmo::joint_param_handles`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PointHandleKind {
    /// The anchor on body A — the filled dot.
    AnchorA,
    /// The anchor on body B — the hollow ring.
    AnchorB,
    /// The **lower** wall of a hinge's limit arc.
    LimitMin,
    /// The **upper** wall of a hinge's limit arc.
    LimitMax,
    /// The length ring — a spring's rest length, a rope's maximum.
    Length,
    /// **O CENTRO de uma roldana** (W-Pulley W1) — onde ela está.
    ///
    /// Não é uma âncora e não é um parâmetro do joint: uma roldana é uma
    /// ENTIDADE própria, e este dot é o gesto de canvas que a move. Ela não tem
    /// sprite, então o gizmo de caixa não a alcança — o mesmo vão que o dot da
    /// âncora de joint existe para cobrir.
    WheelCentre,
    /// **O ARO de uma roldana** — o raio dela.
    ///
    /// O segundo ponto que o artista pediu: *"um ponto central para deslocamento
    /// e um ponto no raio externo para definir o tamanho"*. Arrastar para fora
    /// engorda a roda, o que muda por onde a corda passa e quanto dela existe.
    WheelRim,
    /// **O aro de SAÍDA de um tambor diferencial** (W-Pulley W6) — o segundo
    /// diâmetro, o que a corda LARGA.
    ///
    /// ⚠️ **Ele é o denominador da vantagem mecânica**, então esta alça é a única
    /// do app cujo arrasto muda quanta força a máquina faz: `2·R/r` no rig
    /// composto do W5. Até aqui o número só era digitável, e uma vantagem que se
    /// digita é uma vantagem que não se descobre desenhando.
    ///
    /// Oferecida **só** quando a roldana tem um segundo raio. Numa comum ele
    /// cairia exatamente sobre o de entrada — duas alças no mesmo pixel são uma
    /// alça que às vezes faz outra coisa.
    WheelRimOut,
}

impl PointHandleKind {
    /// True for the two ANCHOR handles. The shell branches on this to choose the
    /// door a drag writes through (the bridge's anchor door vs the joint
    /// component), and asking it here keeps that a property of the kind rather
    /// than a list every caller re-derives.
    #[must_use]
    pub fn is_anchor(self) -> bool {
        matches!(self, Self::AnchorA | Self::AnchorB)
    }

    /// True para as duas alças de ROLDANA — a terceira família, e a única que
    /// autora uma ENTIDADE que não é o joint.
    #[must_use]
    pub fn is_wheel(self) -> bool {
        matches!(self, Self::WheelCentre | Self::WheelRim | Self::WheelRimOut)
    }

    /// **Esta alça dimensiona um RAIO da roldana?** — as duas que medem do centro
    /// dela, contra a que a MOVE.
    ///
    /// Porta única: o `open_drag` pergunta para escolher a aritmética de agarre e
    /// o apply pergunta para escolher onde o número pousa. Enumerar os dois lados
    /// à mão é como o terceiro raio nasceria mexendo em metade dos sítios.
    #[must_use]
    pub fn is_wheel_radius(self) -> bool {
        matches!(self, Self::WheelRim | Self::WheelRimOut)
    }
}

/// One grabbable point: where it is, whose it is, and what it authors.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PointHandle {
    /// The owning entity (`Entity::to_bits`), opaque here. It is what makes two
    /// joints' handles distinguishable, both in the hit id and in the map the
    /// shell reads back.
    pub key: u64,
    pub kind: PointHandleKind,
    /// World position of the value this handle authors.
    pub world: [f32; 2],
}

/// The point handles to draw this frame — the anchor gizmo.
///
/// Carries only the camera fields the projection needs. A point has no bbox and
/// no rotation, which is exactly why it could not be a [`super::GizmoView`]: that
/// type is a rotated box, and there is nothing here to rotate.
#[derive(Clone, Debug, PartialEq)]
pub struct PointGizmoView {
    /// Every anchor on screen, in a stable order. Empty is not published (the
    /// shell hands out `None` instead), so a non-empty list is the invariant.
    pub handles: Vec<PointHandle>,
    /// The snap candidate the live drag has caught, if any — drawn as a
    /// crosshair through the dot so the artist can see *why* it stopped moving
    /// freely. `None` whenever nothing is snapped (including when no drag is in
    /// flight).
    pub snap_world: Option<[f32; 2]>,
    pub camera_center: [f32; 2],
    pub camera_height_world: f32,
    pub window_w: f32,
    pub window_h: f32,
    /// Canvas rect in screen px — carried for symmetry with [`super::GizmoView`]
    /// (a future scissor against chrome would read it; the dot ignores it today).
    pub canvas: Rect,
    /// **Out of reach this frame** — draw the marks, register nothing.
    ///
    /// Set while another canvas gesture owns the pointer (today: the W-J4 joint
    /// drawing). The handles stay VISIBLE because during that gesture the artist
    /// wants to see where the existing anchors are — that is how you avoid
    /// stacking a second joint on top of one — but a press must not move them:
    /// the drawing gesture is modal and precedes the gizmos, so a handle that
    /// still caught the press would be a mark that the pointer visibly ignores.
    ///
    /// ⚠️ **One flag, both halves, in the same function.** The dimming and the
    /// missing hit rect are two expressions of a single fact, so nothing can end
    /// up dimmed-but-live (the *"dim is not a refusal"* failure this repo has paid
    /// for repeatedly) or live-but-invisible.
    pub inert: bool,
}

/// Visual radius of the anchor dot, screen px.
///
/// Sized up from 6 on Enio's smoke of W-J2 (*"os círculos das pontas precisam
/// ser maiores"*): a mark you have to aim at is a mark you have to find first,
/// and these are now offered for every joint in the scene rather than for the
/// one already selected. 9 px puts the dot's extent at 1.5× the box gizmo's
/// `HANDLE_SIZE_PX` corner square (12), so it reads as a deliberate grab target
/// next to one rather than as a marker.
const JOINT_ANCHOR_DOT_PX: f32 = 9.0;

/// Visual radius of the B ring, screen px — outside A's dot, so a coincident
/// pair reads as one mark inside another rather than as one mark. Holds the
/// same 5:3 ratio to the dot that the pair had at 6/10, which is what keeps the
/// concentric reading legible at the new size.
const JOINT_ANCHOR_RING_PX: f32 = 15.0;

/// Stroke width of the B ring, screen px. Wider than the 1.5 the pair shipped
/// with, so the bigger circle keeps the same visual weight instead of thinning
/// into a hairline.
const JOINT_ANCHOR_RING_STROKE_PX: f64 = 2.0;

/// Arm length of the snap crosshair, screen px — past the B ring, so the mark is
/// legible even when both handles sit on the snapped point.
const SNAP_CROSS_PX: f32 = 20.0;

/// Visual radius of a PARAMETER grip, screen px — smaller than either anchor
/// mark, because it is a grip ON geometry the overlay already drew (the limit
/// wall, the length ring) rather than a mark of its own. It says *this line can
/// be moved*; the line itself says what the value is.
const JOINT_PARAM_GRIP_PX: f32 = 6.0;

/// Half-extent of a handle's hit square, screen px, **by kind**.
///
/// ⚠️ These are the VISUAL radii, deliberately: a mark drawn larger than the
/// rect that catches it is a dot the artist clicks and nothing happens. A takes
/// the inner square and B the band outside it, which is the whole of how a
/// coincident pair stays two handles.
const fn hit_half_px(kind: PointHandleKind) -> f32 {
    match kind {
        PointHandleKind::AnchorA => JOINT_ANCHOR_DOT_PX,
        PointHandleKind::AnchorB => JOINT_ANCHOR_RING_PX,
        // A parameter grip gets a touch more than it draws: it sits on a thin
        // line, so the extra couple of pixels are what make it catchable
        // without widening the mark into the arc it grips.
        PointHandleKind::LimitMin | PointHandleKind::LimitMax | PointHandleKind::Length => {
            JOINT_PARAM_GRIP_PX + 2.0
        }
        // Uma roldana é desenhada MAIOR que uma âncora (é uma roda, não um
        // ponto de amarração), e o alvo acompanha o desenho pela mesma razão
        // que o resto desta tabela existe.
        PointHandleKind::WheelCentre | PointHandleKind::WheelRim | PointHandleKind::WheelRimOut => {
            JOINT_ANCHOR_RING_PX * 2.0
        }
    }
}

/// The hit id of one joint's handle — `canonical ^ hash(key)`.
///
/// ⚠️ **The multipliers are odd and DIFFERENT per kind, and none is the one the
/// box gizmo's extras use.** The failure this avoids is documented at
/// `gizmo::paint::keyed_handle_id`: a *linear* scrambler (`canonical ^ bits ^
/// CONST`) cancels when two ids are compared, so consecutive entity bits and
/// consecutive canonical ids collide constantly — which is how a click on one
/// sprite's handle came to resolve to a different sprite in 2026-06. Multiplying
/// is non-linear, so consecutive keys land far apart; a distinct constant per
/// kind means the kinds hash independently rather than differing by the one bit
/// that separates their canonical ids.
#[must_use]
pub fn point_handle_id(key: u64, kind: PointHandleKind) -> NodeId {
    let (canonical, mul) = match kind {
        PointHandleKind::AnchorA => (ids::GIZMO_JOINT_ANCHOR, 0x_C2B2_AE3D_27D4_EB4F_u64),
        PointHandleKind::AnchorB => (ids::GIZMO_JOINT_ANCHOR_B, 0x_D6E8_FEB8_6659_FD93_u64),
        PointHandleKind::LimitMin => (ids::GIZMO_JOINT_LIMIT_MIN, 0x_A24B_AF11_9E37_79B1_u64),
        PointHandleKind::LimitMax => (ids::GIZMO_JOINT_LIMIT_MAX, 0x_8F51_2C6D_B3A7_45C9_u64),
        PointHandleKind::Length => (ids::GIZMO_JOINT_LENGTH, 0x_F1B7_39D5_6C82_A0E3_u64),
        PointHandleKind::WheelCentre => (ids::GIZMO_WHEEL_CENTRE, 0x_9E37_79B9_7F4A_7C15_u64),
        PointHandleKind::WheelRim => (ids::GIZMO_WHEEL_RIM, 0x_BF58_476D_1CE4_E5B9_u64),
        PointHandleKind::WheelRimOut => (ids::GIZMO_WHEEL_RIM_OUT, 0x_94D0_49BB_1331_11EB_u64),
    };
    NodeId(canonical.0 ^ key.wrapping_mul(mul))
}

/// Amber — the joint overlay's colour, so the grabbable dot reads as "the thing
/// you already see in the overlay, now grab it" rather than as a new element.
/// Theme-independent for the same reason the pivot ring is (the meaning does not
/// change between Forge / Workshop / Sunstone / Blueprint).
fn anchor_color() -> VelloColor {
    VelloColor::from_rgba8(0xFA, 0xBF, 0x40, 0xFF) // matches `JOINT_RGBA` in the physics overlay
}

/// Alpha of an **inert** handle (see [`PointGizmoView::inert`]).
///
/// A rung on the ladder the physics overlay already uses, not a new number:
/// `JOINT_GHOST_RGBA` is 0.28 (*"this is a projection, not a thing"*) and
/// `JOINT_DIM_RGBA` is 0.5 (*"a secondary line of something live"*). An inert
/// handle is neither — it marks a REAL anchor that is out of reach — so it sits
/// between them, clearly weaker than the live mark (which is fully opaque) so the
/// artist reads *"not now"* without the anchor disappearing.
const INERT_ALPHA: u8 = 0x59; // 0.35

/// The mark's colour, dimmed when the handle cannot be grabbed. **The single door**
/// for both — a second `if inert` at a call site is how one mark ends up live-looking
/// and unclickable.
fn handle_color(inert: bool) -> VelloColor {
    if inert {
        VelloColor::from_rgba8(0xFA, 0xBF, 0x40, INERT_ALPHA)
    } else {
        anchor_color()
    }
}

/// Draw every joint's point handles and register their hit rects, recording
/// `id -> handle` in `hit_map` so a Down can be resolved back to the joint and
/// what it authors.
///
/// Order is load-bearing twice. The **snap crosshair first** (it is a backdrop
/// for the marks that sit on it), then the kinds in [`PAINT_ORDER`] —
/// `HitIndex::hit` walks backwards, so the last registration wins, and A must
/// win the square it shares with B on a coincident pair. One pass per kind
/// rather than per-joint interleaving: with a single pass the next joint's B
/// would be registered after this joint's A and would swallow it wherever two
/// joints overlap.
pub fn paint_point_gizmo(
    scene: &mut VectorScene,
    view: &PointGizmoView,
    theme: Theme,
    hit_index: &mut HitIndex,
    hit_map: &mut BTreeMap<NodeId, PointHandle>,
) {
    let _ = theme; // colour is theme-independent (see `anchor_color`)
    let project = |w: [f32; 2]| {
        world_to_screen_px(
            view.camera_center,
            view.camera_height_world,
            view.window_w,
            view.window_h,
            w,
        )
    };
    if let Some(snap) = view.snap_world {
        paint_snap_cross(scene, project(snap));
    }
    for kind in PAINT_ORDER {
        for h in view.handles.iter().filter(|h| h.kind == kind) {
            let s = project(h.world);
            let half = hit_half_px(kind);
            let id = point_handle_id(h.key, kind);
            // ⚠️ An inert view registers NOTHING — not the rect and not the map
            // entry. The rect is the load-bearing half (no rect, no hit), and the
            // map is skipped with it so a stale `id -> handle` cannot answer a
            // press that some other path produced.
            if !view.inert {
                hit_index.register(
                    id,
                    Rect::new(s[0] - half, s[1] - half, half * 2.0, half * 2.0),
                );
                hit_map.insert(id, *h);
            }
            let centre = Point::new(f64::from(s[0]), f64::from(s[1]));
            match kind {
                // Hollow — the B end, in the same amber as A (module docs).
                PointHandleKind::AnchorB => {
                    let ring = Circle::new(centre, f64::from(JOINT_ANCHOR_RING_PX));
                    scene.inner_mut().stroke(
                        &Stroke::new(JOINT_ANCHOR_RING_STROKE_PX),
                        Affine::IDENTITY,
                        handle_color(view.inert),
                        None,
                        &ring,
                    );
                }
                // As alças de ROLDANA: anéis GROSSOS, do dobro do raio de uma
                // âncora — uma roda, não um ponto de amarração. Mesmo âmbar,
                // porque são parte do mesmo vínculo.
                PointHandleKind::WheelCentre
                | PointHandleKind::WheelRim
                | PointHandleKind::WheelRimOut => {
                    let ring = Circle::new(centre, f64::from(JOINT_ANCHOR_RING_PX * 2.0));
                    scene.inner_mut().stroke(
                        &Stroke::new(JOINT_ANCHOR_RING_STROKE_PX * 1.5),
                        Affine::IDENTITY,
                        handle_color(view.inert),
                        None,
                        &ring,
                    );
                }
                // Filled dot — the A end.
                PointHandleKind::AnchorA => {
                    let dot = Circle::new(centre, f64::from(JOINT_ANCHOR_DOT_PX));
                    scene.inner_mut().fill(
                        ph2d_vector::Fill::NonZero,
                        Affine::IDENTITY,
                        handle_color(view.inert),
                        None,
                        &dot,
                    );
                }
                // A grip on the overlay's own line — small, filled, same amber.
                PointHandleKind::LimitMin | PointHandleKind::LimitMax | PointHandleKind::Length => {
                    let dot = Circle::new(centre, f64::from(JOINT_PARAM_GRIP_PX));
                    scene.inner_mut().fill(
                        ph2d_vector::Fill::NonZero,
                        Affine::IDENTITY,
                        handle_color(view.inert),
                        None,
                        &dot,
                    );
                }
            }
        }
    }
}

/// Back-to-front paint (and therefore registration) order.
///
/// The **anchors go last**, so where a parameter grip and an anchor overlap the
/// anchor wins: a limit wall can be dragged from anywhere along its tick, while
/// the anchor is a single point with nowhere else to go. Within the anchors, A
/// after B — see [`paint_point_gizmo`].
const PAINT_ORDER: [PointHandleKind; 5] = [
    PointHandleKind::LimitMin,
    PointHandleKind::LimitMax,
    PointHandleKind::Length,
    PointHandleKind::AnchorB,
    PointHandleKind::AnchorA,
];

/// A crosshair through the snapped candidate — the only thing on screen that
/// says *the dot stopped here on purpose*. Without it a snap is indistinguishable
/// from a drag that will not track the cursor.
fn paint_snap_cross(scene: &mut VectorScene, s: [f32; 2]) {
    let (cx, cy) = (f64::from(s[0]), f64::from(s[1]));
    let arm = f64::from(SNAP_CROSS_PX);
    let mut path = BezPath::new();
    path.move_to(Point::new(cx - arm, cy));
    path.line_to(Point::new(cx + arm, cy));
    path.move_to(Point::new(cx, cy - arm));
    path.line_to(Point::new(cx, cy + arm));
    scene.inner_mut().stroke(
        &Stroke::new(1.0),
        Affine::IDENTITY,
        anchor_color(),
        None,
        &path,
    );
}

#[cfg(test)]
#[path = "point_tests.rs"]
mod tests;
