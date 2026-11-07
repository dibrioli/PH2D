//! §11 Physics Body — row-painting helpers split out of `physics.rs` for the
//! panel's 600-LOC file cap (W-Mass pushed it over).
//!
//! These are pure "which rows does this body show" helpers; the section painter in
//! `physics.rs` calls them. They take resolved booleans (`mass_is_read`, `mass_manual`)
//! rather than the whole `InspectorPhysicsInfo`, so this file shares no private
//! consts with `physics.rs` — the two only meet at the call site.

use super::rows::{Campo, Escolha, num_row, par_row, seg_par_row, seg_row};
use super::*;
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;
/// ⭐ **O que um corpo EMITE, e PARA QUEM** — as rows de sinal e o filtro por tag, num irmão por
/// `#[path]`, cortado em 2026-09-14 pelo tecto de 600 LOC deste ficheiro (ele chegou a `677`).
///
/// ⚠️ **O corte é por ASSUNTO:** tudo o resto aqui descreve o CORPO (massa, atrito, camada, forma,
/// zona); estas três rows descrevem a **conversa** dele com o resto da cena — e a do filtro
/// pergunta à árvore do projecto, que nenhuma das vizinhas conhece. ⛔ Subir o número seria adiar
/// com juros.
#[path = "physics_tag_row.rs"]
mod tag_row;
pub(in crate::sections) use tag_row::paint_signal_rows;
pub(crate) use tag_row::phys_tag_options;

/// Mass-source toggle labels, indexed by `mass_manual as u8`: `0` Auto (mass is
/// density×area, the Density row) · `1` Manual (an explicit mass in kg, the Mass row).
pub(super) const MASS_MODE_LABELS: [TextKey; 2] = [
    TextKey::new("panel.inspector.physics.auto"),
    TextKey::new("panel.inspector.physics.manual"),
];

/// Combine-rule labels, indexed by `CombineRule` tag: how two colliders'
/// friction/restitution merge on contact (Unity's `PhysicMaterial` combine).
/// `Max` makes a superball bounce off any floor; `Average` (tag 0) is the default.
pub(super) const COMBINE_LABELS: [TextKey; 4] = [
    TextKey::new("panel.inspector.physics.average"),
    TextKey::new("panel.inspector.physics.min"),
    TextKey::new("panel.inspector.physics.multiply"),
    TextKey::new("panel.inspector.physics.max"),
];

/// Damping-mode toggle labels, indexed by `DampMode` tag: `0` Combine (adds to the
/// world default drag) · `1` Replace (ignores it — Unity's absolute per-body drag).
pub(super) const DAMP_MODE_LABELS: [TextKey; 2] = [
    TextKey::new("panel.inspector.physics.combine"),
    TextKey::new("panel.inspector.physics.replace"),
];

/// The **Dynamic-only** damping rows: linear + angular drag, and the mode that says
/// how they meet the world default drag (Combine adds, Replace ignores) (W-Damping).
///
/// Damping decays a velocity the solver owns, so it is meaningless on a Static
/// (never moves) or Kinematic (pose-driven) body — the same Dynamic-only rule the
/// gravity/velocity rows follow. Split here so the caller stays under the panel's
/// 200-LOC fn cap; the mode selection reads straight off the snapshot, so only the
/// two number boxes are synced.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_damping_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    damp_mode_tag: u8,
) -> f32 {
    // T56: no cartão, Linear | Angular numa linha, um rótulo sobre cada caixa.
    let yy = par_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        None,
        [
            Campo::new(
                TextKey::new("component.field.damping_override.0"),
                ids::INSP_PHYS_LINEAR_DAMPING,
                None,
            ),
            Campo::new(
                TextKey::new("component.field.damping_override.1"),
                ids::INSP_PHYS_ANGULAR_DAMPING,
                None,
            ),
        ],
    );
    seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        TextKey::new("component.field.damping_override.2"),
        ids::INSP_LIVE_PHYSICS_DAMPMODE,
        &ids::INSP_PHYS_DAMPMODE,
        &DAMP_MODE_LABELS.map(TextKey::tr),
        damp_mode_tag,
    )
}

/// The collider MATERIAL rows: **Bounce** + **Friction** (the coefficients) and,
/// right under each, how it COMBINES with the other collider on contact — a Bounce
/// Combine and a Friction Combine segmented control (W-Material).
///
/// Offered for ANY body kind, not Dynamic-only: a static floor's combine rule
/// matters too, because rapier takes the higher-priority of the two colliders' rules
/// (so a `Max` superball bounces off any floor). The two combine selections read
/// straight off the snapshot, so there is nothing to sync. Split here so
/// `paint_physics_section` stays under the panel's 200-LOC fn cap.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_material_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    restitution_combine_tag: u8,
    friction_combine_tag: u8,
) -> f32 {
    let yy = paint_bounce_friction(scene, text_system, theme, hit_index, store, x, w, y);
    paint_combine_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        restitution_combine_tag,
        friction_combine_tag,
    )
}

/// T56: no cartão, Bounce | Friction numa linha (o par do material, como o Physics Material 2D da Unity).
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_bounce_friction(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    par_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        None,
        [
            Campo::new(
                TextKey::new("component.field.collider.2"),
                ids::INSP_PHYS_RESTITUTION,
                None,
            ),
            Campo::new(
                TextKey::new("component.field.collider.3"),
                ids::INSP_PHYS_FRICTION,
                None,
            ),
        ],
    )
}

/// As duas regras de combinar — lado a lado quando cabem.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_combine_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    restitution_combine_tag: u8,
    friction_combine_tag: u8,
) -> f32 {
    let combine = COMBINE_LABELS.map(TextKey::tr);
    seg_par_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        Escolha {
            rotulo: TextKey::new("component.field.material_combine.0").into(),
            group: ids::INSP_LIVE_PHYSICS_REST_COMBINE,
            ids: &ids::INSP_PHYS_REST_COMBINE,
            labels: &combine,
            selected: restitution_combine_tag,
        },
        Escolha {
            rotulo: TextKey::new("component.field.material_combine.1").into(),
            group: ids::INSP_LIVE_PHYSICS_FRIC_COMBINE,
            ids: &ids::INSP_PHYS_FRIC_COMBINE,
            labels: &combine,
            selected: friction_combine_tag,
        },
    )
}

/// The mass-source rows: for a Dynamic body, the **Auto | Manual** toggle plus the
/// single live quantity row (Density in Auto, Mass in Manual); for any other kind, a
/// plain Density row.
///
/// Density and mass are the same quantity by two roads (`mass = density × area`), so
/// exactly one is ever live — showing both would be the "two doors to one quantity"
/// bug.
///
/// ⚠️ **The toggle is offered where the mass is READ, which is no longer the same as
/// "the body is Dynamic".** This doc used to say *Dynamic-only because a
/// Static/Kinematic body has infinite mass (rapier ignores both)* — still true of the
/// SOLVER, and false of the **kinematic player**, whose weight reaches the ground
/// through the 3rd law (K6): measured, a Snap player presses with **100.0% of `m·g`**,
/// exactly like the dynamic one. The caller resolves the question once
/// (`InspectorPhysicsInfo::mass_is_read`); bodies whose mass nothing reads keep the
/// plain Density row, unchanged from before this existed.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_mass_source(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    mass_is_read: bool,
    mass_manual: bool,
) -> f32 {
    let mut yy = y;
    if mass_is_read {
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            tr("panel.inspector.physics.mass"),
            ids::INSP_LIVE_PHYSICS_MASSMODE,
            &ids::INSP_PHYS_MASSMODE,
            &MASS_MODE_LABELS.map(TextKey::tr),
            u8::from(mass_manual),
        );
        let (label, id) = if mass_manual {
            (
                TextKey::new("component.field.mass_override.0"),
                ids::INSP_PHYS_MASS,
            )
        } else {
            (
                TextKey::new("component.field.collider.1"),
                ids::INSP_PHYS_DENSITY,
            )
        };
        yy = num_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            label,
            id,
        );
    } else {
        yy = num_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            TextKey::new("component.field.collider.1"),
            ids::INSP_PHYS_DENSITY,
        );
    }
    yy
}

/// Collision-layer chip labels. Bare numbers because a layer has no meaning of
/// its own — what it MEANS is the row it occupies in the world matrix, and that
/// is where the naming belongs. Naming them here would be a second place to
/// keep names in sync with a matrix that does not know about them.
pub(crate) const LAYER_LABELS: [&str; 8] = ["0", "1", "2", "3", "4", "5", "6", "7"];

/// Sensor toggle labels, indexed by `is_sensor as u8`: `0` a solid collider,
/// `1` a sensor (trigger).
pub(super) const SENSOR_LABELS: [TextKey; 2] = [
    TextKey::new("panel.inspector.physics.solid"),
    TextKey::new("panel.inspector.physics.sensor"),
];

/// One-way toggle labels, indexed by `one_way as u8`: `0` an ordinary solid collider,
/// `1` a jump-through platform (solid only from its local +Y side).
pub(super) const ONEWAY_LABELS: [TextKey; 2] = [
    TextKey::new("panel.inspector.physics.off"),
    TextKey::new("panel.inspector.physics.on"),
];

/// Wall-material labels, indexed by `no_wall_cling as u8`: `0` — a ausência do
/// marcador, o caso comum — é PAREDE; `1` a mão não a segura (`W-WallMaterial`).
pub(super) const WALLMAT_LABELS: [TextKey; 2] = [
    TextKey::new("panel.inspector.physics.on"),
    TextKey::new("panel.inspector.physics.off"),
];

/// The per-collider COLLISION rules: which layer it is on, whether it is solid or a
/// trigger, and then the one question that follows from THAT answer — a solid collider
/// asks *from which side* (one-way), a sensor asks *with what force* (the force zone).
///
/// ⚠️ **Those last two are mutually exclusive, and that is physics, not layout.** A
/// one-way platform is realised by modifying solver CONTACTS, and a sensor generates
/// none; a force zone is realised from the narrow phase's INTERSECTION graph, which
/// only records a pair when one side is a sensor. Each control is dead in the other
/// mode, so each is offered only in its own — the first §11 controls gated on another
/// CONTROL rather than on `kind_tag`.
///
/// **None is Dynamic-only:** the layer is a filter, a trigger is commonly Static
/// scenery, a jump-through platform is almost always Static and so is a wind column —
/// gating any of them on Dynamic would delete the control from its own use case. Split
/// here so `paint_physics_section` stays under the panel's 200-LOC fn cap; the
/// selections read straight off the snapshot, so only the force numbers are synced.
///
/// ⚠️ **O bloco de ZONA saiu daqui** (W-PartFace) para o irmão [`physics_area_rows::paint_area_rows`](super::physics_area_rows::paint_area_rows),
/// e o corte é o que o doc acima já desenhava: estas três rows dizem *como este
/// COLLIDER participa de uma colisão*, e as da zona dizem *o que esta ÁREA faz a
/// quem está dentro dela* — perguntas diferentes com respostas de escopo
/// diferente. A separação existe porque uma **peça** (`Collider` sem `RigidBody`)
/// tem as três primeiras vivas e **nenhuma** da zona: `reconcile_parts` não lê
/// efetor nenhum, então pintá-las numa peça seriam sete knobs que o solver
/// ignora. A ORDEM na tela não muda para um corpo — one-way e zona são
/// mutuamente exclusivos, então o chamador as pinta em sequência e o resultado é
/// o mesmo retângulo de sempre.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_collision_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    layer: u8,
    is_sensor: bool,
    one_way: bool,
    no_wall_cling: bool,
    // ⭐ O FILTRO por tag dos sinais (TOP-20 #9, W3c): `None` = sem componente, `Some(0)` = anexado
    // e por escolher, e o CAMINHO vazio com um id vivo = a tag foi apagada.
    signal_tag: Option<u64>,
    signal_tag_path: &str,
) -> f32 {
    let mut yy = y;
    // The per-body half of collision layers. The other half — WHICH layers collide —
    // is a world rule and lives in the Physics panel; a body only says where it belongs.
    for (label, group, opts, labels, sel) in [
        (
            TextKey::new("component.field.collider.4"),
            ids::INSP_LIVE_PHYSICS_LAYER,
            &ids::INSP_PHYS_LAYER[..],
            &LAYER_LABELS[..],
            layer,
        ),
        (
            TextKey::new("component.field.collider.5"),
            ids::INSP_LIVE_PHYSICS_SENSOR,
            &ids::INSP_PHYS_SENSOR[..],
            &SENSOR_LABELS.map(TextKey::tr),
            u8::from(is_sensor),
        ),
    ] {
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            label,
            group,
            opts,
            labels,
            sel,
        );
    }
    if !is_sensor {
        // A SOLID collider: which side is it solid from?
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            tr("panel.inspector.physics.one_way"),
            ids::INSP_LIVE_PHYSICS_ONEWAY,
            &ids::INSP_PHYS_ONEWAY,
            &ONEWAY_LABELS.map(TextKey::tr),
            u8::from(one_way),
        );
    }
    yy = paint_surface_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        no_wall_cling,
    );
    paint_signal_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        signal_tag,
        signal_tag_path,
    )
}

/// Wall Cling, Grip e Belt.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_surface_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    no_wall_cling: bool,
) -> f32 {
    // **De que este chão é feito para quem ANDA sobre ele** (`W-Surface`).
    //
    // ⚠️ **Aqui, e oferecidas em TODO collider — sólido ou sensor, qualquer
    // kind.** A superfície que importa é quase sempre um chão ESTÁTICO (o gelo,
    // a esteira), então restringi-la a Dynamic deletaria o controle exatamente
    // onde ele serve; e a peça de um corpo composto carrega a sua, que é o que
    // deixa uma plataforma ter uma face de gelo e outra de borracha.
    //
    // ⚠️ **Só a lei do PLAYER as lê**, e isto é uma limitação NOMEADA e não um
    // descuido: um caixote sobre a esteira não é levado por ela. O
    // `SurfaceEffector2D` da Unity leva qualquer rigidbody; aqui o alcance é o
    // que o nome diz.
    let yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        tr("panel.inspector.physics.wall_cling"),
        ids::INSP_LIVE_PHYSICS_WALLMAT,
        &ids::INSP_PHYS_WALLMAT,
        &WALLMAT_LABELS.map(TextKey::tr),
        u8::from(no_wall_cling),
    );
    // T56: no cartão, Grip | Belt numa linha — as duas metades da superfície de andar.
    par_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        None,
        [
            Campo::new(
                TextKey::new("component.field.walk_surface.0"),
                ids::INSP_PHYS_WALK_GRIP,
                None,
            ),
            Campo::new(
                TextKey::new("component.field.walk_surface.1"),
                ids::INSP_PHYS_WALK_BELT,
                Some(ph2d_editor_core::widget::Unit::MetersPerSecond),
            ),
        ],
    )
}
