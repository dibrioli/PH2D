//! **A matemática dos oito pincéis** — e as constantes que fazem o Reshape "sentir
//! como o Grease Pencil".
//!
//! Cada constante aqui tem FONTE no `sculpt_*.cc` da referência (5.2). Nenhuma foi
//! calibrada no olho: quando um número parece arbitrário, o comentário diz de onde
//! ele veio e o que ele significa. (`docs/Flip/02 §7`.)
//!
//! **Unidades.** O GP esculpe em espaço de TELA e converte o delta para o objeto no
//! fim. Em 2D-ortográfico a projeção é uma **similaridade** — escala uniforme, sem
//! perspectiva — então distância, direção e ângulo são iguais nos dois espaços, e
//! tudo aqui roda em espaço LOCAL. **Uma** constante escapa: a amplitude do
//! [`randomize`], que no GP é literalmente "pixels de tela"; ela é convertida com o
//! `px_to_local` (ver lá).

use crate::blur::{self, Ends};
use crate::{InputSample, ReshapeKind, ReshapeParams, influence};
use ph2d_core::Vec2;
use ph2d_flip::FlipStroke;

/// Iterações do kernel do Smooth. **Hard-coded em 2 no GP** (`sculpt_smooth.cc:124`,
/// `const int iterations = 2`) — não é um parâmetro do brush lá, e não é aqui.
const SMOOTH_ITERATIONS: u32 = 2;

/// O divisor do Pinch (`sculpt_pinch.cc`: `influence * influence / 25.0f`).
/// Quadrático **e** dividido por 25: no máximo 4% de aproximação por amostra — o
/// aperto é deliberadamente lento e "cremoso". Mexer nisto é mexer na sensação.
const PINCH_DIVISOR: f32 = 25.0;

/// O ângulo do Twist por amostra, em radianos: **1 grau** (`sculpt_twist.cc`:
/// `DEG2RADF(invert ? -1.0f : 1.0f) * influence`).
const TWIST_RAD_PER_SAMPLE: f32 = std::f32::consts::PI / 180.0;

/// O passo do Strength (`sculpt_strength.cc`: *"Brush influence mapped to opacity by
/// a factor of 0.125"*).
const STRENGTH_STEP: f32 = 0.125;

/// O passo do Thickness, em **px de tela de LARGURA** por amostra.
///
/// O GP soma `influence · 0.001` ao **raio**, em unidades de mundo
/// (`sculpt_thickness.cc`: *"Factor 1/1000 is used to map arbitrary influence value
/// to a sensible radius"*), onde o raio default é `0.01` — ou seja, **10% do raio
/// default por amostra**.
///
/// A nossa largura é o **diâmetro em px de TELA** (pincel absoluto, Enio 2026-07-11)
/// e o default é 6 px. Preservando a mesma razão — 10% do default por amostra — o
/// passo é `0.6` px de largura. É a MESMA sensação, traduzida para a nossa unidade;
/// copiar o `0.001` cru daria um pincel que não faz nada visível (0,001 px!).
const THICKNESS_STEP_PX: f32 = 0.6;

/// Sinal do gesto: `-1` com Ctrl (nos pincéis que têm direção), `+1` sem.
fn sign(p: &ReshapeParams) -> f32 {
    if p.invert { -1.0 } else { 1.0 }
}

/// **Smooth** — alisa, com a influência como peso de mistura.
///
/// O kernel roda sobre o traço INTEIRO e a máscara entra só na mistura (o vizinho de
/// um ponto influenciado pode estar fora do alcance do pincel — ver `blur`). As
/// pontas ficam ancoradas: alisar a ponta a puxa para dentro, e o traço encurta.
fn smooth(pts: &mut [Vec2], p: &ReshapeParams, s: &InputSample, closed: bool) -> bool {
    let pos = pts.to_vec();
    let inf = |i: usize| influence(p, s, pos[i]);
    if !pos.iter().any(|&pt| influence(p, s, pt) > 0.0) {
        return false;
    }
    let out = blur::binomial(&pos, SMOOTH_ITERATIONS, &inf, Ends::Anchored, closed);
    let mut changed = false;
    for (dst, src) in pts.iter_mut().zip(out) {
        if *dst != src {
            *dst = src;
            changed = true;
        }
    }
    changed
}

/// **Push** — empurra na direção do movimento do cursor
/// (`sculpt_push.cc`: `positions += mouse_delta * influence`).
fn push(pts: &mut [Vec2], p: &ReshapeParams, s: &InputSample) -> bool {
    let delta = s.delta;
    if delta.x == 0.0 && delta.y == 0.0 {
        return false; // parado: o Push não faz nada (a dose é o MOVIMENTO)
    }
    let mut changed = false;
    for pt in pts.iter_mut() {
        let inf = influence(p, s, *pt);
        if inf > 0.0 {
            *pt += delta * inf;
            changed = true;
        }
    }
    changed
}

/// **Grab** — carrega o trecho agarrado no pen-down.
///
/// Não recalcula influência nenhuma: os pesos vieram congelados (`Session::begin`).
/// É essa a diferença de UX para o Push — o Push *varre* os pontos por onde passa; o
/// Grab *segura* os que estavam sob o pincel quando você o pegou, e os leva junto,
/// mesmo que o cursor saia de perto deles.
pub(crate) fn grab(
    strokes: &mut [FlipStroke],
    frozen: &[(usize, Option<usize>, usize, f32)],
    delta: Vec2,
) -> bool {
    if delta.x == 0.0 && delta.y == 0.0 {
        return false;
    }
    let mut changed = false;
    for &(si, ring, pi, w) in frozen {
        let Some(st) = strokes.get_mut(si) else {
            continue;
        };
        // `ring = None` → o contorno; `Some(k)` → o k-ésimo buraco (que tem de ser
        // carregado junto, senão a rosquinha se abre).
        let slot = match ring {
            None => st.positions_mut().get_mut(pi),
            Some(k) => st.holes.get_mut(k).and_then(|h| h.get_mut(pi)),
        };
        if let Some(pt) = slot {
            *pt += delta * w;
            changed = true;
        }
    }
    changed
}

/// **Pinch** — aperta os pontos em direção ao cursor (com Ctrl: infla).
///
/// `sculpt_pinch.cc`: o fator é `influence²/25`, e o deslocamento é uma FRAÇÃO do
/// vetor até o cursor — quem já está perto anda pouco, quem está longe (mas dentro do
/// raio) anda mais. É por isso que o Pinch afina uma silhueta em vez de colapsá-la
/// num ponto.
fn pinch(pts: &mut [Vec2], p: &ReshapeParams, s: &InputSample) -> bool {
    let mut changed = false;
    let cursor = s.pos;
    for pt in pts.iter_mut() {
        let inf = influence(p, s, *pt);
        if inf <= 0.0 {
            continue;
        }
        let f = sign(p) * inf * inf / PINCH_DIVISOR;
        *pt += (cursor - *pt) * f;
        changed = true;
    }
    changed
}

/// Rotação de um ângulo **pequeno**, sem `sin`/`cos` (HR-5).
///
/// O Twist gira no máximo **1 grau por amostra** (`|θ| ≤ 0,01745 rad`). Nessa faixa,
/// a série de Taylor truncada é exata para `f32`: o 1º termo omitido do seno é
/// `θ⁵/120 ≈ 1,3e-11` e o do cosseno é `θ⁶/720 ≈ 3e-14` — ambos ordens de grandeza
/// abaixo do épsilon relativo do `f32` (1,2e-7). E, ao contrário da `libm`, um
/// polinômio é **bit-idêntico entre plataformas** — que é a razão de o HR-5 existir
/// (replay-hash é contrato do projeto).
fn rotate_small(v: Vec2, theta: f32) -> Vec2 {
    let t2 = theta * theta;
    let sin = theta * (1.0 - t2 / 6.0 + t2 * t2 / 120.0);
    let cos = 1.0 - t2 / 2.0 + t2 * t2 / 24.0;
    Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}

/// **Twist** — torce rigidamente ao redor do cursor (com Ctrl: para o outro lado).
///
/// `sculpt_twist.cc`: `angle = ±1° · influence`, e o ponto anda pela DIFERENÇA entre
/// o raio girado e o raio original — uma rotação rígida do trecho ao redor do cursor,
/// não um arrasto tangencial.
fn twist(pts: &mut [Vec2], p: &ReshapeParams, s: &InputSample) -> bool {
    let mut changed = false;
    let cursor = s.pos;
    for pt in pts.iter_mut() {
        let inf = influence(p, s, *pt);
        if inf <= 0.0 {
            continue;
        }
        let theta = sign(p) * TWIST_RAD_PER_SAMPLE * inf;
        let radial = *pt - cursor;
        *pt += rotate_small(radial, theta) - radial;
        changed = true;
    }
    changed
}

/// **Thickness** — engrossa (com Ctrl: afina), **aditivo, nunca proporcional**.
///
/// `sculpt_thickness.cc`: `radius = max(radius ± influence·k, 0)`. Aditivo é
/// deliberado: um passo proporcional nunca sairia do zero (um ponto de largura 0 ficaria 0
/// para sempre) e engrossaria o grosso mais que o fino, exagerando a diferença em vez
/// de nivelá-la.
pub(crate) fn thickness(st: &mut FlipStroke, p: &ReshapeParams, s: &InputSample) -> bool {
    let pos = st.positions().to_vec();
    let mut changed = false;
    for (i, w) in st.widths_mut().iter_mut().enumerate() {
        let inf = influence(p, s, pos[i]);
        if inf <= 0.0 {
            continue;
        }
        let next = (*w + sign(p) * inf * THICKNESS_STEP_PX).max(0.0);
        if next != *w {
            *w = next;
            changed = true;
        }
    }
    changed
}

/// **Strength** — a opacidade por-ponto (com Ctrl: apaga aos poucos).
///
/// `sculpt_strength.cc`: `opacity = clamp(opacity ± influence·0.125, 0, 1)`.
pub(crate) fn strength(st: &mut FlipStroke, p: &ReshapeParams, s: &InputSample) -> bool {
    let pos = st.positions().to_vec();
    let mut changed = false;
    for (i, o) in st.opacities_mut().iter_mut().enumerate() {
        let inf = influence(p, s, pos[i]);
        if inf <= 0.0 {
            continue;
        }
        let next = (*o + sign(p) * inf * STRENGTH_STEP).clamp(0.0, 1.0);
        if next != *o {
            *o = next;
            changed = true;
        }
    }
    changed
}

/// Hash determinístico → `[0,1)`. **splitmix64** (a mesma família do `jitter.rs` do
/// Painter — precedente do projeto; o GP usa um hash próprio, mas a propriedade que
/// importa é a mesma: determinístico, sem estado, decorrelacionado entre índices).
fn hash01(seed: u64, index: u64) -> f32 {
    let mut z = seed
        .wrapping_add(index.wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    // 24 bits de mantissa: exato em f32.
    ((z >> 40) as f32) / ((1u32 << 24) as f32)
}

/// **Randomize** — bagunça a posição, **só perpendicular ao movimento** do cursor
/// (`sculpt_randomize.cc:81-96`).
///
/// Três coisas que o fonte revela e que a intuição erraria:
///
/// 1. **Perpendicular, não radial.** O ruído é aplicado ao longo do `sideways` =
///    a normal da direção do mouse. Ruído radial engrossaria/afinaria a silhueta; o
///    perpendicular *ondula* a linha, que é o efeito de "mão trêmula" desejado.
/// 2. **Re-semeado por AMOSTRA** (`unique_seed()` a cada extensão): parado, o pincel
///    faz um passeio browniano — o traço continua vibrando enquanto você segura. Uma
///    semente por GESTO congelaria o deslocamento e o efeito morreria ao parar.
/// 3. **A amplitude é `influence · noise` PIXELS** (as posições ali são de VIEW).
///    Não há constante de escala escondida: no máximo 1 px por amostra, e o efeito
///    se acumula. É a única fórmula desta crate que carrega a unidade de tela — daí
///    o `px_to_local`.
fn randomize(pts: &mut [Vec2], p: &ReshapeParams, s: &InputSample, sample_no: u64) -> bool {
    // A direção do movimento; parado, o gesto ainda vibra — usa-se um eixo estável
    // (o GP normaliza um delta zero e obtém NaN; aqui o zero cai no eixo X, que é
    // determinístico e não produz NaN nenhum).
    let d = s.delta;
    let len = (d.x * d.x + d.y * d.y).sqrt();
    let forward = if len > 1e-9 {
        Vec2::new(d.x / len, d.y / len)
    } else {
        Vec2::new(1.0, 0.0)
    };
    let sideways = Vec2::new(-forward.y, forward.x);

    let pos = pts.to_vec();
    let mut changed = false;
    for (i, pt) in pts.iter_mut().enumerate() {
        let inf = influence(p, s, pos[i]);
        if inf <= 0.0 {
            continue;
        }
        let noise = 2.0 * hash01(sample_no, i as u64) - 1.0; // [-1, 1)
        *pt += sideways * (inf * noise * p.px_to_local);
        changed = true;
    }
    changed
}

/// **O funil dos pincéis de POSIÇÃO** — o que vale para um anel de pontos qualquer: o
/// contorno de um traço, o contorno de uma região, ou o anel de um BURACO dela.
///
/// É essa indiferença que faz a cor acompanhar a linha (smoke do Enio + o Suzanne): no
/// GP o sculpt edita todas as curvas, e o preenchimento é a triangulação dos pontos
/// delas — mover os pontos re-tria o fill no mesmo frame.
pub(crate) fn position(
    pts: &mut [Vec2],
    p: &ReshapeParams,
    s: &InputSample,
    closed: bool,
    sample_no: u64,
) -> bool {
    match p.kind {
        ReshapeKind::Smooth => smooth(pts, p, s, closed),
        ReshapeKind::Push => push(pts, p, s),
        ReshapeKind::Pinch => pinch(pts, p, s),
        ReshapeKind::Twist => twist(pts, p, s),
        ReshapeKind::Randomize => randomize(pts, p, s, sample_no),
        // O Grab tem caminho próprio (pesos congelados); os de atributo não mexem em
        // posição nenhuma.
        ReshapeKind::Grab | ReshapeKind::Thickness | ReshapeKind::Strength => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A rotação por Taylor bate com `sin`/`cos` de verdade na faixa em que o Twist
    /// vive (|θ| ≤ 1°) — e o erro é MUITO menor que o épsilon do `f32`.
    ///
    /// (Este teste PODE usar transcendentais: ele é o oráculo, não o produto. É
    /// justamente para provar que o produto não precisa delas.)
    #[test]
    fn the_taylor_rotation_matches_the_real_one_within_the_twist_range() {
        for deg in [-1.0f32, -0.5, -0.01, 0.0, 0.01, 0.5, 1.0] {
            let theta = deg * std::f32::consts::PI / 180.0;
            let v = Vec2::new(3.0, -7.0);
            let got = rotate_small(v, theta);
            let (sin, cos) = theta.sin_cos();
            let want = Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos);
            let err = ((got.x - want.x).powi(2) + (got.y - want.y).powi(2)).sqrt();
            assert!(err < 1e-6, "theta={deg}deg: erro {err:e} (|v| = 7,6)");
        }
    }

    /// O hash é determinístico, fica em `[0,1)` e decorrelaciona índices vizinhos.
    #[test]
    fn the_hash_is_deterministic_and_bounded() {
        for seed in 0..8u64 {
            for i in 0..64u64 {
                let a = hash01(seed, i);
                assert_eq!(a, hash01(seed, i), "o mesmo par tem de dar o mesmo valor");
                assert!((0.0..1.0).contains(&a), "fora de [0,1): {a}");
            }
        }
        // Índices vizinhos não andam juntos (o splitmix64 é um bit-mixer).
        let d: f32 = (0..32)
            .map(|i| (hash01(7, i) - hash01(7, i + 1)).abs())
            .sum::<f32>()
            / 32.0;
        assert!(
            d > 0.2,
            "vizinhos correlacionados demais: media |dif| = {d}"
        );
    }
}
