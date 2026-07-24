//! **Warp** — a família paramétrica do menu *Effect > Warp* do Illustrator.
//!
//! Cada estilo é um `R² → R²` sobre a posição NORMALIZADA na bounding box (`[-1, 1]²`) — o mesmo
//! espaço em que o diálogo Warp do Illustrator age, que deforma DENTRO da caixa da forma. São
//! distintos do [`crate::fx_warp`] (Pucker & Bloat), que é RADIAL e mexe âncora-contra-alça; estes
//! deformam a **silhueta inteira** segundo a posição 2D de cada ponto.
//!
//! # Reamostra por ARCO, como o Zig Zag — e é o que os torna baratos SEM rasgar
//!
//! Um warp é um campo não-afim; aplicá-lo só às âncoras faz o *lowpoly* que matou o Twist (a saga
//! no cabeçalho de [`crate::fx_warp`]). A cura é a MESMA do [`crate::fx_zigzag`]: reamostrar o
//! caminho denso por comprimento de arco — **união com as âncoras de entrada**, para as quinas
//! sobreviverem exatas — e deslocar cada amostra. As alças saem de **Catmull-Rom** sobre os pontos
//! DEFORMADOS, então a poligonal densa lê como uma curva lisa.
//!
//! ⚠️ **O Twist NÃO entra aqui** (ainda). Ele foi cortado de propósito (`fx_warp.rs`: rasgava mesmo
//! com subdivisão), e reabri-lo exige *render-and-look*, não um palpite — a reamostragem densa é
//! condição necessária, não prova de que basta.

use crate::arc_path::ArcPath;
use crate::effect::FxCtx;
use crate::{VecVertex, VertexKind};

/// Abaixo desta dobra (fração) o efeito é o ponto neutro.
const EPS: f64 = 1e-12;

/// Amostras uniformes por contorno. O recurso é o tempo de `cooked()` (chamado por vários
/// consumidores por frame), **linear nas amostras** — a mesma medição do [`crate::fx_zigzag`]
/// (256 amostras = 0,48 ms) põe 128 em ~0,24 ms. As âncoras de entrada entram por união além
/// destas, então um caminho já detalhado não perde nada.
const SAMPLES: usize = 128;

/// Teto de amostras por contorno — guarda contra um caminho patológico virar gigabytes.
const MAX_SAMPLES: usize = 4096;

/// Duas amostras mais próximas do que esta fração do caminho são o mesmo sítio (a segunda cai).
const MERGE_FRACTION: f64 = 1e-6;

/// **Os estilos de Warp.** A ordem é a das entradas no menu Add ([`crate::effect::PathEffect`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WarpStyle {
    /// Arqueia a forma — o meio sobe/desce em relação às pontas (o *Arc/Arch*).
    Arc,
    /// Abaula/aperta os quatro lados de uma vez (o *Bulge*, a almofada).
    Bulge,
    /// Uma onda senoidal única ao longo do eixo x (o *Wave*, o S).
    Wave,
    /// Lente esférica: o centro incha e a borda encolhe (o *Fisheye*).
    Fisheye,
    /// Inclina a forma da esquerda para a direita (o *Rise*, a perspectiva/cisalhamento).
    Rise,
}

impl WarpStyle {
    /// Todos os estilos, na ordem do menu — a fonte única do menu Add e do `from_kind`.
    pub const ALL: &'static [WarpStyle] = &[
        WarpStyle::Arc,
        WarpStyle::Bulge,
        WarpStyle::Wave,
        WarpStyle::Fisheye,
        WarpStyle::Rise,
    ];

    /// O nome que o painel mostra (o `label` do efeito sai daqui).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            WarpStyle::Arc => "Arc",
            WarpStyle::Bulge => "Bulge",
            WarpStyle::Wave => "Wave",
            WarpStyle::Fisheye => "Fisheye",
            WarpStyle::Rise => "Rise",
        }
    }

    /// O deslocamento em espaço NORMALIZADO `[-1, 1]²`. `b` é a dobra (Bend) em `[-1, 1]`.
    ///
    /// Cada fórmula é fechada e distinta; nenhuma é o gizmo (uma escala uniforme não é um efeito).
    fn deform(self, u: f64, v: f64, b: f64) -> (f64, f64) {
        match self {
            // O meio (`u≈0`) levanta `b`; as pontas (`|u|≈1`) ficam — o arco.
            WarpStyle::Arc => (u, v + b * (1.0 - u * u)),
            // Cada eixo escala pelo quão CENTRAL é o outro ⇒ os quatro lados abaúlam juntos
            // (`b>0`) ou apertam (`b<0`). O centro é o ponto fixo.
            WarpStyle::Bulge => (u * (1.0 + b * (1.0 - v * v)), v * (1.0 + b * (1.0 - u * u))),
            // Uma onda inteira ao longo de x: cada coluna sobe por `b·sin(π·u)`.
            WarpStyle::Wave => (u, v + b * (std::f64::consts::PI * u).sin()),
            // Escala radial que decai com o raio, NORMALIZADO pelo canto (`√2` na caixa
            // unitária): o centro estica por `1+b`, o CANTO fica, e as bordas abaúlam no meio —
            // um barril. ⚠️ Normalizar por `1` (não `√2`) deixava a silhueta inerte: o contorno
            // de um quadrado vive todo em `r ≥ 1`, então o clamp o congelava.
            WarpStyle::Fisheye => {
                let r = (u * u + v * v).sqrt();
                let s = 1.0 + b * (1.0 - (r / std::f64::consts::SQRT_2).min(1.0));
                (u * s, v * s)
            }
            // Cisalhamento vertical linear em x — a forma sobe da esquerda para a direita.
            WarpStyle::Rise => (u, v + b * u),
        }
    }
}

/// **Os parâmetros de um Warp.** Neutro em `bend == 0`.
///
/// O `style` é escolhido no menu Add (uma entrada por estilo) e não é um parâmetro vivo: o artista
/// pega *"Wave"*, não *"Warp e depois Wave"*. Trocar de estilo é remover e re-adicionar — o mesmo
/// custo que trocar de efeito.
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WarpSpec {
    pub style: WarpStyle,
    /// A dobra, em **percentagem** `-100..100` (o *Bend* do Illustrator).
    pub bend: f64,
}

impl WarpSpec {
    /// Um Warp novo do estilo `style`, no ponto NEUTRO.
    #[must_use]
    pub fn new(style: WarpStyle) -> Self {
        Self { style, bend: 0.0 }
    }

    /// Sem dobra não há deformação — e o neutro tem de ser no-op byte-idêntico (ADR-0132).
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.bend.abs() <= EPS
    }
}

impl Default for WarpSpec {
    fn default() -> Self {
        Self::new(WarpStyle::Arc)
    }
}

/// **Aplica o Warp a UM contorno.** Devolve `(verts, closed)` — deformar não abre nem fecha.
#[must_use]
pub fn warp_contour(
    verts: &[VecVertex],
    closed: bool,
    spec: &WarpSpec,
    ctx: &FxCtx,
) -> (Vec<VecVertex>, bool) {
    let (hx, hy) = (ctx.half[0], ctx.half[1]);
    // Neutro, poucos pontos, ou caixa degenerada nos dois eixos: nada a normalizar.
    if spec.is_neutral() || verts.len() < 2 || (hx <= EPS && hy <= EPS) {
        return (verts.to_vec(), closed);
    }
    let Some(ap) = ArcPath::from_contour(verts, closed) else {
        return (verts.to_vec(), closed);
    };
    let total = ap.total();
    if total <= EPS {
        return (verts.to_vec(), closed);
    }
    let b = spec.bend / 100.0;

    // A grade uniforme por arco + a UNIÃO com as âncoras de entrada (as quinas sobrevivem exatas),
    // o mesmo padrão do Zig Zag — sem ela, reamostrar seria amostrar mais grosso que a curva.
    let grid_n = if closed { SAMPLES } else { SAMPLES + 1 };
    #[allow(clippy::cast_precision_loss)]
    let step = total / SAMPLES as f64;
    let mut at: Vec<f64> = (0..grid_n)
        .map(|k| {
            #[allow(clippy::cast_precision_loss)]
            let s = k as f64 * step;
            s.min(total)
        })
        .collect();
    at.extend_from_slice(ap.anchor_arcs());
    if !closed {
        at.push(total);
    }
    at.sort_by(f64::total_cmp);
    let merge = total * MERGE_FRACTION;
    at.dedup_by(|a, b| (*a - *b).abs() <= merge);
    at.truncate(MAX_SAMPLES);
    if at.len() < 2 {
        return (verts.to_vec(), closed);
    }

    // Normaliza -> deforma -> desnormaliza. Um eixo degenerado (`half==0`) fica em 0 e o warp não
    // o toca (uma linha horizontal warpada só na vertical, o que é honesto).
    let warp = |p: [f64; 2]| -> [f64; 2] {
        let u = if hx > EPS {
            (p[0] - ctx.center[0]) / hx
        } else {
            0.0
        };
        let v = if hy > EPS {
            (p[1] - ctx.center[1]) / hy
        } else {
            0.0
        };
        let (u2, v2) = spec.style.deform(u, v, b);
        [u2.mul_add(hx, ctx.center[0]), v2.mul_add(hy, ctx.center[1])]
    };
    let pts: Vec<[f64; 2]> = at.iter().map(|&s| warp(ap.frame_at(s).0)).collect();
    (catmull_rom_verts(&pts, closed), closed)
}

/// Vértices lisos (C¹) que passam por `pts`, com alças de **Catmull-Rom** (tensão 0): a alça de
/// um ponto aponta ao longo do vetor entre os vizinhos, um sexto dele. É o que faz a poligonal
/// densa dos warps ler como curva em vez de facetas.
fn catmull_rom_verts(pts: &[[f64; 2]], closed: bool) -> Vec<VecVertex> {
    let n = pts.len();
    (0..n)
        .map(|k| {
            let prev = if k > 0 {
                pts[k - 1]
            } else if closed {
                pts[n - 1]
            } else {
                pts[0]
            };
            let next = if k + 1 < n {
                pts[k + 1]
            } else if closed {
                pts[0]
            } else {
                pts[n - 1]
            };
            let t = [(next[0] - prev[0]) / 6.0, (next[1] - prev[1]) / 6.0];
            VecVertex {
                anchor: pts[k],
                in_handle: [pts[k][0] - t[0], pts[k][1] - t[1]],
                out_handle: [pts[k][0] + t[0], pts[k][1] + t[1]],
                kind: VertexKind::Smooth,
                corner_radius: 0.0,
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "fx_warp_presets_tests.rs"]
mod tests;
