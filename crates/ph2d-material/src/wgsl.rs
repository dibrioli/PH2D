//! ⭐⭐⭐ **A MESMA LEI, EM WGSL** — o material no dispositivo.
//!
//! # Porque este port é o BARATO desta crate
//!
//! O [`crate::bsdf`] já é um **port fiel, função a função, do GLSL de referência do MaterialX**
//! (Apache-2.0, o artefacto instalado, com os ficheiros de origem nomeados no cabeçalho dele), e o
//! doc daquele módulo declara por escrito que *«os nomes e as constantes são os do GLSL de
//! propósito»*. ⇒ **ir para WGSL é voltar à língua de origem**: a estrutura mapeia uma para uma, e
//! cada função daqui tem o nome da irmã em Rust.
//!
//! ⚠️ *Isto não torna o port livre de erro — torna-o VERIFICÁVEL.* O que prova que ele é a mesma
//! lei é o gate de paridade, que avalia as duas sobre a mesma grelha de entradas.
//!
//! # ⚠️ O CÉU é uma RANHURA, como o campo é no traçador
//!
//! A lei indirecta pergunta ao ambiente duas coisas (`radiance` e `irradiance`), e quem as responde
//! muda: o produto tem o estúdio com a caixa de luz, um gate quer um céu chapado. ⇒ o [`SOURCE`]
//! traz [`ENV_SLOT`] onde essas duas funções entram, exactamente como o `{FIELD}` do
//! `ph2d_field_gpu` recebe a fita da peça. *Uma ranhura é o que impede o gate de medir o céu do
//! produto quando o que ele quer medir é o material.*

use crate::{Rgb, Surface};

/// Onde as duas funções do ambiente entram no [`SOURCE`].
///
/// O que substituir esta marca tem de declarar **exactamente**:
///
/// ```wgsl
/// fn env_radiance(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32>
/// fn env_irradiance(n: vec3<f32>) -> vec3<f32>
/// ```
///
/// ⭐⭐⭐ **O `shrink` é o encolhimento do lóbulo, e ele viaja porque é `f64` na CPU.** O estúdio do
/// produto calcula-o com um logaritmo e uma diferença quase singular (há um ramo explícito para
/// `|α² − 1| < 1e-3`), logo a réplica em `f32` divergiria. ⚠️ **Mas ele é função só do `α`, que é
/// constante por MATERIAL** — há exactamente dois no grafo (o da reflexão principal e o do verniz).
/// ⇒ ele chega pronto, e quem não o usa ignora-o.
///
/// *Uma lei que precisa de `f64` e não varia por pixel não é um bloqueador: é uma constante.*
pub const ENV_SLOT: &str = "{ENV}";

/// Quantos `f32` o [`pack`] escreve — **doze** `vec4`.
///
/// ⚠️ Eram oito até a subsuperfície chegar (`docs/Render3d/10`). ⛔ O último slot — a **curvatura**
/// — é o único que o [`pack`] deixa a **zero de propósito**: ele é do PIXEL e não do material, e
/// quem o escreve é o passe, antes de compor. É o gémeo exacto do [`crate::Surface::at_curvature`].
pub const PACKED: usize = 48;

/// ⭐⭐⭐ **O material pronto, no formato que o WGSL lê.**
///
/// ⚠️ **O [`Surface::prepare`] fica na CPU, e isso não é uma concessão — é a divisão certa.** Ele é
/// por MATERIAL e não por pixel: correr no dispositivo o que já está calculado poria a mesma conta
/// a correr dois milhões de vezes para dar o mesmo número.
#[must_use]
pub fn pack(s: &Surface, lobe: EnvLobe) -> [f32; PACKED] {
    let m = &s.m;
    let mut o = [0.0f32; PACKED];
    let put = |o: &mut [f32; PACKED], i: usize, c: Rgb| {
        o[i] = c[0];
        o[i + 1] = c[1];
        o[i + 2] = c[2];
    };
    put(&mut o, 0, m.base_color);
    o[3] = m.base_weight;
    put(&mut o, 4, m.specular_color);
    o[7] = m.base_metalness;
    put(&mut o, 8, m.coat_color);
    o[11] = m.base_diffuse_roughness;
    put(&mut o, 12, m.emission_color);
    o[15] = m.specular_weight;
    put(&mut o, 16, s.modulated_base_darkening);
    o[19] = m.coat_weight;
    put(&mut o, 20, s.coat_attenuation);
    o[23] = m.coat_ior;
    o[24] = s.modulated_eta_s;
    o[25] = s.main_alpha;
    o[26] = s.coat_alpha;
    o[27] = s.coat_f0;
    o[28] = m.emission_luminance;
    o[29] = lobe.main;
    o[30] = lobe.coat;
    put(&mut o, 32, s.subsurface_colour);
    o[35] = m.subsurface_weight;
    put(&mut o, 36, s.subsurface_mfp);
    o[39] = m.subsurface_scatter_anisotropy;
    put(&mut o, 40, s.thin_brdf_factor);
    o[43] = if m.geometry_thin_walled { 1.0 } else { 0.0 };
    put(&mut o, 44, s.thin_btdf_factor);
    // ⛔ `o[47]` fica a ZERO: é a curvatura, e ela é do PIXEL — ver [`PACKED`].
    o
}

/// ⭐ **O encolhimento do lóbulo para os DOIS `α` do grafo** — ver [`ENV_SLOT`].
///
/// ⚠️ Quem não tem céu direccional (um gate com ambiente analítico) passa [`EnvLobe::IGNORED`], e
/// a ranhura dele ignora o valor. *Um campo que o consumidor não lê não é um campo errado — é um
/// campo que aquele céu não tem.*
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnvLobe {
    /// `lobe_shrink(clamp(main_alpha, EPS, 1))`.
    pub main: f32,
    /// `lobe_shrink(clamp(coat_alpha, EPS, 1))`.
    pub coat: f32,
}

impl EnvLobe {
    /// Para um céu que não o lê.
    pub const IGNORED: Self = Self {
        main: 0.0,
        coat: 0.0,
    };

    /// ⭐⭐ **A composição que os consumidores escreviam à mão** — [`alphas`] e depois
    /// [`crate::lobe_shrink`] em cada metade.
    ///
    /// ⚠️ Ela existe porque o doc dos dois campos acima **nomeia a função por escrito** e a crate não
    /// a dava: com um consumidor isso era dívida, com dois é a lei escrita em dois sítios. Há gate a
    /// afirmar que esta porta é exactamente aquela composição, com o controlo de que os dois `α` se
    /// separam na fixtura.
    #[must_use]
    pub fn of(s: &Surface) -> Self {
        let (main, coat) = alphas(s);
        Self {
            main: crate::lobe_shrink(main),
            coat: crate::lobe_shrink(coat),
        }
    }
}

/// O `α` que o [`EnvLobe`] tem de resolver — a reflexão principal e o verniz, já cortados como as
/// closures os cortam.
#[must_use]
pub fn alphas(s: &Surface) -> (f32, f32) {
    (
        s.main_alpha.clamp(crate::bsdf::EPS, 1.0),
        s.coat_alpha.clamp(crate::bsdf::EPS, 1.0),
    )
}

/// O corpo do sombreador, com a ranhura do ambiente por preencher.
///
/// ⚠️ **A ordem das operações é a do Rust, que é a do GLSL** — um `a*b*c` reassociado move o
/// resultado no último bit, e a barra do gate de paridade é o erro de representação.
pub const SOURCE: &str = r#"
// ── o material empacotado (ver `ph2d_material::wgsl::pack`) ───────────────────────────────────
struct Mat {
    base_color_weight: vec4<f32>,      // rgb = base_color, a = base_weight
    specular_color_metal: vec4<f32>,   // rgb = specular_color, a = base_metalness
    coat_color_diffrough: vec4<f32>,   // rgb = coat_color, a = base_diffuse_roughness
    emission_specweight: vec4<f32>,    // rgb = emission_color, a = specular_weight
    darkening_coatweight: vec4<f32>,   // rgb = modulated_base_darkening, a = coat_weight
    attenuation_coatior: vec4<f32>,    // rgb = coat_attenuation, a = coat_ior
    prepared: vec4<f32>,               // modulated_eta_s, main_alpha, coat_alpha, coat_f0
    emissive: vec4<f32>,               // emission_luminance, shrink_main, shrink_coat, _
    // ⭐⭐⭐ A SUBSUPERFICIE (docs/Render3d/10) — ver `ph2d_material::subsurface`.
    ss_color_weight: vec4<f32>,        // rgb = subsurface_colour (>= 0), a = subsurface_weight
    ss_mfp_aniso: vec4<f32>,           // rgb = subsurface_mfp, a = scatter_anisotropy
    ss_brdf_thin: vec4<f32>,           // rgb = thin_brdf_factor, a = thin_walled (0 ou 1)
    // ⚠️ O `a` daqui NAO vem do `pack`: e' a CURVATURA do PIXEL, escrita pelo passe antes de
    // compor — o gemeo exacto do `Surface::at_curvature` da CPU.
    ss_btdf_curv: vec4<f32>,           // rgb = thin_btdf_factor, a = curvatura
};

const MX_EPS: f32 = 1.0e-8;
const MX_PI: f32 = 3.14159265358979323846;
const MX_HALF_PI: f32 = 1.5707964;
const MX_PI_INV: f32 = 0.31830988618379067154;

// O par que toda closure devolve — `ph2d_material::bsdf::Bsdf`.
struct Bsdf { response: vec3<f32>, throughput: vec3<f32> };

fn bsdf_none() -> Bsdf { return Bsdf(vec3<f32>(0.0), vec3<f32>(1.0)); }
fn bsdf_dark() -> Bsdf { return Bsdf(vec3<f32>(0.0), vec3<f32>(0.0)); }

// ── as leis de composição ─────────────────────────────────────────────────────────────────────
fn bsdf_add(a: Bsdf, b: Bsdf) -> Bsdf {
    return Bsdf(a.response + b.response, max(a.throughput + b.throughput - vec3<f32>(1.0), vec3<f32>(0.0)));
}
fn bsdf_mul_float(a: Bsdf, w: f32) -> Bsdf {
    return Bsdf(a.response * clamp(w, 0.0, 1.0), a.throughput);
}
fn bsdf_mul_color(a: Bsdf, c: vec3<f32>) -> Bsdf {
    return Bsdf(a.response * clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), a.throughput);
}
fn bsdf_layer(top: Bsdf, base: Bsdf) -> Bsdf {
    return Bsdf(top.response + base.response * top.throughput, top.throughput * base.throughput);
}

// ── microfacetas ──────────────────────────────────────────────────────────────────────────────
fn mx_forward_facing(n: vec3<f32>, v: vec3<f32>) -> vec3<f32> {
    if (dot(n, v) < 0.0) { return -n; }
    return n;
}

// ⭐⭐⭐ O GEMEO do `Surface::at_base_color` — a cor do PIXEL entra como `base_color`.
//
// Ver o doc daquela porta para porque multiplicar a resposta pelo albedo DEPOIS esta' errado (ela
// tinge o destaque especular, que e' o que um METAL faz). E' a irmao da curvatura: uma grandeza do
// PIXEL escrita no `Mat` antes de compor.
//
// ⛔⛔ FRONTEIRA DECLARADA: com verniz A ESCURECER (`coat_weight x coat_darkening > 0`) o
// `modulated_base_darkening` depende da cor, e ele NAO e' re-derivavel aqui — o `coat_darkening`
// nao viaja no `pack` (a CPU dobra-o dentro do peso). ⇒ nesse regime este gemeo DIVERGE da CPU, e
// a cura e' o `coat_darkening` ganhar a ranhura livre do `emissive.w`, nao uma conta escrita aqui.
fn mx_at_base_color(m: Mat, rgb: vec3<f32>) -> Mat {
    var o = m;
    o.base_color_weight = vec4<f32>(rgb, m.base_color_weight.a);
    return o;
}

fn mx_ior_to_f0(ior: f32) -> f32 {
    let r = (ior - 1.0) / (ior + 1.0);
    return r * r;
}

// ⭐⭐⭐ **O valor RASANTE exacto da Fresnel dieléctrica** — `1` para todo interface e `0` quando não
// há interface nenhum (`η = 1`). O gémeo do `ph2d_material::bsdf::grazing_dielectric`, que é onde
// vivem o report do dono, a medição e a divergência declarada contra o `1.0` cravado da referência.
//
// ⚠️ Uma IGUALDADE e não um epsilon: `mx_fresnel_dielectric` escreve `g²` como `η·η + c·c − 1`, e
// com `η = 1` e um `c` pequeno a soma cancela para zero e a fórmula devolve `1,0` justamente no caso
// que ela tinha de separar.
fn mx_grazing_dielectric(ior: f32) -> f32 {
    if (ior * ior == 1.0) { return 0.0; }
    return 1.0;
}

fn mx_fresnel_dielectric(cos_theta: f32, ior: f32) -> f32 {
    // ⚠️⚠️ A guarda do interface ausente — a MESMA lei do `mx_grazing_dielectric` acima, e o único
    // sítio onde ela está escrita. Sem ela, com `η = 1` esta fórmula devolve `1,0` no rasante por
    // cancelamento de `f32`.
    if (mx_grazing_dielectric(ior) == 0.0) { return 0.0; }
    let c = cos_theta;
    let g2 = ior * ior + c * c - 1.0;
    if (g2 < 0.0) { return 1.0; }
    let g = sqrt(g2);
    let a = (g - c) / (g + c);
    let b = ((g + c) * c - 1.0) / ((g - c) * c + 1.0);
    return 0.5 * a * a * (1.0 + b * b);
}

fn mx_fresnel_hoffman_schlick(
    cos_theta: f32, f0: vec3<f32>, f82: vec3<f32>, f90: vec3<f32>, exponent: f32
) -> vec3<f32> {
    let COS_THETA_MAX = 1.0 / 7.0;
    let factor = 1.0 / (COS_THETA_MAX * pow(1.0 - COS_THETA_MAX, 6.0));
    let x = clamp(cos_theta, 0.0, 1.0);
    let at_max = mix(f0, f90, pow(1.0 - COS_THETA_MAX, exponent));
    let a = at_max * (vec3<f32>(1.0) - f82) * factor;
    let base = mix(f0, f90, pow(1.0 - x, exponent));
    let tail = x * pow(1.0 - x, 6.0);
    return base - a * tail;
}

fn mx_ggx_dir_albedo(ndv: f32, alpha: f32, f0: vec3<f32>, f90: vec3<f32>) -> vec3<f32> {
    let x = ndv;
    let y = alpha;
    let x2 = x * x;
    let y2 = y * y;
    let t = array<f32, 9>(1.0, x, y, x * y, x2, y2, x2 * y, x * y2, x2 * y2);
    var r = vec4<f32>(0.0);
    // A mesma tabela do `ph2d_material::bsdf::ggx_dir_albedo`, linha a linha.
    r = r + vec4<f32>( 0.1003,  0.9345,   1.0,     1.0  ) * t[0];
    r = r + vec4<f32>(-0.6303, -2.323,  -1.765,   0.2281) * t[1];
    r = r + vec4<f32>( 9.748,   2.229,   8.263,  15.94  ) * t[2];
    r = r + vec4<f32>(-2.038,  -3.748,  11.53,  -55.83  ) * t[3];
    r = r + vec4<f32>(29.34,    1.424,  28.96,   13.08  ) * t[4];
    r = r + vec4<f32>(-8.245,  -0.7684, -7.507,  41.26  ) * t[5];
    r = r + vec4<f32>(-26.44,   1.436, -36.11,   54.9   ) * t[6];
    r = r + vec4<f32>(19.99,    0.2913, 15.86,  300.2   ) * t[7];
    r = r + vec4<f32>(-5.448,   0.6286, 33.37, -285.1   ) * t[8];
    let a = clamp(r.x / r.z, 0.0, 1.0);
    let b = clamp(r.y / r.w, 0.0, 1.0);
    return f0 * a + f90 * b;
}

fn mx_ggx_energy_compensation(ndv: f32, alpha: f32, fss: vec3<f32>) -> vec3<f32> {
    let ess = mx_ggx_dir_albedo(ndv, alpha, vec3<f32>(1.0), vec3<f32>(1.0)).x;
    return vec3<f32>(1.0) + fss * ((1.0 - ess) / ess);
}

// ⚠️ `|H × N|²` e NÃO `1 − (H·N)²` — a subtracção perde os dígitos que um lóbulo estreito precisa,
// e o Rust tem a medição ao lado (`1,12e-4` relativo contra o oráculo, acima da barra).
fn mx_ggx_ndf_isotropic(h: vec3<f32>, n: vec3<f32>, alpha: f32) -> f32 {
    let t = cross(h, n);
    let ndh = dot(h, n);
    let he2 = dot(t, t) / (alpha * alpha);
    let denom = he2 + ndh * ndh;
    return 1.0 / (MX_PI * alpha * alpha * denom * denom);
}

fn mx_ggx_smith_g2(ndl: f32, ndv: f32, alpha: f32) -> f32 {
    let a2 = alpha * alpha;
    let lambda_l = sqrt(a2 + (1.0 - a2) * ndl * ndl);
    let lambda_v = sqrt(a2 + (1.0 - a2) * ndv * ndv);
    return 2.0 * ndl * ndv / (lambda_l * ndv + lambda_v * ndl);
}

struct Facet { ndv: f32, alpha: f32, vdh: f32, d: f32, g: f32 };

fn mx_facet(n_in: vec3<f32>, v: vec3<f32>, l: vec3<f32>, alpha_in: f32) -> Facet {
    let n = mx_forward_facing(n_in, v);
    let ndv = clamp(dot(n, v), MX_EPS, 1.0);
    let alpha = clamp(alpha_in, MX_EPS, 1.0);
    let h = normalize(l + v);
    let ndl = clamp(dot(n, l), MX_EPS, 1.0);
    let vdh = clamp(dot(v, h), MX_EPS, 1.0);
    return Facet(ndv, alpha, vdh, mx_ggx_ndf_isotropic(h, n, alpha), mx_ggx_smith_g2(ndl, ndv, alpha));
}

fn mx_dielectric_reflection(
    weight: f32, tint: vec3<f32>, ior: f32, alpha: f32, n: vec3<f32>, v: vec3<f32>, l: vec3<f32>
) -> Bsdf {
    if (weight < MX_EPS) { return bsdf_none(); }
    let f = mx_facet(n, v, l, alpha);
    let fresnel = vec3<f32>(mx_fresnel_dielectric(f.vdh, ior));
    let comp = mx_ggx_energy_compensation(f.ndv, f.alpha, fresnel);
    let f0 = mx_ior_to_f0(ior);
    // ⭐⭐⭐ O `F90` sai do próprio índice — o gémeo do `ph2d_material::bsdf::grazing_dielectric`.
    let dir_albedo = mx_ggx_dir_albedo(f.ndv, f.alpha, vec3<f32>(f0), vec3<f32>(mx_grazing_dielectric(ior))) * comp;
    return Bsdf(
        fresnel * comp * max(tint, vec3<f32>(0.0)) * (f.d * f.g * weight / (4.0 * f.ndv)),
        vec3<f32>(1.0) - dir_albedo * weight
    );
}

fn mx_schlick_reflection(
    weight: f32, color0: vec3<f32>, color82: vec3<f32>, color90: vec3<f32>, exponent: f32,
    alpha: f32, n: vec3<f32>, v: vec3<f32>, l: vec3<f32>
) -> Bsdf {
    if (weight < MX_EPS) { return bsdf_none(); }
    let c0 = max(color0, vec3<f32>(0.0));
    let c82 = max(color82, vec3<f32>(0.0));
    let c90 = max(color90, vec3<f32>(0.0));
    let f = mx_facet(n, v, l, alpha);
    let fresnel = mx_fresnel_hoffman_schlick(f.vdh, c0, c82, c90, exponent);
    let comp = mx_ggx_energy_compensation(f.ndv, f.alpha, fresnel);
    let dir_albedo = mx_ggx_dir_albedo(f.ndv, f.alpha, c0, c90) * comp;
    let avg = (dir_albedo.x + dir_albedo.y + dir_albedo.z) * (1.0 / 3.0);
    return Bsdf(
        fresnel * comp * (f.d * f.g * weight / (4.0 * f.ndv)),
        vec3<f32>(1.0 - avg * weight)
    );
}

// ⚠️⚠️ **DERIVADAS, nunca transcritas.** A primeira redacção deste ficheiro escreveu-as de
// cabeça e as DUAS saíram erradas no 6.º dígito — o `FUJII_2` por um FACTOR (`0,0626` contra
// `0,0725`). Escritas como a expressão que o Rust usa, não há o que transcrever.
const FUJII_1: f32 = 0.5 - 2.0 / (3.0 * MX_PI);
const FUJII_2: f32 = 2.0 / 3.0 - 28.0 / (15.0 * MX_PI);

fn mx_fujii_dir_albedo(cos_theta: f32, roughness: f32) -> f32 {
    let a = 1.0 / (1.0 + FUJII_1 * roughness);
    let b = roughness * a;
    let si = sqrt(max(1.0 - cos_theta * cos_theta, 0.0));
    let g = si * (acos(clamp(cos_theta, -1.0, 1.0)) - si * cos_theta)
          + 2.0 * ((si / cos_theta) * (1.0 - si * si * si) - si) / 3.0;
    return a + b * g * MX_PI_INV;
}

fn mx_fujii_avg_albedo(roughness: f32) -> f32 {
    let a = 1.0 / (1.0 + FUJII_1 * roughness);
    return a * (1.0 + FUJII_2 * roughness);
}

fn mx_multi_scatter_colour(color: vec3<f32>, avg_albedo: f32) -> vec3<f32> {
    let k = max(1.0 - avg_albedo, 0.0);
    return color * color * avg_albedo / (vec3<f32>(1.0) - color * k);
}

// ⚠️ O Oren-Nayar CLASSICO, sem compensacao — o valor de omissao da nodedef e' `false`, e a
// reflexao da parede fina da subsuperficie e' a UNICA closure do OpenPBR que o deixa por escrever.
// ⛔ E o `stinv` dele e' `0` quando `s <= 0`, onde a compensada guarda o `s` negativo.
fn mx_oren_nayar_plain(ndv: f32, ndl: f32, ldv: f32, roughness: f32) -> f32 {
    let s = ldv - ndl * ndv;
    var stinv = 0.0;
    if (s > 0.0) { stinv = s / max(ndl, ndv); }
    let sigma2 = roughness * roughness;
    let a = 1.0 - 0.5 * (sigma2 / (sigma2 + 0.33));
    let b = 0.45 * sigma2 / (sigma2 + 0.09);
    return a + b * stinv;
}

fn mx_oren_nayar_plain_dir_albedo(ndv: f32, roughness: f32) -> f32 {
    let r2 = roughness * roughness;
    let rx = 1.0 + (-0.4297) * roughness + (-0.7632) * ndv * roughness + 1.4385 * r2;
    let ry = 1.0 + (-0.6076) * roughness + (-0.4993) * ndv * roughness + 2.0315 * r2;
    return clamp(rx / ry, 0.0, 1.0);
}

fn mx_oren_nayar_compensated(ndv: f32, ndl: f32, ldv: f32, roughness: f32, color: vec3<f32>) -> vec3<f32> {
    let s = ldv - ndl * ndv;
    var stinv = s;
    if (s > 0.0) { stinv = s / max(ndl, ndv); }
    let a = 1.0 / (1.0 + FUJII_1 * roughness);
    let single = color * (a * (1.0 + roughness * stinv));
    let avg = mx_fujii_avg_albedo(roughness);
    let multi = mx_multi_scatter_colour(color, avg)
        * (max(1.0 - mx_fujii_dir_albedo(ndv, roughness), MX_EPS)
         * max(1.0 - mx_fujii_dir_albedo(ndl, roughness), MX_EPS)
         / max(1.0 - avg, MX_EPS));
    return single + multi;
}

fn mx_oren_nayar_reflection(
    weight: f32, color: vec3<f32>, roughness: f32, n_in: vec3<f32>, v: vec3<f32>, l: vec3<f32>,
    energy_compensation: bool
) -> Bsdf {
    if (weight < MX_EPS) { return bsdf_dark(); }
    let n = mx_forward_facing(n_in, v);
    let ndv = clamp(dot(n, v), MX_EPS, 1.0);
    let ndl = clamp(dot(n, l), MX_EPS, 1.0);
    let ldv = clamp(dot(l, v), MX_EPS, 1.0);
    var diffuse = color * mx_oren_nayar_plain(ndv, ndl, ldv, roughness);
    if (energy_compensation) {
        diffuse = mx_oren_nayar_compensated(ndv, ndl, ldv, roughness, color);
    }
    return Bsdf(diffuse * (weight * ndl * MX_PI_INV), vec3<f32>(0.0));
}

// ── as closures INDIRECTAS (`mx_environment_radiance`, método PREFILTER) ──────────────────────
fn mx_env_mirror(n: vec3<f32>, v: vec3<f32>, alpha: f32, shrink: f32, dir_albedo: vec3<f32>) -> vec3<f32> {
    let d = 2.0 * dot(n, v);
    let mirror = d * n - v;
    return env_radiance(mirror, alpha, shrink) * dir_albedo;
}

fn mx_dielectric_indirect(
    weight: f32, tint: vec3<f32>, ior: f32, alpha_in: f32, shrink: f32, n_in: vec3<f32>, v: vec3<f32>
) -> Bsdf {
    if (weight < MX_EPS) { return bsdf_none(); }
    let n = mx_forward_facing(n_in, v);
    let ndv = clamp(dot(n, v), MX_EPS, 1.0);
    let alpha = clamp(alpha_in, MX_EPS, 1.0);
    let fresnel = vec3<f32>(mx_fresnel_dielectric(ndv, ior));
    let comp = mx_ggx_energy_compensation(ndv, alpha, fresnel);
    let f0 = mx_ior_to_f0(ior);
    // ⭐⭐⭐ O `F90` sai do próprio índice — o gémeo do `ph2d_material::bsdf::grazing_dielectric`.
    let fg = mx_ggx_dir_albedo(ndv, alpha, vec3<f32>(f0), vec3<f32>(mx_grazing_dielectric(ior)));
    let dir_albedo = fg * comp;
    let li = mx_env_mirror(n, v, alpha, shrink, fg);
    return Bsdf(li * max(tint, vec3<f32>(0.0)) * comp * weight, vec3<f32>(1.0) - dir_albedo * weight);
}

fn mx_schlick_indirect(
    weight: f32, color0: vec3<f32>, color82: vec3<f32>, color90: vec3<f32>, exponent: f32,
    alpha_in: f32, shrink: f32, n_in: vec3<f32>, v: vec3<f32>
) -> Bsdf {
    if (weight < MX_EPS) { return bsdf_none(); }
    let c0 = max(color0, vec3<f32>(0.0));
    let c82 = max(color82, vec3<f32>(0.0));
    let c90 = max(color90, vec3<f32>(0.0));
    let n = mx_forward_facing(n_in, v);
    let ndv = clamp(dot(n, v), MX_EPS, 1.0);
    let alpha = clamp(alpha_in, MX_EPS, 1.0);
    let fresnel = mx_fresnel_hoffman_schlick(ndv, c0, c82, c90, exponent);
    let comp = mx_ggx_energy_compensation(ndv, alpha, fresnel);
    let fg = mx_ggx_dir_albedo(ndv, alpha, c0, c90);
    let dir_albedo = fg * comp;
    let avg = (dir_albedo.x + dir_albedo.y + dir_albedo.z) * (1.0 / 3.0);
    let li = mx_env_mirror(n, v, alpha, shrink, fg);
    return Bsdf(li * comp * weight, vec3<f32>(1.0 - avg * weight));
}

fn mx_oren_nayar_indirect(
    weight: f32, color: vec3<f32>, roughness: f32, n_in: vec3<f32>, v: vec3<f32>,
    energy_compensation: bool
) -> Bsdf {
    if (weight < MX_EPS) { return bsdf_dark(); }
    let n = mx_forward_facing(n_in, v);
    let ndv = clamp(dot(n, v), MX_EPS, 1.0);
    var albedo = color * mx_oren_nayar_plain_dir_albedo(ndv, roughness);
    if (energy_compensation) {
        let dir_albedo = mx_fujii_dir_albedo(ndv, roughness);
        let avg = mx_fujii_avg_albedo(roughness);
        albedo = mix(mx_multi_scatter_colour(color, avg), color, vec3<f32>(dir_albedo));
    }
    return Bsdf(env_irradiance(n) * albedo * weight, vec3<f32>(0.0));
}

// ── a SUBSUPERFICIE (`ph2d_material::subsurface`) ─────────────────────────────────────────────
//
// ⚠️ A `translucent` NEGA a normal e nao a vira para o observador: a luz que ela devolve e' a que
// entra pelas COSTAS, e e' essa linha que faz uma folha acender com o sol atras.
fn mx_translucent(weight: f32, color: vec3<f32>, n_in: vec3<f32>, l: vec3<f32>, direto: bool) -> Bsdf {
    if (weight < MX_EPS) { return bsdf_dark(); }
    let n = -n_in;
    if (direto) {
        let ndl = clamp(dot(n, l), 0.0, 1.0);
        return Bsdf(color * (weight * ndl * MX_PI_INV), vec3<f32>(0.0));
    }
    return Bsdf(env_irradiance(n) * color * weight, vec3<f32>(0.0));
}

fn mx_burley_profile(dist: f32, shape: vec3<f32>) -> vec3<f32> {
    let num1 = exp(-shape * dist);
    let num2 = exp(-shape * dist / 3.0);
    return (num1 + num2) / max(dist, MX_EPS);
}

// ⚠️ O `acos` e' CORTADO — o do GLSL de referencia nao e', e `dot` de dois unitarios le
// `1,0000001` em f32: um pixel NaN e um pixel legitimamente preto leem-se iguais.
fn mx_integrate_burley(n: vec3<f32>, l: vec3<f32>, radius: f32, mfp: vec3<f32>) -> vec3<f32> {
    let theta = acos(clamp(dot(n, l), -1.0, 1.0));
    let shape = vec3<f32>(1.0) / max(mfp, vec3<f32>(0.1));
    var sum_d = vec3<f32>(0.0);
    var sum_r = vec3<f32>(0.0);
    let width = (2.0 * MX_PI) / 32.0;
    let meia = width * 0.5;
    for (var i: i32 = 0; i < 32; i = i + 1) {
        let x = -MX_PI + (f32(i) + 0.5) * width;
        let dist = radius * abs(2.0 * sin(x * 0.5));
        let r = mx_burley_profile(dist, shape);
        // ⭐ O cosseno é integrado EXACTAMENTE dentro da célula — gémeo do `integrate_burley` da
        // CPU, que traz a medição e a divergência declarada contra o ponto médio do oráculo.
        let a = clamp(theta + x - meia, -MX_HALF_PI, MX_HALF_PI);
        let b = clamp(theta + x + meia, -MX_HALF_PI, MX_HALF_PI);
        sum_d = sum_d + r * ((sin(b) - sin(a)) / width);
        sum_r = sum_r + r;
    }
    return sum_d / sum_r;
}

fn mx_subsurface_thick(
    weight: f32, color: vec3<f32>, mfp: vec3<f32>, curvature: f32,
    n_in: vec3<f32>, v: vec3<f32>, l: vec3<f32>, direto: bool
) -> Bsdf {
    if (weight < MX_EPS) { return bsdf_dark(); }
    let n = mx_forward_facing(n_in, v);
    if (direto) {
        let radius = 1.0 / max(curvature, 0.01);
        let sss = color * mx_integrate_burley(n, l, radius, mfp) * MX_PI_INV;
        return Bsdf(sss * weight, vec3<f32>(0.0));
    }
    return Bsdf(env_irradiance(n) * color * weight, vec3<f32>(0.0));
}

fn bsdf_mix(fg: Bsdf, bg: Bsdf, t: f32) -> Bsdf {
    return Bsdf(mix(bg.response, fg.response, vec3<f32>(t)),
                mix(bg.throughput, fg.throughput, vec3<f32>(t)));
}

// ── a COMPOSIÇÃO do grafo gerado, closure a closure ──────────────────────────────────────────
// `direto = 1` avalia a luz que vem de `l`; `direto = 0` avalia o céu e ignora `l`.
fn mx_compose(m: Mat, n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, direto: bool) -> vec3<f32> {
    let base_color = m.base_color_weight.rgb;
    let base_weight = m.base_color_weight.a;
    let specular_color = m.specular_color_metal.rgb;
    let base_metalness = m.specular_color_metal.a;
    let coat_color = m.coat_color_diffrough.rgb;
    let base_diffuse_roughness = m.coat_color_diffrough.a;
    let specular_weight = m.emission_specweight.a;
    let coat_weight = m.darkening_coatweight.a;
    let coat_ior = m.attenuation_coatior.a;
    let modulated_eta_s = m.prepared.x;
    let main_alpha = m.prepared.y;
    let coat_alpha = m.prepared.z;

    var metal_c: Bsdf;
    if (direto) {
        metal_c = mx_schlick_reflection(specular_weight, base_color * base_weight, specular_color,
                                        vec3<f32>(1.0), 5.0, main_alpha, n, v, l);
    } else {
        metal_c = mx_schlick_indirect(specular_weight, base_color * base_weight, specular_color,
                                      vec3<f32>(1.0), 5.0, main_alpha, m.emissive.y, n, v);
    }
    let metal = bsdf_add(bsdf_none(), metal_c);
    let base_fg = bsdf_mul_float(metal, base_metalness);

    var diel_c: Bsdf;
    if (direto) {
        diel_c = mx_dielectric_reflection(1.0, specular_color, modulated_eta_s, main_alpha, n, v, l);
    } else {
        diel_c = mx_dielectric_indirect(1.0, specular_color, modulated_eta_s, main_alpha, m.emissive.y, n, v);
    }
    let dielectric_reflection = bsdf_add(bsdf_none(), diel_c);

    // A transmissão com peso zero: resposta zero, throughput um.
    let transmission = bsdf_mul_float(bsdf_none(), 0.0);
    let base_colour = max(base_color, vec3<f32>(0.0));
    var diffuse_c: Bsdf;
    if (direto) {
        diffuse_c = mx_oren_nayar_reflection(base_weight, base_colour, base_diffuse_roughness, n, v, l, true);
    } else {
        diffuse_c = mx_oren_nayar_indirect(base_weight, base_colour, base_diffuse_roughness, n, v, true);
    }
    // ⭐⭐⭐ A SUBSUPERFICIE — ver `Surface::subsurface`. O ramo do selector da' o mesmo que o `mix`
    // do grafo porque o selector vale exactamente 0 ou 1 e nenhum lado pode ser NaN (o `acos` e'
    // cortado), e ele poupa o laco de 32 termos do Burley em toda peca de parede fina.
    let ss_weight = m.ss_color_weight.a;
    var subsurface = bsdf_dark();
    if (ss_weight > 0.0) {
        let ss_colour = m.ss_color_weight.rgb;
        if (m.ss_brdf_thin.a > 0.5) {
            var refl: Bsdf;
            if (direto) {
                refl = mx_oren_nayar_reflection(1.0, ss_colour, base_diffuse_roughness, n, v, l, false);
            } else {
                refl = mx_oren_nayar_indirect(1.0, ss_colour, base_diffuse_roughness, n, v, false);
            }
            let reflection = bsdf_mul_color(refl, m.ss_brdf_thin.rgb);
            let transm = bsdf_mul_color(mx_translucent(1.0, ss_colour, n, l, direto), m.ss_btdf_curv.rgb);
            subsurface = bsdf_mix(reflection, transm, 0.5);
        } else {
            subsurface = mx_subsurface_thick(1.0, ss_colour, m.ss_mfp_aniso.rgb,
                                             m.ss_btdf_curv.a, n, v, l, direto);
        }
    }
    let opaque = bsdf_mix(subsurface, diffuse_c, ss_weight);
    let substrate = bsdf_add(transmission, bsdf_mul_float(opaque, 1.0));
    let dielectric_base = bsdf_layer(dielectric_reflection, substrate);

    let base_substrate = bsdf_add(base_fg, bsdf_mul_float(dielectric_base, 1.0 - base_metalness));
    let attenuated = bsdf_mul_color(
        bsdf_mul_color(base_substrate, m.darkening_coatweight.rgb),
        m.attenuation_coatior.rgb
    );
    var coat_c: Bsdf;
    if (direto) {
        coat_c = mx_dielectric_reflection(coat_weight, vec3<f32>(1.0), coat_ior, coat_alpha, n, v, l);
    } else {
        coat_c = mx_dielectric_indirect(coat_weight, vec3<f32>(1.0), coat_ior, coat_alpha, m.emissive.z, n, v);
    }
    let coat_layer = bsdf_layer(coat_c, attenuated);
    return bsdf_layer(bsdf_none(), coat_layer).response;
}

/// A radiância que uma luz DIRECCIONAL devolve — `Surface::direct`.
fn mx_direct(m: Mat, n: vec3<f32>, v: vec3<f32>, to_light: vec3<f32>, radiance: vec3<f32>) -> vec3<f32> {
    return radiance * mx_compose(m, n, v, to_light, true);
}

// ⭐⭐⭐ **A MESMA lei com DUAS radiancias: a subsuperficie le a sua** — o gemeo do
// `ph2d_material::Surface::direct_sss`, linha a linha.
//
// A luz que uma peca translucida devolve NAO entrou por este ponto: ela entrou a' volta dele e
// espalhou-se por baixo da superficie. ⇒ a visibilidade que esta closure le e' a da VIZINHANCA, e e'
// isso que faz a borda de uma sombra num jade ser MOLE enquanto a do especular ao lado continua dura.
//
// ⭐⭐ **A separacao e' EXACTA e nao uma aproximacao:** tudo o que esta' a jusante da mistura da
// subsuperficie e' LINEAR na resposta dela (o `mix`, o `add` e o `layer`, cujo throughput nao depende
// da radiancia) ⇒ compor COM e SEM o peso de subsuperficie e ficar com a diferenca da' exactamente a
// parcela dela atraves de toda a pilha. *Nao e' preciso partir o `mx_compose` em dois.*
//
// ⚠️ **Com as duas radiancias iguais ele e' o `mx_direct`, AO BIT** — o braco curto sai antes de
// compor a segunda vez, e ele e' o caminho de todo material sem subsuperficie, que assim nao paga
// nada.
fn mx_direct_sss(
    m: Mat, n: vec3<f32>, v: vec3<f32>, to_light: vec3<f32>, radiance: vec3<f32>, sss: vec3<f32>
) -> vec3<f32> {
    let cheio = mx_compose(m, n, v, to_light, true);
    if (all(sss == radiance) || m.ss_color_weight.a <= 0.0) { return radiance * cheio; }
    // ⚠️ SO' o peso muda: tudo o que o `prepare` ja' derivou (o indice modulado, os `alpha`, a cor da
    // subsuperficie, os factores de parede fina) fica, exactamente como do lado da CPU.
    var sem_ss = m;
    sem_ss.ss_color_weight.a = 0.0;
    let sem = mx_compose(sem_ss, n, v, to_light, true);
    return radiance * sem + sss * (cheio - sem);
}

/// A radiância que o CÉU devolve — `Surface::indirect`.
fn mx_indirect(m: Mat, n: vec3<f32>, v: vec3<f32>) -> vec3<f32> {
    return mx_compose(m, n, v, vec3<f32>(0.0, 0.0, 1.0), false);
}

/// A EMISSÃO, filtrada pelo Fresnel do verniz — `Surface::emission`.
fn mx_emission(m: Mat, n: vec3<f32>, v: vec3<f32>) -> vec3<f32> {
    let luminance = m.emissive.x;
    if (luminance == 0.0) { return vec3<f32>(0.0); }
    let uncoated = m.emission_specweight.rgb * luminance;
    let nf = mx_forward_facing(n, v);
    let ndv = clamp(dot(nf, v), MX_EPS, 1.0);
    let x = pow(clamp(1.0 - ndv, 0.0, 1.0), 5.0);
    let fresnel = mix(vec3<f32>(1.0 - m.prepared.w), vec3<f32>(0.0), vec3<f32>(x));
    let coated = uncoated * m.coat_color_diffrough.rgb * fresnel;
    return mix(uncoated, coated, vec3<f32>(m.darkening_coatweight.a));
}
{ENV}
"#;
