//! ⭐⭐⭐ **O FUNDO DA PILHA E O PLANO DA CPU ATRASADO** (`docs/3D/30` §13, W1b)
//! — filho (`#[path]`) de [`super`] (`pilha_da_peca.rs`).
//!
//! ⛔⛔ **O fundo é FIXADO quando a pilha nasce** (a cor por vértice daquele
//! instante) e viaja no ficheiro. Lia-se da cor por vértice VIVA, que a
//! recomposição reescreve com o composto: uma pilha translúcida andava a cada
//! recomposição (`50`, `37`, `27` degraus de sRGB8 — gate
//! `uma_pilha_translucida_recomposta_fica`).
//!
//! ⭐ **O plano da CPU pode ficar ATRÁS da pilha** enquanto a placa compõe
//! (o arrasto do painel): o prefixo dos vértices fica sempre em dia
//! ([`PilhaDaPeca::por_vertice`]), o resto só quando alguém o pede inteiro
//! ([`PilhaDaPeca::em_dia`]).

use ph2d_mesh::Mesh;
use ph2d_mesh_colors::Tinta;

use super::{PilhaDaPeca, achata};

/// O plano da CPU está atrás da pilha? Estado da SESSÃO: duas pilhas iguais
/// são iguais com o plano delas em dia ou não (`PartialEq`).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Atraso(bool);

impl PartialEq for Atraso {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl PilhaDaPeca {
    /// O fundo, por vértice.
    #[must_use]
    pub(crate) fn fundo(&self) -> &[[f32; 3]] {
        &self.fundo
    }

    /// ⭐⭐ **O fundo amostra a amostra**: a semente da cor por vértice FIXADA
    /// (a mesma porta de onde um plano nasce, `Tinta::semeada`).
    #[must_use]
    pub(crate) fn fundo_semeado(&self, mesh: &Mesh, nivel: u8) -> Vec<[f32; 3]> {
        if self.fundo.len() != mesh.vert_count() {
            debug_assert!(false, "o fundo é de outra malha");
            return crate::tinta_da_peca::semente(mesh, nivel)
                .amostras()
                .to_vec();
        }
        let faces = || mesh.faces().iter().map(ph2d_mesh::Face::verts);
        Tinta::semeada(&self.fundo, faces(), nivel)
            .amostras()
            .to_vec()
    }

    /// ⭐⭐⭐ **A cor da peça nos VÉRTICES** — o prefixo `0..V` do plano (as
    /// amostras dos vértices vêm primeiro) composto sozinho: ao bit igual ao
    /// mesmo pedaço de [`Self::pinta_tinta`], a uma fracção do preço.
    #[must_use]
    pub(crate) fn por_vertice(&self) -> Vec<[f32; 3]> {
        let v = self.fundo.len().min(self.amostras);
        let mut out = vec![[0.0; 3]; v];
        achata(&self.compor_faixa(0, v), |i| self.fundo[i], &mut out);
        out
    }

    /// ⭐⭐ **A placa compôs a pilha e o plano da CPU ficou para trás** — menos
    /// o prefixo dos vértices, que se escreve aqui.
    ///
    /// ⚠️ O RELEVO não se toca: até à W4 só a base o leva, e nenhuma operação
    /// que chega aqui (painel, balde, desfazer deles) o muda — copiá-lo era
    /// `24 MB` por passo a `64x`.
    ///
    /// ⚠️ Com um efeito de VIZINHANÇA na pilha o prefixo dos vértices só existe
    /// com a peça inteira composta: ele fica atrás COM o resto (e igual à cor
    /// por vértice, que só o acompanha — `tinta_da_peca::pilha::em_dia`).
    pub(crate) fn atrasa(&mut self, tinta: &mut Tinta) {
        if self.le_a_vizinhanca() {
            self.cpu = Atraso(true);
            return;
        }
        let v = self.por_vertice();
        tinta.amostras_mut()[..v.len()].copy_from_slice(&v);
        self.cpu = Atraso(true);
    }

    /// O plano da CPU está atrás da pilha?
    #[must_use]
    pub(crate) fn atrasada(&self) -> bool {
        self.cpu.0
    }

    /// ⭐⭐⭐ **O plano da CPU em dia** — a composição inteira, a REFERÊNCIA
    /// (assar, exportar, doar ao 2D). Nada a fazer se ele já está.
    pub(crate) fn em_dia(&mut self, tinta: &mut Tinta, mesh: &Mesh) {
        if !self.cpu.0 {
            return;
        }
        let nivel = tinta.nivel();
        self.garante_vizinhanca(tinta, mesh);
        self.pinta_tinta(tinta, || self.fundo_semeado(mesh, nivel));
        self.cpu = Atraso(false);
    }
}
