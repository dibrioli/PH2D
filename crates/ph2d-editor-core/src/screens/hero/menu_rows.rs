//! **A TABELA de rows de cada menu** — a porta única de *«que linhas este menu tem?»*.
//!
//! Ela vivia INLINE dentro do `paint_context_menu_overlay`, e enquanto o único consumidor era o
//! pintor isso estava certo. Deixou de estar quando a **paleta de comandos global** passou a
//! oferecer os mesmos verbos: uma segunda lista escrita à mão ali seria a tabela paralela que este
//! repo já viu apodrecer duas vezes no chrome da timeline (*o `Nearest` entrou na tabela e a row
//! nasceu morta*), e desta vez com o sintoma pior — a paleta a oferecer um comando que o menu já
//! não tem, ou a esquecer um que ele ganhou.
//!
//! ⚠️ **Isto é *pure code motion*:** o `match` é o mesmo, arm a arm, e o pintor passou a chamá-lo.
//!
//! # O que uma row É, e a fronteira que a paleta respeita
//!
//! Uma row é `(id, rótulo, swatch opcional)`. O **id é o verbo**: apertar a row é levantar
//! `WidgetEvent::Click(id)`, e o `chrome::dispatch_all` resolve — sem rectângulo nenhum. É por
//! isso que a paleta consegue oferecer estas rows e **não** consegue oferecer os *pills* que as
//! abrem (um pill ancora um menu a um rectângulo, e um pick não tem rectângulo).
//!
//! ⚠️ Mas nem toda row é servível assim, e a lista está em
//! [`ContextMenuKind::rows_are_context_free_leaves`].

use super::menu_tables;
use crate::ids;
use crate::interaction::ContextMenuKind;
use ph2d_a11y::NodeId;
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;

/// As rows deste menu, na ordem em que ele as pinta.
///
/// Um `&[]` significa *este menu desenha o próprio corpo* (a lista de cenas, o diálogo de rename, o
/// de imagem nova) — não *«menu vazio»*.
#[must_use]
pub fn menu_rows(kind: ContextMenuKind) -> &'static [crate::ids::MenuRow] {
    match kind {
        ContextMenuKind::CreateNote { .. } => menu_tables::CREATE_NOTE_ROWS,
        ContextMenuKind::SectionOutline { .. } => menu_tables::SECTION_OUTLINE_ROWS,
        ContextMenuKind::NoteBackground { .. } => menu_tables::NOTE_BACKGROUND_ROWS,
        ContextMenuKind::ThemeSelector if crate::paint::ui_is_redesign() => {
            menu_tables::THEME_SELECTOR_REDESIGN_ROWS
        }
        ContextMenuKind::ThemeSelector => menu_tables::THEME_SELECTOR_ROWS,
        ContextMenuKind::MenuBarFile => menu_tables::MENU_BAR_FILE_ROWS,
        ContextMenuKind::MenuBarEdit => menu_tables::MENU_BAR_EDIT_ROWS,
        ContextMenuKind::MenuBarView => menu_tables::MENU_BAR_VIEW_ROWS,
        ContextMenuKind::MenuBarWindow => menu_tables::MENU_BAR_WINDOW_ROWS,
        ContextMenuKind::MenuBarRun => menu_tables::MENU_BAR_RUN_ROWS,
        ContextMenuKind::SaveMenu => menu_tables::SAVE_MENU_ROWS,
        ContextMenuKind::OpenMenu => menu_tables::OPEN_MENU_ROWS,
        ContextMenuKind::SettingsMenu => menu_tables::SETTINGS_MENU_ROWS,
        ContextMenuKind::SettingsPpmSubmenu => menu_tables::SETTINGS_PPM_SUBMENU_ROWS,
        ContextMenuKind::SettingsUnitSubmenu => menu_tables::SETTINGS_UNIT_SUBMENU_ROWS,
        ContextMenuKind::SettingsAngleSubmenu => menu_tables::SETTINGS_ANGLE_SUBMENU_ROWS,
        ContextMenuKind::SettingsFilterSubmenu => menu_tables::SETTINGS_FILTER_SUBMENU_ROWS,
        ContextMenuKind::SettingsDisplaySubmenu => menu_tables::SETTINGS_DISPLAY_SUBMENU_ROWS,
        ContextMenuKind::SettingsTextSubmenu => menu_tables::SETTINGS_TEXT_SUBMENU_ROWS,
        ContextMenuKind::SettingsMotionSubmenu => menu_tables::SETTINGS_MOTION_SUBMENU_ROWS,
        // The SceneList kind is rendered by its dedicated branch
        // below — `items` stays empty so the simple-row loop is
        // skipped.
        ContextMenuKind::SceneList => &[],
        // ⭐ **Desenha o próprio corpo** (ver o doc desta função): os chips que não couberam na
        // fila, com os MESMOS ids — ver `context_menu_overlay::paint_tool_bar_overflow`.
        ContextMenuKind::ToolBarOverflow => &[],
        // ⭐⭐⭐ **As rows são DINÂMICAS** — quem as publica é o módulo que tem o canvas
        // (`WidgetStore::area_menus`), e o pintor acrescenta-as a estas. ⛔ Uma tabela estática
        // aqui teria de conhecer os ids de TODO módulo do app, que é o acoplamento que a **D2**
        // existe para não ter.
        //
        // ⚠️ E o `&[]` deixou de ser um caso especial: o pintor soma `estáticas + contribuídas`
        // para **todo** menu, e um pulldown de área é simplesmente aquele cuja metade estática é
        // vazia. É o mesmo mecanismo que põe *Export Draft* no `MenuBarFile` abaixo.
        ContextMenuKind::AreaCommands { .. } => &[],
        // The palette-rename modal paints a TextInput + Rename button in its own branch below.
        ContextMenuKind::RenamePaletteDialog => &[],
        // The New-image modal paints its size/bg radios + Create in its own branch below.
        ContextMenuKind::NewImageDialog => &[],
        // O modal de resolução da folha desenha o próprio corpo, como os irmãos acima.
        ContextMenuKind::SheetSizeDialog => &[],
        ContextMenuKind::HierarchyRow { .. } => menu_tables::HIERARCHY_ROW_ROWS,
        ContextMenuKind::CatalogRow { .. } => menu_tables::CATALOG_ROW_ROWS,
        ContextMenuKind::AssetCard { .. } => menu_tables::ASSET_CARD_ROWS,
        ContextMenuKind::FalloffPointHandle => menu_tables::FALLOFF_POINT_HANDLE_ROWS,
        ContextMenuKind::CurvePointHandle => menu_tables::CURVE_POINT_HANDLE_ROWS,
        ContextMenuKind::MotionPathAnchor { .. } => menu_tables::MOTION_PATH_ANCHOR_ROWS,
        // Timeline key: the interpolation leaving it, and the family submenu the
        // three cascade rows open. Both tables live in `ids` so the overlay, the
        // populate pass and the shell's resolver can never drift apart.
        // The SCOPE picks the table: a key gets the interpolation presets, a strip's fade
        // gets the easings alone (`TimelineInterpScope::menu_table` — the same door the
        // chrome handler asks, so painted rows and live rows cannot drift).
        ContextMenuKind::TimelineSegment { scope } => scope.menu_table(),
        ContextMenuKind::TimelineSegmentEase { .. } => &ids::TIMELINE_EASE_MENU,
        // Timeline track row (label column): whole-track actions.
        ContextMenuKind::TimelineTrack { .. } => &ids::TIMELINE_TRACK_MENU,
        ContextMenuKind::TimelineTrackAxis { .. } => &ids::TIMELINE_AXIS_TRACK_MENU,
        ContextMenuKind::TimelineTrackPath { .. } => &ids::TIMELINE_PATH_TRACK_MENU,
        ContextMenuKind::TimelineTrackTimeRemap { .. } => &ids::TIMELINE_TIMEREMAP_TRACK_MENU,
        // The four-mode extrapolation submenu (plan §6), opened by a track menu's
        // Pre/Post cascade row.
        ContextMenuKind::TimelineExtrap { .. } => &ids::TIMELINE_EXTRAP_MENU,
        // Timeline clip strip (a stack lane): what a pointer cannot say.
        ContextMenuKind::TimelineStrip { .. } => &ids::TIMELINE_STRIP_MENU,
        // Timeline stack lane (its label): how it blends, and whether it stays.
        ContextMenuKind::TimelineLane { .. } => &ids::TIMELINE_LANE_MENU,
        // Timeline marker pennant: its whole edit surface (ADR-0143).
        ContextMenuKind::TimelineMarker { .. } => &ids::TIMELINE_MARKER_MENU,
    }
}

/// **Os menus da barra de topo cujas rows são verbos SERVÍVEIS por id.**
///
/// A paleta de comandos global projecta as rows destes — e de mais nenhum. As três exclusões são
/// por MECANISMO, não por gosto, e cada uma tem um modo de falha diferente:
///
/// - **Menus parameterizados pelo alvo do clique** (`CreateNote { panel }`, `SectionOutline
///   { section }`, `NoteBackground`, `HierarchyRow`, os da timeline): o verbo precisa de um sujeito
///   que o CLIQUE forneceu. Servido de uma paleta, ele agiria sobre o que quer que o
///   `last_context_menu` ainda tivesse — um comando que faz alguma coisa, à coisa errada.
/// - **[`ContextMenuKind::SettingsMenu`]**: cada row dele ABRE um submenu, ancorado ao rectângulo
///   da row. Servida de uma paleta, ela abriria um menu numa posição obsoleta. As rows dos
///   SUBMENUS entram, porque essas são folhas.
/// - **Os que desenham o próprio corpo** (`SceneList`, `RenamePaletteDialog`, `NewImageDialog`):
///   [`menu_rows`] devolve `&[]` para eles, então já não contribuem nada.
///
/// ⚠️ A prova de que os que ENTRAM são context-free não é opinião: os `chrome/*.rs` que os tratam
/// só tocam o contexto para `close_context_menu()`, que é um no-op sem menu aberto. Há gate.
/// ⭐⭐⭐ **Os menus do chrome LEGADO e o pill que os abria** — a tabela do que a barra de menus
/// teve de **realojar**.
///
/// Ela existe por um defeito medido (2026-09-04, report do Enio: *«funções criadas por outros
/// módulos não aparecem na UI»*, exemplo *Export SVG…*). A retirada dos pills (2026-08-30) tirou o
/// único botão que abria estes cinco menus; a barra prometeu levar as rows deles para *File* /
/// *Edit* / *View*, e a prova dessa promessa era **prosa numa lista de excepções** do gate
/// `every_topbar_verb_has_a_door_that_is_not_the_legacy_key` — *«as duas linhas do `SaveMenu` estão
/// no menu File»*. ⚠️ **Uma isenção que CONTA linhas envelhece no dia em que alguém acrescenta
/// uma:** a `line/Vector` juntou o *Export SVG…* ao `SaveMenu` em 2026-09-02, a frase passou a
/// descrever três linhas dizendo duas, e o verbo ficou sem porta **sem acordar gate nenhum** — o
/// censo daquele gate é sobre os **ids de pill declarados**, não sobre as **rows** que eles abriam.
///
/// ⇒ a promessa passa a ser DADO, e quem a mede é
/// `the_bar_relocated_every_row_of_the_menus_it_replaced`.
///
/// ⚠️ **Ela tem dois consumidores, de propósito:** o despacho
/// (`interaction::dispatch::pointer_down_menus::menu_opened_by`, que continua a abrir estes menus
/// quando a `F9` devolve o chrome legado) e o gate. *Uma lei escrita em dois sítios ainda não é uma
/// lei — só uma porta é.*
pub const LEGACY_PILL_MENUS: &[(NodeId, ContextMenuKind)] = &[
    (ids::TOPBAR_THEME, ContextMenuKind::ThemeSelector),
    (ids::TOPBAR_SAVE, ContextMenuKind::SaveMenu),
    (ids::TOPBAR_OPEN, ContextMenuKind::OpenMenu),
    (ids::TOPBAR_SETTINGS, ContextMenuKind::SettingsMenu),
    (ids::TOPBAR_PROJECT, ContextMenuKind::SceneList),
];

/// ⭐⭐ **Os BOTÕES directos da barra legada que faziam alguma coisa** — a outra metade do censo de
/// alcance. O `LEGACY_PILL_MENUS` responde por *menus* que a barra substituiu; um chip que
/// despachava sozinho (sem menu) não estava em lista nenhuma, e foi assim que o navegador de
/// **Assets** ficou sem porta no redesenho (Enio, 2026-09-05: *«não há meio de abrir assets»*) —
/// a porta dele era só o chip `TOPBAR_RIGHT_ASSETS` do grupo direito, que o redesenho não pinta.
///
/// ⚠️ `TOPBAR_RIGHT_LAYERS` e `TOPBAR_RIGHT_SCRIPT` **não** estão aqui de propósito: nenhum
/// `apply_event` do app os trata (medido em 2026-09-05 — eram chips MUDOS também no clássico), e
/// uma linha de menu para um id sem handler é uma linha morta, que o
/// `every_menu_row_reaches_a_handler` recusa. Quando um deles ganhar handler, entra aqui.
pub const LEGACY_PILL_BUTTONS: &[(NodeId, TextKey)] =
    &[(ids::TOPBAR_RIGHT_ASSETS, TextKey::new("chrome.menu.assets"))];

pub const TOPBAR_LEAF_MENUS: &[ContextMenuKind] = &[
    ContextMenuKind::SaveMenu,
    ContextMenuKind::OpenMenu,
    ContextMenuKind::ThemeSelector,
    ContextMenuKind::SettingsPpmSubmenu,
    ContextMenuKind::SettingsUnitSubmenu,
    ContextMenuKind::SettingsAngleSubmenu,
    ContextMenuKind::SettingsFilterSubmenu,
    ContextMenuKind::SettingsDisplaySubmenu,
    ContextMenuKind::SettingsTextSubmenu,
    ContextMenuKind::SettingsMotionSubmenu,
];

/// O nome humano do menu — o título do grupo na paleta.
///
/// ⚠️ Escrito à mão, e é a única coisa desta wave que é: os `ContextMenuKind` não carregam rótulo
/// (o pill que os abre é que tem um ícone), e derivá-lo do `Debug` daria *«SettingsPpmSubmenu»*.
#[must_use]
pub fn menu_title(kind: ContextMenuKind) -> Option<&'static str> {
    Some(match kind {
        ContextMenuKind::SaveMenu => tr("chrome.menu.file"),
        ContextMenuKind::OpenMenu => tr("chrome.menu.open"),
        ContextMenuKind::ThemeSelector => tr("chrome.menu.look"),
        ContextMenuKind::SettingsPpmSubmenu => tr("chrome.menu.pixels_per_meter"),
        ContextMenuKind::SettingsUnitSubmenu => tr("chrome.menu.display_unit"),
        ContextMenuKind::SettingsAngleSubmenu => tr("chrome.menu.angle_unit"),
        ContextMenuKind::SettingsFilterSubmenu => tr("chrome.menu.image_filter"),
        ContextMenuKind::SettingsDisplaySubmenu => tr("chrome.menu.display"),
        ContextMenuKind::SettingsTextSubmenu => tr("chrome.menu.text_rendering"),
        ContextMenuKind::SettingsMotionSubmenu => tr("chrome.menu.motion"),
        _ => return None,
    })
}
