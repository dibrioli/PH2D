//! **OS TRÊS NÚMEROS DO PINCEL DE POSE** — irmão (`#[path]`) do [`super::rows`],
//! cortado por ASSUNTO, como os do tecido.
//!
//! ⚠️ **As faixas são as do alvo** (espec §1.1), e cada uma diz de que recurso
//! é: os segmentos e as suavizações são **tempo** (a construção é `O(V)` por
//! varredura e a suavização é `O(V·N)` **por segmento**, e o produto dos dois é
//! a queixa pública de desempenho deste pincel); o desvio é **alcance**, em
//! múltiplos do raio.

use ph2d_sculpt3d::Verb;

use super::{Place, Row};
use crate::state::{Sculpt3dUi, UiLevel};

/// **Esta row é do pincel de POSE?** — a pergunta é ao VERBO, como a do tecido.
/// ⛔ Uma lista paralela de nomes seria um knob que aparece noutra ferramenta e
/// não move um vértice.
fn is_pose(u: &Sculpt3dUi) -> bool {
    u.brush.verb == Verb::Pose
}

pub(super) const POSE_SEGMENTS: Row = Row {
    label: "panel.sculpt3d.pose_segments",
    slider: crate::ids::SCULPT3D_POSE_SEGMENTS,
    chip: crate::ids::SCULPT3D_POSE_SEGMENTS_NUM,
    min: 1.0,  // LITERAL-PX-OK: piso da faixa do alvo, em segmentos
    max: 20.0, // LITERAL-PX-OK: teto da faixa do alvo, em segmentos
    step: 1.0, // LITERAL-PX-OK: um segmento é inteiro
    decimals: 0,
    get: |u| u.brush.pose.segmentos as f32,
    // ⚠️ **Arredondar, e não truncar:** o slider entrega `f32` e um `as u32`
    // sobre `2,999…` daria `2` — o artista largaria o dedo no `3` e leria `2`.
    set: |u, v| u.brush.pose.segmentos = v.round().max(1.0) as u32,
    show: is_pose,
    level: UiLevel::Basic,
    place: Place::Knobs,
};

pub(super) const POSE_OFFSET: Row = Row {
    label: "panel.sculpt3d.pose_offset",
    slider: crate::ids::SCULPT3D_POSE_OFFSET,
    chip: crate::ids::SCULPT3D_POSE_OFFSET_NUM,
    min: 0.0,
    max: 2.0,   // LITERAL-PX-OK: teto da faixa do alvo, em raios de pincel
    step: 0.05, // LITERAL-PX-OK: knob em raios de pincel
    decimals: 2,
    get: |u| u.brush.pose.desvio_da_origem,
    set: |u, v| u.brush.pose.desvio_da_origem = v,
    show: is_pose,
    level: UiLevel::Basic,
    place: Place::Knobs,
};

pub(super) const POSE_SMOOTHINGS: Row = Row {
    label: "panel.sculpt3d.pose_smoothings",
    slider: crate::ids::SCULPT3D_POSE_SMOOTHINGS,
    chip: crate::ids::SCULPT3D_POSE_SMOOTHINGS_NUM,
    min: 0.0,
    max: 100.0, // LITERAL-PX-OK: teto da faixa do alvo, em iterações
    step: 1.0,  // LITERAL-PX-OK: uma iteração é inteira
    decimals: 0,
    get: |u| u.brush.pose.suavizacoes_do_peso as f32,
    set: |u, v| u.brush.pose.suavizacoes_do_peso = v.round().max(0.0) as u32,
    show: is_pose,
    // ⚠️ **`Pro` e não `Basic`:** ela é a única das três cujo efeito é de
    // ACABAMENTO (o peso de cada segmento é uma diferença, e a suavização
    // esbate a fronteira entre anéis) — e é a que custa mais caro, `O(V·N)` por
    // segmento. As outras duas mudam o que o gesto FAZ.
    level: UiLevel::Pro,
    place: Place::Knobs,
};
