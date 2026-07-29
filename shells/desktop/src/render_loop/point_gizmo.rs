//! **Which anchors get a canvas handle** — the joint-anchor gizmo's publish rule.
//!
//! Extracted from `snapshots::publish` so the rule is gated HEADLESS: the
//! publish itself needs a camera, a present world and a live `HeroScreen`, none
//! of which a unit test has, but the decision is a pure function of the sim
//! world, the bridge and the clock.
//!
//! The point gizmo is `ph2d_editor::PointGizmoView` (editor-core): grabbable
//! dots at world anchors, for entities that are POINTS rather than boxes. A
//! physics joint is exactly that — it carries a `Transform` but no drawable
//! geometry, so `snapshots::build_view` returns `None` for it and it would
//! otherwise get no canvas handle at all.
//!
//! # Every joint, not the selected one (W-J2b)
//!
//! W-JointAnchor and W-J2 offered the handles only for the SELECTED joint, and
//! Enio's smoke named the cost (2026-07-25: *"precisam ser selecionados e
//! arrastável diretamente no canvas sem necessitar selecionar no hierarchy"*).
//! The two halves of that are one fact: a joint has **no sprite**, so a canvas
//! click can never reach it through `pick_sprites_at_world`, and a selection is
//! the only thing that made its dots appear — which means the dots were reachable
//! only by first finding the joint in the Hierarchy. A handle you must find
//! somewhere else before you can grab it is not on the canvas.
//!
//! So every joint publishes its handles, and grabbing one SELECTS its joint (the
//! shell's Down does that, so §12 opens on the joint you just grabbed).

use ph2d_ecs::{Entity, SimWorld};
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{JointSide, PhysicsBridge};
use ph2d_render::Camera2d;

use ph2d_editor::gizmo::{PointGizmoView, PointHandle, PointHandleKind};

use super::physics_overlay_joint_glyphs::{length_handle_world, limit_end_screen};

/// Every anchor that should be grabbable this frame, sorted by `(entity, side)`.
///
/// ⚠️ **Rest-only, and this closes a gap rather than opening one:**
/// `sync_joint_pivots` already declares in its own doc that "during play the dot
/// is not shown and the overlay draws the live solver anchors" — and that claim
/// was false, because nothing asked. During play the anchors the overlay draws
/// are the solver's, which is what the artist should be reading; the authored
/// handles are a rest-time thing, and a handle that took a drag against a
/// swinging body would author against a pose nobody chose.
///
/// Locked entities are skipped: `joint_anchor_drag::open_drag` refuses them, and
/// a handle that paints, registers a hit and then declines the gesture is worse
/// than one that is not offered.
///
/// The A end of a **dormant** joint is included — the bridge's door answers it
/// from the authored `Transform` — because that is precisely the joint the
/// artist is in the middle of fixing. Its B end has no body, so no anchor, so no
/// handle.
#[must_use]
pub(super) fn joint_anchor_handles(
    sim: &SimWorld,
    physics: &PhysicsBridge,
    at_rest: bool,
) -> Vec<PointHandle> {
    if !at_rest {
        return Vec::new();
    }
    let mut out: Vec<PointHandle> = Vec::new();
    for &e in physics.joint_entities() {
        if ph2d_ecs::is_locked_for_edit(sim.world(), e) {
            continue;
        }
        for (side, kind) in [
            (JointSide::A, PointHandleKind::AnchorA),
            (JointSide::B, PointHandleKind::AnchorB),
        ] {
            if let Some(world) = physics.joint_anchor_world(sim, e, side) {
                out.push(PointHandle {
                    key: e.to_bits(),
                    kind,
                    world,
                });
            }
        }
    }
    // Deterministic display order. The hit ids are per-handle, so ordering
    // cannot decide WHO owns a pixel — it decides only which of two overlapping
    // joints paints on top, and that should not depend on archetype layout.
    out.sort_by_key(|h| (h.key, h.kind));
    out
}

/// The **parameter** grips (W-J3): the two walls of a hinge's limit arc, and the
/// length ring of a spring / rope.
///
/// # Why these two and not the motor's speed
///
/// A limit is an **angle** and a rest length is a **distance**: each already has
/// a place on the canvas, so dragging the wall to 30° or the ring to 2 m needs
/// no scale to convert anything — the position IS the value, and the numeric row
/// and the grip cannot drift because there is nothing between them.
///
/// ⚠️ Motor speed is a **rate**, and no place on the canvas is 120 °/s. Every
/// handle for it needs a px-per-°/s constant, and the §12 row it would mirror is
/// a free-form `num_row` with **no range** to derive one from — so the constant
/// would be invented, which is exactly the kind of number this project refuses
/// to write (§0: measure before you write a limit). The two alternatives that
/// need no constant both fail on their own terms: the arc's SWEEP saturates (the
/// glyph is 270°, so every speed past the top reads the same), and an angular
/// position that maps one revolution to 360 °/s **wraps** — 400 °/s and 40 °/s
/// would draw the identical dot. Deferred with the reason, not dropped: what
/// unblocks it is a decision about the control law (or a range on the row), and
/// that is Enio's to make.
///
/// The grips are offered only while the joint overlay is SHOWN, because they are
/// grips on ITS geometry: with the arc and the ring not drawn, a dot on them is
/// a control over an invisible line.
#[must_use]
pub(super) fn joint_param_handles(
    physics: &PhysicsBridge,
    camera: &Camera2d,
    window: WindowSize,
    show_overlay: bool,
    at_rest: bool,
) -> Vec<PointHandle> {
    if !show_overlay || !at_rest {
        return Vec::new();
    }
    let mut out = Vec::new();
    for v in physics.joint_views() {
        let key = v.entity.to_bits();
        // The walls, through the door that DRAWS them: the grip and the tick are
        // one place (see `limit_end_screen`).
        if let Some(l) = v.limits {
            for (i, kind) in [PointHandleKind::LimitMin, PointHandleKind::LimitMax]
                .into_iter()
                .enumerate()
            {
                // ⚠️ **Two geometries, because there are two kinds of range.** A
                // hinge's ends live on the ARC; a rail's live ON THE RAIL, where
                // `slider_rail` already draws its end-of-travel ticks. The
                // question is `limits_in_metres` — the SAME door the §12 label
                // and the shell's conversion ask, and the one this publisher did
                // not ask until 2026-07-26: it put a rail's 0.5 *metre* stroke on
                // the arc at 0.5 *radians*, and the drag then authored a bearing
                // into a field read as metres (the runaway Enio reported).
                let world = if v.kind.limits_in_metres() {
                    let axis = match v.axis {
                        Some(a) => a,
                        // A rail with no resolved axis draws no rail, so there is
                        // nothing for a grip to sit on.
                        None => continue,
                    };
                    [
                        v.anchor_a[0] + axis[0] * l[i],
                        v.anchor_a[1] + axis[1] * l[i],
                    ]
                } else {
                    let p = limit_end_screen(camera, window, v.anchor_a, v.angle_a, l[i]);
                    // ⚠️ Unprojected. The arc lives at a fixed SCREEN radius (an
                    // angle is not a length — the module's own law), while a
                    // `PointHandle` is a world point so that one projection
                    // serves every handle. The round trip is a linear map, so it
                    // re-projects to the pixel it came from.
                    camera.screen_to_world((p.x as f32, p.y as f32), window)
                };
                out.push(PointHandle { key, kind, world });
            }
        }
        // ⚠️ **`length_is_a_radius`, não `v.length.is_some()`** — e a diferença
        // nasceu com a polia: ela TEM comprimento (a corda) e ele **não** é a
        // distância entre as âncoras, então um anel de raio `L0` em volta de A
        // descreveria uma distância que não existe na cena, com um grip que
        // autoraria a corda arrastando um círculo que não a mede. É a mesma
        // classe do bug do trilho de 2026-07-26, uma porta adiante.
        if let Some(len) = v.length.filter(|_| v.kind.length_is_a_radius()) {
            out.push(PointHandle {
                key,
                kind: PointHandleKind::Length,
                world: length_handle_world(v.anchor_a, v.anchor_b, len),
            });
        }
        // ⚠️ **As roldanas NÃO são alças do joint** (W-Pulley W1): cada uma é
        // uma entidade, e as alças dela — centro e aro — são publicadas quando
        // ELA está selecionada, por `wheel_handles`. Eram alças da corda enquanto
        // eram dois campos dela.
    }
    out.sort_by_key(|h| (h.key, h.kind));
    out
}

/// The point gizmo for this frame, or `None` when there is nothing to grab.
///
/// This function decides *whether* there are handles and where the camera is;
/// it never decides *where* an anchor is. That answer comes from the bridge's
/// anchor door (see [`joint_anchor_handles`]), and re-deriving it here would be
/// the second opinion that drifts — the failure W-AnchorFollow paid 1.771 m for.
#[must_use]
pub(super) fn build_point_view(
    handles: Vec<PointHandle>,
    camera: &Camera2d,
    window_size: WindowSize,
    snap: Option<[f32; 2]>,
    // W-J4b: o gesto de DESENHAR um joint está armado ⇒ as alças existentes ficam
    // visíveis e fora de alcance (ver `PointGizmoView::inert`). O mesmo booleano
    // que pinta o botão Pressed, lido aqui — duas cópias da pergunta *"o gesto
    // está armado?"* divergiriam, e a divergência seria uma alça apagada que
    // ainda pega o clique.
    inert: bool,
) -> Option<PointGizmoView> {
    if handles.is_empty() {
        return None;
    }
    Some(PointGizmoView {
        handles,
        snap_world: snap,
        camera_center: camera.center,
        camera_height_world: camera.height_world,
        window_w: window_size.width as f32,
        window_h: window_size.height as f32,
        canvas: ph2d_editor::zones::Rect::new(
            0.0,
            0.0,
            window_size.width as f32,
            window_size.height as f32,
        ),
        inert,
    })
}

/// **As duas alças da roldana SELECIONADA** — o centro e o aro.
///
/// O pedido (6) do artista: *"um ponto central para deslocamento e um ponto no
/// raio externo para definir o tamanho"*. Uma roldana não tem sprite, então o
/// gizmo de caixa não a alcança — é o mesmo vão que o dot da âncora de joint
/// cobre, e por isso o primitivo é o mesmo.
///
/// ⚠️ **Só a roldana SELECIONADA**, ao contrário das alças de joint, que todo
/// joint publica: uma corda com seis roldanas publicaria doze alças, e o aro de
/// uma cairia sobre o centro da vizinha. Selecionar é o que diz *qual*.
///
/// A geometria vem do estado AUTORADO (o `Transform` e o componente), não da
/// arena do solver, e aqui isso é o certo: estas alças autoram aqueles dois
/// números, e ler a cópia do solver poria um frame de atraso entre o dedo e o
/// desenho.
#[must_use]
pub(super) fn wheel_handles(
    sim: &SimWorld,
    selection: Option<u64>,
    show_overlay: bool,
    at_rest: bool,
) -> Vec<PointHandle> {
    let mut out = Vec::new();
    if !show_overlay || !at_rest {
        return out;
    }
    if let Some(e) = selection.map(Entity::from_bits)
        && let (Some(w), Some(t)) = (
            sim.world().get::<ph2d_physics_ecs::PulleyWheel>(e),
            sim.world().get::<ph2d_ecs::Transform>(e),
        )
    {
        let centre = [t.translation.x, t.translation.y];
        out.push(PointHandle {
            key: e.to_bits(),
            kind: PointHandleKind::WheelCentre,
            world: centre,
        });
        // O aro à DIREITA do centro: uma direção fixa, para a alça não saltar
        // quando o raio passa por zero (o ângulo de um raio nulo é indefinido).
        let w = w.clamped();
        out.push(PointHandle {
            key: e.to_bits(),
            kind: PointHandleKind::WheelRim,
            world: [centre[0] + w.radius, centre[1]],
        });
        // **O segundo diâmetro** (W6) — e ele é oferecido SÓ quando existe.
        //
        // ⚠️ **Numa roldana comum ele cairia exatamente sobre o de entrada**, e
        // duas alças no mesmo pixel são uma alça que às vezes faz outra coisa:
        // qual das duas o hit-test devolve vira acidente de ordem de registro. É
        // a mesma lei que fez o W4 desenhar o segundo anel só quando ele difere.
        //
        // À ESQUERDA do centro, e não à direita: o de entrada já ocupa aquele
        // raio, e um tambor cujos dois raios são próximos porá as duas alças a
        // poucos pixels uma da outra se as duas saírem do mesmo lado.
        if w.radius_out > 0.0 {
            out.push(PointHandle {
                key: e.to_bits(),
                kind: PointHandleKind::WheelRimOut,
                world: [centre[0] - w.radius_out, centre[1]],
            });
        }
    }
    out
}

/// The joint a hit id belongs to, and which end — read back from the map the
/// painter filled while registering.
///
/// The ids are keyed by entity bits (`ph2d_editor::gizmo::point_handle_id`), so
/// no static table can classify them; this is the same shape as the box gizmo's
/// `gizmo_hit_map` and for the same reason.
#[must_use]
pub(crate) fn resolve_anchor_hit(
    hit_map: &std::collections::BTreeMap<ph2d_editor::NodeId, PointHandle>,
    id: ph2d_editor::NodeId,
) -> Option<(Entity, PointHandleKind)> {
    let h = hit_map.get(&id)?;
    Some((Entity::from_bits(h.key), h.kind))
}

/// The [`JointSide`] a handle kind authors, or `None` for the parameter grips
/// (they write the joint COMPONENT, not an anchor).
#[must_use]
pub(crate) fn anchor_side(kind: PointHandleKind) -> Option<JointSide> {
    match kind {
        PointHandleKind::AnchorA => Some(JointSide::A),
        PointHandleKind::AnchorB => Some(JointSide::B),
        _ => None,
    }
}

#[cfg(test)]
#[path = "point_gizmo_tests.rs"]
mod tests;
