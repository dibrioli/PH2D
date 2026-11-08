//! O que o `.ase` guarda de um tilemap. Só a grelha quadrada cabe no formato.

use super::*;

/// Só a grelha quadrada cabe no formato.
pub(super) fn vai_em_tiles(s: &Sprite, l: &Layer) -> bool {
    l.tileset()
        .and_then(|t| s.tileset(t))
        .is_some_and(|t| t.forma == Forma::Square)
}

/// O mapa de um cel, com as máscaras da spec.
pub(super) fn mapa_saida(c: &Cel) -> AseTilemap {
    let g = c.tiles.as_deref().cloned().unwrap_or_default();
    AseTilemap {
        w: g.w as u16,
        h: g.h as u16,
        tile_mask: TileRef::ID_MASK,
        x_flip: TileRef::FLIP_X,
        y_flip: TileRef::FLIP_Y,
        d_flip: TileRef::FLIP_D,
        tiles: g.tiles.iter().map(|t| t.0).collect(),
    }
}

/// Os tilesets do documento em tiras verticais.
pub(super) fn tilesets_saida(s: &Sprite) -> Vec<AseTileset> {
    s.tilesets()
        .iter()
        .filter(|t| t.forma == Forma::Square)
        .map(|t| AseTileset {
            id: t.id.0,
            name: t.name.clone(),
            zero_is_empty: true,
            tile_w: t.tile_w,
            tile_h: t.tile_h,
            count: t.tiles.len() as u32,
            pixels: t.tiles.concat(),
            user_data: None,
        })
        .collect()
}
