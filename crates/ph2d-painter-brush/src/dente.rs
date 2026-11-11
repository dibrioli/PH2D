//! ⭐ **O dente do papel** — o segundo fator de cobertura, preso à TELA, que o papel do documento põe no
//! traço do Digital POR CIMA do Grain do próprio pincel (dono 08/11: *«a pintura parecia chapada mesmo
//! com paper com relevo»*). Sem Grain, o papel continua a ser o Grain (`papel_como_grain`, ao byte) e o
//! dente fica `None`; com Grain (ou Grain Colors, que pinta a COR com o grão), o papel morde o ALFA aqui.
//!
//! O fator é exactamente o que o papel dá quando faz de Grain: a sua textura, à sua lei, à profundidade
//! `Tooth / PAPER_TOOTH_MAX`, ancorado à tela pela base de um grão sem Rake nem acaso.

use crate::texture::{TexDabBasis, TextureSettings};

/// O papel que morde a cobertura do dab (ver o módulo). Só procedural: um papel de IMAGEM não chega ao
/// núcleo por este slot (o núcleo só recebe a imagem do Grain).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dente {
    pub papel: TextureSettings,
    pub base: TexDabBasis,
    /// `0..1` — a profundidade do Grain que o papel teria (`paper_depth / PAPER_TOOTH_MAX`).
    pub profundidade: f32,
}

impl Dente {
    /// O dente do `papel` a `profundidade`; `None` quando não morderia nada (papel inativo, de imagem, ou
    /// profundidade 0) — a porta que deixa o traço sem papel ao byte.
    #[must_use]
    pub fn do_papel(papel: TextureSettings, profundidade: f32) -> Option<Self> {
        let profundidade = profundidade.clamp(0.0, 1.0);
        if !papel.is_active()
            || papel.kind == crate::texture::TextureKind::Image
            || profundidade <= 0.0
        {
            return None;
        }
        let base = crate::texture::dab_basis(
            &papel,
            &mut 0u64,
            [1.0, 1.0],
            crate::footprint::FootprintDeform::identity(),
        );
        Some(Self {
            papel,
            base,
            profundidade,
        })
    }

    /// O fator do dente no píxel `(px, py)` da tela, com a ponta `w` (os modos que lêem a ponta).
    #[inline]
    #[must_use]
    pub fn em(&self, w: f32, px: i64, py: i64) -> f32 {
        self.papel
            .lei
            .factor(w, self.t(px, py), self.profundidade, 0.0)
    }

    /// O `t` do papel no píxel: a única amostra do dente.
    #[inline]
    fn t(&self, px: i64, py: i64) -> f32 {
        #[cfg(test)]
        AMOSTRAS.with(|c| c.set(c.get() + 1));
        crate::texture::sample(&self.papel, &self.base, px, py, [0.0, 0.0], 1.0, None)
    }

    /// [`Self::em`] lendo o `t` da memória `mem` ([`crate::texture::grao_cache::para_o_dente`], B26.A2): o mesmo
    /// `f32`, amostrado uma vez por píxel do traço em vez de uma vez por dab.
    #[inline]
    pub(crate) fn em_com(
        &self,
        mem: Option<&crate::texture::grao_cache::GraoCache>,
        w: f32,
        px: i64,
        py: i64,
    ) -> f32 {
        let t = match mem {
            Some(m) => m.em(px, py, || self.t(px, py)),
            None => self.t(px, py),
        };
        self.papel.lei.factor(w, t, self.profundidade, 0.0)
    }
}

#[cfg(test)]
thread_local! {
    /// Quantas vezes o papel do dente foi amostrado neste fio (o contador do gate B26.A2).
    pub(crate) static AMOSTRAS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// O fator do dente do `spec` no píxel `(px, py)` — `1` sem dente. A porta ÚNICA dos núcleos.
#[inline]
#[must_use]
pub fn dente_at(spec: &crate::BrushSpec, w: f32, px: i64, py: i64) -> f32 {
    spec.dente.as_ref().map_or(1.0, |d| d.em(w, px, py))
}

/// [`dente_at`] com a memória do `t` ([`Dente::em_com`]).
#[inline]
#[must_use]
pub(crate) fn dente_com(
    spec: &crate::BrushSpec,
    mem: Option<&crate::texture::grao_cache::GraoCache>,
    w: f32,
    px: i64,
    py: i64,
) -> f32 {
    spec.dente
        .as_ref()
        .map_or(1.0, |d| d.em_com(mem, w, px, py))
}

thread_local! {
    static SEM_DENTE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// ⚠️ **Medição, nunca produto** (como [`crate::ablate`]): `true` faz a ferramenta não encher o dente neste
/// fio — o traço de antes da cura, para a prova de vermelho e o custo A × B no MESMO processo.
#[doc(hidden)]
pub fn sonda_sem_dente(desligado: bool) {
    SEM_DENTE.with(|c| c.set(desligado));
}

/// Se a sonda desligou o dente neste fio ([`sonda_sem_dente`]). `false` em todo caminho de produto.
#[doc(hidden)]
#[must_use]
pub fn sonda_desligou_o_dente() -> bool {
    SEM_DENTE.with(std::cell::Cell::get)
}
