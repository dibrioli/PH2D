//! **The collider outline** — what shape is this thing, *physically*? A collider is invisible, so
//! a ball under a square sprite looks like a box until it rolls (*"os colliders parecem redondos
//! mas os desenhos são box"*, Enio 2026-07-18). Like Unity, Godot and Box2D's debug draw, the
//! collider is painted as a wireframe over the art.
//!
//! - **Screen space, deliberately.** Points go through the camera, but the path is stroked under
//!   `Affine::IDENTITY`: in Vello the stroke transform MULTIPLIES the width (the Flip selection-halo
//!   scar, 2026-07-13; `flip_cursor` draws the same way).
//! - **Free when there is no physics.** A scene with no colliders draws nothing; the toggle (`B`)
//!   hides the outlines when they are in the way.

use ph2d_ecs::SimWorld;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::Collider;
use ph2d_render::Camera2d;
use ph2d_tokens::Theme;
use ph2d_vector::{BezPath, Stroke, VectorScene};

/// A GEOMETRIA do contorno mora no irmão (cap de 600 LOC da shell); re-exportada
/// por este caminho porque é por ele que os chamadores sempre a pediram — o corte
/// é meu, não deles.
pub use crate::overlay::shapes::{collider_outline, shape_outline};

use crate::overlay::annotations::{
    BELT_RGBA, EFFECTOR_RGBA, FALLOFF_RGBA, FALLOFF_RING, TORQUE_RGBA, VELOCITY_RGBA, belt_arrow,
    effector_arrow, torque_glyph, velocity_arrow, zone_mirror, zone_pushes,
};
use crate::overlay::contacts::{
    CONTACT_FLASH_RGBA, CONTACT_RGBA, WATERLINE_RGBA, contact_flashes, contact_marks,
    waterline_marks,
};
use crate::overlay::joints::joint_marks;

/// Outline thickness, in screen px. Thinner than the selection halo (2 px):
/// a collider is standing information, not a thing you just did.
pub const OUTLINE_PX: f64 = 1.5; // LITERAL-PX-OK: chrome de overlay, espessura de tela

/// How many segments approximate a circle. 32 is smooth at any zoom a body
/// is readable at, and the path is rebuilt per frame anyway.
pub const CIRCLE_SEGS: u32 = 32;

pub use crate::overlay::tinta::{Marca, Tinta, com_tom, sombra_do_contorno, tinta_do_contorno};

/// **What to draw, decided once** — pure, returned as data (the `hit_plan` shape). `show_velocity`
/// adds the armed launch arrow, true only at the AUTHORED rest (the caller passes `false` while the
/// clock runs).
/// De quem é esta forma: o corpo dela ou o ancestral mais próximo (`governing_kind`); `None` = solta.
use ph2d_physics_ecs::governing_kind as owning_kind;

pub fn outlines(
    show: bool,
    show_velocity: bool,
    sim: &mut SimWorld,
    triggered: &[ph2d_ecs::Entity],
    ghost: bool,
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<(BezPath, Tinta)> {
    if !show {
        return Vec::new();
    }
    // ⚠️ **`&Collider` sem `&RigidBody`** (W-Compound): um corpo composto tem mais
    // de uma forma, e as extras são FILHOS que carregam só o collider. Pedir os
    // dois componentes aqui — como esta query pedia — deixava toda peça de todo (…)
    let mut q = sim.world_mut().query::<(
        ph2d_ecs::Entity,
        &Collider,
        Option<&ph2d_physics_ecs::OneWayPlatform>,
    )>();
    let world = sim.world();
    // ⚠️ WORLD, never the raw `Transform`. The outline annotates a SPRITE, and
    // the sprite is drawn from the composed chain — so an outline placed at the
    // entity's LOCAL pose lands a full parent-offset away from the thing it is (…)
    let mut chain = Vec::new();
    let mut out = Vec::new();
    for (e, col, one_way) in q.iter(world) {
        if ph2d_physics_ecs::forma_desligada(world, e) {
            continue;
        }
        // De QUEM é esta forma? Dela mesma, se ela for um corpo; senão do corpo
        // ancestral mais próximo (W-Compound — o mesmo walk do `rig_edges`, e um
        // GRUPO no meio fica transparente pelo mesmo motivo).
        let kind = owning_kind(world, e);
        let Some(t) = ph2d_ecs::world_transform_into(world, e, &mut chain) else {
            continue;
        };
        // The collider offset, placed exactly where the solver puts it. The
        // bridge folds the body's SIGNED scale into the offset (a flip mirrors it)
        // and rapier rotates the result with the body; the overlay does the same (…)
        let c = ph2d_physics_ecs::forma_resolvida(world, e, col).offset;
        let (ox, oy) = (c[0] * t.scale.x, c[1] * t.scale.y);
        let (sin_r, cos_r) = t.rotation.sin_cos();
        let (wox, woy) = (ox * cos_r - oy * sin_r, ox * sin_r + oy * cos_r);
        // The SAME resolution the bridge does (size, ellipse, Outline rings, signed mirror).
        let path = shape_outline(
            world,
            e,
            col.shape,
            t.scale,
            t.translation.x + wox,
            t.translation.y + woy,
            t.rotation,
            camera,
            window,
        );
        let tinta = crate::overlay::tinta::marca(
            col,
            kind,
            triggered.contains(&e),
            ghost && one_way.is_some(),
        );
        // O Look da forma (GEF02 onda 8): cor, opacidade e traço; sem Look, os do tema e do corpo.
        let look = world.get::<ph2d_physics_ecs::ShapeLook>(e);
        out.push((path, look.map_or(tinta, |l| com_tom(tinta, l.into()))));
        // Sem corpo acima o solver não a vê: nenhuma anotação de corpo.
        if kind.is_none() {
            continue;
        }
        // The armed launch, when the bodies are at rest (so `t` is the authored
        // position the arrow springs from). Absent component = no arrow.
        if show_velocity
            && let Some(iv) = world.get::<ph2d_physics_ecs::InitialVelocity>(e)
            && let Some(arrow) =
                velocity_arrow(t.translation.x, t.translation.y, iv.linvel, camera, window)
        {
            out.push((arrow, Tinta::Fixa(VELOCITY_RGBA)));
        }
        // The force zone's push — WHICH WAY DOES THIS BLOW, and how hard.
        if show
            && let Some(a) = world.get::<ph2d_physics_ecs::AreaEffector>(e)
            && let Some(arrow) = effector_arrow(
                t.translation.x + wox,
                t.translation.y + woy,
                ph2d_physics_ecs::zone_force_world_at(
                    a.force,
                    world
                        .get::<ph2d_physics_ecs::AreaForceWorldAxes>(e)
                        .is_some(),
                    t.rotation,
                    // ⚠️ A lateralidade da MESMA escala que o `body_desc` dobra (W-AreaMirror).
                    zone_mirror(t.scale),
                ),
                camera,
                window,
            )
        {
            out.push((arrow, Tinta::Fixa(EFFECTOR_RGBA)));
        }
        // A ESTEIRA desta superfície — PARA QUE LADO ELA CORRE (W-Surface; ver `belt_arrow`).
        if show
            && let Some(surf) = world.get::<ph2d_physics_ecs::WalkSurface>(e)
            && let Some(arrow) = belt_arrow(
                t.translation.x + wox,
                t.translation.y + woy,
                t.rotation,
                surf.belt,
                camera,
                window,
            )
        {
            out.push((arrow, Tinta::Fixa(BELT_RGBA)));
        }
        // The torque zone's spin — WHICH WAY DOES THIS TURN (the force arrow's sibling).
        if show
            && let Some(a) = world.get::<ph2d_physics_ecs::AreaTorque>(e)
            && let Some(glyph) = torque_glyph(
                t.translation.x + wox,
                t.translation.y + woy,
                // ⚠️ Num ESPELHO o giro inverte (pseudoescalar): a porta do solver.
                a.0 * ph2d_physics_ecs::zone_spin_sign(zone_mirror(t.scale)),
                camera,
                window,
            )
        {
            out.push((glyph, Tinta::Fixa(TORQUE_RGBA)));
        }
        // E o ANEL DO FALLOFF — a curva de nível de meio caminho, quando o empurrão desta
        // zona desvanece para a borda. Sem ele o falloff é o único número do modelo de
        // área que não deixa marca nenhuma na tela: a seta continua do mesmo tamanho (ela (…)
        if show
            && let Some(f) = world.get::<ph2d_physics_ecs::AreaFalloff>(e)
            && f.0 > 0.0
            && zone_pushes(world, e)
        {
            out.push((
                shape_outline(
                    world,
                    e,
                    col.shape,
                    ph2d_core::Vec2::new(t.scale.x * FALLOFF_RING, t.scale.y * FALLOFF_RING),
                    t.translation.x + wox,
                    t.translation.y + woy,
                    t.rotation,
                    camera,
                    window,
                ),
                Tinta::Fixa(FALLOFF_RGBA),
            ));
        }
    }
    out
}

/// Paint them. No-op when [`outlines`] returns nothing.
#[allow(clippy::too_many_arguments)]
pub fn draw(
    show: bool,
    show_velocity: bool,
    sim: &mut SimWorld,
    joint_views: &[ph2d_physics_ecs::JointView],
    // A arena de roldanas que as views indexam (W-Pulley W1).
    joint_wheels: &[ph2d_physics_ecs::rope_route::RopeWheel],
    // O ângulo de cada roldana, paralelo à arena (W-Pulley W1).
    joint_spins: &[f32],
    joint_gravity: [f32; 2],
    // O limite sendo POSADO `(joint, rad relativo)`: o fantasma de B (W-J3).
    posed_limit: Option<(ph2d_ecs::Entity, f32)>,
    // W-J4: a banda do gesto de criar `(de, para)`; gesto, desenhada mesmo com `show` falso.
    join_band: Option<([f32; 2], [f32; 2])>,
    // W-Grab: a mola da MÃO `(cursor, ponto de pega)`; gesto, como a banda.
    grab: Option<([f32; 2], [f32; 2])>,
    // W-Hand: a MIRA `(centro, alcance)`; `None` para a mão e com o gesto inerte.
    aim: Option<([f32; 2], f32)>,
    // W-Hand: o campo de atração VIVO `(centro, alcance)`, lido da ponte.
    pull: Option<([f32; 2], f32)>,
    // W-Hand: o último estouro `(centro, alcance)`, enquanto o flash dura.
    blast: Option<([f32; 2], f32)>,
    contacts: &[ph2d_physics_ecs::BodyContact],
    flashes: &[ph2d_physics_ecs::ContactFlash],
    waterlines: &[([f32; 2], [f32; 2])],
    // W-Probes: onde os sensores do player olharam, e o que responderam.
    probes: &[ph2d_physics_ecs::ProbeMark],
    triggered: &[ph2d_ecs::Entity],
    // W20: algum player atravessa uma prancha AGORA (veste todas as one-way, `PASSABLE_RGBA`).
    ghost: bool,
    // W-J7b: o selecionado, para o readout de carga de um joint ainda não quebrável.
    selected_joint: Option<ph2d_ecs::Entity>,
    camera: &Camera2d,
    window: WindowSize,
    theme: Theme,
    vector_scene: &mut VectorScene,
    text_system: &mut ph2d_text::TextSystem,
) {
    use ph2d_vector::{Affine, Brush, Color};
    let contornos = outlines(show, show_velocity, sim, triggered, ghost, camera, window);
    // As sombras TODAS antes das cores: a de uma forma vizinha nunca tapa a cor de outra.
    let sombras = contornos
        .iter()
        .filter_map(|(p, t)| Some((p, sombra_do_contorno(*t, theme)?)));
    let cores = contornos
        .iter()
        .map(|(p, t)| (p, tinta_do_contorno(*t, theme)));
    for (path, (traco, cor)) in sombras.chain(cores) {
        vector_scene
            .inner_mut()
            .stroke(&traco, Affine::IDENTITY, &Brush::Solid(cor), None, path);
    }
    // A linha d'água ANTES dos corpos: é o cenário; por cima lê-se o que boia.
    for path in waterline_marks(show, waterlines, camera, window) {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(WATERLINE_RGBA)),
            None,
            &path,
        );
    }
    // ⚠️ Os SENSORES depois dos corpos e ANTES dos contatos (W-Probes).
    // O raio do objecto veste a cor da camada com a sombra por baixo, como o contorno (emenda 4).
    let sondas = crate::overlay::probes::probe_marks(show, probes, sim, camera, window);
    let sombras = sondas
        .iter()
        .filter_map(|(p, t)| Some((p, sombra_do_contorno(*t, theme)?)));
    let cores = sondas
        .iter()
        .map(|(p, t)| (p, tinta_do_contorno(*t, theme)));
    for (path, (traco, cor)) in sombras.chain(cores) {
        vector_scene
            .inner_mut()
            .stroke(&traco, Affine::IDENTITY, &Brush::Solid(cor), None, path);
    }
    // Contacts ON TOP: the smallest mark, sitting exactly ON the two outlines that meet.
    for path in contact_marks(show, contacts, camera, window) {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(CONTACT_RGBA)),
            None,
            &path,
        );
    }
    // The BEGIN-flash on top of the standing marks, never buried by its own cross.
    for path in contact_flashes(show, flashes, camera, window) {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(CONTACT_FLASH_RGBA)),
            None,
            &path,
        );
    }
    // Joints ON TOP of the colliders: the link runs between two bodies and
    // would otherwise be hidden by whichever outline was drawn last.
    // (…)
    if let Some(band) = crate::overlay::gesture::draw_band(join_band, camera, window) {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(crate::overlay::joints::JOINT_RGBA)),
            None,
            &band,
        );
    }
    // E a MÃO, como a banda (gesto, sem o gate de `show`; as duas nunca coexistem).
    if let Some(path) = crate::overlay::gesture::draw_grab(grab, camera, window) {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(crate::overlay::gesture::GRAB_RGBA)),
            None,
            &path,
        );
    }
    // ── As ferramentas de PONTO (W-Hand) ──────────────────────────────────
    // Sem o gate de `show`: são gesto, como a banda e a mão.
    crate::overlay::annotations::draw_interaction(aim, pull, blast, camera, window, vector_scene);
    // O FANTASMA primeiro: é o fundo do arco arrastado, por baixo da agulha viva.
    if show
        && let Some(ghost) =
            crate::overlay::joint_ghost::limit_ghost(sim, joint_views, posed_limit, camera, window)
    {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(crate::overlay::joint_ghost::JOINT_GHOST_RGBA)),
            None,
            &ghost,
        );
    }
    // ⚠️ A lista sai do DESENHO e é lida pelo READOUT (W-RopeSays): uma pergunta, não duas.
    let marks = joint_marks(
        show,
        joint_views,
        joint_wheels,
        joint_spins,
        joint_gravity,
        camera,
        window,
    );
    for (path, rgba) in marks.paths {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(rgba)),
            None,
            &path,
        );
    }
    // ⚠️ Os NÚMEROS por ÚLTIMO: a cena tem de estar livre para o renderizador de texto.
    for r in crate::overlay::joint_readout::joint_readouts(
        show,
        joint_views,
        selected_joint,
        camera,
        window,
        &marks.not_acting,
    ) {
        // Centrado numa caixa larga o bastante para o rótulo mais longo
        // (`1234 / 1234 N.m`) e alta o bastante para uma linha.
        let rect = ph2d_editor_core::zones::Rect::new(
            r.at.x as f32 - READOUT_BOX_W_PX * 0.5,
            r.at.y as f32 - READOUT_BOX_H_PX * 0.5,
            READOUT_BOX_W_PX,
            READOUT_BOX_H_PX,
        );
        ph2d_editor_core::paint::paint_text_centered(
            text_system,
            vector_scene,
            &r.text,
            rect,
            crate::overlay::joint_readout::READOUT_PX,
            Color::new(r.rgba),
        );
    }
}

/// Caixa do rótulo de readout, px de tela — larga o bastante para o par mais
/// longo que ele pode escrever, e de uma linha de altura.
const READOUT_BOX_W_PX: f32 = 110.0; // LITERAL-PX-OK: chrome de overlay
const READOUT_BOX_H_PX: f32 = 14.0; // LITERAL-PX-OK: chrome de overlay

// ⚠️ `pub(crate)` e não privado: as três funções de fixture (a câmera, a janela
// e a leitura de pontos) têm agora um QUARTO consumidor — os gates do desenho
// dos sensores (`W-Probes`), que moram num módulo IRMÃO. Duas câmeras fariam as (…)
#[cfg(test)]
#[path = "tests.rs"]
pub mod tests;

#[cfg(test)]
#[path = "scene_tests.rs"]
mod scene_tests;

/// As ANOTAÇÕES sobre o contorno (setas + glifos) — o terceiro arquivo de gates do
/// overlay, separado dos irmãos pelo cap de 600 LOC (W-AreaFrame).
#[cfg(test)]
#[path = "annotation_tests.rs"]
mod annotation_tests;

#[cfg(test)]
#[path = "passable_tests.rs"]
mod passable_tests;

#[cfg(test)]
#[path = "loose_tests.rs"]
mod loose_tests;

#[cfg(test)]
#[path = "body_kind_tests.rs"]
mod body_kind_tests;
