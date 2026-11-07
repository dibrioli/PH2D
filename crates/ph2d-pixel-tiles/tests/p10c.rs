//! **P10c: iso, hex e o gabarito.** A lei lê a rodada do Godot (`docs/PixelArt/P10_oraculos.md`, M2iso,
//! M3b, M8a) e confere com `ph2d-grid`.

use ph2d_grid::GridMath;
use ph2d_grid::hex::{HexGrid, HexOffset, OffsetCell};
use ph2d_grid::iso::{IsoGrid, iso_to_world, world_to_iso};
use ph2d_grid::square::SquareNeighborhood;
use ph2d_grid::staggered::StaggeredHexGrid;
use ph2d_pixel_core::codec::{read_library, write_library};
use ph2d_pixel_core::tiles::{expand, new_tilemap_layer_forma};
use ph2d_pixel_core::{
    Background, ColorMode, Forma, FrameId, History, LayerId, NewSprite, Sprite, TERRAIN_TILES,
    TileGrid, TileRef, TilesSet, Tileset, TilesetId, derived_is_consistent,
};
use ph2d_pixel_tiles::desenho::{Modo, aplica};
use ph2d_pixel_tiles::exemplo::celulas_reais;
use ph2d_pixel_tiles::mao::{Alvo, Carimbo, Mao, Marca, marca_de, pega};
use ph2d_pixel_tiles::mapa::{Mapa, carimba};
use ph2d_pixel_tiles::{terreno, tileset as lei};
use serde_json::Value;
use std::collections::BTreeMap;

const PRETO: u32 = 0xff00_0000;

fn oraculo() -> Value {
    serde_json::from_str(include_str!("fixtures/oracle_godot_p10c.json")).unwrap()
}

fn forma_de(nome: &str) -> Forma {
    Forma::TODAS.into_iter().find(|f| f.nome() == nome).unwrap()
}

fn doc(forma: Forma, w: u16, h: u16) -> (Sprite, History, LayerId, FrameId, TilesetId) {
    let mut s = Sprite::new(NewSprite {
        width: 128,
        height: 128,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .unwrap();
    let mut hist = History::default();
    let (acima, f) = (s.layers()[0].id, s.frames()[0].id);
    let (op, l, set) = new_tilemap_layer_forma(&mut s, acima, w, h, forma).unwrap();
    hist.commit(&mut s, op).unwrap();
    (s, hist, l, f, set)
}

fn set_de(s: &Sprite, id: TilesetId) -> &Tileset {
    s.tileset(id).unwrap()
}

/// Cada píxel do M3b: o nosso clique, em inteiros, contra o `local_to_map` do Godot.
#[test]
fn iso_hex_batem_com_godot_e_ph2d_grid() {
    let o = oraculo();
    let mut configs = 0;
    for m in o["M3B"].as_array().unwrap() {
        let forma = forma_de(m["forma"].as_str().unwrap());
        let (w, h) = (
            m["tile"][0].as_i64().unwrap() as i32,
            m["tile"][1].as_i64().unwrap() as i32,
        );
        for c in m["centros"].as_array().unwrap() {
            let (cx, cy) = (c[0].as_i64().unwrap() as i32, c[1].as_i64().unwrap() as i32);
            let (ox, oy) = forma.origem(cx, cy, w, h);
            let got = (
                f64::from(ox) + f64::from(w) / 2.0,
                f64::from(oy) + f64::from(h) / 2.0,
            );
            assert!(
                (got.0 - c[2].as_f64().unwrap()).abs() < 1.0,
                "{forma:?} {w}x{h} {c}"
            );
            assert!(
                (got.1 - c[3].as_f64().unwrap()).abs() < 1.0,
                "{forma:?} {w}x{h} {c}"
            );
            assert_eq!(
                forma.celula(ox + w / 2, oy + h / 2, w, h),
                (cx, cy),
                "volta {forma:?} {w}x{h}"
            );
        }
        let (mut tot, mut difere) = (0, 0);
        let y0 = m["y0"].as_i64().unwrap() as i32;
        for (j, linha) in m["linhas"].as_array().unwrap().iter().enumerate() {
            let mut x = m["x0"].as_i64().unwrap() as i32;
            let y = y0 + j as i32;
            for corrida in linha.as_array().unwrap() {
                let n = corrida[0].as_i64().unwrap();
                let g = (
                    corrida[1].as_i64().unwrap() as i32,
                    corrida[2].as_i64().unwrap() as i32,
                );
                for _ in 0..n {
                    tot += 1;
                    let nossa = forma.celula(x, y, w, h);
                    if nossa != g {
                        difere += 1;
                        // Discordar só pode acontecer na aresta: a célula do Godot cobre o píxel.
                        let (ox, oy) = forma.origem(g.0, g.1, w, h);
                        assert!(
                            x >= ox - 1 && y >= oy - 1 && x <= ox + w && y <= oy + h,
                            "{forma:?} {w}x{h} ({x},{y}): nossa {nossa:?}, Godot {g:?}"
                        );
                    }
                    x += 1;
                }
            }
        }
        // Iso é exato; hex com arestas sobre o centro do píxel discorda só no desempate (≤ 3 %).
        if forma == Forma::Iso || (forma == Forma::Hex && w % 2 == 0 && h % 4 == 0) {
            assert_eq!(difere, 0, "{forma:?} {w}x{h}");
        } else {
            assert!(
                difere * 100 <= tot * 3,
                "{forma:?} {w}x{h}: {difere} de {tot}"
            );
        }
        configs += 1;
    }
    assert_eq!(configs, 15);
    // ph2d-grid: a mesma geometria, em f32. O hex regular tem largura √3·s; compara-se pela escala.
    for (c, r) in [(0, 0), (1, 0), (0, 1), (3, -2), (-4, 5)] {
        let g = IsoGrid::dimetric_2_1(16.0, SquareNeighborhood::Von4);
        let [x, y] = iso_to_world(&g, (c, r));
        assert_eq!(Forma::Iso.origem(c, r, 32, 16), (x as i32, y as i32));
    }
    let g = IsoGrid::dimetric_2_1(16.0, SquareNeighborhood::Von4);
    let mut difere = 0;
    for y in -64..64 {
        for x in -64..64 {
            let (px, py) = (x as f32 + 0.5 - 16.0, y as f32 + 0.5);
            difere += usize::from(world_to_iso(&g, [px, py]) != Forma::Iso.celula(x, y, 32, 16));
        }
    }
    assert_eq!(difere, 0, "iso × ph2d-grid");
    for (forma, grade, variante) in [
        (Forma::Hex, HexGrid::pointy(16.0), HexOffset::OddR),
        (Forma::HexFlat, HexGrid::flat(16.0), HexOffset::OddQ),
    ] {
        let grelha = StaggeredHexGrid::new(grade, variante);
        // Tile de 16×16 sobre o hexágono regular de raio 16.
        let raiz3 = 3.0_f32.sqrt();
        let (ex, ey) = if forma == Forma::Hex {
            (raiz3 * 16.0 / 16.0, 2.0)
        } else {
            (2.0, raiz3 * 16.0 / 16.0)
        };
        let (mut tot, mut difere) = (0, 0);
        for y in -48..48 {
            for x in -48..48 {
                let mundo = [(x as f32 + 0.5 - 8.0) * ex, (y as f32 + 0.5 - 8.0) * ey];
                let OffsetCell { col, row } = grelha.world_to_cell(mundo);
                tot += 1;
                difere += usize::from((col, row) != forma.celula(x, y, 16, 16));
            }
        }
        assert!(
            difere * 100 <= tot,
            "{forma:?} × ph2d-grid: {difere} de {tot}"
        );
    }
}

#[test]
fn a_rodada_do_godot_confirma_as_leis() {
    let o = oraculo();
    assert_eq!(o["M2ISO"]["diferem"], 0);
    assert_eq!(o["M2ISO"]["vizinhancas"], 256);
    assert_eq!(o["FLIPS"]["pixeis_diferem"], 0);
    for c in o["ORDEM"].as_array().unwrap() {
        assert_eq!(c["diferem_por_ordem"]["coluna_linha"], 0, "{}", c["forma"]);
    }
}

/// Um tile opaco por célula: quem fica por cima é a última coluna, e nela a última linha (M8a).
#[test]
fn a_sobreposicao_desenha_coluna_a_coluna() {
    let (mut s, mut h, l, f, set) = doc(Forma::Hex, 16, 16);
    let tile = |v: u32| v.to_le_bytes().repeat(256);
    let op = TilesSet {
        counts: vec![(set, 4)],
        tiles: vec![
            (set, 1, tile(0xff00_0001)),
            (set, 2, tile(0xff00_0002)),
            (set, 3, tile(0xff00_0003)),
        ],
        ..TilesSet::default()
    };
    h.commit(&mut s, ph2d_pixel_core::Op::Tiles(Box::new(op)))
        .unwrap();
    for ((c, r), id) in [((0, 0), 1), ((1, 0), 2), ((0, 1), 3)] {
        let t = TileRef::new(id, false, false, false);
        let op = carimba(&s, l, f, &[(c, r)], t).unwrap();
        h.commit(&mut s, op).unwrap();
    }
    assert!(derived_is_consistent(&s));
    let c = s.entry((l, f)).unwrap();
    let px = |x: i32, y: i32| {
        let i = ((y - c.bbox.y) as usize * c.bbox.w as usize + (x - c.bbox.x) as usize) * 4;
        u32::from_le_bytes(c.data[i..i + 4].try_into().unwrap())
    };
    // (0,1) e (1,0) sobrepõem-se na banda 12..16, à direita: a coluna 1 ganha, ainda que a linha seja menor.
    assert_eq!(px(20, 14), 0xff00_0002);
    // Na mesma coluna, a linha de baixo ganha: (0,1) sobre (0,0).
    assert_eq!(px(10, 14), 0xff00_0003);
}

#[test]
fn iso_e_hex_gravam_reabrem_e_voltam_ao_byte() {
    for forma in [Forma::Iso, Forma::Hex, Forma::HexFlat] {
        let (mut s, mut h, l, f, set) = doc(forma, 16, 16);
        let op = lei::novo_tile(&s, set).unwrap();
        h.commit(&mut s, op).unwrap();
        let tile = PRETO.to_le_bytes().repeat(256);
        let op = TilesSet {
            tiles: vec![(set, 1, tile)],
            ..TilesSet::default()
        };
        h.commit(&mut s, ph2d_pixel_core::Op::Tiles(Box::new(op)))
            .unwrap();
        let antes = s.clone();
        let op = carimba(
            &s,
            l,
            f,
            &[(2, 3), (3, 3), (-1, 1)],
            TileRef::new(1, true, false, false),
        )
        .unwrap();
        h.commit(&mut s, op).unwrap();
        assert!(derived_is_consistent(&s), "{forma:?}");
        // O canvas desloca a grelha por pixeis, inclusive um múltiplo do tile: o índice fica.
        let g = s.entry((l, f)).unwrap().tiles.clone().unwrap();
        assert_eq!(g.ancora(forma, 16, 16).0, (-1, 1), "{forma:?}");
        let mapa = Mapa::de(&s, l, f, set_de(&s, set));
        assert_eq!(mapa.get(2, 3).id(), 1);
        let bytes = write_library(&BTreeMap::from([(1, s.clone())]));
        let lido = read_library(&bytes).unwrap().remove(&1).unwrap();
        assert_eq!(lido.tileset(set).unwrap().forma, forma);
        assert_eq!(lido.entry((l, f)), s.entry((l, f)), "{forma:?}");
        assert!(h.undo(&mut s));
        assert_eq!(s.entry((l, f)), antes.entry((l, f)), "{forma:?}");
    }
}

/// Um terreno novo sem guia seria invisível ao pintar: as 47 formas nascem distintas.
#[test]
fn o_guia_tem_47_formas_distintas_nas_duas_grelhas() {
    for (forma, w, h) in [
        (Forma::Square, 16, 16),
        (Forma::Iso, 32, 16),
        (Forma::Square, 8, 8),
    ] {
        let (mut s, mut hist, _, _, set) = doc(forma, w, h);
        let op = terreno::novo_terreno_com_guia(&s, set, "Grass", PRETO).unwrap();
        hist.commit(&mut s, op).unwrap();
        let t = set_de(&s, set);
        assert_eq!(t.tiles.len() as u32, 1 + TERRAIN_TILES);
        let mut vistos: Vec<&Vec<u8>> = t.tiles[1..].iter().collect();
        vistos.sort();
        vistos.dedup();
        assert_eq!(vistos.len(), TERRAIN_TILES as usize, "{forma:?} {w}x{h}");
        assert!(t.tiles[0].iter().all(|b| *b == 0), "o vazio continua vazio");
    }
    // Em iso o guia não sai do losango: os cantos da caixa ficam vazios.
    assert!(!terreno::guia_cheio(Forma::Iso, 0xff, 0, 0, 32, 16));
    assert!(terreno::guia_cheio(Forma::Iso, 0xff, 16, 8, 32, 16));
    assert!(terreno::guia_cheio(Forma::Square, 0xff, 0, 0, 16, 16));
    assert!(!terreno::guia_cheio(Forma::Square, 0, 0, 0, 16, 16));
    // Cada canto no seu lugar: um guia espelhado também tem 47 formas distintas.
    use terreno::{NE, NW, SE, SW, guia_cheio as g};
    for (lado, cantos) in [
        (NE, [(15, 0), (0, 0), (15, 15), (0, 15)]),
        (SE, [(15, 15), (0, 0), (15, 0), (0, 15)]),
        (SW, [(0, 15), (0, 0), (15, 15), (15, 0)]),
        (NW, [(0, 0), (15, 0), (15, 15), (0, 15)]),
    ] {
        let [sim, a, b, c] = cantos;
        assert!(g(Forma::Square, lado, sim.0, sim.1, 16, 16), "{lado}");
        for n in [a, b, c] {
            assert!(!g(Forma::Square, lado, n.0, n.1, 16, 16), "{lado} {n:?}");
        }
    }
    // Em iso, (1,-1) é o canto da direita e (-1,-1) o de cima (M2iso).
    assert!(g(Forma::Iso, NE, 29, 8, 32, 16));
    assert!(!g(Forma::Iso, NE, 16, 1, 32, 16));
    assert!(g(Forma::Iso, NW, 16, 1, 32, 16));
    assert!(!g(Forma::Iso, NW, 29, 8, 32, 16));
}

/// O terreno em iso resolve pela mesma regra: o M2iso corrido; hex recusa, com a frase.
#[test]
fn o_terreno_corre_em_iso_e_recusa_em_hex() {
    let (mut s, mut h, l, f, set) = doc(Forma::Iso, 32, 16);
    let op = terreno::novo_terreno_com_guia(&s, set, "Grass", PRETO).unwrap();
    h.commit(&mut s, op).unwrap();
    let celulas = celulas_reais();
    assert!(celulas.len() >= 200, "{}", celulas.len());
    let op = terreno::pinta(&s, l, f, &celulas, Some(0)).unwrap();
    h.commit(&mut s, op).unwrap();
    assert!(derived_is_consistent(&s));
    let first = set_de(&s, set).terrains[0].first;
    let m = Mapa::de(&s, l, f, set_de(&s, set));
    for &(c, r) in &celulas {
        let mut mascara = 0;
        for (b, (dx, dy)) in terreno::VIZINHOS.iter().enumerate() {
            if celulas.contains(&(c + dx, r + dy)) {
                mascara |= 1 << b;
            }
        }
        assert_eq!(
            m.get(c, r).id(),
            first + terreno::casa(mascara),
            "({c},{r})"
        );
    }
    let (s2, _, l2, f2, set2) = doc(Forma::Hex, 16, 16);
    assert!(terreno::novo_terreno(&s2, set2, "x").is_none());
    assert!(terreno::pinta(&s2, l2, f2, &[(0, 0)], Some(0)).is_none());
    let mut mao = Mao {
        alvo: Alvo::Terreno,
        ..Mao::default()
    };
    assert_eq!(
        marca_de(&mao, set_de(&s2, set2), false),
        Err("pixel.tiles.refuse.terrain_shape")
    );
    let t = set_de(&s, set);
    assert_eq!(marca_de(&mao, t, true), Ok(Marca::Terreno(None)));
    assert_eq!(marca_de(&mao, t, false), Ok(Marca::Terreno(Some(0))));
    mao.terreno = 3;
    assert_eq!(marca_de(&mao, t, false), Err("pixel.tiles.no_terrain"));
    pega(&mut mao, t, TileRef::new(first + 5, false, false, false));
    assert_eq!(mao.terreno, 0, "o conta-gotas fica com o terreno da célula");
}

/// A camada do gabarito traz as 47 formas à vista, cresce o canvas e sai num só desfazer.
#[test]
fn o_gabarito_em_camada_cresce_o_canvas_e_vale_um_passo() {
    for (forma, w, h) in [(Forma::Square, 16, 16), (Forma::Iso, 32, 16)] {
        let (mut s, mut hist, l, f, set) = doc(forma, w, h);
        let op = terreno::novo_terreno_com_guia(&s, set, "Grass", PRETO).unwrap();
        hist.commit(&mut s, op).unwrap();
        let antes = s.clone();
        let (op, nova) = terreno::gabarito(&mut s, l, f, set, 0, "Grass template").unwrap();
        hist.commit(&mut s, op).unwrap();
        assert!(derived_is_consistent(&s));
        assert_eq!(s.layer(nova).unwrap().name, "Grass template");
        let t = set_de(&s, set);
        let m = Mapa::de(&s, nova, f, t);
        let mut achados = 0;
        for r in 0..6 {
            for c in -8..16 {
                if m.get(c, r).id() != 0 {
                    achados += 1;
                    let (x, y) = m.origem(c, r);
                    assert!(x >= 0 && y >= 0, "{forma:?} ({c},{r}) em ({x},{y})");
                    assert!(
                        x + i32::from(w) <= s.width() as i32
                            && y + i32::from(h) <= s.height() as i32
                    );
                }
            }
        }
        assert_eq!(achados, TERRAIN_TILES);
        // Pintar nele edita o tile do gabarito no lugar, em qualquer modo.
        let n = set_de(&s, set).tiles.len();
        let (x, y) = m.origem(if forma == Forma::Iso { 5 } else { 0 }, 0);
        let (mx, my) = (x + i32::from(w) / 2, y + i32::from(h) / 2);
        let op = aplica(&s, nova, f, &[(mx, my, 0xff00_00ff)], Modo::Stack).unwrap();
        hist.commit(&mut s, op).unwrap();
        assert_eq!(
            set_de(&s, set).tiles.len(),
            n,
            "o gabarito edita-se no lugar"
        );
        assert!(hist.undo(&mut s));
        assert!(hist.undo(&mut s));
        assert_eq!((s.width(), s.height()), (antes.width(), antes.height()));
        assert!(s.layer(nova).is_none());
    }
}

#[test]
fn apagar_um_terreno_renumera_e_volta_num_passo() {
    let (mut s, mut h, l, f, set) = doc(Forma::Square, 8, 8);
    for n in ["A", "B"] {
        let op = terreno::novo_terreno(&s, set, n).unwrap();
        h.commit(&mut s, op).unwrap();
    }
    let segundo = set_de(&s, set).terrains[1].first;
    let op = terreno::pinta(&s, l, f, &[(0, 0)], Some(1)).unwrap();
    h.commit(&mut s, op).unwrap();
    let op = terreno::pinta(&s, l, f, &[(5, 5)], Some(0)).unwrap();
    h.commit(&mut s, op).unwrap();
    let antes = s.clone();
    let op = lei::apaga_terreno(&s, set, 0).unwrap();
    h.commit(&mut s, op).unwrap();
    assert!(derived_is_consistent(&s));
    let t = set_de(&s, set);
    assert_eq!(t.terrains.len(), 1);
    assert_eq!(t.terrains[0].first, segundo - TERRAIN_TILES);
    let m = Mapa::de(&s, l, f, t);
    assert_eq!(m.get(5, 5).id(), 0, "a célula do terreno apagado esvazia");
    assert_eq!(m.get(0, 0).id(), segundo - TERRAIN_TILES + terreno::casa(0));
    assert!(h.undo(&mut s));
    assert_eq!(s.tileset(set), antes.tileset(set));
    assert_eq!(s.entry((l, f)), antes.entry((l, f)));
}

/// Trocar a forma mantém índices e arte, e o caminho inverso volta ao byte, mesmo com a grelha deslocada.
#[test]
fn mudar_a_forma_mantem_o_indice_e_volta_ao_byte() {
    let (mut s, mut h, l, f, set) = doc(Forma::Square, 16, 16);
    let op = lei::novo_tile(&s, set).unwrap();
    h.commit(&mut s, op).unwrap();
    let op = carimba(
        &s,
        l,
        f,
        &[(1, 2), (2, 2)],
        TileRef::new(1, false, false, false),
    )
    .unwrap();
    h.commit(&mut s, op).unwrap();
    let antes = s.entry((l, f)).cloned();
    for forma in [Forma::Iso, Forma::Hex, Forma::HexFlat] {
        let op = lei::muda_forma(&s, set, forma).unwrap();
        h.commit(&mut s, op).unwrap();
        assert!(derived_is_consistent(&s));
        let m = Mapa::de(&s, l, f, set_de(&s, set));
        assert_eq!((m.get(1, 2).id(), m.get(2, 2).id()), (1, 1), "{forma:?}");
        assert!(h.undo(&mut s));
    }
    assert_eq!(s.entry((l, f)).cloned(), antes);
    // Com terreno, hex recusa.
    let op = terreno::novo_terreno(&s, set, "A").unwrap();
    h.commit(&mut s, op).unwrap();
    assert!(lei::muda_forma(&s, set, Forma::Hex).is_none());
    assert!(lei::muda_forma(&s, set, Forma::Iso).is_some());
}

/// O lápis em Pixels numa grelha iso escreve na célula que o clique acerta, e o modo vale.
#[test]
fn o_lapis_em_pixels_segue_a_forma() {
    let (mut s, mut h, l, f, set) = doc(Forma::Iso, 16, 8);
    let op = lei::novo_tile(&s, set).unwrap();
    h.commit(&mut s, op).unwrap();
    let op = carimba(
        &s,
        l,
        f,
        &[(2, 0), (3, 0)],
        TileRef::new(1, false, false, false),
    )
    .unwrap();
    h.commit(&mut s, op).unwrap();
    let m = Mapa::de(&s, l, f, set_de(&s, set));
    let (x, y) = m.origem(2, 0);
    let (cx, cy) = (x + 8, y + 4);
    assert_eq!(m.celula(cx, cy), (2, 0));
    let op = aplica(&s, l, f, &[(cx, cy, PRETO)], Modo::Manual).unwrap();
    h.commit(&mut s, op).unwrap();
    assert!(derived_is_consistent(&s));
    let t = set_de(&s, set);
    assert_ne!(t.tiles[1], t.tiles[0], "o tile foi editado");
    assert_eq!(t.tiles.len(), 2, "Manual não cria tile");
    // A outra cópia mostra a mesma arte: todas as cópias mudaram.
    let c = s.entry((l, f)).unwrap();
    let (ox, oy) = m.origem(3, 0);
    let i =
        (((oy + 4) - c.bbox.y) as usize * c.bbox.w as usize + ((ox + 8) - c.bbox.x) as usize) * 4;
    assert_eq!(
        u32::from_le_bytes(c.data[i..i + 4].try_into().unwrap()),
        PRETO
    );
}

/// Um arrasto em iso e hex não salta células: cada passo é uma vizinha, e não há repetição seguida.
#[test]
fn o_arrasto_em_iso_e_hex_nao_salta() {
    for forma in [Forma::Iso, Forma::Hex, Forma::HexFlat] {
        let (s, _, l, f, _) = doc(forma, 16, 16);
        let g = Carimbo {
            layer: l,
            frame: f,
            marca: Marca::Tile(TileRef::EMPTY),
            celulas: vec![(0, 0)],
            passo: false,
        };
        let c = g.ate(&s, (110, 70)).unwrap();
        assert_eq!(
            c.last().copied(),
            Some(forma.celula(110, 70, 16, 16)),
            "{forma:?}"
        );
        for par in c.windows(2) {
            let (a, b) = (par[0], par[1]);
            assert_ne!(a, b);
            assert!(
                (a.0 - b.0).abs() <= 1 && (a.1 - b.1).abs() <= 1,
                "{forma:?}: {a:?} → {b:?}"
            );
        }
    }
}

/// Em quadrado a forma não muda nada: `expand` e a caixa continuam a ser os da grelha simples.
#[test]
fn a_grelha_quadrada_nao_mudou() {
    let (s, _, _, _, set) = doc(Forma::Square, 4, 4);
    let g = TileGrid {
        x: 4,
        y: 8,
        w: 2,
        h: 1,
        tiles: vec![TileRef::EMPTY; 2],
        ..TileGrid::default()
    };
    let t = set_de(&s, set);
    let r = g.caixa(t);
    assert_eq!((r.x, r.y, r.w, r.h), (4, 8, 8, 4));
    assert_eq!(expand(&g, t, 4).len(), 8 * 4 * 4);
    assert_eq!(g.ancora(Forma::Square, 4, 4), ((1, 2), (0, 0)));
}

/// A cena 22 (dono 07/11, «pouco compreensível»): as faixas do desenho das 47 formas casam nas duas
/// emendas de todo o mapa, ou cada célula mostraria um degrau na praia.
#[test]
fn the_island_art_of_scene_22_matches_across_every_seam() {
    use ph2d_pixel_tiles::exemplo::{ILHAS, LADO, celulas_ilhas, terreno_ilhas};
    assert!(ILHAS.iter().all(|l| l.len() == ILHAS[0].len()));
    let (mut s, mut h, l, f, set) = doc(Forma::Square, LADO, LADO);
    let op = terreno_ilhas(&s, set, "Island").unwrap();
    h.commit(&mut s, op).unwrap();
    let celulas = celulas_ilhas();
    let op = terreno::pinta(&s, l, f, &celulas, Some(0)).unwrap();
    h.commit(&mut s, op).unwrap();
    let t = set_de(&s, set);
    let m = Mapa::de(&s, l, f, t);
    let (lado, bpp) = (usize::from(LADO), s.mode().bpp());
    let cor =
        |i: usize| s.value_of_rgba(*s.palette().get(i).unwrap()).to_le_bytes()[..bpp].to_vec();
    let vazio = t.tiles[0][..bpp].to_vec();
    let (espuma, areia, areia2, escura) = (cor(20), cor(7), cor(6), cor(12));
    // 0 água · 1 espuma · 2 areia · 3 relva escura · 4 relva.
    let faixa = |id: u32, x: usize, y: usize| {
        let p = (y * lado + x) * bpp;
        let v = &t.tiles[id as usize][p..p + bpp];
        match () {
            () if v == vazio.as_slice() => 0,
            () if v == espuma.as_slice() => 1,
            () if v == areia.as_slice() || v == areia2.as_slice() => 2,
            () if v == escura.as_slice() => 3,
            () => 4,
        }
    };
    let mut emendas = 0;
    for &(c, r) in &celulas {
        let id = m.get(c, r).id();
        assert_eq!(
            faixa(id, lado / 2, lado / 2),
            4,
            "({c},{r}): sem relva no meio"
        );
        if celulas.contains(&(c + 1, r)) {
            let vizinha = m.get(c + 1, r).id();
            for y in 0..lado {
                assert_eq!(
                    faixa(id, lado - 1, y),
                    faixa(vizinha, 0, y),
                    "({c},{r}) y={y}"
                );
                emendas += 1;
            }
        }
        if celulas.contains(&(c, r + 1)) {
            let vizinha = m.get(c, r + 1).id();
            for x in 0..lado {
                assert_eq!(
                    faixa(id, x, lado - 1),
                    faixa(vizinha, x, 0),
                    "({c},{r}) x={x}"
                );
                emendas += 1;
            }
        }
    }
    assert!(emendas > 1000, "{emendas}");
}
