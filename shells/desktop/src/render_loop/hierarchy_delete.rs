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
//! ⛔ **Até 2026-09-06 a segunda era uma RECUSA em voz alta**, e ela estava certa para o modelo de
//! então: a forma de uma cópia era **sempre** a da receita, e um `despawn` cru era desfeito pelo
//! passe estrutural no quadro seguinte — *«um gesto que não faz nada é mau; um que desfaz outra
//! coisa é pior»*. O que faltava não era permitir o `despawn`: era o modelo ter onde guardar a
//! **intenção**.
//!
//! ⚠️ **E o gesto NÃO despawna a peça recusada** — ele escreve o id dela no `removed` da raiz, e o
//! passe estrutural faz o resto (sepultar as excepções, apagar a sub-árvore, nunca mais a
//! materializar). *Despawnar aqui saltaria o sepultador.*

use ph2d_editor::Toast;
use ph2d_editor::screens::hero::HeroScreen;

/// Drena o pedido de apagar. Devolve `true` quando o documento mudou.
pub(super) fn drain(
    delete_row: Option<ph2d_editor::NodeId>,
    hero: &mut HeroScreen,
    hero_live: Option<&super::HeroLive>,
    sim: &mut ph2d_ecs::SimWorld,
    toasts: &mut ph2d_editor::ToastQueue,
) -> bool {
    let mut title_dirty = false;
    if let Some(row) = delete_row
        && let Some(live) = hero_live
        && let Some(clicked_entity_bits) = live.bridge.entity_for(row)
    {
        // Onda 2 fix: if the clicked row is part of the multi-
        // selection, delete EVERY selected sprite (Photoshop / Figma
        // convention — multi-select right-click → Delete affects the
        // whole group). Otherwise just the clicked row. bevy_ecs 0.19
        // `ChildOf` cascade despawns descendants.
        let wanted: Vec<u64> = if hero.gizmo.is_selected(clicked_entity_bits) {
            hero.gizmo.iter_selected().collect()
        } else {
            vec![clicked_entity_bits]
        };
        // ⭐⭐⭐ **UMA PEÇA QUE A RECEITA DEU NÃO SE APAGA NUMA CÓPIA** (report do Enio,
        // 2026-09-05: *«ao tentar deletar o objeto, ele não é deletado e volta para sua posição de
        // origem»*).
        //
        // ⛔⛔ **Sem esta guarda o gesto era o pior dos três resultados possíveis**, e foi medido:
        // o `despawn` passava, o passe estrutural **re-materializava** a peça no quadro seguinte
        // (o mestre continua a tê-la — *só o que a receita deu é que a receita tira*), e ela
        // voltava com a pose do **MESTRE** ⇒ a edição do artista naquela peça **desaparecia em
        // silêncio**, com a chave de override dela a sobreviver a apontar para o valor que já não
        // existe. *Um gesto que não faz nada é mau; um que desfaz outra coisa é pior.*
        //
        // ⚠️ **A voz diz ONDE fazer**: a mesma pergunta com resposta «não» e sem saída foi o que
        // levou este report a existir — o artista tinha duas linhas chamadas `Body` na Hierarquia,
        // uma da receita e uma da cópia, e nada lhe disse qual era qual.
        //
        // ⛔ **A recusa é NARROW e a porta é uma só** ([`crate::instance_verbs_walk::is_a_recipe_given_piece`]):
        // apagar a cópia INTEIRA continua a ser um gesto normal, e o que o artista pendurou dentro
        // dela (sem elo) também — *o passe estrutural já declara as duas metades*.
        let (to_delete, from_a_recipe): (Vec<u64>, Vec<u64>) =
            wanted.into_iter().partition(|bits| {
                !crate::instance_verbs::is_a_recipe_given_piece(
                    sim,
                    ph2d_ecs::Entity::from_bits(*bits),
                )
            });
        for bits in &to_delete {
            let entity = ph2d_ecs::Entity::from_bits(*bits);
            sim.world_mut().despawn(entity);
        }
        // Remove every deleted bits from the selection set. Without
        // this, `selected_len()` stays > 1 even after a multi-delete,
        // which keeps the global gizmo painted around vanished sprites
        // (user-reported: "se deletar algumas e sobrar 1, o gizmo
        // global fica aparecendo mesmo com uma sprite").
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
        // ⭐⭐⭐ **A peça da receita passa a ser RECUSADA por esta cópia** (F5.10, Enio 2026-09-06:
        // *«sim, quero que construa»*) — o *Removed GameObject* do Unity.
        //
        // ⛔ **Até aqui isto era uma RECUSA em voz alta**, e ela estava certa para o modelo de
        // então: a forma de uma cópia era sempre a da receita, e um `despawn` cru era desfeito pelo
        // passe estrutural no quadro seguinte — *«um gesto que não faz nada é mau; um que desfaz
        // outra coisa é pior»*. O que faltava não era permitir o `despawn`: era o modelo ter onde
        // guardar a **intenção**.
        //
        // ⚠️ **E o gesto NÃO despawna** — ele escreve o id da peça do mestre no `removed` da raiz, e
        // o passe estrutural faz tudo o resto (sepultar as excepções dela, apagar a sub-árvore, e
        // nunca mais a materializar). *Despawnar aqui saltaria o sepultador e deixaria a excepção
        // daquela peça nem viva nem enterrada* — a mesma lei que a raiz do `swap` pagou ontem.
        let refused = crate::instance_structure::refuse_pieces(sim, &from_a_recipe);
        // ⭐⭐ **Recusar uma peça move a selecção para a CÓPIA**, e não é conveniência: a peça que
        // estava escolhida vai deixar de existir no quadro seguinte, e uma selecção pendurada num
        // objecto morto **apaga o cartão do Inspector** — que é exactamente onde vive o *Put back*
        // que desfaz este gesto. *O gesto que esconde a sua própria saída é irreversível pelo
        // painel.* ⚠️ Vai para a RAIZ porque é lá que a decisão fica guardada, e é o cartão dela
        // que a lista.
        if refused > 0
            && let Some(first) = from_a_recipe.first()
            && let Some(root) =
                crate::instance_verbs::instance_root_of(sim, ph2d_ecs::Entity::from_bits(*first))
        {
            hero.gizmo.replace_selection(Some(root.to_bits()));
        }
        let n = to_delete.len();
        let kept = from_a_recipe.len() - refused;
        // ⚠️ **Os dois números são ditos, e o zero também**: um gesto que apaga metade e cala a
        // outra metade lê-se como um apagar que falhou às vezes.
        // ⚠️ **Os TRÊS números são ditos, e o zero também**: um gesto que apaga uma parte e cala a
        // outra lê-se como um apagar que falhou às vezes. E *«removido só desta cópia»* é uma
        // resposta diferente de *«apagado»* — o artista tem de saber que a receita ficou intacta.
        if n > 0 || refused > 0 {
            toasts.push(Toast::warning(match (n, refused, kept) {
                (1, 0, 0) => "Deleted entity".to_string(),
                (0, 1, 0) => {
                    "Removed from this copy \u{2014} the component still has it".to_string()
                }
                (0, r, 0) => format!("Removed {r} piece(s) from this copy only"),
                (_, 0, 0) => format!("Deleted {n} entities"),
                (_, r, 0) => format!("Deleted {n} \u{2014} {r} removed from this copy only"),
                _ => format!(
                    "Deleted {n} \u{2014} {refused} removed from this copy \u{2014} {kept} stayed"
                ),
            }));
            title_dirty = true;
        } else if kept > 0 {
            toasts.push(Toast::warning(
                "That piece comes from a component \u{2014} delete it in the component, or Detach this copy first",
            ));
        }
    }
    title_dirty
}
