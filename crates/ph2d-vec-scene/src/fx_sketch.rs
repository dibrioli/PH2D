//! **Sketch** — o traço à mão: N passadas do caminho, cada uma com um wobble próprio.
//!
//! O *Sketch* LPE do Inkscape e o traço do Rough.js. UMA passada perturbada lê como *errado*, não
//! como *esboçado* — o olho precisa de ≥2 linhas quase-coincidentes para ler "desenhado à mão"
//! (Inkscape semeia 2, o Rough.js `roughness` desenha 2). Então este efeito **multiplica** o
//! caminho: cada contorno vira `passes` cópias, cada cópia deslocada pela NORMAL por um ruído
//! COERENTE de baixa frequência (não branco — um traço treme suave, não serrilha).
//!
//! # Ruído coerente, não branco
//!
//! O `splitmix64` (o mesmo do jitter do Painter / do Roughen do Zig Zag — HR-5, sem transcendental
//! nem `rand`) semeia `detail` offsets ao longo do arco; o valor num ponto é a interpolação
//! suave (`smoothstep`) entre os dois vizinhos. Poucos offsets = ondas largas (esboço solto);
//! muitos = tremor fino. Num contorno FECHADO o anel de offsets fecha (índice `mod detail`), então
//! não há emenda.
//!
//! # Multi-output, como o Repeater
//!
//! A saída é UM `VecPath` cujo contorno primário é a 1ª passada e os `subpaths` são as demais (e
//! as passadas de cada buraco do compound). É o padrão do `fx_repeat`/`fx_knot`: um efeito que
//! olha o caminho inteiro e emite muitos contornos, em vez do `apply_per_contour`.
//!
//! # Falloff
//!
//! `falloff` (opcional) escala a amplitude por-amostra pela força no ponto-base: `w=0` deixa a
//! amostra na curva original, `w=1` é o tremor cheio. Um Sketch que treme só no centro sai de um
//! Radial. `None` é byte-idêntico ao Sketch sem campo.

use crate::arc_path::ArcPath;
use crate::compound::Contour;
use crate::fx_falloff::Falloff;
use crate::{VecPath, VecVertex, VertexKind};

/// Abaixo desta amplitude (em unidades de mundo, após a percentagem) o efeito é o ponto neutro.
const EPS: f64 = 1e-12;

/// Menos de uma passada não desenha nada — o caminho volta intacto.
const MIN_PASSES: f64 = 1.0;

/// O menor número de offsets de ruído que produz um wobble (com um só não há entre-o-que-lerpar).
const MIN_DETAIL: f64 = 2.0;

/// Teto de amostras por passada — guarda contra um save corrompido com `detail` absurdo virar uma
/// alocação de gigabytes. **Não é o teto do artista** (o slider para muito antes).
const MAX_SAMPLES: usize = 2048;

/// Teto de passadas — idem: o slider do painel para muito antes.
const MAX_PASSES: usize = 12;

/// Quantas amostras densas por offset de ruído — o suficiente para o wobble ler liso entre os
/// offsets sem virar poligonal a olho.
const SAMPLES_PER_DETAIL: usize = 6;

/// **Os parâmetros do Sketch.** Neutro em `roughness == 0`.
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SketchSpec {
    /// Quantas passadas (linhas quase-coincidentes). 2 é o mínimo que lê como "à mão".
    pub passes: f64,
    /// A amplitude do tremor, em **percentagem da forma** (`100` = a média entre largura e
    /// altura, [`crate::effect::FxCtx`]). Percentagem, não mundo, para o mesmo número desenhar
    /// a mesma coisa em qualquer escala.
    pub roughness: f64,
    /// Quantos offsets de ruído ao longo do caminho: poucos = ondas largas, muitos = tremor fino.
    pub detail: f64,
    /// A semente do gerador. Muda o desenho do tremor sem mudar a estrutura — o botão "outra
    /// tentativa" que o Roughen do Zig Zag não tem.
    pub seed: u64,
}

impl Default for SketchSpec {
    fn default() -> Self {
        // ⚠️ Nasce NEUTRO (`roughness == 0`): a lei do `every_kind_is_born_neutral` (ADR-0132) —
        // um efeito recém-posto na pilha é um no-op byte-idêntico até o artista o configurar,
        // exatamente como o Zig Zag nasce com `amplitude == 0`. As demais defaults (2 passadas,
        // detalhe 6) são a forma que ele toma assim que a Roughness sobe.
        Self {
            passes: 2.0,
            roughness: 0.0,
            detail: 6.0,
            seed: 1,
        }
    }
}

impl SketchSpec {
    /// Sem amplitude não há tremor — e o neutro tem de ser um no-op byte-idêntico, senão a pilha
    /// não pode saltá-lo e o `Cow::Borrowed` do `cooked()` morre (ADR-0132).
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.roughness.abs() <= EPS || self.passes < MIN_PASSES
    }
}

/// `splitmix64` — o mesmo gerador do jitter do Painter e do Roughen do Zig Zag. Determinístico,
/// sem transcendental (HR-5). Re-declarado aqui (é O algoritmo, não uma pergunta com duas
/// respostas: não há o que divergir).
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Um número em `[-1, 1]` a partir do gerador.
fn signed_unit(state: &mut u64) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let u = (splitmix64(state) >> 11) as f64 / (1u64 << 53) as f64;
    u.mul_add(2.0, -1.0)
}

/// Interpolação suave entre `a` e `b` por `t ∈ [0,1]` — `smoothstep(t) = t²(3 − 2t)`, polinômio
/// (HR-5, sem transcendental).
fn smooth_lerp(a: f64, b: f64, t: f64) -> f64 {
    let w = t * t * 2.0f64.mul_add(-t, 3.0);
    (b - a).mul_add(w, a)
}

/// A tabela de offsets de ruído de UMA passada — `detail` valores em `[-1,1]`. Num contorno
/// fechado o consumidor lê o índice `mod detail` (o anel fecha); num aberto ele clampa nas pontas.
fn noise_table(seed: u64, pass: usize, contour: usize, detail: usize) -> Vec<f64> {
    let mut st = seed
        ^ (pass as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (contour as u64).wrapping_mul(0xD1B5_4A32_D192_ED03);
    (0..detail).map(|_| signed_unit(&mut st)).collect()
}

/// O valor do ruído na fração de arco `u ∈ [0,1)` — interpolação suave entre os offsets vizinhos.
fn noise_at(table: &[f64], u: f64, closed: bool) -> f64 {
    let detail = table.len();
    if detail == 0 {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let x = u * detail as f64;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let i0 = (x.floor() as usize).min(detail - 1);
    let frac = x - x.floor();
    let i1 = if closed {
        (i0 + 1) % detail
    } else {
        (i0 + 1).min(detail - 1)
    };
    smooth_lerp(table[i0], table[i1], frac)
}

/// **Uma passada de um contorno.** Devolve os vértices (Smooth, com alças na tangente) ou `None`
/// se o contorno não tem arco (grau ou cúspide total).
fn sketch_pass(
    ap: &ArcPath,
    total: f64,
    closed: bool,
    table: &[f64],
    amplitude: f64,
    falloff: Option<&Falloff>,
) -> Option<Vec<VecVertex>> {
    if total <= EPS {
        return None;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let count = (table.len() * SAMPLES_PER_DETAIL).clamp(8, MAX_SAMPLES);
    // Fechado: `count` amostras dão a volta. Aberto: as duas pontas são amostras próprias (`+1`).
    let n = if closed { count } else { count + 1 };
    #[allow(clippy::cast_precision_loss)]
    let step = total / count as f64;
    let mut out = Vec::with_capacity(n);
    for k in 0..n {
        #[allow(clippy::cast_precision_loss)]
        let s = (k as f64 * step).min(total);
        let (point, tangent) = ap.frame_at(s);
        let normal = [-tangent[1], tangent[0]];
        let u = s / total;
        let w = falloff.map_or(1.0, |f| f.eval(point));
        let lift = noise_at(table, u, closed) * amplitude * w;
        let anchor = [
            normal[0].mul_add(lift, point[0]),
            normal[1].mul_add(lift, point[1]),
        ];
        // Alças na tangente (um terço do passo, como o Zig Zag suave) ⇒ a sequência de amostras
        // reproduz uma curva lisa em vez de uma poligonal arredondada a olho.
        let arm = step / 3.0;
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
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        });
    }
    Some(out)
}

/// **Aplica o Sketch ao caminho inteiro.** Cada contorno (primário + buracos) vira `passes`
/// passadas; a 1ª é o contorno primário da saída, o resto são `subpaths`.
#[must_use]
pub fn sketch_path(
    path: &VecPath,
    spec: &SketchSpec,
    ref_size: f64,
    falloff: Option<&Falloff>,
) -> VecPath {
    if spec.is_neutral() || ref_size <= EPS {
        return path.clone();
    }
    let amplitude = spec.roughness / 100.0 * ref_size;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let passes = (spec.passes.floor().max(MIN_PASSES) as usize).min(MAX_PASSES);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let detail = (spec.detail.floor().max(MIN_DETAIL) as usize).min(MAX_SAMPLES);

    // Todos os contornos de entrada, na ordem: primário, depois cada subpath.
    let contours: Vec<(&[VecVertex], bool)> = std::iter::once((path.verts.as_slice(), path.closed))
        .chain(path.subpaths.iter().map(|c| (c.verts.as_slice(), c.closed)))
        .collect();

    let mut produced: Vec<(Vec<VecVertex>, bool)> = Vec::new();
    for (ci, &(verts, closed)) in contours.iter().enumerate() {
        let Some(ap) = ArcPath::from_contour(verts, closed) else {
            continue;
        };
        let total = ap.total();
        for p in 0..passes {
            let table = noise_table(spec.seed, p, ci, detail);
            if let Some(v) = sketch_pass(&ap, total, closed, &table, amplitude, falloff) {
                produced.push((v, closed));
            }
        }
    }

    // Nada produzido (tudo degenerado) ⇒ o neutro é o caminho intacto.
    if produced.is_empty() {
        return path.clone();
    }
    let mut out = path.clone();
    let (verts0, closed0) = produced.remove(0);
    out.verts = verts0;
    out.closed = closed0;
    out.subpaths = produced
        .into_iter()
        .map(|(verts, closed)| Contour { verts, closed })
        .collect();
    out
}

#[cfg(test)]
#[path = "fx_sketch_tests.rs"]
mod tests;
