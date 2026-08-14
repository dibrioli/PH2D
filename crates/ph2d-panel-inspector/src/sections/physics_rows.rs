//! §11 Physics Body — row-painting helpers split out of `physics.rs` for the
//! panel's 600-LOC file cap (W-Mass pushed it over).
//!
//! These are pure "which rows does this body show" helpers; the section painter in
//! `physics.rs` calls them. They take resolved booleans (`mass_is_read`, `mass_manual`)
//! rather than the whole `InspectorPhysicsInfo`, so this file shares no private
//! consts with `physics.rs` — the two only meet at the call site.

use super::rows::{num_row, seg_row};
use super::*;

/// Mass-source toggle labels, indexed by `mass_manual as u8`: `0` Auto (mass is
/// density×area, the Density row) · `1` Manual (an explicit mass in kg, the Mass row).
const MASS_MODE_LABELS: [&str; 2] = ["Auto", "Manual"];

/// Combine-rule labels, indexed by `CombineRule` tag: how two colliders'
/// friction/restitution merge on contact (Unity's `PhysicMaterial` combine).
/// `Max` makes a superball bounce off any floor; `Average` (tag 0) is the default.
const COMBINE_LABELS: [&str; 4] = ["Average", "Min", "Multiply", "Max"];

/// Damping-mode toggle labels, indexed by `DampMode` tag: `0` Combine (adds to the
/// world default drag) · `1` Replace (ignores it — Unity's absolute per-body drag).
const DAMP_MODE_LABELS: [&str; 2] = ["Combine", "Replace"];

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
    let mut yy = y;
    for (label, id) in [
        ("Linear Damping", ids::INSP_PHYS_LINEAR_DAMPING),
        ("Angular Damping", ids::INSP_PHYS_ANGULAR_DAMPING),
    ] {
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
    }
    seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Damp Mode",
        ids::INSP_LIVE_PHYSICS_DAMPMODE,
        &ids::INSP_PHYS_DAMPMODE,
        &DAMP_MODE_LABELS,
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
    let mut yy = y;
    for (label, id) in [
        ("Bounce", ids::INSP_PHYS_RESTITUTION),
        ("Friction", ids::INSP_PHYS_FRICTION),
    ] {
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
    }
    // How Bounce/Friction combine with the OTHER collider — one segmented control
    // each, sitting right under the value it governs.
    for (label, group, ids, tag) in [
        (
            "Bounce Combine",
            ids::INSP_LIVE_PHYSICS_REST_COMBINE,
            &ids::INSP_PHYS_REST_COMBINE,
            restitution_combine_tag,
        ),
        (
            "Friction Combine",
            ids::INSP_LIVE_PHYSICS_FRIC_COMBINE,
            &ids::INSP_PHYS_FRIC_COMBINE,
            friction_combine_tag,
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
            ids,
            &COMBINE_LABELS,
            tag,
        );
    }
    yy
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
            "Mass",
            ids::INSP_LIVE_PHYSICS_MASSMODE,
            &ids::INSP_PHYS_MASSMODE,
            &MASS_MODE_LABELS,
            u8::from(mass_manual),
        );
        let (label, id) = if mass_manual {
            ("Mass (kg)", ids::INSP_PHYS_MASS)
        } else {
            ("Density", ids::INSP_PHYS_DENSITY)
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
            "Density",
            ids::INSP_PHYS_DENSITY,
        );
    }
    yy
}

/// Collision-layer chip labels. Bare numbers because a layer has no meaning of
/// its own — what it MEANS is the row it occupies in the world matrix, and that
/// is where the naming belongs. Naming them here would be a second place to
/// keep names in sync with a matrix that does not know about them.
const LAYER_LABELS: [&str; 8] = ["0", "1", "2", "3", "4", "5", "6", "7"];

/// Sensor toggle labels, indexed by `is_sensor as u8`: `0` a solid collider,
/// `1` a sensor (trigger).
const SENSOR_LABELS: [&str; 2] = ["Solid", "Sensor"];

/// One-way toggle labels, indexed by `one_way as u8`: `0` an ordinary solid collider,
/// `1` a jump-through platform (solid only from its local +Y side).
const ONEWAY_LABELS: [&str; 2] = ["Off", "On"];

/// Force-frame labels, indexed by `world_axes as u8`: `0` the zone's own frame (turn
/// the sensor and the wind turns with it), `1` pinned to world axes (the zone turns,
/// the blow does not).
const FORCE_AXES_LABELS: [&str; 2] = ["Zone", "World"];

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
/// ⚠️ **O bloco de ZONA saiu daqui** (W-PartFace) para o irmão [`paint_area_rows`],
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
) -> f32 {
    let mut yy = y;
    // The per-body half of collision layers. The other half — WHICH layers collide —
    // is a world rule and lives in the Physics panel; a body only says where it belongs.
    for (label, group, opts, labels, sel) in [
        (
            "Layer",
            ids::INSP_LIVE_PHYSICS_LAYER,
            &ids::INSP_PHYS_LAYER[..],
            &LAYER_LABELS[..],
            layer,
        ),
        (
            "Trigger",
            ids::INSP_LIVE_PHYSICS_SENSOR,
            &ids::INSP_PHYS_SENSOR[..],
            &SENSOR_LABELS[..],
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
            "One-Way",
            ids::INSP_LIVE_PHYSICS_ONEWAY,
            &ids::INSP_PHYS_ONEWAY,
            &ONEWAY_LABELS,
            u8::from(one_way),
        );
    }
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
    for (label, id) in [
        ("Grip", ids::INSP_PHYS_WALK_GRIP),
        ("Belt (m/s)", ids::INSP_PHYS_WALK_BELT),
    ] {
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
    }
    // **O que este objeto GRITA quando algo chega nele — e quando algo sai**
    // (W-Signal · W-SignalLeave).
    //
    // ⚠️ Aqui e não num card próprio: elas respondem *"e daí?"* à pergunta que as
    // rows acima fazem — um Trigger sem sinal detecta e não acorda nada —, e um
    // controle que só faz sentido junto de outros tem de estar onde eles estão
    // (o argumento do `INSP_JOINT_ANCHOR_B_GROUP`, palavra por palavra).
    //
    // ⚠️ **Oferecidas em TODO collider**, sólido ou sensor: as fontes de cada
    // extremo são um contato que começa/termina e uma entrada/saída de sensor, e
    // restringir a row a um dos dois deixaria metade dos casos sem porta. Uma
    // entidade é uma coisa ou a outra, então um nome por extremo responde às duas.
    //
    // ⚠️ **DUAS rows, e não uma com seletor:** os dois extremos são dois
    // CONTRATOS que o artista quer autorar ao mesmo tempo (`door_open` /
    // `door_close`), e um campo que trocasse de significado tornaria o caso de
    // uso inteiro inexprimível.
    for (id, placeholder) in [
        (ids::INSP_PHYS_SIGNAL, "Signal on hit\u{2026}"),
        (ids::INSP_PHYS_SIGNAL_LEAVE, "Signal on leave\u{2026}"),
    ] {
        yy = signal_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            id,
            placeholder,
        );
    }
    yy
}

/// A row de um nome de sinal — um `TextInput`, porque o valor É uma string e o
/// contrato é por NOME (ADR-0143).
///
/// ⚠️ **Uma função, dois extremos.** Chegada e saída desenham a MESMA coisa e só
/// diferem no id e no placeholder; duas cópias divergiriam no dia em que uma
/// delas ganhasse um estado (foco, erro, dimmed) e a outra não.
#[allow(clippy::too_many_arguments)]
fn signal_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    placeholder: &str,
) -> f32 {
    let host = Rect::new(x, y, w, ROW_H_PX);
    hit_index.register(id, host);
    let (state, text, caret, anchor) = match store.get(id) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, Some(text.as_str()), *caret, *selection_anchor),
        _ => (TextInputState::Normal, None, 0, None),
    };
    let input = TextInput::new(id, "").placeholder(placeholder).state(state);
    paint_text_input_with_buffer(
        &input,
        text,
        Some(caret),
        anchor,
        host,
        scene,
        text_system,
        theme,
    );
    y + ROW_H_PX
}

/// **O que esta ÁREA faz a quem está dentro dela** — o bloco de zona, irmão do
/// [`paint_collision_rows`] (W-PartFace).
///
/// Só faz sentido num collider **sensor**, e o chamador é quem decide isso: um
/// corpo sólido não tem zona, e uma **peça** não tem zona alguma (a ponte não lê
/// efetor nenhum de uma peça — ver o doc do irmão).
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_area_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    force_world_axes: bool,
) -> f32 {
    let mut yy = y;
    {
        // A SENSOR: what force does this area apply to whatever is inside it? Wind,
        // an updraft, a conveyor. Newtons, so it is resisted by mass — the number an
        // artist tunes against a body's own weight.
        // Force is what the area PUSHES with; Drag is what it RESISTS with. Together
        // they are the difference between wind (push, no resistance) and water.
        for (label, id) in [
            ("Force X (N)", ids::INSP_PHYS_FORCE_X),
            ("Force Y (N)", ids::INSP_PHYS_FORCE_Y),
        ] {
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
        }
        // In WHOSE axes are those two numbers? Directly under them, and deliberately
        // ABOVE everything else in this branch: it governs the FORCE and nothing else.
        // That is geometry rather than a scope someone chose — a 2D torque is a scalar
        // about Z and an in-plane rotation is about Z, so there is nothing to turn; drag
        // is isotropic; buoyancy measures its surface from GRAVITY (water is level even
        // in a tilted pool); and shape drag pushes along each edge normal of the BODY.
        // Painted below the others, the row would read as qualifying all of them.
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            "Force Axes",
            ids::INSP_LIVE_PHYSICS_FORCE_AXES,
            &ids::INSP_PHYS_FORCE_AXES,
            &FORCE_AXES_LABELS,
            u8::from(force_world_axes),
        );
        // Then the PUSH block closes with the two rows that qualify it: the spin the area
        // imprints, and how much of both survives the trip to the edge.
        //
        // ⚠️ Falloff sits directly under Torque, and ABOVE Drag, because that is exactly
        // the boundary of what it weighs: the force and the torque — the two PUSHES —
        // and nothing below. Drag, Fluid Density and Shape Drag describe a MEDIUM, and a
        // medium does not thin out near its own edge (the water at the side of the pool
        // is just as wet). Painted below them the row would read as governing all six.
        for (label, id) in [
            ("Torque (N·m)", ids::INSP_PHYS_AREA_TORQUE),
            ("Falloff", ids::INSP_PHYS_AREA_FALLOFF),
            ("Drag", ids::INSP_PHYS_AREA_DRAG),
            ("Fluid Density", ids::INSP_PHYS_AREA_DENSITY),
            ("Shape Drag", ids::INSP_PHYS_AREA_FORM_DRAG),
        ] {
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
        }
    }
    yy
}
