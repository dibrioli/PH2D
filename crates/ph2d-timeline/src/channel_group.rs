//! ⭐⭐ **O losango de um GRUPO de canais** (plano 35 §5) — a porta única que todo painel usa para
//! dizer o estado do que um campo mostra e para escrever o clique nele.
//!
//! Um campo mostra um GRUPO: a cor inteira (r, g, b, a), a forma inteira, a posição (X e Y). O
//! losango acende se QUALQUER membro tem chave, e o clique grava ou apaga o grupo todo. Os grupos
//! de família (`PropKind::Field`) alargam-se por `FieldKey::mesmo_grupo`; os outros são a lista dada.

use std::collections::BTreeMap;

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

/// O GRUPO de um canal: o da família (`FieldKey::grupo`), ou o próprio canal — dois canais são do
/// mesmo grupo se e só se o endereço é igual (a lei de `FieldKey::mesmo_grupo`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Grupo {
    Familia((u16, u16, u32)),
    Canal(PropKind),
}

fn grupo_de(p: PropKind) -> Grupo {
    match p {
        PropKind::Field(k) => k.grupo().map_or(Grupo::Canal(p), Grupo::Familia),
        _ => Grupo::Canal(p),
    }
}

/// ⭐ **As ligações dos objectos `entidades` por GRUPO, e o relógio de cada um** — construídas UMA
/// vez por quadro por quem pergunta por centenas de grupos (o Inspector inteiro, os campos do
/// vetor, as camadas do Flip). Sem elas cada grupo varria todas as ligações do documento e recompunha
/// o `remapped_time`: 17 ms por quadro com 462 canais (plano F27 §4).
pub struct Ligacoes<'d> {
    doc: &'d TimelineDoc,
    inst: Instante,
    /// `(entidade, t_src)`, pela ordem de `entidades`.
    relogios: Vec<(u64, f64)>,
    /// Os índices em `doc.bindings()` de cada `(entidade, grupo)`, pela ordem do documento.
    por_grupo: BTreeMap<(u64, Grupo), Vec<usize>>,
}

impl<'d> Ligacoes<'d> {
    /// As ligações dos objectos `entidades` do documento, no instante `inst`.
    #[must_use]
    pub fn de(doc: &'d TimelineDoc, entidades: &[u64], inst: Instante) -> Self {
        let mut por_grupo: BTreeMap<(u64, Grupo), Vec<usize>> = BTreeMap::new();
        for (i, b) in doc.bindings().iter().enumerate() {
            if entidades.contains(&b.entity) {
                por_grupo
                    .entry((b.entity, grupo_de(b.prop)))
                    .or_default()
                    .push(i);
            }
        }
        let relogios = (entidades.iter())
            .map(|&e| (e, crate::remapped_time(doc, e, inst.t_cut)))
            .collect();
        Self {
            doc,
            inst,
            relogios,
            por_grupo,
        }
    }

    /// ⭐ **O estado do grupo `reps`** — o mais forte entre membros e objectos — e as chaves deste
    /// quadro (para o clique as apagar). `valor(e, p)` = o que o mundo vale agora.
    #[must_use]
    pub fn estado(
        &self,
        reps: &[PropKind],
        valor: &impl Fn(u64, PropKind) -> Option<f32>,
    ) -> (Forca, Vec<SelectedKey>) {
        let mut grupos: Vec<Grupo> = reps.iter().map(|&r| grupo_de(r)).collect();
        grupos.sort_unstable();
        grupos.dedup();
        let mut forca = Forca::Livre;
        let mut chaves = Vec::new();
        for &(e, t_src) in &self.relogios {
            let mut ix: Vec<usize> = (grupos.iter())
                .filter_map(|&g| self.por_grupo.get(&(e, g)))
                .flatten()
                .copied()
                .collect();
            ix.sort_unstable();
            for b in ix.into_iter().map(|i| &self.doc.bindings()[i]) {
                let mark = crate::channel_mark::channel_mark_b(
                    self.doc,
                    b,
                    t_src,
                    self.inst.solo,
                    valor(e, b.prop),
                );
                let m = match mark {
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
}

/// [`Ligacoes::estado`] de um grupo só (o clique; quem pergunta por muitos constrói as
/// [`Ligacoes`] uma vez).
#[must_use]
pub fn estado_do_grupo(
    doc: &TimelineDoc,
    entidades: &[u64],
    reps: &[PropKind],
    inst: Instante,
    valor: &impl Fn(u64, PropKind) -> Option<f32>,
) -> (Forca, Vec<SelectedKey>) {
    Ligacoes::de(doc, entidades, inst).estado(reps, valor)
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
