//! **Os ids do Pixel Art que o chrome lê** (ADR-0196): o rect do painel PIXEL, o abridor dele no menu
//! *Window* e os controlos do diálogo *New Sprite* (o chrome pinta-o e despacha-o).
//!
//! ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use super::super::hash_node_id;
use crate::pixel_panel::{NewSpriteBackground, NewSpriteMode, PixelDither, PixelToGray};
use ph2d_a11y::NodeId;

pub const PIXEL_PANEL: NodeId = hash_node_id("pixel.panel");
/// O rect da linha do tempo do Pixel (camadas × frames, P4) — a banda de baixo.
pub const PIXEL_TIMELINE_PANEL: NodeId = hash_node_id("pixel_timeline.panel");
/// O rect da janela Preview do Pixel (P5): flutuante, arrasta-se pelo título.
pub const PIXEL_PREVIEW_PANEL: NodeId = hash_node_id("pixel_preview.panel");
/// O rect da janela de Export e Import do Pixel (P9b).
pub const PIXEL_EXPORT_PANEL: NodeId = hash_node_id("pixel_export.panel");
/// O rect da tela dos tilemaps do Pixel (P10b).
pub const PIXEL_TILES_PANEL: NodeId = hash_node_id("pixel_tiles.panel");
/// O rect da janela SLICES do Pixel (P11b).
pub const PIXEL_SLICES_PANEL: NodeId = hash_node_id("pixel_slices.panel");
/// O rect da janela KEYBOARD SHORTCUTS do Pixel (P12a).
pub const PIXEL_KEYS_PANEL: NodeId = hash_node_id("pixel_keys.panel");
/// A janela UNDO HISTORY do Pixel (P12a, `ph2d-panel-pixel-history`).
pub const PIXEL_HISTORY_PANEL: NodeId = hash_node_id("pixel_history.panel");
/// A janela GRID & BACKGROUND do Pixel (P12 P, `ph2d-panel-pixel-prefs`).
pub const PIXEL_PREFS_PANEL: NodeId = hash_node_id("pixel_prefs.panel");

/// As linhas do Pixel no menu *File*: só existem com um Pixel Sprite em Draw.
pub const CTX_MENU_PIXEL_OPEN: NodeId = hash_node_id("ctx_menu_pixel_open");
pub const CTX_MENU_PIXEL_IMPORT_SHEET: NodeId = hash_node_id("ctx_menu_pixel_import_sheet");
pub const CTX_MENU_PIXEL_EXPORT_AS: NodeId = hash_node_id("ctx_menu_pixel_export_as");
pub const CTX_MENU_PIXEL_EXPORT_SHEET: NodeId = hash_node_id("ctx_menu_pixel_export_sheet");
pub const CTX_MENU_PIXEL_EXPORT_TILESET: NodeId = hash_node_id("ctx_menu_pixel_export_tileset");
pub const CTX_MENU_PIXEL_REPEAT_EXPORT: NodeId = hash_node_id("ctx_menu_pixel_repeat_export");

/// A linha *Pixel* do menu *Window*.
pub const TOPBAR_PIXEL: NodeId = hash_node_id("topbar_pixel");
/// A linha *Pixel Preview* do menu *Window* (a janela Preview, P5; também F7).
pub const TOPBAR_PIXEL_PREVIEW: NodeId = hash_node_id("topbar_pixel_preview");
/// As janelas acessórias do Pixel no menu *Window* (smoke do dono 08/11: «nem estão presentes no
/// menu Window»): Slices, Tilemap, Undo History, Keyboard Shortcuts, Grid & Background.
pub const TOPBAR_PIXEL_SLICES: NodeId = hash_node_id("topbar_pixel_slices");
pub const TOPBAR_PIXEL_TILES: NodeId = hash_node_id("topbar_pixel_tiles");
pub const TOPBAR_PIXEL_HISTORY: NodeId = hash_node_id("topbar_pixel_history");
pub const TOPBAR_PIXEL_KEYS: NodeId = hash_node_id("topbar_pixel_keys");
pub const TOPBAR_PIXEL_PREFS: NodeId = hash_node_id("topbar_pixel_prefs");

/// A amostra da cor de FRENTE (a coluna da esquerda em Draw): um Down abre o seletor partilhado, e
/// a família lê de volta a cor dele (`ph2d_app_pixel::panel`).
pub const PIXEL_FG: NodeId = hash_node_id("pixel.fg");
/// A amostra da cor de FUNDO, na mesma forma.
pub const PIXEL_BG: NodeId = hash_node_id("pixel.bg");

/// O campo da largura do *New Sprite*.
pub const NEW_SPRITE_WIDTH: NodeId = hash_node_id("pixel.new_sprite.width");
/// O campo da altura do *New Sprite*.
pub const NEW_SPRITE_HEIGHT: NodeId = hash_node_id("pixel.new_sprite.height");
pub const NEW_SPRITE_OK: NodeId = hash_node_id("pixel.new_sprite.ok");
pub const NEW_SPRITE_CANCEL: NodeId = hash_node_id("pixel.new_sprite.cancel");

/// Os modos de cor: o valor, o botão e o rótulo — pintar, registar e despachar leem a MESMA tabela.
pub const NEW_SPRITE_MODES: [(NewSpriteMode, NodeId, &str); 3] = [
    (
        NewSpriteMode::Rgba,
        hash_node_id("pixel.new_sprite.mode.rgba"),
        "pixel.new_sprite.mode.rgba",
    ),
    (
        NewSpriteMode::Gray,
        hash_node_id("pixel.new_sprite.mode.gray"),
        "pixel.new_sprite.mode.gray",
    ),
    (
        NewSpriteMode::Indexed,
        hash_node_id("pixel.new_sprite.mode.indexed"),
        "pixel.new_sprite.mode.indexed",
    ),
];

/// Os fundos, na mesma forma.
pub const NEW_SPRITE_BGS: [(NewSpriteBackground, NodeId, &str); 3] = [
    (
        NewSpriteBackground::Transparent,
        hash_node_id("pixel.new_sprite.bg.transparent"),
        "pixel.new_sprite.bg.transparent",
    ),
    (
        NewSpriteBackground::White,
        hash_node_id("pixel.new_sprite.bg.white"),
        "pixel.new_sprite.bg.white",
    ),
    (
        NewSpriteBackground::Black,
        hash_node_id("pixel.new_sprite.bg.black"),
        "pixel.new_sprite.bg.black",
    ),
];

/// Os lados quadrados comuns em pixel art: o lado e o botão; pintar, registar e despachar leem a MESMA tabela.
pub const NEW_SPRITE_SIZES: [(u32, NodeId); 8] = [
    (16, hash_node_id("pixel.new_sprite.size.16")),
    (24, hash_node_id("pixel.new_sprite.size.24")),
    (32, hash_node_id("pixel.new_sprite.size.32")),
    (48, hash_node_id("pixel.new_sprite.size.48")),
    (64, hash_node_id("pixel.new_sprite.size.64")),
    (96, hash_node_id("pixel.new_sprite.size.96")),
    (128, hash_node_id("pixel.new_sprite.size.128")),
    (256, hash_node_id("pixel.new_sprite.size.256")),
];

/// Os ecrãs comuns em pixel art, na mesma forma.
pub const NEW_SPRITE_SCREENS: [([u32; 2], NodeId); 4] = [
    ([160, 144], hash_node_id("pixel.new_sprite.screen.160x144")),
    ([320, 180], hash_node_id("pixel.new_sprite.screen.320x180")),
    ([320, 240], hash_node_id("pixel.new_sprite.screen.320x240")),
    ([640, 360], hash_node_id("pixel.new_sprite.screen.640x360")),
];

/// O *Custom*: acende quando Width × Height não é nenhuma predefinição; carregar leva ao campo Width.
pub const NEW_SPRITE_CUSTOM: NodeId = hash_node_id("pixel.new_sprite.custom");

/// Os dois campos numéricos.
pub const NEW_SPRITE_FIELDS: [NodeId; 2] = [NEW_SPRITE_WIDTH, NEW_SPRITE_HEIGHT];

/// O `id` é um controlo do diálogo *New Sprite*? (Um clique nele não fecha o diálogo.)
#[must_use]
pub fn new_sprite_dialog_owns(id: NodeId) -> bool {
    NEW_SPRITE_FIELDS.contains(&id)
        || id == NEW_SPRITE_OK
        || id == NEW_SPRITE_CANCEL
        || NEW_SPRITE_MODES.iter().any(|(_, b, _)| *b == id)
        || NEW_SPRITE_BGS.iter().any(|(_, b, _)| *b == id)
        || NEW_SPRITE_SIZES.iter().any(|(_, b)| *b == id)
        || NEW_SPRITE_SCREENS.iter().any(|(_, b)| *b == id)
        || id == NEW_SPRITE_CUSTOM
}

/// A amostra do *Edit color* do painel PIXEL: um Down abre o seletor partilhado sobre a cor da
/// amostra clicada da paleta, e a família lê de volta a cor dele (`ph2d_app_pixel::panel`).
pub const PIXEL_PAL_EDIT: NodeId = hash_node_id("pixel.palette.edit");
/// O dropdown de fontes do Text do painel PIXEL; o gesto `fonte` do smoke abre-o.
pub const PIXEL_TEXT_FONT_DD: NodeId = hash_node_id("pixel.opt.text.font");
/// O ≡ da barra da paleta do painel PIXEL (o menu das acções); a foto da cena 3 abre-o.
pub const PIXEL_PALETTE_MENU: NodeId = hash_node_id("pixel.palette.menu");

pub const COLOR_MODE_OK: NodeId = hash_node_id("pixel.color_mode.ok");
pub const COLOR_MODE_CANCEL: NodeId = hash_node_id("pixel.color_mode.cancel");

/// Os alvos do diálogo *Color Mode*: o valor, o botão e o rótulo (as chaves dos modos do New Sprite).
pub const COLOR_MODE_TARGETS: [(NewSpriteMode, NodeId, &str); 3] = [
    (
        NewSpriteMode::Rgba,
        hash_node_id("pixel.color_mode.target.rgba"),
        "pixel.new_sprite.mode.rgba",
    ),
    (
        NewSpriteMode::Gray,
        hash_node_id("pixel.color_mode.target.gray"),
        "pixel.new_sprite.mode.gray",
    ),
    (
        NewSpriteMode::Indexed,
        hash_node_id("pixel.color_mode.target.indexed"),
        "pixel.new_sprite.mode.indexed",
    ),
];

/// Os oito dithers, na ordem da lista do Aseprite.
pub const COLOR_MODE_DITHERS: [(PixelDither, NodeId, &str); 8] = [
    (
        PixelDither::None,
        hash_node_id("pixel.color_mode.dither.none"),
        "pixel.color_mode.dither.none",
    ),
    (
        PixelDither::Ordered2,
        hash_node_id("pixel.color_mode.dither.ordered2"),
        "pixel.color_mode.dither.size2",
    ),
    (
        PixelDither::Ordered4,
        hash_node_id("pixel.color_mode.dither.ordered4"),
        "pixel.color_mode.dither.size4",
    ),
    (
        PixelDither::Ordered8,
        hash_node_id("pixel.color_mode.dither.ordered8"),
        "pixel.color_mode.dither.size8",
    ),
    (
        PixelDither::Old2,
        hash_node_id("pixel.color_mode.dither.old2"),
        "pixel.color_mode.dither.size2",
    ),
    (
        PixelDither::Old4,
        hash_node_id("pixel.color_mode.dither.old4"),
        "pixel.color_mode.dither.size4",
    ),
    (
        PixelDither::Old8,
        hash_node_id("pixel.color_mode.dither.old8"),
        "pixel.color_mode.dither.size8",
    ),
    (
        PixelDither::FloydSteinberg,
        hash_node_id("pixel.color_mode.dither.floyd"),
        "pixel.color_mode.dither.floyd",
    ),
];

/// A arrumação dos dithers no diálogo: a 1.ª fila sem rótulo (os dois nomes longos), depois uma
/// fila por família com o rótulo dela à esquerda e os três tamanhos — os nomes nunca se cortam.
pub const COLOR_MODE_DITHER_WIDE: [PixelDither; 2] =
    [PixelDither::None, PixelDither::FloydSteinberg];
pub const COLOR_MODE_DITHER_ROWS: [(&str, [PixelDither; 3]); 2] = [
    (
        "pixel.color_mode.dither.ordered",
        [
            PixelDither::Ordered2,
            PixelDither::Ordered4,
            PixelDither::Ordered8,
        ],
    ),
    (
        "pixel.color_mode.dither.old",
        [PixelDither::Old2, PixelDither::Old4, PixelDither::Old8],
    ),
];

/// Os três métodos de cinza.
pub const COLOR_MODE_GRAYS: [(PixelToGray, NodeId, &str); 3] = [
    (
        PixelToGray::Luminance,
        hash_node_id("pixel.color_mode.gray.luminance"),
        "pixel.color_mode.gray.luminance",
    ),
    (
        PixelToGray::Hsv,
        hash_node_id("pixel.color_mode.gray.hsv"),
        "pixel.color_mode.gray.hsv",
    ),
    (
        PixelToGray::Hsl,
        hash_node_id("pixel.color_mode.gray.hsl"),
        "pixel.color_mode.gray.hsl",
    ),
];

/// O `id` é um controlo do diálogo *Color Mode*?
#[must_use]
pub fn color_mode_dialog_owns(id: NodeId) -> bool {
    id == COLOR_MODE_OK
        || id == COLOR_MODE_CANCEL
        || COLOR_MODE_TARGETS.iter().any(|(_, b, _)| *b == id)
        || COLOR_MODE_DITHERS.iter().any(|(_, b, _)| *b == id)
        || COLOR_MODE_GRAYS.iter().any(|(_, b, _)| *b == id)
}
