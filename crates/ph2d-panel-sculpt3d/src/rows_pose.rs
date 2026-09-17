//! **OS TRÊS NÚMEROS DO PINCEL DE POSE** — irmão (`#[path]`) do [`super::rows`],
//! cortado por ASSUNTO, como os do tecido.
//!
//! ⚠️ **As faixas são as do alvo** (espec §1.1) **menos uma**, e cada uma diz de
//! que recurso é: os segmentos e as suavizações são **tempo** (a construção é
//! `O(V)` por varredura e a suavização é `O(V·N)` **por segmento**, e o produto
//! dos dois é a queixa pública de desempenho deste pincel); o desvio é
//! **alcance**, em múltiplos do raio.
//!
//! ⭐ **A excepção é o DESVIO, que vai a `3` por ordem do dono** (o alvo pára em
//! `2`) — divergência **declarada**, com a medição no doc da própria row.

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

/// ⭐⭐ **O DESVIO DA ORIGEM chega a `3`, e isso é uma DIVERGÊNCIA DECLARADA do
/// alvo** — ordem do dono, 2026-09-14. O alvo oferece `0..2` (espec §1.1).
///
/// ⚠️ **Medido antes de escrever o número** (`ph2d-pose`, sonda
/// `sonda_o_desvio_da_origem_alem_do_tecto`, três malhas do corpus do oráculo,
/// raio `0,25`, arrasto `0,2`):
///
/// | desvio | comprimento do 1.º segmento | deslocamento máximo | construção |
/// |---|---|---|---|
/// | `0` | `0,223`–`0,251` | `0,156`–`0,180` | `0,17`–`0,55 ms` |
/// | `2` (tecto do alvo) | `0,723`–`0,751` | `0,189`–`0,202` | `0,98`–`5,75 ms` |
/// | **`3`** | `0,973`–`1,001` | `0,192`–`0,201` | `1,35`–`5,76 ms` |
/// | `4` | `1,223`–`1,251` | `0,194`–`0,200` | `1,74`–`5,75 ms` |
///
/// ⭐ **Três leituras, e nenhuma é uma esperança:**
/// 1. a alavanca é **exactamente linear** — `raio × (1 + desvio)`, e a coluna
///    confirma-o ao cêntimo;
/// 2. o efeito **satura**: o deslocamento mal se move depois de `2`, porque com
///    o pivô longe a rotação tende para uma **translação** — o limite
///    geométrico, não um artefacto;
/// 3. o relógio **também satura** nas duas malhas maiores (`5,75` → `5,76` →
///    `5,75 ms` de `2` a `4`), e cresce devagar na pequena.
///
/// ⇒ subir de `2` para `3` **não abre regime novo nenhum** — dá mais alcance à
/// dobradiça ao mesmo preço. ⛔ E o que fica do outro lado do `3` está medido e
/// é mais do mesmo: o tecto é de PRODUTO (até onde vale a pena arrastar um
/// knob), não de recurso.
pub(super) const POSE_OFFSET: Row = Row {
    label: "panel.sculpt3d.pose_offset",
    slider: crate::ids::SCULPT3D_POSE_OFFSET,
    chip: crate::ids::SCULPT3D_POSE_OFFSET_NUM,
    min: 0.0,
    max: 3.0,   // LITERAL-PX-OK: ordem do dono, em raios de pincel — ver acima
    step: 0.05, // LITERAL-PX-OK: knob em raios de pincel
    decimals: 2,
    get: |u| u.brush.pose.desvio_da_origem,
    set: |u, v| u.brush.pose.desvio_da_origem = v,
    show: is_pose,
    level: UiLevel::Basic,
    place: Place::Knobs,
};

/// ⭐⭐⭐ **O TECTO VAI A `300` — report do dono de 2026-09-17** (*«mesmo com
/// Weight Smoothing no máximo não consigo uma transição mais suave. Poderia
/// aumentar o máximo do slider em 3x?»*).
///
/// ⛔⛔ **O knob deixou de ser uma CONTAGEM e passou a ser uma LARGURA.** Ele
/// pedia iterações de difusão, e a largura que elas compram conta-se em
/// **anéis da malha** (`≈ 2,0·√N` arestas) — a mesma posição do slider dava
/// transição larga numa peça grossa e estreita numa fina, e era por isso que o
/// máximo não chegava. Hoje ele é a largura **em raios de pincel**, medida no
/// barro: a tabela das quatro densidades, o joelho de `1,0` e o tecto de `2,0`
/// (que é do NÚCLEO) vivem ao lado da constante, em
/// [`ph2d_sculpt3d::PoseControlos::TRANSICAO_MAX`] — *a faixa mora onde a lei
/// que a justifica mora, e o painel lê-a.*
pub(super) const POSE_TRANSITION: Row = Row {
    label: "panel.sculpt3d.pose_transition",
    slider: crate::ids::SCULPT3D_POSE_TRANSITION,
    chip: crate::ids::SCULPT3D_POSE_TRANSITION_NUM,
    min: 0.0,
    // ⛔ **LIDO da crate da lei, nunca escrito aqui** — um literal neste sítio
    // seria a segunda resposta à pergunta *«até onde vai este knob?»*, e a que
    // o artista vê é sempre a que envelhece.
    max: ph2d_sculpt3d::PoseControlos::TRANSICAO_MAX,
    step: 0.05, // LITERAL-PX-OK: a largura é contínua, em raios de pincel
    decimals: 2,
    get: |u| u.brush.pose.transicao,
    set: |u, v| u.brush.pose.transicao = v.max(0.0),
    show: is_pose,
    // ⚠️⚠️ **`Basic` desde 2026-09-17, e a mudança é MEDIDA.** Ela era `Pro`
    // por ser *«acabamento»* e cara (`O(V·N)` por segmento); hoje ela é o que
    // decide se a borda da deformação **entalha** — a `0,6` o mesmo gesto vira
    // `606` faces do avesso e a `1,0` vira `0` —, e custa `3,4`–`4,7 ms` no
    // pen-down, plano na largura pedida. *Um knob que separa a ferramenta boa da
    // partida não é acabamento.*
    level: UiLevel::Basic,
    place: Place::Knobs,
};
