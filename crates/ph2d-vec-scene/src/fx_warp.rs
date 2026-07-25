//! **Pucker & Bloat** — as âncoras para um lado, a curva para o outro.
//!
//! O deformador radial do Illustrator: puxa as âncoras para dentro enquanto curva os segmentos
//! para fora (*bloat* — a flor), ou o inverso (*pucker* — a estrela de pontas).
//!
//! # ⚠️ O TWIST foi CORTADO daqui (2026-07-18), e a razão importa
//!
//! Ele viveu neste ficheiro e saiu. O sintoma que o Enio viu (*"como se torcesse um lowpoly"*)
//! era real e tinha causa clara — um campo não-afim amostrado só nas âncoras —, e construí a
//! subdivisão adaptativa que o resolve. **A subdivisão funcionava** (havia gate a provar que
//! partir não move a curva). O que não funcionava era o efeito.
//!
//! Quatro tentativas, cada uma verificada na folha de contacto: força a crescer com o raio,
//! força a decrescer, raio de referência pela média, raio pelo máximo, subdivisão seis vezes
//! mais fina. **Todas rasgavam** — sobre uma forma com quinas, qualquer queda radial cria um
//! diferencial enorme ao longo de UMA aresta e o canto chicoteia à volta do corpo.
//!
//! Isso deixou de ser um defeito de código e passou a ser um defeito do meu MODELO do efeito, e
//! eu não tenho referência que consiga verificar. Um item de menu que produz geometria rasgada é
//! pior do que um item que falta. Volta quando eu o souber especificar — a subdivisão e o gate
//! dela estão no histórico, em `fx_warp.rs` antes deste commit.
//!
//! # Este efeito NÃO subdivide, e isso não é esquecimento
//!
//! Ele não aproxima campo nenhum: é definido diretamente sobre os pontos de controlo (as âncoras
//! encolhem, as alças esticam), que é como a Adobe o define. Não há uma curva "certa" a ser
//! amostrada grosso — a curva que sai É a resposta.

use crate::VecVertex;
use crate::effect::FxCtx;
use crate::fx_falloff::Falloff;

/// Abaixo disto o efeito é o ponto neutro.
const EPS: f64 = 1e-12;

/// **Pucker & Bloat** — âncoras para um lado, curva para o outro.
#[derive(Copy, Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BloatSpec {
    /// Quanto, em **percentagem**. Positivo = *bloat* (âncoras para dentro, arestas a
    /// abaular para fora — a flor); negativo = *pucker* (âncoras para fora, arestas a afundar
    /// — a estrela de pontas).
    pub amount: f64,
}

impl BloatSpec {
    /// Sem quantidade não há deformação.
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.amount.abs() <= EPS
    }
}

/// **Aplica o Pucker & Bloat a um contorno.**
///
/// `falloff` (opcional) escala a força por-ponto: cada vértice é `lerp(original, deformado,
/// w(âncora))`, então `w = 0` deixa o vértice onde estava e `w = 1` é o efeito cheio. `None` é
/// byte-idêntico ao efeito sem modulação.
#[must_use]
pub fn bloat_contour(
    verts: &[VecVertex],
    closed: bool,
    spec: &BloatSpec,
    ctx: &FxCtx,
    falloff: Option<&Falloff>,
) -> (Vec<VecVertex>, bool) {
    if spec.is_neutral() {
        return (verts.to_vec(), closed);
    }
    let t = spec.amount / 100.0;
    // ⚠️ **DOIS fatores opostos, e é isso que faz o efeito existir.** A 1ª versão escalava
    // âncoras e alças pelo MESMO fator — o que é uma escala uniforme, e uma escala uniforme não
    // é um efeito: é o gizmo (Enio, 2026-07-18: *"só aumenta e reduz a escala do objeto"*).
    //
    // A definição da Adobe é literalmente um par: *"puxa as âncoras para dentro enquanto curva
    // os segmentos para fora (bloat), ou empurra as âncoras para fora enquanto curva os
    // segmentos para dentro (pucker)"*. Aqui isso são dois números: as âncoras escalam por
    // `1 − t` e as alças por `1 + t`.
    //
    // Num círculo, `t > 0` encolhe as âncoras e estica as alças ⇒ quatro pétalas. Num quadrado
    // (alças coladas às âncoras) as alças passam a apontar para FORA das âncoras ⇒ as arestas
    // abaúlam. Com `t < 0` inverte-se: as âncoras saltam para fora e as arestas afundam ⇒ a
    // estrela de pontas. Em `t = 0` os dois fatores são 1 e o resultado é byte-idêntico.
    let (ka, kh) = (1.0 - t, 1.0 + t);
    let scale = |p: [f64; 2], k: f64| -> [f64; 2] {
        [
            (p[0] - ctx.center[0]).mul_add(k, ctx.center[0]),
            (p[1] - ctx.center[1]).mul_add(k, ctx.center[1]),
        ]
    };
    // `lerp(original, deformado, w)` — em `w = 1` é o efeito cheio (`p_def`), em `w = 0` o ponto
    // fica onde estava. `w` é avaliado na ÂNCORA do vértice e vale para o vértice inteiro (âncora
    // e alças), que é a força que o Falloff descreve *naquele sítio*.
    let mix = |orig: [f64; 2], def: [f64; 2], w: f64| -> [f64; 2] {
        [
            (def[0] - orig[0]).mul_add(w, orig[0]),
            (def[1] - orig[1]).mul_add(w, orig[1]),
        ]
    };
    (
        verts
            .iter()
            .map(|v| {
                let w = falloff.map_or(1.0, |f| f.eval(v.anchor));
                VecVertex {
                    anchor: mix(v.anchor, scale(v.anchor, ka), w),
                    in_handle: mix(v.in_handle, scale(v.in_handle, kh), w),
                    out_handle: mix(v.out_handle, scale(v.out_handle, kh), w),
                    kind: v.kind,
                    // O raio de quina é um comprimento local ANCORADO na âncora, então segue o
                    // fator dela. Os dois fatores divergem, e escolher o das alças poria o raio a
                    // crescer enquanto a quina que ele arredonda encolhe.
                    corner_radius: v.corner_radius * ka.abs(),
                }
            })
            .collect(),
        closed,
    )
}

#[cfg(test)]
#[path = "fx_warp_tests.rs"]
mod tests;
