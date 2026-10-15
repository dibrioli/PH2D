//! **O arnês que ATRAVESSA a fronteira** — e nada mais (HOWTO §2.5).
//!
//! # Por que ele existe
//!
//! Duas costuras de teste desta família **ficaram na shell**, porque o sujeito delas é meio chrome
//! de famílias que ainda não saíram:
//!
//! | o gate | o que ele atravessa | quando ele volta para cá |
//! |---|---|---|
//! | *o `+` deixa o componente na cena **e a secção aparece*** | `render_loop/inspector_presence_probe::slice`, que chama um builder **privado** do `render_loop` | quando a família da **Sprite** sair (o 9-Slice é dela) |
//! | *pintar uma cópia LIGADA muda as irmãs* | `hero_intents/texture_rebind::rebind_to_individual`, o re-alojamento de pixels de uma sprite | idem |
//!
//! ⛔ **Elas não podiam vir com a família, e a razão é estrutural, não de arrumação:** uma crate
//! **nunca pode chamar o `bin`**. A seta aponta sempre shell → família (HOWTO §4), logo um gate
//! cujo sujeito inclui chrome da shell só pode viver na shell.
//!
//! ⚠️ E do outro lado, um `#[cfg(test)]` desta crate é **falso** quando é ela a ser compilada como
//! dependência — é isso que torna este módulo necessário em vez de os gates lá simplesmente
//! chamarem `crate::…`.
//!
//! # ⛔ Ele tem o TAMANHO do que atravessa
//!
//! O piloto abriu **16** itens de uma vez e o compilador acusou: `1 de 16` era lido de fora. Aqui
//! são **seis**, contados nos dois módulos de costura que a shell guarda
//! (`component_attach_seam_tests` e `instance_paint_seam_tests`). *Um arnês que exporta o que
//! ninguém do outro lado chama é uma superfície pública mantida por engano.*

use ph2d_ecs::{Children, Entity, MasterRoot, Name, SimWorld, Transform, scene::ComponentRegistry};
use ph2d_physics_ecs::PhysicsBridge;

/// O catálogo de componentes que as fixturas montam — **o mesmo que o produto monta**, com gate.
///
/// Ver [`crate::component_registry_for_tests`] para a medição que o justifica.
pub fn registo() -> ComponentRegistry {
    crate::component_registry_for_tests::registo()
}

/// Uma sprite seleccionável, para os gates do gesto do `+`.
pub fn image(sim: &mut SimWorld) -> u64 {
    sim.world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Image"),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        ))
        .id()
        .to_bits()
}

/// Um passe de sincronização **sem documentos vectoriais** — ver `crate::instance_sync_docs` para
/// os que têm.
pub fn pass(
    sim: &mut SimWorld,
    r: &ComponentRegistry,
    bridge: &PhysicsBridge,
    echo: &mut crate::instance_sync::MasterEcho,
) -> usize {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    crate::instance_sync::sync_instances(
        sim,
        r,
        bridge,
        echo,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    )
}

/// Instanciar **sem documentos vectoriais** — idem.
pub fn instantiate(
    sim: &mut SimWorld,
    r: &ComponentRegistry,
    master: Entity,
    parent: Option<Entity>,
) -> Result<Entity, crate::instantiate::Refusal> {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    crate::instantiate::instantiate_master(
        sim,
        r,
        master,
        parent,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
    )
}

/// O descendente de `root` com um nome dado.
pub fn piece(sim: &SimWorld, root: Entity, name: &str) -> Entity {
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if e != root && sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) {
            return e;
        }
        if let Some(kids) = sim.world().get::<Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    panic!("nao ha' peca chamada {name:?}");
}

/// Mestre simples — raiz sem geometria + uma peça com sprite, e **sem física**.
///
/// ⚠️ O ragdoll da outra fixtura tem o braço **dinâmico**, e a pose de um corpo dinâmico nem
/// sincroniza nem vira excepção. Um gate sobre a POSE medido lá estaria a medir o `pose_owner`.
pub fn plain_master(sim: &mut SimWorld) -> Entity {
    use ph2d_ecs::ChildOf;
    let root = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Rig"), MasterRoot))
        .id();
    sim.world_mut().spawn((
        Transform::from_translation(ph2d_core::Vec2::new(1.0, 0.0)),
        Name::new("Arm"),
        ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        ChildOf(root),
    ));
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    root
}

/// ⭐ (CodeEditor T7) **As peças dos gates do load de scripts** que a shell guarda
/// (`project_scripts_tests`: o sujeito é o `read_project_file` dela). A lei — abrir, importar,
/// instalar — é desta crate (`crate::script_project`); a shell prova só o ficheiro.
pub mod scripts {
    use ph2d_ecs::scene::{ComponentRegistry, WorldSnapshot};
    use ph2d_script::{LuauScript, ScriptHost, ScriptLibrary};

    /// Um script que põe o `x` do objecto em `7` no primeiro tique.
    pub const ANDA: &str = "function update(self, dt) ph2d.set(self.id, \"x\", 7) end\n";

    fn registo() -> ComponentRegistry {
        let mut r = ComponentRegistry::new();
        ph2d_ecs::scene::register_ecs_components(&mut r);
        ph2d_script::register_script_components(&mut r);
        r
    }

    /// Um snapshot com UM objecto (`StableId(1)`) que corre `cfg`.
    pub fn snapshot(cfg: LuauScript) -> WorldSnapshot {
        let mut sim = ph2d_ecs::SimWorld::new();
        sim.world_mut()
            .spawn((ph2d_ecs::Transform::IDENTITY, ph2d_ecs::StableId(1), cfg));
        let mut prop = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
        let mut work = ph2d_ecs::WorklistBuf::default();
        let mut snap = WorldSnapshot::new();
        ph2d_ecs::scene::world_to_snapshot(
            sim.world_mut(),
            &mut prop,
            &mut work,
            &registo(),
            &mut snap,
        )
        .expect("captura");
        snap
    }

    /// Troca o blob do `LuauScript` do 1.º objecto pelo v2 (o CAMINHO de um ficheiro) — a forma que
    /// um ficheiro v189 tem, escrita campo a campo e não a partir do tipo vivo.
    pub fn as_v2(world: &mut WorldSnapshot, path: &str) {
        let tid = ph2d_ecs::scene::stable_type_id(ph2d_script::project_scripts::CANONICAL_NAME);
        let row = std::sync::Arc::make_mut(&mut world.entities[0]);
        let blob = row
            .components
            .iter_mut()
            .find(|b| b.type_id == tid)
            .expect("blob");
        blob.data = postcard::to_allocvec(&ph2d_script::component::LuauScriptV2 {
            source: path.to_owned(),
            own: Default::default(),
        })
        .expect("v2");
    }

    /// O caminho do Ctrl+O depois de o ficheiro ser aceite: abrir os scripts (importar os caminhos
    /// antigos), restaurar o mundo, instalar a biblioteca e correr um tique. Devolve a biblioteca, o
    /// `LuauScript` do objecto e o `x` dele.
    pub fn open_and_tick(
        mut world: WorldSnapshot,
        scripts: &[u8],
        project_path: &str,
    ) -> (ScriptLibrary, LuauScript, f32) {
        let lib = crate::script_project::open(&mut world, scripts, project_path)
            .expect("scripts legíveis")
            .lib;
        let mut sim = ph2d_ecs::SimWorld::new();
        let e = ph2d_ecs::scene::snapshot_to_world(sim.world_mut(), &world, &registo())
            .expect("restaura")[0];
        crate::script_project::install(sim.world_mut(), lib.clone());
        let mut host = ScriptHost::new().expect("vm");
        host.scene_sync(sim.world_mut());
        let r = host.scene_tick(sim.world_mut(), 0.1);
        assert!(r.failed.is_empty(), "{:?}", r.failed);
        let w = sim.world();
        let cfg = w.get::<LuauScript>(e).expect("cfg").clone();
        let x = w.get::<ph2d_ecs::Transform>(e).expect("pose").translation.x;
        (lib, cfg, x)
    }
}
