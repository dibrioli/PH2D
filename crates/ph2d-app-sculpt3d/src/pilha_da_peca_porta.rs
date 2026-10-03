//! ⭐⭐⭐ **O PAINEL SOBRE A PILHA** (`docs/3D/30` §4 — a W3) — filho (`#[path]`)
//! de [`super`]: lá *as operações que criam e apagam camadas*, aqui *o metadado
//! de uma vez* e *a troca estrutural que o desfazer aplica*.
//!
//! O painel de Layers do Painter calcula o metadado novo sobre o espelho da
//! pilha (`ph2d_tool_painter::PieceLayerOp::Metadata`) e manda-o inteiro:
//! [`PilhaDaPeca::troca_metadado`] aceita-o só se a ESTRUTURA é a mesma — o que
//! tem planos passa pelas operações, que os levam.

use std::mem::discriminant;

use super::*;

/// ⭐⭐ **O que um passo de desfazer do painel guarda**: a pilha (metadado) de
/// antes e os planos que a operação TIROU (ou trocou). Aplicá-la devolve a
/// inversa — a mesma forma, nas duas direcções.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TrocaDaPilha {
    pilha: LayerStack,
    planos: BTreeMap<LayerId, PlanoDaCamada>,
}

impl TrocaDaPilha {
    /// O passo de uma operação: o metadado de antes e os planos que ela tirou
    /// (vazio quando só acrescentou ou mudou metadado — o que nasceu sai pela
    /// regra da troca: plano sem camada vai para a inversa).
    pub(crate) fn de(antes: LayerStack, tirados: BTreeMap<LayerId, PlanoDaCamada>) -> Self {
        Self {
            pilha: antes,
            planos: tirados,
        }
    }

    /// Quanto ela segura — a régua do tecto da história.
    pub(crate) fn bytes(&self) -> usize {
        self.planos
            .values()
            .map(|p| {
                p.rgba8.capacity()
                    + p.relevo
                        .as_ref()
                        .map_or(0, |r| r.capacity() * size_of::<[f32; 2]>())
            })
            .sum()
    }
}

/// O que o painel da peça não pode mudar pelo metadado: a lista de camadas, o
/// tipo de cada uma, as máscaras, o relevo, e o que a peça ainda não oferece
/// (bloqueio de alfa, referência, a profundidade do impasto).
fn mesma_estrutura(a: &LayerStack, b: &LayerStack) -> bool {
    let ids = |s: &LayerStack| {
        let mut v: Vec<LayerId> = s.all_ids().collect();
        v.sort_unstable();
        v
    };
    ids(a) == ids(b)
        && a.all_ids().all(|id| match (a.get(id), b.get(id)) {
            (Some(x), Some(y)) => {
                let tipo = match (&x.kind, &y.kind) {
                    (LayerKind::Adjustment(p), LayerKind::Adjustment(q)) => p.kind == q.kind,
                    (LayerKind::Raster(p), LayerKind::Raster(q)) => {
                        (p.width, p.height) == (q.width, q.height)
                    }
                    (LayerKind::Mask(p), LayerKind::Mask(q)) => {
                        (p.width, p.height) == (q.width, q.height)
                    }
                    (p, q) => discriminant(p) == discriminant(q),
                };
                tipo && x.mask == y.mask
                    && x.has_relief == y.has_relief
                    && x.alpha_locked == y.alpha_locked
                    && x.is_reference == y.is_reference
                    && x.impasto_depth.to_bits() == y.impasto_depth.to_bits()
                    && x.impasto_composite == y.impasto_composite
            }
            _ => false,
        })
}

impl PilhaDaPeca {
    /// Nenhum traço está a pintar a pilha — a condição de toda operação.
    pub(super) fn livre(&self) -> Result<(), RecusaDaPilha> {
        if self.em_traco.is_some() {
            return Err(RecusaDaPilha::TracoAberto);
        }
        Ok(())
    }

    /// ⭐⭐⭐ **O metadado novo de uma vez** (modo, opacidade, visibilidade,
    /// recorte, máscara invertida, activa, ordem, parâmetros) — devolve o de
    /// antes. Recusa o que mude a estrutura, ou tire a BASE do fundo.
    pub(crate) fn troca_metadado(&mut self, nova: LayerStack) -> Result<LayerStack, RecusaDaPilha> {
        self.livre()?;
        if !mesma_estrutura(&self.pilha, &nova) {
            return Err(RecusaDaPilha::MudaAEstrutura);
        }
        if nova.root().last() != self.pilha.root().last() {
            return Err(RecusaDaPilha::ABase);
        }
        let antiga = std::mem::replace(&mut self.pilha, nova);
        if !self.sincronizada() {
            self.pilha = antiga;
            return Err(RecusaDaPilha::MudaAEstrutura);
        }
        Ok(antiga)
    }

    /// ⭐⭐⭐ **Aplica um passo do painel e devolve a inversa** — instala a
    /// pilha guardada e os planos dela; o plano que fica sem camada vai para a
    /// inversa, com o que foi trocado. `None` (e nada muda) com um traço aberto
    /// ou planos de outro tamanho.
    pub(crate) fn troca_estrutura(&mut self, t: TrocaDaPilha) -> Option<TrocaDaPilha> {
        if self.livre().is_err() {
            return None;
        }
        let (l, h) = dobra(self.amostras);
        let bytes = l as usize * h as usize * 4;
        if t.planos.values().any(|p| p.rgba8.len() != bytes) {
            return None;
        }
        let pilha = std::mem::replace(&mut self.pilha, t.pilha);
        let mut inversa = BTreeMap::new();
        for (id, p) in t.planos {
            if let Some(velho) = self.planos.insert(id, p) {
                inversa.insert(id, velho);
            }
        }
        let vivas: Vec<LayerId> = self.pilha.all_ids().collect();
        let (vivos, sem_camada): (BTreeMap<_, _>, BTreeMap<_, _>) =
            std::mem::take(&mut self.planos)
                .into_iter()
                .partition(|(k, _)| vivas.contains(k));
        self.planos = vivos;
        inversa.extend(sem_camada);
        debug_assert!(self.sincronizada(), "a troca deixou a pilha sem par");
        Some(TrocaDaPilha {
            pilha,
            planos: inversa,
        })
    }
}

#[cfg(test)]
#[path = "pilha_da_peca_porta_tests.rs"]
mod tests;
