//! **Quem estica a forma** — o aviso da forma redonda esticada (plano F3, onda 3). Na opção B a forma é filha do corpo
//! e a escala que a deforma é quase sempre a do CORPO: o painel só vê a da seleção, por isso a resposta vem daqui.

use ph2d_ecs::{ChildOf, Entity, Transform, World};
use ph2d_editor_core::InspectorTransformInfo;

/// Abaixo disto, `|x|` e `|y|` são a mesma escala.
const ESCALA_IGUAL: f32 = 1e-4;

fn desigual(s: [f32; 2]) -> bool {
    (s[0].abs() - s[1].abs()).abs() > ESCALA_IGUAL
}

/// A pose do mais próximo, `forma` ou um antepassado, com escala desigual — se a escala da forma no MUNDO (o produto
/// pela cadeia) é desigual; `None` quando não é, mesmo com dois esticões que se anulam.
#[must_use]
pub fn quem_estica(world: &World, forma: Entity) -> Option<InspectorTransformInfo> {
    let cadeia: Vec<(Entity, &Transform)> = std::iter::successors(Some(forma), |&e| {
        world.get::<ChildOf>(e).map(ChildOf::parent)
    })
    .map_while(|e| Some((e, world.get::<Transform>(e)?)))
    .collect();
    let mundo = cadeia.iter().fold([1.0_f32, 1.0], |a, (_, t)| {
        [a[0] * t.scale.x.abs(), a[1] * t.scale.y.abs()]
    });
    if !desigual(mundo) {
        return None;
    }
    let (e, t) = cadeia
        .into_iter()
        .find(|(_, t)| desigual([t.scale.x, t.scale.y]))?;
    Some(InspectorTransformInfo {
        entity_bits: e.to_bits(),
        translation: [t.translation.x, t.translation.y],
        rotation_rad: t.rotation,
        scale: [t.scale.x, t.scale.y],
        skew_rad: [t.skew_x, t.skew_y],
    })
}
