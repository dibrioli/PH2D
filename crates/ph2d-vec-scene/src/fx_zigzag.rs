//! **Zig Zag / Roughen** — o segundo efeito da pilha (ADR-0132), e a prova da promessa dela.
//!
//! O caminho é reamostrado em cristas igualmente espaçadas **por comprimento de arco** e cada
//! amostra é deslocada pela NORMAL, alternando o lado. É o *Zig Zag* do Illustrator e o path
//! operator homónimo do After Effects.
//!
//! # Espaçamento por ARCO, não por segmento — e isto não é preciosismo
//!
//! O Illustrator conta *"cristas por segmento"*, e por isso a mesma forma desenhada com mais
//! âncoras ganha mais cristas: o efeito passa a descrever **como o caminho foi autorado**, não
//! a forma que se vê. É a mesma doença que o `ph2d-vec-blend` pagou caro para curar (*"duas
//! formas que se veem iguais têm de blendar igual"*), e o motor de arco que o Trim trouxe já a
//! resolve. Aqui as cristas são contadas **no caminho inteiro**: picar uma aresta em vinte
//! pedaços não muda um pixel.
//!
//! # Zig Zag e Roughen são o MESMO motor
//!
//! O Illustrator os vende separados; a diferença é só de onde vem o deslocamento — alternado e
//! regular (*Zig Zag*) ou pseudo-aleatório (*Roughen*). Um `bool` decide, e o gerador é o
//! `splitmix64` que o Painter já usa para o jitter dos dabs: determinístico a partir de uma
//! `seed`, então o mesmo documento desenha igual em qualquer máquina (HR-5 — nada de
//! transcendental nem de `rand`).

use crate::arclen::{Cubic, arclen, inv_arclen, point_at, tangent_at};
use crate::corner_live::segment;
use crate::{VecVertex, VertexKind};

/// Abaixo desta amplitude (em unidades de mundo) o efeito é o ponto neutro.
const EPS: f64 = 1e-12;

/// O menor número de cristas que produz geometria. Com menos de uma o efeito não tem o que
/// alternar, e o caminho volta intacto.
const MIN_RIDGES: f64 = 1.0;

/// Teto de amostras por contorno — a guarda contra um `ridges` absurdo vindo de um save
/// corrompido virar uma alocação de gigabytes. **Não é o teto do artista**: o slider do painel
/// para muito antes, e este número só existe para que o pior caso seja feio em vez de fatal.
const MAX_SAMPLES: usize = 4096;

/// **Os parâmetros do Zig Zag.** Neutro em `amplitude == 0`.
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ZigZagSpec {
    /// Quanto cada crista se afasta do caminho, em **PERCENTAGEM da forma**: `100` = a média
    /// entre a largura e a altura dela ([`crate::effect::FxCtx`]).
    ///
    /// ⚠️ A 1ª versão pôs isto em unidades de MUNDO e o doc argumentava que era o certo. Era o
    /// contrário: as formas da cena têm ~2-3 unidades, então o slider era inútil (Enio,
    /// 2026-07-18). Percentagem faz o mesmo número desenhar a mesma coisa em qualquer escala.
    pub amplitude: f64,
    /// Quantas cristas ao longo do caminho INTEIRO.
    pub ridges: f64,
    /// `true` = ondas suaves (as âncoras ganham alças ao longo da tangente); `false` = picos
    /// em quina, o *Corner* do Illustrator.
    pub smooth: bool,
    /// `Some(seed)` = **Roughen**: o deslocamento vira pseudo-aleatório em vez de alternado.
    /// `None` = Zig Zag regular.
    pub rough_seed: Option<u64>,
}

impl Default for ZigZagSpec {
    fn default() -> Self {
        Self {
            amplitude: 0.0,
            ridges: 8.0,
            smooth: false,
            rough_seed: None,
        }
    }
}

impl ZigZagSpec {
    /// Sem amplitude não há onda — e o neutro tem de ser um no-op byte-idêntico, senão a pilha
    /// não pode saltá-lo e o `Cow::Borrowed` morre (ADR-0132).
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.amplitude.abs() <= EPS || self.ridges < MIN_RIDGES
    }
}

/// `splitmix64` — o mesmo gerador do jitter do Painter. Determinístico e sem transcendental.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Um número em `[-1, 1]` a partir do gerador.
fn signed_unit(state: &mut u64) -> f64 {
    // 53 bits de mantissa: a divisão é exata e o resultado cobre `[0, 1)` uniformemente.
    #[allow(clippy::cast_precision_loss)]
    let u = (splitmix64(state) >> 11) as f64 / (1u64 << 53) as f64;
    u.mul_add(2.0, -1.0)
}

/// **Aplica o Zig Zag a UM contorno.** Devolve `(verts, closed)`.
///
/// O contorno **mantém** o seu `closed`: ondular não abre nem fecha uma forma.
#[must_use]
pub fn zigzag_contour(
    verts: &[VecVertex],
    closed: bool,
    spec: &ZigZagSpec,
    ref_size: f64,
) -> (Vec<VecVertex>, bool) {
    let n = verts.len();
    // Sem referência de tamanho não há distância a construir — e é a mesma resposta do neutro.
    if n < 2 || spec.is_neutral() || ref_size <= EPS {
        return (verts.to_vec(), closed);
    }
    // `100` = a média das dimensões da forma. É aqui, e só aqui, que a percentagem vira mundo.
    let amplitude = spec.amplitude / 100.0 * ref_size;
    let seg_count = if closed { n } else { n - 1 };
    let segs: Vec<Cubic> = (0..seg_count).map(|i| segment(verts, i, n)).collect();
    let lens: Vec<f64> = segs.iter().map(arclen).collect();
    let total: f64 = lens.iter().sum();
    if total <= EPS {
        return (verts.to_vec(), closed);
    }

    // Uma crista sobe e a seguinte desce: são DUAS amostras por crista. Num contorno aberto as
    // duas pontas são amostras próprias (o caminho não dá a volta), daí o `+ 1`.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let ridges = spec.ridges.floor().max(MIN_RIDGES) as usize;
    let count = (ridges * 2).min(MAX_SAMPLES);
    // Fechado: `count` amostras que dão a volta. Aberto: `count + 1` pontos, porque as duas
    // pontas são amostras próprias. O PASSO é o mesmo nos dois casos — em ambos há `count`
    // intervalos —, e escrevê-lo como um `if` de ramos iguais foi resíduo de raciocínio meu
    // que o clippy apanhou.
    let samples = if closed { count } else { count + 1 };
    #[allow(clippy::cast_precision_loss)]
    let step = total / count as f64;

    let mut state = spec.rough_seed.unwrap_or(0);
    let mut out = Vec::with_capacity(samples);
    for k in 0..samples {
        let s = (k as f64 * step).min(total);
        let Some((point, tangent)) = walk_to(&segs, &lens, s) else {
            continue;
        };
        // A normal é a tangente rodada um quarto de volta — troca de eixo e sinal, sem
        // transcendental (HR-5).
        let normal = [-tangent[1], tangent[0]];
        let lift = if spec.rough_seed.is_some() {
            signed_unit(&mut state)
        } else if k % 2 == 0 {
            1.0
        } else {
            -1.0
        } * amplitude;
        let anchor = [
            normal[0].mul_add(lift, point[0]),
            normal[1].mul_add(lift, point[1]),
        ];
        // Suave: as alças acompanham a TANGENTE do caminho original naquele ponto, com um
        // terço do passo de cada lado — é a construção que faz uma sequência de amostras
        // reproduzir uma curva lisa em vez de uma poligonal arredondada a olho.
        let arm = if spec.smooth { step / 3.0 } else { 0.0 };
        out.push(VecVertex {
            anchor,
            in_handle: [
                tangent[0].mul_add(-arm, anchor[0]),
                tangent[1].mul_add(-arm, anchor[1]),
            ],
            out_handle: [
                tangent[0].mul_add(arm, anchor[0]),
                tangent[1].mul_add(arm, anchor[1]),
            ],
            kind: if spec.smooth {
                VertexKind::Smooth
            } else {
                VertexKind::Corner
            },
            corner_radius: 0.0,
        });
    }
    if out.len() < 2 {
        return (verts.to_vec(), closed);
    }
    (out, closed)
}

/// Onde o comprimento `s` cai: o ponto e a tangente unitária ali. `None` numa cúspide.
fn walk_to(segs: &[Cubic], lens: &[f64], s: f64) -> Option<([f64; 2], [f64; 2])> {
    let mut walked = 0.0;
    for (i, &l) in lens.iter().enumerate() {
        if s <= walked + l || i + 1 == lens.len() {
            let t = inv_arclen(&segs[i], (s - walked).max(0.0));
            return Some((point_at(&segs[i], t), tangent_at(&segs[i], t)?));
        }
        walked += l;
    }
    None
}

#[cfg(test)]
#[path = "fx_zigzag_tests.rs"]
mod tests;
