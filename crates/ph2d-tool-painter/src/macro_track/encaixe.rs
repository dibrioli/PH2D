//! **O encaixe de uma pintura tocada por cima** (W18, `docs/Macros/16_W18_plano.md` §2.7; decisões do dono
//! 23/10, 5 e 6): a gravação entra na imagem de baixo no TAMANHO ORIGINAL, onde o gizmo do objecto Macro
//! está. Só vale numa trilha cujo estado inicial guarda o tamanho gravado ([`super::inicial::tamanho_gravado`]);
//! as outras pintam a tela delas, como sempre.
//!
//! Uma porta para as posições ([`posicao`], que o tocador chama em todo passo com posição) e uma para o pincel ([`PainterTool::escala_do_pincel_da_macro`], lida onde o raio do traço se
//! resolve: `authored_spec` e os alcances que leem o raio cru).

use super::passo::abs;
use crate::tool::PainterTool;

/// Afim px-da-gravação → px-do-alvo: `x' = a·x + b·y + c`, `y' = d·x + e·y + f`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Encaixe {
    pub afim: [f64; 6],
}

impl Encaixe {
    /// A identidade em px (a gravação encostada ao canto de cima, à esquerda).
    pub const IDENTIDADE: Self = Self {
        afim: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
    };

    #[must_use]
    pub fn aplica(&self, [x, y]: [f64; 2]) -> [f64; 2] {
        let [a, b, c, d, e, f] = self.afim;
        [a * x + b * y + c, d * x + e * y + f]
    }

    /// Quanto o pincel cresce: a raiz do determinante (a escala de área, sem a rotação).
    #[must_use]
    pub fn escala_do_pincel(&self) -> f64 {
        let [a, b, _, d, e, _] = self.afim;
        (a * e - b * d).abs().sqrt()
    }
}

/// O encaixe vivo de um tool que toca por cima: o tamanho gravado e o encaixe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PorCima {
    pub(crate) gravado: (f64, f64),
    pub(crate) encaixe: Encaixe,
}

/// **A posição gravada (relativa) em px da `tela`**: por cima, pelo encaixe sobre o tamanho gravado;
/// senão, relativa à tela. A porta única das posições do tocador.
pub(crate) fn posicao(por_cima: Option<PorCima>, (w, h): (u32, u32), p: [f64; 2]) -> [f32; 2] {
    match por_cima {
        Some(PorCima { gravado, encaixe }) => {
            let [x, y] = abs(p, gravado.0, gravado.1);
            let [x, y] = encaixe.aplica([f64::from(x), f64::from(y)]);
            [x as f32, y as f32]
        }
        None => abs(p, f64::from(w), f64::from(h)),
    }
}

impl PainterTool {
    /// O factor do raio do pincel (`1` fora de uma reprodução por cima).
    #[must_use]
    pub(crate) fn escala_do_pincel_da_macro(&self) -> f32 {
        self.macro_por_cima
            .map_or(1.0, |m| m.encaixe.escala_do_pincel() as f32)
    }

    /// O raio do pincel que o traço pinta (o do painel × [`Self::escala_do_pincel_da_macro`]).
    #[must_use]
    pub(crate) fn raio_do_pincel(&self) -> f32 {
        self.brush_size_px() * self.escala_do_pincel_da_macro()
    }

    /// Põe o encaixe (`None` = a identidade) — só num tool que toca por cima.
    pub(crate) fn poe_encaixe_da_macro(&mut self, e: Option<Encaixe>) {
        if let Some(m) = &mut self.macro_por_cima {
            m.encaixe = e.unwrap_or(Encaixe::IDENTIDADE);
        }
    }
}
