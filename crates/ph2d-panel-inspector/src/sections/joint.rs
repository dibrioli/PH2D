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
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;
use ph2d_i18n::tr_with;

#[path = "joint_custom.rs"]
mod custom;
use custom::{paint_axis_rows, paint_motor_axis_row};

#[path = "joint_kind_rows.rs"]
mod kind_rows;
use kind_rows::paint_kind_params;

use super::rows::seg_row;
use super::*;
use ph2d_editor_core::screens::hero::InspectorJointInfo;
use ph2d_editor_core::widget::SectionFold;

/// Joint-kind labels, indexed by the tag the snapshot carries. Hardcoded here
/// (not read from `ph2d-physics-ecs`) so the panel stays loose-coupled, like
/// every sibling section. English per HR-15.
const KIND_LABELS: [TextKey; 9] = [
    TextKey::new("panel.inspector.joint.pin"),
    TextKey::new("panel.inspector.joint.spring"),
    TextKey::new("panel.inspector.joint.rope"),
    TextKey::new("panel.inspector.joint.weld"),
    TextKey::new("panel.inspector.joint.slider"),
    TextKey::new("panel.inspector.joint.rod"),
    TextKey::new("panel.inspector.joint.wheel"),
    TextKey::new("panel.inspector.joint.pulley"),
    TextKey::new("panel.inspector.joint.custom"),
];

/// The two Pin-only switches. A two-option segmented IS a switch, and it is
/// the widget this section already speaks.
///
/// `pub(super)` because the pair cluster next door speaks the same two words —
/// one list, so an "On"/"Enabled" drift between two halves of one section is not
/// a thing that can happen.
pub(super) const SWITCH_LABELS: [TextKey; 2] = [
    TextKey::new("panel.inspector.joint.off"),
    TextKey::new("panel.inspector.joint.on"),
];

/// **O rótulo do botão Paste** (W-JointCopy) — porta pública porque o gate lê
/// dela, e não de uma segunda cópia da regra: um rótulo afirmado num teste que
/// re-escreve o `format!` fica verde enquanto a tela diz outra coisa (a lição do
/// `bake_label`, um irmão adiante).
///
/// **Um alvo** é o gesto de sempre e não precisa de número; **mais de um** tem de
/// dizer quantos ANTES do clique, porque o fan-out é o que este botão tem de
/// diferente e um clique que muda dez objetos não pode surpreender.
#[must_use]
pub fn paste_label(targets: usize) -> String {
    if targets > 1 {
        tr_with("panel.inspector.joint.paste_to", &[("targets", &targets)])
    } else {
        tr("panel.inspector.joint.paste_properties").to_string()
    }
}

/// Tag of the Pin kind — named because the painter branches on it and a bare
/// `0` at a branch survives a refactor pointing at the wrong variant.
const KIND_PIN: u8 = 0;
/// Tag of the Spring kind.
const KIND_SPRING: u8 = 1;
/// Tag of the Rope kind. Named so the Rope branch is explicit and a Weld
/// falls through to nothing instead of inheriting the Rope's "Max Length" from
/// a bare `else`.
const KIND_ROPE: u8 = 2;
/// Tag of the Weld kind. ⚠️ **Ele deixou de não ter parâmetro nenhum** — o doc do
/// [`KIND_ROPE`] dizia *"which has no parameter rows at all"*, verdade até a
/// W-SoftWeld: uma solda MOLE oferece a chave `Rigid | Soft` e, marcada, os dois
/// campos da mola.
const KIND_WELD: u8 = 3;
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
pub(crate) const KIND_PULLEY: u8 = 7;
/// Tag do CUSTOM — a configuração de eixos é AUTORADA. É o primeiro tipo cujo
/// par de limites único não é usado (os batentes são POR EIXO) e cuja UNIDADE de
/// motor não é função do tipo.
pub(crate) const KIND_CUSTOM: u8 = 8;

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
fn limits_label(kind_tag: u8) -> &'static str {
    if kind_tag == KIND_WHEEL {
        tr("panel.inspector.joint.travel")
    } else {
        tr("panel.inspector.joint.limits")
    }
}

/// Does this kind carry a **spring** the artist tunes (stiffness + damping)? A
/// Spring is one; a Wheel's suspension IS one.
///
/// The two share the fields and the ids because they are the same physical
/// thing — what differs is the SCALE they want (a spring hangs a body, a
/// suspension holds a vehicle up), which is why the kind change re-seeds them
/// engine-side (`PhysicsJoint::default_spring`).
///
/// ⚠️ **E um WELD MOLE é a terceira**, o que fez esta pergunta deixar de ser
/// função só do tipo: a mola de uma solda existe quando a chave `Soft` está
/// marcada. É a mesma forma do `breaks_on_torque` de um Wheel — *o estado em que
/// a row pode ser alcançada* é quem manda —, e o flag chega aqui em vez de o
/// painter perguntar `kind == Weld && info.soft` no sítio da pintura, que é a
/// enumeração que apodrece.
const fn kind_has_spring(kind_tag: u8, soft: bool) -> bool {
    kind_tag == KIND_SPRING || kind_tag == KIND_WHEEL || (kind_tag == KIND_WELD && soft)
}

/// **Rigid · Soft** — os dois estados de uma solda, e não um Off/On genérico: o
/// artista escolhe entre duas coisas que uma solda PODE SER, do jeito que o
/// `Solid | Sensor` e o `Discrete | Continuous` da §11 já falam.
const SOFT_LABELS: [TextKey; 2] = [
    TextKey::new("panel.inspector.joint.rigid"),
    TextKey::new("panel.inspector.joint.soft"),
];

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
pub(crate) const fn kind_has_motor(kind_tag: u8) -> bool {
    kind_tag == KIND_PIN
        || kind_tag == KIND_SLIDER
        || kind_tag == KIND_ROPE
        || kind_tag == KIND_WHEEL
        // O Custom sempre tem motor — o que ele escolhe é o EIXO em que ele age.
        || kind_tag == KIND_CUSTOM
}

/// The unit pair the motor rows are labelled with, **for this kind**:
/// `(rate, place)`. Degrees for a hinge, metres for a rail or a winch.
///
/// ⚠️ **Deliberately not `limit_unit`, and the Rope is why:** a Rope has no limit
/// range at all and still has a linear motor, so one function answering both
/// questions would label a winch's target in degrees. Engine-side the same two
/// doors are `limits_in_metres` and `motor_in_metres`.
///
/// ⚠️ **Toma o `info` inteiro e não o tag, desde que o `Custom` chegou:** nos
/// sete presets o grau de liberdade livre É uma propriedade do tipo, mas num
/// Custom ele é ESCOLHIDO — a resposta passou a ser da instância. Engine-side a
/// porta correspondente é `PhysicsJoint::motor_in_metres` (e não a do
/// `JointKind`), e as duas têm de concordar ou o rótulo mente sobre o número que
/// o solver lê.
/// ⭐⭐ **As duas unidades do motor de uma junta — a TAXA e o DESTINO.**
///
/// ⛔⛔ **Elas eram duas `&str` que iam para DENTRO do rótulo** (`"Speed ({unit})"`), e o dono
/// reprovou-o em 2026-09-15 com foto: *«Speed ficou na Label e não na caixa. Para manter padrão
/// universal melhor todos na caixa»*. ⇒ hoje são [`ph2d_editor_core::widget::Unit`], e quem as recebe passa-as ao CAMPO.
///
/// ⚠️ **O sufixo muda de forma ao mudar de sítio, e é a lei da casa:** o rótulo mostrava `°`, o
/// campo mostra `deg` — *o que se mostra num campo é o que o artista tem de conseguir escrever*.
pub(crate) fn motor_units(
    info: &InspectorJointInfo,
) -> (
    ph2d_editor_core::widget::Unit,
    ph2d_editor_core::widget::Unit,
) {
    let angular = if info.kind_tag == KIND_CUSTOM {
        info.motor_axis_tag == AXIS_ROTATION
    } else {
        info.kind_tag == KIND_PIN || info.kind_tag == KIND_WHEEL
    };
    if angular {
        (
            ph2d_editor_core::widget::Unit::DegreesPerSecond,
            ph2d_editor_core::widget::Unit::Degrees,
        )
    } else {
        (
            ph2d_editor_core::widget::Unit::MetersPerSecond,
            ph2d_editor_core::widget::Unit::Meters,
        )
    }
}

/// A tag do eixo de ROTAÇÃO — o único dos três que é angular.
pub(crate) const AXIS_ROTATION: u8 = 2;

/// Velocity · Position — the two things a motor can be told.
const MOTOR_MODE_LABELS: [TextKey; 2] = [
    TextKey::new("panel.inspector.joint.velocity"),
    TextKey::new("panel.inspector.joint.position"),
];
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
const fn limit_unit(kind_tag: u8) -> ph2d_editor_core::widget::Unit {
    if kind_tag == KIND_SLIDER || kind_tag == KIND_WHEEL {
        ph2d_editor_core::widget::Unit::Meters
    } else {
        ph2d_editor_core::widget::Unit::Degrees
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
    let color_id = core_ids::INSP_LIVE_JOINT_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default section accent
    let header = section_header(
        store,
        core_ids::INSP_LIVE_JOINT_SECTION,
        tr("panel.inspector.joint.physics_joint"),
    )
    .color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    // ⚠️ **A DOBRA do corpo** — o escopo recorta a cena E o hit, e escala o `y` de saída, para
    //    que tudo o que está por baixo suba junto. Ver `SectionFold`.
    // ⚠️ **Pergunta o `t`, e NUNCA o `is_collapsed`:** ao clicar para fechar o flag semântico vira
    //    neste mesmo quadro enquanto o `t` ainda desce, então um corpo gateado no flag sumiria de
    //    repente por baixo de um chevron a rodar — as duas metades a discordar outra vez.
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_JOINT_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };

    let mut yy = y + header_h;
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
        tr("panel.inspector.joint.active"),
        ids::INSP_JOINT_ACTIVE_GROUP,
        &ids::INSP_JOINT_ACTIVE,
        &SWITCH_LABELS.map(TextKey::tr),
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
        tr("panel.inspector.joint.kind"),
        ids::INSP_JOINT_KIND_GROUP,
        &ids::INSP_JOINT_KIND,
        &KIND_LABELS.map(TextKey::tr),
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
        // ⚠️ **Num Custom o eixo do motor vem ANTES dos knobs dele**, porque é
        // ele que decide a UNIDADE que os rótulos abaixo mostram: um alvo em
        // metros rotulado em graus é o artista digitando 90 e a peça andando
        // 1,57 m.
        if info.kind_tag == KIND_CUSTOM && info.motor_enabled {
            yy = paint_motor_axis_row(scene, text_system, theme, hit_index, store, x, w, yy, info);
        }
        yy = paint_motor_rows(scene, text_system, theme, hit_index, store, x, w, yy, info);
    }

    // Breaking comes after the parameters and before Delete: it is a property of
    // the joint as a whole (every kind can be pulled apart), not of one kind's
    // degree of freedom, so it is asked of ALL five.
    if kind_can_break(info.kind_tag) {
        yy = paint_break_rows(scene, text_system, theme, hit_index, store, x, w, yy, info);
    }

    // **Copiar / colar as propriedades** (W-JointCopy) — logo acima do Delete
    // porque são verbos sobre o objeto inteiro, como ele, e não afinação de um
    // parâmetro. O Copy é sempre oferecido: a §12 só existe com um joint
    // selecionado, e todo joint tem propriedades a copiar.
    let copy_rect = ph2d_editor_core::property_row::caixa_do_botao(
        text_system,
        x,
        w,
        yy,
        tr("panel.inspector.joint.copy_properties"),
    );
    let copy = Button::new(
        ids::INSP_JOINT_COPY,
        tr("panel.inspector.joint.copy_properties"),
    )
    .kind(ButtonKind::Default)
    .visual(store.button_visual(ids::INSP_JOINT_COPY));
    paint_button(&copy, copy_rect, scene, text_system, theme);
    hit_index.register(ids::INSP_JOINT_COPY, copy_rect);
    yy = ph2d_editor_core::property_row::abaixo_do_botao(copy_rect);
    // ⚠️ **O Paste só existe com algo copiado**, e a contagem entra no RÓTULO
    // quando ele vai tocar mais de um: o fan-out é o que o gesto tem de valioso,
    // e um clique que muda dez objetos tem de dizer isso antes de ser clicado
    // (a lei do `Bake 5.0s to Timeline`). Zero alvos = sem botão — um Paste
    // vazio não muda um pixel, e ler isso como "quebrado" é o que a ausência
    // evita.
    if info.paste_targets > 0 {
        let label = paste_label(info.paste_targets);
        let paste_rect =
            ph2d_editor_core::property_row::caixa_do_botao(text_system, x, w, yy, &label);
        let paste = Button::new(ids::INSP_JOINT_PASTE, label)
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_JOINT_PASTE));
        paint_button(&paste, paste_rect, scene, text_system, theme);
        hit_index.register(ids::INSP_JOINT_PASTE, paste_rect);
        yy = ph2d_editor_core::property_row::abaixo_do_botao(paste_rect);
    }

    let btn_rect = ph2d_editor_core::property_row::caixa_do_botao(
        text_system,
        x,
        w,
        yy,
        tr("panel.inspector.joint.delete_joint"),
    );
    let btn = Button::new(
        ids::INSP_JOINT_REMOVE,
        tr("panel.inspector.joint.delete_joint"),
    )
    .kind(ButtonKind::Default)
    .visual(store.button_visual(ids::INSP_JOINT_REMOVE));
    paint_button(&btn, btn_rect, scene, text_system, theme);
    hit_index.register(ids::INSP_JOINT_REMOVE, btn_rect);
    fold.finish(
        store,
        scene,
        hit_index,
        ph2d_editor_core::property_row::abaixo_do_botao(btn_rect) + SECTION_BOTTOM_PAD_PX,
    )
}

#[cfg(test)]
mod kind_chip_tests {
    use super::{KIND_LABELS, KIND_SLIDER, limit_unit};

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
            crate::ids::INSP_JOINT_KIND.len(),
            "um rotulo sem id e um chip que o seg_row DESCARTA no zip"
        );
    }

    /// O curso diz **metros** para um trilho e **graus** para o resto — a segunda metade da porta
    /// `JointKind::limits_in_metres` (a primeira converte o número; esta o nomeia).
    ///
    /// ⚠️ **Desde 2026-09-15 a unidade é um [`ph2d_editor_core::widget::Unit`] e não uma string de rótulo** (ordem do dono:
    /// *«todos na caixa»*), e ela é pintada DENTRO do campo. ⛔ O sufixo passa de `°` para `deg` de
    /// propósito: num campo mostra-se o que o artista consegue escrever.
    #[test]
    fn a_rails_range_is_named_in_metres() {
        assert_eq!(
            limit_unit(KIND_SLIDER),
            ph2d_editor_core::widget::Unit::Meters
        );
        for other in [0u8, 1, 2, 3] {
            assert_eq!(limit_unit(other), ph2d_editor_core::widget::Unit::Degrees);
        }
    }
}
