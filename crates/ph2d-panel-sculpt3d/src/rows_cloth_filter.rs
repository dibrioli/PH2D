//! ⭐⭐⭐ **AS PROPRIEDADES DO FILTRO DE TECIDO, no painel** — e a *Quality* do
//! pincel, que é a mesma ideia do outro lado.
//!
//! Irmão do [`super::rows_cloth`], e o corte é o SUJEITO: lá os números do
//! **pincel**, aqui os do **filtro**. Eles não são os mesmos, e tratá-los como
//! se fossem foi o defeito que a pergunta do dono expôs (2026-09-08).
//!
//! # ⛔⛔ A pergunta de visibilidade é OUTRA
//!
//! As fileiras do pincel perguntam *«o pincel de tecido está na mão?»*
//! (`verb == Verb::Cloth`). Estas perguntam *«a lei escolhida é de tecido?»*
//! (`filter_law.is_cloth()`), que é a **mesma** pergunta que a fileira de
//! orientação já faz três linhas acima no pintor.
//!
//! ⚠️ **E a diferença não é estética: é a razão do defeito.** Desde a W9b o
//! filtro corre com **qualquer** verbo na mão — a lei deixou de ser derivada do
//! verbo —, então com o Draw na mão os três números do tecido mexiam na
//! simulação **sem nada na tela os mostrar**. *Vivo e inalcançável é o espelho
//! do knob morto, e nenhuma sonda deste repo vê a segunda espécie.*

use ph2d_editor_core::ids;

use super::types::{Place, Row};
use crate::state::{Sculpt3dUi, UiLevel};
use ph2d_sculpt3d::{ClothFilterProps, Verb};

/// A lei escolhida é de tecido? — a MESMA pergunta da fileira de orientação.
fn is_cloth_filter(u: &Sculpt3dUi) -> bool {
    u.filter_law.is_cloth()
}

/// O pincel de tecido está na mão? — para a *Quality* dele.
fn is_cloth_brush(u: &Sculpt3dUi) -> bool {
    u.brush.verb == Verb::Cloth
}

/// ⚠️ **Um passo de `1` e zero decimais**: são varreduras, e meia varredura não
/// existe. O `set` arredonda na porta — um `u32` escrito a partir de um `f32`
/// truncado daria `31` onde o artista largou o chip em `32`.
const PASSO_INTEIRO: f64 = 1.0; // LITERAL-PX-OK: uma varredura, não uma métrica de layout

pub(super) const CLOTH_SWEEPS: Row = Row {
    label: "panel.sculpt3d.cloth_sweeps",
    slider: ids::SCULPT3D_CLOTH_SWEEPS,
    chip: ids::SCULPT3D_CLOTH_SWEEPS_NUM,
    min: ClothFilterProps::SWEEPS.0 as f32,
    max: ClothFilterProps::SWEEPS.1 as f32,
    step: PASSO_INTEIRO,
    decimals: 0,
    get: |u| u.brush.cloth_sweeps as f32,
    set: |u, v| u.brush.cloth_sweeps = v.round().max(0.0) as u32,
    show: is_cloth_brush,
    // ⚠️ **Pro**: é o knob que troca qualidade por tempo, e a omissão é a do
    // alvo — quem não o procura nunca precisa de o ver.
    level: UiLevel::Pro,
    place: Place::Knobs,
};

pub(super) const CFILTER_MASS: Row = Row {
    label: "panel.sculpt3d.cfilter_mass",
    slider: ids::SCULPT3D_CFILTER_MASS,
    chip: ids::SCULPT3D_CFILTER_MASS_NUM,
    min: ClothFilterProps::MASS.0,
    max: ClothFilterProps::MASS.1,
    step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional
    decimals: 2,
    get: |u| u.cloth_filter.mass,
    set: |u, v| u.cloth_filter.mass = v,
    show: is_cloth_filter,
    level: UiLevel::Pro,
    place: Place::Knobs,
};

pub(super) const CFILTER_DAMPING: Row = Row {
    label: "panel.sculpt3d.cfilter_damping",
    slider: ids::SCULPT3D_CFILTER_DAMPING,
    chip: ids::SCULPT3D_CFILTER_DAMPING_NUM,
    // ⚠️⚠️ **O piso é `0`, e é a metade do defeito que se vê num número:** a
    // faixa do PINCEL começa em `0,01`, e enquanto o filtro lia o campo dele o
    // valor de omissão do próprio filtro era inalcançável.
    min: ClothFilterProps::DAMPING.0,
    max: ClothFilterProps::DAMPING.1,
    step: 0.01, // LITERAL-PX-OK: passo de uma fracção
    decimals: 2,
    get: |u| u.cloth_filter.damping,
    set: |u, v| u.cloth_filter.damping = v,
    show: is_cloth_filter,
    level: UiLevel::Pro,
    place: Place::Knobs,
};

pub(super) const CFILTER_PLASTICITY: Row = Row {
    label: "panel.sculpt3d.cfilter_plasticity",
    slider: ids::SCULPT3D_CFILTER_PLASTICITY,
    chip: ids::SCULPT3D_CFILTER_PLASTICITY_NUM,
    min: ClothFilterProps::PLASTICITY.0,
    max: ClothFilterProps::PLASTICITY.1,
    step: 0.05, // LITERAL-PX-OK: passo de uma fracção
    decimals: 2,
    get: |u| u.cloth_filter.plasticity,
    set: |u, v| u.cloth_filter.plasticity = v,
    show: is_cloth_filter,
    level: UiLevel::Pro,
    place: Place::Knobs,
};

pub(super) const CFILTER_SWEEPS: Row = Row {
    label: "panel.sculpt3d.cfilter_sweeps",
    slider: ids::SCULPT3D_CFILTER_SWEEPS,
    chip: ids::SCULPT3D_CFILTER_SWEEPS_NUM,
    min: ClothFilterProps::SWEEPS.0 as f32,
    max: ClothFilterProps::SWEEPS.1 as f32,
    step: PASSO_INTEIRO,
    decimals: 0,
    get: |u| u.cloth_filter.sweeps as f32,
    set: |u, v| u.cloth_filter.sweeps = v.round().max(0.0) as u32,
    show: is_cloth_filter,
    level: UiLevel::Pro,
    place: Place::Knobs,
};

/// ⭐⭐⭐ **O CONTROLO QUE O ALVO NÃO TEM, e o report de 08/09 pediu** — *«o Cloth
/// não age como pano real, mas como um elástico que estica indefinidamente»*.
///
/// ⚠️ **A faixa acaba em `2,00` e não tem «desligado»:** o comportamento sem
/// tecto É o defeito reportado, e pô-lo no topo de uma faixa larga deixaria o
/// intervalo útil espremido nos primeiros por cento do cursor.
pub(super) const CFILTER_STRETCH: Row = Row {
    label: "panel.sculpt3d.cfilter_stretch",
    slider: ids::SCULPT3D_CFILTER_STRETCH,
    chip: ids::SCULPT3D_CFILTER_STRETCH_NUM,
    min: ClothFilterProps::STRETCH.0,
    max: ClothFilterProps::STRETCH.1,
    step: 0.05, // LITERAL-PX-OK: passo de uma razão de comprimento
    decimals: 2,
    get: |u| u.cloth_filter.stretch_max,
    set: |u, v| u.cloth_filter.stretch_max = v,
    show: is_cloth_filter,
    // ⚠️ **Básico, e é o único dos números do filtro que o é.** Os outros afinam
    // um comportamento certo; este decide se o pano é um pano.
    level: UiLevel::Basic,
    place: Place::Knobs,
};

/// ⭐⭐⭐ ***Preserve Volume*** — *«deve haver a possibilidade de manter volume»*.
///
/// ⚠️⚠️ **Nasce em `0`, e não por ser cara:** ela **cancela a Escala e o Inflate
/// por construção** (os dois existem para mudar o volume), e há gate a afirmá-lo.
/// O que ela transforma é a Gravidade e o **Aperto** — sem ela o aperto implode a
/// peça a `3 %` do volume e deixa `1 151` vincos; com ela, `95 %` e `119`.
pub(super) const CFILTER_VOLUME: Row = Row {
    label: "panel.sculpt3d.cfilter_volume",
    slider: ids::SCULPT3D_CFILTER_VOLUME,
    chip: ids::SCULPT3D_CFILTER_VOLUME_NUM,
    min: ClothFilterProps::VOLUME.0,
    max: ClothFilterProps::VOLUME.1,
    step: 0.05, // LITERAL-PX-OK: passo de uma fracção
    decimals: 2,
    get: |u| u.cloth_filter.volume,
    set: |u, v| u.cloth_filter.volume = v,
    show: is_cloth_filter,
    level: UiLevel::Basic,
    place: Place::Knobs,
};

/// ⭐⭐⭐ ***Bend Stiffness*** — o TAMANHO da ruga, e o report de 09/09.
///
/// ⚠️⚠️ **O tecto dela é a MALHA, e está medido:** o comprimento de onda de uma
/// prega é `~7` a `10` arestas *faça o artista o que fizer*, e este número
/// compra `+33 %` sobre isso. Quem quer couro reduz a malha primeiro — o botão
/// de retopologia existe. ⛔ Decoupar a onda da malha é um solver hierárquico,
/// que é obra com nome.
pub(super) const CFILTER_BEND: Row = Row {
    label: "panel.sculpt3d.cfilter_bend",
    slider: ids::SCULPT3D_CFILTER_BEND,
    chip: ids::SCULPT3D_CFILTER_BEND_NUM,
    min: ClothFilterProps::BEND.0,
    max: ClothFilterProps::BEND.1,
    step: 0.05, // LITERAL-PX-OK: passo de uma fracção
    decimals: 2,
    get: |u| u.cloth_filter.bend,
    set: |u, v| u.cloth_filter.bend = v,
    show: is_cloth_filter,
    level: UiLevel::Basic,
    place: Place::Knobs,
};
