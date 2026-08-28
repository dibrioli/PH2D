//! ⭐⭐⭐ **A lane do MESTRE + INSTÂNCIA** (ADR-0164 / F4.7) — irmão de `main.rs` pelo cap de 700 LOC,
//! cortado por assunto: uma zona é um corpo com um flag, um rig é uma montagem, e isto é uma
//! montagem **copiada**.
//!
//! # O que esta lane prova, e o que nenhum gate de unidade alcança
//!
//! Duas coisas, e as duas só existem **entre** máquinas:
//!
//! 1. ⭐⭐ **A cópia profunda dá as MESMAS identidades nos três SO.** A `line/components` desenhou
//!    para isto — o `instantiate.rs` di-lo pelo nome (*«dar os mesmos ids em qualquer máquina,
//!    senão o `physics_ecs_c9` diverge entre os 3 OS»*) — e até aqui era uma **afirmação**. O hash
//!    da ponte percorre `self.bodies`, que é um `BTreeMap` chaveado por `Entity`: a ordem dele é a
//!    ordem de ALOCAÇÃO, então uma cópia profunda que visitasse a sub-árvore por outra ordem numa
//!    máquina daria outro hash. *É a única lane em que o defeito é invisível numa máquina só.*
//! 2. ⭐ **O mestre é INERTE** (F4.1). As seis consultas cacheadas da ponte filtram
//!    `Without<MasterPiece>`; se uma delas o deixasse entrar, a receita cairia e o hash mudava.
//!
//! # ⚠️ Ela constrói a instância com as PEÇAS do produto, e não com o verbo
//!
//! O verbo (`instantiate_master`) vive na shell, que esta crate não vê — e nem devia: ele faz mais
//! três coisas, e **nenhuma delas toca no solver**:
//!
//! | o que o verbo faz a mais | porque não pode mudar o hash |
//! |---|---|
//! | nome único (`name_unique`) | a ponte não lê `Name` para nada que entre no hash |
//! | clonar documentos possuídos (`instance_docs`) | são `VecPath`/tinta — não há colisor nenhum lá |
//! | `RootOrder`/`SiblingOrder` | ordem de desenho, não de simulação |
//!
//! O que ele faz que **conta** está aqui, pelas mesmas portas: [`ph2d_ecs::deep_copy_subtree`], os
//! remapeadores declarados, e o `InstanceOf` em toda peça. ⚠️ Os remapeadores são os mesmos três da
//! tabela da shell (`instance_refs::REMAPPERS`), que tem censo de dois lados — um campo de
//! referência novo reprova lá antes de chegar aqui.

use ph2d_core::Vec2;
use ph2d_ecs::{
    Entity, InstanceOf, MasterRoot, Name, SimWorld, StableId, Transform, scene::ComponentRegistry,
};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsJoint, RigidBody, remap_joint_refs,
};

/// Onde a receita fica e onde as duas cópias aterram. Longe de tudo o resto: uma lane que
/// colidisse com outra mediria a colisão.
const MASTER_AT: Vec2 = Vec2::new(-88.0, 9.0);
const INSTANCE_X: [f32; 2] = [-84.0, -80.0];
const INSTANCE_Y: f32 = 9.0;
/// A distância do eixo ao braço — o que dá ao pêndulo o que balançar.
const ARM: f32 = 0.9;

/// Monta a lane. Devolve nada: o oráculo é o hash.
pub fn spawn(sim: &mut SimWorld) {
    let mut registry = ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut registry);
    ph2d_physics_ecs::register_physics_components(&mut registry);

    let master = spawn_master(sim);
    // ⚠️ **A marca ANTES de instanciar e antes de correr**, como o produto faz no passe de quadro
    // (`render_loop::physics_bridge::dispatch`). O binário monta o mundo uma vez, então ela é
    // carimbada uma vez — mas se ficasse de fora, a receita entrava no solver e a lane media o
    // contrário do que promete.
    ph2d_ecs::assign_master_pieces(sim.world_mut());

    for (i, x) in INSTANCE_X.into_iter().enumerate() {
        let Ok(copy) = ph2d_ecs::deep_copy_subtree(sim.world_mut(), &registry, master, None) else {
            continue;
        };
        let pieces: Vec<Entity> = copy.copies();
        // ⚠️ **Remapear ANTES de ligar** — a ordem é load-bearing e o erro é mudo: o mapa contém
        // `mestre → cópia do mestre`, então um `InstanceOf` inserido primeiro seria reescrito para
        // a identidade da PRÓPRIA cópia. É a mesma lei que o `instantiate_master` da shell honra,
        // e o gate dela é `the_instance_points_at_the_master_not_at_itself`.
        ph2d_ecs::remap_instance_of(sim.world_mut(), &pieces, &copy.stable_ids);
        remap_joint_refs(sim.world_mut(), &pieces, &copy.stable_ids);

        for (&src, &dst) in &copy.entities {
            let Some(id) = sim.world().get::<StableId>(src).map(|s| s.0) else {
                continue;
            };
            sim.world_mut()
                .entity_mut(dst)
                .insert(InstanceOf { master: id });
        }
        // A cópia NÃO é um mestre: com o marcador ela nascia inerte, e a lane não simularia nada.
        sim.world_mut().entity_mut(copy.root).remove::<MasterRoot>();
        sim.world_mut()
            .entity_mut(copy.root)
            .insert(Transform::from_translation(Vec2::new(x, INSTANCE_Y)));
        // Nome próprio por cópia: o `stable_name_id` é a chave de outras lanes, e dois homónimos
        // aqui envenenariam a resolução por nome delas.
        sim.world_mut()
            .entity_mut(copy.root)
            .insert(Name::new(format!("C9 Instance {}", i + 1)));
    }
    // ⚠️ E a marca outra vez: as peças das cópias NÃO são de mestre nenhum (a raiz perdeu o
    // `MasterRoot`), e sem esta segunda passagem elas ficariam marcadas do momento da cópia —
    // inertes, e a lane mediria dois pêndulos parados.
    ph2d_ecs::assign_master_pieces(sim.world_mut());
}

/// A RECEITA: eixo estático + braço dinâmico + o pino que os prende, sob uma raiz `MasterRoot`.
fn spawn_master(sim: &mut SimWorld) -> Entity {
    let root = sim
        .world_mut()
        .spawn((
            Transform::from_translation(MASTER_AT),
            Name::new("C9 Master Rig"),
            MasterRoot,
        ))
        .id();
    let hub = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("C9 Master Hub"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.15,
                    half_y: 0.15,
                },
                ..Collider::default()
            },
            ph2d_ecs::ChildOf(root),
        ))
        .id();
    let arm = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(ARM, 0.0)),
            Name::new("C9 Master Arm"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.6,
                    half_y: 0.12,
                },
                ..Collider::default()
            },
            ph2d_ecs::ChildOf(root),
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    // ⚠️ As pontas do pino são escritas em `StableId` DIRECTAMENTE, e não pelo hash do nome: esta
    // sub-árvore vai ser copiada duas vezes, e três braços homónimos tornariam a tradução por nome
    // ambígua. *A identidade é a chave; o nome é só o que se lê.*
    let (a, b) = {
        let w = sim.world();
        (
            w.get::<StableId>(hub).expect("id do eixo").0,
            w.get::<StableId>(arm).expect("id do braco").0,
        )
    };
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("C9 Master Pin"),
        PhysicsJoint {
            body_a: a,
            body_b: b,
            ..PhysicsJoint::default()
        },
        ph2d_ecs::ChildOf(root),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    root
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_physics_ecs::PhysicsBridge;

    /// A entidade da sub-árvore de `root` chamada `name`.
    fn piece(sim: &SimWorld, root: Entity, name: &str) -> Entity {
        let mut stack = vec![root];
        while let Some(e) = stack.pop() {
            if sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) && e != root {
                return e;
            }
            if let Some(kids) = sim.world().get::<ph2d_ecs::Children>(e) {
                stack.extend(kids.iter().copied());
            }
        }
        panic!("nao ha' peca chamada {name:?}");
    }

    fn by_name(sim: &mut SimWorld, name: &str) -> Entity {
        let mut q = sim.world_mut().query::<(Entity, &Name)>();
        q.iter(sim.world())
            .find(|(_, n)| n.0 == name)
            .map(|(e, _)| e)
            .unwrap_or_else(|| panic!("nao ha' entidade chamada {name:?}"))
    }

    fn world_at(sim: &SimWorld, e: Entity) -> Vec2 {
        ph2d_ecs::world_transform(sim.world(), e)
            .expect("a peca existe")
            .translation
    }

    /// ⭐⭐⭐ **A lane mede o que promete: a receita fica FORA do solver e as cópias simulam.**
    ///
    /// ⛔ Sem isto, o `body_count` que o binário imprime é a única testemunha — e ele é
    /// **impresso, nunca afirmado**. Uma receita que entrasse no solver mudaria o hash nos três
    /// SO **de igual maneira**, então a comparação cruzada do CI continuaria verde sobre o
    /// contrário do que esta lane existe para provar. *A comparação entre máquinas não vê um
    /// defeito que as três máquinas cometem.*
    #[test]
    fn the_recipe_stays_out_of_the_solver_and_the_copies_swing() {
        let mut sim = SimWorld::new();
        spawn(&mut sim);
        let master = by_name(&mut sim, "C9 Master Rig");
        let master_arm = piece(&sim, master, "C9 Master Arm");
        let before = world_at(&sim, master_arm);

        let mut bridge = PhysicsBridge::new();
        for tick in 1..=120 {
            bridge.dispatch(&mut sim, true, tick);
        }

        for name in ["C9 Master Hub", "C9 Master Arm"] {
            let e = piece(&sim, master, name);
            assert!(
                bridge.body_pose(e).is_none(),
                "{name} da RECEITA entrou no solver — a lane mede o contrario do que promete"
            );
        }
        assert_eq!(
            world_at(&sim, master_arm),
            before,
            "o braco da receita mexeu-se — ela nao esta' inerte"
        );

        // As duas cópias: cada braço preso ao eixo DELA, e em sítios diferentes.
        let mut xs = Vec::new();
        for i in 1..=INSTANCE_X.len() {
            let root = by_name(&mut sim, &format!("C9 Instance {i}"));
            let hub = piece(&sim, root, "C9 Master Hub");
            let arm = piece(&sim, root, "C9 Master Arm");
            assert!(
                bridge.body_pose(arm).is_some(),
                "o braco da copia {i} nao entrou no solver — a copia esta' inerte"
            );
            let d = (world_at(&sim, arm) - world_at(&sim, hub)).length();
            assert!(
                (d - ARM).abs() < 0.05,
                "a copia {i} tem o braco a {d:.3} do eixo dela (o pino manda {ARM:.3}) — \
                 a junta prendeu nos corpos do MESTRE, que nao simulam"
            );
            assert!(
                world_at(&sim, arm).y < world_at(&sim, hub).y,
                "a copia {i} nao balancou — ela nao esta' a simular"
            );
            xs.push(world_at(&sim, arm).x);
        }
        // ⚠️ **O controle da separação:** se as duas convergissem, o que se veria era um rig só e
        // as réguas acima passavam na mesma.
        assert!(
            (xs[0] - xs[1]).abs() > 1.0,
            "os dois bracos convergiram ({xs:?}) — as copias partilham corpos"
        );
    }
}
