//! ⭐⭐⭐ **O HUD** (TOP-20 #20) — o placar, a vida e o menu que ficam sobre o jogo.
//!
//! Quatro descritores: a RAIZ que se cola à vista ([`ph2d_ecs::UiCanvas`]), o RÓTULO cujo número o
//! jogo muda, o BOTÃO que publica um sinal, e o CONTADOR que dá ao rótulo o que mostrar.
//!
//! ⚠️ **`O::ANY` nos quatro**, pela razão que o relógio e a fábrica já escrevem: quem carrega um
//! HUD é quase sempre um objecto **VAZIO** (a raiz), e restringir a `DRAWABLE` tornaria a coisa
//! mais natural — *um objecto vazio chamado «HUD»* — impossível de montar.
//!
//! ⛔ **Não há descritor para o `CounterRuntime`**, e a ausência é a decisão: o valor vivo não é
//! componente registado, o undo não o fotografa, e descrevê-lo aqui seria prometer ao Inspector um
//! valor que ele não deve mostrar nem editar. É a mesma linha que o `TimerRuntime` tem.
//!
//! ⚠️ **A lista está ORDENADA por `canonical_name`** — há gate, e fora de ordem a busca binária
//! devolve `None` para um tipo que existe.

use crate::{
    ComponentCategory as C, ComponentDesc, ComponentDesc as D, FieldDesc, FieldKind as K,
    Hidden, ObjectKinds as O, Propagation, RefKind,
};

/// **`UiArranjo`** — as relações citam os filhos do canvas pelo `StableId`: `RefKind::Object`, remapeador
/// `ph2d_ecs::hud_arranjo::remap_arranjo_refs` (censo em `ph2d-app-components/src/instance_refs.rs`).
const UI_ARRANJO: &[FieldDesc] = &[FieldDesc {
    field_id: 1,
    label_key: "component.field.ui_arranjo.1",
    kind: K::Ref,
    policy: Propagation::Propagate,
    is_ref: Some(RefKind::Object),
    name: "",
    key: "",
    view: crate::View::Same,
}];

const fn f(field_id: u16, label_key: &'static str, kind: K) -> FieldDesc {
    FieldDesc {
        field_id,
        label_key,
        kind,
        policy: Propagation::Propagate,
        is_ref: None,
        name: "",
        key: "",
        view: crate::View::Same,
    }
}

/// O contador — o número que o jogo soma.
///
/// ⚠️ **`Start` e não «Value»:** o que se grava é com que valor ele COMEÇA (e com que valor
/// renasce ao rebobinar); o valor de agora é vivo.
const COUNTER_FIELDS: &[FieldDesc] = &[
    f(1, "component.field.counter_fields.1", K::Text).named("name"),
    f(2, "component.field.counter_fields.2", K::Int).named("start"),
    f(3, "component.field.counter_fields.3", K::Toggle).named("keep_on_restart"),
];

/// O botão que publica um sinal.
const BUTTON_FIELDS: &[FieldDesc] = &[
    f(1, "component.field.button_fields.1", K::Text).named("signal"),
    f(2, "component.field.button_fields.2", K::Toggle).named("disabled"),
];

/// A raiz do HUD.
///
/// ⚠️ **Não há campo para a POSE**: ela é conduzida a cada quadro (`Driver::CanvasPose`) e o que o
/// artista autorasse ali seria apagado no primeiro quadro com câmera de jogo.
const CANVAS_FIELDS: &[FieldDesc] = &[
    f(1, "component.field.canvas_fields.1", K::Scalar).named_as("reference_width", "ref_w"),
    f(2, "component.field.canvas_fields.2", K::Scalar).named_as("reference_height", "ref_h"),
    f(3, "component.field.canvas_fields.3", K::Enum)
        .named("fit")
        .viewed(crate::options![
            "keep" => Keep @ "panel.inspector.hud.fit_keep",
            "stretch" => Stretch @ "panel.inspector.hud.fit_stretch",
            "expand" => Expand @ "panel.inspector.hud.fit_expand",
        ]),
];

/// O rótulo cujo número o jogo muda.
///
/// ⚠️ **`Source Name` é UM campo para três fontes** (o contador, o relógio ou a etiqueta): qual
/// delas é o `Source` que diz. Três campos — um por fonte — deixariam dois sempre mortos.
const LABEL_FIELDS: &[FieldDesc] = &[
    // ⚠️ `source` é o enum INTEIRO (`"Authored"` ou `{"Counter": "<nome>"}`); o «Source Name» é o
    // conteúdo da variante em que ele estiver (`source:*`) — e trocar de fonte CONSERVA-o (F27 E).
    f(1, "component.field.label_fields.1", K::Enum)
        .named("source")
        .viewed(crate::options![
            "authored" => Authored @ "panel.inspector.hud.source_authored",
            "counter" => {"Counter": ""} @ "panel.inspector.hud.source_counter",
            "timer" => {"TimerLeft": ""} @ "panel.inspector.hud.source_timer",
            "tag" => {"TagCount": ""} @ "panel.inspector.hud.source_tag",
        ]),
    f(2, "component.field.label_fields.2", K::Text).named_as("source_name", "source:*"),
    f(3, "component.field.label_fields.3", K::Text).named("prefix"),
    f(4, "component.field.label_fields.4", K::Text).named("suffix"),
];

/// Os quatro, por ordem de `canonical_name`.
pub const DESCS: &[ComponentDesc] = &[
    D::authored(
        "ph2d::ecs::Counter",
        "component.counter.name",
        C::Logic,
        O::ANY,
        COUNTER_FIELDS,
    )
    .coded("counter"),
    // O arranjo (ADR-0194): nasce e muda só pela ferramenta do HUD na barra esquerda (setas na tela) — `Intrinsic`
    // sem cartão, como o `InstanceOf`, porque CITA os filhos pelo `StableId` e a cópia tem de os remapear.
    D::intrinsic(
        "ph2d::ecs::UiArranjo",
        "component.ui_arranjo.name",
        C::Logic,
        UI_ARRANJO,
    )
    .hidden(Hidden::OtherEditor),
    D::authored(
        "ph2d::ecs::UiButton",
        "component.ui_button.name",
        C::Logic,
        O::ANY,
        BUTTON_FIELDS,
    )
    .coded("button"),
    D::authored(
        "ph2d::ecs::UiCanvas",
        "component.ui_canvas.name",
        C::Logic,
        O::ANY,
        CANVAS_FIELDS,
    )
    .coded("hud_canvas"),
    // ⚠️ **O rótulo PEDE o canvas?** Não: um `UiLabel` numa forma solta continua a derivar o texto
    // dela; o que ele perde sem uma raiz é ficar colado à vista. *Exigir o pai tornaria
    // inexprimível um número que vive no MUNDO* — a barra de vida sobre a cabeça de um inimigo.
    D::authored(
        "ph2d::ecs::UiLabel",
        "component.ui_label.name",
        C::Logic,
        O::ANY,
        LABEL_FIELDS,
    )
    .coded("label"),
];
