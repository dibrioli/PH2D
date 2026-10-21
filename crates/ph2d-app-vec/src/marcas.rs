//! ⭐⭐ **O losango de cada campo do painel do vetor** (plano 35 §5; plano 34 §9 metade B) — que
//! canal da linha do tempo cada campo mostra, o estado dele no instante do autor, e o que o clique
//! escreve.
//!
//! A shell publica, a folha lê: [`marcas`] vira o mapa que `key_mark::set_key_marks` entrega ao
//! pintor da linha; [`clique`] vira os intents de um `key_mark::take_key_mark_click`. Um campo de
//! família mostra o seu GRUPO (a cor inteira, a forma inteira — `FieldKey::mesmo_grupo`), porque é o
//! grupo que se grava junto: o losango acende se QUALQUER membro tem chave, e o clique grava ou apaga
//! o grupo todo.

use std::collections::BTreeMap;

use ph2d_ecs::FieldKey;
use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as v;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::widget::key_mark::KeyMark;
use ph2d_panel_vector::ids as painel;
pub use ph2d_timeline::Instante;
use ph2d_timeline::{Forca, PropKind, TimelineDoc, TimelineIntent};
use ph2d_tool_vector::ids as ferramenta;

const fn em(c: u16, i: u16) -> PropKind {
    PropKind::Field(FieldKey::new(VETOR, c, i, 0))
}

const fn campo(c: u16) -> PropKind {
    em(c, 0)
}

/// Os pontos do caminho: o Transform do painel (X/Y/W/H/R movem os PONTOS, não o objecto) e o
/// ponto seleccionado.
const FORMA: PropKind = campo(v::ANCORA_X);
/// A tinta do preenchimento de base inteira: a cor chapada, as paragens e os pontos do gradiente,
/// o ângulo — um grupo só (`canal_vetor`, *Tinta*).
const PREENCHIMENTO: PropKind = campo(v::TINTA_PREENCH + v::OFF_SOLIDA);
const COR_DO_TRACO: PropKind = campo(v::TINTA_TRACO + v::OFF_SOLIDA);
const CONTORNO_COR: PropKind = campo(v::CONTORNO_COR);
/// As duas escalas das pontas (início e fim) são uma linha de dois campos: um losango, as duas.
const PONTAS: &[PropKind] = &[campo(v::TRACO_ESCALA_INICIO), campo(v::TRACO_ESCALA_FIM)];

/// ⭐ **A tabela campo → canal do painel do vetor** (os campos cujo canal não depende do que o
/// painel tem aberto). O gate `architecture_every_editable_field_is_a_channel` lê-a: um campo
/// pintado com coluna de losango tem de estar aqui, em [`DA_CAMADA_ABERTA`], ou na tabela
/// `⛔ não-animável` do gate.
const CAMPOS: &[(NodeId, &[PropKind])] = &[
    (ferramenta::VECTOR_TRANSFORM_X, &[FORMA]),
    (ferramenta::VECTOR_TRANSFORM_Y, &[FORMA]),
    (ferramenta::VECTOR_TRANSFORM_W, &[FORMA]),
    (ferramenta::VECTOR_TRANSFORM_H, &[FORMA]),
    (painel::VECTOR_TRANSFORM_R, &[FORMA]),
    (painel::VECTOR_VERT_X, &[FORMA]),
    (painel::VECTOR_VERT_Y, &[FORMA]),
    (ferramenta::VECTOR_FILL_SWATCH, &[PREENCHIMENTO]),
    (ferramenta::VECTOR_FILL_OPACITY, &[PREENCHIMENTO]),
    (painel::VECTOR_GRAD_ANGLE, &[PREENCHIMENTO]),
    (painel::VECTOR_GRAD_INFLUENCE, &[PREENCHIMENTO]),
    (painel::VECTOR_GRAD_JITTER, &[PREENCHIMENTO]),
    (ferramenta::VECTOR_STROKE_SWATCH, &[COR_DO_TRACO]),
    (ferramenta::VECTOR_STROKE_OPACITY, &[COR_DO_TRACO]),
    (ferramenta::VECTOR_WIDTH, &[campo(v::TRACO_LARGURA)]),
    (ferramenta::VECTOR_CAP_BUTT, &[campo(v::TRACO_PONTA)]),
    (ferramenta::VECTOR_CAP_ROUND, &[campo(v::TRACO_PONTA)]),
    (ferramenta::VECTOR_CAP_SQUARE, &[campo(v::TRACO_PONTA)]),
    (ferramenta::VECTOR_JOIN_MITER, &[campo(v::TRACO_JUNCAO)]),
    (ferramenta::VECTOR_JOIN_ROUND, &[campo(v::TRACO_JUNCAO)]),
    (ferramenta::VECTOR_JOIN_BEVEL, &[campo(v::TRACO_JUNCAO)]),
    (
        ferramenta::VECTOR_ALIGN_CENTRE,
        &[campo(v::TRACO_ALINHAMENTO)],
    ),
    (
        ferramenta::VECTOR_ALIGN_INNER,
        &[campo(v::TRACO_ALINHAMENTO)],
    ),
    (ferramenta::VECTOR_DASH_IDS[0], &[campo(v::TRACO_TRACEJADO)]),
    (ferramenta::VECTOR_DASH_IDS[1], &[campo(v::TRACO_TRACEJADO)]),
    (ferramenta::VECTOR_DASH_IDS[2], &[campo(v::TRACO_TRACEJADO)]),
    (ferramenta::VECTOR_GAP_IDS[0], &[campo(v::TRACO_TRACEJADO)]),
    (ferramenta::VECTOR_GAP_IDS[1], &[campo(v::TRACO_TRACEJADO)]),
    (ferramenta::VECTOR_GAP_IDS[2], &[campo(v::TRACO_TRACEJADO)]),
    (painel::VECTOR_FILL_RULE_NONZERO, &[campo(v::REGRA)]),
    (painel::VECTOR_FILL_RULE_EVENODD, &[campo(v::REGRA)]),
    (painel::VECTOR_OBJ_OPACITY, &[PropKind::Opacity]),
    (painel::VECTOR_OBJ_OPACITY_NUM, &[PropKind::Opacity]),
    (painel::VECTOR_CONTOUR_STEPS, &[campo(v::CONTORNO_PASSOS)]),
    (
        painel::VECTOR_CONTOUR_OFFSET,
        &[campo(v::CONTORNO_DISTANCIA)],
    ),
    (painel::VECTOR_CONTOUR_ACCEL, &[campo(v::CONTORNO_ACEL)]),
    (
        painel::VECTOR_CONTOUR_JOIN_MITER,
        &[campo(v::CONTORNO_JUNCAO)],
    ),
    (
        painel::VECTOR_CONTOUR_JOIN_ROUND,
        &[campo(v::CONTORNO_JUNCAO)],
    ),
    (
        painel::VECTOR_CONTOUR_JOIN_BEVEL,
        &[campo(v::CONTORNO_JUNCAO)],
    ),
    (
        painel::VECTOR_CONTOUR_SIDE_OUTER,
        &[campo(v::CONTORNO_LADO)],
    ),
    (
        painel::VECTOR_CONTOUR_SIDE_INNER,
        &[campo(v::CONTORNO_LADO)],
    ),
    (painel::VECTOR_CONTOUR_SIDE_BOTH, &[campo(v::CONTORNO_LADO)]),
    (painel::VECTOR_CONTOUR_TO, &[CONTORNO_COR]),
    (ferramenta::VECTOR_WIDTH_NUM, &[campo(v::TRACO_LARGURA)]),
    (ferramenta::VECTOR_STROKE_KIND_SOLID, &[COR_DO_TRACO]),
    (ferramenta::VECTOR_STROKE_KIND_PATTERN, &[COR_DO_TRACO]),
    (ferramenta::VECTOR_STROKE_KIND_BRUSH, &[COR_DO_TRACO]),
    (
        ferramenta::VECTOR_MARKER_ROUND,
        &[campo(v::TRACO_MARCA_REDONDA)],
    ),
    (ferramenta::VECTOR_MARKER_SCALE_IDS[0], PONTAS),
    (ferramenta::VECTOR_MARKER_SCALE_IDS[1], PONTAS),
    (painel::VECTOR_TEXT_SIZE, &[campo(v::TEXTO_TAMANHO)]),
    (painel::VECTOR_TEXT_WEIGHT, &[campo(v::TEXTO_PESO)]),
    (
        painel::VECTOR_TEXT_LINE_HEIGHT,
        &[campo(v::TEXTO_ENTRELINHA)],
    ),
    (painel::VECTOR_TEXT_TRACKING, &[campo(v::TEXTO_TRACKING)]),
    (
        painel::VECTOR_TEXT_ALIGN_LEFT,
        &[campo(v::TEXTO_ALINHAMENTO)],
    ),
    (
        painel::VECTOR_TEXT_ALIGN_CENTER,
        &[campo(v::TEXTO_ALINHAMENTO)],
    ),
    (
        painel::VECTOR_TEXT_ALIGN_RIGHT,
        &[campo(v::TEXTO_ALINHAMENTO)],
    ),
    (painel::VECTOR_TEXT_WRAP_W, &[campo(v::TEXTO_QUEBRA)]),
    (painel::VECTOR_EXPAND_OFFSET, &[campo(v::OFFSET_DISTANCIA)]),
    (
        ferramenta::VECTOR_EXPAND_JOIN_MITER,
        &[campo(v::OFFSET_JUNCAO)],
    ),
    (
        ferramenta::VECTOR_EXPAND_JOIN_ROUND,
        &[campo(v::OFFSET_JUNCAO)],
    ),
    (
        ferramenta::VECTOR_EXPAND_JOIN_BEVEL,
        &[campo(v::OFFSET_JUNCAO)],
    ),
    (
        ferramenta::VECTOR_EXPAND_SIDE_OUTER,
        &[campo(v::OFFSET_LADO)],
    ),
    (
        ferramenta::VECTOR_EXPAND_SIDE_INNER,
        &[campo(v::OFFSET_LADO)],
    ),
    (
        ferramenta::VECTOR_EXPAND_SIDE_BOTH,
        &[campo(v::OFFSET_LADO)],
    ),
    (painel::VECTOR_OBJ_BLEND, &[campo(v::MISTURA)]),
    // O Fill Type, ter traço, e a tinta da barra são membros da TINTA (o tipo, o «existe»): o
    // mesmo grupo que a cor.
    (ferramenta::VECTOR_FILL_KIND_SOLID, &[PREENCHIMENTO]),
    (ferramenta::VECTOR_FILL_KIND_LINEAR, &[PREENCHIMENTO]),
    (ferramenta::VECTOR_FILL_KIND_RADIAL, &[PREENCHIMENTO]),
    (ferramenta::VECTOR_FILL_KIND_MULTI, &[PREENCHIMENTO]),
    (ferramenta::VECTOR_FILL_KIND_PATTERN, &[PREENCHIMENTO]),
    (painel::VECTOR_STROKE_PRESENT, &[COR_DO_TRACO]),
    (painel::VECTOR_BAR_KIND, &[COR_DO_TRACO]),
    (
        ferramenta::VECTOR_MARKER_START_DD,
        &[campo(v::TRACO_MARCA_INICIO)],
    ),
    (
        ferramenta::VECTOR_MARKER_END_DD,
        &[campo(v::TRACO_MARCA_FIM)],
    ),
];

/// ⭐ **Os campos da entrada ABERTA da pilha de pinturas** — o id é um só para qualquer entrada; o
/// canal é o campo daqui com `i = aberta + 1` ([`Contexto::aberta`]).
const DA_CAMADA_ABERTA: &[(NodeId, u16)] = &[
    (painel::VECTOR_PAINT_WIDTH, v::TRACO_LARGURA),
    (painel::VECTOR_PAINT_DX, v::PILHA_DESLOCA_X),
    (painel::VECTOR_PAINT_DY, v::PILHA_DESLOCA_Y),
    (painel::VECTOR_PAINT_DILATE, v::PILHA_DILATA),
    (painel::VECTOR_PAINT_JOIN_MITER, v::PILHA_DILATA_JUNCAO),
    (painel::VECTOR_PAINT_JOIN_ROUND, v::PILHA_DILATA_JUNCAO),
    (painel::VECTOR_PAINT_JOIN_BEVEL, v::PILHA_DILATA_JUNCAO),
    (painel::VECTOR_PAINT_OPACITY, v::PILHA_OPACIDADE),
    (painel::VECTOR_PAINT_OPACITY_NUM, v::PILHA_OPACIDADE),
    (painel::VECTOR_PAINT_BLEND, v::PILHA_MISTURA),
];

/// Os controlos do padrão de textura que são membros da TINTA (tamanho, vão, origem, ângulo,
/// reticulado, repetição). ⛔ Fora: a arte e «Use shape» (verbos), o desfasamento (sem campo), os
/// dois elos (gesto da sessão).
/// (O gate lê os nomes por extenso.)
const PADRAO: &[painel::TexPatKnob] = &[
    painel::TexPatKnob::Tile(0),
    painel::TexPatKnob::Tile(1),
    painel::TexPatKnob::Tile(2),
    painel::TexPatKnob::Tile(3),
    painel::TexPatKnob::Mode(0),
    painel::TexPatKnob::Mode(1),
    painel::TexPatKnob::Mode(2),
    painel::TexPatKnob::Width,
    painel::TexPatKnob::WidthNum,
    painel::TexPatKnob::Height,
    painel::TexPatKnob::HeightNum,
    painel::TexPatKnob::Gap,
    painel::TexPatKnob::GapNum,
    painel::TexPatKnob::GapY,
    painel::TexPatKnob::GapYNum,
    painel::TexPatKnob::Angle,
    painel::TexPatKnob::AngleNum,
    painel::TexPatKnob::ShiftX,
    painel::TexPatKnob::ShiftXNum,
    painel::TexPatKnob::ShiftY,
    painel::TexPatKnob::ShiftYNum,
];

fn em_j(c: u16, i: usize, j: usize) -> PropKind {
    let n = |x: usize| u16::try_from(x).unwrap_or(u16::MAX);
    PropKind::Field(FieldKey::new(VETOR, c, n(i), n(j)))
}

/// ⭐ **Os campos de id DERIVADO de um índice** — o padrão de textura de cada tinta (`slot` 0 =
/// preenchimento, 1 = traço), os parâmetros e o olho de cada efeito (a linha `r` do painel é o
/// efeito `r` do documento), os campos da forma paramétrica na fila ② (o número e o gémeo que
/// cicla). Os ids saem de um `format!` + hash: calculam-se UMA vez, nunca por quadro.
fn derivados() -> &'static [(NodeId, PropKind)] {
    static T: std::sync::LazyLock<Vec<(NodeId, PropKind)>> = std::sync::LazyLock::new(|| {
        let mut t = Vec::new();
        for (slot, tinta) in [PREENCHIMENTO, COR_DO_TRACO].into_iter().enumerate() {
            t.extend(PADRAO.iter().map(|&k| (painel::texpat_id(slot, k), tinta)));
        }
        for r in 0..ferramenta::MAX_FX_ROWS {
            t.push((
                ferramenta::vector_fx_hide_id(r),
                em_j(v::EFEITO_LIGADO, r, 0),
            ));
            for p in 0..ferramenta::MAX_FX_ROW_PARAMS {
                let c = em_j(v::EFEITO_PARAM, r, p);
                t.push((ferramenta::vector_fx_param_id(r, p), c));
                t.push((ferramenta::vector_fx_param_num_id(r, p), c));
                t.push((ferramenta::vector_fx_toggle_id(r, p), c));
            }
        }
        for j in 0..ph2d_vec_scene::MAX_SHAPE_FIELDS {
            let c = em_j(v::PARAM_VALOR, 0, j);
            t.push((ferramenta::vector_shape_field_id(j), c));
            t.push((ferramenta::vector_shape_choice_id(j), c));
        }
        t
    });
    &T
}

/// O que o painel mostra agora e muda o canal de um campo: a entrada ABERTA da pilha e quantas
/// entradas há (a swatch da entrada `k` mostra a tinta dela, `i = k + 1`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Contexto {
    pub aberta: Option<usize>,
    pub camadas: usize,
}

impl Contexto {
    /// O estado de vista do painel do vetor neste quadro.
    #[must_use]
    pub fn do_painel() -> Self {
        let camadas = ph2d_panel_vector::state::current_appearance().map_or(0, |a| a.layers.len());
        Self {
            aberta: ph2d_panel_vector::state::open_layer_index().filter(|&k| k < camadas),
            camadas,
        }
    }
}

fn entrada(k: usize) -> u16 {
    u16::try_from(k + 1).unwrap_or(u16::MAX)
}

/// O GRUPO de canais que o campo `id` do painel mostra (vazio = o campo não tem canal).
#[must_use]
pub fn canal_do_campo(id: NodeId, ctx: Contexto) -> Vec<PropKind> {
    if let Some(&(_, g)) = CAMPOS.iter().find(|(c, _)| *c == id) {
        return g.to_vec();
    }
    if let Some(&(_, g)) = derivados().iter().find(|(c, _)| *c == id) {
        return vec![g];
    }
    if let Some(k) = ctx.aberta
        && let Some(&(_, c)) = DA_CAMADA_ABERTA.iter().find(|(x, _)| *x == id)
    {
        return vec![em(c, entrada(k))];
    }
    for k in 0..ctx.camadas {
        let i = entrada(k);
        // A swatch da entrada `k`: a tinta dela — de preenchimento OU de traço; só a que a entrada
        // tem está ligada ou publicada, a outra não responde.
        if painel::vector_paint_swatch_id(k) == id {
            return vec![
                em(v::TINTA_PREENCH + v::OFF_SOLIDA, i),
                em(v::TINTA_TRACO + v::OFF_SOLIDA, i),
            ];
        }
        if painel::vector_paint_eye_id(k) == id {
            return vec![em(v::PILHA_LIGADA, i)];
        }
    }
    Vec::new()
}

/// Todo id que pode ter losango neste contexto.
fn ids(ctx: Contexto) -> impl Iterator<Item = NodeId> {
    let aberta = ctx.aberta.is_some();
    CAMPOS
        .iter()
        .map(|&(id, _)| id)
        .chain(
            DA_CAMADA_ABERTA
                .iter()
                .filter(move |_| aberta)
                .map(|&(id, _)| id),
        )
        .chain(derivados().iter().map(|&(id, _)| id))
        .chain((0..ctx.camadas).flat_map(|k| {
            [
                painel::vector_paint_swatch_id(k),
                painel::vector_paint_eye_id(k),
            ]
        }))
}

/// A força do grupo como o losango a pinta.
#[must_use]
pub fn marca(f: Forca) -> KeyMark {
    match f {
        Forca::Livre => KeyMark::Livre,
        Forca::Animado => KeyMark::Animado,
        Forca::Chave => KeyMark::Chave,
        Forca::Mudado => KeyMark::Mudado,
    }
}

/// ⭐ **O mapa do quadro**: o losango de cada campo para os objectos `entidades` (os do painel — o
/// mais forte entre eles). `valor(e, p)` = o que o mundo vale agora (a porta do autokey). Sem
/// objectos, vazio: nenhum campo tem canal.
#[must_use]
pub fn marcas(
    doc: &TimelineDoc,
    ctx: Contexto,
    entidades: &[u64],
    inst: Instante,
    valor: impl Fn(u64, PropKind) -> Option<f32>,
) -> BTreeMap<NodeId, KeyMark> {
    let mut por_grupo: BTreeMap<Vec<PropKind>, KeyMark> = BTreeMap::new();
    let mut out = BTreeMap::new();
    if entidades.is_empty() {
        return out;
    }
    for id in ids(ctx) {
        let reps = canal_do_campo(id, ctx);
        let m = *por_grupo.entry(reps).or_insert_with_key(|reps| {
            marca(ph2d_timeline::estado_do_grupo(doc, entidades, reps, inst, &valor).0)
        });
        out.insert(id, m);
    }
    out
}

/// ⭐ **O que o clique num losango escreve** — o grupo do primeiro campo da linha que tem canal,
/// pela porta única [`ph2d_timeline::clique_no_grupo`]. Fora da tabela, nada.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn clique(
    doc: &TimelineDoc,
    ctx: Contexto,
    campos: &[NodeId],
    entidades: &[u64],
    inst: Instante,
    lidos: impl Fn(u64) -> Vec<FieldKey>,
    valor: impl Fn(u64, PropKind) -> Option<f32>,
    autorar: impl Fn(u64, PropKind) -> Option<TimelineIntent>,
) -> Vec<TimelineIntent> {
    let reps = campos
        .iter()
        .map(|&c| canal_do_campo(c, ctx))
        .find(|r| !r.is_empty())
        .unwrap_or_default();
    ph2d_timeline::clique_no_grupo(doc, &reps, entidades, inst, lidos, valor, autorar)
}

#[cfg(test)]
#[path = "marcas_tests.rs"]
mod tests;
