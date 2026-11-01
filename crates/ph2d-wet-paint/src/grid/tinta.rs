//! O campo PREGUIÇOSO [`Grid::tinta`] (F4-2 E3, ADR-0197): o estado da lei ÚNICA da tinta por célula —
//! as somas do filme e do aberto ([`Soma`]), o fundo curado `R_g` (o meio `m` vive nas somas), a abertura `o` e a altura
//! que já curou. VAZIO até [`crate::sim::Sim::tinta`] `== Pigmento` correr um passo na grade ([`garante`]):
//! com a lei velha a grade não paga um byte. Nada o escreve ainda (E4–E8); o histórico, os recortes, o
//! clone e a retoma já o levam. Doc: `docs/Painter/Arquitetura/F4_2_plano_wet_paint.md` §2.1.

use super::Grid;
use ph2d_paint_tinta::oklab::oklab_para_srgb;
use ph2d_paint_tinta::{Bandas, Optica, Soma, Tinta, camada};

/// A tinta de uma célula na lei nova (72 B: o meio vive nas somas).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CelulaTinta {
    /// A fatia MOLHADA (o filme): volume `vᶠ` e as somas pesadas por ele.
    pub filme: Soma,
    /// A fatia assentada mas ainda ABERTA (`vᵃ`), que o pincel ainda levanta pela abertura.
    pub aberto: Soma,
    /// A reflectância do fundo já CURADO por banda (`1` = o branco de referência).
    pub r_g: [f32; 3],
    /// O MESMO fundo curado sobre o PRETO (`0` sem fundo): com `r_g` dá a cobertura do que já fechou,
    /// `C = 1 − (R|branco − R|preto)` — sem ele o fecho de uma tinta opaca clara perdia o `α` (E8).
    pub r_g0: [f32; 3],
    /// A abertura `o ∈ [0,1]`: `1` = fresco; `o → 0` fecha o aberto sobre `r_g`.
    pub o: f32,
    /// O volume que já curou (o relevo não some no fecho).
    pub altura_fechada: f32,
}

impl CelulaTinta {
    /// A célula sem tinta: sem volume, sobre o branco, aberta.
    pub const NEUTRA: Self = Self {
        filme: Soma {
            v: 0.0,
            lab: [0.0; 3],
            t: 0.0,
            m: 0.0,
        },
        aberto: Soma {
            v: 0.0,
            lab: [0.0; 3],
            t: 0.0,
            m: 0.0,
        },
        r_g: [1.0; 3],
        r_g0: [0.0; 3],
        o: 1.0,
        altura_fechada: 0.0,
    };

    /// O meio `m ∈ [0,1]` da tinta ainda aberta, médio pelo volume de filme e aberto (vive nas somas).
    #[must_use]
    pub fn meio(&self) -> f32 {
        self.filme.junta(self.aberto).meio()
    }
}

/// Massa (a unidade de `susp`/`sett`) por µm de pigmento: `filme.v · M_UM == susp`, `aberto.v · M_UM == sett`.
/// Calibrada pela E6a (`docs/Painter/Arquitetura/E6a_deposito.md` §calibração): um dab de fábrica (massa 600)
/// cobre sobre branco o que `alpha_of_mass(600)` cobria.
pub const M_UM: f32 = 1374.0;

/// A fatia de `massa` (unidades de `susp`) da tinta sRGB `0..255` com a `transparencia` e o `meio` dados.
#[must_use]
pub fn soma_da_massa(cor_srgb_0_255: [f32; 3], transparencia: f32, meio: f32, massa: f32) -> Soma {
    let tinta = Tinta::de_srgb(cor_srgb_0_255.map(|c| c / 255.0), transparencia);
    Soma::de(tinta, (massa / M_UM).max(0.0)).com_meio(meio)
}

/// A cor sRGB `0..255` de uma fatia (o espelho em `susp_rgb`/`sett_rgb` para os leitores da lei velha).
#[must_use]
pub fn srgb_0_255(s: &Soma) -> Option<[f32; 3]> {
    let c = s.cor()?;
    Some(oklab_para_srgb(c.lab).map(|x| (x * 255.0).clamp(0.0, 255.0)))
}

/// A Transparency da tinta pousada na célula (`0` sem o campo do material).
fn transparencia(g: &Grid, i: usize) -> f32 {
    g.material
        .get(i)
        .map_or(0.0, |m| crate::material::peso(m.transparencia) as f32)
}

/// A célula `i` lida da lei velha: filme e aberto da massa e da cor de `susp`/`sett` (a tela não some ao trocar);
/// o meio é o Real Oil que a célula guarda.
#[must_use]
pub fn migra(g: &Grid, i: usize) -> CelulaTinta {
    let t = transparencia(g, i);
    let meio = super::oleo_celula::da_celula(&g.oleo, i).oleo;
    let fatia = |m: f32, rgb: [f32; 3]| {
        if m > 0.0 {
            soma_da_massa(rgb, t, meio, m)
        } else {
            Soma::default()
        }
    };
    CelulaTinta {
        filme: fatia(g.susp[i], g.susp_rgb[i]),
        aberto: fatia(g.sett[i], g.sett_rgb[i]),
        ..CelulaTinta::NEUTRA
    }
}

/// O campo; se ainda não existia, MIGRADO da tinta velha que a grade já tem ([`migra`]).
pub fn garante(g: &mut Grid) -> &mut [CelulaTinta] {
    if g.tinta.is_empty() {
        g.tinta = (0..g.cells).map(|i| migra(g, i)).collect();
    }
    &mut g.tinta
}

/// A PILHA da célula (filme e aberto com a óptica `o` sobre o fundo curado): `R` sobre branco e a cobertura média
/// `C = 1 − (R|branco − R|preto)` com o fundo `r_g`/`r_g0`.
#[must_use]
pub fn pilha(o: &Optica<3>, c: &CelulaTinta) -> (Bandas<3>, f32) {
    let rw = camada(o, &Bandas(c.r_g));
    let rb = camada(o, &Bandas(c.r_g0));
    let cob: [f32; 3] = core::array::from_fn(|b| 1.0 - (rw.0[b] - rb.0[b]));
    (rw, ((cob[0] + cob[1]) + cob[2]) / 3.0)
}

/// A cobertura FÍSICA da tinta na célula `i` (a da [`pilha`], no estado de agora): `0` sem tinta. Não é o `α` do
/// ecrã — esse é o da decomposição para o compositor ([`ph2d_paint_tinta::decompoe`]), `≥` esta.
#[must_use]
pub fn cobertura(g: &Grid, i: usize) -> f32 {
    let c = em(g, i);
    let molhado = g.film.get(i).is_some_and(|&w| w > 0.0);
    let o = pigmento::optica_celula(&c.filme, &c.aberto, g.pigmento.get(i), c.meio(), molhado);
    pilha(&o, &c).1
}

/// A tinta da célula `i`; [`CelulaTinta::NEUTRA`] sem o campo.
#[must_use]
pub fn em(g: &Grid, i: usize) -> CelulaTinta {
    g.tinta.get(i).copied().unwrap_or(CelulaTinta::NEUTRA)
}

/// Devolve a janela de um recorte. Chamada DEPOIS dos planos velhos: `linhas` vazio (a grade não tinha o campo)
/// ⇒ a janela é migrada deles; o campo que nasce aqui nasce migrado fora da janela.
pub fn restaura_janela(g: &mut Grid, janela: (i32, i32, i32, i32), linhas: &[CelulaTinta]) {
    if g.tinta.is_empty() {
        if linhas.is_empty() {
            return;
        }
        garante(g);
    }
    let (n, s) = (g.cells, g.s);
    super::altura::restaura_campo(&mut g.tinta, n, s, janela, linhas, CelulaTinta::NEUTRA);
    if linhas.is_empty() {
        let (x0, y0, x1, y1) = janela;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let i = y as usize * s + x as usize;
                g.tinta[i] = migra(g, i);
            }
        }
    }
}

/// O lado espectral do «Pigment mix» (E10b).
#[path = "pigmento.rs"]
pub mod pigmento;

#[cfg(test)]
#[path = "tinta_tests.rs"]
mod tests;
