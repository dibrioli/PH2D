//! ⭐ **A FONTE DO SHADER** — as três leis da casa costuradas no desenhista, por ranhura.
//!
//! ⚠️ Cada ranhura é substituída UMA vez e há gate a exigir que nenhuma `{…}` sobre
//! (`nenhuma_ranhura_fica_por_preencher`): uma marca esquecida não compila — mas uma marca que por
//! acaso fosse WGSL válido passaria em silêncio.

/// O corpo do desenhista.
const FORWARD: &str = include_str!("forward.wgsl");

/// O passe que codifica o pixel para o ecrã.
pub(crate) const ECRA: &str = include_str!("ecra.wgsl");

fn f(x: f32) -> String {
    let s = format!("{x:?}");
    if s.contains('.') || s.contains('e') { s } else { format!("{s}.0") }
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
    FORWARD
        .replace("{MATERIAL}", &material)
        .replace("{AMBIENTE}", ambiente.wgsl)
        .replace("{OLHAR}", ph2d_view_transform::wgsl::SOURCE)
        .replace("{MAX_LUZES2}", &(2 * crate::MAX_LUZES).to_string())
        .replace("{MAX_LUZES}", &crate::MAX_LUZES.to_string())
        .replace("{TAB_W}", &crate::TAB_W.to_string())
        .replace("{PISO_LUZ}", &f(ambiente.piso_luz))
}
