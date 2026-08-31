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
/// Título *Run* — o transporte (tocar · pausar · rebobinar).
///
/// ⚠️ **Ele existe porque o `Reset` ficou SEM PORTA.** O `Espaço` alterna tocar/pausar e as
/// vírgulas andam quadro a quadro, mas **rebobinar** só se alcançava pelo pill da barra antiga.
/// Uma auditoria de 2026-08-30 contou-o entre os verbos que a retirada dos pills deixou órfãos.
pub const MENUBAR_RUN: NodeId = hash_node_id("menubar_run");

/// ⛔⛔ **O FUNDO da barra — o rectângulo que ENGOLE o clique.**
///
/// A barra pinta uma faixa opaca de ponta a ponta e só os títulos eram alvos: medido pela
/// auditoria de 2026-08-30, **86,9 %** da barra pintada deixava o ponteiro passar para o desenho
/// por baixo. Com o Painter em mãos, um pen-down entre dois títulos depositava tinta na arte
/// escondida sob a barra.
///
/// ⚠️ **A cura não é nova — é a que o trilho já tinha** (`ids::RAIL_BACKDROP`), acrescentada em
/// 2026-07-16 depois de um report do Enio com exactamente este sintoma. A barra nova nasceu sem
/// ela, e nenhum gate a pediu: o `the_chrome_swallows_the_click_it_was_given` mede que cada
/// consumidor de canvas **PERGUNTA** ao `pointer_over_chrome`, nunca que o chrome REGISTA um
/// rectângulo que responda que sim.
pub const MENUBAR_BACKDROP: NodeId = hash_node_id("menubar_backdrop");

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

/// *Scenes…* — abre a lista de cenas com busca (`ContextMenuKind::SceneList`).
///
/// ⚠️ **Ela ficou SEM PORTA na retirada dos pills**: o `TOPBAR_PROJECT` era o único a abri-la, e
/// com ele o campo de busca `CTX_SCENE_SEARCH` saiu do produto inteiro.
pub const MENUBAR_FILE_SCENES: NodeId = hash_node_id("menubar_file_scenes");

/// *Rulers* — o interruptor `HeroScreen::view.rulers_visible`.
///
/// ⚠️ Ele já tinha um consumidor e **um** sítio de onde se mexia: uma caixa dentro do painel do
/// vetor. Desde que as réguas valem em todos os modos (Enio, 2026-08-30) esse sítio deixou de
/// fazer sentido como o único — a régua é chrome de canvas, e o menu *View* é a casa dela.
pub const MENUBAR_VIEW_RULERS: NodeId = hash_node_id("menubar_view_rulers");

/// ⭐⭐ **REPOR A ARRUMAÇÃO** — devolve cada painel ao encaixe que ele declara, as colunas à largura
/// de fábrica, e apaga o ficheiro `~/.ph2d/layout.txt`.
///
/// > *«Precisamos da opção de resetar. Coloque nas opções de Theme.»* — Enio, 2026-08-30
///
/// ⚠️ **Vive no menu do LOOK e não num painel de preferências**, porque foi onde ele o pediu — e a
/// razão é boa: o que aquele menu já contém (tema, cantos, tamanho dos botões, espelhar a UI) é
/// exactamente a mesma categoria de facto — *como o app se parece e se arruma*.
///
/// ⛔ **É um VERBO, não um estado:** ele não tem marca de «ligado» e clicá-lo duas vezes é o mesmo
/// que uma. Por isso não entra no `MODULE_TRUTHS`, que é a tabela dos alternadores.
pub const MENUBAR_VIEW_RESET_LAYOUT: NodeId = hash_node_id("menubar_view_reset_layout");
