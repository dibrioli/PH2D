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
pub(crate) fn users_of(
    sim: &mut SimWorld,
    asset: DragPayload,
    atlas_assets: &std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
) -> Vec<u64> {
    let mut out: Vec<(u64, u64)> = match asset {
        DragPayload::Prefab { stable_id } => {
            let mut q = sim.world_mut().query::<(Entity, &InstanceOf)>();
            q.iter(sim.world())
                .filter(|(_, link)| link.master == stable_id)
                .map(|(e, _)| e)
                .collect::<Vec<_>>()
        }
        DragPayload::Image { asset } => {
            // ⚠️ **Pela MESMA porta que o índice usa** ([`crate::asset_index_build::texture_of`]) —
            // uma sprite de átlas não carrega `SpritePixels`, e perguntar só por ele fazia o
            // *Select users* devolver **zero** sobre toda imagem importada. *A pergunta «que
            // textura é esta?» tem um dono, e ele não é este ficheiro.*
            let wanted = ph2d_asset::AssetId::from_digest(asset);
            let mut q =
                sim.world_mut()
                    .query::<(Entity, Option<&SpritePixels>, Option<&ph2d_render::Sprite>)>();
            q.iter(sim.world())
                .filter(|(_, px, spr)| {
                    crate::asset_index_build::texture_of(*px, *spr, atlas_assets) == Some(wanted)
                })
                .map(|(e, _, _)| e)
                .collect::<Vec<_>>()
        }
    }
    .into_iter()
    // ⛔⛔ **UMA PEÇA DE RECEITA NÃO É UM UTILIZADOR** (auditoria de 2026-08-30, achado nº 1).
    //
    // As peças de uma receita carregam `SpritePixels` como qualquer outra, então a varredura crua
    // contava-as — e elas **não estão na cena**: o extract salta-as e a Hierarquia não lhes dá
    // linha. Consequência medida: com uma imagem usada só por uma receita, o `Select users` dizia
    // *«Selected 1 object(s)»* e **nada acendia**. É a espécie *«o consumidor que PROJECTA o valor
    // fora»* do §5.0 — o fio estava completo e o consumidor descartava.
    //
    // ⚠️ **A pergunta tem UMA porta nesta casa** ([`crate::render_loop::off_canvas`]), e ela é a
    // mesma que o extract e a lista da Hierarquia consomem. ⛔ Escrever aqui um segundo predicado
    // seria a lei em dois sítios — e o segundo envelheceria no dia em que o modo de edição de
    // mestre mudasse.
    .filter(|&e| !crate::render_loop::off_canvas::is_unedited_recipe(sim.world(), e))
    // A ordem é do `StableId` e não da consulta: a primeira da lista vira a selecção primária, e a
    // ordem de arquétipo do `bevy_ecs` faria o gizmo aterrar noutro objecto entre corridas.
    // ⚠️ **`map_or(u64::MAX, …)` e não `&StableId` na consulta:** uma entidade nascida no mesmo
    // quadro ainda não tem id (o `assign_missing_stable_ids` corre uma vez por quadro), e exigi-lo
    // fá-la-ia desaparecer da resposta **em silêncio**.
    .map(|e| {
        (
            sim.world().get::<StableId>(e).map_or(u64::MAX, |s| s.0),
            e.to_bits(),
        )
    })
    .collect();
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
    // ⭐ `célula do átlas → AssetId` — ver [`crate::asset_index_build::texture_of`].
    atlas_assets: &std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
    select_out: &mut Option<u64>,
) -> bool {
    match (verb, asset) {
        // ── Editar a receita ───────────────────────────────────────────────────────────────────
        //
        // ⭐⭐⭐ **Ele SELECCIONA, e mais nada** (report do Enio, 2026-09-05: *«não tem como editar
        // o componente»*). A receita não está na cena e **volta enquanto está seleccionada** — a
        // marca derivada `MasterEditing` do [`crate::render_loop::master_editing`] —, então o verbo
        // que faltava não era um modo nem uma janela: era um **acesso**. Pôr a selecção na raiz do
        // mestre acende o canvas, arma o gizmo, enche o Inspector e faz cada peça mexida chegar a
        // todas as cópias no mesmo quadro.
        //
        // ⛔ **A biblioteca era read-only para a FORMA**: listava, instanciava e respondia quem usa
        // o quê, e não tinha como abrir um componente. *Um catálogo de onde não se edita o conteúdo
        // é uma vitrina.*
        (AssetCardAction::EditPrefab, DragPayload::Prefab { stable_id }) => {
            let Some(bits) = crate::instance_verbs::entity_for_stable_id(sim, stable_id) else {
                toasts.push(Toast::warning("That prefab is no longer in the project"));
                return false;
            };
            *select_out = Some(bits);
            let name = crate::instance_verbs::master_named(sim, stable_id)
                .unwrap_or_else(|| "component".to_string());
            toasts.push(Toast::success(format!(
                "Editing \u{201c}{name}\u{201d} \u{2014} move a piece and every copy follows"
            )));
            true
        }
        (AssetCardAction::EditPrefab, DragPayload::Image { .. }) => {
            // ⛔ **A recusa nomeia o FACTO, como as outras três da tabela plana.** Uma imagem não
            // tem forma que este app autore — quem a edita são as ferramentas de imagem, sobre o
            // objecto que a desenha, e é por isso que a saída é o *Select users*.
            toasts.push(Toast::info(
                "An image has no shape to edit here \u{2014} use \u{201c}Select users\u{201d} and \
                 edit it on an object",
            ));
            false
        }

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
            let users = users_of(sim, asset, atlas_assets);
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

        // ── Trocar o que está escolhido por este componente ────────────────────────────────────
        //
        // ⭐⭐⭐ **O único verbo deste menu cujo sujeito NÃO é o cartão** (plano F5, o último
        // critério): o cartão é o *objecto* da frase e a **selecção** é o sujeito. É por isso que os
        // três rótulos a nomeiam — um item que age sobre outra coisa que a apontada tem de o dizer.
        //
        // ⚠️ **Três verbos e não um com um modo dentro.** Sem antepassado comum não existe mapa
        // derivado, só palpite, e o plano manda que o palpite seja **pedido pelo gesto**. Ver
        // [`crate::instance_swap_match`].
        (
            AssetCardAction::ReplaceSelection
            | AssetCardAction::ReplaceSelectionByName
            | AssetCardAction::ReplaceSelectionByTree,
            DragPayload::Prefab { stable_id },
        ) => replace_selection(verb, stable_id, sim, echo, gizmo, toasts),
        (
            AssetCardAction::ReplaceSelection
            | AssetCardAction::ReplaceSelectionByName
            | AssetCardAction::ReplaceSelectionByTree,
            DragPayload::Image { .. },
        ) => {
            // ⛔ A quarta recusa da tabela plana, e ela nomeia o facto como as outras três: trocar
            // é trocar de RECEITA, e uma imagem não é uma. O que o artista quer aqui tem outro
            // gesto — largá-la sobre o objecto — e a frase manda-o para lá.
            toasts.push(Toast::info(
                "An image is not a component \u{2014} drop it on an object to change what it draws",
            ));
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
        (AssetCardAction::RemoveFromLibrary, DragPayload::Image { asset: id }) => {
            let n = users_of(sim, asset, atlas_assets).len();
            if n == 0 {
                // ⭐⭐⭐ **NINGUÉM a usa ⇒ ela SAI** (report do Enio, 2026-08-30, 2.ª ronda:
                // *«uma sprite que foi deletada do canvas não consegui deletar do painel»*).
                //
                // ⛔ A 1.ª versão recusava sempre, e com a sprite apagada a frase virava *«está na
                // biblioteca porque 0 objecto(s) a usam — mude esses para a tirar»*: um beco sem
                // saída que manda mudar um conjunto vazio.
                crate::asset_index_build::forget_texture(ph2d_asset::AssetId::from_digest(id));
                toasts.push(Toast::success("Removed from library"));
                // ⭐⭐ **`true` desde 2026-08-30, e a inversão é o pedido do Enio** (*«deveria ter
                // undo/redo no painel inclusive em del»*). A 1.ª versão devolvia `false` com o
                // motivo *«a biblioteca é memória de SESSÃO e o undo não desfaz isto»* — e era
                // verdade: o gesto era **irreversível**, porque uma imagem sem utilizadores não
                // tem quem a re-lembre no quadro seguinte.
                //
                // ⇒ hoje o `forget` é uma **lápide** que viaja no `ProjectState`
                // ([`crate::project_library`]), logo isto É uma edição do documento.
                return true;
            }
            // ⚠️ **Com utilizadores a recusa CONTINUA certa**, e o número é o corpo dela: tirar a
            // imagem deixaria aqueles objectos sem pixels, e não há saída sem perda. *O que estava
            // errado era aplicar esta frase ao caso em que ninguém tem nada a perder.*
            toasts.push(Toast::info(format!(
                "This image is in the library because {n} object(s) use it \u{2014} change those to remove it"
            )));
            false
        }
    }
}

/// ⭐⭐⭐ **Trocar cada cópia escolhida pelo componente do cartão** (ADR-0164 / plano F5).
///
/// # ⚠️ O sujeito é a SELECÇÃO, e ele espalha-se por ela
///
/// *«Replace selection»* no plural é o que o rótulo promete, e cumpri-lo é a diferença entre um
/// verbo e uma armadilha: com cinco cópias escolhidas, trocar só a primária seria uma acção
/// **parcial em silêncio**. ⚠️ E o sujeito de cada uma é a **raiz da instância**, não a entidade
/// clicada — o artista escolhe uma peça dentro da cópia tantas vezes quantas escolhe a raiz, e
/// exigir a raiz faria o gesto falhar sem dizer porquê.
///
/// # ⚠️ Uma voz só, com os números dentro
///
/// Uma selecção mista (cópias, objectos soltos, cópias que já são deste componente) daria um toast
/// por objecto. ⇒ um resumo, e ele **nomeia o que não aconteceu**: o que ficou por trocar e os
/// nomes que apareceram mais que uma vez. *A metade DURÁVEL do relatório é outra* — cada excepção
/// que perdeu o alvo vai para a lista de *sem alvo* do cartão do Inspector (F5.6), com a peça de
/// que era. Um toast diz **quantas**; a lista diz **quais**.
fn replace_selection(
    verb: AssetCardAction,
    stable_id: u64,
    sim: &mut SimWorld,
    echo: &mut crate::instance_sync::MasterEcho,
    gizmo: &ph2d_editor::screens::hero::GizmoStateGroup,
    toasts: &mut ph2d_editor::ToastQueue,
) -> bool {
    use crate::instance_swap_match::WhenUnrelated;
    let how = match verb {
        AssetCardAction::ReplaceSelectionByName => WhenUnrelated::ByName,
        AssetCardAction::ReplaceSelectionByTree => WhenUnrelated::ByHierarchy,
        // ⚠️ O item sem adjectivo. ⛔ **Nunca `Refuse`** — aqui o gesto JÁ nomeou um modo; a recusa
        // é o caminho de quem não pediu nada, e esse não passa por esta função.
        _ => WhenUnrelated::CarryNothing,
    };
    let chosen: Vec<u64> = gizmo.iter_selected().collect();
    if chosen.is_empty() {
        toasts.push(Toast::info(
            "Pick the copy you want to replace first \u{2014} then choose this again",
        ));
        return false;
    }
    // As raízes, sem repetições: duas peças da MESMA cópia escolhidas são uma troca, não duas.
    let mut roots: Vec<u64> = chosen
        .into_iter()
        .filter_map(|bits| {
            crate::instance_verbs::instance_root_of(sim, Entity::from_bits(bits))
                .map(|e| e.to_bits())
        })
        .collect();
    roots.sort_unstable();
    roots.dedup();

    let (mut done, mut kept, mut ambiguous, mut already, mut skipped) = (0usize, 0, 0, 0, 0);
    for bits in roots {
        match crate::instance_variant::swap(sim, echo, Entity::from_bits(bits), stable_id, how) {
            Ok(r) => {
                done += 1;
                kept += r.overrides_kept;
                ambiguous += r.ambiguous;
            }
            Err(crate::instance_variant::SwapRefusal::Already) => already += 1,
            Err(_) => skipped += 1,
        }
    }
    if done == 0 {
        // ⚠️ **Cada caminho vazio diz uma coisa DIFERENTE**, e as três são accionáveis: *já é este*
        // não pede nada, *não é uma cópia* diz o que escolher, e a terceira é o resto.
        toasts.push(Toast::info(if already > 0 {
            "Those are already copies of this component"
        } else if skipped > 0 {
            "That copy cannot become this component"
        } else {
            "Nothing you picked is a copy of a component"
        }));
        return false;
    }
    let name = crate::instance_verbs::master_named(sim, stable_id)
        .unwrap_or_else(|| "component".to_string());
    let mut say = format!(
        "Replaced {done} object(s) with \u{201c}{name}\u{201d} \u{2014} {kept} override(s) kept"
    );
    if ambiguous > 0 {
        say.push_str(&format!(
            " \u{b7} {ambiguous} name(s) used more than once were skipped"
        ));
    }
    if already + skipped > 0 {
        say.push_str(&format!(" \u{b7} {} left alone", already + skipped));
    }
    toasts.push(Toast::success(say));
    true
}

#[cfg(test)]
#[path = "asset_card_verbs_tests.rs"]
mod tests;
