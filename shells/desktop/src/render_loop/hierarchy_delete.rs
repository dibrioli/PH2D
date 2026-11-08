//! ⭐⭐⭐ **O GESTO DE APAGAR na Hierarquia, e as TRÊS respostas que ele pode ter** (ADR-0164).
//!
//! ⚠️ **Irmão por ASSUNTO do [`super::hierarchy`]**, cortado quando a recusa de uma peça (F5.10)
//! levou aquele ficheiro a 618 de 600. *Um tecto paga-se com um corte.*
//!
//! # As três respostas, e por que elas não podem ser duas
//!
//! | o que se aponta | o que acontece |
//! |---|---|
//! | um objecto da cena, ou o que o artista pendurou numa cópia | **apaga** |
//! | uma peça que a RECEITA deu a esta cópia | **recusa-se** — a cópia deixa de a mostrar, e a
//! receita e as irmãs ficam intactas (F5.10) |
//! | uma peça de cópia cuja raiz não se resolve | fica, e a voz diz onde a apagar |
//!
//! ⛔ Até 06/09 a segunda era uma RECUSA em voz alta: faltava ao modelo onde guardar a **intenção**.
//!
//! ⚠️ **E o gesto NÃO despawna a peça recusada** — ele escreve o id dela no `removed` da raiz, e o
//! passe estrutural faz o resto (sepultar as excepções, apagar a sub-árvore, nunca mais a
//! materializar). *Despawnar aqui saltaria o sepultador.*

use ph2d_editor_core::Toast;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_i18n::{tr, tr_with};

/// Drena o pedido de apagar. Devolve `true` quando o documento mudou.
pub(super) fn drain(
    delete_row: Option<ph2d_editor_core::NodeId>,
    hero: &mut HeroScreen,
    hero_live: Option<&super::HeroLive>,
    sim: &mut ph2d_ecs::SimWorld,
    toasts: &mut ph2d_editor_core::ToastQueue,
) -> bool {
    let mut title_dirty = false;
    if let Some(row) = delete_row
        && let Some(live) = hero_live
        && let Some(clicked_entity_bits) = live.bridge.entity_for(row)
    {
        // A linha clicada dentro da multi-selecção apaga a selecção inteira (Figma); o `ChildOf` cascateia.
        let wanted: Vec<u64> = if hero.gizmo.is_selected(clicked_entity_bits) {
            hero.gizmo.iter_selected().collect()
        } else {
            vec![clicked_entity_bits]
        };
        // ⭐⭐⭐ **Uma peça que a RECEITA deu não se apaga numa cópia** (Enio 05/09; o `despawn` era
        // re-materializado com a pose do mestre) — ver o cabeçalho; a porta é `is_a_recipe_given_piece`.
        let (to_delete, from_a_recipe): (Vec<u64>, Vec<u64>) =
            wanted.into_iter().partition(|bits| {
                !ph2d_app_components::instance_verbs::is_a_recipe_given_piece(
                    sim,
                    ph2d_ecs::Entity::from_bits(*bits),
                )
            });
        for bits in &to_delete {
            let entity = ph2d_ecs::Entity::from_bits(*bits);
            sim.world_mut().despawn(entity);
        }
        // Tira os apagados da selecção: senão o gizmo global fica à volta de sprites que sumiram.
        for bits in &to_delete {
            if hero.gizmo.selection == Some(*bits) {
                hero.gizmo.selection = None;
            }
            hero.gizmo.extra_selection.retain(|b| b != bits);
        }
        // If primary was deleted but extras remain, promote the
        // oldest extra so the selection isn't headless.
        if hero.gizmo.selection.is_none() && !hero.gizmo.extra_selection.is_empty() {
            hero.gizmo.selection = Some(hero.gizmo.extra_selection.remove(0));
        }
        // ⭐⭐⭐ **A peça da receita passa a ser RECUSADA por esta cópia** (F5.10, o *Removed GameObject*
        // do Unity): o id vai ao `removed` da raiz e o passe estrutural faz o resto (cabeçalho).
        let refused = ph2d_app_components::instance_structure::refuse_pieces(sim, &from_a_recipe);
        // ⭐⭐ **Recusar uma peça move a selecção para a CÓPIA**: o cartão dela tem o *Put back*.
        if refused > 0
            && let Some(first) = from_a_recipe.first()
            && let Some(root) = ph2d_app_components::instance_verbs::instance_root_of(
                sim,
                ph2d_ecs::Entity::from_bits(*first),
            )
        {
            hero.gizmo.replace_selection(Some(root.to_bits()));
        }
        let n = to_delete.len();
        let kept = from_a_recipe.len() - refused;
        // ⚠️ Os TRÊS números são ditos, e o zero também; «removido desta cópia» ≠ «apagado».
        if n > 0 || refused > 0 {
            toasts.push(Toast::warning(match (n, refused, kept) {
                (1, 0, 0) => tr("shell.hierarchy_delete.deleted_entity").to_string(),
                (0, 1, 0) => tr("shell.hierarchy_delete.removed_from_this_copy").to_string(),
                (0, r, 0) => tr_with("shell.hierarchy_delete.removed_piece_s_from", &[("r", &r)]),
                (_, 0, 0) => tr_with("shell.hierarchy_delete.deleted_entities", &[("n", &n)]),
                (_, r, 0) => tr_with(
                    "shell.hierarchy_delete.deleted_removed_from",
                    &[("n", &n), ("r", &r)],
                ),
                _ => tr_with(
                    "shell.hierarchy_delete.deleted_removed_from_2",
                    &[("n", &n), ("refused", &refused), ("kept", &kept)],
                ),
            }));
            title_dirty = true;
        } else if kept > 0 {
            toasts.push(Toast::warning(tr(
                "shell.hierarchy_delete.that_piece_comes_from",
            )));
        }
    }
    title_dirty
}
