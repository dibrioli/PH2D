//! ⭐ **Repor no presente os donos que um passe TARDIO moveu** (`docs/BlenderConstraints/07_W6_pontas.md`
//! §3): o extract corre antes do cozimento do Motion, e o passe da pilha depois dele (Copy Particle)
//! move donos já extraídos. Só esses e os descendentes: o `GlobalTransform` do espelho e a pose de
//! cada instância (os quads de um 9-slice e os extras da folha partilham-na). Nada movido ⇒ nada lido.

use std::collections::BTreeSet;

use ph2d_ecs::{Children, Entity, GlobalTransform, SimRef, World, world_transform};

use crate::RenderInstance;

/// Repõe `donos` (e os descendentes) no `present`. Devolve `(entidade, antes, depois)` por espelho
/// reposto (o primeiro de cada entidade), para quem desenha por cima dele (o gizmo).
pub fn repoe_no_presente(
    sim: &World,
    present: &mut World,
    donos: &[Entity],
) -> Vec<(Entity, GlobalTransform, GlobalTransform)> {
    if donos.is_empty() {
        return Vec::new();
    }
    let mut todos = BTreeSet::new();
    let mut pilha = donos.to_vec();
    while let Some(e) = pilha.pop() {
        if todos.insert(e)
            && let Some(c) = sim.get::<Children>(e)
        {
            pilha.extend(c.iter());
        }
    }
    let mut feitos = Vec::new();
    let mut q = present.query::<(&SimRef, &mut GlobalTransform, Option<&mut RenderInstance>)>();
    for (s, mut gt, ri) in q.iter_mut(present) {
        if !todos.contains(&s.0) {
            continue;
        }
        let Some(t) = world_transform(sim, s.0) else {
            continue;
        };
        let novo = GlobalTransform::from_transform(t);
        if !feitos.iter().any(|(e, _, _)| *e == s.0) {
            feitos.push((s.0, *gt, novo));
        }
        *gt = novo;
        if let Some(mut ri) = ri {
            (ri.world_pos, ri.basis) = RenderInstance::pose_de(&novo);
        }
    }
    feitos
}
