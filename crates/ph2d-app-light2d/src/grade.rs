//! **A grade de luz do quadro** — ancorada no mundo (G11), com dimensões que só dependem da
//! JANELA (o zoom não realoca).
//!
//! - O texel é uma potência de dois em metros: `texel = 2^⌈log₂(c0_px / ppm)⌉`. Dentro de uma
//!   oitava de zoom ele não muda (o texel no ecrã vai de `c0_px` a `2·c0_px`); só a passagem de
//!   oitava troca a rede.
//! - As dimensões cobrem a vista no PIOR caso da oitava (`c0_px` por texel) mais a margem de cada
//!   lado e um bloco de folga para a âncora — logo dependem só da janela e do degrau.
//! - A origem encaixa num múltiplo do espaçamento da cascata de TOPO (`2^(n−1)` texels): toda sonda
//!   de toda cascata cai nos MESMOS pontos do mundo de um quadro para o outro, e o pan não treme.

use kurbo::{Affine, Point};
use ph2d_light2d_gpu::{Config, Grade};

/// O degrau da luz no ecrã. ⚠️ Escolhido por MEDIÇÃO — a tabela está no handoff W3.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Degrau {
    /// O lado de um texel em píxeis no zoom mais fino da oitava.
    pub c0_px: f64,
    /// A margem fora da vista, em fração da vista, de CADA lado (emissores fora da tela iluminam
    /// dentro dela).
    pub margem: f64,
    /// Píxeis do alvo do Vello por texel da grade na camada do vetor que bloqueia (a C0 amostra-a
    /// bilinear em 8×8 subamostras por texel).
    pub vetor_px_por_texel: u32,
}

/// O degrau por omissão (ver a tabela da rodada no handoff W3).
pub const DEGRAU_PADRAO: Degrau = Degrau {
    c0_px: 2.0,
    margem: 0.25,
    vetor_px_por_texel: 2,
};

/// O afim píxel → mundo: o INVERSO do mundo → tela que o Vello usou, nas duas linhas que o
/// `receber.wgsl` lê.
#[must_use]
pub fn tela_para_mundo(mundo_para_tela: Affine) -> [[f64; 3]; 2] {
    let [a, b, c, d, e, f] = mundo_para_tela.inverse().as_coeffs();
    [[a, c, e], [b, d, f]]
}

/// A grade deste quadro para a vista `mundo_para_tela` sobre um alvo de `alvo_px` píxeis.
///
/// `None` com uma câmera degenerada (escala zero ou não finita).
#[must_use]
pub fn grade_do_quadro(
    mundo_para_tela: Affine,
    alvo_px: (u32, u32),
    degrau: Degrau,
) -> Option<Grade> {
    let ppm = mundo_para_tela.determinant().abs().sqrt();
    if !(ppm.is_finite() && ppm > 0.0) || alvo_px.0 == 0 || alvo_px.1 == 0 {
        return None;
    }
    let texel_m = (degrau.c0_px / ppm).log2().ceil().exp2();
    let lado = |px: u32| (f64::from(px) / degrau.c0_px * (1.0 + 2.0 * degrau.margem)).ceil() as u32;
    let base = (lado(alvo_px.0).max(1), lado(alvo_px.1).max(1));

    // As dimensões efetivas com um bloco de folga — o bloco depende das dimensões (o número de
    // cascatas), então itera até parar.
    let efetiva = |w: u32, h: u32| {
        let g = Grade {
            origem_m: [0.0, 0.0],
            texel_m,
            largura: w,
            altura: h,
        };
        let cfg = Config::padrao(g);
        (cfg.grade_efetiva(), 1u32 << (cfg.n_cascatas() - 1))
    };
    let mut bloco = efetiva(base.0, base.1).1;
    let mut g = loop {
        let (g, b) = efetiva(base.0 + bloco, base.1 + bloco);
        if b == bloco {
            break g;
        }
        bloco = b;
    };

    // A origem: centrada na vista e ENCAIXADA na rede do bloco (arredondada, logo a folga de um
    // bloco cobre os dois lados).
    let centro = mundo_para_tela.inverse()
        * Point::new(f64::from(alvo_px.0) / 2.0, f64::from(alvo_px.1) / 2.0);
    let passo = f64::from(bloco) * texel_m;
    let encaixa = |c: f64, n: u32| ((c - f64::from(n) * texel_m / 2.0) / passo).round() * passo;
    g.origem_m = [encaixa(centro.x, g.largura), encaixa(centro.y, g.altura)];
    Some(g)
}
