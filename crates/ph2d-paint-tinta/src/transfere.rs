//! Eq. (4): pigmento CONSERVADO entre pincel, filme, aberto e fechado.

use crate::bandas::Bandas;
use crate::lei::{Optica, camada, mistura};
use crate::pigmento::Pigmento;

/// Volume por área (µm de pigmento puro) de cada pigmento da paleta `P`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Volumes<const P: usize>(pub [f32; P]);

impl<const P: usize> Default for Volumes<P> {
    fn default() -> Self {
        Self([0.0; P])
    }
}

impl<const P: usize> Volumes<P> {
    /// Soma dos volumes, em ordem.
    #[must_use]
    pub fn total(&self) -> f32 {
        self.0.iter().fold(0.0, |a, v| a + v)
    }
}

/// Move a fracção `f ∈ [0,1]` de cada volume de `de` para `para`; devolve o que moveu.
/// O que chega é `antes − depois` lido de `de` (não `v·f`), para que o arredondamento não crie nem
/// apague pigmento além de 1 ulp; debug-assert de conservação por pigmento.
pub fn transfere<const P: usize>(de: &mut Volumes<P>, para: &mut Volumes<P>, f: f32) -> Volumes<P> {
    let f = f.clamp(0.0, 1.0);
    Volumes(core::array::from_fn(|i| {
        let (antes_de, antes_para) = (de.0[i], para.0[i]);
        de.0[i] = antes_de - antes_de * f;
        let movido = antes_de - de.0[i];
        para.0[i] = antes_para + movido;
        debug_assert!(de.0[i] >= 0.0 && para.0[i] >= 0.0, "volume negativo");
        debug_assert!(
            ((de.0[i] + para.0[i]) - (antes_de + antes_para)).abs()
                <= 4.0 * f32::EPSILON * (antes_de + antes_para),
            "transferência criou/perdeu pigmento"
        );
        movido
    }))
}

/// Pick-up (F2 eq. 4): o pincel recolhe `k_u · o · A_c` do pigmento ABERTO da tela.
pub fn pick_up<const P: usize>(
    tela: &mut Volumes<P>,
    pincel: &mut Volumes<P>,
    k_u: f32,
    abertura: f32,
    area_contacto: f32,
) -> Volumes<P> {
    transfere(tela, pincel, k_u * abertura * area_contacto)
}

/// Eq. (1) de uma célula: filme e aberto entram JUNTOS (assentar não muda o que se vê).
#[must_use]
pub fn optica_celula<const N: usize, const P: usize>(
    paleta: &[Pigmento<N>; P],
    filme: &Volumes<P>,
    aberto: &Volumes<P>,
    s_fator: f32,
) -> Optica<N> {
    let itens: [(&Pigmento<N>, f32); P] =
        core::array::from_fn(|i| (&paleta[i], filme.0[i] + aberto.0[i]));
    mistura(&itens, s_fator)
}

/// Fecho: o aberto cura e passa a ser fundo, `R_g ← R(𝒦^a, 𝒮^a, R_g)`, `v^a ← 0`. O que se via
/// antes do fecho (aberto sobre `R_g`) é bit a bit o novo `R_g`: é a mesma chamada.
pub fn fecha<const N: usize, const P: usize>(
    rg: &mut Bandas<N>,
    aberto: &mut Volumes<P>,
    paleta: &[Pigmento<N>; P],
    s_fator: f32,
) {
    let o = optica_celula(paleta, &Volumes::default(), aberto, s_fator);
    *rg = camada(&o, rg);
    *aberto = Volumes::default();
}
