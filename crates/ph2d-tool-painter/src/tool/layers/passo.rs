//! **Toda edição de camada ou de ajuste é UM passo de undo por gesto e marca o documento** (missão 4K,
//! B05 do [BACKLOG](../../../../../docs/Painter/missao_4k/BACKLOG.md)). Gate: `passo_tests`.

use super::super::*;
use crate::undo::CoalesceKind;

/// A propriedade que um gesto contínuo arrasta — com a camada e o índice, a chave da coalescência.
pub(crate) mod prop {
    pub const SLIDER: u8 = 0;
    pub const MIXER: u8 = 1;
    pub const SELECTIVA: u8 = 2;
    pub const CURVA: u8 = 3;
    pub const GRADIENTE_POS: u8 = 4;
    pub const GRADIENTE_COR: u8 = 5;
    pub const OPACIDADE: u8 = 6;
    pub const IMPASTO: u8 = 7;
    pub const NOME: u8 = 8;
    pub const TEX_PARAM: u8 = 9;
    pub const TEX_TAMANHO: u8 = 10;
    pub const TEX_DESLOCA: u8 = 11;
    pub const TEX_RAMPA_POS: u8 = 12;
    pub const TEX_RAMPA_COR: u8 = 13;
}

/// Os quadros de um arrasto de `p`/`i` sobre `id` coalescem num passo; qualquer outro passo parte o run.
pub(crate) fn arrasto(id: RtLayerId, p: u8, i: usize) -> Option<CoalesceKind> {
    Some(CoalesceKind::Camada(
        id,
        p,
        u16::try_from(i).unwrap_or(u16::MAX),
    ))
}

impl PainterTool {
    /// Corre `muda` e, se a pilha mudou, fecha o passo: `None` = um passo próprio (clique), `Some` = o
    /// passo coalesce com o anterior do mesmo gesto (arrasto). Pilha igual = nenhum passo.
    pub(crate) fn passo_de_camada<R>(
        &mut self,
        gesto: Option<CoalesceKind>,
        muda: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let antes = self.abre_edicao_estrutural();
        let r = muda(self);
        if self.layers == antes.layers {
            return r;
        }
        match gesto {
            Some(k) => self.commit_structural_edit_coalesced(k, antes),
            None => self.commit_structural_edit(antes),
        }
        r
    }

    /// A porta única de um ajuste que mudou (as 12 dele): só os cortes ACIMA dele caem (a pista rápida do
    /// arrasto — o `invalidate_composite` refaria a pilha toda a cada quadro), o documento fica «não
    /// assado», o painel re-publica e o passo fecha (`gesto` como em [`Self::passo_de_camada`]).
    pub(crate) fn ajuste_mudou(
        &mut self,
        id: RtLayerId,
        antes: crate::undo::ModelSnapshot,
        gesto: Option<CoalesceKind>,
    ) {
        self.compositor_cache.invalidate_above(id, &self.layers);
        self.compositor_cache.recicla(self.composited.take());
        self.dirty_rect = None;
        self.adjustment_cache_pending = true;
        self.preview_dirty = true;
        self.edited_since_bind = true;
        self.layers_revision = self.layers_revision.wrapping_add(1);
        match gesto {
            Some(k) => self.commit_structural_edit_coalesced(k, antes),
            None => self.commit_structural_edit(antes),
        }
    }
}
