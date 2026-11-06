//! O gabarito de 47: a regra, um mapa real e o desfazer.

use ph2d_pixel_core::tiles::new_tilemap_layer;
use ph2d_pixel_core::{
    Background, ColorMode, FrameId, History, LayerId, NewSprite, Op, Sprite, TERRAIN_TILES, TileRef,
    derived_is_consistent,
};
use ph2d_pixel_tiles::mapa::{Mapa, carimba};
use ph2d_pixel_tiles::terreno::{self, E, N, NE, NW, S, SE, SW, VIZINHOS, W};

fn doc(h: &mut History) -> (Sprite, LayerId, FrameId, u32) {
    let mut s = Sprite::new(NewSprite {
        width: 64,
        height: 64,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .unwrap();
    let (acima, f) = (s.layers()[0].id, s.frames()[0].id);
    let (op, l, set) = new_tilemap_layer(&mut s, acima, 4, 4).unwrap();
    h.commit(&mut s, op).unwrap();
    let op = terreno::novo_terreno(&s, set, "Grass").unwrap();
    h.commit(&mut s, op).unwrap();
    let first = s.tileset(set).unwrap().terrains[0].first;
    (s, l, f, first)
}

fn mapa(s: &Sprite, l: LayerId, f: FrameId) -> Mapa {
    let set = s.tileset(s.layer(l).unwrap().tileset().unwrap()).unwrap();
    Mapa::de(s, l, f, set)
}

#[test]
fn every_neighbourhood_lands_on_one_of_exactly_47_shapes() {
    let formas = terreno::formas();
    assert_eq!(formas.len() as u32, TERRAIN_TILES);
    for m in 0..=u8::MAX {
        let i = terreno::casa(m) as usize;
        assert_eq!(formas[i], terreno::reduz(m), "máscara {m}");
    }
}

#[test]
fn a_corner_counts_only_with_both_of_its_edges() {
    assert_eq!(terreno::reduz(N | E), N | E);
    assert_eq!(terreno::reduz(N | E | NE), N | E | NE);
    assert_eq!(terreno::reduz(N | NE), N);
    assert_eq!(terreno::reduz(NE | SE | SW | NW), 0);
    assert_eq!(terreno::reduz(S | W | SW), S | W | SW);
}

/// Um mapa de verdade: sala, corredor de uma célula, buraco e ilha. Cada célula tem de ter o tile
/// da vizinhança que o mapa dá, calculada aqui à parte.
const REAL: [&str; 12] = [
    "..............................",
    ".########......##########.....",
    ".########......#........#.....",
    ".###..###......#.######.#.....",
    ".###..###......#.#....#.#.....",
    ".########......#.#.##.#.#.....",
    ".########......#.#.##.#.#.....",
    "....##.........#.#....#.#.....",
    "....##....##...#.######.#.....",
    "....#############.......#.....",
    "....##....##...##########.....",
    "..............................",
];

fn cheias() -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    for (r, linha) in REAL.iter().enumerate() {
        for (c, ch) in linha.bytes().enumerate() {
            if ch == b'#' {
                out.push((c as i32, r as i32));
            }
        }
    }
    out
}

#[test]
fn a_real_map_resolves_cell_by_cell() {
    let mut h = History::default();
    let (mut s, l, f, first) = doc(&mut h);
    let cheio = cheias();
    assert!(cheio.len() >= 100, "{}", cheio.len());
    let op = terreno::pinta(&s, l, f, &cheio, Some(0)).unwrap();
    h.commit(&mut s, op).unwrap();
    assert!(derived_is_consistent(&s));
    let m = mapa(&s, l, f);
    for &(c, r) in &cheio {
        let mut mascara = 0;
        for (b, (dx, dy)) in VIZINHOS.iter().enumerate() {
            if cheio.contains(&(c + dx, r + dy)) {
                mascara |= 1 << b;
            }
        }
        assert_eq!(m.get(c, r).id(), first + terreno::casa(mascara), "({c},{r})");
    }
}

/// Pintar em pedaços dá o mesmo que pintar de uma vez: as vizinhas corrigem-se sozinhas.
#[test]
fn painting_piece_by_piece_equals_painting_at_once() {
    let mut h = History::default();
    let (mut s, l, f, _) = doc(&mut h);
    let cheio = cheias();
    let mut um = s.clone();
    let op = terreno::pinta(&um, l, f, &cheio, Some(0)).unwrap();
    ph2d_pixel_core::apply(&mut um, op).unwrap();
    for pedaco in cheio.chunks(7) {
        let op = terreno::pinta(&s, l, f, pedaco, Some(0)).unwrap();
        h.commit(&mut s, op).unwrap();
    }
    assert_eq!(mapa(&s, l, f), mapa(&um, l, f));
}

#[test]
fn erasing_fixes_the_neighbours_and_undo_returns_the_byte() {
    let mut h = History::default();
    let (mut s, l, f, first) = doc(&mut h);
    let bloco: Vec<(i32, i32)> = (0..3).flat_map(|r| (0..3).map(move |c| (c, r))).collect();
    let op = terreno::pinta(&s, l, f, &bloco, Some(0)).unwrap();
    h.commit(&mut s, op).unwrap();
    let antes = s.clone();
    let op = terreno::pinta(&s, l, f, &[(1, 1)], None).unwrap();
    h.commit(&mut s, op).unwrap();
    let m = mapa(&s, l, f);
    assert_eq!(m.get(1, 1), TileRef::EMPTY);
    // A de cima perde o vizinho de baixo: fica só com lados E e W.
    assert_eq!(m.get(1, 0).id(), first + terreno::casa(E | W));
    assert!(h.undo(&mut s));
    assert_eq!(s, antes);
}

/// Um terreno não conta outro como vizinho (decisão do dono, 06/11).
#[test]
fn another_terrain_counts_as_empty() {
    let mut h = History::default();
    let (mut s, l, f, first) = doc(&mut h);
    let set = s.layer(l).unwrap().tileset().unwrap();
    let op = terreno::novo_terreno(&s, set, "Stone").unwrap();
    h.commit(&mut s, op).unwrap();
    let second = s.tileset(set).unwrap().terrains[1].first;
    let op = terreno::pinta(&s, l, f, &[(0, 0)], Some(0)).unwrap();
    h.commit(&mut s, op).unwrap();
    let op = terreno::pinta(&s, l, f, &[(1, 0)], Some(1)).unwrap();
    h.commit(&mut s, op).unwrap();
    let m = mapa(&s, l, f);
    assert_eq!(m.get(0, 0).id(), first + terreno::casa(0));
    assert_eq!(m.get(1, 0).id(), second + terreno::casa(0));
}

/// Quem foi carimbado à mão não é terreno: o pincel automático não o troca.
#[test]
fn a_hand_stamped_tile_is_left_alone_by_the_terrain() {
    let mut h = History::default();
    let (mut s, l, f, _) = doc(&mut h);
    let set = s.layer(l).unwrap().tileset().unwrap();
    let op = Op::Tiles(Box::new(ph2d_pixel_core::TilesSet {
        counts: vec![(set, s.tileset(set).unwrap().tiles.len() as u32 + 1)],
        ..Default::default()
    }));
    h.commit(&mut s, op).unwrap();
    let livre = s.tileset(set).unwrap().tiles.len() as u32 - 1;
    let tile = TileRef::new(livre, true, false, false);
    let op = carimba(&s, l, f, &[(5, 5)], tile).unwrap();
    h.commit(&mut s, op).unwrap();
    let op = terreno::pinta(&s, l, f, &[(5, 4), (4, 5), (6, 5), (5, 6)], Some(0)).unwrap();
    h.commit(&mut s, op).unwrap();
    assert_eq!(mapa(&s, l, f).get(5, 5), tile);
}

#[test]
fn the_box_shrinks_to_the_used_cells_and_a_map_can_go_negative() {
    let mut h = History::default();
    let (mut s, l, f, _) = doc(&mut h);
    let op = terreno::pinta(&s, l, f, &[(-2, -1), (3, 2)], Some(0)).unwrap();
    h.commit(&mut s, op).unwrap();
    let g = s.cel((l, f)).unwrap().tiles.clone().unwrap();
    assert_eq!((g.x, g.y, g.w, g.h), (-8, -4, 6, 4));
    let op = terreno::pinta(&s, l, f, &[(-2, -1)], None).unwrap();
    h.commit(&mut s, op).unwrap();
    let g = s.cel((l, f)).unwrap().tiles.clone().unwrap();
    assert_eq!((g.x, g.y, g.w, g.h), (12, 8, 1, 1));
    let op = terreno::pinta(&s, l, f, &[(3, 2)], None).unwrap();
    h.commit(&mut s, op).unwrap();
    assert!(s.entry((l, f)).is_none(), "sem células, sem cel");
    assert!(derived_is_consistent(&s));
}

/// A ordem dos bits é a da spec §15, e o gabarito depende dela.
#[test]
fn the_bit_order_is_clockwise_from_north() {
    assert_eq!([N, NE, E, SE, S, SW, W, NW], [1, 2, 4, 8, 16, 32, 64, 128]);
}
