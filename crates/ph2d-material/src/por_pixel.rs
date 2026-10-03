//! ⭐⭐ **A RUGOSIDADE DO PIXEL** — a irmã do [`Surface::at_base_color`]: um mapa de rugosidade dá
//! um número por pixel, e das grandezas do [`OpenPbr::prepare`] só o `main_alpha` depende dele.
//!
//! O encolhimento do lóbulo (o `shrink` do [`crate::wgsl::EnvLobe`]) é `f64` na CPU; por pixel, a
//! placa lê-o de uma tabela de [`SHRINK_N`] entradas em `√α` ([`por_pixel_wgsl`]). Medido: o
//! desvio da tabela é `5,7e-5`, o mesmo do salto da própria lei no ramo perto de `α = 1`.

use crate::{OpenPbr, Surface, bsdf};

/// Entradas da tabela do encolhimento, em `√α` de `0` a `1`.
pub const SHRINK_N: usize = 65;

/// O `main_alpha` de uma rugosidade com o verniz que alarga a reflexão — as contas do `prepare`
/// (`coat_roughness⁴ = coat_alpha²`).
fn main_alpha_de(r: f32, coat_alpha: f32, coat_weight: f32) -> f32 {
    let r2 = r * r;
    let coat_affected = (2.0 * (coat_alpha * coat_alpha) + r2 * r2)
        .min(1.0)
        .sqrt()
        .sqrt();
    let e = bsdf::mix(r, coat_affected, coat_weight);
    e * e
}

impl Surface {
    /// ⭐ **Este material com a rugosidade `roughness` NESTE pixel** (o `specular_roughness`).
    #[must_use]
    pub fn at_roughness(self, roughness: f32) -> Self {
        let m = OpenPbr {
            specular_roughness: roughness,
            ..self.m
        };
        Self {
            main_alpha: main_alpha_de(roughness, self.coat_alpha, m.coat_weight),
            m,
            ..self
        }
    }
}

/// ⭐ O WGSL por pixel: a tabela do encolhimento e `mx_at_roughness(m, r)` — depois do
/// [`crate::wgsl::SOURCE`] (lê o `Mat` e o `MX_EPS` dele).
#[must_use]
pub fn por_pixel_wgsl() -> String {
    let tab: Vec<String> = (0..SHRINK_N)
        .map(|i| {
            let s = i as f32 / (SHRINK_N - 1) as f32;
            format!("{:?}", crate::lobe_shrink(s * s))
        })
        .collect();
    format!(
        "const MX_SHRINK_N: u32 = {SHRINK_N}u;\n\
         const MX_SHRINK: array<f32, {SHRINK_N}> = array<f32, {SHRINK_N}>({});\n{CORPO}",
        tab.join(", ")
    )
}

const CORPO: &str = r"
fn mx_lobe_shrink(alpha: f32) -> f32 {
    let x = sqrt(clamp(alpha, 0.0, 1.0)) * f32(MX_SHRINK_N - 1u);
    let i = min(u32(x), MX_SHRINK_N - 2u);
    let f = x - f32(i);
    return MX_SHRINK[i] + (MX_SHRINK[i + 1u] - MX_SHRINK[i]) * f;
}

// O gemeo do `Surface::at_roughness`: o `main_alpha` (prepared.y) e o encolhimento dele (emissive.y).
fn mx_at_roughness(m: Mat, r: f32) -> Mat {
    var o = m;
    let r2 = r * r;
    let ca = m.prepared.z;
    let coat_affected = sqrt(sqrt(min(2.0 * (ca * ca) + r2 * r2, 1.0)));
    let e = r + (coat_affected - r) * m.darkening_coatweight.a;
    o.prepared.y = e * e;
    o.emissive.y = mx_lobe_shrink(clamp(o.prepared.y, MX_EPS, 1.0));
    return o;
}
";

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **A porta é o `prepare` com aquela rugosidade** — com e sem verniz.
    #[test]
    fn a_rugosidade_do_pixel_e_o_prepare_com_ela() {
        for coat in [0.0, 0.4, 1.0] {
            for cr in [0.0, 0.3, 0.9] {
                let base = OpenPbr {
                    specular_roughness: 0.3,
                    coat_weight: coat,
                    coat_roughness: cr,
                    ..OpenPbr::default()
                }
                .prepare();
                for r in [0.0, 0.05, 0.37, 0.8, 1.0] {
                    let a = base.at_roughness(r);
                    let b = OpenPbr {
                        specular_roughness: r,
                        ..base.m
                    }
                    .prepare();
                    let d = (a.main_alpha - b.main_alpha).abs();
                    assert!(d <= 2.0e-7, "coat {coat} cr {cr} r {r}: {d}");
                    assert_eq!(a.m, b.m);
                    assert_eq!(a.coat_alpha, b.coat_alpha);
                }
            }
        }
    }

    /// ⭐ **A tabela é o `lobe_shrink`** a `≤ 1e-4` (o piso é o ramo da lei perto de `α = 1`).
    #[test]
    fn a_tabela_do_encolhimento_e_a_lei() {
        let tab: Vec<f32> = (0..SHRINK_N)
            .map(|i| {
                let s = i as f32 / (SHRINK_N - 1) as f32;
                crate::lobe_shrink(s * s)
            })
            .collect();
        let mut pior = 0.0f32;
        for k in 0..=20_000 {
            let alpha = k as f32 / 20_000.0;
            let x = alpha.sqrt() * (SHRINK_N - 1) as f32;
            let i = (x as usize).min(SHRINK_N - 2);
            let f = x - i as f32;
            let t = tab[i] + (tab[i + 1] - tab[i]) * f;
            pior = pior.max((t - crate::lobe_shrink(alpha)).abs());
        }
        assert!(pior <= 1.0e-4, "a tabela desvia {pior}");
    }
}
