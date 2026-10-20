//! ⭐⭐ **O losango de um GRUPO de canais** (plano 35 §5) — a porta única que todo painel usa para
//! dizer o estado do que um campo mostra e para escrever o clique nele.
//!
//! Um campo mostra um GRUPO: a cor inteira (r, g, b, a), a forma inteira, a posição (X e Y). O
//! losango acende se QUALQUER membro tem chave, e o clique grava ou apaga o grupo todo. Os grupos
//! de família (`PropKind::Field`) alargam-se por `FieldKey::mesmo_grupo`; os outros são a lista dada.

use crate::{ChannelMark, PropKind, SelectedKey, TimelineDoc, TimelineIntent};
use ph2d_ecs::FieldKey;

/// O instante do autor: o relógio da vista JÁ CORTADO ([`TimelineDoc::author_cut`]) e se a vista é
/// a Keys (a curva do clip activo, sem a pilha). O `t_src` de cada objecto compõe-se por cima.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Instante {
    /// O relógio da vista, cortado.
    pub t_cut: f64,
    /// A vista Keys (a curva do clip activo).
    pub solo: bool,
}

/// O estado de um grupo, do mais fraco ao mais forte — o losango mostra o mais forte dos membros.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Forca {
    /// Nenhuma curva conduz o grupo.
    Livre,
    /// Uma curva conduz, sem chave neste quadro.
    Animado,
    /// Há chave neste quadro.
    Chave,
    /// O mundo saiu da curva e ninguém o gravou.
    Mudado,
}

fn mesmo_canal(p: PropKind, rep: PropKind) -> bool {
    match (p, rep) {
        (PropKind::Field(a), PropKind::Field(b)) => a.mesmo_grupo(b),
        _ => p == rep,
    }
}

/// Os canais do grupo `reps` que `entity` tem LIGADOS na timeline.
fn ligados(doc: &TimelineDoc, entity: u64, reps: &[PropKind]) -> Vec<PropKind> {
    doc.bindings()
        .iter()
        .filter(|b| b.entity == entity && reps.iter().any(|&r| mesmo_canal(b.prop, r)))
        .map(|b| b.prop)
        .collect()
}

/// ⭐ **O estado do grupo `reps` nos objectos `entidades`** — o mais forte entre membros e objectos
/// — e as chaves deste quadro (para o clique as apagar). `valor(e, p)` = o que o mundo vale agora.
#[must_use]
pub fn estado_do_grupo(
    doc: &TimelineDoc,
    entidades: &[u64],
    reps: &[PropKind],
    inst: Instante,
    valor: &impl Fn(u64, PropKind) -> Option<f32>,
) -> (Forca, Vec<SelectedKey>) {
    let mut forca = Forca::Livre;
    let mut chaves = Vec::new();
    for &e in entidades {
        let t_src = crate::remapped_time(doc, e, inst.t_cut);
        for p in ligados(doc, e, reps) {
            let m = match crate::channel_mark(doc, e, p, t_src, inst.solo, valor(e, p)) {
                ChannelMark::Livre => Forca::Livre,
                ChannelMark::Animado => Forca::Animado,
                ChannelMark::Chave(k) => {
                    chaves.push(k);
                    Forca::Chave
                }
                ChannelMark::Mudado => Forca::Mudado,
            };
            forca = forca.max(m);
        }
    }
    (forca, chaves)
}

/// ⭐ **O que o clique no losango do grupo `reps` escreve.** Com chave neste quadro em algum membro
/// ⇒ apaga as chaves do grupo neste quadro (selecciona-as e `DeleteSelection`). Senão ⇒ grava o
/// grupo inteiro: num grupo de família, cada campo que ela publicou para o objecto (`lidos`); nos
/// outros, cada canal da lista — cada um pela porta da tecla K (`autorar` devolve o `AddKey`, ou
/// `None` quando a K recusaria), num passo de undo.
#[must_use]
pub fn clique_no_grupo(
    doc: &TimelineDoc,
    reps: &[PropKind],
    entidades: &[u64],
    inst: Instante,
    lidos: impl Fn(u64) -> Vec<FieldKey>,
    valor: impl Fn(u64, PropKind) -> Option<f32>,
    autorar: impl Fn(u64, PropKind) -> Option<TimelineIntent>,
) -> Vec<TimelineIntent> {
    if reps.is_empty() {
        return Vec::new();
    }
    let (_, chaves) = estado_do_grupo(doc, entidades, reps, inst, &valor);
    if !chaves.is_empty() {
        let mut out = vec![TimelineIntent::BeginEdit, TimelineIntent::ClearSelection];
        out.extend(chaves.into_iter().map(TimelineIntent::AddToSelection));
        out.extend([TimelineIntent::DeleteSelection, TimelineIntent::EndEdit]);
        return out;
    }
    let mut out = vec![TimelineIntent::BeginEdit];
    for &e in entidades {
        let publicados = if reps.iter().any(|r| matches!(r, PropKind::Field(_))) {
            lidos(e)
        } else {
            Vec::new()
        };
        let mut membros: Vec<PropKind> = Vec::new();
        for &r in reps {
            match r {
                PropKind::Field(k) => membros.extend(
                    publicados
                        .iter()
                        .filter(|o| o.mesmo_grupo(k))
                        .map(|&o| PropKind::Field(o)),
                ),
                p => membros.push(p),
            }
        }
        membros.sort_unstable();
        membros.dedup();
        out.extend(membros.into_iter().filter_map(|p| autorar(e, p)));
    }
    out.push(TimelineIntent::EndEdit);
    out
}

#[cfg(test)]
#[path = "channel_group_tests.rs"]
mod tests;
