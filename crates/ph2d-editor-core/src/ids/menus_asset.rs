//! ⭐⭐ **O menu de um CARTÃO da biblioteca de assets** (plano `docs/Components/07`, etapa C).
//!
//! Cortado de `menus.rs` (que passou o tecto de LOC do ficheiro) pelo mesmo critério que cortou o
//! `menus_timeline.rs`: eles são **um assunto**. Aqui o assunto é *«o que se faz a um asset da
//! biblioteca»* — usar, ver quem o usa, tirá-lo.
//!
//! # ⚠️ A tabela é PLANA, e é isso que obriga cada item a falar
//!
//! O menu não sabe se o cartão é um **Prefab** ou uma **Imagem** — o
//! [`crate::interaction::ContextMenuKind::AssetCard`] carrega a CÉLULA, e quem a converte em
//! endereço de asset é o painel (a única crate que conhece o `AssetRef`). ⇒ os três itens aparecem
//! sobre as duas famílias, e **três das seis células são recusas**. Quem as redige é o shell
//! (`shells/desktop/src/asset_card_verbs.rs`), porque o `PanelHostInternal` não dá `ToastQueue` e
//! uma recusa muda é pior que um item ausente.

use super::*;

/// ⭐ **Instanciar** o prefab do cartão. Numa Imagem responde com o motivo: pôr uma imagem na cena
/// é a **queda** num alvo, porque *qual* objecto a recebe é o que a queda responde e um item de
/// menu não.
pub const CTX_MENU_ASSET_INSTANTIATE: NodeId = hash_node_id("ctx_menu_asset_instantiate");
/// ⭐⭐ **Quem usa isto?** — a metade que o Godot chama *Owners* e que responde *«posso apagar?»*.
///
/// ⚠️ Ela **selecciona** os utilizadores na cena em vez de os listar: uma lista diz um número, uma
/// selecção põe o artista em cima deles, com o gizmo e o Inspector já apontados.
pub const CTX_MENU_ASSET_SELECT_USERS: NodeId = hash_node_id("ctx_menu_asset_select_users");
/// ⭐⭐ **Tirar da biblioteca** (report do Enio, 2026-08-30: *«não dá para tirar um asset da
/// biblioteca. Ele entra e fica»*). A lei das duas metades — as cópias destacam-se e a receita
/// dissolve-se, **ou** ela volta à cena por ser a última cópia — vive em
/// `shells/desktop/src/instance_unmake.rs`.
pub const CTX_MENU_ASSET_REMOVE: NodeId = hash_node_id("ctx_menu_asset_remove");

/// ⭐⭐ **Tirar da biblioteca, a partir da HIERARQUIA** — o mesmo verbo, o outro sujeito.
///
/// ⚠️ **Ele existe porque o `Verb::Unmake` já aceita os dois** (a receita e uma cópia dela), e sem
/// este item essa metade era **inalcançável**: o único produtor era o cartão do navegador, cujo
/// sujeito é sempre a raiz da receita. A auditoria de 2026-08-30 apanhou-o — *um gate que chama a
/// função directamente fabrica um alcance que a UI não tem*.
///
/// ⚠️ ⛔ Id PRÓPRIO, e não o do cartão: a tabela do menu da Hierarquia despacha por `row` e a do
/// cartão por `cell`. Partilhar o id faria o guarda de um painel consumir o pedido do outro.
pub const CTX_MENU_HIER_REMOVE_FROM_LIBRARY: NodeId =
    hash_node_id("ctx_menu_hier_remove_from_library");

// ── ⭐⭐ O menu de uma LINHA DE CATÁLOGO (wave A3) ───────────────────────────────────────────────
//
// ⚠️ **Ids próprios, e não os do cartão:** os dois menus despacham por sujeitos diferentes (uma
// célula · uma linha), e partilhar um id faria o guarda de um consumir o pedido do outro.

/// Renomear o catálogo — abre o campo in-place sobre a linha.
pub const CTX_MENU_CATALOG_RENAME: NodeId = hash_node_id("ctx_menu_catalog_rename");
/// Apagar o catálogo **e os descendentes**. ⛔ Nunca apaga um asset — eles voltam a *Unassigned*, e
/// a voz di-lo.
pub const CTX_MENU_CATALOG_DELETE: NodeId = hash_node_id("ctx_menu_catalog_delete");
