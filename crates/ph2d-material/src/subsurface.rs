//! ⭐⭐⭐ **A SUBSUPERFÍCIE** — a luz que ENTRA na peça, espalha lá dentro e sai por outro sítio.
//!
//! # Proveniência e licença
//!
//! Porta **Apache-2.0** do MaterialX 1.39.5, como o resto desta crate (ver [`crate::bsdf`]).
//! Ficheiros de origem, em `libraries/pbrlib/genglsl/`: `mx_translucent_bsdf.glsl` ·
//! `mx_subsurface_bsdf.glsl` · `lib/mx_microfacet_diffuse.glsl`
//! (`mx_burley_diffusion_profile`, `mx_integrate_burley_diffusion`,
//! `mx_subsurface_scattering_approx`). A composição é a de `libraries/bxdf/open_pbr_surface.mtlx`.
//!
//! # ⭐⭐⭐ São DOIS caminhos e não um, e a diferença é a PEÇA
//!
//! O OpenPBR escolhe entre eles pelo `geometry_thin_walled`, e os dois respondem a perguntas
//! diferentes:
//!
//! | | a peça | a lei | o que se vê |
//! |---|---|---|---|
//! | **parede fina** | uma folha, uma pétala, papel, um abajur | difusa à frente **+ [`translucent`] por trás**, meio a meio | com a luz ATRÁS, ela acende inteira |
//! | **maciça** | jade, cera, mármore fino, leite | [`thick`] — o perfil de difusão de Burley integrado sobre a curvatura local | a luz **contorna** a quina e o terminador amacia |
//!
//! ⚠️ **A parede fina não sabe nada sobre a forma da peça** (é a lambertiana do lado de lá), e a
//! maciça precisa de UMA grandeza geométrica: a **curvatura**. É essa diferença que faz o primeiro
//! caminho custar nada e o segundo custar uma amostra de campo.
//!
//! # ⭐⭐⭐ A lei maciça é a que ESTA CASA já tinha escrita, e a medição é que o mostrou
//!
//! O `ph2d_mesh_render::sss` (a tabela pré-integrada de Penner & Borshukov, portada para o
//! esculpir) integra **o mesmo integral**:
//!
//! ```text
//! D(θ, r) = ∫ clamp(cos(θ + x), 0, 1) · R(2r·sin(x/2)) dx  /  ∫ R(2r·sin(x/2)) dx
//! ```
//!
//! — a mesma **corda** `2r·sin(x/2)`, a mesma normalização. O que muda é o perfil `R`: aquela usa a
//! soma de seis gaussianas de d'Eon & Luebke, esta usa o **Burley** de duas exponenciais
//! (Pixar, *Approximate Reflectance Profiles for Efficient Subsurface Scattering*), e esta soma
//! `32` termos por avaliação em vez de ler uma tabela. ⇒ *não é um sistema novo: é o mesmo
//! fenómeno com o perfil que o oráculo desta linha usa.*
//!
//! # ⭐⭐ E ela depende só do ÂNGULO e do quociente `mfp/raio` — medido, não deduzido
//!
//! Escalar o raio e o caminho livre médio pelo mesmo factor deixa `shape·dist` invariante e `R`
//! escala por `1/k` **uniformemente**, logo o quociente `ΣD/ΣR` não se mexe. Medido sobre uma faixa
//! de `16×` (`r = 0,25 · 1 · 4` com `mfp` igual), o desvio é **`0,00e+00`** em todos os ângulos.
//! ⇒ a grandeza é o adimensional `mfp·κ` — *exactamente o eixo `t = scatter·|κ|` que a tabela do
//! esculpir declara no cabeçalho dela*.
//!
//! # ⚠️ A única divergência declarada: o `acos`
//!
//! O `mx_acos` do MaterialX é o `acos` do GLSL **sem corte** (`stdlib/genglsl/lib/mx_math.glsl`
//! linha 11, um `#define` directo), e `dot` de dois unitários pode ler `1,0000001` em `f32` ⇒ o
//! oráculo devolve **`NaN`** ali. Esta porta corta a `[-1, 1]`.
//!
//! ⛔ *Um pixel `NaN` e um pixel legitimamente preto leem-se iguais*, que é o pior modo de falha que
//! existe — e o corte é **no-op dentro do domínio**, logo não há entrada válida em que as duas leis
//! discordem. Há gate (`o_corte_do_acos_so_actua_onde_a_referencia_daria_nan`).

use crate::bsdf::{self, Bsdf, EPS, V3, dot, forward_facing, mul3, scale3};

const PI: f32 = core::f32::consts::PI;
const PI_INV: f32 = 1.0 / PI;

/// Quantos termos o [`integrate_burley`] soma — `SAMPLE_COUNT` do GLSL de referência.
///
/// ⛔ **Não é um número desta casa e não se afina:** é a quadratura que o oráculo corre, e mudá-la
/// mudaria a resposta contra a qual a paridade é medida.
const SAMPLE_COUNT: usize = 32;

/// O piso do `max(mfp, 0.1)` do `mx_integrate_burley_diffusion`.
const MFP_FLOOR: f32 = 0.1;

/// O piso do `max(curvature, 0.01)` do `mx_subsurface_scattering_approx`.
const CURVATURE_FLOOR: f32 = 0.01;

/// `mx_translucent_bsdf` — a lambertiana do **outro lado** da superfície.
///
/// ⚠️ **Ela nega a normal e NÃO a vira para o observador** (o GLSL faz `N = -N` e mais nada): a luz
/// que esta closure devolve é a que entra pelas costas, logo o `max(0)` dela é sobre `−N·L`. É essa
/// linha, e só ela, que faz uma folha acender com o sol atrás.
pub(crate) fn translucent(weight: f32, color: V3, n: V3, c: &crate::Closure<'_>) -> Bsdf {
    let dark = Bsdf {
        response: [0.0; 3],
        throughput: [0.0; 3],
    };
    if weight < EPS {
        return dark;
    }
    let n = [-n[0], -n[1], -n[2]];
    match c {
        crate::Closure::Reflection(l) => {
            let ndl = dot(n, *l).clamp(0.0, 1.0);
            Bsdf {
                response: scale3(color, weight * ndl * PI_INV),
                ..dark
            }
        }
        crate::Closure::Indirect(env) => Bsdf {
            response: scale3(mul3(env.irradiance(n), color), weight),
            ..dark
        },
    }
}

/// `mx_subsurface_bsdf` — a peça MACIÇA, pela aproximação de curvatura do GLSL de referência.
///
/// `curvature` é `1/raio` em unidades do MUNDO, e `mfp` é o `subsurface_radius` já multiplicado
/// pela escala por canal.
///
/// # ⚠️ Onde esta porta se separa do renderizador de referência: de ONDE vem a curvatura
///
/// O GLSL estima-a por **derivada de ecrã** — `length(fwidth(N)) / length(fwidth(P))`, uma
/// diferença finita entre píxeis vizinhos. ⛔ **Isso aqui era inexprimível sem partir uma
/// propriedade que esta linha tem em 100,000 %:** o `fwidth` da placa é por *quad* de `2×2` e a
/// diferença do traçador de CPU seria por pixel, logo os dois motores deixariam de dar o mesmo
/// byte. ⇒ a curvatura chega **do CAMPO**, que é a mesma resposta nos dois — e é ela que o
/// `Surface::at_curvature` transporta.
///
/// ⚠️ O `closureData.occlusion` do GLSL vale `1` no renderizador que serve de oráculo (o
/// `visibleOcclusion` colapsa em `1`), e a oclusão deste produto é aplicada **fora** do material,
/// sobre o céu. ⇒ a porta escreve o mesmo `1`, e a linha do GLSL fica visível no corpo para quem a
/// for ligar.
pub(crate) fn thick(
    weight: f32,
    color: V3,
    mfp: V3,
    curvature: f32,
    n: V3,
    v: V3,
    c: &crate::Closure<'_>,
) -> Bsdf {
    let dark = Bsdf {
        response: [0.0; 3],
        throughput: [0.0; 3],
    };
    if weight < EPS {
        return dark;
    }
    let n = forward_facing(n, v);
    match c {
        crate::Closure::Reflection(l) => {
            let radius = 1.0 / curvature.max(CURVATURE_FLOOR);
            let sss = scale3(mul3(color, integrate_burley(n, *l, radius, mfp)), PI_INV);
            // `visibleOcclusion = 1 - N·L·(1 - occlusion)`, e `occlusion = 1` aqui (ver o doc).
            Bsdf {
                response: scale3(sss, weight),
                ..dark
            }
        }
        // O GLSL de referência escreve, por extenso: *«for now, we render indirect subsurface as
        // simple indirect diffuse»* — a irradiância pela normal, sem o perfil.
        crate::Closure::Indirect(env) => Bsdf {
            response: scale3(mul3(env.irradiance(n), color), weight),
            ..dark
        },
    }
}

/// `mx_burley_diffusion_profile` — as duas exponenciais sobre a distância.
fn diffusion_profile(dist: f32, shape: V3) -> V3 {
    let denom = dist.max(EPS);
    [0, 1, 2].map(|k| {
        let s = shape[k];
        ((-s * dist).exp() + (-s * dist / 3.0).exp()) / denom
    })
}

/// `mx_integrate_burley_diffusion` — o perfil integrado sobre uma esfera do raio dado.
///
/// ⚠️ **O `acos` é cortado** — ver a divergência declarada no cabeçalho do módulo.
fn integrate_burley(n: V3, l: V3, radius: f32, mfp: V3) -> V3 {
    let theta = dot(n, l).clamp(-1.0, 1.0).acos();
    let shape = mfp.map(|m| 1.0 / m.max(MFP_FLOOR));
    let width = (2.0 * PI) / SAMPLE_COUNT as f32;
    let mut sum_d = [0.0f32; 3];
    let mut sum_r = [0.0f32; 3];
    for i in 0..SAMPLE_COUNT {
        let x = -PI + (i as f32 + 0.5) * width;
        let dist = radius * (2.0 * (x * 0.5).sin()).abs();
        let r = diffusion_profile(dist, shape);
        let w = (theta + x).cos().max(0.0);
        for k in 0..3 {
            sum_d[k] += r[k] * w;
            sum_r[k] += r[k];
        }
    }
    [0, 1, 2].map(|k| sum_d[k] / sum_r[k])
}

/// `mx_mix_bsdf` — `mix(bg, fg, t)`, nas duas metades.
pub(crate) fn mix_bsdf(fg: Bsdf, bg: Bsdf, t: f32) -> Bsdf {
    Bsdf {
        response: bsdf::mix3(bg.response, fg.response, t),
        throughput: bsdf::mix3(bg.throughput, fg.throughput, t),
    }
}
