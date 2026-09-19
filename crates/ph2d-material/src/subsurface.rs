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
const FRAC_PI_2: f32 = core::f32::consts::FRAC_PI_2;
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

/// ⭐⭐⭐ **O ALBEDO CRU a partir da reflectância difusa** — a inversão publicada de Christensen &
/// Burley (*Approximate Reflectance Profiles for Efficient Subsurface Scattering*, Pixar, 2015), o
/// ajuste da fonte pontual.
///
/// ⚠️ `α ≥ A` **sempre**, e por uma razão física: o espalhamento múltiplo perde energia, logo a
/// reflectância que se vê é menor que o albedo de um evento. É essa desigualdade que faz a matiz
/// LAVAR ao afinar a peça — e é também por ela que a correcção crua **clareia**, o que a
/// [`cor_na_profundidade`] tem de desfazer.
fn albedo_cru(a: f32) -> f32 {
    let a = a.clamp(0.0, 1.0);
    1.0 - (-5.09406 * a + 2.61188 * a * a - 4.31805 * a * a * a).exp()
}

/// A profundidade adimensional (`mfp·κ`) a que a transição está a meio — **calibrada no oráculo**.
const X0_DA_TRANSICAO: f32 = 0.306;

/// Quão abrupta é a transição — **calibrada no oráculo**.
const N_DA_TRANSICAO: f32 = 2.25;

/// ⭐⭐⭐ **A COR QUE A PROFUNDIDADE DEIXA** — a cura do defeito medido na
/// [`§17`](../../../docs/Render3d/10_a_luz_que_atravessa_a_peca.md).
///
/// # ⛔⛔ O defeito
///
/// O [`integrate_burley`] devolve `Σ(R·w) / Σ R`, e o perfil aparece **em cima e em baixo**: tudo o
/// que ele sabe sobre a profundidade cancela-se. Com o `mfp` partilhado pelos canais a matiz que
/// sai é a que o artista escreveu — **a qualquer profundidade**, medido a `2,14286` em dezasseis
/// células (quatro profundidades × quatro ângulos, cinco casas). ⚠️ *É a aproximação publicada que
/// é surda; o nosso porte dela é fiel.*
///
/// # ⭐⭐⭐ Os DOIS extremos são DERIVADOS, e só a transição é calibrada
///
/// | regime | o que acontece | a matiz |
/// |---|---|---|
/// | `mfp ≪ peça` | **muitos** eventos de espalhamento | a **reflectância** autorada — *a lei de hoje* |
/// | `mfp ≳ peça` | **poucos**; a luz atravessa e não volta | o **albedo cru** ([`albedo_cru`]) |
///
/// ⇒ a lei é `A · (α/A)^(1−f)`, com `f: 1 → 0` a percorrer a transição. ⭐ **Escrita assim, em
/// `f = 1` o expoente é `0` e a potência devolve `1` EXACTAMENTE** — a cor volta a ser `A` ao bit,
/// que é o que faz o ponto neutro ser byte-idêntico sem uma cerca a lembrar.
///
/// # ⚠️ Ela preserva a LUMINÂNCIA, e isso é uma decisão MEDIDA
///
/// `α ≥ A` sempre ⇒ a correcção crua **clareia** toda a peça. ⛔ Mas o defeito medido na §17.6 é
/// **só de matiz**: a magnitude da nossa lei responde `1,76×` no terminador e `3,50×` do lado
/// escuro sobre a mesma faixa de profundidade — *ela já faz o trabalho dela*. ⇒ curar o que não
/// está partido seria trocar um defeito medido por um não medido, e a correcção é renormalizada
/// para deixar a luminância onde estava. ⚠️ Reescalar por um escalar **não muda `R/B`**, logo a
/// curva que a calibração mediu fica intacta.
///
/// # ⛔ O `mfp` entra CRU, e não pelo piso do perfil
///
/// O `max(mfp, 0.1)` do [`integrate_burley`] guarda um recurso **numérico** — o perfil diverge em
/// `mfp → 0`. Esta lei não tem essa divergência (`mfp → 0` ⇒ `f → 1` ⇒ a cor autorada), logo herdar
/// aquele piso importaria para aqui um defeito que a §17.7 mediu do outro lado: *um quarto do raio
/// da peça é o chão do botão do artista, e abaixo dele ele não faz nada.*
///
/// # A calibração
///
/// `X0 = 0,306` · `N = 2,25`, contra as quatro profundidades do oráculo convergido (raio da peça
/// `0,42`), com o piso fino **derivado** e não ajustado:
///
/// | `mfp/raio` | matiz MEDIDA | matiz da lei | erro |
/// |---:|---:|---:|---:|
/// | `0,0714` | `2,1088` | `2,0992` | `−0,46 %` |
/// | `0,2381` | `1,7010` | `1,7467` | `+2,69 %` |
/// | `0,7143` | `1,3472` | `1,3114` | `−2,65 %` |
/// | `2,3810` | `1,1934` | `1,2259` | `+2,73 %` |
///
/// ⇒ pior erro de matiz **`2,73 %`** sobre uma faixa de `33×`, contra os **`+79,6 %`** que a lei
/// de hoje erra no ponto mais fundo.
pub(crate) fn cor_na_profundidade(cor: V3, mfp: V3, curvature: f32, peso: f32) -> V3 {
    // ⭐⭐ **Isto é um atalho de DESEMPENHO e não uma cerca de correcção — e há prova de mutação.**
    //
    // Apagá-lo deixa os quatro gates **VERDES** (medido), porque com `peso = 0` o expoente é
    // exactamente `0`, `x^0` é `1` ao bit, e a renormalização divide `lum(cor)` por si próprio, que
    // é `1,0` exacto em IEEE-754. ⇒ *o ponto neutro é byte-idêntico por ÁLGEBRA*, e este `if` só
    // evita três `powf` e três `exp` por pixel no caminho de omissão, que é todo o produto de hoje.
    //
    // ⚠️ As outras seis mutações desta lei **sangram** — ver o cabeçalho acima.
    if peso <= 0.0 {
        return cor;
    }
    let k = curvature.max(CURVATURE_FLOOR);
    let bruto = [0, 1, 2].map(|i| {
        let a = cor[i];
        if a <= 0.0 {
            return a;
        }
        let x = (mfp[i].max(0.0) * k) / X0_DA_TRANSICAO;
        let f = 1.0 / (1.0 + x.powf(N_DA_TRANSICAO));
        a * (albedo_cru(a) / a).powf((1.0 - f) * peso)
    });
    // ⭐ A luminância de Rec.709 — a mesma que as réguas desta linha usam.
    let lum = |c: V3| 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
    let (antes, depois) = (lum(cor), lum(bruto));
    if depois <= EPS {
        return cor;
    }
    let escala = antes / depois;
    bruto.map(|c| c * escala)
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
    let meia = width * 0.5;
    let mut sum_d = [0.0f32; 3];
    let mut sum_r = [0.0f32; 3];
    for i in 0..SAMPLE_COUNT {
        let x = -PI + (i as f32 + 0.5) * width;
        let dist = radius * (2.0 * (x * 0.5).sin()).abs();
        let r = diffusion_profile(dist, shape);
        // ⭐⭐⭐ **O cosseno é integrado EXACTAMENTE dentro da célula, e não amostrado no meio dela.**
        //
        // ⛔⛔ O `max(cos(θ+x), 0)` do ponto médio põe um VINCO na resposta em cada nó da
        // quadratura, e o perfil de Burley é singular em `x = 0` ⇒ as duas células vizinhas do zero
        // levam quase todo o peso e o vinco delas é o que se VÊ. Medido na lei sozinha: o pior
        // salto da 2.ª derivada cai em **`N·L = −0,0980`**, que é `sin(π/32)` — e **não se move**
        // com o `Subsurface Radius` (`0,1` · `1,0` · `4,0` dão todos o mesmo ponto). *Uma feição
        // cuja posição não depende de nenhum parâmetro físico é da discretização.*
        //
        // ⭐ A média do cosseno na célula é `(sin b − sin a)/largura` com os extremos cortados ao
        // domínio onde ele é positivo. Ela é **C¹ em θ** — nos cortes a derivada é `cos(±π/2) = 0`,
        // logo a emenda não deixa quina — e converge para o MESMO integral, com o mesmo número de
        // amostras do perfil (a grelha do oráculo não se mexe).
        let a = (theta + x - meia).clamp(-FRAC_PI_2, FRAC_PI_2);
        let b = (theta + x + meia).clamp(-FRAC_PI_2, FRAC_PI_2);
        let w = (b.sin() - a.sin()) / width;
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
