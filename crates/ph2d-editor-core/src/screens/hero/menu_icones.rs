//! ⭐ **O ÍCONE de cada linha dos menus da barra** (onda 8, dono 13/11: «nossos menus sem ícones e sem
//! divisórias ainda não estão tão bonitos»; maquete C `.c15it`, coluna de 16 à esquerda).
//!
//! Uma tabela `id → ícone`, a fonte única: o pintor lê-a, e o portão
//! `toda_folha_dos_menus_da_barra_tem_icone` exige uma entrada para cada linha estática da barra (as
//! listas de VALORES — Pixels per Meter ▸ 16, os temas — mostram a escolha, não um ícone). As linhas que
//! uma família soma trazem o ícone dela (`DocCommand::icon`, `ToolRailEntry::Icon`).

use crate::icons::IconId;
use crate::ids;
use ph2d_a11y::NodeId;

/// `(linha, ícone)`, pela ordem dos menus.
pub const ICONES: &[(NodeId, IconId)] = &[
    // A raiz (o botão «Menu»).
    (ids::MENUBAR_FILE, IconId::File),
    (ids::MENUBAR_EDIT, IconId::Modify),
    (ids::MENUBAR_VIEW, IconId::Eye),
    (ids::MENUBAR_WINDOW, IconId::LayoutGrid),
    (ids::MENUBAR_HELP, IconId::Help),
    // File
    (ids::MENUBAR_FILE_NEW_MENU, IconId::Plus),
    (ids::CTX_MENU_OPEN_PROJECT, IconId::Folder),
    (ids::MENUBAR_FILE_RECENT_MENU, IconId::History),
    (ids::MENUBAR_FILE_SCENES, IconId::Scene),
    (ids::CTX_MENU_SAVE, IconId::Save),
    (ids::CTX_MENU_SAVE_AS, IconId::Duplicate),
    (ids::MENUBAR_FILE_IMPORT_MENU, IconId::Open),
    (ids::MENUBAR_FILE_EXPORT_MENU, IconId::Export),
    // File ▸ New · Import · Export
    (ids::MENUBAR_FILE_NEW, IconId::Image),
    (ids::CTX_MENU_IMPORT, IconId::Image),
    (ids::CTX_MENU_PIXEL_IMPORT_SHEET, IconId::LayoutGrid),
    (ids::CTX_MENU_PIXEL_OPEN, IconId::PixelSprite),
    (ids::CTX_MENU_EXPORT_SVG, IconId::Vector),
    (ids::CTX_MENU_PIXEL_EXPORT_AS, IconId::Export),
    (ids::CTX_MENU_PIXEL_EXPORT_SHEET, IconId::LayoutGrid),
    (ids::CTX_MENU_PIXEL_EXPORT_TILESET, IconId::Grid),
    (ids::CTX_MENU_PIXEL_REPEAT_EXPORT, IconId::RotateCw),
    // Edit
    (ids::TOOL_UNDO, IconId::Undo),
    (ids::TOOL_REDO, IconId::Redo),
    (ids::MENUBAR_EDIT_PREFERENCES, IconId::Settings),
    // View
    (ids::RAIL_SHOW_HIERARCHY, IconId::Hierarchy),
    (ids::RAIL_SHOW_INSPECTOR, IconId::Inspector),
    (ids::MENUBAR_VIEW_RULERS, IconId::CadDistance),
    (ids::MENUBAR_VIEW_FOCUS, IconId::Zen),
    (ids::MENUBAR_VIEW_FRAME_ALL, IconId::FitView),
    (ids::MENUBAR_VIEW_FRAME_SELECTED, IconId::MarqueeBox),
    (ids::MENUBAR_VIEW_TOOL_ROW_BOTTOM, IconId::SplitHorizontal),
    (ids::MENUBAR_VIEW_THEME, IconId::Palette),
    (ids::MENUBAR_VIEW_RESET_LAYOUT, IconId::Reset),
    (ids::MENUBAR_VIEW_COLUMN_LEFT, IconId::AlignLeft),
    (ids::MENUBAR_VIEW_COLUMN_RIGHT, IconId::AlignRight),
    // Window
    (ids::CTX_MENU_SETTINGS_INPUT_MAP, IconId::Kbd),
    (ids::TOPBAR_RIGHT_ASSETS, IconId::Asset),
    (ids::TOPBAR_PHYSICS, IconId::Physics),
    (ids::TOPBAR_TAGS, IconId::Tag),
    (ids::TOPBAR_SKELETON, IconId::Bone),
    (ids::TOPBAR_MACROS, IconId::Macro),
    (ids::TOPBAR_MOTION, IconId::MotionNodes),
    (ids::TOPBAR_GRID_SETTINGS, IconId::GridSettings),
    (ids::MENUBAR_WINDOW_AUDIO_MENU, IconId::Audio),
    (ids::MENUBAR_WINDOW_PIXEL_MENU, IconId::PixelSprite),
    (ids::MENUBAR_WINDOW_PLAN_MENU, IconId::Calendar),
    // Window ▸ Audio · Pixel
    (ids::TOPBAR_AUDIO_MIXER, IconId::Mixer),
    (ids::TOPBAR_AUDIO_EDITOR, IconId::Volume2),
    (ids::TOPBAR_PIXEL, IconId::PixelSprite),
    (ids::TOPBAR_PIXEL_PREVIEW, IconId::Eye),
    (ids::TOPBAR_PIXEL_SLICES, IconId::Knife),
    (ids::TOPBAR_PIXEL_TILES, IconId::Grid),
    (ids::TOPBAR_PIXEL_HISTORY, IconId::History),
    (ids::TOPBAR_PIXEL_KEYS, IconId::Kbd),
    (ids::TOPBAR_PIXEL_PREFS, IconId::Settings),
    (ids::TOPBAR_PIXEL_LINT, IconId::Warning),
    // Help ▸ Developer
    (ids::MENUBAR_HELP_COMMANDS, IconId::Command),
    (ids::MENUBAR_HELP_DEVELOPER_MENU, IconId::Bug),
    (ids::TOPBAR_TOKENS, IconId::Palette),
    (ids::TOPBAR_AUTHORED, IconId::Build),
    (ids::TOPBAR_WIDGET_GALLERY, IconId::LayoutGrid),
    (ids::TOPBAR_WIDGET_LAB, IconId::Probe),
    // A seta do Play.
    (ids::MENUBAR_RUN_EDITOR, IconId::PlayEditor),
    (ids::MENUBAR_RUN_WINDOW, IconId::PlayWindow),
    (ids::MENUBAR_RUN_FULLSCREEN, IconId::PlayFullscreen),
    // Edit ▸ Preferences (as categorias; os VALORES de cada uma mostram a escolha).
    (ids::CTX_MENU_SETTINGS_PPM, IconId::CadDistance),
    (ids::CTX_MENU_SETTINGS_UNIT, IconId::Line),
    (ids::CTX_MENU_SETTINGS_ANGLE, IconId::CadAngle),
    (ids::CTX_MENU_SETTINGS_FILTER, IconId::Image),
    (ids::CTX_MENU_SETTINGS_DISPLAY, IconId::Fps),
    (ids::CTX_MENU_SETTINGS_TEXT, IconId::Text),
    (ids::CTX_MENU_SETTINGS_FONT, IconId::LetterI),
    (ids::CTX_MENU_SETTINGS_WEIGHT, IconId::CardWeight),
    (ids::CTX_MENU_SETTINGS_SIZE, IconId::LetterH),
    (ids::CTX_MENU_SETTINGS_SCALE, IconId::Scale),
    (ids::CTX_MENU_SETTINGS_MOTION, IconId::Film),
    (ids::CTX_MENU_SETTINGS_LIGHT, IconId::Light),
    (ids::CTX_MENU_SETTINGS_RECOVERY, IconId::History),
];

/// O ícone da linha `id` num menu da barra, se a tabela o tem.
#[must_use]
pub fn icone(id: NodeId) -> Option<IconId> {
    ICONES
        .iter()
        .find_map(|(row, icon)| (*row == id).then_some(*icon))
}
