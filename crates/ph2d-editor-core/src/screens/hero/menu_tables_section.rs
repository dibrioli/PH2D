//! ⭐⭐ **O MENU DE BOTÃO DIREITO NO TÍTULO DE UMA SECÇÃO** — o tema da secção e o contorno de
//! marcador (2026-09-29).
//!
//! ⚠️ **Irmão do [`super::menu_tables`] pelo tecto de 700 linhas** que as duas tabelas deste menu
//! (uma por família de temas) empurravam. O gate `every_menu_row_reaches_a_handler` lê os três
//! ficheiros como UMA população — acrescentar um ficheiro de tabelas sem o pôr na lista dele faria
//! as linhas daqui desaparecerem da conferência em silêncio.

use crate::ids::{self, MenuRow, menu_row, menu_row_swatch};
use crate::widget::panel_chrome::HIGHLIGHTER_RGBA;

/// ⭐⭐ As linhas de `ContextMenuKind::SectionOutline { .. }` no REDESENHO — o TEMA da secção
/// primeiro (ordem do dono, 2026-09-29: *«com o botão direito do mouse sobre o título da seção
/// poderemos escolher o theme da seção entre os themes disponíveis para o app»*), depois o
/// contorno de marcador que este menu já oferecia.
///
/// ⭐ **As linhas de tema não levam cor na tabela** (2026-09-30): a amostra são quatro ladrilhos
/// TIRADOS do tema ao pintar ([`super::theme_menu::cores_do_tema`]) — a cor única escrita aqui era
/// uma cópia à mão que já tinha divergido uma vez (a base do Candy).
///
/// ⚠️ **Os temas oferecidos são os da FAMÍLIA do app** — a mesma lei do seletor do topo: misturar
/// um tema tingido com um plano poria o artista a escolher entre dois sistemas sem o saber.
pub(super) const SECTION_MENU_REDESIGN_ROWS: &[MenuRow] = &[
    menu_row(
        ids::CTX_MENU_SECTION_THEME_APP,
        "chrome.menu.section_theme_app",
    ),
    menu_row(ids::CTX_MENU_SECTION_THEME_DARK, "chrome.menu.dark"),
    menu_row(ids::CTX_MENU_SECTION_THEME_GRAY, "chrome.menu.gray"),
    menu_row(ids::CTX_MENU_SECTION_THEME_LIGHT, "chrome.menu.light"),
    menu_row(ids::CTX_MENU_SECTION_THEME_OLED, "chrome.menu.black_oled"),
    menu_row(
        ids::CTX_MENU_SECTION_THEME_PLUMBER_RED,
        "chrome.menu.plumber_red",
    ),
    menu_row(
        ids::CTX_MENU_SECTION_THEME_PLUMBER_GREEN,
        "chrome.menu.plumber_green",
    ),
    menu_row(ids::CTX_MENU_SECTION_THEME_SUNSET, "chrome.menu.sunset"),
    menu_row(ids::CTX_MENU_SECTION_THEME_CANDY, "chrome.menu.candy"),
    menu_row(
        ids::CTX_MENU_SECTION_THEMES_RESET,
        "chrome.menu.section_themes_reset",
    ),
    menu_row(
        ids::CTX_MENU_SECTION_THEMES_SAVE_CUSTOM,
        "chrome.menu.section_themes_save_custom",
    ),
    menu_row(
        ids::CTX_MENU_SECTION_THEMES_LOAD_CUSTOM,
        "chrome.menu.section_themes_load_custom",
    ),
    menu_row(ids::CTX_MENU_OUTLINE_NONE, "chrome.menu.no_outline"),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_0,
        "chrome.menu.yellow",
        HIGHLIGHTER_RGBA[0],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_1,
        "chrome.menu.pink",
        HIGHLIGHTER_RGBA[1],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_2,
        "chrome.menu.green",
        HIGHLIGHTER_RGBA[2],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_3,
        "chrome.menu.blue",
        HIGHLIGHTER_RGBA[3],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_4,
        "chrome.menu.orange",
        HIGHLIGHTER_RGBA[4],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_5,
        "chrome.menu.vivid_red",
        HIGHLIGHTER_RGBA[5],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_6,
        "chrome.menu.vivid_blue",
        HIGHLIGHTER_RGBA[6],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_7,
        "chrome.menu.vivid_green",
        HIGHLIGHTER_RGBA[7],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_8,
        "chrome.menu.vivid_yellow",
        HIGHLIGHTER_RGBA[8],
    ),
];

/// As mesmas linhas no tema CLÁSSICO — os quatro temas dele.
pub(super) const SECTION_MENU_CLASSIC_ROWS: &[MenuRow] = &[
    menu_row(
        ids::CTX_MENU_SECTION_THEME_APP,
        "chrome.menu.section_theme_app",
    ),
    menu_row(ids::CTX_MENU_SECTION_THEME_FORGE, "chrome.menu.forge_dark"),
    menu_row(
        ids::CTX_MENU_SECTION_THEME_PAINT,
        "chrome.menu.workshop_dark",
    ),
    menu_row(
        ids::CTX_MENU_SECTION_THEME_SUNSTONE,
        "chrome.menu.sunstone_light",
    ),
    menu_row(
        ids::CTX_MENU_SECTION_THEME_BLUEPRINT,
        "chrome.menu.blueprint_light",
    ),
    menu_row(
        ids::CTX_MENU_SECTION_THEMES_RESET,
        "chrome.menu.section_themes_reset",
    ),
    menu_row(
        ids::CTX_MENU_SECTION_THEMES_SAVE_CUSTOM,
        "chrome.menu.section_themes_save_custom",
    ),
    menu_row(
        ids::CTX_MENU_SECTION_THEMES_LOAD_CUSTOM,
        "chrome.menu.section_themes_load_custom",
    ),
    menu_row(ids::CTX_MENU_OUTLINE_NONE, "chrome.menu.no_outline"),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_0,
        "chrome.menu.yellow",
        HIGHLIGHTER_RGBA[0],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_1,
        "chrome.menu.pink",
        HIGHLIGHTER_RGBA[1],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_2,
        "chrome.menu.green",
        HIGHLIGHTER_RGBA[2],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_3,
        "chrome.menu.blue",
        HIGHLIGHTER_RGBA[3],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_4,
        "chrome.menu.orange",
        HIGHLIGHTER_RGBA[4],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_5,
        "chrome.menu.vivid_red",
        HIGHLIGHTER_RGBA[5],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_6,
        "chrome.menu.vivid_blue",
        HIGHLIGHTER_RGBA[6],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_7,
        "chrome.menu.vivid_green",
        HIGHLIGHTER_RGBA[7],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_8,
        "chrome.menu.vivid_yellow",
        HIGHLIGHTER_RGBA[8],
    ),
];
