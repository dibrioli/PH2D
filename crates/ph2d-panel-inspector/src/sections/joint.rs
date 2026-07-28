//! Physics Joint — Inspector §12 section painter (W3).
//!
//! A joint is an **entity**, so this section describes the selected joint
//! object rather than a property of a body. It appears when — and only when —
//! the selection carries a `PhysicsJoint`.
//!
//! **Only the chosen kind's parameters are painted.** A stiffness field on a
//! rope is a control that cannot do anything, which is worse than a missing
//! one because it looks like it should work — the same rule §11 already
//! follows for a radius on a box. The question *"does this kind have a
//! motor?"* is answered by `JointKind::is_hinge` in `ph2d-physics-ecs`, and
//! the bridge asks the SAME function before handing a motor to the solver, so
//! a knob cannot be painted for a kind that ignores it.

#[path = "joint_cards.rs"]
mod cards;
use cards::{paint_break_rows, paint_motor_rows};

use super::rows::{num_row, seg_row};
use super::*;
use ph2d_editor_core::screens::hero::InspectorJointInfo;

/// Joint-kind labels, indexed by the tag the snapshot carries. Hardcoded here
/// (not read from `ph2d-physics-ecs`) so the panel stays loose-coupled, like
/// every sibling section. English per HR-15.
const KIND_LABELS: [&str; 8] = [
    "Pin", "Spring", "Rope", "Weld", "Slider", "Rod", "Wheel", "Pulley",
];

/// The two Pin-only switches. A two-option segmented IS a switch, and it is
/// the widget this section already speaks.
///
/// `pub(super)` because the pair cluster next door speaks the same two words —
/// one list, so an "On"/"Enabled" drift between two halves of one section is not
/// a thing that can happen.
pub(super) const SWITCH_LABELS: [&str; 2] = ["Off", "On"];

/// Tag of the Pin kind — named because the painter branches on it and a bare
/// `0` at a branch survives a refactor pointing at the wrong variant.
const KIND_PIN: u8 = 0;
/// Tag of the Spring kind.
const KIND_SPRING: u8 = 1;
/// Tag of the Rope kind. Named so the Rope branch is explicit and a Weld
/// (tag 3) — which has no parameter rows at all — falls through to nothing
/// instead of inheriting the Rope's "Max Length" from a bare `else`.
const KIND_ROPE: u8 = 2;
/// Tag of the Rod kind. A rigid bar: ONE number (the length), no limits and no
/// motor — so it paints exactly one row, and shares `INSP_JOINT_MAX_LENGTH` with
/// the Rope because engine-side it is the same authored field.
const KIND_ROD: u8 = 5;
/// Tag of the Slider kind. It shares the **Limits** switch with the Pin (both
/// have a range) and, since W-J6, a motor as well — so the painter asks the two
/// questions separately instead of branching on "is it a Pin?", which is the
/// same split `JointKind::has_limits` and `has_motor` made engine-side.
const KIND_SLIDER: u8 = 4;
/// Tag of the Wheel kind — a hub that **spins and rides a suspension**, so it
/// is the first kind to want TWO families of row at once (a travel range like a
/// Slider, a spring like a Spring). That is what turned the `else if` chain
/// below into independent questions.
const KIND_WHEEL: u8 = 6;
/// Tag da POLIA — uma corda por duas roldanas. É o primeiro tipo que **não é um
/// joint do rapier** (a ponte o roteia para um passe de impulso próprio), e o
/// primeiro que não pode PARTIR: nada mede a reação de algo que não está no
/// `ImpulseJointSet`, então a caixa de Break não é oferecida a ele.
const KIND_PULLEY: u8 = 7;

/// Does this kind have a limit RANGE? A Pin's angular arc, a Slider's stroke,
/// a Wheel's suspension travel.
///
/// ⚠️ Second STATEMENT of `JointKind::has_limits`, like `kind_has_motor` below —
/// the panel never sees `ph2d-physics-ecs`, and the bridge asks the engine-side
/// door before handing limits to the solver.
const fn kind_has_limits(kind_tag: u8) -> bool {
    kind_tag == KIND_PIN || kind_tag == KIND_SLIDER || kind_tag == KIND_WHEEL
}

/// What the limits switch is CALLED for this kind. A Pin and a Slider are
/// *limited*; a Wheel's range is its suspension **travel**, which is the word
/// the artist is looking for — the same "same id, different label" the Rope and
/// the Rod already share for their one number.
const fn limits_label(kind_tag: u8) -> &'static str {
    if kind_tag == KIND_WHEEL {
        "Travel"
    } else {
        "Limits"
    }
}

/// Does this kind carry a **spring** the artist tunes (stiffness + damping)? A
/// Spring is one; a Wheel's suspension IS one.
///
/// The two share the fields and the ids because they are the same physical
/// thing — what differs is the SCALE they want (a spring hangs a body, a
/// suspension holds a vehicle up), which is why the kind change re-seeds them
/// engine-side (`PhysicsJoint::default_spring`).
const fn kind_has_spring(kind_tag: u8) -> bool {
    kind_tag == KIND_SPRING || kind_tag == KIND_WHEEL
}

/// **Este tipo pode PARTIR sob carga?** Todos, hoje.
///
/// A POLIA era a exceção — ela não é um joint do rapier, e nada publicava a
/// reação que decidiria a ruptura. O W-Pulley W2 fez o passe publicar a tensão
/// (`λ/dt`), então a exceção caiu.
///
/// ⚠️ Segundo ENUNCIADO de `JointKind::can_break`, não uma segunda fonte de
/// verdade (o painel é loose-coupled, a convenção de toda seção irmã) — e a
/// função FICA, constante, pelo mesmo motivo que a do motor: ela é o lugar onde
/// a pergunta é feita, e o próximo tipo que não puder partir volta a ter onde
/// dizê-lo.
const fn kind_can_break(_kind_tag: u8) -> bool {
    true
}

/// Can this kind be DRIVEN? A Pin's hinge, a Slider's rail, a Rope's distance
/// (the winch), a Wheel's spin (the drive).
///
/// ⚠️ Second STATEMENT of `JointKind::has_motor`, not a second source of truth —
/// the panel is loose-coupled and never sees `ph2d-physics-ecs` (the convention
/// of every sibling section). The bridge asks the engine-side door before handing
/// a motor to the solver, so a kind that gained a card here without gaining one
/// there would paint a knob the solver drops; a seam gate walks all five kinds
/// and pins which ones offer the card.
const fn kind_has_motor(kind_tag: u8) -> bool {
    kind_tag == KIND_PIN
        || kind_tag == KIND_SLIDER
        || kind_tag == KIND_ROPE
        || kind_tag == KIND_WHEEL
}

/// The unit pair the motor rows are labelled with, **for this kind**:
/// `(rate, place)`. Degrees for a hinge, metres for a rail or a winch.
///
/// ⚠️ **Deliberately not `limit_unit`, and the Rope is why:** a Rope has no limit
/// range at all and still has a linear motor, so one function answering both
/// questions would label a winch's target in degrees. Engine-side the same two
/// doors are `limits_in_metres` and `motor_in_metres`.
const fn motor_units(kind_tag: u8) -> (&'static str, &'static str) {
    if kind_tag == KIND_PIN || kind_tag == KIND_WHEEL {
        ("\u{00b0}/s", "\u{00b0}")
    } else {
        ("m/s", "m")
    }
}

/// Velocity · Position — the two things a motor can be told.
const MOTOR_MODE_LABELS: [&str; 2] = ["Velocity", "Position"];
/// Tag of the Position (servo) mode, named because the painter branches on it.
const MOTOR_MODE_POSITION: u8 = 1;

/// The unit the limit rows are in, **for this kind**. Degrees for a hinge's
/// angular range, metres for a slider's stroke.
///
/// ⚠️ The panel is loose-coupled and hardcodes its own labels (the convention of
/// every sibling section), so this is a second STATEMENT of
/// `JointKind::limits_in_metres` rather than a second source of truth: the
/// shell converts the value, this only names it. A seam gate pins that a slider's
/// rows say metres, so the two cannot drift apart in silence.
const fn limit_unit(kind_tag: u8) -> &'static str {
    if kind_tag == KIND_SLIDER || kind_tag == KIND_WHEEL {
        "m"
    } else {
        "\u{00b0}"
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_joint_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorJointInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let collapsed = store.is_collapsed(ids::INSP_LIVE_JOINT_SECTION);
    let color_id = ids::INSP_LIVE_JOINT_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default section accent
    let header = SectionHeader::new(ids::INSP_LIVE_JOINT_SECTION, "Physics Joint")
        .collapsible(!collapsed)
        .color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    if collapsed {
        return y + header_h;
    }

    let mut yy = y + header_h;
    let h = ROW_H_PX;

    // **Active comes FIRST** (W-J8) — it qualifies everything below it. The rows
    // stay painted and stay editable while it is off: an inactive joint is one
    // you are still authoring, which is the whole difference from a deleted one.
    yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Active",
        ids::INSP_JOINT_ACTIVE_GROUP,
        &ids::INSP_JOINT_ACTIVE,
        &SWITCH_LABELS,
        u8::from(info.active),
    );

    // The PAIR cluster: who the two ends are, the gesture that exchanges them,
    // and the one fact that is about the pair rather than about the constraint.
    // Its own module (`joint_pair_rows`) — the same cut the section draws on
    // screen, and the one the 600-LOC panel cap asked for.
    yy = super::joint_pair_rows::paint_pair_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        info,
    );

    yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Kind",
        ids::INSP_JOINT_KIND_GROUP,
        &ids::INSP_JOINT_KIND,
        &KIND_LABELS,
        info.kind_tag,
    );

    // Os parâmetros do tipo escolhido — a família inteira num helper, tanto pelo
    // cap de 200 LOC desta fn quanto porque *"o que este tipo tem a afinar"* é um
    // assunto só, e ele cresce a cada tipo novo.
    yy = paint_kind_params(scene, text_system, theme, hit_index, store, x, w, yy, info);

    // The motor comes LAST and is asked of every driven kind, rather than living
    // inside the Pin's branch as it did until W-J6: a rail and a winch are driven
    // too, and burying the card in one kind's arm is how the other two would have
    // been given a knob the solver ignores.
    if kind_has_motor(info.kind_tag) {
        yy = paint_motor_rows(scene, text_system, theme, hit_index, store, x, w, yy, info);
    }

    // Breaking comes after the parameters and before Delete: it is a property of
    // the joint as a whole (every kind can be pulled apart), not of one kind's
    // degree of freedom, so it is asked of ALL five.
    if kind_can_break(info.kind_tag) {
        yy = paint_break_rows(scene, text_system, theme, hit_index, store, x, w, yy, info);
    }

    let btn_rect = Rect::new(x, yy, w, h);
    let btn = Button::new(ids::INSP_JOINT_REMOVE, "Delete Joint")
        .kind(ButtonKind::Default)
        .state(
            store
                .button_state(ids::INSP_JOINT_REMOVE)
                .unwrap_or(ButtonState::Normal),
        );
    paint_button(&btn, btn_rect, scene, text_system, theme);
    hit_index.register(ids::INSP_JOINT_REMOVE, btn_rect);
    yy + h + SECTION_BOTTOM_PAD_PX
}

/// **Os parâmetros do TIPO escolhido** — limites, mola e comprimento.
///
/// Fn própria pelo cap de 200 LOC da seção e porque a família cresce por tipo:
/// o Wheel trouxe a primeira combinação de DUAS famílias (curso + mola), que foi
/// o que transformou a cadeia `else if` daqui em perguntas independentes.
#[allow(clippy::too_many_arguments)]
fn paint_kind_params(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorJointInfo,
) -> f32 {
    let mut yy = y;
    // ⚠️ **Perguntas INDEPENDENTES, não uma cadeia `else if`** — o
    // [`KIND_WHEEL`] é o primeiro tipo que quer DUAS famílias de linha (o curso,
    // que era do Pin/Slider, e a mola, que era da Spring), e numa cadeia ele
    // teria de escolher uma. A cadeia também já era frágil pelo outro lado: o
    // comentário do [`KIND_ROPE`] registra que um Weld herdaria o "Max Length"
    // de um `else` nu. Cada família agora se oferece sozinha.
    if kind_has_limits(info.kind_tag) {
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            limits_label(info.kind_tag),
            ids::INSP_JOINT_LIMITS_GROUP,
            &ids::INSP_JOINT_LIMITS,
            &SWITCH_LABELS,
            u8::from(info.limits_enabled),
        );
        if info.limits_enabled {
            let unit = limit_unit(info.kind_tag);
            for (label, id) in [
                (format!("Min ({unit})"), ids::INSP_JOINT_LIMIT_MIN),
                (format!("Max ({unit})"), ids::INSP_JOINT_LIMIT_MAX),
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
                    &label,
                    id,
                );
            }
        }
    }
    if info.kind_tag == KIND_SPRING {
        yy = num_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            "Rest Length (m)",
            ids::INSP_JOINT_REST_LENGTH,
        );
    }
    // A mola: da Spring (que PENDURA um corpo) e do Wheel (cuja suspensão
    // SUSTENTA um). Mesmos dois campos, mesmos dois ids — é a mesma coisa
    // física, e por isso a troca de tipo re-semeia a ESCALA deles.
    if kind_has_spring(info.kind_tag) {
        for (label, id) in [
            ("Stiffness", ids::INSP_JOINT_STIFFNESS),
            ("Damping", ids::INSP_JOINT_DAMPING),
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
    if info.kind_tag == KIND_ROPE || info.kind_tag == KIND_ROD || info.kind_tag == KIND_PULLEY {
        // O MESMO id, rótulo diferente: numa corda o número é um TETO, numa
        // barra é o comprimento em si, e numa polia é a corda INTEIRA (a soma
        // dos dois ramos). Um segundo id seria um segundo lugar para o mesmo
        // campo do componente.
        let label = match info.kind_tag {
            KIND_ROD => "Length (m)",
            KIND_PULLEY => "Rope Length (m)",
            _ => "Max Length (m)",
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
            ids::INSP_JOINT_MAX_LENGTH,
        );
    }
    if info.kind_tag == KIND_PULLEY {
        // **Acrescentar uma roldana** (pedido 4). O botão mora aqui — na seção da
        // CORDA — porque é a corda que possui a lista, e porque é onde o artista
        // está quando pensa *"esta corda precisa de mais uma"*. A contagem no
        // rótulo é o que torna o clique VISÍVEL: a roldana nova nasce SOBRE a
        // corda, para não dar um puxão, e ali o desenho quase não muda.
        let rect = Rect::new(x, yy, w, ROW_H_PX);
        let btn = Button::new(
            ids::INSP_JOINT_ADD_WHEEL,
            format!("Add Wheel ({} on this rope)", info.wheel_count),
        )
        .kind(ButtonKind::Default)
        .state(
            store
                .button_state(ids::INSP_JOINT_ADD_WHEEL)
                .unwrap_or(ButtonState::Normal),
        );
        paint_button(&btn, rect, scene, text_system, theme);
        hit_index.register(ids::INSP_JOINT_ADD_WHEEL, rect);
        yy += ROW_H_PX;
    }
    yy
}

#[cfg(test)]
mod kind_chip_tests {
    use super::{KIND_LABELS, KIND_SLIDER, limit_unit};
    use ph2d_editor_core::ids;

    /// **Um rótulo por id, e a razão é um `zip` que TRUNCA.**
    ///
    /// O `seg_row` casa `option_ids.zip(labels)`, então um rótulo sem id **não é
    /// pintado** — sem erro, sem warning, e o chip nasce inalcançável. Foi
    /// exatamente o que aconteceu quando o Slider chegou: cinco rótulos, quatro
    /// ids, e o gate de seam dos chips ficou verde porque ele iterava a lista
    /// CURTA (os ids). Comparar os dois comprimentos é a asserção que nenhuma
    /// das duas listas pode satisfazer sozinha.
    ///
    /// ⚠️ **E o par existe DUAS vezes:** este (§12, o tipo que a joint É) e o
    /// *Join As* do §11 (o tipo que o próximo gesto CRIA), com gate irmão em
    /// `sections::physics_rows`. Foi escrever só ESTE que deixou o chip do Slider
    /// faltar no seletor de criação — o artista via o tipo na simulação e não
    /// conseguia escolhê-lo.
    #[test]
    fn every_kind_label_has_an_id_to_be_clicked_by() {
        assert_eq!(
            KIND_LABELS.len(),
            ids::INSP_JOINT_KIND.len(),
            "um rotulo sem id e um chip que o seg_row DESCARTA no zip"
        );
    }

    /// O rótulo do curso diz **metros** para um trilho e **graus** para o resto —
    /// a segunda metade da porta `JointKind::limits_in_metres` (a primeira
    /// converte o número; esta o nomeia).
    #[test]
    fn a_rails_range_is_named_in_metres() {
        assert_eq!(limit_unit(KIND_SLIDER), "m");
        for other in [0u8, 1, 2, 3] {
            assert_eq!(limit_unit(other), "\u{00b0}");
        }
    }
}
