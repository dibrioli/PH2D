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
/// ⚠️ **Os temas oferecidos são os da FAMÍLIA do app** — a mesma lei do seletor do topo: misturar
/// um tema tingido com um plano poria o artista a escolher entre dois sistemas sem o saber.
pub(super) const SECTION_MENU_REDESIGN_ROWS: &[MenuRow] = &[
    menu_row(
        ids::CTX_MENU_SECTION_THEME_APP,
        "chrome.menu.section_theme_app",
    ),
    menu_row_swatch(
        ids::CTX_MENU_SECTION_THEME_DARK,
        "chrome.menu.dark",
        [0x56, 0x9e, 0xff, 0xFF],
    ),
    menu_row_swatch(
        ids::CTX_MENU_SECTION_THEME_GRAY,
        "chrome.menu.gray",
        [0x70, 0xba, 0xfa, 0xFF],
    ),
    menu_row_swatch(
        ids::CTX_MENU_SECTION_THEME_LIGHT,
        "chrome.menu.light",
        [0x2e, 0x80, 0xff, 0xFF],
    ),
    menu_row_swatch(
        ids::CTX_MENU_SECTION_THEME_OLED,
        "chrome.menu.black_oled",
        [0x73, 0xbf, 0xff, 0xFF],
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
];

/// As mesmas linhas no tema CLÁSSICO — os quatro temas dele.
pub(super) const SECTION_MENU_CLASSIC_ROWS: &[MenuRow] = &[
    menu_row(
        ids::CTX_MENU_SECTION_THEME_APP,
        "chrome.menu.section_theme_app",
    ),
    menu_row_swatch(
        ids::CTX_MENU_SECTION_THEME_FORGE,
        "chrome.menu.forge_dark",
        [0xc8, 0x4b, 0xa0, 0xFF],
    ),
    menu_row_swatch(
        ids::CTX_MENU_SECTION_THEME_PAINT,
        "chrome.menu.workshop_dark",
        [0x4b, 0xa0, 0xc8, 0xFF],
    ),
    menu_row_swatch(
        ids::CTX_MENU_SECTION_THEME_SUNSTONE,
        "chrome.menu.sunstone_light",
        [0xf0, 0xc0, 0x4f, 0xFF],
    ),
    menu_row_swatch(
        ids::CTX_MENU_SECTION_THEME_BLUEPRINT,
        "chrome.menu.blueprint_light",
        [0x6c, 0x8e, 0xc8, 0xFF],
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
];
