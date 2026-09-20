//! As tabelas de **UI dos params** do `motion.wave` — o slider, o tecto digitável e as unidades.
//!
//! ⚠️ Irmão por `#[path]` e **FILHO** do `lib.rs` (o `use super::*` alcança o `MAX_SIDE` e o
//! `CFL_MAX`), como o `lib_tests.rs` deste mesmo nó.
//!
//! ⛔ **O corte é por RESPONSABILIDADE e a catraca de LOC é que o pediu:** ao ganhar a tabela
//! medida do tecto (doc 114 §9), o `lib.rs` passou de `700` para `709` linhas. A lei desta casa é
//! *cortar, nunca subir o número nem escrever uma isenção* — e o sítio do corte não foi inventado:
//! o irmão `motion.soft_body` guarda exactamente estas três tabelas num `params_ui.rs` próprio.

use super::{CFL_MAX, MAX_SIDE};
use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget};

pub(super) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "rows",
        label: "node.motion.wave.param.rows",
        min: 2.0,
        max: 64.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "cols",
        label: "node.motion.wave.param.cols",
        min: 2.0,
        max: 64.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "spacing",
        label: "node.motion.wave.param.spacing",
        min: 0.1,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "speed",
        label: "node.motion.wave.param.speed",
        min: 0.0,
        max: CFL_MAX,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "damping",
        label: "node.motion.wave.param.damping",
        min: 0.0,
        max: 0.3,
        step: 0.005,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **O rótulo diz o que a PAREDE faz, não como ela está implementada.** «Sponge» é o
    // nome da técnica; o artista quer saber se a onda volta ou se some — e é isso que ele lê.
    // ⚠️ **A faixa é a que a MÃO percorre**, e o `0` do default deixa a porta inerte — um
    // artista que ligue o fio e não veja nada tem o knob mesmo ali para o dizer.
    ParamUiHint {
        param: "inject_gain",
        label: "node.motion.wave.param.inject_gain",
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "edges",
        label: "node.motion.wave.param.edges",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Reflect", "Absorb"],
        },
    },
    // ⚠️ **PARA ONDE a altura vai** — um seletor NOMEADO, nunca um slider de passos a decorar
    // (doc 89 folha 06). O vocabulário é o da casa (`motion.drive`), e por isso o artista que
    // aprendeu «Size» num nó não o re-aprende aqui.
    ParamUiHint {
        param: "height_channel",
        label: "node.motion.wave.param.height_channel",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Size", "Y", "Rotation"],
        },
    },
    ParamUiHint {
        param: "center_x",
        label: "node.motion.wave.param.center_x",
        min: -20.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "center_y",
        label: "node.motion.wave.param.center_y",
        min: -20.0,
        max: 20.0,
        step: 0.1,
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
/// O tecto **DIGITÁVEL** de `rows`/`cols` — igual ao clamp, nunca abaixo dele.
///
/// ⚠️ **Sem isto o slider É o tecto**, que era o estado deste nó: o `max` do hint valia `60` e o
/// clamp valia `60`, logo a capacidade do motor acabava onde o dedo acabava. O doc 91 curou essa
/// forma em 25 params, e este ficou de fora.
pub(super) static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "rows",
        max: MAX_SIDE as f32,
    },
    ParamHardMax {
        param: "cols",
        max: MAX_SIDE as f32,
    },
];

pub(super) static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "spacing",
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
