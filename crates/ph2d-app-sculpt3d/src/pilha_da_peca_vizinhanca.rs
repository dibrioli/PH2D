//! ⭐⭐⭐⭐ **OS EFEITOS DE VIZINHANÇA NA PEÇA** (`docs/3D/30` §14, a W6) — filho
//! (`#[path]`) de [`super`] (`pilha_da_peca.rs`).
//!
//! Um desfoque, uma nitidez, um brilho ou umas sombras/realces leem os VIZINHOS
//! de cada amostra: a pilha compõe-se então INTEIRA, com a retícula da peça como
//! vizinhança ([`VizinhancaDaPeca`]) — o pedaço `a..b` de uma composição que lê
//! vizinhos não existe sozinho. A placa compõe a mesma coisa
//! (`composto_na_placa`); a CPU é a referência.

use std::sync::Arc;

use ph2d_mesh::Mesh;
use ph2d_mesh_colors::Tinta;
use ph2d_tool_painter::{AdjustWindow, LayerKind, Neighbourhood, composite_over};

use super::{PilhaDaPeca, dobra};
use crate::vizinhanca_da_peca::{Impressao, VizinhancaDaPeca};

/// O último recurso de uma pilha que lê vizinhos sem a vizinhança à mão: os
/// passa-baixos são a identidade. Inalcançável pelas portas
/// ([`PilhaDaPeca::garante_vizinhanca`] corre antes de toda composição que lê
/// vizinhos) — e um `debug_assert` di-lo.
struct SemVizinhos(AdjustWindow);

impl Neighbourhood for SemVizinhos {
    fn window(&self) -> AdjustWindow {
        self.0
    }
    fn blur4(&self, _: f32, _: &mut [[f32; 4]]) {}
    fn blur1(&self, _: f32, _: &mut [f32]) {}
    fn image_plane(&self) -> bool {
        false
    }
}

impl PilhaDaPeca {
    /// ⭐ **A pilha tem um efeito que lê os vizinhos?** — a mesma pergunta que o
    /// compositor 2D faz para não repartir em faixas
    /// (`AdjustmentKind::reads_the_image_layout`), visível ou não.
    #[must_use]
    pub(crate) fn le_a_vizinhanca(&self) -> bool {
        self.pilha.all_ids().any(|id| {
            matches!(
                self.pilha.get(id).map(|c| &c.kind),
                Some(LayerKind::Adjustment(a)) if a.kind.reads_the_image_layout()
            )
        })
    }

    /// ⭐⭐ **A vizinhança em dia com esta geometria** — refeita só quando a
    /// impressão muda (esculpir, outro degrau), largada quando a pilha deixa de
    /// ler vizinhos.
    pub(crate) fn garante_vizinhanca(&mut self, tinta: &Tinta, mesh: &Mesh) {
        if !self.le_a_vizinhanca() || tinta.amostras().len() != self.amostras {
            self.vizinhanca.0 = None;
            return;
        }
        let impressao = Impressao::de(tinta, mesh);
        if self
            .vizinhanca
            .0
            .as_ref()
            .is_some_and(|v| v.serve(&impressao))
        {
            return;
        }
        let v = VizinhancaDaPeca::nova(tinta, mesh, dobra(self.amostras));
        self.vizinhanca.0 = Some(Arc::new(v));
    }

    /// ⭐ **A base pintada de uma vez** — a fixtura da cena `=54` (uma peça já
    /// pintada, como um ficheiro aberto). `false` se `px` não tem `N` amostras.
    pub(crate) fn pinta_a_base(&mut self, px: &[[u8; 4]]) -> bool {
        let n = self.amostras;
        match self.base().and_then(|b| self.planos.get_mut(&b)) {
            Some(plano) if px.len() == n => {
                let relevo = plano.relevo.take();
                plano.escreve(px, relevo);
                true
            }
            _ => false,
        }
    }

    /// A vizinhança da peça, se a pilha a tem.
    #[must_use]
    pub(crate) fn vizinhanca(&self) -> Option<&VizinhancaDaPeca> {
        self.vizinhanca.0.as_deref()
    }

    /// A vizinhança partilhada (a placa guarda-a com o polinómio dela).
    #[must_use]
    pub(crate) fn vizinhanca_partilhada(&self) -> Option<Arc<VizinhancaDaPeca>> {
        self.vizinhanca.0.clone()
    }

    /// ⭐⭐⭐ **As amostras `inicio..fim` da peça composta INTEIRA sobre a
    /// retícula** — o caminho da [`Self::compor_faixa`] quando a pilha lê
    /// vizinhos.
    pub(super) fn faixa_da_superficie(&self, inicio: usize, fim: usize) -> Vec<u8> {
        let todo = self.com_vizinhos(|nb| composite_over(&self.pilha, self, nb));
        todo[inicio * 4..fim * 4].to_vec()
    }

    /// `f` com a vizinhança da peça — a cor e o relevo leem a MESMA.
    pub(super) fn com_vizinhos<R>(&self, f: impl FnOnce(&dyn Neighbourhood) -> R) -> R {
        match self.vizinhanca() {
            Some(v) => f(v),
            None => {
                debug_assert!(false, "uma pilha que lê vizinhos sem a vizinhança da peça");
                let (l, h) = dobra(self.amostras);
                f(&SemVizinhos(AdjustWindow::full(l, h)))
            }
        }
    }
}
