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
use ph2d_timeline::{ChannelMark, PropKind, SelectedKey, TimelineDoc, TimelineIntent};
use ph2d_tool_vector::ids as ferramenta;

const fn campo(c: u16) -> PropKind {
    PropKind::Field(FieldKey::new(VETOR, c, 0, 0))
}

/// Os pontos do caminho: o Transform do painel (X/Y/W/H/R movem os PONTOS, não o objecto) e o
/// ponto seleccionado.
const FORMA: PropKind = campo(v::ANCORA_X);
const PREENCHIMENTO: PropKind = campo(v::TINTA_PREENCH + v::OFF_SOLIDA);
const COR_DO_TRACO: PropKind = campo(v::TINTA_TRACO + v::OFF_SOLIDA);

/// ⭐ **A tabela campo → canal do painel do vetor.** Um campo pintado com coluna de losango e fora
/// daqui pinta o ponto inerte. (Fora, com o porquê no handoff F24b: as linhas da pilha de pinturas
/// — o id não diz QUAL entrada —, o gradiente e os contornos.)
const CAMPOS: &[(NodeId, PropKind)] = &[
    (ferramenta::VECTOR_TRANSFORM_X, FORMA),
    (ferramenta::VECTOR_TRANSFORM_Y, FORMA),
    (ferramenta::VECTOR_TRANSFORM_W, FORMA),
    (ferramenta::VECTOR_TRANSFORM_H, FORMA),
    (painel::VECTOR_TRANSFORM_R, FORMA),
    (painel::VECTOR_VERT_X, FORMA),
    (painel::VECTOR_VERT_Y, FORMA),
    (ferramenta::VECTOR_FILL_SWATCH, PREENCHIMENTO),
    (ferramenta::VECTOR_FILL_OPACITY, PREENCHIMENTO),
    (ferramenta::VECTOR_STROKE_SWATCH, COR_DO_TRACO),
    (ferramenta::VECTOR_STROKE_OPACITY, COR_DO_TRACO),
    (ferramenta::VECTOR_WIDTH, campo(v::TRACO_LARGURA)),
    (ferramenta::VECTOR_DASH_IDS[0], campo(v::TRACO_TRACEJADO)),
    (ferramenta::VECTOR_DASH_IDS[1], campo(v::TRACO_TRACEJADO)),
    (ferramenta::VECTOR_DASH_IDS[2], campo(v::TRACO_TRACEJADO)),
    (ferramenta::VECTOR_GAP_IDS[0], campo(v::TRACO_TRACEJADO)),
    (ferramenta::VECTOR_GAP_IDS[1], campo(v::TRACO_TRACEJADO)),
    (ferramenta::VECTOR_GAP_IDS[2], campo(v::TRACO_TRACEJADO)),
    (painel::VECTOR_OBJ_OPACITY, PropKind::Opacity),
];

/// O canal que o campo `id` do painel mostra.
#[must_use]
pub fn canal_do_campo(id: NodeId) -> Option<PropKind> {
    CAMPOS.iter().find(|(c, _)| *c == id).map(|&(_, p)| p)
}

/// O instante do autor: o relógio da vista JÁ CORTADO (`TimelineDoc::author_cut`) e se a vista é a
/// Keys (a curva do clip activo, sem a pilha). O `t_src` de cada objecto compõe-se por cima.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Instante {
    pub t_cut: f64,
    pub solo: bool,
}

/// Os canais de `rep` que `entity` tem LIGADOS: o próprio, ou — campo de família — todo o grupo.
fn ligados(doc: &TimelineDoc, entity: u64, rep: PropKind) -> Vec<PropKind> {
    doc.bindings()
        .iter()
        .filter(|b| b.entity == entity && mesmo_canal(b.prop, rep))
        .map(|b| b.prop)
        .collect()
}

fn mesmo_canal(p: PropKind, rep: PropKind) -> bool {
    match (p, rep) {
        (PropKind::Field(a), PropKind::Field(b)) => a.mesmo_grupo(b),
        _ => p == rep,
    }
}

/// O estado do canal `rep` em `entity` — o mais forte entre os membros — e as chaves deste quadro.
fn marca_do_canal(
    doc: &TimelineDoc,
    entity: u64,
    rep: PropKind,
    inst: Instante,
    valor: &impl Fn(u64, PropKind) -> Option<f32>,
) -> (KeyMark, Vec<SelectedKey>) {
    let t_src = ph2d_timeline::remapped_time(doc, entity, inst.t_cut);
    let mut marca = KeyMark::Livre;
    let mut chaves = Vec::new();
    for p in ligados(doc, entity, rep) {
        let m =
            match ph2d_timeline::channel_mark(doc, entity, p, t_src, inst.solo, valor(entity, p)) {
                ChannelMark::Livre => KeyMark::Livre,
                ChannelMark::Animado => KeyMark::Animado,
                ChannelMark::Chave(k) => {
                    chaves.push(k);
                    KeyMark::Chave
                }
                ChannelMark::Mudado => KeyMark::Mudado,
            };
        marca = marca.max(m);
    }
    (marca, chaves)
}

/// ⭐ **O mapa do quadro**: o losango de cada campo da tabela para os objectos `entidades` (os do
/// painel — o mais forte entre eles). `valor(e, p)` = o que o mundo vale agora (a porta do autokey).
/// Sem objectos, vazio: nenhum campo tem canal.
#[must_use]
pub fn marcas(
    doc: &TimelineDoc,
    entidades: &[u64],
    inst: Instante,
    valor: impl Fn(u64, PropKind) -> Option<f32>,
) -> BTreeMap<NodeId, KeyMark> {
    let mut por_canal: BTreeMap<PropKind, KeyMark> = BTreeMap::new();
    let mut out = BTreeMap::new();
    if entidades.is_empty() {
        return out;
    }
    for &(id, rep) in CAMPOS {
        let m = *por_canal.entry(rep).or_insert_with(|| {
            entidades
                .iter()
                .map(|&e| marca_do_canal(doc, e, rep, inst, &valor).0)
                .max()
                .unwrap_or(KeyMark::Livre)
        });
        out.insert(id, m);
    }
    out
}

/// ⭐ **O que o clique num losango escreve.** Com chave neste quadro em algum membro ⇒ apaga as
/// chaves do grupo neste quadro (selecciona-as e `DeleteSelection`, sem intent nova). Senão ⇒ grava
/// o grupo inteiro: cada campo que a família publicou para o objecto (`lidos`), cada um pela porta
/// da tecla K (`autorar` devolve o `AddKey`, ou `None` quando a K recusaria), num passo de undo.
#[must_use]
pub fn clique(
    doc: &TimelineDoc,
    campos: &[NodeId],
    entidades: &[u64],
    inst: Instante,
    lidos: impl Fn(u64) -> Vec<FieldKey>,
    valor: impl Fn(u64, PropKind) -> Option<f32>,
    autorar: impl Fn(u64, PropKind) -> Option<TimelineIntent>,
) -> Vec<TimelineIntent> {
    let Some(rep) = campos.iter().find_map(|&c| canal_do_campo(c)) else {
        return Vec::new();
    };
    let chaves: Vec<SelectedKey> = entidades
        .iter()
        .flat_map(|&e| marca_do_canal(doc, e, rep, inst, &valor).1)
        .collect();
    if !chaves.is_empty() {
        let mut out = vec![TimelineIntent::BeginEdit, TimelineIntent::ClearSelection];
        out.extend(chaves.into_iter().map(TimelineIntent::AddToSelection));
        out.extend([TimelineIntent::DeleteSelection, TimelineIntent::EndEdit]);
        return out;
    }
    let mut out = vec![TimelineIntent::BeginEdit];
    for &e in entidades {
        let membros: Vec<PropKind> = match rep {
            PropKind::Field(k) => lidos(e)
                .into_iter()
                .filter(|o| o.mesmo_grupo(k))
                .map(PropKind::Field)
                .collect(),
            p => vec![p],
        };
        out.extend(membros.into_iter().filter_map(|p| autorar(e, p)));
    }
    out.push(TimelineIntent::EndEdit);
    out
}

#[cfg(test)]
#[path = "marcas_tests.rs"]
mod tests;
