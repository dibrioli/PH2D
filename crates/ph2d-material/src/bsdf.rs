//! **As peças do BSDF** — o port, função a função, do GLSL de referência do MaterialX.
//!
//! # Proveniência e licença
//!
//! Porta **Apache-2.0** (a licença lida no artefacto instalado: `materialx 1.39.5`,
//! `/usr/share/licenses/materialx/LICENSE`). Copyright Contributors to the MaterialX Project.
//! Ficheiros de origem, em `/usr/share/materialx/libraries/pbrlib/genglsl/`:
//! `lib/mx_microfacet.glsl` · `lib/mx_microfacet_specular.glsl` · `lib/mx_microfacet_diffuse.glsl` ·
//! `mx_dielectric_bsdf.glsl` · `mx_generalized_schlick_bsdf.glsl` · `mx_oren_nayar_diffuse_bsdf.glsl` ·
//! `mx_add_bsdf.glsl` · `mx_layer_bsdf.glsl` · `mx_multiply_bsdf_float.glsl` ·
//! `mx_multiply_bsdf_color3.glsl`.
//!
//! ⚠️ **Os nomes e as constantes são os do GLSL de propósito** — um port que «melhorasse» uma fórmula
//! deixaria de ser a referência, e a paridade com o oráculo (`fixtures/`) é o que prova que não o fez.
//!
//! # ⚠️ Só a metade ISOTRÓPICA, e não é simplificação escondida
//!
//! O GLSL avalia a NDF no referencial da tangente (`Ht = (H·X, H·Y, H·N)`). Com as duas rugosidades
//! iguais o termo depende só de `H·N` (`|Ht.xy|² = 1 − (H·N)²` para `H` unitário), e o traçador do
//! modelador não tem tangentes. A anisotropia entra quando houver quem forneça `X`.

pub(crate) type V3 = [f32; 3];

/// `M_FLOAT_EPS` do MaterialX.
pub(crate) const EPS: f32 = 1.0e-8;
const PI: f32 = core::f32::consts::PI;
const PI_INV: f32 = 1.0 / PI;

/// O par que toda closure devolve: o que ela reflecte e o que deixa passar para a camada de baixo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Bsdf {
    pub(crate) response: V3,
    pub(crate) throughput: V3,
}

impl Bsdf {
    /// `BSDF(vec3(0.0), vec3(1.0))` — o valor com que o shader gerado inicializa toda closure, e o
    /// que uma closure de peso zero devolve.
    pub(crate) const NONE: Self = Self {
        response: [0.0; 3],
        throughput: [1.0; 3],
    };
}

// ── álgebra mínima ────────────────────────────────────────────────────────────────────────────

pub(crate) fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(crate) fn normalize(a: V3) -> V3 {
    let n = dot(a, a).sqrt();
    [a[0] / n, a[1] / n, a[2] / n]
}

pub(crate) fn add3(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub(crate) fn mul3(a: V3, b: V3) -> V3 {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2]]
}

pub(crate) fn scale3(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// O `mix` do GLSL: `a + (b − a)·t`.
pub(crate) fn mix3(a: V3, b: V3, t: f32) -> V3 {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

pub(crate) fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn max0(a: V3) -> V3 {
    [a[0].max(0.0), a[1].max(0.0), a[2].max(0.0)]
}

/// `mx_forward_facing_normal`.
pub(crate) fn forward_facing(n: V3, v: V3) -> V3 {
    if dot(n, v) < 0.0 {
        [-n[0], -n[1], -n[2]]
    } else {
        n
    }
}

// ── as leis de composição (`mx_add_bsdf` · `mx_multiply_bsdf_*` · `mx_layer_bsdf`) ────────────

/// `mx_add_bsdf`: as respostas somam e o throughput é `t₁ + t₂ − 1`, nunca negativo.
pub(crate) fn add(a: Bsdf, b: Bsdf) -> Bsdf {
    Bsdf {
        response: add3(a.response, b.response),
        throughput: [
            (a.throughput[0] + b.throughput[0] - 1.0).max(0.0),
            (a.throughput[1] + b.throughput[1] - 1.0).max(0.0),
            (a.throughput[2] + b.throughput[2] - 1.0).max(0.0),
        ],
    }
}

/// `mx_multiply_bsdf_float`: o peso é cortado a `0..=1` e **o throughput não se toca**.
pub(crate) fn mul_float(a: Bsdf, w: f32) -> Bsdf {
    Bsdf {
        response: scale3(a.response, w.clamp(0.0, 1.0)),
        throughput: a.throughput,
    }
}

/// `mx_multiply_bsdf_color3`: a cor é cortada a `0..=1` e o throughput não se toca.
pub(crate) fn mul_color(a: Bsdf, c: V3) -> Bsdf {
    Bsdf {
        response: mul3(
            a.response,
            [
                c[0].clamp(0.0, 1.0),
                c[1].clamp(0.0, 1.0),
                c[2].clamp(0.0, 1.0),
            ],
        ),
        throughput: a.throughput,
    }
}

/// `mx_layer_bsdf`: o de cima reflecte, e o de baixo só reflecte o que o de cima deixou passar.
pub(crate) fn layer(top: Bsdf, base: Bsdf) -> Bsdf {
    Bsdf {
        response: add3(top.response, mul3(base.response, top.throughput)),
        throughput: mul3(top.throughput, base.throughput),
    }
}

// ── microfacetas (`mx_microfacet*.glsl`) ─────────────────────────────────────────────────────

/// `mx_ior_to_f0`.
pub(crate) fn ior_to_f0(ior: f32) -> f32 {
    let r = (ior - 1.0) / (ior + 1.0);
    r * r
}

/// `mx_fresnel_dielectric` — a de Fresnel exacta, com reflexão interna total.
pub(crate) fn fresnel_dielectric(cos_theta: f32, ior: f32) -> f32 {
    let c = cos_theta;
    let g2 = ior * ior + c * c - 1.0;
    if g2 < 0.0 {
        return 1.0;
    }
    let g = g2.sqrt();
    let a = (g - c) / (g + c);
    let b = ((g + c) * c - 1.0) / ((g - c) * c + 1.0);
    0.5 * a * a * (1.0 + b * b)
}

/// `mx_fresnel_hoffman_schlick` — o Schlick generalizado com o termo `F82` (a cor na borda de um metal).
pub(crate) fn fresnel_hoffman_schlick(
    cos_theta: f32,
    f0: V3,
    f82: V3,
    f90: V3,
    exponent: f32,
) -> V3 {
    const COS_THETA_MAX: f32 = 1.0 / 7.0;
    let factor = 1.0 / (COS_THETA_MAX * (1.0 - COS_THETA_MAX).powf(6.0));
    let x = cos_theta.clamp(0.0, 1.0);
    let at_max = mix3(f0, f90, (1.0 - COS_THETA_MAX).powf(exponent));
    let a = scale3(mul3(at_max, add3([1.0; 3], scale3(f82, -1.0))), factor);
    let base = mix3(f0, f90, (1.0 - x).powf(exponent));
    let tail = x * (1.0 - x).powf(6.0);
    add3(base, scale3(a, -tail))
}

/// `mx_ggx_dir_albedo_analytic` — o ajuste racional quadrático aos dados de Monte Carlo.
pub(crate) fn ggx_dir_albedo(ndv: f32, alpha: f32, f0: V3, f90: V3) -> V3 {
    let (x, y) = (ndv, alpha);
    let (x2, y2) = (x * x, y * y);
    const C: [[f32; 4]; 9] = [
        [0.1003, 0.9345, 1.0, 1.0],
        [-0.6303, -2.323, -1.765, 0.2281],
        [9.748, 2.229, 8.263, 15.94],
        [-2.038, -3.748, 11.53, -55.83],
        [29.34, 1.424, 28.96, 13.08],
        [-8.245, -0.7684, -7.507, 41.26],
        [-26.44, 1.436, -36.11, 54.9],
        [19.99, 0.2913, 15.86, 300.2],
        [-5.448, 0.6286, 33.37, -285.1],
    ];
    let t = [1.0, x, y, x * y, x2, y2, x2 * y, x * y2, x2 * y2];
    let mut r = [0.0_f32; 4];
    for (row, tk) in C.iter().zip(t) {
        for i in 0..4 {
            r[i] += row[i] * tk;
        }
    }
    let a = (r[0] / r[2]).clamp(0.0, 1.0);
    let b = (r[1] / r[3]).clamp(0.0, 1.0);
    add3(scale3(f0, a), scale3(f90, b))
}

/// `mx_ggx_energy_compensation` (Turquin 2019): o que a reflexão simples perde nas múltiplas.
pub(crate) fn ggx_energy_compensation(ndv: f32, alpha: f32, fss: V3) -> V3 {
    let ess = ggx_dir_albedo(ndv, alpha, [1.0; 3], [1.0; 3])[0];
    add3([1.0; 3], scale3(fss, (1.0 - ess) / ess))
}

/// `mx_ggx_NDF`, isotrópica — ver o doc do módulo.
///
/// ⚠️⚠️ **`|H × N|²` e NÃO `1 − (H·N)²`, e a diferença foi MEDIDA contra o oráculo:** o GLSL projecta
/// `H` no plano da tangente (`Ht.xy`), cujas componentes são pequenas e exactas; `1 − (H·N)²` subtrai
/// dois números quase iguais e, em `f32`, perde os dígitos exactamente onde um lóbulo estreito os
/// precisa. No espelho da fixture (rugosidade `0,02`, `α = 4e-4`) a subtracção afastava a luz directa
/// do oráculo `1,12e-4` relativo — acima da barra — enquanto a réplica em `f64` ficava a `7,7e-6`. O
/// produto vectorial É a projecção, sem a subtracção.
fn ggx_ndf_isotropic(h: V3, n: V3, alpha: f32) -> f32 {
    let t = [
        h[1] * n[2] - h[2] * n[1],
        h[2] * n[0] - h[0] * n[2],
        h[0] * n[1] - h[1] * n[0],
    ];
    let ndh = dot(h, n);
    let he2 = dot(t, t) / (alpha * alpha);
    let denom = he2 + ndh * ndh;
    1.0 / (PI * alpha * alpha * denom * denom)
}

/// `mx_ggx_smith_G2` — o mascaramento-sombreamento correlacionado em altura.
fn ggx_smith_g2(ndl: f32, ndv: f32, alpha: f32) -> f32 {
    let a2 = alpha * alpha;
    let lambda_l = (a2 + (1.0 - a2) * ndl * ndl).sqrt();
    let lambda_v = (a2 + (1.0 - a2) * ndv * ndv).sqrt();
    2.0 * ndl * ndv / (lambda_l * ndv + lambda_v * ndl)
}

/// O que as duas closures de microfacetas calculam igual antes de divergir no Fresnel.
struct Facet {
    ndv: f32,
    alpha: f32,
    vdh: f32,
    d: f32,
    g: f32,
}

/// ⚠️ O `N·L` não sai daqui: ele só entra no `G2`, e a resposta cancela-o («Note: NdotL is cancelled
/// out», no GLSL).
fn facet(n: V3, v: V3, l: V3, alpha: f32) -> Facet {
    let n = forward_facing(n, v);
    let ndv = dot(n, v).clamp(EPS, 1.0);
    let alpha = alpha.clamp(EPS, 1.0);
    let h = normalize(add3(l, v));
    let ndl = dot(n, l).clamp(EPS, 1.0);
    let vdh = dot(v, h).clamp(EPS, 1.0);
    Facet {
        ndv,
        alpha,
        vdh,
        d: ggx_ndf_isotropic(h, n, alpha),
        g: ggx_smith_g2(ndl, ndv, alpha),
    }
}

/// `mx_dielectric_bsdf`, closure de REFLEXÃO, sem película fina.
pub(crate) fn dielectric_reflection(
    weight: f32,
    tint: V3,
    ior: f32,
    alpha: f32,
    n: V3,
    v: V3,
    l: V3,
) -> Bsdf {
    if weight < EPS {
        return Bsdf::NONE;
    }
    let f = facet(n, v, l, alpha);
    let fresnel = [fresnel_dielectric(f.vdh, ior); 3];
    let comp = ggx_energy_compensation(f.ndv, f.alpha, fresnel);
    let f0 = ior_to_f0(ior);
    let dir_albedo = mul3(ggx_dir_albedo(f.ndv, f.alpha, [f0; 3], [1.0; 3]), comp);
    Bsdf {
        response: scale3(
            mul3(mul3(fresnel, comp), max0(tint)),
            f.d * f.g * weight / (4.0 * f.ndv),
        ),
        throughput: add3([1.0; 3], scale3(dir_albedo, -weight)),
    }
}

/// `mx_generalized_schlick_bsdf`, closure de REFLEXÃO, sem película fina.
#[allow(clippy::too_many_arguments)] // a assinatura é a do GLSL, parâmetro a parâmetro
pub(crate) fn schlick_reflection(
    weight: f32,
    color0: V3,
    color82: V3,
    color90: V3,
    exponent: f32,
    alpha: f32,
    n: V3,
    v: V3,
    l: V3,
) -> Bsdf {
    if weight < EPS {
        return Bsdf::NONE;
    }
    let (c0, c82, c90) = (max0(color0), max0(color82), max0(color90));
    let f = facet(n, v, l, alpha);
    let fresnel = fresnel_hoffman_schlick(f.vdh, c0, c82, c90, exponent);
    let comp = ggx_energy_compensation(f.ndv, f.alpha, fresnel);
    let dir_albedo = mul3(ggx_dir_albedo(f.ndv, f.alpha, c0, c90), comp);
    let avg = (dir_albedo[0] + dir_albedo[1] + dir_albedo[2]) * (1.0 / 3.0);
    Bsdf {
        response: scale3(mul3(fresnel, comp), f.d * f.g * weight / (4.0 * f.ndv)),
        throughput: [1.0 - avg * weight; 3],
    }
}

const FUJII_CONSTANT_1: f32 = 0.5 - 2.0 / (3.0 * PI);
const FUJII_CONSTANT_2: f32 = 2.0 / 3.0 - 28.0 / (15.0 * PI);

/// `mx_oren_nayar_fujii_diffuse_dir_albedo`.
pub(crate) fn fujii_dir_albedo(cos_theta: f32, roughness: f32) -> f32 {
    let a = 1.0 / (1.0 + FUJII_CONSTANT_1 * roughness);
    let b = roughness * a;
    let si = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    let g = si * (cos_theta.clamp(-1.0, 1.0).acos() - si * cos_theta)
        + 2.0 * ((si / cos_theta) * (1.0 - si * si * si) - si) / 3.0;
    a + b * g * PI_INV
}

/// `mx_oren_nayar_fujii_diffuse_avg_albedo`.
pub(crate) fn fujii_avg_albedo(roughness: f32) -> f32 {
    let a = 1.0 / (1.0 + FUJII_CONSTANT_1 * roughness);
    a * (1.0 + FUJII_CONSTANT_2 * roughness)
}

/// A cor que o espalhamento múltiplo devolve (comum ao lóbulo e ao albedo direcional).
pub(crate) fn multi_scatter_colour(color: V3, avg_albedo: f32) -> V3 {
    let k = (1.0 - avg_albedo).max(0.0);
    [0, 1, 2].map(|i| color[i] * color[i] * avg_albedo / (1.0 - color[i] * k))
}

/// `mx_oren_nayar_diffuse_bsdf` com `energy_compensation = true`, closure de REFLEXÃO.
///
/// ⚠️ **O throughput é ZERO, e antes do teste do peso** — é como o GLSL o escreve, e é o que faz uma
/// soma com a difusa (`mx_add_bsdf`) fechar a camada de baixo.
pub(crate) fn oren_nayar_reflection(
    weight: f32,
    color: V3,
    roughness: f32,
    n: V3,
    v: V3,
    l: V3,
    energy_compensation: bool,
) -> Bsdf {
    let dark = Bsdf {
        response: [0.0; 3],
        throughput: [0.0; 3],
    };
    if weight < EPS {
        return dark;
    }
    let n = forward_facing(n, v);
    let ndv = dot(n, v).clamp(EPS, 1.0);
    let ndl = dot(n, l).clamp(EPS, 1.0);
    let ldv = dot(l, v).clamp(EPS, 1.0);
    let diffuse = if energy_compensation {
        oren_nayar_compensated(ndv, ndl, ldv, roughness, color)
    } else {
        scale3(color, oren_nayar_plain(ndv, ndl, ldv, roughness))
    };
    Bsdf {
        response: scale3(diffuse, weight * ndl * PI_INV),
        ..dark
    }
}

/// `mx_oren_nayar_compensated_diffuse` — a metade com compensação de energia (Fujii).
fn oren_nayar_compensated(ndv: f32, ndl: f32, ldv: f32, roughness: f32, color: V3) -> V3 {
    let s = ldv - ndl * ndv;
    let stinv = if s > 0.0 { s / ndl.max(ndv) } else { s };
    let a = 1.0 / (1.0 + FUJII_CONSTANT_1 * roughness);
    let single = scale3(color, a * (1.0 + roughness * stinv));
    let avg = fujii_avg_albedo(roughness);
    let multi = scale3(
        multi_scatter_colour(color, avg),
        (1.0 - fujii_dir_albedo(ndv, roughness)).max(EPS)
            * (1.0 - fujii_dir_albedo(ndl, roughness)).max(EPS)
            / (1.0 - avg).max(EPS),
    );
    add3(single, multi)
}

/// `mx_oren_nayar_diffuse` — o Oren-Nayar CLÁSSICO, **sem** compensação de energia.
///
/// ⚠️ **Ele existe por um valor de omissão:** o `energy_compensation` da nodedef
/// `ND_oren_nayar_diffuse_bsdf` vale **`false`** (`pbrlib/pbrlib_defs.mtlx`), e a única closure do
/// `open_pbr_surface` que o deixa por escrever é a **reflexão da parede fina da subsuperfície** —
/// a base escreve `true`. *Passar-lhe a compensada mudaria o número que o oráculo mede.*
///
/// ⛔ E o `stinv` dele é **`0` quando `s ≤ 0`**, onde a compensada guarda o `s` negativo: são duas
/// leis publicadas diferentes e não uma simplificação.
fn oren_nayar_plain(ndv: f32, ndl: f32, ldv: f32, roughness: f32) -> f32 {
    let s = ldv - ndl * ndv;
    let stinv = if s > 0.0 { s / ndl.max(ndv) } else { 0.0 };
    let sigma2 = roughness * roughness;
    let a = 1.0 - 0.5 * (sigma2 / (sigma2 + 0.33));
    let b = 0.45 * sigma2 / (sigma2 + 0.09);
    a + b * stinv
}

/// `mx_oren_nayar_diffuse_dir_albedo` — o ajuste racional publicado.
///
/// ⚠️ É o ramo **analítico**: o outro (`DIRECTIONAL_ALBEDO_METHOD == 2`) é uma soma de Monte Carlo
/// de 64 amostras por avaliação, e o de omissão do gerador é este.
pub(crate) fn oren_nayar_plain_dir_albedo(ndv: f32, roughness: f32) -> f32 {
    let r2 = roughness * roughness;
    let rx = 1.0 + (-0.4297) * roughness + (-0.7632) * ndv * roughness + 1.4385 * r2;
    let ry = 1.0 + (-0.6076) * roughness + (-0.4993) * ndv * roughness + 2.0315 * r2;
    (rx / ry).clamp(0.0, 1.0)
}
