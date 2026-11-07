//! O tilemap: o mapa é a verdade, a imagem deriva dele, e desfazer volta ao byte.

use std::collections::BTreeMap;

use ph2d_pixel_core::codec::{read_library, write_library};
use ph2d_pixel_core::frames::{NewFrame, new_frame};
use ph2d_pixel_core::tiles::new_tilemap_layer;
use ph2d_pixel_core::{
    Background, Cel, ColorMode, FrameId, History, LayerId, NewSprite, Op, OpError, Patch, Rect,
    Sprite, TileGrid, TileRef, TilesSet, TilesetId, compose_rgba, derived_is_consistent,
};

const T: u16 = 2;

fn base(mode: ColorMode) -> Sprite {
    Sprite::new(NewSprite {
        width: 8,
        height: 8,
        mode,
        background: Background::Transparent,
    })
    .expect("sprite")
}

/// Um valor distinto por célula do tile, sem repetir, com alfa cheio no RGBA.
fn tile_px(s: &Sprite, seed: u8) -> Vec<u8> {
    let bpp = s.mode().bpp();
    let mut out = Vec::new();
    for i in 0..(T * T) as u8 {
        let v = seed + i;
        match bpp {
            4 => out.extend_from_slice(&[v, v, v, 255]),
            2 => out.extend_from_slice(&[v, 255]),
            _ => out.push(v),
        }
    }
    out
}

/// Sprite com uma camada de tilemap, dois tiles e um mapa 2×1 `[1, 2]`.
fn with_map(mode: ColorMode, h: &mut History) -> (Sprite, LayerId, FrameId, TilesetId) {
    let mut s = base(mode);
    let above = s.layers()[0].id;
    let f = s.frames()[0].id;
    let (op, l, set) = new_tilemap_layer(&mut s, above, T, T).expect("camada");
    h.commit(&mut s, op).expect("commit camada");
    let (a, b) = (tile_px(&s, 10), tile_px(&s, 20));
    let op = TilesSet {
        counts: vec![(set, 3)],
        tiles: vec![(set, 1, a), (set, 2, b)],
        cels: vec![(
            (l, f),
            Some(Cel::tilemap(TileGrid {
                x: 2,
                y: 0,
                w: 2,
                h: 1,
                tiles: vec![
                    TileRef::new(1, false, false, false),
                    TileRef::new(2, false, false, false),
                ],
                ..TileGrid::default()
            })),
        )],
        ..TilesSet::default()
    };
    h.commit(&mut s, Op::Tiles(Box::new(op)))
        .expect("commit mapa");
    (s, l, f, set)
}

#[test]
fn the_image_of_a_cel_is_its_map_expanded() {
    let mut h = History::default();
    let (s, l, f, _) = with_map(ColorMode::Rgba, &mut h);
    assert!(derived_is_consistent(&s));
    let c = s.cel((l, f)).expect("cel");
    assert_eq!(c.bbox, Rect::new(2, 0, 4, 2));
    // Primeira linha: as duas primeiras células de cada tile; segunda: as duas seguintes.
    let cinzas: Vec<u8> = c.data.as_chunks::<4>().0.iter().map(|p| p[0]).collect();
    assert_eq!(cinzas, [10, 11, 20, 21, 12, 13, 22, 23]);
}

/// As 8 combinações: D troca os eixos, depois X e Y (medir no Trial é a M4).
#[test]
fn each_flip_combination_lands_each_cell_where_the_law_says() {
    let casos: [((bool, bool, bool), [u8; 4]); 8] = [
        ((false, false, false), [0, 1, 2, 3]),
        ((true, false, false), [1, 0, 3, 2]),
        ((false, true, false), [2, 3, 0, 1]),
        ((true, true, false), [3, 2, 1, 0]),
        ((false, false, true), [0, 2, 1, 3]),
        ((true, false, true), [2, 0, 3, 1]),
        ((false, true, true), [1, 3, 0, 2]),
        ((true, true, true), [3, 1, 2, 0]),
    ];
    for ((x, y, d), esperado) in casos {
        let mut h = History::default();
        let (mut s, l, f, _) = with_map(ColorMode::Rgba, &mut h);
        let op = TilesSet {
            cels: vec![(
                (l, f),
                Some(Cel::tilemap(TileGrid {
                    x: 0,
                    y: 0,
                    w: 1,
                    h: 1,
                    tiles: vec![TileRef::new(1, x, y, d)],
                    ..TileGrid::default()
                })),
            )],
            ..TilesSet::default()
        };
        h.commit(&mut s, Op::Tiles(Box::new(op))).expect("commit");
        let got: Vec<u8> = s
            .cel((l, f))
            .unwrap()
            .data
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| p[0] - 10)
            .collect();
        assert_eq!(got, esperado, "x={x} y={y} d={d}");
    }
}

#[test]
fn editing_a_tile_changes_every_copy_in_every_frame_and_undo_returns_the_byte() {
    let mut h = History::default();
    let (mut s, l, f, set) = with_map(ColorMode::Rgba, &mut h);
    let (op, f2) = new_frame(&mut s, f, NewFrame::Copy).expect("frame");
    h.commit(&mut s, op).expect("commit frame");
    assert!(
        s.cel((l, f2)).expect("copiado").tiles.is_some(),
        "o quadro novo leva o mapa"
    );
    let antes = s.clone();
    let novo = tile_px(&s, 100);
    let op = TilesSet {
        tiles: vec![(set, 1, novo)],
        ..TilesSet::default()
    };
    h.commit(&mut s, Op::Tiles(Box::new(op))).expect("commit");
    assert!(derived_is_consistent(&s));
    for frame in [f, f2] {
        assert_eq!(s.cel((l, frame)).unwrap().data[0], 100, "{frame:?}");
    }
    assert!(h.undo(&mut s));
    assert_eq!(s, antes);
    assert!(h.redo(&mut s));
    assert!(derived_is_consistent(&s));
}

/// Refazer só as linhas que mudaram tem de dar a imagem inteira, em cada flip e em cada linha do tile.
#[test]
fn editing_one_row_of_a_tile_redraws_every_flip_exactly() {
    const LADO: u16 = 4;
    let mut h = History::default();
    let mut s = base(ColorMode::Rgba);
    let (acima, f) = (s.layers()[0].id, s.frames()[0].id);
    let (op, l, set) = new_tilemap_layer(&mut s, acima, LADO, LADO).expect("camada");
    h.commit(&mut s, op).expect("commit camada");
    let celulas = usize::from(LADO) * usize::from(LADO);
    let tile: Vec<u8> = (0..celulas * 4).map(|i| i as u8).collect();
    let flips = |i: u32| TileRef::new(1, i & 1 != 0, i & 2 != 0, i & 4 != 0);
    let op = TilesSet {
        counts: vec![(set, 2)],
        tiles: vec![(set, 1, tile.clone())],
        cels: vec![(
            (l, f),
            Some(Cel::tilemap(TileGrid {
                x: 0,
                y: 0,
                w: 8,
                h: 1,
                tiles: (0..8).map(flips).collect(),
                ..TileGrid::default()
            })),
        )],
        ..TilesSet::default()
    };
    h.commit(&mut s, Op::Tiles(Box::new(op))).expect("mapa");
    for linha in 0..usize::from(LADO) {
        let antes = s.clone();
        let mut novo = tile.clone();
        novo[linha * usize::from(LADO) * 4 + 4] ^= 0xff;
        let op = TilesSet {
            tiles: vec![(set, 1, novo)],
            ..TilesSet::default()
        };
        h.commit(&mut s, Op::Tiles(Box::new(op))).expect("edita");
        assert!(derived_is_consistent(&s), "linha {linha}");
        assert_ne!(s.cel((l, f)).unwrap().data, antes.cel((l, f)).unwrap().data);
        assert!(h.undo(&mut s));
        assert_eq!(s, antes, "desfazer da linha {linha}");
    }
    let mut novo = tile.clone();
    novo[4] ^= 0xff;
    novo[(usize::from(LADO) - 1) * usize::from(LADO) * 4 + 4] ^= 0xff;
    let op = TilesSet {
        tiles: vec![(set, 1, novo)],
        ..TilesSet::default()
    };
    h.commit(&mut s, Op::Tiles(Box::new(op)))
        .expect("duas linhas");
    assert!(derived_is_consistent(&s), "a primeira e a última linha");
}

#[test]
fn undo_takes_back_a_new_layer_and_its_tileset_to_the_byte() {
    let mut h = History::default();
    let vazio = base(ColorMode::Rgba);
    let (mut s, ..) = with_map(ColorMode::Rgba, &mut h);
    assert!(h.undo(&mut s));
    assert!(h.undo(&mut s));
    assert_eq!(s.layers(), vazio.layers());
    assert!(s.tilesets().is_empty());
    assert_eq!(s.entries().count(), 0);
}

#[test]
fn the_empty_tile_cannot_be_edited_and_a_used_tile_cannot_vanish() {
    let mut h = History::default();
    let (mut s, l, f, set) = with_map(ColorMode::Rgba, &mut h);
    let antes = s.clone();
    let zero = TilesSet {
        tiles: vec![(set, 0, tile_px(&s, 1))],
        ..TilesSet::default()
    };
    assert_eq!(apply_err(&mut s, zero), OpError::BadTiles("tiles.index"));
    let encolhe = TilesSet {
        counts: vec![(set, 2)],
        ..TilesSet::default()
    };
    assert_eq!(apply_err(&mut s, encolhe), OpError::BadTiles("grid.tile"));
    let fora = TilesSet {
        cels: vec![(
            (l, f),
            Some(Cel::tilemap(TileGrid {
                w: 1,
                h: 1,
                tiles: vec![TileRef::new(9, false, false, false)],
                ..TileGrid::default()
            })),
        )],
        ..TilesSet::default()
    };
    assert_eq!(apply_err(&mut s, fora), OpError::BadTiles("grid.tile"));
    let sem_tileset_em_uso = TilesSet {
        sets: vec![(set, None)],
        ..TilesSet::default()
    };
    assert_eq!(
        apply_err(&mut s, sem_tileset_em_uso),
        OpError::BadTiles("sets.in_use")
    );
    assert_eq!(s, antes, "recusa não muda nada");
}

fn visto(s: &Sprite, f: FrameId) -> Vec<u8> {
    let mut out = Vec::new();
    compose_rgba(s, f, None, &mut out);
    out
}

fn apply_err(s: &mut Sprite, t: TilesSet) -> OpError {
    ph2d_pixel_core::apply(s, Op::Tiles(Box::new(t))).expect_err("recusa")
}

/// Encolher um tileset cujos últimos tiles ninguém usa volta ao byte no desfazer.
#[test]
fn shrinking_restores_the_removed_tiles_on_undo() {
    let mut h = History::default();
    let (mut s, _, _, set) = with_map(ColorMode::Rgba, &mut h);
    let cresce = TilesSet {
        counts: vec![(set, 4)],
        tiles: vec![(set, 3, tile_px(&s, 60))],
        ..TilesSet::default()
    };
    h.commit(&mut s, Op::Tiles(Box::new(cresce)))
        .expect("cresce");
    let antes = s.clone();
    let encolhe = TilesSet {
        counts: vec![(set, 3)],
        ..TilesSet::default()
    };
    h.commit(&mut s, Op::Tiles(Box::new(encolhe)))
        .expect("encolhe");
    assert_eq!(s.tileset(set).unwrap().tiles.len(), 3);
    assert!(h.undo(&mut s));
    assert_eq!(s, antes);
}

#[test]
fn a_pixel_patch_on_a_tilemap_layer_is_refused() {
    let mut h = History::default();
    let (mut s, l, f, _) = with_map(ColorMode::Rgba, &mut h);
    let p = Patch {
        layer: l,
        frame: f,
        bbox: Some(Rect::new(0, 0, 1, 1)),
        rect: Rect::new(0, 0, 1, 1),
        pixels: vec![1, 2, 3, 255],
    };
    assert_eq!(
        ph2d_pixel_core::apply(&mut s, Op::Patch(p)),
        Err(OpError::TilemapLayer(l))
    );
    assert!(derived_is_consistent(&s));
}

#[test]
fn the_file_returns_equal_and_the_bytes_are_deterministic() {
    for mode in [ColorMode::Rgba, ColorMode::Gray, ColorMode::Indexed] {
        let mut h = History::default();
        let (s, l, f, set) = with_map(mode, &mut h);
        let mut lib = BTreeMap::new();
        lib.insert(5, s.clone());
        let bytes = write_library(&lib);
        let volta = read_library(&bytes).expect("lê");
        assert_eq!(volta, lib, "{mode:?}");
        assert_eq!(write_library(&volta), bytes);
        let lida = &volta[&5];
        assert_eq!(lida.layer(l).unwrap().tileset(), Some(set));
        assert!(lida.cel((l, f)).unwrap().tiles.is_some());
        assert!(derived_is_consistent(lida));
    }
}

/// O mapa manda: píxeis adulterados no ficheiro não sobrevivem à leitura.
#[test]
fn opening_a_file_rederives_the_image_from_the_map() {
    let mut h = History::default();
    let (s, l, f, _) = with_map(ColorMode::Rgba, &mut h);
    let mut lib = BTreeMap::new();
    lib.insert(5, s.clone());
    let bytes = write_library(&lib);
    let px = s.cel((l, f)).unwrap().data.clone();
    let at = bytes
        .windows(px.len())
        .position(|w| w == px)
        .expect("data gravado");
    let mut torto = bytes.clone();
    torto[at] ^= 0xff;
    assert_eq!(read_library(&torto).expect("lê")[&5], s);
}

#[test]
fn a_map_pointing_at_a_missing_tile_is_refused_on_open() {
    let mut h = History::default();
    let (s, ..) = with_map(ColorMode::Rgba, &mut h);
    let mut lib = BTreeMap::new();
    lib.insert(5, s);
    let bytes = write_library(&lib);
    // O id 2 do mapa grava-se como 02 00 00 00 logo depois do id 1.
    let alvo = [1, 0, 0, 0, 2, 0, 0, 0];
    let at = bytes
        .windows(8)
        .position(|w| w == alvo)
        .expect("mapa gravado");
    let mut torto = bytes;
    torto[at + 4] = 99;
    assert!(read_library(&torto).is_err());
}

/// Trocar o modo de cor e reordenar a paleta levam o tileset junto, e a imagem fica igual.
#[test]
fn color_mode_and_palette_remap_carry_the_tileset() {
    let mut h = History::default();
    let (mut s, l, f, set) = with_map(ColorMode::Indexed, &mut h);
    let visto = visto(&s, f);
    let n = s.palette().len();
    let perm: Vec<usize> = (0..n).rev().collect();
    h.commit(&mut s, Op::Remap { perm }).expect("remap");
    assert!(derived_is_consistent(&s));
    assert_eq!(self::visto(&s, f), visto, "remap não muda o desenho");
    let w = ph2d_pixel_core::convert::convert(
        &s,
        ColorMode::Rgba,
        ph2d_pixel_core::convert::Convert::default(),
    )
    .expect("converte");
    h.commit(&mut s, Op::Whole(Box::new(w))).expect("commit");
    assert!(derived_is_consistent(&s));
    assert_eq!(
        s.tileset(set).unwrap().tiles[1].len(),
        usize::from(T * T) * 4
    );
    assert_eq!(
        self::visto(&s, f),
        visto,
        "trocar de modo não muda o desenho"
    );
    assert_eq!(s.cel((l, f)).unwrap().bbox, Rect::new(2, 0, 4, 2));
    assert!(h.undo(&mut s));
    assert!(derived_is_consistent(&s));
}

/// Fixtura real: um ficheiro MINOR 3 escrito pelo código do `main` de 06/11 (`bbfefaf393`).
#[test]
fn a_minor_3_file_written_by_the_old_code_opens_the_same() {
    let antigo: &[u8] = include_bytes!("../fixtures/pixel_minor3_bbfefaf393.bin");
    assert_eq!(&antigo[9..13], &[1, 0, 3, 0], "MAJOR 1, MINOR 3");
    let lib = read_library(antigo).expect("lê");
    assert!(lib.len() >= 2);
    let novo = write_library(&lib);
    // O documento não mudou: regravar só troca o MINOR.
    assert_eq!(&novo[..9], &antigo[..9]);
    assert_eq!(&novo[11..13], &ph2d_pixel_core::MINOR.to_le_bytes());
    assert_eq!(&novo[13..], &antigo[13..]);
}

/// Grava um blob MINOR 4 para a prova do binário antigo (`docs/PixelArt/P10_plano.md` M7).
#[test]
#[ignore = "só gera a entrada da prova do binário antigo"]
fn write_a_minor_4_blob_for_the_old_binary_proof() {
    let mut h = History::default();
    let (s, _, f, _) = with_map(ColorMode::Rgba, &mut h);
    let mut lib = BTreeMap::new();
    lib.insert(5, s.clone());
    let out = std::env::var("PH2D_SAI").expect("PH2D_SAI");
    std::fs::write(&out, write_library(&lib)).expect("grava");
    std::fs::write(format!("{out}.rgba"), visto(&s, f)).expect("grava");
}
