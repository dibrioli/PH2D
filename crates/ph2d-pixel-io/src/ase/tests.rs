use super::*;

/// O formato antigo (bit 4 desligado): 0 é um tile de verdade, e o vazio é `0xffffffff`.
#[test]
fn without_the_zero_is_empty_bit_every_tile_stays_and_the_ids_shift() {
    let t = AseTileset {
        zero_is_empty: false,
        tile_w: 1,
        tile_h: 1,
        count: 2,
        pixels: vec![1, 2, 3, 255, 4, 5, 6, 255],
        ..AseTileset::default()
    };
    let set = tileset_de(&t, TilesetId(1), 4, 0, &[]).expect("tileset");
    assert_eq!(set.tiles.len(), 3, "o vazio entra no índice 0");
    assert_eq!(set.tiles[1], [1, 2, 3, 255]);
    let m = AseTilemap {
        w: 3,
        h: 1,
        tile_mask: TileRef::ID_MASK,
        x_flip: TileRef::FLIP_X,
        y_flip: TileRef::FLIP_Y,
        d_flip: TileRef::FLIP_D,
        tiles: vec![0, 1, u32::MAX],
    };
    let (g, corrompido) = mapa_de(&m, 0, 0, &set, false);
    assert!(!corrompido);
    let ids: Vec<u32> = g.tiles.iter().map(|t| t.id()).collect();
    assert_eq!(ids, [1, 2, 0]);
}

/// Índice fora do tileset e D em tile não quadrado: a célula sai sã, e o artista é avisado.
#[test]
fn a_bad_cell_stays_sane_and_says_so() {
    let t = AseTileset {
        zero_is_empty: true,
        tile_w: 2,
        tile_h: 1,
        count: 2,
        pixels: vec![0; 16],
        ..AseTileset::default()
    };
    let set = tileset_de(&t, TilesetId(1), 4, 0, &[]).expect("tileset");
    let m = AseTilemap {
        w: 2,
        h: 1,
        tile_mask: TileRef::ID_MASK,
        x_flip: TileRef::FLIP_X,
        y_flip: TileRef::FLIP_Y,
        d_flip: TileRef::FLIP_D,
        tiles: vec![7, 1 | TileRef::FLIP_D],
    };
    let (g, corrompido) = mapa_de(&m, 0, 0, &set, true);
    assert!(corrompido);
    assert_eq!(g.tiles[0], TileRef::EMPTY);
    assert_eq!(g.tiles[1], TileRef::new(1, false, false, false));
}
