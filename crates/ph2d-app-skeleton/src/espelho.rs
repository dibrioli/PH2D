//! ⭐ **ESPELHAR UM RAMO** — a porta que o verbo *Mirror Branch* chama: a lei é a
//! [`ph2d_skeleton_live::espelho::espelha`], e aqui sai da cópia o que aquela crate não conhece — o
//! **âncora** (IK ou Look At), que nomeia o alvo do ORIGINAL (duas correntes a puxar o mesmo losango).

use ph2d_ecs::{Entity, SimWorld, scene::ComponentRegistry};

/// Espelha o ramo de `osso` e devolve a raiz da cópia; os ossos copiados não levam âncora.
pub fn espelha(sim: &mut SimWorld, registry: &ComponentRegistry, osso: Entity) -> Option<Entity> {
    let copia = ph2d_skeleton_live::espelho::espelha(sim, registry, osso)?;
    let mut pilha = vec![copia];
    while let Some(e) = pilha.pop() {
        ph2d_constraints_ecs::tirar_ancora(sim.world_mut(), e);
        if let Some(f) = sim.world().get::<ph2d_ecs::Children>(e) {
            pilha.extend(f.iter().copied());
        }
    }
    Some(copia)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⛔⛔ **A IK NÃO VIAJA** — ela nomeia o alvo do original: copiá-la daria duas correntes a puxar o
    /// mesmo losango, e o braço espelhado seguiria a mão do original.
    #[test]
    fn a_ik_fica_para_tras_no_espelho() {
        let (mut sim, [ombro, cotovelo]) = crate::goal::braco();
        sim.world_mut()
            .get_mut::<ph2d_ecs::Transform>(cotovelo)
            .expect("t")
            .rotation = 0.5;
        crate::goal::add(&mut sim, cotovelo).expect("a IK");
        let mut reg = ComponentRegistry::new();
        ph2d_ecs::scene::register_ecs_components(&mut reg);
        ph2d_skeleton_ecs::register_skeleton_components(&mut reg);
        ph2d_constraints_ecs::register_constraint_components(&mut reg);
        let copia = espelha(&mut sim, &reg, ombro).expect("o espelho");
        let mut pilha = vec![copia];
        let mut vistos = 0;
        while let Some(e) = pilha.pop() {
            vistos += 1;
            assert!(
                ph2d_constraints_ecs::ik_de(sim.world(), e).is_none(),
                "a copia trouxe a IK: duas correntes a puxar o mesmo losango"
            );
            if let Some(f) = sim.world().get::<ph2d_ecs::Children>(e) {
                pilha.extend(f.iter().copied());
            }
        }
        assert_eq!(vistos, 2, "a copia tem os dois ossos do braco");
        // ⚠️ O CONTROLO: o original continua com a IK — e a cópia profunda levou-a (sem a porta,
        // este gate ficaria vermelho), porque o registo descreve a pilha.
        assert!(
            ph2d_constraints_ecs::ik_de(sim.world(), cotovelo).is_some(),
            "o espelho tirou a IK do ORIGINAL"
        );
    }
}
