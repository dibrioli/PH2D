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
    ObjectKinds as O, Propagation,
};

const fn f(field_id: u16, label_key: &'static str, kind: K) -> FieldDesc {
    FieldDesc {
        field_id,
        label_key,
        kind,
        policy: Propagation::Propagate,
        is_ref: None,
    }
}

/// O contador — o número que o jogo soma.
///
/// ⚠️ **`Start` e não «Value»:** o que se grava é com que valor ele COMEÇA (e com que valor
/// renasce ao rebobinar); o valor de agora é vivo.
const COUNTER_FIELDS: &[FieldDesc] = &[
    f(1, "component.field.counter_fields.1", K::Text),
    f(2, "component.field.counter_fields.2", K::Int),
];

/// O botão que publica um sinal.
const BUTTON_FIELDS: &[FieldDesc] = &[
    f(1, "component.field.button_fields.1", K::Text),
    f(2, "component.field.button_fields.2", K::Toggle),
];

/// A raiz do HUD.
///
/// ⚠️ **Não há campo para a POSE**: ela é conduzida a cada quadro (`Driver::CanvasPose`) e o que o
/// artista autorasse ali seria apagado no primeiro quadro com câmera de jogo.
const CANVAS_FIELDS: &[FieldDesc] = &[
    f(1, "component.field.canvas_fields.1", K::Scalar),
    f(2, "component.field.canvas_fields.2", K::Scalar),
    f(3, "component.field.canvas_fields.3", K::Enum),
];

/// O rótulo cujo número o jogo muda.
///
/// ⚠️ **`Source Name` é UM campo para três fontes** (o contador, o relógio ou a etiqueta): qual
/// delas é o `Source` que diz. Três campos — um por fonte — deixariam dois sempre mortos.
const LABEL_FIELDS: &[FieldDesc] = &[
    f(1, "component.field.label_fields.1", K::Enum),
    f(2, "component.field.label_fields.2", K::Text),
    f(3, "component.field.label_fields.3", K::Text),
    f(4, "component.field.label_fields.4", K::Text),
];

/// Os quatro, por ordem de `canonical_name`.
pub const DESCS: &[ComponentDesc] = &[
    D::authored(
        "ph2d::ecs::Counter",
        "component.counter.name",
        C::Logic,
        O::ANY,
        COUNTER_FIELDS,
    ),
    D::authored(
        "ph2d::ecs::UiButton",
        "component.ui_button.name",
        C::Logic,
        O::ANY,
        BUTTON_FIELDS,
    ),
    D::authored(
        "ph2d::ecs::UiCanvas",
        "component.ui_canvas.name",
        C::Logic,
        O::ANY,
        CANVAS_FIELDS,
    ),
    // ⚠️ **O rótulo PEDE o canvas?** Não: um `UiLabel` numa forma solta continua a derivar o texto
    // dela; o que ele perde sem uma raiz é ficar colado à vista. *Exigir o pai tornaria
    // inexprimível um número que vive no MUNDO* — a barra de vida sobre a cabeça de um inimigo.
    D::authored(
        "ph2d::ecs::UiLabel",
        "component.ui_label.name",
        C::Logic,
        O::ANY,
        LABEL_FIELDS,
    ),
];
