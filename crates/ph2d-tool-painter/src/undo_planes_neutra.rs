//! O teste «a linha é a neutra» de cada plano do relevo — filho de `undo_planes`, cortado pelo tecto de 700
//! linhas.

use ph2d_painter_brush::material::MaterialBytes;

pub(super) fn linha_de_altura_neutra(l: &[f32]) -> bool {
    l.iter().fold(0u32, |a, v| a | v.to_bits()) == 0
}

pub(super) fn altura_neutra(v: &f32) -> bool {
    v.to_bits() == 0
}

pub(super) fn linha_de_cobertura_neutra(l: &[u8]) -> bool {
    l.iter().fold(0u8, |a, &b| a | b) == 0
}

pub(super) const fn cobertura_neutra(v: &u8) -> bool {
    *v == 0
}

/// A linha de material é a neutra: comparada em BYTES contra a linha-modelo (`memcmp`), não texel a texel —
/// o OR por texel de `[u8; 7]` não vectoriza e era 16 % do Ctrl+Z do 1.º traço Impasto a 4K (B08c.M2).
pub(super) fn linha_de_material_neutra(l: &[MaterialBytes], modelo: &[MaterialBytes]) -> bool {
    l.as_flattened() == modelo[..l.len()].as_flattened()
}

#[cfg(test)]
mod linha_neutra_tests {
    use super::{MaterialBytes, linha_de_material_neutra};

    /// O `memcmp` contra a linha-modelo responde o mesmo que o OR texel a texel (o oráculo de antes),
    /// também quando só o ÚLTIMO byte do ÚLTIMO texel foge do neutro.
    #[test]
    fn a_linha_de_material_neutra_por_bytes_e_a_de_texel_a_texel() {
        let neutro = ph2d_painter_brush::material::Material::NEUTRAL.to_bytes();
        let velho = |l: &[MaterialBytes]| {
            l.iter().fold(0u8, |a, m| {
                a | (0..7).fold(0u8, |b, k| b | (m[k] ^ neutro[k]))
            }) == 0
        };
        let w = 37;
        let modelo = vec![neutro; w];
        let mut casos = vec![modelo.clone()];
        for i in [0, 18, w - 1] {
            for k in 0..7 {
                let mut l = modelo.clone();
                l[i][k] ^= 1;
                casos.push(l);
            }
        }
        for l in &casos {
            assert_eq!(linha_de_material_neutra(l, &modelo), velho(l));
        }
        assert!(linha_de_material_neutra(&casos[0], &modelo));
        assert_eq!(
            casos
                .iter()
                .filter(|l| !linha_de_material_neutra(l, &modelo))
                .count(),
            21
        );
    }
}
