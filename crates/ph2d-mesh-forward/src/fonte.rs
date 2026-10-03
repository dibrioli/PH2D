//! ⭐ **A FONTE DO SHADER** — as três leis da casa costuradas no desenhista, por ranhura.
//!
//! ⚠️ Cada ranhura é substituída UMA vez e há gate a exigir que nenhuma `{…}` sobre
//! (`nenhuma_ranhura_fica_por_preencher`): uma marca esquecida não compila — mas uma marca que por
//! acaso fosse WGSL válido passaria em silêncio.

/// O corpo do desenhista.
const FORWARD: &str = include_str!("forward.wgsl");

/// ⭐ **O passe que codifica o pixel para o ecrã, com o halo por cima** — a lei do brilho e a da
/// composição são as do `ph2d_bloom::wgsl` (uma porta para os três motores), o olhar é o da casa.
#[must_use]
pub(crate) fn ecra() -> String {
    format!(
        "{}\n{}\n{}\n",
        ph2d_bloom::wgsl::fonte(include_str!("ecra.wgsl")),
        ph2d_bloom::wgsl::COMPOE,
        ph2d_view_transform::wgsl::SOURCE
    )
}

/// ⭐ **A cadeia do brilho** (descer e subir) — a lei do `ph2d_bloom::wgsl` com a porta de leitura
/// desta crate no meio.
#[must_use]
pub(crate) fn brilho() -> String {
    ph2d_bloom::wgsl::fonte(include_str!("brilho.wgsl"))
}

fn f(x: f32) -> String {
    let s = format!("{x:?}");
    if s.contains('.') || s.contains('e') {
        s
    } else {
        format!("{s}.0")
    }
}

/// ⭐ **O shader dos objetos, do chão e da sombra**, com o céu de quem chama.
#[must_use]
pub fn fonte(ambiente: &crate::Ambiente<'_>) -> String {
    let material = ph2d_material::wgsl::SOURCE.replace(
        ph2d_material::wgsl::ENV_SLOT,
        "fn env_radiance(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32> {\n    \
         return env_radiance_da_cena(dir, alpha, shrink);\n}\n\
         fn env_irradiance(n: vec3<f32>) -> vec3<f32> {\n    return env_irradiance_da_cena(n);\n}\n",
    );
    let material = format!("{material}\n{}", ph2d_material::wgsl::por_pixel());
    FORWARD
        .replace("{MATERIAL}", &material)
        .replace("{TRIPLANAR}", ph2d_triplanar::wgsl::fonte())
        .replace("{CONTACTO}", &ph2d_contacto::wgsl::fonte())
        .replace("{CHAO_TAPA}", &crate::chao_tapa::wgsl())
        .replace(
            "{MAX_CONTACTO}",
            &crate::gpu_contacto::MAX_CONTACTO.to_string(),
        )
        .replace("{LADO_CONTACTO}", &ph2d_contacto::LADO.to_string())
        .replace("{COL_TEX}", &crate::MATERIAL_V4.to_string())
        .replace("{AMBIENTE}", ambiente.wgsl)
        .replace("{OLHAR}", ph2d_view_transform::wgsl::SOURCE)
        .replace("{ESTILO}", &ph2d_style::wgsl::source())
        .replace("{CEU_FOTO}", &ph2d_sky::wgsl::fonte())
        .replace("{MAX_LUZES2}", &(2 * crate::MAX_LUZES).to_string())
        .replace("{MAX_LUZES}", &crate::MAX_LUZES.to_string())
        .replace("{TAB_W}", &crate::TAB_W.to_string())
        .replace("{PISO_LUZ}", &f(ambiente.piso_luz))
}
