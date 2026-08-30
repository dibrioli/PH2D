//! ⭐⭐ **O que os três itens do menu de um cartão FAZEM** (plano 07, etapa C).
//!
//! O painel transporta o par `(endereço, verbo)` e **não decide nada** — ele não tem `ToastQueue`
//! (ver [`ph2d_editor::action_bus::EditorAction::AssetCardVerb`]). Quem decide e fala é este
//! módulo, que é o único lado com as três coisas na mão: o mundo (*quem usa isto?*), a voz e os
//! verbos de instância.
//!
//! # ⭐⭐⭐ A tabela é PLANA, e é por isso que as recusas são o corpo deste ficheiro
//!
//! Os três itens aparecem sobre um **Prefab** e sobre uma **Imagem**, porque o menu não sabe qual
//! é (a mesma lei do menu da Hierarquia). ⇒ das seis células, **três** são recusas — e cada uma
//! **nomeia o facto**, nunca *«não dá»*:
//!
//! | | Prefab | Imagem |
//! |---|---|---|
//! | **Instantiate** | põe uma cópia (cascata) | ⛔ *qual objecto a recebe?* — é a queda que responde |
//! | **Select users** | selecciona as instâncias | selecciona os objectos que a desenham |
//! | **Remove from Library** | a lei das duas metades ([`crate::instance_unmake`]) | ⛔ ela está lá **porque N objectos a usam** — tirá-la seria tirá-la deles |
//!
//! ⚠️ **A recusa da Imagem no *Remove* traz o NÚMERO**, e não é decoração: sem ele a frase é uma
//! opinião, e com ele o artista sabe exactamente o que tem de fazer para a tirar (mudar aqueles N).
//! *É a mesma informação que o `Select users` entrega como gesto.*

use ph2d_ecs::{Entity, InstanceOf, SimWorld, SpritePixels, StableId};
use ph2d_editor::Toast;
use ph2d_editor::action_bus::AssetCardAction;
use ph2d_editor::interaction::drag_payload::DragPayload;

/// **Quem usa este asset**, em bits de entidade, ordenado por `StableId`.
///
/// ⚠️ **A ordem é do `StableId` e não da consulta**: a primeira da lista vira a selecção primária,
/// e a ordem de arquétipo do `bevy_ecs` faria o gizmo aterrar noutro objecto entre corridas.
///
/// ⚠️ **Um Prefab e uma Imagem respondem à MESMA pergunta por caminhos diferentes** — o elo
/// (`InstanceOf`) contra o conteúdo (`SpritePixels`) — e é por isso que a resposta é uma função só:
/// duas funções divergiriam no dia em que uma delas ganhasse um filtro.
pub(crate) fn users_of(sim: &mut SimWorld, asset: DragPayload) -> Vec<u64> {
    let mut out: Vec<(u64, u64)> = match asset {
        DragPayload::Prefab { stable_id } => {
            let mut q = sim.world_mut().query::<(Entity, &InstanceOf, &StableId)>();
            q.iter(sim.world())
                .filter(|(_, link, _)| link.master == stable_id)
                .map(|(e, _, sid)| (sid.0, e.to_bits()))
                .collect()
        }
        DragPayload::Image { asset } => {
            let wanted = ph2d_asset::AssetId::from_digest(asset);
            let mut q = sim
                .world_mut()
                .query::<(Entity, &SpritePixels, &StableId)>();
            q.iter(sim.world())
                .filter(|(_, px, _)| px.0 == wanted)
                .map(|(e, _, sid)| (sid.0, e.to_bits()))
                .collect()
        }
    };
    out.sort_unstable();
    out.into_iter().map(|(_, bits)| bits).collect()
}

/// O que o menu do cartão faz. Devolve `true` quando alguma coisa mudou no documento.
///
/// ⚠️ **O `Select users` devolve `false` de propósito** — mudar a selecção não é uma edição, e
/// marcá-la como tal poria um passo de undo sobre um gesto de *ver*.
// um endereço, um verbo, o mundo, o registo, o eco, o gizmo, a voz, os documentos, o passo e o
// destino da selecção — a mesma conta do `instance_verbs::drain`, que ele reusa
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain(
    asset: DragPayload,
    verb: AssetCardAction,
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    echo: &mut crate::instance_sync::MasterEcho,
    gizmo: &mut ph2d_editor::screens::hero::GizmoStateGroup,
    toasts: &mut ph2d_editor::ToastQueue,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
    place_step: [f32; 2],
    select_out: &mut Option<u64>,
) -> bool {
    match (verb, asset) {
        // ── Instanciar ─────────────────────────────────────────────────────────────────────────
        (AssetCardAction::Instantiate, DragPayload::Prefab { stable_id }) => {
            let Some(bits) = crate::instance_verbs::entity_for_stable_id(sim, stable_id) else {
                toasts.push(Toast::warning("That prefab is no longer in the project"));
                return false;
            };
            crate::instance_verbs::drain(
                crate::instance_verbs::Verb::Place,
                sim,
                registry,
                echo,
                bits,
                toasts,
                docs,
                place_step,
                select_out,
            )
        }
        (AssetCardAction::Instantiate, DragPayload::Image { .. }) => {
            // ⛔ A mesma recusa que o duplo-clique já declara, agora **em voz alta**: no
            // duplo-clique o silêncio era defensável (ninguém apertou um item que prometia algo);
            // num item de menu com o nome escrito, não é.
            toasts.push(Toast::info(
                "Drop an image on an object to use it \u{2014} an image has no place of its own",
            ));
            false
        }

        // ── Quem usa isto ──────────────────────────────────────────────────────────────────────
        (AssetCardAction::SelectUsers, _) => {
            let users = users_of(sim, asset);
            if users.is_empty() {
                toasts.push(Toast::info(match asset {
                    DragPayload::Prefab { .. } => "No copies of this prefab in the scene",
                    DragPayload::Image { .. } => "Nothing is using this image",
                }));
                return false;
            }
            let n = users.len();
            gizmo.replace_selection(Some(users[0]));
            for bits in &users[1..] {
                gizmo.add_to_selection(*bits);
            }
            toasts.push(Toast::success(format!("Selected {n} object(s)")));
            false
        }

        // ── Tirar da biblioteca ────────────────────────────────────────────────────────────────
        (AssetCardAction::RemoveFromLibrary, DragPayload::Prefab { stable_id }) => {
            let Some(bits) = crate::instance_verbs::entity_for_stable_id(sim, stable_id) else {
                toasts.push(Toast::warning("That prefab is no longer in the project"));
                return false;
            };
            crate::instance_verbs::drain(
                crate::instance_verbs::Verb::Unmake,
                sim,
                registry,
                echo,
                bits,
                toasts,
                docs,
                place_step,
                select_out,
            )
        }
        (AssetCardAction::RemoveFromLibrary, DragPayload::Image { .. }) => {
            // ⚠️ O número é o corpo da recusa — ver o cabeçalho do módulo.
            let n = users_of(sim, asset).len();
            toasts.push(Toast::info(format!(
                "This image is in the library because {n} object(s) use it \u{2014} change those to remove it"
            )));
            false
        }
    }
}
