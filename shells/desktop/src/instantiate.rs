//! ⭐ **INSTANCIAR** — a porta do produto (ADR-0164 / plano F4.2).
//!
//! Ela compõe as duas metades que sozinhas não fazem uma instância:
//!
//! 1. [`ph2d_ecs::deep_copy_subtree`] — copia os bytes de toda a subárvore e dá **identidade
//!    nova** a cada peça;
//! 2. [`crate::instance_refs::remap_object_refs`] — reescreve as referências guardadas por
//!    identidade, para que a junta da instância prenda **os corpos dela**.
//!
//! ⛔ **Nunca chame a primeira sozinha do produto.** Uma cópia sem remap é o defeito que esta
//! wave existe para curar, e ele é MUDO: a junta prende no mestre (que não simula), então as
//! peças da instância caem soltas e nada na tela diz porquê. O gate
//! `only_the_instantiate_door_calls_the_deep_copy` mantém esta função como o único chamador.

use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{Entity, InstanceOf, MasterRoot, Name, SimWorld, StableId};

/// **Instancia o mestre `master_root`**, devolvendo a raiz da instância.
///
/// `parent` diz onde ela aterra (`None` = raiz da cena).
///
/// Devolve `None` quando `master_root` não é um mestre — e a recusa é deliberada: pôr um
/// [`InstanceOf`] a apontar para uma subárvore que não é receita daria ao sync (F4.3) um mestre
/// que o artista edita como um objeto qualquer, e cada edição da cena seria propagada como se
/// fosse autoria de biblioteca.
pub(crate) fn instantiate_master(
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    master_root: Entity,
    parent: Option<Entity>,
) -> Option<Entity> {
    sim.world().get::<MasterRoot>(master_root)?;
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let master_id = sim.world().get::<StableId>(master_root)?.0;
    let base = sim
        .world()
        .get::<Name>(master_root)
        .map_or_else(|| "Instance".to_string(), |n| n.0.clone());

    let copy = ph2d_ecs::deep_copy_subtree(sim.world_mut(), registry, master_root, parent).ok()?;
    let pieces = copy.copies();

    // ⚠️⚠️ **A ORDEM destes dois passos é load-bearing, e o erro é silencioso.**
    //
    // O mapa contém `mestre → cópia do mestre` (tem de conter: uma junta ancorada na raiz da
    // receita precisa dele). Se o `InstanceOf` fosse inserido ANTES, o remapeador dele — que é
    // uma linha da mesma tabela — reescreveria o elo para a identidade da **própria cópia**, e a
    // instância passaria a dizer-se instância de si mesma. O sync da F4.3 leria isso como *"o
    // mestre sou eu"* e nunca mais propagaria nada.
    //
    // ⇒ remapear primeiro, ligar depois. Gate: `the_instance_points_at_the_master_not_at_itself`.
    crate::instance_refs::remap_object_refs(sim.world_mut(), &pieces, &copy.stable_ids);

    let unique = crate::name_unique::unique_name(sim, &base);
    let mut root = sim.world_mut().entity_mut(copy.root);
    // ⚠️ A instância NÃO é um mestre: com o marcador ela nasceria **inerte** (F4.1) — três
    // ragdolls no lugar certo, nenhum a cair.
    root.remove::<MasterRoot>();
    root.insert(InstanceOf { master: master_id });
    root.insert(Name::new(unique));

    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    ph2d_ecs::assign_missing_sibling_order(sim.world_mut());
    // As peças da cópia deixam de ser peças de mestre no mesmo quadro em que nascem — sem isto
    // elas só voltariam a simular no próximo passe da ponte.
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    Some(copy.root)
}

/// ⭐ **DUPLICAR** — a mesma cópia profunda, **sem** elo ao original.
///
/// A cópia aterra ao lado da fonte (mesmo pai) e é um objeto independente. As referências internas
/// são remapeadas pela mesma tabela: *a junta de uma cópia prende os corpos DELA*.
///
/// ⚠️ **Isto substitui uma cópia RASA** que levava quatro componentes (`Transform`, `Sprite`,
/// `Name`, `ChildOf`) e **nenhum filho** — duplicar um ragdoll dava uma linha vazia na Hierarquia.
/// O ADR-0164 nomeia esse defeito; ele existia porque copiar bytes de tipos que a shell não conhece
/// não tinha porta, e agora tem.
///
/// ⚠️ **Uma cópia de um MESTRE é outro mestre** (o `MasterRoot` viaja no blob), e uma cópia de uma
/// INSTÂNCIA é outra instância do mesmo mestre (o elo aponta para fora do que se copiou, e por isso
/// o remap não lhe toca). As duas são o que o artista espera de *Duplicar*.
pub(crate) fn duplicate_subtree(
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    src: Entity,
) -> Option<Entity> {
    let parent = sim.world().get::<ph2d_ecs::ChildOf>(src).map(|c| c.0);
    let base = sim
        .world()
        .get::<Name>(src)
        .map_or_else(|| "Entity".to_string(), |n| n.0.clone());

    let copy = ph2d_ecs::deep_copy_subtree(sim.world_mut(), registry, src, parent).ok()?;
    crate::instance_refs::remap_object_refs(sim.world_mut(), &copy.copies(), &copy.stable_ids);

    let unique = crate::name_unique::unique_name(sim, &base);
    sim.world_mut()
        .entity_mut(copy.root)
        .insert(Name::new(unique));
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    ph2d_ecs::assign_missing_sibling_order(sim.world_mut());
    // A cópia de um mestre é um mestre: as peças dela têm de ser marcadas já.
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    Some(copy.root)
}

#[cfg(test)]
#[path = "instantiate_tests.rs"]
mod tests;
