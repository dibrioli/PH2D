//! ⭐ **O NASCIMENTO DE UMA PEÇA** — a porta única por onde uma peça entra na cena (a semente do
//! módulo e o menu Add). Saiu do [`crate::scene`] pelo tecto de 700 linhas por ficheiro na rodada de
//! integração de 04/10: a `line/3DModeling` e a `line/UIUX` cresciam o mesmo ficheiro, e só juntas
//! passavam do tecto.

/// ⭐ **PLANTA uma peça** a partir de um documento — a porta do nascimento, partilhada pela
/// semente do módulo e pelo menu Add (que planta a 2.ª, 3.ª… peça ao lado das outras, spec/06 F3).
/// Devolve a raiz e quem seleccionar ao nascer: o 1.º filho (ver [`crate::scene::sync_scene_and_birth`]).
pub(crate) fn plant(
    world: &mut bevy_ecs::world::World,
    doc: &ph2d_field::FieldDoc,
) -> (bevy_ecs::entity::Entity, u64) {
    // ⚠️ O nome é ÚNICO entre as raízes: duas linhas «Model» na Hierarquia não se distinguem.
    let name = ph2d_field_ecs::unique_root_name(world, crate::scene::PART_NAME);
    let root = ph2d_field_ecs::spawn_doc(world, doc, &name);
    // ⭐⭐⭐ **E O MATERIAL QUE A CENA PEDE entra AQUI, folha a folha** (2026-09-18).
    //
    // ⛔⛔ Ele existe por um report do dono: a lei da matiz que segue a profundidade
    // (`docs/Render3d/10` §18) foi construída e ligada, e ele viu **a mesma imagem** dos
    // dois lados — porque o material de omissão é **CINZENTO**, e uma lei que redistribui
    // saturação *entre canais* não tem o que fazer quando os três já são iguais. *Uma cena
    // que não consegue conter o fenómeno não pode demonstrá-lo*, e até aqui uma cena não
    // tinha como pedir o material de que precisa.
    //
    // ⚠️ **A ordem é a das FOLHAS**, a mesma que o `spawn_doc` usa (`spawned[i]` indexado
    // pelo nó) e a mesma que a [`crate::materials::leaves`] percorre. ⭐ E a semente
    // **gasta-se**: depois do plantio quem manda no material é o mundo, senão isto
    // sobrescreveria o que o artista pintou a cada replantio.
    if let Some(pedidos) = crate::smoke::with_smoke(|s| s.seed_materials.take()).flatten() {
        let folhas: Vec<bevy_ecs::entity::Entity> = ph2d_field_ecs::walk(world, root)
            .into_iter()
            .filter(|(e, _)| {
                matches!(
                    world.get::<ph2d_field_ecs::FieldNode>(*e),
                    Some(ph2d_field_ecs::FieldNode {
                        shape: ph2d_field::NodeShape::Leaf(_)
                    })
                )
            })
            .map(|(e, _)| e)
            .collect();
        for (e, m) in folhas.into_iter().zip(pedidos) {
            world.entity_mut(e).insert(m);
        }
    }
    crate::texturas::planta(world, root);
    // ⭐⭐⭐ **E UMA LUZ NASCE COM ELA** (ordem do dono, 14/09: *«a luz deve virar objeto 3d
    // como nos app 3d»*) — como num aplicativo 3D, uma cena nova já tem uma.
    //
    // ⚠️ **Sem isto o modo Render perdia a lâmpada de estúdio e não ganhava nenhuma:** o
    // rig ancorado no ecrã deixou de acender o Render nesta wave, e uma cena sem luz sai
    // acesa **só pelo céu**. *Uma feature que exige um gesto antes de a cena voltar a
    // parecer-se com ela própria não é uma feature: é uma regressão com um botão ao lado.*
    //
    // ⚠️ **Acima e à esquerda, e não na origem** — é a direcção que o `Light::KEY` do rig
    // sempre teve (`230°` de azimute, `30°` de elevação), lida dali e convertida uma vez.
    // *A peça abre com a luz que ela tinha, num sítio que o artista pode agarrar.*
    //
    // ⚠️ **Uma luz por CENA, não por peça** (spec/06 F3): a 2.ª peça do menu Add nasce ao lado da
    // 1.ª, e uma 2.ª lâmpada dobraria a luz que o artista já tinha afinado.
    if !world
        .query::<&ph2d_field_ecs::FieldLight>()
        .iter(world)
        .any(|_| true)
    {
        let cam = crate::smoke::with_smoke(|s| s.vp().cam).unwrap_or_default();
        let (onde, lampada) = crate::lights::opening_light(&cam);
        ph2d_field_ecs::add_light(world, onde, lampada);
    }
    // ⭐⭐⭐ **A SEMENTE DECLARA-SE AUTORADA** (2026-09-04) — a mesma lei da forma da
    // paleta e da escultura importada (W115): ela nasce num quadro **sem evento**, e sem
    // esta linha fundia-se no PRIMEIRO passo do artista. Medido pelo pill: `nos=0→4`
    // suprimido em 20 quadros seguidos, e o primeiro `Ctrl+Z` depois de criar uma forma
    // apagava a forma **e** a peça de demo (`nos=5→0`) — *«um Ctrl+Z apaga tudo»*.
    crate::smoke::mark_authored_change();
    let first = world
        .get::<bevy_ecs::hierarchy::Children>(root)
        .and_then(|c| c.iter().copied().next())
        .unwrap_or(root)
        .to_bits();
    (root, first)
}
