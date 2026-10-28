//! **Onde assenta o texto que muda** (W5.5): um rótulo cujo texto é DERIVADO (o placar do HUD) cresce a partir do
//! seu ALINHAMENTO — à esquerda, a borda esquerda fica onde o artista a pôs e ele cresce para a direita; à direita,
//! o contrário; ao centro, centra (o comportamento anterior, ao bit). Sem isto um placar preso ao canto esquerdo
//! crescia para os dois lados e saía da tela.

use ph2d_vec_scene::VecPath;

use crate::glyph::{offset_path, path_center};

fn caixa(path: &VecPath) -> Option<([f64; 2], [f64; 2])> {
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for v in path.verts_all() {
        for p in [v.anchor, v.in_handle, v.out_handle] {
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
    }
    lo[0].is_finite().then_some((lo, hi))
}

/// ⭐ Põe `derivado` (cozido na baseline) no sítio do `autorado` (o caminho do documento, já centrado): em `y`
/// centra-o como sempre; em `x` segue o `align` (`0` esquerda · `1` centro · `2` direita, o do `VecTextParams`).
pub fn ao_autorado(derivado: &mut VecPath, autorado: &VecPath, align: u8) {
    let c = path_center(derivado);
    let dx = match (align, caixa(derivado), caixa(autorado)) {
        (0, Some(d), Some(a)) => a.0[0] - d.0[0],
        (2, Some(d), Some(a)) => a.1[0] - d.1[0],
        _ => -c[0],
    };
    offset_path(derivado, [dx, -c[1]]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vec_scene::rectangle;

    /// «Score: 0» autorado e centrado (`[-2, 2]`); o derivado «Score: 12345» cozido na baseline (`[0, 7]`).
    fn caso(align: u8) -> ([f64; 2], [f64; 2]) {
        let autorado = rectangle([-2.0, -0.5], [2.0, 0.5]);
        let mut derivado = rectangle([0.0, -0.2], [7.0, 0.8]);
        ao_autorado(&mut derivado, &autorado, align);
        caixa(&derivado).unwrap()
    }

    #[test]
    fn a_left_aligned_label_grows_to_the_right_of_its_authored_edge() {
        let (lo, hi) = caso(0);
        assert_eq!((lo[0], hi[0]), (-2.0, 5.0));
        assert_eq!(lo[1] + hi[1], 0.0, "em y centra como sempre");
    }

    #[test]
    fn a_right_aligned_label_grows_to_the_left() {
        assert_eq!((caso(2).0[0], caso(2).1[0]), (-5.0, 2.0));
    }

    #[test]
    fn a_centred_label_keeps_the_old_centring() {
        let (lo, hi) = caso(1);
        assert_eq!((lo[0], hi[0]), (-3.5, 3.5));
    }
}
