//! **Como uma marca da trajetória é COLORIDA e o que ela É** — a fita de velocidade e
//! os papéis dos grupos de desenho, extraídos de `motion_path_overlay` (que bateu o teto
//! de LOC do shell). O `marks()` monta os grupos; isto define do que eles são feitos.
//!
//! # A FITA DE VELOCIDADE
//!
//! O fio da trajetória deixou de ser um traço subordinado: ele É a curva, e a cor dele é
//! o TEMPO do gesto — brasa (lento) → âmbar → ouro branco-quente (rápido). Os pontos
//! seguem dando a leitura por-quadro (o espaçamento); a fita reforça o ritmo de um
//! relance e dá beleza. Nasce de um GLOW largo e fraco por baixo + N bandas nítidas.

use ph2d_vector::BezPath;

/// Largura da fita nítida, px de tela. Um toque mais firme que o fio antigo (1,0): a
/// fita é a estrela agora, não um traço de fundo.
pub(crate) const RIBBON_PX: f64 = 1.6; // LITERAL-PX-OK: chrome de overlay, espessura de tela

/// Largura do glow que passa por baixo da fita — o halo macio do acabamento.
pub(crate) const RIBBON_GLOW_PX: f64 = 5.5; // LITERAL-PX-OK: chrome de overlay, espessura de tela

/// A cor/alfa do glow: âmbar difusa e fraca. É halo, não leitura — nunca compete.
pub(crate) const RIBBON_GLOW_RGBA: [f32; 4] = [1.0, 0.62, 0.22, 0.12]; // LITERAL-COLOR-OK: overlay de trajetoria

/// Quantas bandas de cor a fita usa. Um gradiente contínuo seria um traço por segmento
/// (milhares, um por amostra); 24 bandas leem como liso e mantêm o desenho barato.
pub(crate) const RIBBON_BANDS: usize = 24;

/// Abaixo de qual variação RELATIVA de velocidade a fita é pintada uniforme (banda do
/// meio). 1% fica ~200× acima do ruído de `f64` medido na projeção de tela (~5e-5) e
/// abaixo do que o olho separa em 24 bandas — um ritmo praticamente constante lê como
/// constante, em vez de amplificar ruído num gradiente falso.
pub(crate) const RIBBON_FLAT_REL: f64 = 0.01;

/// A rampa de calor (lento → rápido), 3 paradas. Fica na família QUENTE — adjacente à
/// âmbar da identidade da trajetória, e FORA do vocabulário frio do `physics_overlay`
/// (verde/ciano/violeta/magenta): brasa profunda · âmbar · ouro branco-quente.
const RAMP_SLOW: [f32; 4] = [0.80, 0.28, 0.10, 0.60]; // LITERAL-COLOR-OK: overlay de trajetoria
const RAMP_MID: [f32; 4] = [1.00, 0.72, 0.20, 0.82]; // LITERAL-COLOR-OK: overlay de trajetoria
const RAMP_FAST: [f32; 4] = [1.00, 0.95, 0.78, 0.98]; // LITERAL-COLOR-OK: overlay de trajetoria

/// A cor da fita para uma velocidade normalizada `u ∈ [0,1]` (0 = o trecho mais lento
/// DESTE caminho, 1 = o mais rápido). Dois lerps entre as três paradas — sem
/// transcendental. Monótona em cada canal, então "mais quente" é uma leitura confiável
/// (é o que o gate afirma).
pub(crate) fn heat_ramp(u: f32) -> [f32; 4] {
    let u = u.clamp(0.0, 1.0);
    let lerp = |a: [f32; 4], b: [f32; 4], t: f32| {
        [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
            a[3] + (b[3] - a[3]) * t,
        ]
    };
    if u < 0.5 {
        lerp(RAMP_SLOW, RAMP_MID, u * 2.0)
    } else {
        lerp(RAMP_MID, RAMP_FAST, (u - 0.5) * 2.0)
    }
}

/// O PAPEL de um grupo de desenho da trajetória — o que ele É, para o consumidor
/// (`draw`) e para os gates o acharem por FUNÇÃO em vez de por posição na lista (a fita
/// virou N bandas, então índice fixo mentiria).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MarkRole {
    /// O halo difuso sob a fita.
    Glow,
    /// Uma banda de cor da fita de velocidade (a forma da curva, colorida pelo ritmo).
    Ribbon,
    /// A linha de uma alça de tangente.
    TangentLine,
    /// Os pontos de tempo (losangos) — a leitura por-quadro.
    Dot,
    /// As âncoras (quadrados) — o que se agarra.
    Anchor,
    /// As pontas agarráveis das alças (círculos).
    TangentTip,
}

impl MarkRole {
    /// A CAMADA de pintura deste papel — menor pinta primeiro (por baixo). O empilhamento
    /// é uma propriedade do que a marca É, não da ordem em que foi empurrada: o glow é
    /// halo (fundo), a fita a forma, os pontos/âncoras a leitura, e a ponta da alça é o
    /// alvo mais fino, que não pode ser enterrado. `draw` ordena por isto, então uma
    /// marca empurrada fora de ordem ainda pinta na camada certa.
    pub(crate) fn layer(self) -> u8 {
        match self {
            MarkRole::Glow => 0,
            MarkRole::Ribbon => 1,
            MarkRole::TangentLine => 2,
            MarkRole::Dot => 3,
            MarkRole::Anchor => 4,
            MarkRole::TangentTip => 5,
        }
    }
}

/// Um grupo de desenho: um caminho, sua cor, sua espessura, e o PAPEL que cumpre.
#[derive(Clone)]
pub(crate) struct OverlayMark {
    pub path: BezPath,
    pub rgba: [f32; 4],
    pub width: f64,
    pub role: MarkRole,
}
