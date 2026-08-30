//! **A BARRA DE MENUS** — os quatro títulos e as linhas que só ela traz.
//!
//! ⚠️ **A maioria das linhas destes menus NÃO tem id aqui, e isso é a decisão.** Um menu de
//! *Ficheiro* que inventasse um `MENUBAR_SAVE` teria um segundo id para o verbo que o
//! `CTX_MENU_SAVE` já é — e dois ids para um verbo são duas coisas a apodrecer em separado. A
//! barra **realoja** os verbos que já existem (D2, `docs/UI_New_and_Simple/00_DECISOES_DO_ENIO.md`):
//! a linha *Save* leva o id do `io_menu`, a linha *Vector* leva o `TOPBAR_VECTOR` que o pill
//! levava, e o despacho é o mesmo de sempre.
//!
//! ⇒ o que nasce aqui é só o que **não existia**: os quatro títulos, e as três linhas cujo verbo
//! nenhum botão do app alcançava (a régua, o tema como categoria, as preferências).

use super::{NodeId, hash_node_id};

/// Título *File*.
pub const MENUBAR_FILE: NodeId = hash_node_id("menubar_file");
/// Título *Edit*.
pub const MENUBAR_EDIT: NodeId = hash_node_id("menubar_edit");
/// Título *View*.
pub const MENUBAR_VIEW: NodeId = hash_node_id("menubar_view");
/// Título *Window*.
pub const MENUBAR_WINDOW: NodeId = hash_node_id("menubar_window");

/// *New Image…* — o modal que até aqui só a tecla `Cmd/Ctrl+N` alcançava.
///
/// ⚠️ O modal **já existia** (`WidgetStore::open_new_image_dialog`, `ContextMenuKind::NewImageDialog`)
/// e o `CTX_MENU_NEW_IMAGE_CREATE` é o CTA **dentro** dele — o que faltava era a porta.
pub const MENUBAR_FILE_NEW: NodeId = hash_node_id("menubar_file_new");

/// *Preferences…* — abre o [`crate::interaction::ContextMenuKind::SettingsMenu`], que era
/// alcançável só pelo pill da engrenagem.
pub const MENUBAR_EDIT_PREFERENCES: NodeId = hash_node_id("menubar_edit_preferences");

/// *Theme…* — abre o [`crate::interaction::ContextMenuKind::ThemeSelector`], que era alcançável
/// só pelo pill do tema.
pub const MENUBAR_VIEW_THEME: NodeId = hash_node_id("menubar_view_theme");

/// *Rulers* — o interruptor `HeroScreen::view.rulers_visible`.
///
/// ⚠️ Ele já tinha um consumidor e **um** sítio de onde se mexia: uma caixa dentro do painel do
/// vetor. Desde que as réguas valem em todos os modos (Enio, 2026-08-30) esse sítio deixou de
/// fazer sentido como o único — a régua é chrome de canvas, e o menu *View* é a casa dela.
pub const MENUBAR_VIEW_RULERS: NodeId = hash_node_id("menubar_view_rulers");
