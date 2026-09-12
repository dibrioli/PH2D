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
