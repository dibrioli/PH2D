//! **The solver's poses flow back into `Transform`** — the read half of the
//! per-frame seam. Its own module for the 700-LOC cap (the gold-standard joint
//! anchor added the pivot sync to `bridge.rs`); see [`PhysicsBridge::readback`].

use super::PhysicsBridge;
use super::space;
use ph2d_ecs::{ChildOf, Entity, SimWorld, World};

/// Quantos saltos de `ChildOf` separam `e` da raiz.
///
/// ⚠️ **Sem guarda de ciclo**, pelo mesmo motivo do `parent_world_transform_into`
/// que o `write_world_pose` chama logo abaixo: uma hierarquia cíclica já trava
/// aquele caminho, e uma segunda resposta a *"a árvore pode ciclar?"* seria pior
/// que nenhuma.
fn depth_of(world: &World, e: Entity) -> u32 {
    let mut d = 0;
    let mut cur = world.get::<ChildOf>(e).map(ChildOf::parent);
    while let Some(p) = cur {
        d += 1;
        cur = world.get::<ChildOf>(p).map(ChildOf::parent);
    }
    d
}

impl PhysicsBridge {
    /// Read each dynamic body's pose back into its entity's `Transform`
    /// (meters, radians CCW, Y-up — no conversion; ADR-0131 D4). Static
    /// bodies never move and kinematic ones are DRIVEN by that same
    /// `Transform` ([`Self::drive_kinematic`]), so both are skipped — asked
    /// through [`BodyKind::solver_owns_pose`], the one door, because a body
    /// that both stages claimed would have its pose written twice a tick.
    ///
    /// [`BodyKind::solver_owns_pose`]: crate::BodyKind::solver_owns_pose
    ///
    /// ⚠️ The solver answers in WORLD space and `Transform` is LOCAL, so the
    /// pose goes back through [`space::write_world_pose`]. Assigning it raw
    /// works for a root (where the two coincide) and is wrong for every child:
    /// the renderer composes the parent onto it again, so the body simulates
    /// in one place and draws in another. A parent that cannot be inverted
    /// (scaled to zero) leaves the `Transform` untouched rather than storing
    /// a non-finite pose that would poison the whole subtree.
    ///
    /// ⚠️ **E a ORDEM é ancestral-antes-de-descendente, o que é correção e não
    /// arrumação.** A conversão mundo→local pergunta ao pai VIGENTE; escrito
    /// antes do pai, um filho é convertido contra a pose que este mesmo passe
    /// está prestes a substituir, e o erro é exatamente o quanto o pai andou
    /// entre as duas escritas. O mapa de corpos é um `BTreeMap<Entity>` e a
    /// ordem de `Entity` **não** é a de spawn — medido, um par pai/filho itera
    /// **o filho primeiro**.
    ///
    /// Durante o play isso era um atraso de UM frame (3,2 mm num par caindo:
    /// pequeno, e some no ruído do movimento — foi assim que atravessou o W5
    /// inteiro). Num REWIND o "frame anterior" é o mundo antes do salto, e o
    /// filho aterrissava a **4,91 m**: a distância que o pai tinha caído. Foi o
    /// *"o reset não consegue devolver o conjunto à posição original"* que o Enio
    /// reportou no smoke da cena 67 — e nem o rig nem os joints tinham parte
    /// nisso, era um par pai/filho comum.
    ///
    /// Ordenar por PROFUNDIDADE resolve a classe inteira: todo ancestral tem
    /// profundidade estritamente menor, então já foi escrito. Ordenar apenas
    /// *"raízes primeiro"* consertaria um nível e deixaria o neto com o mesmo
    /// defeito (gate `the_order_holds_for_a_chain_three_levels_deep`).
    pub(super) fn readback(&mut self, sim: &mut SimWorld) {
        // O scratch sai de `self` para o laço de escrita poder emprestar `chain`.
        let mut order = std::mem::take(&mut self.readback_order);
        order.clear();
        {
            let world = sim.world();
            for (&e, b) in self.bodies.iter() {
                if !b.kind.solver_owns_pose() {
                    continue;
                }
                if let Some(pose) = self.world.body_pose(b.handle) {
                    order.push((
                        depth_of(world, e),
                        e,
                        pose.translation.x,
                        pose.translation.y,
                        pose.rotation.angle(),
                    ));
                }
            }
        }
        // Desempate por entidade: dois irmãos não se afetam, mas uma ordem
        // reproduzível é o que mantém duas corridas iguais byte a byte.
        order.sort_unstable_by_key(|&(d, e, ..)| (d, e));
        {
            let world = sim.world_mut();
            for &(_, e, x, y, rot) in &order {
                space::write_world_pose(world, e, x, y, rot, &mut self.chain);
            }
        }
        self.readback_order = order;
    }
}
