//! **O QUE O PAINEL OFERECE** deste nó — as quatro tabelas de declaração de UI
//! (`ParamUiHint` · `ParamHardMax` · `ParamGroup` · `ParamUnitDecl`).
//!
//! ⚠️ Elas vivem à parte da LEI (o manifesto, a matemática do sector e o kernel de GPU, no
//! [`super`]) pela razão de sempre nesta casa: são responsabilidades diferentes, e o molde já
//! existe nos irmãos (`ph2d-node-force-wind/src/params_ui.rs`,
//! `ph2d-node-motion-trail/src/ui.rs`). ⛔ O corte foi **forçado por um tecto de LOC** quando a
//! migração do HR-15 trocou as opções por chaves — que são mais longas que as palavras — e o
//! ficheiro passou de `698` para `703` linhas; a cura de um tecto é o corte, nunca uma entrada
//! nova de dívida.

use super::*;

/// **O teto DIGITÁVEL dos dois raios, MEDIDO** — o irmão exacto do `field.box`.
///
/// ⚠️ **O neutro que o doc-comment deste módulo promete era inalcançável:** ele diz
/// *"`radius` larger than the scene"* ⇒ máscara `1` em toda a parte, e sem entrada aqui o
/// digitado parava em **40** (o fim do arrasto — `ui.rs:206`).
///
/// **O recurso é a PRECISÃO** (`CLAUDE.md` §0.0): nada nesta lei satura — um raio maior que a
/// cena É o neutro, e o nó honra-o. O que acaba é o `f32`: acima de `2²¹` somar o `step` do
/// slider (0,1) **não move o número**, então dois raios autoráveis vizinhos são o mesmo campo.
/// O valor é `2²¹ − 1 ulp`, derivado a cada corrida pelo gate
/// `every_precision_bound_param_types_to_the_measured_ceiling` (`ph2d-node-registry-init`).
///
/// ⚠️ **Os dois raios levam o MESMO teto, pela mesma razão que já governa a faixa do arrasto:**
/// o anel só existe enquanto `inner < radius`, e um teto menor no interno esconderia metade dos
/// anéis que o externo alcança.
pub(super) static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "radius",
        max: 2_097_152.0 - 0.125,
    },
    ParamHardMax {
        param: INNER_RADIUS,
        max: 2_097_152.0 - 0.125,
    },
];

/// As SEÇÕES deste nó (doc 88 B3).
///
/// ⚠️ A VARREDURA fica solta inteira (`radius`, `start_angle`, `end_angle`, `repetitions`):
/// ela é a razão de existir do nó, e sepultá-la atrás de um clique é o erro que o gate do
/// `field.remap` já pegou uma vez nesta linha.
///
/// ⚠️ O `curve` aqui é o SELETOR de contorno do falloff (um `Enum`), não um editor de curva —
/// por isso ele agrupa com `soft`/`invert` em vez de ficar solto como o do `field.remap`.
pub(super) static PARAM_GROUPS: &[ParamGroup] = &[
    // Onde o radar está plantado, e para onde ele aponta.
    ParamGroup::new("center_x", "node.group.placement"),
    ParamGroup::new("center_y", "node.group.placement"),
    ParamGroup::new("inner_radius", "node.group.placement"),
    ParamGroup::new("rotation", "node.group.placement"),
    // Como a borda do feixe desvanece.
    ParamGroup::new("soft", "node.group.falloff"),
    ParamGroup::new("curve", "node.group.falloff"),
    ParamGroup::new("invert", "node.group.falloff"),
];

/// Param UI hints (M1.P1): the sweep's radius (gizmo-driven), the angular sector
/// (start/end in degrees), the repetition count, a normalized softness, a signed centre,
/// a named Curve selector, an Invert checkbox. `soft` is a FRACTION `[0,1]` of the extent
/// (dimensionless — it softens both the angular and the radial edge; see the module doc).
pub(super) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "radius",
        label: "node.field.radial_sweep.param.radius",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **A mesma faixa do `radius`, de propósito**: o anel só existe enquanto
    // `inner < radius`, e um teto menor esconderia metade dos anéis que o raio externo
    // alcança. Acima do externo o campo fica vazio — que é uma resposta, não um erro.
    ParamUiHint {
        param: INNER_RADIUS,
        label: "node.field.radial_sweep.param.inner_radius",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "start_angle",
        label: "node.field.radial_sweep.param.start_angle",
        min: -360.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "end_angle",
        label: "node.field.radial_sweep.param.end_angle",
        min: -360.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "repetitions",
        label: "node.field.radial_sweep.param.repetitions",
        min: 1.0,
        max: 32.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "soft",
        label: "node.field.radial_sweep.param.soft",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: SOFT_ANGULAR,
        // O rótulo diz que ele é RELATIVO — «Angular Softness» prometeria uma
        // maciez própria, e o modelo entrega um viés sobre a de cima.
        label: "node.field.radial_sweep.param.soft_angular",
        min: 0.0,
        // O curso vai a 2 porque o interessante é a razão entre as duas bordas, e
        // acima do dobro a angular já está saturada na sua própria extensão.
        max: 2.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "center_x",
        label: "node.field.radial_sweep.param.center_x",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "center_y",
        label: "node.field.radial_sweep.param.center_y",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "rotation",
        label: "node.field.radial_sweep.param.rotation",
        min: -180.0,
        max: 180.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "curve",
        label: "node.field.radial_sweep.param.curve",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.field.radial_sweep.param.curve.0",
                "node.field.radial_sweep.param.curve.1",
                "node.field.radial_sweep.param.curve.2",
                "node.field.radial_sweep.param.curve.3",
            ],
        },
    },
    ParamUiHint {
        param: "invert",
        label: "node.field.radial_sweep.param.invert",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

/// **What each of this node's numbers IS** (doc 88, Wave A) — never how it is
/// shown. A `Length` is stored in world METRES and the panel resolves the face
/// the artist reads (`px` or `m`) from `ProjectSettings::display_unit`; a node
/// that could pin one would be overriding a setting it does not own.
///
/// Only params whose value is a world COORDINATE or a world DISTANCE are declared
/// here. A weight, a fraction, a rate and a count are left bare on purpose: a unit
/// that is wrong is worse than a unit that is missing, because the artist can read
/// a bare number but a mislabelled one teaches them something false.
pub(super) static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "radius",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "center_x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "center_y",
        unit: ParamUnit::Length,
    },
];
