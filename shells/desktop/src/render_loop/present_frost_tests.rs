//! Os gates de **quem sobe para cima do vidro** ([`super::lift`]).
//!
//! ⚠️ O passe de GPU não é alcançável daqui (ele precisa de um adapter, e os gates de GPU deste
//! repo são `#[ignore]`) — o que se prende aqui é a **partição**, que é onde o defeito visível
//! nasce: uma peça em nenhum dos dois lados desaparece do ecrã, e uma peça nos DOIS ganha um halo
//! do próprio borrão.

use super::lift;
use ph2d_ecs::{Entity, GlobalTransform, MasterEditing, PresentWorld, SimRef, SimWorld, Transform};
use ph2d_render::RenderInstance;

/// Uma instância mínima com um `z_order` reconhecível.
///
/// ⚠️ Escrita por extenso porque a `RenderInstance` **não implementa `Default`** — ela é `Pod` e vai
/// para a GPU, onde um campo esquecido a zero desenha algo errado em silêncio.
fn instance(z: u32) -> RenderInstance {
    RenderInstance {
        world_pos: [0.0, 0.0],
        size: [1.0, 1.0],
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0; 4],
        basis: RenderInstance::IDENTITY_BASIS,
        texture_id: 0,
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: 0,
        z_order: z,
        sampling: 0,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    }
}

/// Uma cena com duas sprites: a `peca` é da receita aberta, a `alheia` não.
fn scene() -> (SimWorld, PresentWorld, Entity, Entity) {
    let mut sim = SimWorld::default();
    let peca = sim
        .world_mut()
        .spawn((Transform::default(), MasterEditing))
        .id();
    let alheia = sim.world_mut().spawn((Transform::default(),)).id();
    let mut present = PresentWorld::new();
    for (e, z) in [(peca, 7u32), (alheia, 3u32)] {
        present.world_mut().spawn((
            SimRef(e),
            GlobalTransform::from_transform(Transform::default()),
            instance(z),
        ));
    }
    (sim, present, peca, alheia)
}

/// ⭐⭐⭐ **Só a peça da receita sobe — e ela sobe nas DUAS respostas.**
///
/// A lista de retenção e a lista de desenho saem da mesma varredura de propósito: se divergissem,
/// uma peça ficaria retida e nunca desenhada (some do ecrã) ou desenhada duas vezes (halo).
///
/// **Mutação que deve sangrar:** o `held.insert` ou o `out.push` sozinhos.
#[test]
fn only_the_open_recipe_rises_above_the_glass() {
    let (sim, mut present, peca, alheia) = scene();
    let mut out = Vec::new();
    let held = lift(&sim, &mut present, &mut out);
    assert!(held.contains(&peca), "a peca da receita nao foi retida");
    assert!(
        !held.contains(&alheia),
        "uma sprite alheia foi retida — ela desapareceria do fundo sem nunca ser desenhada"
    );
    assert_eq!(
        out.len(),
        1,
        "a lista de cima tem de ter exactamente a peca"
    );
    assert_eq!(
        out[0].z_order, 7,
        "a instancia levantada nao e' a da peca da receita"
    );
}

/// ⛔ **Sem receita aberta ninguém sobe** — e é isso que faz o quadro comum não ter passe nenhum a
/// mais.
#[test]
fn without_an_open_recipe_nobody_rises() {
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn((Transform::default(),)).id();
    let mut present = PresentWorld::new();
    present.world_mut().spawn((
        SimRef(e),
        GlobalTransform::from_transform(Transform::default()),
        instance(0),
    ));
    let mut out = Vec::new();
    assert!(lift(&sim, &mut present, &mut out).is_empty());
    assert!(out.is_empty());
}

/// ⚠️ **A lista é LIMPA a cada quadro.** Ela vive no `App` para não realocar; sem o `clear` as
/// peças de todos os quadros anteriores continuariam a ser desenhadas por cima do vidro, e o
/// sintoma seria um rasto que só cresce.
#[test]
fn the_lifted_list_does_not_accumulate_across_frames() {
    let (sim, mut present, _, _) = scene();
    let mut out = vec![instance(99), instance(98)];
    lift(&sim, &mut present, &mut out);
    assert_eq!(
        out.len(),
        1,
        "a lista do quadro anterior sobreviveu: {out:?}"
    );
}
