//! **A BARRA DE MENUS** — os quatro títulos e as linhas que só ela traz.
//!
//! ⚠️ **A maioria das linhas destes menus NÃO tem id aqui, e isso é a decisão.** Um menu de
//! *Ficheiro* que inventasse um `MENUBAR_SAVE` teria um segundo id para o verbo que o
//! `CTX_MENU_SAVE` já é — e dois ids para um verbo são duas coisas a apodrecer em separado. A
//! barra **realoja** os verbos que já existem (D2, `docs/UI_New_and_Simple/00_DECISOES_DO_ENIO.md`):
//! a linha *Save* leva o id do `io_menu`, a linha *Motion Nodes* leva o `TOPBAR_MOTION` que o pill
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
/// F5: *Ver ▸ Modo foco* — a porta VISÍVEL do toque de 4 dedos e do `Ctrl+Space` (um verbo).
pub const MENUBAR_VIEW_FOCUS: NodeId = hash_node_id("menubar_view_focus");
/// F8: *Ver ▸ Enquadrar tudo* — a porta VISÍVEL do beliscão rápido (um verbo).
pub const MENUBAR_VIEW_FRAME_ALL: NodeId = hash_node_id("menubar_view_frame_all");
/// F8: *Ver ▸ Enquadrar seleção* — a porta visível do `F`/`Home` (um verbo).
pub const MENUBAR_VIEW_FRAME_SELECTED: NodeId = hash_node_id("menubar_view_frame_selected");
/// F5 (D10 decisão 2): *Ver ▸ Opções da ferramenta embaixo* — a fila ② passa para baixo da área (um
/// ALTERNADOR, por layout).
pub const MENUBAR_VIEW_TOOL_ROW_BOTTOM: NodeId = hash_node_id("menubar_view_tool_row_bottom");

/// ⭐⭐⭐ **A COLUNA ESQUERDA — fechar e reabrir**, o verbo que o dono mandou mudar de sítio.
///
/// > *«Vamos retirar a opção de colapsar arrastando. Deixa o colapsar apenas no menu da barra
/// > superior.»* — Enio, 2026-09-09
///
/// ⛔⛔ **Metade daquela ordem foi cumprida e a outra metade não, e o resultado foi a CAPACIDADE
/// desaparecer.** O gesto da borda saiu na w49; o item de menu nunca foi escrito. Medido em
/// 2026-09-19: o [`crate::screens::hero::dock_columns::close`] tinha **ZERO chamadores de
/// produto** — a lei da coluna estava viva, gateada e **inalcançável**, e o artista não tinha
/// maneira nenhuma de fechar uma coluna.
///
/// ⚠️ *Retirar um gesto deixa a lei dele viva e órfã, e nenhuma sonda deste repo pergunta se uma
/// PORTA tem chamador* — o roteador do `CLAUDE.md` escrevia isto por extenso e ninguém o lia como
/// uma feature em falta.
///
/// ⭐ **É um ALTERNADOR e não um verbo** (ao contrário do [`MENUBAR_VIEW_RESET_LAYOUT`] acima):
/// ele entra no `MODULE_TRUTHS`, acende quando a coluna está aberta, e o segundo clique desfaz o
/// primeiro — que é exactamente a involução que o `dock_columns` já garante.
pub const MENUBAR_VIEW_COLUMN_LEFT: NodeId = hash_node_id("menubar_view_column_left");

/// ⭐⭐⭐ **A COLUNA DIREITA — fechar e reabrir.** Irmã da [`MENUBAR_VIEW_COLUMN_LEFT`], e existe
/// pela mesma ordem do dono.
///
/// ⚠️ **Duas linhas e não uma:** o `dock_columns` guarda a memória **por lado**
/// (`dock_closed: [Option<ClosedColumn>; 2]`), e uma linha só que fechasse as duas perderia a
/// distinção que a involução precisa — reabrir devolveria a união dos dois fechos.
pub const MENUBAR_VIEW_COLUMN_RIGHT: NodeId = hash_node_id("menubar_view_column_right");

/// **Play in a Window**, na barra — escolhe [`crate::game_stage::GameStage::Window`] e joga.
pub const MENUBAR_RUN_WINDOW: NodeId = hash_node_id("menubar_run_window");
/// **Play Fullscreen**, na barra — escolhe [`crate::game_stage::GameStage::Fullscreen`] e joga.
pub const MENUBAR_RUN_FULLSCREEN: NodeId = hash_node_id("menubar_run_fullscreen");
/// **Play in the Editor**, na barra — escolhe [`crate::game_stage::GameStage::Editor`] e joga.
pub const MENUBAR_RUN_EDITOR: NodeId = hash_node_id("menubar_run_editor");

// ── A barra arrumada (line/BarraDeMenus, onda 1, dono 12/11) ──────────────────────────────────
// Só os ids que não existiam: o título Help, as linhas que ABREM um submenu (cascata) e o
// «Commands…». Os verbos dentro dos submenus continuam com o id de sempre.

/// Título *Help*.
pub const MENUBAR_HELP: NodeId = hash_node_id("menubar_help");
/// *File ▸ New* — abre o submenu dos «New».
pub const MENUBAR_FILE_NEW_MENU: NodeId = hash_node_id("menubar_file_new_menu");
/// *File ▸ Import* — abre o submenu dos jeitos de trazer coisa de fora.
pub const MENUBAR_FILE_IMPORT_MENU: NodeId = hash_node_id("menubar_file_import_menu");
/// *File ▸ Export* — abre o submenu das exportações (sempre as mesmas, cinzentas quando não servem).
pub const MENUBAR_FILE_EXPORT_MENU: NodeId = hash_node_id("menubar_file_export_menu");
/// *Window ▸ Audio*.
pub const MENUBAR_WINDOW_AUDIO_MENU: NodeId = hash_node_id("menubar_window_audio_menu");
/// *Window ▸ Pixel*.
pub const MENUBAR_WINDOW_PIXEL_MENU: NodeId = hash_node_id("menubar_window_pixel_menu");
/// *Window ▸ Plan* — os lugares do Kanban e do Planner.
pub const MENUBAR_WINDOW_PLAN_MENU: NodeId = hash_node_id("menubar_window_plan_menu");
/// *Help ▸ Commands…* — abre a paleta `Ctrl+K`.
pub const MENUBAR_HELP_COMMANDS: NodeId = hash_node_id("menubar_help_commands");
/// *Help ▸ Developer* — as ferramentas de quem faz o app.
pub const MENUBAR_HELP_DEVELOPER_MENU: NodeId = hash_node_id("menubar_help_developer_menu");

// ── A faixa que responde (onda 2, versão C do dono) ───────────────────────────────────────────
/// O botão «Menu» da faixa: abre os cinco menus em cascata (`Alt` também).
pub const MENUBAR_W: NodeId = hash_node_id("menubar_w");
/// O campo «Find anything» no centro da faixa: abre a busca (`Ctrl+K`).
pub const MENUBAR_FIND: NodeId = hash_node_id("menubar_find");
/// A seta ao lado do Play: escolhe onde o jogo se mostra.
pub const MENUBAR_PLAY_MENU: NodeId = hash_node_id("menubar_play_menu");

// ── A faixa única (onda 4): os botões da janela, desenhados pelo app ───────────────────────────
/// Minimizar a janela.
pub const MENUBAR_WIN_MINIMIZE: NodeId = hash_node_id("menubar_win_minimize");
/// Maximizar ou restaurar a janela (o duplo clique no fundo da faixa faz o mesmo).
pub const MENUBAR_WIN_MAXIMIZE: NodeId = hash_node_id("menubar_win_maximize");
/// Fechar a janela (o mesmo caminho do fechar do sistema).
pub const MENUBAR_WIN_CLOSE: NodeId = hash_node_id("menubar_win_close");

// ── A busca que responde (onda 3) ─────────────────────────────────────────────────────────────
/// O chip da faixa que ensina o atalho («Ctrl+E is Export Sheet…»).
pub const MENUBAR_TEACH: NodeId = hash_node_id("menubar_teach");
/// O «×» do chip: o artista já sabe; o chip deste comando nunca volta.
pub const MENUBAR_TEACH_CLOSE: NodeId = hash_node_id("menubar_teach_close");
/// *File ▸ Open Recent* — os projetos abertos por último.
pub const MENUBAR_FILE_RECENT_MENU: NodeId = hash_node_id("menubar_file_recent_menu");
/// A linha cinzenta do *Open Recent* vazio.
pub const MENUBAR_FILE_RECENT_NONE: NodeId = hash_node_id("menubar_file_recent_none");
/// *Open Recent ▸ Clear Recent*.
pub const MENUBAR_FILE_RECENT_CLEAR: NodeId = hash_node_id("menubar_file_recent_clear");
const RECENT_PROJECT_BASE: NodeId = hash_node_id("menubar_file_recent_project");

/// A linha `i` do *Open Recent* (o projeto `i`, o mais novo primeiro).
#[must_use]
pub const fn recent_project_row(i: usize) -> NodeId {
    NodeId(RECENT_PROJECT_BASE.0 ^ (i as u64 + 1))
}
