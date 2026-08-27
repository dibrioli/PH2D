//! `PH2D_INSTANCE_SMOKE=<n>` — as cenas PRONTAS-DE-VER do núcleo de instância
//! (ADR-0164 / plano F4).
//!
//! ⚠️ **Nada aqui pré-monta física.** As peças são entidades ECS com `RigidBody`/`Collider`/
//! `PhysicsJoint`, e quem as põe no solver é a ponte — se ela estivesse morta, os pêndulos
//! ficavam pendurados no ar em vez de balançar, que é a falha honesta.
//!
//! # Cena 1 — o ragdoll instanciado 3× (o smoke-gate 1 da F4)
//!
//! Um MESTRE lá em cima (que **não** se mexe — é receita, não objeto) e três instâncias dele em
//! baixo. Cada instância tem o pino DELA a prender os corpos DELA: os três balançam.
//!
//! ⛔ **O que o defeito parecia**, antes do remap da F4.2: as três juntas continuavam a nomear os
//! corpos do MESTRE — que não simulam —, então os braços caíam soltos no chão.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, MasterRoot, Name, SimWorld, StableId, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// Onde as três instâncias aterram, e onde a receita fica.
const INSTANCE_X: [f32; 3] = [-2.4, 0.0, 2.4];
const INSTANCE_Y: f32 = 1.2;
const MASTER_AT: Vec2 = Vec2::new(0.0, 3.4);
/// A distância do eixo ao braço — o que faz o pêndulo ter o que balançar.
const ARM: f32 = 0.9;

/// ⭐ **Monta o MESTRE**: eixo estático + braço dinâmico + o pino que os prende, tudo pendurado
/// numa raiz marcada [`MasterRoot`].
///
/// ⚠️ As referências do pino são escritas em `StableId` **diretamente**, e não pelo hash do nome:
/// esta cena vai ser copiada três vezes, e três braços chamados "Arm" tornariam a tradução por
/// nome ambígua. *A identidade é a chave; o nome é só o que o artista lê.*
pub(crate) fn spawn_master(sim: &mut SimWorld) -> ph2d_ecs::Entity {
    let root = sim
        .world_mut()
        .spawn((
            Transform::from_translation(MASTER_AT),
            Name::new("Ragdoll"),
            MasterRoot,
        ))
        .id();
    let hub = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Hub"),
            Sprite::atlas(WHITE_TILE_KEY, [0.3, 0.3], [0.55, 0.57, 0.64, 1.0]),
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
            ChildOf(root),
        ))
        .id();
    let arm = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(ARM, 0.0)),
            Name::new("Arm"),
            Sprite::atlas(WHITE_TILE_KEY, [1.2, 0.24], [0.90, 0.55, 0.25, 1.0]),
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
            ChildOf(root),
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let (a, b) = {
        let w = sim.world();
        (
            w.get::<StableId>(hub).expect("id do eixo").0,
            w.get::<StableId>(arm).expect("id do braco").0,
        )
    };
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Pin"),
        PhysicsJoint {
            body_a: a,
            body_b: b,
            ..PhysicsJoint::default()
        },
        ChildOf(root),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    ph2d_ecs::assign_missing_sibling_order(sim.world_mut());
    root
}

/// **Monta o mestre e instancia-o 3×** — o corpo da cena 1, partilhado com os gates.
pub(crate) fn spawn_ragdoll_scene(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
) -> (ph2d_ecs::Entity, Vec<ph2d_ecs::Entity>) {
    let master = spawn_master(sim);
    let mut roots = Vec::new();
    for x in INSTANCE_X {
        let Ok(inst) = crate::instantiate::instantiate_master(sim, registry, master, None, docs)
        else {
            continue;
        };
        sim.world_mut()
            .entity_mut(inst)
            .insert(Transform::from_translation(Vec2::new(x, INSTANCE_Y)));
        roots.push(inst);
    }
    (master, roots)
}

impl crate::App {
    /// Prólogo do quadro, uma vez. No-op sem a env.
    pub(crate) fn instance_smoke(&mut self) {
        let Some(which) = std::env::var("PH2D_INSTANCE_SMOKE").ok() else {
            return;
        };
        if self.instance_smoke_done {
            return;
        }
        if self.gfx.is_none() {
            return; // o mundo ainda não subiu; tenta no próximo quadro
        }
        self.instance_smoke_done = true;
        match which.trim() {
            "1" => self.instance_smoke_ragdoll(),
            other => println!("[instance smoke] cena {other:?} nao existe (ha' a 1)"),
        }
        // ⚠️ **O relógio TEM de partir a andar**, e a linha vive no prólogo pela razão do smoke da
        // física: uma lista por-cena seria a enumeração de que a próxima cena nasce fora. Sem isto
        // os três pêndulos ficam pendurados no ar e o smoke lê-se como *"a física está morta"* —
        // que é precisamente o defeito que ele existe para distinguir.
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.panel_visibility.insert("timeline", true);
        }
        self.playhead.rewind();
        self.playhead.play();
    }

    /// Cena 1 — ver o cabeçalho do módulo.
    fn instance_smoke_ragdoll(&mut self) {
        let vec_entities = &mut self.vec_entities;
        let gfx = self.gfx.as_mut().expect("gfx");
        crate::physics_smoke::spawn_floor(gfx.sim.world_mut());
        // Campos DISJUNTOS do `AppGfx` (+ o mapa, que é do `App`) — empréstimos separados, sem
        // clonar o registo nem o documento.
        let mut docs = crate::instance_docs::OwnedDocs {
            vec_scene: &mut gfx.vec_scene,
            vec_entities,
        };
        let (_master, roots) =
            spawn_ragdoll_scene(&mut gfx.sim, &gfx.component_registry, &mut docs);
        // ⚠️ A cena **imprime o que montou** — se estas linhas não aparecerem, PARE: o que está
        // na tela não é o que este smoke descreve.
        println!("[instance smoke 1] receita 'Ragdoll' la' em cima (ela NAO se mexe)");
        for (i, r) in roots.iter().enumerate() {
            let name = gfx
                .sim
                .world()
                .get::<Name>(*r)
                .map(|n| n.0.clone())
                .unwrap_or_default();
            println!(
                "[instance smoke 1] instancia {} = {name:?} em x = {}",
                i + 1,
                INSTANCE_X[i]
            );
        }
        println!(
            "[instance smoke 1] os {} bracos tem de BALANCAR cada um no eixo dele; \
             braco no chao = a junta prendeu no mestre",
            roots.len()
        );
        // ⭐ **A segunda metade do smoke é o SYNC** (F4.3) — e ela precisa de um gesto, então a
        // cena diz qual. Sem esta linha o artista vê três pêndulos e não descobre sozinho que
        // editar a receita muda os três.
        println!(
            "[instance smoke 1] agora escolha 'Ragdoll > Arm' (o de CIMA, a receita) e mude a cor \
             em 'Color & Tint': os tres bracos de baixo mudam com ele"
        );
        // ⭐ E a terceira metade é o OVERRIDE (F4.4): a excepção que o artista faz numa cópia tem
        // de sobreviver à edição seguinte da receita. Sem a instrução, ele nunca a descobre.
        println!(
            "[instance smoke 1] e a EXCEPCAO: pinte o 'Arm' de UMA das copias de baixo, depois \
             pinte o da receita outra vez — a que voce tocou fica com a cor dela"
        );
        println!(
            "[instance smoke 1] para desfazer a excepcao: botao direito na linha da copia -> \
             'Revert to Master'"
        );
    }
}
