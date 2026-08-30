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
