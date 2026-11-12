//! **AS TABELAS DO MENU DA ENGRENAGEM (Settings)** — a lista de categorias e um submenu por
//! categoria.
//!
//! ⚠️ **Irmão do [`super::menu_tables`] pelo tecto de 700 linhas** (fecho P12, 08/11). O gate
//! `every_menu_row_reaches_a_handler` lê os ficheiros de tabelas como UMA população — este está na
//! lista dele.

use crate::ids::{self, MenuRow, menu_row};

// Settings cluster (gear) — TOP-LEVEL categories; each gets a `ChevronRight` from the row loop.
/// As linhas de `ContextMenuKind::SettingsMenu`.
pub(super) const SETTINGS_MENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_SETTINGS_PPM, "chrome.menu.pixels_per_meter"),
    menu_row(ids::CTX_MENU_SETTINGS_UNIT, "chrome.menu.display_unit"),
    menu_row(ids::CTX_MENU_SETTINGS_ANGLE, "chrome.menu.angle_unit"),
    menu_row(ids::CTX_MENU_SETTINGS_FILTER, "chrome.menu.image_filter"),
    menu_row(ids::CTX_MENU_SETTINGS_DISPLAY, "chrome.menu.display"),
    menu_row(ids::CTX_MENU_SETTINGS_TEXT, "chrome.menu.text_rendering"),
    // ⭐ Os três eixos do texto, ao lado da nitidez (o Blender e a Godot separam os quatro).
    menu_row(ids::CTX_MENU_SETTINGS_FONT, "chrome.menu.interface_font"),
    menu_row(ids::CTX_MENU_SETTINGS_WEIGHT, "chrome.menu.font_weight"),
    menu_row(ids::CTX_MENU_SETTINGS_SIZE, "chrome.menu.font_size"),
    // ⭐ E a interface INTEIRA (2026-10-02), ao lado do tamanho do texto.
    menu_row(ids::CTX_MENU_SETTINGS_SCALE, "chrome.menu.interface_scale"),
    menu_row(ids::CTX_MENU_SETTINGS_MOTION, "chrome.menu.motion"),
    menu_row(ids::CTX_MENU_SETTINGS_LIGHT, "chrome.menu.light_2d"),
    menu_row(ids::CTX_MENU_SETTINGS_RECOVERY, "chrome.menu.recovery"),
    // O Input Map saiu daqui para Window (dono 12/11): era a única linha sem submenu e levava «›».
];

// Pixels-per-meter submenu — 5 presets (retro 16 · Unity 32 · Godot 100 · HD 256 · 4K 1024).
/// As linhas de `ContextMenuKind::SettingsPpmSubmenu`.
pub(super) const SETTINGS_PPM_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_PPM_16, "chrome.menu.n16_retro_tile"),
    menu_row(ids::CTX_MENU_PPM_32, "chrome.menu.n32_unity_2d"),
    menu_row(ids::CTX_MENU_PPM_100, "chrome.menu.n100_godot"),
    menu_row(ids::CTX_MENU_PPM_256, "chrome.menu.n256_hd_2d"),
    menu_row(ids::CTX_MENU_PPM_1024, "chrome.menu.n1024_4k_ref"),
];

/// As linhas de `ContextMenuKind::SettingsUnitSubmenu`.
pub(super) const SETTINGS_UNIT_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_UNIT_METERS, "chrome.menu.meters"),
    menu_row(ids::CTX_MENU_UNIT_PIXELS, "chrome.menu.pixels"),
];

// Angle-unit submenu — o irmão do de cima, para o ÂNGULO. O armazenamento
// continua em radianos; isto só troca o FORMATO.
/// As linhas de `ContextMenuKind::SettingsAngleSubmenu`.
pub(super) const SETTINGS_ANGLE_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_ANGLE_DEGREES, "chrome.menu.degrees"),
    menu_row(ids::CTX_MENU_ANGLE_RADIANS, "chrome.menu.radians"),
];

// Image-filter submenu — the single global sampling mode
// applied to every sprite/texture + the Vello preview.
/// As linhas de `ContextMenuKind::SettingsFilterSubmenu`.
pub(super) const SETTINGS_FILTER_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_FILTER_PIXELART, "chrome.menu.pixel_art_crisp"),
    menu_row(ids::CTX_MENU_FILTER_SMOOTH, "chrome.menu.smooth_bilinear"),
];

// Display submenu — runtime swap-chain present mode. VSync is
// perfectly smooth; Immediate is non-blocking (no mouse-stutter)
// at the cost of vsync-pacing.
/// As linhas de `ContextMenuKind::SettingsDisplaySubmenu`.
pub(super) const SETTINGS_DISPLAY_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_DISPLAY_VSYNC, "chrome.menu.vsync_smooth"),
    menu_row(
        ids::CTX_MENU_DISPLAY_IMMEDIATE,
        "chrome.menu.immediate_no_stutter",
    ),
];

// Text rendering submenu — pela ordem de `ph2d_tokens::TextRendering::ALL`.
/// As linhas de `ContextMenuKind::SettingsTextSubmenu`.
pub(super) const SETTINGS_TEXT_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_TEXT_DEFAULT, "chrome.menu.default"),
    menu_row(ids::CTX_MENU_TEXT_CRISP_HEAVY, "chrome.menu.crisp_heavy"),
    menu_row(
        ids::CTX_MENU_TEXT_CRISP_HEAVY_PLUS,
        "chrome.menu.crisp_heavy_plus",
    ),
    menu_row(
        ids::CTX_MENU_TEXT_CRISP_DARKENED,
        "chrome.menu.crisp_stem_darkening",
    ),
];

// Motion submenu — o carácter da UI viva + o reduced motion.
//
// ⚠️ As duas primeiras linhas são um RÁDIO (o gosto) e a terceira é um TOGGLE (a garantia).
// O bullet significa a mesma coisa nas três — *este é o estado corrente* — que é a
// convenção de menu de plataforma, e é por isso que as três cabem numa tabela só.
/// As linhas de `ContextMenuKind::SettingsFontSubmenu` — pela ordem de [`ph2d_tokens::UiFont::ALL`].
pub(super) const SETTINGS_FONT_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_FONT_INTER, "chrome.menu.font_inter"),
    menu_row(ids::CTX_MENU_FONT_NOTO_SANS, "chrome.menu.font_noto_sans"),
    menu_row(ids::CTX_MENU_FONT_ATKINSON, "chrome.menu.font_atkinson"),
];

/// As linhas de `ContextMenuKind::SettingsWeightSubmenu`.
pub(super) const SETTINGS_WEIGHT_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_WEIGHT_LIGHT, "chrome.menu.weight_light"),
    menu_row(ids::CTX_MENU_WEIGHT_NORMAL, "chrome.menu.weight_normal"),
    menu_row(ids::CTX_MENU_WEIGHT_STRONG, "chrome.menu.weight_strong"),
];

/// As linhas de `ContextMenuKind::SettingsSizeSubmenu`.
pub(super) const SETTINGS_SIZE_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_SIZE_SMALL, "chrome.menu.size_small"),
    menu_row(ids::CTX_MENU_SIZE_NORMAL, "chrome.menu.size_normal"),
    menu_row(ids::CTX_MENU_SIZE_LARGE, "chrome.menu.size_large"),
];

/// As linhas de `ContextMenuKind::SettingsScaleSubmenu`.
pub(super) const SETTINGS_SCALE_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_SCALE_80, "chrome.menu.scale_80"),
    menu_row(ids::CTX_MENU_SCALE_90, "chrome.menu.scale_90"),
    menu_row(ids::CTX_MENU_SCALE_100, "chrome.menu.scale_100"),
    menu_row(ids::CTX_MENU_SCALE_125, "chrome.menu.scale_125"),
    menu_row(ids::CTX_MENU_SCALE_150, "chrome.menu.scale_150"),
    menu_row(ids::CTX_MENU_SCALE_175, "chrome.menu.scale_175"),
    menu_row(ids::CTX_MENU_SCALE_200, "chrome.menu.scale_200"),
];

/// As linhas de `ContextMenuKind::SettingsMotionSubmenu`.
pub(super) const SETTINGS_MOTION_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_MOTION_EXPRESSIVE, "chrome.menu.expressive"),
    menu_row(ids::CTX_MENU_MOTION_DISCRETE, "chrome.menu.discrete"),
    menu_row(ids::CTX_MENU_MOTION_REDUCED, "chrome.menu.reduced_motion"),
];

/// As linhas de `ContextMenuKind::SettingsLightSubmenu` — os três níveis e o «Automatic».
pub(super) const SETTINGS_LIGHT_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_LIGHT_NIVEL[0], "chrome.menu.light_low"),
    menu_row(ids::CTX_MENU_LIGHT_NIVEL[1], "chrome.menu.light_medium"),
    menu_row(ids::CTX_MENU_LIGHT_NIVEL[2], "chrome.menu.light_high"),
    menu_row(ids::CTX_MENU_LIGHT_AUTO, "chrome.menu.light_automatic"),
];

/// As linhas de `ContextMenuKind::SettingsRecoverySubmenu` — pela ordem de
/// [`crate::recovery_pref::OPTIONS_SECS`].
pub(super) const SETTINGS_RECOVERY_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_RECOVERY[0], "chrome.menu.recovery_off"),
    menu_row(ids::CTX_MENU_RECOVERY[1], "chrome.menu.recovery_10s"),
    menu_row(ids::CTX_MENU_RECOVERY[2], "chrome.menu.recovery_30s"),
    menu_row(ids::CTX_MENU_RECOVERY[3], "chrome.menu.recovery_1m"),
    menu_row(ids::CTX_MENU_RECOVERY[4], "chrome.menu.recovery_2m"),
    menu_row(ids::CTX_MENU_RECOVERY[5], "chrome.menu.recovery_5m"),
    menu_row(ids::CTX_MENU_RECOVERY[6], "chrome.menu.recovery_10m"),
    menu_row(ids::CTX_MENU_RECOVERY[7], "chrome.menu.recovery_15m"),
    menu_row(ids::CTX_MENU_RECOVERY[8], "chrome.menu.recovery_30m"),
];
