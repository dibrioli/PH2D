//! ⭐⭐⭐ **AS CAMADAS DE UM PLANO, COMO O ARQUIVO AS GUARDA** (documento v6,
//! `docs/3D/30` §3) — filho (`#[path]`) de [`super`] (`doc.rs`), irmão do
//! [`super::doc_tinta`], cujas duas formas (cruas · corridas, a menor ganha)
//! ele usa para o RGBA8 de cada camada.
//!
//! ⚠️ **O [`LayerStack`] viaja como o Painter 2D o serializa**: a forma dele é
//! da `ph2d-tool-painter`, e o postcard é POSICIONAL — um campo novo em
//! `Layer` muda o blob da escultura sem tocar no `SCULPT_DOC_VERSION`. O gate
//! `a_forma_da_pilha_gravada_e_pinada` transforma isso em vermelho.
//!
//! ⚠️ **O plano guarda-se SEM a cauda da dobra** — a dobra é a disposição da
//! memória para o compositor, não um dado da peça.

use ph2d_tool_painter::{LayerId, LayerKind, LayerStack};
use serde::{Deserialize, Serialize};

use super::doc_tinta::{self, Forma, RelevoDoc};
use crate::pilha_da_peca::{PilhaDaPeca, PlanoDaCamada};

/// O RGBA8 de uma camada, como o ficheiro o guarda.
pub(super) type CamadaDoc = Forma<[u8; 4]>;

/// Um plano de camada, como o ficheiro o guarda.
#[derive(Serialize, Deserialize)]
pub(super) struct PlanoDoc {
    pub(super) id: LayerId,
    pub(super) rgba: CamadaDoc,
    /// `None` = a camada nunca levou impasto, e custa UM byte.
    pub(super) relevo: Option<RelevoDoc>,
}

/// ⭐⭐⭐ **A pilha da peça, como o ficheiro a guarda.**
#[derive(Serialize, Deserialize)]
pub(super) struct CamadasDoc {
    pub(super) pilha: LayerStack,
    /// Na ordem das chaves (`LayerId` crescente): o ficheiro sai igual byte a
    /// byte para a mesma pilha.
    pub(super) planos: Vec<PlanoDoc>,
}

impl CamadasDoc {
    pub(super) fn da_pilha(p: &PilhaDaPeca) -> Self {
        let n = p.amostras();
        let planos = p
            .pilha()
            .all_ids()
            .filter_map(|id| p.plano(id).map(|pl| (id, pl)))
            .collect::<std::collections::BTreeMap<_, _>>()
            .into_iter()
            .map(|(id, pl)| PlanoDoc {
                id,
                rgba: doc_tinta::a_menor_forma(&como_pixeis(pl.rgba8(n))),
                relevo: pl.relevo().map(doc_tinta::a_menor_forma),
            })
            .collect();
        Self {
            pilha: p.pilha().clone(),
            planos,
        }
    }

    /// ⭐⭐ **A pilha de volta**, para um plano de `n` amostras — `None` quando
    /// o documento não a descreve: um plano com a contagem errada, um plano
    /// sem camada (ou o contrário), uma camada de um tipo que a peça não tem.
    ///
    /// ⚠️ As dimensões dos rasters vêm da DOBRA de `n` e não do ficheiro: são
    /// derivadas, como a topologia do plano.
    pub(super) fn pilha(self, n: usize) -> Option<PilhaDaPeca> {
        let (l, h) = crate::pilha_da_peca::dobra(n);
        let mut pilha = self.pilha;
        let ids: Vec<LayerId> = pilha.all_ids().collect();
        for id in ids {
            match pilha.get_mut(id).map(|c| &mut c.kind) {
                Some(LayerKind::Raster(r)) => (r.width, r.height) = (l, h),
                Some(LayerKind::Mask(m)) => (m.width, m.height) = (l, h),
                _ => {}
            }
        }
        let mut planos = std::collections::BTreeMap::new();
        for p in self.planos {
            let px = p.rgba.amostras(n)?;
            let relevo = match p.relevo {
                Some(r) => Some(r.amostras(n)?),
                None => None,
            };
            let mut plano = PlanoDaCamada::transparente(n);
            plano.escreve(&px, relevo);
            if planos.insert(p.id, plano).is_some() {
                return None;
            }
        }
        let p = PilhaDaPeca::de_partes(pilha, planos, n);
        p.sincronizada().then_some(p)
    }
}

/// As amostras de um plano RGBA8 como `[u8; 4]`.
fn como_pixeis(rgba8: &[u8]) -> Vec<[u8; 4]> {
    rgba8
        .chunks_exact(4)
        .map(|c| [c[0], c[1], c[2], c[3]])
        .collect()
}
