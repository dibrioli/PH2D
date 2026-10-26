//! **Warp** — a família paramétrica do menu *Effect > Warp* do Illustrator.
//!
//! Cada estilo é um `R² → R²` sobre a posição NORMALIZADA na bounding box (`[-1, 1]²`) — o mesmo
//! espaço em que o diálogo Warp do Illustrator age, que deforma DENTRO da caixa da forma. São
//! distintos do [`crate::fx_warp`] (Pucker & Bloat), que é RADIAL e mexe âncora-contra-alça; estes
//! deformam a **silhueta inteira** segundo a posição 2D de cada ponto.
//!
//! # Três controles, como o diálogo real
//!
//! O diálogo Warp do Illustrator tem **três** sliders, e este motor tem os três: o **Bend** (a
//! dobra do estilo) e as duas distorções de **perspectiva** — *Horizontal* e *Vertical* — que dão
//! o *keystone* (uma borda alarga contra a oposta). As duas perspectivas leem o ponto JÁ dobrado,
//! então **compõem** com o estilo em vez de o preceder: um Arc com Horizontal é um arco em fuga,
//! não um arco OU uma fuga.
//!
//! # Reamostra por ARCO, como o Zig Zag — e é o que os torna baratos SEM rasgar
//!
//! Um warp é um campo não-afim; aplicá-lo só às âncoras faz o *lowpoly* que matou o Twist (a saga
//! no cabeçalho de [`crate::fx_warp`]). A cura é a MESMA do [`crate::fx_zigzag`]: reamostrar o
//! caminho denso por comprimento de arco — **união com as âncoras de entrada**, para as quinas
//! sobreviverem exatas — e deslocar cada amostra. As alças são a tangente TRANSPORTADA pelo campo,
//! de cada lado (quina fica quina), com o vão de cada lado. Esse esqueleto é [`resample_displace`],
//! partilhado com quem mais aproxima um campo não-afim.
//!
//! ⚠️ **O Twist voltou** — mora no módulo irmão [`crate::fx_twist`], e RIDA ESTE esqueleto
//! ([`resample_displace`]). Ele foi cortado em 2026-07-18 (`fx_warp.rs`: rasgava mesmo com uma
//! subdivisão *anterior* a este esqueleto), e a cerca dizia que reabri-lo exigia *render-and-look*,
//! não um palpite — a reamostragem densa é condição necessária, não prova de que basta. A prova é a
//! folha de contacto (`tests/fx_look.rs`) sobre uma forma COM QUINAS, o caso de falha documentado.

use crate::VecVertex;
use crate::arc_path::ArcPath;
use crate::effect::FxCtx;
use crate::fx_falloff::Falloff;
use crate::fx_resample;

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

/// O passo da diferença central do campo, em fração do arco: o erro de truncatura cai com `h²` e
/// o de arredondamento sobe com `1/h`; `1e-6` do arco fica abaixo dos dois a ~`1e-9` relativo.
const JACOBIAN_STEP: f64 = 1e-6;

/// **Os estilos de Warp vêm do CATÁLOGO ÚNICO** [`ph2d_warp_style::WarpStyle`] — a MESMA lista que
/// os presets do Envelope, para os dois não divergirem (Enio 2026-07-25). O efeito usa o estilo como
/// CAMPO ([`WarpStyle::deform`]); o Envelope, como gaiola (`WarpStyle::cage`). São 9: Arc, ArcUpper,
/// ArcLower, Bulge, Flag, Wave, Squeeze, Fisheye, Rise.
pub use ph2d_warp_style::WarpStyle;

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
    /// A distorção de perspectiva HORIZONTAL, em **percentagem** `-100..100` (o *Horizontal
    /// Distortion* do Illustrator): alarga a borda de CIMA contra a de baixo em função da altura
    /// — o *keystone*. **Apendada por último** (postcard é posicional).
    pub h_distort: f64,
    /// A distorção de perspectiva VERTICAL, em **percentagem** `-100..100` (o *Vertical
    /// Distortion*): o *keystone* do outro eixo, alongando a borda da DIREITA contra a esquerda.
    /// **Apendada por último**.
    pub v_distort: f64,
}

impl WarpSpec {
    /// Um Warp novo do estilo `style`, no ponto NEUTRO.
    #[must_use]
    pub fn new(style: WarpStyle) -> Self {
        Self {
            style,
            bend: 0.0,
            h_distort: 0.0,
            v_distort: 0.0,
        }
    }

    /// Sem dobra NEM distorção não há deformação — e o neutro tem de ser no-op byte-idêntico
    /// (ADR-0132), então os TRÊS têm de estar em zero (o `Cow::Borrowed` do `cooked()` depende).
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.bend.abs() <= EPS && self.h_distort.abs() <= EPS && self.v_distort.abs() <= EPS
    }
}

impl Default for WarpSpec {
    fn default() -> Self {
        Self::new(WarpStyle::Arc)
    }
}

/// **Aplica o Warp a UM contorno.** Devolve `(verts, closed)` — deformar não abre nem fecha.
///
/// `falloff` (opcional) escala a força por-amostra: cada amostra é `lerp(original, deformado,
/// w(original))`, avaliado na posição ANTES da dobra, então `w = 0` reconstrói a curva de entrada
/// (a região sem influência) e `w = 1` é a dobra cheia. `None` é byte-idêntico ao Warp sem campo.
#[must_use]
pub fn warp_contour(
    verts: &[VecVertex],
    closed: bool,
    spec: &WarpSpec,
    ctx: &FxCtx,
    falloff: Option<&Falloff>,
) -> (Vec<VecVertex>, bool) {
    let (hx, hy) = (ctx.half[0], ctx.half[1]);
    // Neutro, poucos pontos, ou caixa degenerada nos dois eixos: nada a normalizar.
    if spec.is_neutral() || verts.len() < 2 || (hx <= EPS && hy <= EPS) {
        return (verts.to_vec(), closed);
    }
    let b = spec.bend / 100.0;
    let (dh, dv) = (spec.h_distort / 100.0, spec.v_distort / 100.0);

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
        // As duas perspectivas (o *keystone*): a horizontal alarga uma borda vertical contra a
        // oposta em função da ALTURA; a vertical faz o inverso. Ambas leem o ponto JÁ dobrado
        // `(u2, v2)`, então compõem com o estilo — um Arc em fuga, não Arc-ou-fuga.
        let u3 = u2 * dh.mul_add(v2, 1.0);
        let v3 = v2 * dv.mul_add(u2, 1.0);
        [u3.mul_add(hx, ctx.center[0]), v3.mul_add(hy, ctx.center[1])]
    };
    match resample_displace(verts, closed, falloff, warp) {
        Some(v) => (v, closed),
        None => (verts.to_vec(), closed),
    }
}

/// **Reamostra um contorno denso por ARCO e desloca cada amostra** — o esqueleto partilhado pelos
/// deformadores que aproximam um campo NÃO-AFIM (Warp, Twist). Uma segunda cópia deste laço seria
/// duas respostas à mesma pergunta (*"como se reamostra um caminho sem o facetar?"*), e divergiriam
/// no primeiro fix.
///
/// A grade uniforme por arco ∪ as âncoras vem do [`crate::fx_resample`]. Cada amostra é
/// `lerp(original, deslocada, w(original))`, com o `w` do Falloff avaliado na posição ANTES do
/// deslocamento, então `w = 0` reconstrói a curva de entrada.
///
/// ⭐ **As alças são a tangente TRANSPORTADA pelo campo** (`J·t`, diferença central do mapa
/// inteiro, peso incluído), de cada lado da âncora, × o vão desse lado / 3. Era Catmull-Rom sobre
/// os pontos deslocados, e isso comia as quinas e fazia laços entre amostras coladas — os vincos
/// do Bevel (F33, `docs/UI_New_and_Simple/F33_plano.md`).
///
/// Devolve `None` quando não há geometria a reamostrar — o CHAMADOR devolve a entrada intacta.
/// Ele também verifica o seu próprio ponto neutro e a sua própria degenerescência ANTES de chamar;
/// aqui só a geometria decide.
pub(crate) fn resample_displace(
    verts: &[VecVertex],
    closed: bool,
    falloff: Option<&Falloff>,
    displace: impl Fn([f64; 2]) -> [f64; 2],
) -> Option<Vec<VecVertex>> {
    if verts.len() < 2 {
        return None;
    }
    let ap = ArcPath::from_contour(verts, closed)?;
    let total = ap.total();
    if total <= EPS {
        return None;
    }
    let grid_n = if closed { SAMPLES } else { SAMPLES + 1 };
    #[allow(clippy::cast_precision_loss)]
    let step = total / SAMPLES as f64;
    #[allow(clippy::cast_precision_loss)]
    let grade = (0..grid_n).map(|k| (k as f64 * step).min(total));
    let at = fx_resample::amostras(&ap, grade, total * MERGE_FRACTION, MAX_SAMPLES);
    if at.len() < 2 {
        return None;
    }
    // O mapa inteiro: `w` na posição ORIGINAL (antes do deslocamento) — é onde o Falloff diz quanta
    // força há *naquele sítio* da forma. `w = 0` deixa a amostra na curva de entrada.
    let mapa = |p: [f64; 2]| -> [f64; 2] {
        let d = displace(p);
        let w = falloff.map_or(1.0, |f| f.eval(p));
        [
            (d[0] - p[0]).mul_add(w, p[0]),
            (d[1] - p[1]).mul_add(w, p[1]),
        ]
    };
    let h = total * JACOBIAN_STEP;
    let transporta = |p: [f64; 2], t: [f64; 2]| -> [f64; 2] {
        let a = mapa([t[0].mul_add(h, p[0]), t[1].mul_add(h, p[1])]);
        let b = mapa([t[0].mul_add(-h, p[0]), t[1].mul_add(-h, p[1])]);
        [(a[0] - b[0]) / (2.0 * h), (a[1] - b[1]) / (2.0 * h)]
    };
    Some(
        at.iter()
            .enumerate()
            .map(|(k, a)| {
                let (p, t_chega, t_sai) = fx_resample::lados(&ap, a);
                fx_resample::vertice(
                    mapa(p),
                    transporta(p, t_chega),
                    transporta(p, t_sai),
                    fx_resample::vaos(&at, k, closed, total),
                )
            })
            .collect(),
    )
}

#[cfg(test)]
#[path = "fx_warp_presets_tests.rs"]
mod tests;
