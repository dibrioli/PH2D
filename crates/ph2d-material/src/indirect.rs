//! **As closures INDIRECTAS** — a mesma superfície, iluminada pelo céu em vez de uma luz.
//!
//! # Proveniência e licença
//!
//! Porta Apache-2.0 do MaterialX 1.39.5 (ver `bsdf.rs`): os ramos `CLOSURE_TYPE_INDIRECT` de
//! `mx_dielectric_bsdf.glsl`, `mx_generalized_schlick_bsdf.glsl` e `mx_oren_nayar_diffuse_bsdf.glsl`,
//! mais o `mx_environment_radiance` do método **`PREFILTER`** (`lib/mx_environment_prefilter.glsl`)
//! e o `mx_oren_nayar_compensated_diffuse_dir_albedo` de `lib/mx_microfacet_diffuse.glsl`.
//!
//! ⚠️ **É o `PREFILTER` e não o `FIS`**, e a escolha é de desenho: o `FIS` amostra um mapa de ambiente
//! com importância (é ruído e custo por pixel), e o `PREFILTER` separa a pergunta em duas — *que luz
//! chega pela direcção espelhada, pré-filtrada para esta rugosidade* (o [`Environment`]) × *quanto
//! dela a superfície devolve* (o albedo direcional). Um céu analítico responde a primeira de graça.

use crate::Environment;
use crate::bsdf::{
    self, Bsdf, EPS, V3, add3, dot, forward_facing, fresnel_dielectric, fresnel_hoffman_schlick,
    ggx_dir_albedo, ggx_energy_compensation, ior_to_f0, mix3, mul3, scale3,
};

/// `mx_environment_radiance` (`PREFILTER`): a luz da direcção espelhada, vezes o albedo direcional.
///
/// `n` já vem virado para o observador, como o GLSL o recebe.
fn radiance(n: V3, v: V3, alpha: f32, dir_albedo: V3, env: &dyn Environment) -> V3 {
    // `L = -reflect(V, N) = 2 (N·V) N − V`.
    let d = 2.0 * dot(n, v);
    let mirror = [d * n[0] - v[0], d * n[1] - v[1], d * n[2] - v[2]];
    mul3(env.radiance(mirror, alpha), dir_albedo)
}

/// `mx_dielectric_bsdf`, closure INDIRECTA, sem película fina.
pub(crate) fn dielectric(
    weight: f32,
    tint: V3,
    ior: f32,
    alpha: f32,
    n: V3,
    v: V3,
    env: &dyn Environment,
) -> Bsdf {
    if weight < EPS {
        return Bsdf::NONE;
    }
    let n = forward_facing(n, v);
    let ndv = dot(n, v).clamp(EPS, 1.0);
    let alpha = alpha.clamp(EPS, 1.0);
    let fresnel = [fresnel_dielectric(ndv, ior); 3];
    let comp = ggx_energy_compensation(ndv, alpha, fresnel);
    let f0 = ior_to_f0(ior);
    // ⭐⭐⭐ **O `F90` SAI DO PRÓPRIO ÍNDICE** — ver [`bsdf::grazing_dielectric`] para o report do
    // dono, a medição (`0,4296` de céu num realce DESLIGADO) e a divergência declarada contra o
    // `1.0` cravado da referência. ⚠️ Para todo índice a sério ele é `1,0` **ao bit**.
    let fg = ggx_dir_albedo(ndv, alpha, [f0; 3], [bsdf::grazing_dielectric(ior); 3]);
    let dir_albedo = mul3(fg, comp);
    let li = radiance(n, v, alpha, fg, env);
    let tint = tint.map(|t| t.max(0.0));
    Bsdf {
        response: scale3(mul3(mul3(li, tint), comp), weight),
        throughput: add3([1.0; 3], scale3(dir_albedo, -weight)),
    }
}

/// `mx_generalized_schlick_bsdf`, closure INDIRECTA, sem película fina.
///
/// ⚠️ **O `F82` entra no Fresnel da compensação e NÃO no albedo do ambiente** — o `FresnelData` de
/// Schlick responde ao `mx_ggx_dir_albedo` só com `F0` e `F90`. É o GLSL, e não um descuido deste
/// port.
#[allow(clippy::too_many_arguments)] // a assinatura é a do GLSL, parâmetro a parâmetro
pub(crate) fn schlick(
    weight: f32,
    color0: V3,
    color82: V3,
    color90: V3,
    exponent: f32,
    alpha: f32,
    n: V3,
    v: V3,
    env: &dyn Environment,
) -> Bsdf {
    if weight < EPS {
        return Bsdf::NONE;
    }
    let max0 = |c: V3| c.map(|x| x.max(0.0));
    let (c0, c82, c90) = (max0(color0), max0(color82), max0(color90));
    let n = forward_facing(n, v);
    let ndv = dot(n, v).clamp(EPS, 1.0);
    let alpha = alpha.clamp(EPS, 1.0);
    let fresnel = fresnel_hoffman_schlick(ndv, c0, c82, c90, exponent);
    let comp = ggx_energy_compensation(ndv, alpha, fresnel);
    let fg = ggx_dir_albedo(ndv, alpha, c0, c90);
    let dir_albedo = mul3(fg, comp);
    let avg = (dir_albedo[0] + dir_albedo[1] + dir_albedo[2]) * (1.0 / 3.0);
    let li = radiance(n, v, alpha, fg, env);
    Bsdf {
        response: scale3(mul3(li, comp), weight),
        throughput: [1.0 - avg * weight; 3],
    }
}

/// `mx_oren_nayar_diffuse_bsdf` com compensação de energia, closure INDIRECTA.
pub(crate) fn oren_nayar(
    weight: f32,
    color: V3,
    roughness: f32,
    n: V3,
    v: V3,
    env: &dyn Environment,
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
    let albedo = if energy_compensation {
        let dir_albedo = bsdf::fujii_dir_albedo(ndv, roughness);
        let avg = bsdf::fujii_avg_albedo(roughness);
        mix3(bsdf::multi_scatter_colour(color, avg), color, dir_albedo)
    } else {
        // ⚠️ Ver [`bsdf::oren_nayar_plain_dir_albedo`]: a omissão da nodedef é SEM compensação, e a
        // parede fina da subsuperfície é a única closure do OpenPBR que a deixa por escrever.
        scale3(color, bsdf::oren_nayar_plain_dir_albedo(ndv, roughness))
    };
    Bsdf {
        response: scale3(mul3(env.irradiance(n), albedo), weight),
        ..dark
    }
}
