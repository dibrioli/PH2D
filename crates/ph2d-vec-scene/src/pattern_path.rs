//! **Pattern Along Path** — um motivo copiado, rígido, ao longo de um caminho-guia
//! (plano [`docs/Vector Module/23_plano_pattern_along_path.md`], item #11 da pesquisa 20).
//!
//! É o **irmão do texto em caminho**: onde o texto cavalga a curva com **glifos** rígidos, o
//! pattern a cavalga com **cópias de uma forma**. Um glifo é uma forma; o motor é o mesmo — o
//! [`crate::arc_path::ArcPath`] localiza o arco e o [`crate::text_path::GlyphFrame`] devolve o afim
//! **rígido** (rotação + translação) que leva o motivo ao mundo. Como o mapa é afim, ele **comuta**
//! com a avaliação de Bézier ([`crate::text_path`] §Rigidez): a curva que sai é a imagem EXATA da
//! que entra, e **não há refit** — que é o que separa este (rígido) do Envelope (ADR-0129), cujo
//! mapa não é afim e por isso precisa de `fit_to_bezpath`.
//!
//! # A cópia é ancorada pelo CENTRO
//!
//! Cada cópia ocupa uma fatia de arco de comprimento `avanço = largura(bbox) × spacing`, e o
//! caminho é amostrado no **centro** da fatia — a mesma convenção do `<textPath>` (o glifo roda em
//! torno de `x + avanço/2`), pela mesma razão: ancorar pela borda faria a forma girar **para fora**
//! da curva. O vértice do motivo entra em espaço local `[mx − cx, my − cy]` (o motivo recentrado no
//! seu bbox), e o `apply` do frame leva `+x` do motivo **ao longo** da guia e `+y` **à normal**.
//!
//! # O que este módulo NÃO faz (plano §3)
//!
//! Não **deforma** o motivo para dobrar ao longo da curva (isso é o Envelope) · não escala/roda por
//! progressão (isso é o Repeater, `fx_repeat`, dirigido por parâmetro e não pela guia) · não
//! **estica o avanço** para fechar exato no fim (a cauda pode sobrar; *fit* é refinamento próprio).

use crate::arc_path::ArcPath;
use crate::text_path::GlyphFrame;
use crate::{Contour, VecPath, VecVertex};

/// Avanço mínimo, em unidades de mundo. Um motivo degenerado (largura ~0, uma reta vertical) daria
/// avanço zero e uma divisão por zero na contagem de cópias; o piso o transforma numa tilagem
/// densa em vez de num laço infinito. **Não é um teto de produto** — é uma guarda de representação
/// (a contagem `total/avanço` tem de ser finita); o `spacing` é quem controla o espaçamento real.
const MIN_ADVANCE: f64 = 1e-3;

/// Teto de cópias emitidas — guarda de **memória / contagem de vértices** (`N × verts(motivo)`),
/// não um limite de produto. A perf real é medida no gate (`a_keystroke_recook_stays_under_the_kill`);
/// este número só existe para uma guia longuíssima com um motivo minúsculo não alocar sem fim.
const MAX_COPIES: usize = 4096;

/// Folga de encaixe, em **frações de fatia** (unidades de `k`), na fronteira "a cópia cabe exato".
///
/// O `total` de um contorno é MEDIDO (Gauss-Legendre), não exato — o círculo de quatro cúbicas
/// carrega ~1,4e-4 relativo. Numa guia cujo comprimento é múltiplo do avanço, a última cópia cabe
/// **exatamente**, e uma comparação estrita a deixaria dentro ou fora conforme o ruído da
/// quadratura. Esta folga (1% de uma fatia = invisível) absorve o ruído sem nunca admitir uma cópia
/// que transborde de verdade.
const FIT_EPS: f64 = 1e-2;

/// **Os parâmetros de um pattern-along.**
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PatternSpec {
    /// Arco onde a tilagem começa, em unidades de mundo ao longo da guia. É o irmão do
    /// `start_offset` do texto: correr este valor desliza o padrão inteiro pela curva.
    pub start_offset: f64,
    /// Multiplica a **largura** do bbox do motivo para dar o avanço por cópia. `1.0` encaixa as
    /// cópias borda-a-borda; `0.5` sobrepõe metade; `2.0` deixa um vão de uma cópia.
    pub spacing: f64,
    /// Desvio da guia ao longo da **normal**, positivo para a ESQUERDA do sentido de marcha (a
    /// mesma convenção — e o mesmo sinal — do `dy` do texto, `GlyphFrame::on_path`).
    pub offset: f64,
    /// Põe o padrão do **outro lado** da curva, a percorrê-la ao contrário.
    pub flip: bool,
}

impl Default for PatternSpec {
    fn default() -> Self {
        Self {
            start_offset: 0.0,
            spacing: 1.0,
            offset: 0.0,
            flip: false,
        }
    }
}

/// A largura do bbox do motivo (ao longo de x) e o **centro** do bbox.
///
/// Sobre âncoras **e alças**: o casco convexo dos pontos de controle limita a curva, então a caixa
/// que os inclui é um superconjunto que de fato contém o desenho — que é a medida certa para
/// encaixar cópias sem sobrepor. Uma caixa só das âncoras cortaria uma curva que estufa para fora
/// delas.
fn motif_bbox(motif: &VecPath) -> (f64, [f64; 2]) {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    let mut seen = false;
    for v in motif.verts_all() {
        for p in [v.anchor, v.in_handle, v.out_handle] {
            seen = true;
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
    }
    if !seen {
        return (0.0, [0.0, 0.0]);
    }
    (hi[0] - lo[0], [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5])
}

/// Um vértice do motivo levado ao mundo pelo referencial da cópia, RÍGIDO.
///
/// Âncora **e as duas alças** passam pelo mesmo afim — é isso que mantém a curva sendo a imagem da
/// curva. `corner_radius` é comprimento LOCAL e o afim é rotação + translação (sem escala), então
/// ele sobrevive intacto (a mesma razão do `map_vert` do Repeater).
fn map_vert(v: &VecVertex, frame: &GlyphFrame, center: [f64; 2]) -> VecVertex {
    let m = |p: [f64; 2]| frame.apply([p[0] - center[0], p[1] - center[1]]);
    VecVertex {
        anchor: m(v.anchor),
        in_handle: m(v.in_handle),
        out_handle: m(v.out_handle),
        kind: v.kind,
        corner_radius: v.corner_radius,
    }
}

/// **Tila o `motif` ao longo da `guide`**, devolvendo os contornos de TODAS as cópias.
///
/// Vazio se a guia é degenerada (`total <= 0`) ou nenhuma cópia cabe. A cópia `k` ocupa a fatia de
/// arco `[start + avanço·k, start + avanço·(k+1)]`, com o centro em `start + avanço·(k + ½)`; emitem-se
/// as cópias cuja **fatia inteira cabe** em `[0, total]` — nada transborda as pontas (a cauda pode
/// sobrar, plano 23 §3, e é isso que distingue de deixar meia-cópia pendurada no fim).
///
/// ⚠️ Numa **cúspide** o [`GlyphFrame::on_path`] devolve `None` (não há tangente) e a cópia é
/// **pulada** — a mesma escolha do texto: sem direção, inventar um referencial poria a forma num
/// ângulo que ninguém autorou.
#[must_use]
pub fn pattern_along(motif: &VecPath, guide: &ArcPath, spec: &PatternSpec) -> Vec<Contour> {
    let total = guide.total();
    if total <= 0.0 {
        return Vec::new();
    }
    let (width, center) = motif_bbox(motif);
    let advance = (width * spec.spacing).max(MIN_ADVANCE);
    let inv = 1.0 / advance;

    // As cópias cuja FATIA `[lead_k, lead_k + avanço]` cabe em `[0, total]`, `lead_k = start + avanço·k`.
    // `k_lo` = primeiro `k` com `lead_k >= 0`; `k_hi` = último com `lead_k + avanço <= total`. Fechado
    // para o laço ser O(cópias) mesmo com um `start_offset` muito negativo (saltar as cópias antes do
    // começo num laço à parte seria O(k) desperdiçado). `FIT_EPS` absorve o ruído da medida de arco na
    // fronteira do encaixe exato.
    // Ambos são FINITOS: `total > 0` já foi testado e `advance >= MIN_ADVANCE`, então `inv` e os dois
    // são finitos; `k_lo` nunca é NaN (o `.max(0.0)` o garante). Um `<` cru basta e é claro.
    let k_lo = (-spec.start_offset * inv).ceil().max(0.0);
    let k_hi = ((total - spec.start_offset) * inv - 1.0 + FIT_EPS).floor();
    if k_hi < k_lo {
        return Vec::new();
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let count = ((k_hi - k_lo + 1.0).min(MAX_COPIES as f64)) as usize;

    let mut out = Vec::new();
    for j in 0..count {
        #[allow(clippy::cast_precision_loss)]
        let k = k_lo + j as f64;
        let s = spec.start_offset + advance * (k + 0.5);
        let Some(frame) = GlyphFrame::on_path(guide, s, spec.offset, spec.flip) else {
            continue; // cúspide: sem tangente, pula a cópia (como o texto pula o glifo)
        };
        for c in 0..motif.contour_count() {
            // `contour(c)` nunca é `None` para `c < contour_count()`.
            if let Some((verts, closed)) = motif.contour(c) {
                out.push(Contour {
                    verts: verts.iter().map(|v| map_vert(v, &frame, center)).collect(),
                    closed,
                });
            }
        }
    }
    out
}

#[cfg(test)]
#[path = "pattern_path_tests.rs"]
mod tests;
