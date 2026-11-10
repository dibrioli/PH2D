//! **Quem estica a forma** — o aviso da forma redonda esticada (plano F3, onda 3) e o «Set Scale to 1» (dono 10/11:
//! um clique conserta a cadeia inteira). Na opção B a forma é filha do corpo e a escala que a deforma é quase sempre
//! a do CORPO: o painel só vê a da seleção, por isso a resposta vem daqui.

use ph2d_core::Vec2;
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{ChildOf, Entity, Transform, World};

/// Abaixo disto, `|x|` e `|y|` são a mesma escala.
const ESCALA_IGUAL: f32 = 1e-4;
const TRANSFORM: &str = "ph2d::ecs::Transform";

fn desigual(s: [f32; 2]) -> bool {
    (s[0].abs() - s[1].abs()).abs() > ESCALA_IGUAL
}

/// Cada um da cadeia (`forma` e os antepassados) com a PRÓPRIA escala desigual, do mais próximo ao mais longe — se a
/// escala da forma no MUNDO (o produto pela cadeia) é desigual; vazio quando não é, mesmo com esticões que se anulam.
#[must_use]
pub fn quem_estica(world: &World, forma: Entity) -> Vec<Entity> {
    let cadeia: Vec<(Entity, &Transform)> = std::iter::successors(Some(forma), |&e| {
        world.get::<ChildOf>(e).map(ChildOf::parent)
    })
    .map_while(|e| Some((e, world.get::<Transform>(e)?)))
    .collect();
    let mundo = cadeia.iter().fold([1.0_f32, 1.0], |a, (_, t)| {
        [a[0] * t.scale.x.abs(), a[1] * t.scale.y.abs()]
    });
    if !desigual(mundo) {
        return Vec::new();
    }
    cadeia
        .into_iter()
        .filter(|(_, t)| desigual([t.scale.x, t.scale.y]))
        .map(|(e, _)| e)
        .collect()
}

/// O «Set Scale to 1»: a escala de cada um de [`quem_estica`] volta a 1 e o espelho (o sinal de x e de y) fica; quem
/// tem escala igual, mesmo 2 × 2, fica como está. Tudo na mesma edição: um quadro, um passo de undo.
pub fn escala_a_um(
    world: &World,
    forma: Entity,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let um = |s: f32| if s < 0.0 { -1.0 } else { 1.0 };
    for e in quem_estica(world, forma) {
        if let Some(mut t) = world.get::<Transform>(e).copied() {
            t.scale = Vec2::new(um(t.scale.x), um(t.scale.y));
            ph2d_inspector_ordering::queue_set(queue, registry, e.to_bits(), TRANSFORM, &t);
        }
    }
}
