//! **The collider outline** — what shape is this thing, *physically*?
//!
//! A sprite is a textured QUAD. A collider is a circle, or a box, or (later)
//! a capsule, and it is **invisible**. So a ball collider under a square
//! sprite looks exactly like a box collider under a square sprite, right up
//! until it rolls — which is precisely the report that produced this module
//! (*"os colliders parecem redondos mas os desenhos são box"*, Enio
//! 2026-07-18).
//!
//! That mismatch is not a bug in the demo scene: it is the **normal case**.
//! In a real project the art is whatever the artist drew and the collider is
//! a shape they chose, and the two are only related by intent. Every physics
//! editor answers this the same way — Unity, Godot and Box2D's own debug draw
//! all paint the collider as a wireframe on top of the art — so that is what
//! this does. Making the *sprite* round would only fix the demo.
//!
//! ## Screen space, deliberately
//!
//! The geometry is built in world units and every POINT is pushed through the
//! camera, but the resulting path is in screen pixels and is stroked under
//! `Affine::IDENTITY`. In Vello the stroke transform **multiplies the
//! width**, so handing the world→screen affine to `stroke` turns a 1.5 px
//! outline into `1.5 × pixels-per-world-unit` — hundreds of pixels of paint.
//! That is a scar, not a hypothesis: it is what happened to the Flip
//! selection halo (smoke, 2026-07-13), and `flip_cursor` has always drawn
//! this way for the same reason.
//!
//! ## Free when there is no physics
//!
//! Nothing is drawn for a scene with no bodies, so a painter or vector user
//! never sees physics chrome and never pays for it. The toggle (`B`) exists
//! for the case where the outlines are in the way; W2 moves it into the
//! physics panel, reading this same flag.

use ph2d_ecs::SimWorld;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{BodyKind, Collider, scaled_shape};
use ph2d_render::Camera2d;
use ph2d_vector::{BezPath, VectorScene};

/// A GEOMETRIA do contorno mora no irmão (cap de 600 LOC da shell); re-exportada
/// por este caminho porque é por ele que os chamadores sempre a pediram — o corte
/// é meu, não deles.
pub(crate) use super::physics_overlay_shapes::collider_outline;

use super::physics_overlay_annotations::{
    EFFECTOR_RGBA, FALLOFF_RGBA, FALLOFF_RING, TORQUE_RGBA, VELOCITY_RGBA, effector_arrow,
    torque_glyph, velocity_arrow, zone_mirror, zone_pushes,
};
use super::physics_overlay_contacts::{
    CONTACT_FLASH_RGBA, CONTACT_RGBA, WATERLINE_RGBA, contact_flashes, contact_marks,
    waterline_marks,
};
use super::physics_overlay_joints::joint_marks;

/// Outline thickness, in screen px. Thinner than the selection halo (2 px):
/// a collider is standing information, not a thing you just did.
pub(super) const OUTLINE_PX: f64 = 1.5; // LITERAL-PX-OK: chrome de overlay, espessura de tela

/// How many segments approximate a circle. 32 is smooth at any zoom a body
/// is readable at, and the path is rebuilt per frame anyway.
pub(super) const CIRCLE_SEGS: u32 = 32;

/// Static bodies — the scenery. Cool green, the Unity/Box2D convention.
const STATIC_RGBA: [f32; 4] = [0.36, 0.85, 0.52, 0.85]; // LITERAL-COLOR-OK: overlay de collider

/// Dynamic bodies — the things that move. Cyan, chosen to stay clear of the
/// amber selection halo: a selected dynamic body must still read as selected.
const DYNAMIC_RGBA: [f32; 4] = [0.35, 0.80, 1.0, 0.85]; // LITERAL-COLOR-OK: overlay de collider

/// Kinematic bodies — moved by the scene, not by the solver (a baked body, an
/// animated platform). Violet: it must not read as either neighbour, because
/// the whole question the overlay answers here is *who is driving this*. Box2D
/// and Unity both give the kind its own colour for the same reason.
const KINEMATIC_RGBA: [f32; 4] = [0.72, 0.55, 1.0, 0.85]; // LITERAL-COLOR-OK: overlay de collider

/// A **sensor** (trigger) with nothing inside it — magenta, a hue no body kind
/// or the amber joint uses, so a trigger zone never reads as a solid collider.
/// Dim, because an empty trigger is standing information, not an event.
const SENSOR_IDLE_RGBA: [f32; 4] = [0.95, 0.45, 0.90, 0.55]; // LITERAL-COLOR-OK: overlay de collider

/// A sensor with a body inside it — the SAME magenta, bright and opaque. The
/// jump from idle to this is the whole point of a trigger: you see it fire.
const SENSOR_ACTIVE_RGBA: [f32; 4] = [1.0, 0.55, 0.98, 1.0]; // LITERAL-COLOR-OK: overlay de collider

/// **Uma plataforma jump-through que o player está a ATRAVESSAR agora** — a
/// mesma família do par idle/active do sensor: o mesmo assunto, outra
/// intensidade. Apagada, porque o que ela diz é *"eu não estou aqui para ele"*.
///
/// ⚠️ **Sem isto a descida é INVISÍVEL**, e é essa a razão de existir: toda a
/// classe de defeitos desta feature é *a prancha ficou fantasma e ninguém viu*
/// (ver `bridge::player::retire_drops`). Um estado que muda a colisão da cena
/// inteira e não se vê é um estado que o artista descobre por acidente.
const PASSABLE_RGBA: [f32; 4] = [0.55, 0.62, 0.58, 0.40]; // LITERAL-COLOR-OK: overlay de collider

/// The colour of one collider outline: a sensor is magenta (bright when a body
/// is inside it), and any solid collider is coloured by its body kind. A sensor
/// overrides the kind colour on purpose — "is this a trigger?" is the first
/// thing you need to know about it, ahead of who moves it.
fn outline_rgba(is_sensor: bool, triggered: bool, passable: bool, kind: BodyKind) -> [f32; 4] {
    if passable {
        // Antes do sensor de propósito: *"isto não é sólido para ele agora"* é o
        // fato mais forte que se pode dizer de uma forma, e uma plataforma
        // one-way nunca é sensor (as duas metades são mutuamente exclusivas —
        // W-Area).
        PASSABLE_RGBA
    } else if is_sensor {
        if triggered {
            SENSOR_ACTIVE_RGBA
        } else {
            SENSOR_IDLE_RGBA
        }
    } else {
        match kind {
            BodyKind::Static => STATIC_RGBA,
            BodyKind::Dynamic => DYNAMIC_RGBA,
            BodyKind::Kinematic => KINEMATIC_RGBA,
        }
    }
}

/// **What to draw, decided once.** Pure: the toggle and the "is there any
/// physics here at all" question are answered here and returned as data, not
/// resolved inside a paint loop. That is the repo's `hit_plan` shape — a
/// refusal that lives in a loop cannot be tested, and an overlay that quietly
/// draws when it was switched off is exactly the kind of thing nobody notices
/// until it is in a screenshot.
/// `show_velocity` adds the initial-velocity arrow to each body that has a
/// launch armed. It is separate from `show` because the arrow is only truthful
/// while the bodies are at their AUTHORED rest (before the sim steps): once a
/// body has moved, its live velocity is no longer the authored one, so the
/// caller passes `false` while the clock is running.
/// **De quem é esta forma** — o `BodyKind` que a colore.
///
/// Dela mesma quando a entidade É um corpo; senão do corpo ancestral mais
/// próximo, que é o que faz de uma PEÇA (W-Compound) uma forma a mais do dono e
/// não um objeto solto. `None` quando não há corpo nenhum acima: aquele collider
/// não é simulado, e desenhá-lo afirmaria que é.
///
/// ⚠️ **DELEGA** (W-PartFace): o walk vive em `ph2d_physics_ecs::governing_kind`,
/// ao lado da regra que a PONTE usa para pendurar a peça. Ele já esteve escrito
/// aqui, no Inspector e na ponte — três cópias da mesma frase, e um contorno que
/// colorisse por um dono diferente daquele em que o solver pendurou a forma seria
/// o drift mais difícil de ver, porque os dois lados parecem certos.
use ph2d_physics_ecs::governing_kind as owning_kind;

pub(crate) fn outlines(
    show: bool,
    show_velocity: bool,
    sim: &mut SimWorld,
    triggered: &[ph2d_ecs::Entity],
    ghost: bool,
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<(BezPath, [f32; 4])> {
    if !show {
        return Vec::new();
    }
    // ⚠️ **`&Collider` sem `&RigidBody`** (W-Compound): um corpo composto tem mais
    // de uma forma, e as extras são FILHOS que carregam só o collider. Pedir os
    // dois componentes aqui — como esta query pedia — deixava toda peça de todo
    // corpo composto **invisível**, que é exatamente o que o contorno existe para
    // não deixar acontecer (um collider é invisível e um sprite é um quad).
    let mut q = sim.world_mut().query::<(
        ph2d_ecs::Entity,
        &Collider,
        Option<&ph2d_physics_ecs::OneWayPlatform>,
    )>();
    let world = sim.world();
    // ⚠️ WORLD, never the raw `Transform`. The outline annotates a SPRITE, and
    // the sprite is drawn from the composed chain — so an outline placed at the
    // entity's LOCAL pose lands a full parent-offset away from the thing it is
    // supposed to be describing. Under a rig at x = -3 every collider drew at
    // x = 0 instead, so all of them piled up in the middle of the scene, far
    // from their art. That shipped, and it is what Enio saw
    // (`docs/Physics/BUGS_physics.md` #2 — the overlay was a SIXTH reader of
    // "where is this body", missed when the bridge's five were converted).
    //
    // One scratch buffer for the whole pass, so the chain walk allocates once
    // rather than once per body.
    let mut chain = Vec::new();
    let mut out = Vec::new();
    for (e, col, one_way) in q.iter(world) {
        // De QUEM é esta forma? Dela mesma, se ela for um corpo; senão do corpo
        // ancestral mais próximo (W-Compound — o mesmo walk do `rig_edges`, e um
        // GRUPO no meio fica transparente pelo mesmo motivo).
        //
        // ⚠️ **Um collider sem corpo nenhum acima NÃO é desenhado**, e isso é o
        // contorno sendo honesto: ele não é simulado, e uma marca sobre ele diria
        // que é.
        let Some(kind) = owning_kind(world, e) else {
            continue;
        };
        let Some(t) = ph2d_ecs::world_transform_into(world, e, &mut chain) else {
            continue;
        };
        // The collider offset, placed exactly where the solver puts it. The
        // bridge folds the body's SIGNED scale into the offset (a flip mirrors it)
        // and rapier rotates the result with the body; the overlay does the same
        // so the wireframe sits on the collider, not the sprite centre. Reading
        // `col.offset` any other way here is how the outline and the solver would
        // come to disagree about WHERE the collider is.
        let (ox, oy) = (col.offset[0] * t.scale.x, col.offset[1] * t.scale.y);
        let (sin_r, cos_r) = t.rotation.sin_cos();
        let (wox, woy) = (ox * cos_r - oy * sin_r, ox * sin_r + oy * cos_r);
        // The SAME resolution the bridge does — so the outline is drawn at the
        // size (and shape: circle vs ellipse) the solver actually simulates
        // under this body's world scale.
        let path = collider_outline(
            scaled_shape(col.shape, t.scale),
            t.translation.x + wox,
            t.translation.y + woy,
            t.rotation,
            camera,
            window,
        );
        out.push((
            path,
            outline_rgba(
                col.is_sensor,
                triggered.contains(&e),
                ghost && one_way.is_some(),
                kind,
            ),
        ));
        // The armed launch, when the bodies are at rest (so `t` is the authored
        // position the arrow springs from). Absent component = no arrow.
        if show_velocity
            && let Some(iv) = world.get::<ph2d_physics_ecs::InitialVelocity>(e)
            && let Some(arrow) =
                velocity_arrow(t.translation.x, t.translation.y, iv.linvel, camera, window)
        {
            out.push((arrow, VELOCITY_RGBA));
        }
        // The force zone's push — WHICH WAY DOES THIS BLOW, and how hard. A zone is a
        // sensor, so it is already magenta; the arrow is what makes it a *directed*
        // area rather than a region that merely notices things.
        //
        // Unlike the launch above it is drawn whether or not the clock is running: a
        // force is a property of the AREA, authored once, and it does not stop being
        // true because the simulation started (the launch arrow is hidden while
        // playing precisely because it stops being true the moment the body moves).
        //
        // ⚠️ The force is authored in the ZONE's frame, so the arrow is drawn through
        // `zone_force_world_at` — the SAME door the solver's substep asks (re-exported
        // for exactly this). Rotating `a.force` here with a second copy of the rule
        // would draw a wind that does not blow: the arrow is the only place a person
        // ever reads this direction, and a screenshot is not something a gate reads.
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
                    // ⚠️ A lateralidade do frame vem da MESMA escala sincada que o `body_desc`
                    // dobra para o solver (W-AreaMirror). Uma seta desenhada de um espelho
                    // que o solver não usa aponta para onde o vento não sopra, e é o único
                    // lugar onde uma pessoa lê essa direção.
                    zone_mirror(t.scale),
                ),
                camera,
                window,
            )
        {
            out.push((arrow, EFFECTOR_RGBA));
        }
        // The torque zone's spin — WHICH WAY DOES THIS TURN. The rotational sibling of the
        // force arrow, drawn for the same reason and under the same rule (authored, so it
        // is true whether or not the clock is running). A pure whirlpool carries no force
        // arrow, so without this glyph a spin zone would be an invisible property.
        if show
            && let Some(a) = world.get::<ph2d_physics_ecs::AreaTorque>(e)
            && let Some(glyph) = torque_glyph(
                t.translation.x + wox,
                t.translation.y + woy,
                // ⚠️ Um giro visto num ESPELHO gira ao contrário — um torque 2D é um
                // pseudoescalar. Pela mesma porta que o solver (`zone_spin_sign`), senão o
                // glifo violeta apontaria a mão errada e ninguém confere isso numa foto.
                a.0 * ph2d_physics_ecs::zone_spin_sign(zone_mirror(t.scale)),
                camera,
                window,
            )
        {
            out.push((glyph, TORQUE_RGBA));
        }
        // E o ANEL DO FALLOFF — a curva de nível de meio caminho, quando o empurrão desta
        // zona desvanece para a borda. Sem ele o falloff é o único número do modelo de
        // área que não deixa marca nenhuma na tela: a seta continua do mesmo tamanho (ela
        // desenha a força AUTORADA, que é a do centro) e a diferença só aparece nos corpos
        // se moverem de formas diferentes.
        //
        // ⚠️ Desenhado só quando há o que atenuar — um falloff sobre uma zona que não
        // empurra nem gira descreveria o desvanecimento de nada. Mesma regra das duas
        // anotações acima: cada uma aparece exatamente quando a sua grandeza existe.
        //
        // ⚠️ A metade sai pela MESMA `scaled_shape` do contorno, com a escala do corpo
        // reduzida à metade — não por uma segunda função que encolhe formas. Halvar as
        // duas componentes preserva a igualdade `|sx| == |sy|` que decide círculo-ou-elipse
        // lá dentro, então o fantasma é da mesma FAMÍLIA que o contorno, sempre.
        if show
            && let Some(f) = world.get::<ph2d_physics_ecs::AreaFalloff>(e)
            && f.0 > 0.0
            && zone_pushes(world, e)
        {
            out.push((
                collider_outline(
                    scaled_shape(
                        col.shape,
                        ph2d_core::Vec2::new(t.scale.x * FALLOFF_RING, t.scale.y * FALLOFF_RING),
                    ),
                    t.translation.x + wox,
                    t.translation.y + woy,
                    t.rotation,
                    camera,
                    window,
                ),
                FALLOFF_RGBA,
            ));
        }
    }
    out
}

/// Paint them. No-op when [`outlines`] returns nothing.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    show: bool,
    show_velocity: bool,
    sim: &mut SimWorld,
    joint_views: &[ph2d_physics_ecs::JointView],
    // A arena de roldanas que as views indexam (W-Pulley W1).
    joint_wheels: &[ph2d_physics_ecs::rope_route::RopeWheel],
    // O ângulo de cada roldana, paralelo à arena (W-Pulley W1).
    joint_spins: &[f32],
    joint_gravity: [f32; 2],
    // O limite sendo POSADO agora `(joint, rad relativo)` — desenha o fantasma
    // de B (W-J3). `None` sem arrasto de limite em voo.
    posed_limit: Option<(ph2d_ecs::Entity, f32)>,
    // W-J4: a banda elástica do gesto de criar `(de, para)` em mundo. Desenhada
    // mesmo com `show` FALSO — é gesto, não anotação (ver `draw_band`).
    join_band: Option<([f32; 2], [f32; 2])>,
    // W-Grab: a mola da MÃO `(cursor, ponto de pega)` em mundo. Desenhada mesmo
    // com `show` FALSO, pela mesma razão da banda: é gesto, não anotação.
    grab: Option<([f32; 2], [f32; 2])>,
    // W-Hand: a MIRA das ferramentas de ponto `(centro, alcance)` — onde o próximo
    // estouro/puxão vai agir. `None` para a mão (que não tem alcance) e sempre que
    // o gesto está inerte (relógio parado / física desarmada): uma mira que promete
    // o que o clique não faz é pior que mira nenhuma.
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
    // W20: algum player está a atravessar uma prancha AGORA? O bit da descida
    // viaja no CORPO e vale para toda plataforma one-way da cena, então a
    // resposta é uma só e o contorno de TODAS elas a veste (ver `PASSABLE_RGBA`).
    ghost: bool,
    // W-J7b: quem está selecionado, para o readout de carga de um joint que
    // ainda NÃO é quebrável — é preciso ler a carga antes de escolher um teto.
    selected_joint: Option<ph2d_ecs::Entity>,
    camera: &Camera2d,
    window: WindowSize,
    vector_scene: &mut VectorScene,
    text_system: &mut ph2d_text::TextSystem,
) {
    use ph2d_vector::{Affine, Brush, Color, Stroke};
    for (path, rgba) in outlines(show, show_velocity, sim, triggered, ghost, camera, window) {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(rgba)),
            None,
            &path,
        );
    }
    // A linha d'água ANTES dos corpos: ela é o cenário (onde a superfície está), e o
    // que se lê por cima dela são as coisas que boiam.
    for path in waterline_marks(show, waterlines, camera, window) {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(WATERLINE_RGBA)),
            None,
            &path,
        );
    }
    // ⚠️ **Os SENSORES depois dos corpos e ANTES dos contatos** (W-Probes): eles
    // descrevem para onde o personagem olha, então têm de ficar por cima da arte
    // que atravessam — e por baixo das cruzes, que marcam um evento e são as
    // menores marcas da tela.
    for (path, rgba) in
        super::physics_overlay_probes::probe_marks(show, probes, sim, camera, window)
    {
        vector_scene.inner_mut().stroke(
            &Stroke::new(super::physics_overlay_probes::PROBE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(rgba)),
            None,
            &path,
        );
    }
    // Contacts ON TOP of everything: a contact is the smallest mark on screen (a few
    // pixels) and it sits exactly ON the outlines of the two bodies that meet, so any
    // other order buries it under the shapes it is describing.
    for path in contact_marks(show, contacts, camera, window) {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(CONTACT_RGBA)),
            None,
            &path,
        );
    }
    // And the BEGIN-flash on top of the standing marks: it lives a few ticks and it is
    // the visible half of the contact-events channel (`contact_flashes`). Last, so the
    // spark is never buried by the very cross it is announcing.
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
    //
    // W-J1: uma cor por FATO (glifo · posse · limite · deformação), então o
    // laço percorre pares — a mesma forma que o `outlines` já usa.
    // A BANDA do gesto de criar, ANTES de tudo e SEM o gate de `show`: ela é o
    // feedback de algo que o artista está fazendo agora.
    if let Some(band) = super::physics_overlay_gesture::draw_band(join_band, camera, window) {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(super::physics_overlay_joints::JOINT_RGBA)),
            None,
            &band,
        );
    }
    // E a MÃO, ao lado da banda e pela mesma razão (gesto em andamento, sem o
    // gate de `show`). Depois da banda porque as duas nunca coexistem — criar um
    // joint é gesto de repouso, pegar um corpo é de play — e a ordem entre elas
    // só teria de ser decidida se algum dia coexistissem.
    if let Some(path) = super::physics_overlay_gesture::draw_grab(grab, camera, window) {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(super::physics_overlay_gesture::GRAB_RGBA)),
            None,
            &path,
        );
    }
    // ── As ferramentas de PONTO (W-Hand) ──────────────────────────────────
    // Sem o gate de `show`, pela mesma razão da banda e da mão: são gesto. O
    // desenho mora em `physics_overlay_annotations` porque é isso que ele é — uma
    // anotação — e porque este arquivo tem cap de 600 LOC.
    super::physics_overlay_annotations::draw_interaction(
        aim,
        pull,
        blast,
        camera,
        window,
        vector_scene,
    );
    // O FANTASMA primeiro: ele é o fundo do arco que o artista está arrastando,
    // e desenhá-lo por cima do glifo faria a silhueta apagar a agulha viva.
    if show
        && let Some(ghost) = super::physics_overlay_joint_ghost::limit_ghost(
            sim,
            joint_views,
            posed_limit,
            camera,
            window,
        )
    {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(
                super::physics_overlay_joint_ghost::JOINT_GHOST_RGBA,
            )),
            None,
            &ghost,
        );
    }
    // ⚠️ **A lista sai do DESENHO e é lida pelo READOUT**, nesta ordem
    // (W-RopeSays): quem sabe se a rota de uma corda resolve é a chamada que a
    // desenhou, e uma segunda pergunta divergiria — o desenho vermelho com um
    // `0 N` âmbar ao lado era exatamente essa divergência.
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
    // ⚠️ Os NÚMEROS por ÚLTIMO, depois do último uso do `vector_scene` para
    // traço — a cena tem de estar livre para o renderizador de texto, que é a
    // mesma ordem (e o mesmo comentário) do overlay de dimensões do Line.
    for r in super::physics_overlay_joint_readout::joint_readouts(
        show,
        joint_views,
        selected_joint,
        camera,
        window,
        &marks.not_acting,
    ) {
        // Centrado numa caixa larga o bastante para o rótulo mais longo
        // (`1234 / 1234 N.m`) e alta o bastante para uma linha.
        let rect = ph2d_editor::zones::Rect::new(
            r.at.x as f32 - READOUT_BOX_W_PX * 0.5,
            r.at.y as f32 - READOUT_BOX_H_PX * 0.5,
            READOUT_BOX_W_PX,
            READOUT_BOX_H_PX,
        );
        ph2d_editor::paint::paint_text_centered(
            text_system,
            vector_scene,
            &r.text,
            rect,
            super::physics_overlay_joint_readout::READOUT_PX,
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
// dos sensores (`W-Probes`), que moram num módulo IRMÃO. Duas câmeras fariam as
// duas famílias de gate medir projeções diferentes, e o dia em que uma delas
// mudasse a outra ficaria verde sobre outra premissa.
#[cfg(test)]
#[path = "physics_overlay_tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "physics_overlay_scene_tests.rs"]
mod scene_tests;

/// As ANOTAÇÕES sobre o contorno (setas + glifos) — o terceiro arquivo de gates do
/// overlay, separado dos irmãos pelo cap de 600 LOC (W-AreaFrame).
#[cfg(test)]
#[path = "physics_overlay_annotation_tests.rs"]
mod annotation_tests;

#[cfg(test)]
#[path = "physics_overlay_passable_tests.rs"]
mod passable_tests;
