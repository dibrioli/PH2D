//! **OS CINCO NÚMEROS DO PINCEL DE TECIDO** — irmão (`#[path]`) do
//! [`super::rows`], cortado por ASSUNTO.
//!
//! ⚠️⚠️ **Eles existiam na lei e não existiam no painel até 07/09.** A tradução
//! `Brush → Pincel` do `ph2d-sculpt3d` escrevia os cinco como omissão literal, e
//! o corpus do oráculo tem fixture para cada um (`massa2`, `amort05`, `amort1`,
//! `plast05`, `pino`, e o `preset` com limite `5`). *Uma lei medida na bancada e
//! não ligada no produto é uma lei que o artista não tem* — a mesma conta que os
//! oito modos e as três áreas pagaram em 06/09.
//!
//! ⚠️ **As faixas e as omissões são as do alvo** (espec §8.1), e cada piso diz de
//! que recurso ele é: a massa é um ganho INVERSO, logo `0` seria uma divisão por
//! zero com nome de knob; o amortecimento abre em `0,01`, que é a omissão, e é a
//! retenção de velocidade que faz o traço ASSENTAR.
//!
//! ⚠️ **O gate de LOC foi o gatilho, não a razão.** O `rows.rs` cruzou os 600 do
//! `architecture_panel_loc_cap` ao ganhar estas cinco, e o que saiu foi a metade
//! com fronteira própria — não a última coisa que alguém escreveu. ⚠️ E ele fica
//! **latente**: aquele gate mora em `ph2d-editor-core/tests/`, então um
//! fechamento por `cargo test -p ph2d-panel-sculpt3d` nunca o alcança.

use ph2d_sculpt3d::Verb;

use super::{Place, Row};
use crate::state::{Sculpt3dUi, UiLevel};

/// **Esta row é do pincel de TECIDO?** — a pergunta é ao VERBO, e é a mesma que
/// o roteador faz para honrar o clique. ⛔ Uma lista paralela de nomes seria um
/// knob que aparece noutra ferramenta e não move um vértice.
fn is_cloth(u: &Sculpt3dUi) -> bool {
    u.brush.verb == Verb::Cloth
}

pub(super) const CLOTH_LIMIT: Row = Row {
    label: "panel.sculpt3d.cloth_limit",
    slider: crate::ids::SCULPT3D_CLOTH_LIMIT,
    chip: crate::ids::SCULPT3D_CLOTH_LIMIT_NUM,
    // A faixa é a do alvo (espec §8.1). O recurso que ela nomeia é TEMPO ×
    // ALCANCE: o limite é `R·(1+L)`, logo `10` simula uma esfera de `11·R`.
    min: 0.1,  // LITERAL-PX-OK: piso da faixa do alvo, em raios de pincel
    max: 10.0, // LITERAL-PX-OK: teto da faixa do alvo, em raios de pincel
    step: 0.1, // LITERAL-PX-OK: passo de um knob em raios de pincel
    decimals: 2,
    get: |u| u.brush.cloth_limit,
    set: |u, v| u.brush.cloth_limit = v,
    show: is_cloth,
    level: UiLevel::Basic,
    place: Place::Knobs,
};

pub(super) const CLOTH_FALLOFF: Row = Row {
    label: "panel.sculpt3d.cloth_falloff",
    slider: crate::ids::SCULPT3D_CLOTH_FALLOFF,
    chip: crate::ids::SCULPT3D_CLOTH_FALLOFF_NUM,
    min: 0.0,
    max: 1.0,
    step: 0.05, // LITERAL-PX-OK: knob adimensional
    decimals: 2,
    get: |u| u.brush.cloth_falloff,
    set: |u, v| u.brush.cloth_falloff = v,
    show: is_cloth,
    level: UiLevel::Basic,
    place: Place::Knobs,
};

pub(super) const CLOTH_MASS: Row = Row {
    label: "panel.sculpt3d.cloth_mass",
    slider: crate::ids::SCULPT3D_CLOTH_MASS,
    chip: crate::ids::SCULPT3D_CLOTH_MASS_NUM,
    // ⚠️ **O piso não é zero e não é escolha:** a massa é um ganho INVERSO
    // (espec §5.4), logo `0` é uma divisão por zero com o nome de knob.
    min: 0.01, // LITERAL-PX-OK: piso de um ganho INVERSO, nao metrica de design
    max: 2.0,
    step: 0.05, // LITERAL-PX-OK: knob adimensional
    decimals: 2,
    get: |u| u.brush.cloth_mass,
    set: |u, v| u.brush.cloth_mass = v,
    show: is_cloth,
    level: UiLevel::Basic,
    place: Place::Knobs,
};

pub(super) const CLOTH_DAMPING: Row = Row {
    label: "panel.sculpt3d.cloth_damping",
    slider: crate::ids::SCULPT3D_CLOTH_DAMPING,
    chip: crate::ids::SCULPT3D_CLOTH_DAMPING_NUM,
    // ⚠️ **O piso é `0,01`, que é a omissão** — a faixa do alvo abre onde ele
    // a põe, e é medido que a retenção de velocidade é o que faz o traço
    // ASSENTAR: a `1` o pano pára no instante em que a mão pára.
    min: 0.01, // LITERAL-PX-OK: piso da faixa do alvo, nao metrica de design
    max: 1.0,
    step: 0.01, // LITERAL-PX-OK: knob adimensional
    decimals: 2,
    get: |u| u.brush.cloth_damping,
    set: |u, v| u.brush.cloth_damping = v,
    show: is_cloth,
    level: UiLevel::Basic,
    place: Place::Knobs,
};

pub(super) const CLOTH_PLASTICITY: Row = Row {
    label: "panel.sculpt3d.cloth_plasticity",
    slider: crate::ids::SCULPT3D_CLOTH_PLASTICITY,
    chip: crate::ids::SCULPT3D_CLOTH_PLASTICITY_NUM,
    min: 0.0,
    max: 1.0,
    step: 0.05, // LITERAL-PX-OK: knob adimensional
    decimals: 2,
    get: |u| u.brush.cloth_plasticity,
    set: |u, v| u.brush.cloth_plasticity = v,
    show: is_cloth,
    level: UiLevel::Basic,
    place: Place::Knobs,
};
