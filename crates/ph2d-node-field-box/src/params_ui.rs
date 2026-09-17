//! **AS TABELAS DE PAINEL do `field.box`** — tectos digitáveis, secções, hints e unidades.
//!
//! Separado do [`super`] pelo tecto de LOC (HR-18, 700 para `crates/`), no corte que os nós
//! grandes desta casa já usam (`force.wind/params_ui.rs`, `sim.collide/ui.rs`): o pai responde
//! *o que o campo calcula*, aqui fica *como o artista o disca*.
//!
//! ⚠️ **O ficheiro passou o tecto na wave do ciclo 4** — a que lhe deu as secções `Placement` e
//! `Falloff` para ele falar a mesma língua do irmão `field.radial_sweep` — e o vermelho
//! sobreviveu a dois fechos porque o gate que o mede vive em `ph2d-editor-core/tests/` e varre a
//! ÁRVORE: nem `--bins` nem um `clippy -p` lhe tocam.

use super::STRENGTH;
use ph2d_node_registry::{
    ParamGroup, ParamHardMax, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget,
};

/// O teto que a MÁQUINA (ou o bom senso) impõe, alcançável por DIGITAÇÃO — o slider fica
/// onde a MÃO trabalha (soft/hard do Blender; doc 88 §11). O curso de antes é este número:
/// nada ficou inalcançável, só deixou de ser o que o dedo percorre.
pub(super) static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "soft",
        max: 20.0,
    },
    // ⚠️ **O NEUTRO DESTE NÓ ERA INALCANÇÁVEL POR GESTO NENHUM.** O doc-comment do módulo
    // promete *"a box larger than the scene with `soft = 0`"* e o teste dele usa `width = 100`
    // — mas sem entrada aqui o digitado para no fim do ARRASTO (`ui.rs:206`: *"a param with no
    // entry here types to its soft `max`"*), que são **40**. Uma promessa que a UI não executa.
    //
    // **De que recurso é este teto: da PRECISÃO** (`CLAUDE.md` §0.0). Nada nesta lei satura —
    // uma caixa maior que a cena É o neutro, e o nó honra-a. O que acaba é o `f32`: acima de
    // `2²¹` somar o `step` do slider (0,1) já **não move o número**, então dois valores
    // autoráveis vizinhos passam a ser o mesmo campo, e aceitá-los seria aceitar e mentir (a
    // lei do `sim.spawn::burst`, doc 88 §B2). O número é `2²¹ − 1 ulp`, MEDIDO e re-derivado a
    // cada corrida pelo gate `every_precision_bound_param_types_to_the_measured_ceiling` —
    // escrito à mão ele envelheceria no dia em que alguém afinasse o `step` do arrasto.
    ParamHardMax {
        param: "width",
        max: 2_097_152.0 - 0.125,
    },
    ParamHardMax {
        param: "height",
        max: 2_097_152.0 - 0.125,
    },
];

/// Param UI hints (M1.P1): full Width/Height/Softness in world-units, signed
/// centre, a named Curve selector, an Invert checkbox.
/// **AS SEÇÕES DESTE NÓ** (ciclo 4, W2 — [doc 107](../../../docs/Motion%20Nodes/107_ciclo_4_foco_os_campos.md)).
///
/// ⛔ **Ele era o IRMÃO desalinhado:** o `field.radial_sweep` é o mesmo tipo de campo, com o
/// mesmo vocabulário (`center_*`, `rotation`, `soft`, `curve`, `invert`), e agrupa-o em
/// **Placement** e **Falloff** — este pintava as nove rows em fila. *Um artista que aprendeu um
/// tem de reconhecer o outro*, e com o painel lateral fora o cartão é a única superfície onde
/// estes nomes aparecem.
///
/// ⚠️ **A lei é a que os dois irmãos já escreveram, palavra por palavra:** *param sem grupo
/// pinta antes de toda secção, e é ali que os essenciais devem estar — a razão de existir do nó,
/// e pô-la numa secção seria escondê-la atrás de um clique*. A razão de existir deste nó é uma
/// caixa ⇒ `width` e `height` ficam **soltos**.
///
/// ⚠️⚠️ **A primeira redacção desta tabela deixava o `soft` solto também**, com a razão *«uma
/// caixa que mascara com borda macia»* — e o **irmão põe-no em `Falloff`**. O portão
/// `the_two_spatial_boxes_group_a_shared_param_the_same_way` apanhou-o antes de a wave fechar:
/// *quando o objectivo é alinhar dois irmãos, a autoridade é o irmão, não a minha leitura do
/// que é essencial.*
///
/// ⚠️ **O preço é MEDIDO e é de espaço, não de relógio:** uma secção aberta custa **+1 fileira**
/// (`band_len = params + sections`), então o cartão passa de `9` para `11` — ainda abaixo das
/// `12` que o irmão já shipa. ⛔ **Por isso os cartões pequenos do grupo ficam em fila:** numa
/// carta de 4 ou 6 rows dois cabeçalhos organizam menos do que ocupam.
pub(super) static PARAM_GROUPS: &[ParamGroup] = &[
    // Onde a caixa está plantada, e para onde ela aponta.
    ParamGroup::new("center_x", "Placement"),
    ParamGroup::new("center_y", "Placement"),
    ParamGroup::new("rotation", "Placement"),
    // Como a borda desvanece, e com que força ela pesa.
    ParamGroup::new("soft", "Falloff"),
    ParamGroup::new("curve", "Falloff"),
    ParamGroup::new("invert", "Falloff"),
    ParamGroup::new("strength", "Falloff"),
];

pub(super) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "width",
        label: "node.field.box.param.width",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "height",
        label: "node.field.box.param.height",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "soft",
        label: "node.field.box.param.soft",
        min: 0.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "center_x",
        label: "node.field.box.param.center_x",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "center_y",
        label: "node.field.box.param.center_y",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "rotation",
        label: "node.field.box.param.rotation",
        min: -180.0,
        max: 180.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "curve",
        label: "node.field.box.param.curve",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Linear", "Quad", "Smooth", "Smoother"],
        },
    },
    ParamUiHint {
        param: "invert",
        label: "node.field.box.param.invert",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    // ⚠️ A faixa é o CURSO ÚTIL e não um teto — ver [`STRENGTH`].
    ParamUiHint {
        param: STRENGTH,
        label: "node.field.box.param.strength",
        min: -1.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
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
        param: "width",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "height",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "soft",
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
