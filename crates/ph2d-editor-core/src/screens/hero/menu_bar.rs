//! ⭐⭐ **A BARRA DE MENUS** — as TABELAS dos cinco menus (*File · Edit · View · Window · Help*) e
//! a faixa que os hospeda.
//!
//! Desde a onda 2 (versão C, dono 12/11 — revoga a D2 «menus à vista») os cinco moram dentro do
//! botão «Menu» ([`super::menu_faixa`]), em cascata de verdade ([`super::menu_cascade`]); este
//! ficheiro guarda as tabelas ([`MENUS`], [`CASCADES`], [`GROUP_BREAKS`], [`MODULE_TRUTHS`]) e o
//! pintor da faixa.
//!
//! # A lei: a barra REALOJA, não constrói
//!
//! ⚠️ **Quase toda linha destes menus leva um id que já existia** (o *Save* é o `CTX_MENU_SAVE`
//! do `chrome::io_menu`; o *Motion Nodes* é o `TOPBAR_MOTION`): existe um sítio canónico para cada
//! comando, e um comando com dois ids tem dois sítios a apodrecer em separado.
//!
//! ⚠️ **A faixa SUBTRAI altura, nunca flutua** (`ChromeBands::top_bar_h`): a área de desenho
//! começa por baixo dela.

use super::HeroLayout;
use crate::ids;
use crate::interaction::{ContextMenuKind, HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::rect_to_vello;
use crate::widget::ButtonState;
use ph2d_a11y::NodeId;
use ph2d_i18n::TextKey;
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Theme};
use ph2d_vector::VectorScene;

/// A altura da barra — **uma linha de menu**, que é o que um título é.
///
/// ⚠️ Deriva do mesmo token que a linha de um menu aberto (`ROW_H_PX`), e não de um número
/// escolhido: o título e as linhas que ele abre são a mesma família de alvo.
pub const MENU_BAR_H: f32 = ROW_H_PX;

/// ⭐ **A TABELA** — título, id, e o menu que ele abre. A fonte única desta barra.
pub const MENUS: [(NodeId, TextKey, ContextMenuKind); 5] = [
    (
        ids::MENUBAR_FILE,
        TextKey::new("chrome.menu_bar.file"),
        ContextMenuKind::MenuBarFile,
    ),
    (
        ids::MENUBAR_EDIT,
        TextKey::new("chrome.menu_bar.edit"),
        ContextMenuKind::MenuBarEdit,
    ),
    (
        ids::MENUBAR_VIEW,
        TextKey::new("chrome.menu_bar.view"),
        ContextMenuKind::MenuBarView,
    ),
    (
        ids::MENUBAR_WINDOW,
        TextKey::new("chrome.menu_bar.window"),
        ContextMenuKind::MenuBarWindow,
    ),
    (
        ids::MENUBAR_HELP,
        TextKey::new("chrome.menu_bar.help"),
        ContextMenuKind::MenuBarHelp,
    ),
];

/// ⭐ **AS CASCATAS** — a linha que abre um submenu e o submenu que ela abre (onda 1, dono 12/11).
///
/// Uma tabela, quatro consumidores: o despacho (`chrome::menu_bar` abre o submenu ancorado na
/// linha), o fecho antecipado ([`close_on_row_click`] deixa-as de fora), o pintor (o «›» só nelas)
/// e [`leaf_rows`] (a paleta e os censos atravessam a cascata até às folhas).
pub const CASCADES: [(NodeId, ContextMenuKind); 13] = [
    // Os cinco menus, linhas do botão «Menu» (onda 2).
    (ids::MENUBAR_FILE, ContextMenuKind::MenuBarFile),
    (ids::MENUBAR_EDIT, ContextMenuKind::MenuBarEdit),
    (ids::MENUBAR_VIEW, ContextMenuKind::MenuBarView),
    (ids::MENUBAR_WINDOW, ContextMenuKind::MenuBarWindow),
    (ids::MENUBAR_HELP, ContextMenuKind::MenuBarHelp),
    (ids::MENUBAR_FILE_NEW_MENU, ContextMenuKind::MenuBarFileNew),
    (
        ids::MENUBAR_FILE_IMPORT_MENU,
        ContextMenuKind::MenuBarFileImport,
    ),
    (
        ids::MENUBAR_FILE_EXPORT_MENU,
        ContextMenuKind::MenuBarFileExport,
    ),
    (
        ids::MENUBAR_FILE_RECENT_MENU,
        ContextMenuKind::MenuBarFileRecent,
    ),
    (
        ids::MENUBAR_WINDOW_AUDIO_MENU,
        ContextMenuKind::MenuBarWindowAudio,
    ),
    (
        ids::MENUBAR_WINDOW_PIXEL_MENU,
        ContextMenuKind::MenuBarWindowPixel,
    ),
    (
        ids::MENUBAR_WINDOW_PLAN_MENU,
        ContextMenuKind::MenuBarWindowPlan,
    ),
    (
        ids::MENUBAR_HELP_DEVELOPER_MENU,
        ContextMenuKind::MenuBarHelpDeveloper,
    ),
];

/// O submenu que esta linha abre, se é uma cascata da barra.
#[must_use]
pub fn cascade_of(id: NodeId) -> Option<ContextMenuKind> {
    CASCADES
        .iter()
        .find_map(|(row, kind)| (*row == id).then_some(*kind))
}

/// Este menu é da barra — o do botão «Menu», um dos cinco ou um submenu deles?
#[must_use]
pub fn is_bar_menu(kind: ContextMenuKind) -> bool {
    kind == ContextMenuKind::MenuBarRoot || CASCADES.iter().any(|(_, k)| *k == kind)
}

/// ⭐ **OS GRUPOS** — a linha ANTES da qual o menu da barra pinta um divisor. Um menu de 9 linhas
/// lê-se de relance em grupos; um de 18 sem separador, não (Apple HIG, Blender, Godot).
pub const GROUP_BREAKS: [NodeId; 8] = [
    // File: abrir · gravar · trazer/levar
    ids::CTX_MENU_SAVE,
    ids::MENUBAR_FILE_IMPORT_MENU,
    // Edit: desfazer · preferências
    ids::MENUBAR_EDIT_PREFERENCES,
    // View: painéis · régua e enquadrar · aparência e arrumação
    ids::MENUBAR_VIEW_RULERS,
    ids::MENUBAR_VIEW_THEME,
    // Window: abas · painéis de mundo · submenus
    ids::TOPBAR_PHYSICS,
    ids::MENUBAR_WINDOW_AUDIO_MENU,
    // Help: a paleta · quem faz o app
    ids::MENUBAR_HELP_DEVELOPER_MENU,
];

/// **Todas as folhas** de um menu da barra: as linhas estáticas dele e, por cada cascata, as do
/// submenu (recursivo), sem as próprias linhas-cascata. É o que a paleta e os censos de alcance
/// perguntam — «este verbo tem porta na barra?» — depois de as linhas terem ido para submenus.
#[must_use]
pub fn leaf_rows(kind: ContextMenuKind) -> Vec<crate::ids::MenuRow> {
    let mut out = Vec::new();
    for &row in super::menu_rows::menu_rows(kind) {
        match cascade_of(row.0) {
            Some(sub) => out.extend(leaf_rows(sub)),
            None => out.push(row),
        }
    }
    out
}

/// Os ids de linha que **esta** barra trouxe — os que nenhum outro botão do app alcançava.
///
/// ⚠️ Existe para o registo não ser uma lista escrita à mão ao lado da tabela de rows: quem
/// acrescentar um verbo novo a um destes menus acrescenta-o aqui, e o gate
/// `every_menu_bar_row_is_registered` reprova se esquecer.
pub const OWN_ROWS: [NodeId; 21] = [
    ids::MENUBAR_FILE_NEW,
    ids::MENUBAR_FILE_NEW_MENU,
    ids::MENUBAR_FILE_IMPORT_MENU,
    ids::MENUBAR_FILE_EXPORT_MENU,
    ids::MENUBAR_FILE_RECENT_MENU,
    ids::MENUBAR_WINDOW_AUDIO_MENU,
    ids::MENUBAR_WINDOW_PIXEL_MENU,
    ids::MENUBAR_WINDOW_PLAN_MENU,
    ids::MENUBAR_HELP_COMMANDS,
    ids::MENUBAR_HELP_DEVELOPER_MENU,
    ids::MENUBAR_FILE_SCENES,
    ids::MENUBAR_EDIT_PREFERENCES,
    ids::MENUBAR_VIEW_THEME,
    ids::MENUBAR_VIEW_RULERS,
    ids::MENUBAR_VIEW_RESET_LAYOUT,
    ids::MENUBAR_VIEW_COLUMN_LEFT,
    ids::MENUBAR_VIEW_COLUMN_RIGHT,
    ids::MENUBAR_VIEW_FOCUS,
    ids::MENUBAR_VIEW_TOOL_ROW_BOTTOM,
    ids::MENUBAR_VIEW_FRAME_ALL,
    ids::MENUBAR_VIEW_FRAME_SELECTED,
];

/// Regista os quatro títulos e as quatro linhas próprias. Chamado por
/// [`super::pre_populate::populate_shared`].
///
/// ⚠️ **Sem `InteractiveState` um item de menu é pintado e nasce morto**: ele não é focável, o
/// Down não arma o `active` e o Up nunca emite `Click`. É o mesmo defeito que matou o pill
/// `[SHEET]` e os quatro pills de vetor.
pub fn populate(store: &mut WidgetStore) {
    for (id, ..) in MENUS {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    for id in [
        ids::MENUBAR_W,
        ids::MENUBAR_FIND,
        ids::MENUBAR_TEACH,
        ids::MENUBAR_TEACH_CLOSE,
    ] {
        let state = ButtonState::Normal;
        store.register(id, InteractiveState::Button { state });
    }
    // ⛔ O fundo é `Plain`: ele não é um botão, é a superfície que ENGOLE o clique — ver
    // `ids::MENUBAR_BACKDROP`.
    store.register(ids::MENUBAR_BACKDROP, InteractiveState::Plain);
    for id in OWN_ROWS {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// ⛔⛔ **O ESTADO da linha *Rulers* vai para o STORE, para o menu o poder mostrar.**
///
/// O `context_menu_overlay::id_is_currently_selected` — quem pinta o marcador de *«é este o valor
/// actual»* — recebe o `WidgetStore`, não o [`super::HeroScreen`]. A régua vive em
/// `view.rulers_visible`, que ele não alcança; sem esta publicação a linha ficaria **sem marca
/// para sempre**, e o menu diria a mesma coisa com a régua ligada e desligada.
///
/// ⚠️ **É a lei que o próprio ficheiro do overlay documenta**, paga na unidade de ângulo:
/// *«fiar o clique não é fiar o ESTADO»*. As outras quinze linhas de estado desta barra (os treze
/// módulos e os dois painéis) já têm o `ButtonState` mantido por quem as despacha — só esta não
/// tinha ninguém a publicá-la.
pub fn publish_toggle_state(hero: &mut super::HeroScreen) {
    for (id, truth) in MODULE_TRUTHS {
        let on = truth.resolve(hero);
        if let Some(InteractiveState::Button { state }) = hero.store.get_mut(id) {
            *state = if on {
                ButtonState::Pressed
            } else {
                ButtonState::Normal
            };
        }
    }
}

/// ⭐⭐ **ONDE VIVE A VERDADE de cada linha de alternância** — a tabela que substituiu treze
/// respostas espalhadas, das quais **dez não existiam**.
///
/// ⛔⛔ **O defeito que ela cura tinha duas caras, e a segunda é a cara.** Medido em 2026-08-30:
/// ninguém escrevia o `ButtonState` dos pills de módulo — o laço de reconciliação da shell só
/// percorre os clusters `image_tools` e `vector_tools` do registry, e um pill de módulo não está em
/// cluster nenhum (`hash_node_id("topbar_vector")` ≠ `hash_node_id("vector")`).
///
/// 1. **a marca não aparecia** — o menu *Window* dizia o mesmo com o Vector aberto e fechado;
/// 2. ⚠️ **e o `chrome::vector_toggle` (que saiu com o pill, spec/06 F3) LIA esse estado para
///    escolher a direcção** (activar ou
///    cancelar). Preso em `Normal`, o segundo clique voltava a activar. *Um estado que ninguém
///    escreve e alguém lê não é uma marca em falta: é um `if` com um lado morto.*
///
/// ⭐ **Uma tabela, dois consumidores:** a marca do menu ([`publish_toggle_state`]) e a direcção do
/// toggle ([`module_is_on`]). Escrever a verdade duas vezes era como as duas se separaram.
#[derive(Copy, Clone, Debug)]
pub enum ModuleTruth {
    /// A visibilidade de um painel registado.
    Panel(&'static str),
    /// A ferramenta activa, pelo id do manifesto que a shell espelha em
    /// `ImageEditState::active_tool_id`.
    Tool(&'static str),
    /// O interruptor das réguas (`ViewState::rulers_visible`).
    Rulers,
    /// ⭐⭐ **Uma COLUNA lateral está aberta?** — a verdade das duas linhas que o menu *View*
    /// ganhou em 2026-09-19.
    ///
    /// ⚠️ **A pergunta é *«está ABERTA»* e não *«está fechada»***, para a marca ter o mesmo sentido
    /// das quinze linhas à volta: aceso = a coisa está lá. ⛔ Invertida, o menu diria *ligado* com a
    /// coluna escondida, que é pior do que não ter marca nenhuma.
    ///
    /// ⚠️ E a verdade é a MEMÓRIA do fecho (`hero.dock_closed`), não a largura: uma coluna
    /// arrastada até ao mínimo continua **aberta**, e desde 2026-09-09 a borda já nem a pode
    /// fechar.
    Column(crate::screens::layout::DockSide),
    /// A fila ② está embaixo da área (F5)? Aceso = embaixo.
    ToolRowBottom,
    /// O Play joga neste palco (a seta ao lado do Play).
    Stage(crate::game_stage::GameStage),
}

impl ModuleTruth {
    /// A linha está ligada?
    #[must_use]
    pub fn resolve(self, hero: &super::HeroScreen) -> bool {
        match self {
            Self::Panel(name) => hero.is_panel_visible(name),
            Self::Tool(id) => hero.image_edit.active_tool_id == Some(id),
            Self::Rulers => hero.view.rulers_visible,
            Self::Column(side) => !super::dock_columns::is_closed(hero, side),
            Self::ToolRowBottom => hero.store.tool_row_bottom(),
            Self::Stage(s) => hero.view.game_stage == s,
        }
    }
}

/// A tabela. ⚠️ **Toda linha de alternância dos menus tem de estar aqui**, e há censo a exigi-lo
/// (`every_toggle_row_of_the_bar_is_marked_by_its_own_state`).
pub const MODULE_TRUTHS: [(NodeId, ModuleTruth); 30] = [
    (
        ids::MENUBAR_RUN_EDITOR,
        ModuleTruth::Stage(crate::game_stage::GameStage::Editor),
    ),
    (
        ids::MENUBAR_RUN_WINDOW,
        ModuleTruth::Stage(crate::game_stage::GameStage::Window),
    ),
    (
        ids::MENUBAR_RUN_FULLSCREEN,
        ModuleTruth::Stage(crate::game_stage::GameStage::Fullscreen),
    ),
    (ids::TOPBAR_MOTION, ModuleTruth::Tool("motion")),
    (ids::TOPBAR_PHYSICS, ModuleTruth::Panel("physics")),
    // ⭐⭐⭐ **OS OSSOS** (ordem do dono, 2026-09-09). ⚠️ O literal é o `Panel::ID` da
    // `ph2d-panel-skeleton` — esta camada é chrome e não depende de painel nenhum, que é a mesma
    // cerca das linhas à volta.
    (ids::TOPBAR_SKELETON, ModuleTruth::Panel("skeleton")),
    // ⭐⭐⭐ **AS TAGS** (TOP-20 #9, W4). ⚠️ O literal é o `Panel::ID` da `ph2d-panel-tags` —
    // esta camada é chrome e não depende de painel nenhum, que é a mesma cerca das vizinhas.
    (ids::TOPBAR_TAGS, ModuleTruth::Panel("tags")),
    // O literal é o `Panel::ID` da `ph2d-panel-macros` (W5d).
    (ids::TOPBAR_MACROS, ModuleTruth::Panel("macros")),
    // O literal é o `Panel::ID` da `ph2d-panel-pixel-tools` (ADR-0196).
    (ids::TOPBAR_PIXEL, ModuleTruth::Panel(super::PANEL_PIXEL)),
    (
        ids::TOPBAR_PIXEL_PREVIEW,
        ModuleTruth::Panel(super::PANEL_PIXEL_PREVIEW),
    ),
    (
        ids::TOPBAR_PIXEL_SLICES,
        ModuleTruth::Panel(super::PANEL_PIXEL_SLICES),
    ),
    (
        ids::TOPBAR_PIXEL_TILES,
        ModuleTruth::Panel(super::PANEL_PIXEL_TILES),
    ),
    (
        ids::TOPBAR_PIXEL_HISTORY,
        ModuleTruth::Panel(super::PANEL_PIXEL_HISTORY),
    ),
    (
        ids::TOPBAR_PIXEL_KEYS,
        ModuleTruth::Panel(super::PANEL_PIXEL_KEYS),
    ),
    (
        ids::TOPBAR_PIXEL_PREFS,
        ModuleTruth::Panel(super::PANEL_PIXEL_PREFS),
    ),
    (
        ids::TOPBAR_PIXEL_LINT,
        ModuleTruth::Panel(super::PANEL_PIXEL_LINT),
    ),
    (ids::TOPBAR_AUDIO_MIXER, ModuleTruth::Panel("audio_mixer")),
    (ids::TOPBAR_AUDIO_EDITOR, ModuleTruth::Panel("audio_editor")),
    (ids::TOPBAR_TOKENS, ModuleTruth::Panel("tokens")),
    (ids::TOPBAR_AUTHORED, ModuleTruth::Panel("authored")),
    (
        ids::TOPBAR_WIDGET_GALLERY,
        ModuleTruth::Panel("widget_gallery"),
    ),
    (ids::TOPBAR_WIDGET_LAB, ModuleTruth::Panel("widget_lab")),
    (ids::TOPBAR_GRID_SETTINGS, ModuleTruth::Panel("grid_snap")),
    // ⭐⭐⭐ **A BIBLIOTECA** (report do Enio, 2026-09-05). ⚠️ O literal é o `PANEL_ID` do
    // `ph2d-panel-asset-browser` — esta camada é chrome e não depende de painel nenhum, que é a
    // mesma cerca das treze linhas acima.
    (
        ids::TOPBAR_RIGHT_ASSETS,
        ModuleTruth::Panel("asset_browser"),
    ),
    (ids::RAIL_SHOW_HIERARCHY, ModuleTruth::Panel("hierarchy")),
    (ids::RAIL_SHOW_INSPECTOR, ModuleTruth::Panel("inspector")),
    (ids::MENUBAR_VIEW_RULERS, ModuleTruth::Rulers),
    (
        ids::MENUBAR_VIEW_TOOL_ROW_BOTTOM,
        ModuleTruth::ToolRowBottom,
    ),
    // ⭐⭐⭐ **As duas colunas** (2026-09-19) — ver `ModuleTruth::Column`.
    (
        ids::MENUBAR_VIEW_COLUMN_LEFT,
        ModuleTruth::Column(crate::screens::layout::DockSide::Left),
    ),
    (
        ids::MENUBAR_VIEW_COLUMN_RIGHT,
        ModuleTruth::Column(crate::screens::layout::DockSide::Right),
    ),
];

/// ⭐ **Este módulo está LIGADO?** — a pergunta que um toggle faz para escolher a direcção.
///
/// ⛔ Os três activadores de ferramenta (`vector`, `motion`, `flip`) perguntavam ao
/// `store.button_state(…)`, que **ninguém escrevia** ⇒ o ramo *cancelar* nunca corria.
#[must_use]
pub fn module_is_on(hero: &super::HeroScreen, id: NodeId) -> Option<bool> {
    MODULE_TRUTHS
        .iter()
        .find(|(mid, _)| *mid == id)
        .map(|(_, truth)| truth.resolve(hero))
}

/// **Esta linha mostra o próprio estado pelo `ButtonState`?**
///
/// ⭐ A lista dos módulos é **derivada** da tabela do menu *Window* — um módulo novo entra sozinho.
/// As três nomeadas são as de fora dela.
#[must_use]
pub fn row_is_marked_by_button_state(id: NodeId) -> bool {
    MODULE_TRUTHS.iter().any(|(mid, _)| *mid == id)
}

/// Desenha a faixa (versão C): «Menu» · abas de documento · «Find anything» · o jogo. A geometria
/// é a de [`super::menu_faixa::faixa`].
#[allow(clippy::too_many_arguments)]
pub fn paint_menu_bar(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    motion: &crate::motion::UiMotion,
    docs: &crate::documents::Documents,
    run: crate::game_stage::GameRun,
) {
    let bar = layout.top_bar;
    if bar.w <= 0.0 || bar.h <= 0.0 {
        return;
    }
    let chao = super::transport_bar::chao_da_barra(theme);
    scene.fill_rect(rect_to_vello(bar), chao);
    // ⛔ PRIMEIRO o fundo: o `HitIndex` caminha de trás para a frente, e ele engole o clique que
    // não é de ninguém (senão a barra opaca deixava-o passar para o desenho).
    hit_index.register(ids::MENUBAR_BACKDROP, bar);
    let f = super::menu_faixa::faixa(bar, text_system);
    super::menu_faixa::paint(&f, scene, text_system, theme, hit_index, store, motion);
    super::transport_bar::paint(&f.jogo, run, scene, text_system, theme, hit_index, store);
    super::menu_teach::paint(&f, scene, text_system, theme, hit_index, store);
    super::document_tabs::paint(
        bar,
        f.docs.0,
        f.docs.1,
        docs,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
}

/// ⛔⛔ **O clique numa linha destes menus FECHA o menu antes de qualquer um agir** — e tem de
/// correr aqui, no topo do [`super::HeroScreen::apply_event`], porque o registo de painéis é
/// caminhado **antes** do `chrome::dispatch_all`: um `Click(TOPBAR_AUDIO_MIXER)` é consumido pelo
/// painel do mixer e o chrome nunca o vê. Um fecho escrito num handler de chrome ficaria morto
/// exactamente nas treze linhas do menu *Window*.
///
/// ⚠️ **As duas linhas que ABREM outro menu ficam de fora**, e não por precaução: elas fecham e
/// reabrem por si (`chrome::menu_bar`), e fechar aqui apagaria o `last_context_menu` de que o
/// `cascade_anchor` se serve.
pub fn close_on_row_click(hero: &mut super::HeroScreen, event: WidgetEvent) {
    let WidgetEvent::Click(id) = event else {
        return;
    };
    let chain = hero.store.menu_chain();
    if !chain.first().is_some_and(|r| is_bar_menu(r.kind)) {
        return;
    }
    if id == ids::MENUBAR_FILE_SCENES
        || super::menu_cascade::opens_submenu(id)
        // A linha cinzenta não faz nada — nem fechar o menu (o verbo recusa; ver `io_menu`).
        || hero.store.menu_row_is_unavailable(id)
    {
        return;
    }
    // A linha de QUALQUER nível aberto fecha a cascata inteira — e conta como escolha PELO MENU.
    if super::menu_cascade::level_of_row(&hero.store, id).is_some() {
        super::menu_teach::picked_from_menu(hero, id);
        hero.store.close_context_menu();
    }
}

#[cfg(test)]
#[path = "menu_bar_tests.rs"]
mod tests;
