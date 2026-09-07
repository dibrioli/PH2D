//! **OS DOIS NÚMEROS DO EXTRACT** — irmão (`#[path]`) do [`super::rows`],
//! cortado por ASSUNTO.
//!
//! ⚠️ **Eles são os ARGUMENTOS de um BOTÃO, e não knobs do pincel** — é por isso
//! que vivem em `Place::AfterExtract`, colados ao gesto que os lê. É a mesma
//! decisão que trouxe a pista de `Alpha Scale` para a cauda: um controlo e o que
//! ele governa têm de estar no campo de visão um do outro.
//!
//! ⚠️ **O gate de LOC foi o gatilho, não a razão.** O `rows.rs` cruzou os 600 do
//! `architecture_panel_loc_cap` na wave dos knobs do tecido, e o que saiu foram
//! as duas metades com fronteira própria — os cinco do tecido, num irmão, e
//! estes dois, que nem sequer são do pincel.

use ph2d_editor_core::ids;

use super::{MAX_EXTRACT_SMOOTH, Place, Row};
use crate::state::UiLevel;

pub(super) const EXTRACT_THICKNESS: Row = Row {
    label: "panel.sculpt3d.extract_thickness",
    slider: ids::SCULPT3D_EXTRACT_THICK,
    chip: ids::SCULPT3D_EXTRACT_THICK_NUM,
    // ⚠️ **A faixa é sobre a ESCALA LOCAL da malha, e as primitivas desta
    // casa nascem com raio 1** — meia unidade é meia peça, e é a faixa
    // confortável do arrasto. O sinal escolhe o lado: para fora é armadura,
    // para dentro é forro. **Zero é uma folha só**, e é ele que está no meio
    // da pista de propósito.
    min: -0.5,
    max: 0.5,
    step: 0.01, // LITERAL-PX-OK: passo de uma espessura em unidades de malha
    decimals: 3,
    get: |u| u.extract.thickness,
    set: |u, v| u.extract.thickness = v,
    show: |_| true,
    level: UiLevel::Basic,
    place: Place::AfterExtract,
};

pub(super) const EXTRACT_SMOOTH: Row = Row {
    label: "panel.sculpt3d.extract_smooth",
    slider: ids::SCULPT3D_EXTRACT_SMOOTH,
    chip: ids::SCULPT3D_EXTRACT_SMOOTH_NUM,
    min: 0.0,
    max: MAX_EXTRACT_SMOOTH,
    step: 1.0, // LITERAL-PX-OK: uma passada e' inteira
    decimals: 0,
    get: |u| u.extract.smooth as f32,
    // ⚠️ O `round` é a fronteira de DISPLAY: a pista fala em `f32` como toda
    // row desta tabela, e o que o kernel conta é uma passada inteira.
    set: |u, v| u.extract.smooth = v.round().max(0.0) as u32,
    show: |_| true,
    level: UiLevel::Basic,
    place: Place::AfterExtract,
};
