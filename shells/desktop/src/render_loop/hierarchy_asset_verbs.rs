//! ⭐⭐ **O menu de um CARTÃO da biblioteca, e a poda que ele obrigou** — irmão por ASSUNTO do
//! [`super::hierarchy`], e pelo tecto de 600 LOC do shell (HR-18).
//!
//! ⚠️ **Este corte foi imposto por um gate que esteve VERMELHO sem ninguém ver**
//! (`shell_files_respect_hr18_loc_cap`): ele vive em `shells/desktop/tests/` e o portão de fecho
//! desta linha corria `cargo test --bins`, que **não toca** naquele diretório. Cinco ficheiros
//! desta linha estavam acima do tecto; é a 5.ª ocorrência registada desta família.
//!
//! ⚠️ O corpo continua a correr **de dentro** do `hierarchy::dispatch`, e tem de continuar: é lá
//! que o `sim`, a voz e o gizmo estão os três emprestados ao mesmo tempo.

use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;

/// O dreno do menu do cartão. Devolve **para onde a selecção tem de ir**, se para algum lado, e
/// levanta o `title_dirty` quando o documento mudou.
#[allow(clippy::too_many_arguments)] // o slot, o mundo, o registo, o eco, o hero, a voz, os dois documentos, a câmera, a janela e o sinal de sujo
pub(super) fn drain_card_verb(
    asset_card_verb: Option<(
        ph2d_editor::interaction::drag_payload::DragPayload,
        ph2d_editor::action_bus::AssetCardAction,
    )>,
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    echo: &mut crate::instance_sync::MasterEcho,
    hero: &mut HeroScreen,
    toasts: &mut ph2d_editor::ToastQueue,
    vec_scene: &mut ph2d_vec_scene::VecScene,
    vec_entities: &mut crate::vec_entities::VecEntityMap,
    camera: &ph2d_render::Camera2d,
    window_size: ph2d_host::WindowSize,
    title_dirty: &mut bool,
) -> Option<u64> {
    // ⚠️ Um `select_out` PRÓPRIO: ele é exclusivo dos outros verbos por gesto, mas partilhar a
    // variável faria o mesmo defeito que a nota dos DOIS slots do irmão descreve.
    let mut card_select: Option<u64> = None;
    if let Some((asset, verb)) = asset_card_verb
        && crate::asset_card_verbs::drain(
            asset,
            verb,
            sim,
            registry,
            echo,
            &mut hero.gizmo,
            toasts,
            &mut crate::instance_docs::OwnedDocs {
                vec_scene,
                vec_entities,
            },
            {
                let (dx, dy) = crate::input_dispatch::screen_offset_world(
                    camera,
                    window_size,
                    crate::input_dispatch::PASTE_OFFSET_PX,
                );
                [dx as f32, dy as f32]
            },
            &mut card_select,
        )
    {
        *title_dirty = true;
    }
    card_select
}

/// ⭐⭐⭐ **A selecção nunca guarda bits de uma entidade que já não existe.**
///
/// ⚠️ **Isto não é defensivo — é a metade que o `Remove from Library` obriga a escrever.** Ele é o
/// **primeiro** verbo desta família que APAGA entidades (os outros marcam, copiam ou desligam
/// elos), e uma receita que estivesse seleccionada — pelo modo de edição de mestre — deixaria o
/// gizmo a desenhar à volta de bits mortos. ⛔ O `HierDelete` cura o caso dele à mão, linha a
/// linha, e por isso a cura não estava disponível para mais ninguém.
pub(super) fn prune_dead_selection(hero: &mut HeroScreen, sim: &SimWorld) {
    let dead: Vec<u64> = hero
        .gizmo
        .iter_selected()
        .filter(|bits| {
            sim.world()
                .get_entity(ph2d_ecs::Entity::from_bits(*bits))
                .is_err()
        })
        .collect();
    if dead.is_empty() {
        return;
    }
    if hero.gizmo.selection.is_some_and(|s| dead.contains(&s)) {
        hero.gizmo.selection = None;
    }
    hero.gizmo.extra_selection.retain(|b| !dead.contains(b));
    if hero.gizmo.selection.is_none() && !hero.gizmo.extra_selection.is_empty() {
        hero.gizmo.selection = Some(hero.gizmo.extra_selection.remove(0));
    }
}
