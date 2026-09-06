//! O afim 2D que leva a geometria **local** de um path para o **mundo**.
//!
//! Desde o ADR-0110 cada path é uma entidade, e desde o ADR-0111 essa entidade tem
//! um `Transform` como qualquer sprite. A geometria guardada em [`crate::VecPath`]
//! passa a ser **local**; quem a coloca no mundo é este afim, publicado pela shell
//! uma vez por frame (`parent_world_transform ∘ Transform`).
//!
//! Identidade ⇒ local **é** mundo, que é o estado de todo path recém-criado e o de
//! todos os testes puros. Nada regride quando ninguém transforma nada.
//!
//! O documento não conhece ECS: a shell traduz `Transform` (e a cadeia de pais) num
//! [`Xform`] e entrega o mapa pronto. Aqui só há álgebra.
//!
//! ⚠️ **A ÁLGEBRA mudou de dono e a API não mudou** — o [`Xform`] mora na
//! [`ph2d_affine`], uma folha sem dependência nenhuma, porque seis crates que nada
//! têm de vectorial já o consumiam e porque a lei do esqueleto (que serve raster, 3D
//! e Flip) não pode puxar a cena vectorial para multiplicar duas matrizes. O que
//! FICA aqui é o que de facto conhece um path: o mapa por `VecPathId`.

use crate::VecPathId;
use std::collections::BTreeMap;

pub use ph2d_affine::Xform;

/// O afim de cada path, publicado pela shell. Ausente ⇒ identidade.
pub type VecXforms = BTreeMap<VecPathId, Xform>;

/// O afim de `id`, ou identidade. É o acesso canônico: um path recém-criado ainda
/// não está no mapa, e isso tem de significar "local = mundo", não um panic.
#[must_use]
pub fn xform_of(xforms: &VecXforms, id: VecPathId) -> Xform {
    xforms.get(&id).copied().unwrap_or(Xform::IDENTITY)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠️ A álgebra do [`Xform`] é gateada na `ph2d-affine`, que é a dona dela. O que este
    /// ficheiro ainda deve provar é só a metade que conhece um path: **ausente do mapa é
    /// identidade**, e não um panic nem um zero.
    #[test]
    fn a_path_that_is_not_in_the_map_is_at_the_identity_not_missing() {
        assert_eq!(xform_of(&VecXforms::new(), 1), Xform::IDENTITY);
        let mut m = VecXforms::new();
        m.insert(7, Xform([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]));
        assert_eq!(xform_of(&m, 7).apply([1.0, 1.0]), [2.0, 2.0]);
        assert_eq!(xform_of(&m, 8), Xform::IDENTITY);
    }
}
