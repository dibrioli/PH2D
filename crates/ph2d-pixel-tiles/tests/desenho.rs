//! Manual, Auto e Stack: o Pixelorama corrido (M1), e o que ele não mediu, por propriedade.

use ph2d_pixel_core::tiles::new_tilemap_layer;
use ph2d_pixel_core::{
    Background, Cel, ColorMode, FrameId, History, LayerId, NewSprite, Op, Sprite, TERRAIN_TILES,
    TileGrid, TileRef, TilesSet, TilesetId, derived_is_consistent,
};
use ph2d_pixel_tiles::desenho::{Modo, aplica, no_tile, usos};
use ph2d_pixel_tiles::mapa::Mapa;
use ph2d_pixel_tiles::terreno;

const T: u16 = 8;
const AZUL: u32 = u32::from_le_bytes([0, 0, 255, 255]);
const VERMELHO: u32 = u32::from_le_bytes([255, 0, 0, 255]);
const VERDE: u32 = u32::from_le_bytes([0, 255, 0, 255]);

struct Doc {
    s: Sprite,
    h: History,
    l: LayerId,
    f: FrameId,
    set: TilesetId,
}

impl Doc {
    fn novo(lado: u32, tile: u16) -> Self {
        let mut s = Sprite::new(NewSprite {
            width: lado,
            height: lado,
            mode: ColorMode::Rgba,
            background: Background::Transparent,
        })
        .unwrap();
        let mut h = History::default();
        let (acima, f) = (s.layers()[0].id, s.frames()[0].id);
        let (op, l, set) = new_tilemap_layer(&mut s, acima, tile, tile).unwrap();
        h.commit(&mut s, op).unwrap();
        Self { s, h, l, f, set }
    }

    fn tile_com(&self, px: &[(u32, u32, u32)]) -> Vec<u8> {
        let t = self.s.tileset(self.set).unwrap();
        let w = u32::from(t.tile_w);
        let mut out = t.tiles[0].clone();
        for &(x, y, v) in px {
            let i = (y * w + x) as usize * 4;
            out[i..i + 4].copy_from_slice(&v.to_le_bytes());
        }
        out
    }

    /// Põe `tiles` depois do 0 e `mapa` no cel do quadro `f`.
    fn com_tiles(&mut self, tiles: Vec<Vec<u8>>, f: FrameId, mapa: TileGrid) {
        let n = self.s.tileset(self.set).unwrap().tiles.len() as u32;
        let t = TilesSet {
            counts: vec![(self.set, n + tiles.len() as u32)],
            tiles: (n..).zip(tiles).map(|(i, p)| (self.set, i, p)).collect(),
            cels: vec![((self.l, f), Some(Cel::tilemap(mapa)))],
            ..TilesSet::default()
        };
        self.h.commit(&mut self.s, Op::Tiles(Box::new(t))).unwrap();
    }

    /// O mapa do M1: 4×4; tile1 vermelho em (0,0), (1,0), (2,0); tile2 verde em (0,1).
    fn m1() -> Self {
        let mut d = Self::novo(32, T);
        let (a, b) = (
            d.tile_com(&[(0, 0, VERMELHO)]),
            d.tile_com(&[(1, 1, VERDE)]),
        );
        let r = |i| TileRef::new(i, false, false, false);
        let g = TileGrid {
            x: 0,
            y: 0,
            w: 3,
            h: 2,
            tiles: vec![r(1), r(1), r(1), r(2), r(0), r(0)],
            ..TileGrid::default()
        };
        d.com_tiles(vec![a, b], d.f, g);
        d
    }

    fn mapa(&self) -> Mapa {
        Mapa::de(&self.s, self.l, self.f, self.s.tileset(self.set).unwrap())
    }

    fn id(&self, c: i32, r: i32) -> u32 {
        self.mapa().get(c, r).id()
    }

    fn n_tiles(&self) -> usize {
        self.s.tileset(self.set).unwrap().tiles.len()
    }

    /// Aplica o traço; devolve se houve op. Confere a lei do derivado e o desfazer ao byte.
    fn traco(&mut self, e: &[(i32, i32, u32)], m: Modo) -> bool {
        let Some(op) = aplica(&self.s, self.l, self.f, e, m) else {
            return false;
        };
        let antes = self.s.clone();
        self.h.commit(&mut self.s, op).unwrap();
        assert!(derived_is_consistent(&self.s));
        let depois = self.s.clone();
        assert!(self.h.undo(&mut self.s));
        assert_eq!(self.s, antes, "desfazer volta ao byte");
        assert!(self.h.redo(&mut self.s));
        assert_eq!(self.s, depois, "refazer idem");
        true
    }

    fn visto(&self, x: i32, y: i32) -> u32 {
        self.s.value_at((self.l, self.f), x, y)
    }
}

fn px(c: i32, r: i32, x: i32, y: i32, v: u32) -> (i32, i32, u32) {
    (c * i32::from(T) + x, r * i32::from(T) + y, v)
}

/// Os cenários da fixtura (o cabeçalho dela os define): `(célula, píxel, valor)`, um traço cada.
fn cenario(nome: &str) -> Vec<Vec<(i32, i32, u32)>> {
    match nome {
        "vazia" => vec![vec![px(3, 3, 2, 2, AZUL)]],
        "uma_vez" => vec![vec![px(0, 1, 5, 5, AZUL)]],
        "varias" => vec![vec![px(1, 0, 5, 5, AZUL)]],
        "apaga" => vec![vec![px(0, 1, 1, 1, 0)]],
        "apaga_varias" => vec![vec![px(1, 0, 0, 0, 0)]],
        "igual_a_outro" => vec![vec![px(3, 3, 0, 0, AZUL)]],
        "dois_iguais" => vec![vec![px(3, 3, 2, 2, AZUL)], vec![px(3, 2, 2, 2, AZUL)]],
        o => panic!("cenário sem receita: {o}"),
    }
}

/// O M1 corrido, cenário a cenário. **Por escolha nossa, Eraser em Manual e em Stack não seguem:** a
/// sonda viu «nada muda», sem saber se é a lei ou o toque dela, e a lei do dono é a de baixo.
#[test]
fn os_tres_modos_batem_com_o_pixelorama_corrido() {
    let txt = include_str!("fixtures/oracle_pixelorama_modos.json");
    let v: serde_json::Value = serde_json::from_str(txt).unwrap();
    let mut conferidos = 0;
    for l in v["linhas"].as_array().unwrap() {
        let tag = l["tag"].as_str().unwrap();
        let [modo, nome, fase] = tag.split('/').collect::<Vec<_>>()[..] else {
            panic!("{tag}")
        };
        if fase != "depois" {
            continue;
        }
        if modo != "auto" && nome.starts_with("apaga") {
            continue;
        }
        let m = match modo {
            "manual" => Modo::Manual,
            "auto" => Modo::Auto,
            _ => Modo::Stack,
        };
        let mut d = Doc::m1();
        for e in cenario(nome) {
            d.traco(&e, m);
        }
        for (k, c) in l["cells"].as_object().unwrap() {
            let (x, y) = k.split_once(',').unwrap();
            let (x, y): (i32, i32) = (x.parse().unwrap(), y.parse().unwrap());
            assert_eq!(
                i64::from(d.id(x, y)),
                c[0].as_i64().unwrap(),
                "{tag} célula {k}"
            );
            assert_eq!(
                d.mapa().get(x, y).0 & !TileRef::ID_MASK,
                0,
                "{tag} sem flips"
            );
        }
        let set = d.s.tileset(d.set).unwrap();
        let tiles = l["tiles"].as_array().unwrap();
        assert_eq!(set.tiles.len(), tiles.len(), "{tag} número de tiles");
        let n_usos = usos(&d.s, set);
        for (i, t) in tiles.iter().enumerate() {
            let (px4, _) = set.tiles[i].as_chunks::<4>();
            let cheios = px4.iter().filter(|p| p[3] != 0).count();
            assert_eq!(cheios as i64, t["px"].as_i64().unwrap(), "{tag} tile {i}");
            if i != 0 {
                assert_eq!(
                    i64::from(n_usos[i]),
                    t["used"].as_i64().unwrap(),
                    "{tag} usos {i}"
                );
            }
        }
        conferidos += 1;
    }
    assert_eq!(
        conferidos, 17,
        "3 modos × 7 cenários, menos os 4 de Eraser fora do Auto"
    );
}

/// O inverso de `place` e a ordem D, depois X e Y, em todas as combinações e num tile assimétrico.
#[test]
fn flips_xyd_o_inverso_desfaz_o_place_nas_oito_combinacoes() {
    for b in 0_u8..8 {
        let t = TileRef::new(1, b & 1 != 0, b & 2 != 0, b & 4 != 0);
        for sy in 0..4 {
            for sx in 0..4 {
                let (dx, dy) = t.place(sx, sy, 4, 4);
                assert_eq!(no_tile(t, dx, dy, 4, 4), (sx, sy), "flips {b}");
            }
        }
    }
    // A combinação que distingue a ordem: com X e D, (1, 0) do tile vai a (3, 1) e volta.
    let t = TileRef::new(1, true, false, true);
    assert_eq!(t.place(1, 0, 4, 4), (3, 1));
    assert_eq!(no_tile(t, 3, 1, 4, 4), (1, 0));
}

/// O traço cai onde a célula mostra, com qualquer flip, nos três modos.
#[test]
fn o_traco_cai_onde_a_celula_mostra_em_todos_os_flips() {
    for m in [Modo::Manual, Modo::Auto, Modo::Stack] {
        for b in 0_u8..8 {
            let mut d = Doc::novo(16, 4);
            let semente: Vec<(u32, u32, u32)> = (0..16)
                .map(|i| (i % 4, i / 4, u32::from_le_bytes([i as u8 + 1, 7, 7, 255])))
                .collect();
            let tile = d.tile_com(&semente);
            let g = TileGrid {
                x: 4,
                y: 4,
                w: 1,
                h: 1,
                tiles: vec![TileRef::new(1, b & 1 != 0, b & 2 != 0, b & 4 != 0)],
                ..TileGrid::default()
            };
            d.com_tiles(vec![tile], d.f, g);
            for (x, y) in [(4, 4), (5, 4), (7, 6), (6, 7)] {
                let antes: Vec<u32> = (0..16)
                    .flat_map(|yy| (0..16).map(move |xx| (xx, yy)))
                    .map(|(xx, yy)| d.visto(xx, yy))
                    .collect();
                assert!(d.traco(&[(x, y, AZUL)], m), "{m:?} flips {b}");
                let depois = |xx: i32, yy: i32| {
                    if (xx, yy) == (x, y) {
                        AZUL
                    } else {
                        antes[(yy * 16 + xx) as usize]
                    }
                };
                for yy in 0..16 {
                    for xx in 0..16 {
                        assert_eq!(
                            d.visto(xx, yy),
                            depois(xx, yy),
                            "{m:?} flips {b} em {xx},{yy}"
                        );
                    }
                }
            }
        }
    }
}

/// Um traço por várias células e vários píxeis: em Auto e Stack, vê-se o que se escreveu e nada mais.
#[test]
fn um_traco_por_varias_celulas_mostra_o_que_escreveu() {
    for m in [Modo::Auto, Modo::Stack] {
        let mut d = Doc::m1();
        let mut e = Vec::new();
        for i in 0..24 {
            e.push((i, 3, AZUL));
            e.push((i, 4, u32::from_le_bytes([i as u8, 0, 9, 255])));
            e.push((3, i, 0));
        }
        let antes: Vec<u32> = (0..32 * 32).map(|i| d.visto(i % 32, i / 32)).collect();
        assert!(d.traco(&e, m));
        for y in 0..32 {
            for x in 0..32 {
                let escrito = e.iter().rev().find(|w| (w.0, w.1) == (x, y)).map(|w| w.2);
                let quer = escrito.unwrap_or(antes[(y * 32 + x) as usize]);
                assert_eq!(d.visto(x, y), quer, "{m:?} em {x},{y}");
            }
        }
    }
}

/// Fora do canvas não se escreve, e a última escrita da mesma célula vence.
#[test]
fn fora_do_canvas_nao_conta_e_a_ultima_escrita_vence() {
    let mut d = Doc::m1();
    assert!(!d.traco(&[(-1, 0, AZUL), (40, 40, AZUL)], Modo::Auto));
    assert!(d.traco(&[(30, 30, AZUL), (30, 30, VERDE)], Modo::Auto));
    assert_eq!(d.visto(30, 30), VERDE);
}

#[test]
fn manual_edita_todas_as_copias_e_nao_faz_nada_em_vazia() {
    let mut d = Doc::m1();
    let n = d.n_tiles();
    assert!(d.traco(&[px(1, 0, 5, 5, AZUL)], Modo::Manual));
    for c in 0..3 {
        assert_eq!(d.visto(c * 8 + 5, 5), AZUL, "cópia {c}");
    }
    assert_eq!(d.n_tiles(), n);
    assert!(!d.traco(&[px(3, 3, 2, 2, AZUL)], Modo::Manual));
    // Sem dedupe: apagar em Manual deixa o tile vazio no lugar.
    assert!(d.traco(&[px(0, 1, 1, 1, 0)], Modo::Manual));
    assert_eq!(d.id(0, 1), 2);
    assert_eq!(d.n_tiles(), n);
}

#[test]
fn apagar_ate_vazio_e_dedupe_com_o_tile_zero() {
    let mut d = Doc::m1();
    assert!(d.traco(&[px(1, 0, 0, 0, 0)], Modo::Stack));
    assert_eq!(d.mapa().get(1, 0), TileRef::EMPTY);
    assert_eq!(d.n_tiles(), 3, "o Stack não tira nada");
    assert!(d.traco(&[px(0, 1, 1, 1, 0)], Modo::Stack));
    assert_eq!(d.mapa().get(0, 1), TileRef::EMPTY);
    assert_eq!(d.n_tiles(), 3, "o tile 2 fica, com 0 usos");
}

/// Apagar a célula sem deixar nada: o cel sai, e desfazer repõe.
#[test]
fn apagar_o_ultimo_tile_tira_o_cel_e_desfazer_o_repoe() {
    let mut d = Doc::novo(16, 8);
    let t = d.tile_com(&[(0, 0, AZUL)]);
    let g = TileGrid {
        x: 0,
        y: 0,
        w: 1,
        h: 1,
        tiles: vec![TileRef::new(1, false, false, false)],
        ..TileGrid::default()
    };
    d.com_tiles(vec![t], d.f, g);
    assert!(d.traco(&[(0, 0, 0)], Modo::Auto));
    assert!(d.s.entry((d.l, d.f)).is_none());
    assert_eq!(d.n_tiles(), 1);
}

#[test]
fn o_resultado_igual_a_um_tile_existente_o_reutiliza() {
    let mut d = Doc::m1();
    // Vermelho em (0,0) numa vazia: a imagem exata do tile 1.
    for m in [Modo::Auto, Modo::Stack] {
        let mut c = Doc::m1();
        assert!(c.traco(&[px(3, 3, 0, 0, VERMELHO)], m));
        assert_eq!(c.id(3, 3), 1);
        assert_eq!(c.n_tiles(), 3, "{m:?}");
    }
    // Duas células novas no mesmo traço, a mesma imagem: um só tile.
    assert!(d.traco(&[px(3, 3, 2, 2, AZUL), px(3, 2, 2, 2, AZUL)], Modo::Auto));
    assert_eq!(d.id(3, 3), 3);
    assert_eq!(d.id(3, 2), 3);
    assert_eq!(d.n_tiles(), 4);
}

/// Os usos somam todos os cels donos de todas as camadas com o tileset; um ligado não conta.
#[test]
fn auto_conta_os_usos_em_todo_o_sprite_e_ligado_nao_conta() {
    use ph2d_pixel_core::frames::{NewFrame, new_frame};
    // Copiado: dois usos. Editar num quadro faz nascer tile, e o outro quadro não muda.
    let mut d = Doc::m1();
    let (op, f2) = new_frame(&mut d.s, d.f, NewFrame::Copy).unwrap();
    d.h.commit(&mut d.s, op).unwrap();
    assert!(d.traco(&[px(0, 1, 5, 5, AZUL)], Modo::Auto));
    assert_eq!(d.id(0, 1), 3);
    assert_eq!(
        d.s.value_at((d.l, f2), 5, 13),
        0,
        "o outro quadro não mudou"
    );
    // Ligado: um uso só. Edita no lugar, e o quadro ligado vê.
    let mut d = Doc::m1();
    let (op, f2) = new_frame(&mut d.s, d.f, NewFrame::Linked).unwrap();
    d.h.commit(&mut d.s, op).unwrap();
    assert!(d.traco(&[px(0, 1, 5, 5, AZUL)], Modo::Auto));
    assert_eq!(d.id(0, 1), 2, "no lugar");
    assert_eq!(d.s.value_at((d.l, f2), 5, 13), AZUL);
    // Outra camada com o mesmo tileset conta.
    let mut d = Doc::m1();
    let (op, _copia) = ph2d_pixel_core::layers::duplicate(&mut d.s, d.l).unwrap();
    d.h.commit(&mut d.s, op).unwrap();
    assert!(d.traco(&[px(0, 1, 5, 5, AZUL)], Modo::Auto));
    assert_eq!(d.id(0, 1), 3);
}

/// Um tile de gabarito é do artista: em qualquer modo edita no lugar e nunca sai.
#[test]
fn o_tile_de_terreno_comporta_se_como_manual() {
    for m in [Modo::Manual, Modo::Auto, Modo::Stack] {
        let mut d = Doc::novo(32, 8);
        let op = terreno::novo_terreno(&d.s, d.set, "Grass").unwrap();
        d.h.commit(&mut d.s, op).unwrap();
        let first = d.s.tileset(d.set).unwrap().terrains[0].first;
        let casa = first + 3;
        let r = TileRef::new(casa, false, false, false);
        let g = TileGrid {
            x: 0,
            y: 0,
            w: 2,
            h: 1,
            tiles: vec![r, r],
            ..TileGrid::default()
        };
        let t = TilesSet {
            cels: vec![((d.l, d.f), Some(Cel::tilemap(g)))],
            ..TilesSet::default()
        };
        d.h.commit(&mut d.s, Op::Tiles(Box::new(t))).unwrap();
        let n = d.n_tiles();
        assert!(d.traco(&[(1, 1, AZUL)], m));
        assert_eq!(
            (d.id(0, 0), d.n_tiles()),
            (casa, n),
            "{m:?}: no lugar, sem tile novo"
        );
        assert_eq!(d.visto(9, 1), AZUL, "{m:?}: a outra cópia mudou");
        assert!(d.traco(&[(1, 1, 0)], m));
        assert_eq!(
            (d.id(0, 0), d.n_tiles()),
            (casa, n),
            "{m:?}: apagar não o tira"
        );
        assert_eq!(n as u32, 1 + TERRAIN_TILES);
    }
}

/// Uma célula livre nunca entra num gabarito por acaso: viraria terreno no próximo resolve.
#[test]
fn o_dedupe_nao_entra_num_gabarito() {
    let mut d = Doc::novo(32, 8);
    let op = terreno::novo_terreno(&d.s, d.set, "Grass").unwrap();
    d.h.commit(&mut d.s, op).unwrap();
    let first = d.s.tileset(d.set).unwrap().terrains[0].first;
    let r = TileRef::new(first, false, false, false);
    let g = TileGrid {
        x: 0,
        y: 0,
        w: 1,
        h: 1,
        tiles: vec![r],
        ..TileGrid::default()
    };
    let t = TilesSet {
        cels: vec![((d.l, d.f), Some(Cel::tilemap(g)))],
        ..TilesSet::default()
    };
    d.h.commit(&mut d.s, Op::Tiles(Box::new(t))).unwrap();
    assert!(
        d.traco(&[(1, 1, AZUL)], Modo::Manual),
        "o gabarito ganha um desenho"
    );
    assert!(
        d.traco(&[(9, 1, AZUL)], Modo::Auto),
        "a mesma imagem, numa célula livre"
    );
    assert_eq!(d.id(1, 0), 1 + TERRAIN_TILES);
}

/// Sair do conjunto renumera todos os mapas de todos os cels, leva os flips e move o gabarito.
#[test]
fn a_renumeracao_alcanca_todos_os_cels_no_mesmo_op() {
    use ph2d_pixel_core::frames::{NewFrame, new_frame};
    let mut d = Doc::m1();
    let flip = TileRef::new(2, true, false, false);
    let (a, b) = (
        d.tile_com(&[(0, 0, VERMELHO)]),
        d.tile_com(&[(1, 1, VERDE)]),
    );
    let c = d.tile_com(&[(2, 2, AZUL)]);
    let _ = (a, b);
    let g = TileGrid {
        x: 0,
        y: 0,
        w: 2,
        h: 1,
        tiles: vec![
            TileRef::new(1, false, false, false),
            TileRef::new(3, false, true, false),
        ],
        ..TileGrid::default()
    };
    d.com_tiles(vec![c], d.f, g);
    let (op, f2) = new_frame(&mut d.s, d.f, NewFrame::Empty).unwrap();
    d.h.commit(&mut d.s, op).unwrap();
    let g2 = TileGrid {
        x: 8,
        y: 8,
        w: 2,
        h: 1,
        tiles: vec![TileRef::new(3, true, false, false), flip],
        ..TileGrid::default()
    };
    let t = TilesSet {
        cels: vec![((d.l, f2), Some(Cel::tilemap(g2)))],
        ..TilesSet::default()
    };
    d.h.commit(&mut d.s, Op::Tiles(Box::new(t))).unwrap();
    let op = terreno::novo_terreno(&d.s, d.set, "Grass").unwrap();
    d.h.commit(&mut d.s, op).unwrap();
    assert_eq!(d.s.tileset(d.set).unwrap().terrains[0].first, 4);
    let g = |d: &Doc, f| -> TileGrid {
        (**d.s.entry((d.l, f)).unwrap().tiles.as_ref().unwrap()).clone()
    };
    // O tile 1 só está na célula (0,0) do quadro 1: apagá-lo tira-o do conjunto.
    let visto_f2 = d.s.cel((d.l, f2)).unwrap().data.clone();
    assert!(d.traco(&[(0, 0, 0)], Modo::Auto));
    assert_eq!(d.n_tiles(), 3 + TERRAIN_TILES as usize);
    assert_eq!(
        d.s.tileset(d.set).unwrap().terrains[0].first,
        3,
        "o gabarito desce"
    );
    // A caixa encolhe às células usadas: sobra a segunda.
    assert_eq!(g(&d, d.f).tiles, vec![TileRef::new(2, false, true, false)]);
    assert_eq!(g(&d, d.f).x, 8);
    assert_eq!(
        g(&d, f2).tiles,
        vec![
            TileRef::new(2, true, false, false),
            TileRef::new(1, true, false, false)
        ]
    );
    assert_eq!(
        d.s.cel((d.l, f2)).unwrap().data,
        visto_f2,
        "o quadro 2 vê o mesmo"
    );
}

/// Um tile que já estava sem uso não sai por causa de um traço noutro lado.
#[test]
fn so_sai_o_tile_que_este_gesto_largou() {
    let mut d = Doc::m1();
    assert!(d.traco(&[px(0, 1, 5, 5, AZUL)], Modo::Stack));
    let n = d.n_tiles();
    assert_eq!(usos(&d.s, d.s.tileset(d.set).unwrap())[2], 0);
    assert!(d.traco(&[px(3, 3, 2, 2, AZUL)], Modo::Auto));
    assert_eq!(d.n_tiles(), n + 1, "o tile 2 continua");
}

/// O tile 0 nunca se edita, em nenhum modo.
#[test]
fn tile_zero_e_vazio_sempre() {
    for m in [Modo::Manual, Modo::Auto, Modo::Stack] {
        let mut d = Doc::m1();
        let zero = d.s.tileset(d.set).unwrap().tiles[0].clone();
        d.traco(&[px(3, 3, 2, 2, AZUL)], m);
        assert_eq!(d.s.tileset(d.set).unwrap().tiles[0], zero, "{m:?}");
    }
}

/// O tile que uma célula larga e a seguinte reutiliza no mesmo traço fica no conjunto.
#[test]
fn o_tile_largado_e_reutilizado_no_mesmo_traco_fica() {
    let mut d = Doc::novo(32, T);
    let tile = d.tile_com(&[(0, 0, AZUL)]);
    let g = TileGrid {
        x: 0,
        y: 0,
        w: 1,
        h: 1,
        tiles: vec![TileRef::new(1, false, false, false)],
        ..TileGrid::default()
    };
    d.com_tiles(vec![tile], d.f, g);
    assert!(d.traco(&[px(0, 0, 0, 0, 0), px(1, 0, 0, 0, AZUL)], Modo::Auto));
    assert_eq!((d.id(0, 0), d.id(1, 0), d.n_tiles()), (0, 1, 2));
    assert_eq!(d.visto(8, 0), AZUL);
}

/// Fora de uma camada de tilemap, a porta recusa.
#[test]
fn a_porta_recusa_quem_nao_e_tilemap() {
    let d = Doc::m1();
    let img = d.s.layers()[0].id;
    assert!(aplica(&d.s, img, d.f, &[(0, 0, AZUL)], Modo::Auto).is_none());
    assert!(aplica(&d.s, d.l, FrameId(9999), &[(0, 0, AZUL)], Modo::Auto).is_none());
    assert!(aplica(&d.s, d.l, d.f, &[], Modo::Auto).is_none());
}

fn cor(ch: char) -> u32 {
    match ch {
        'R' => VERMELHO,
        'G' => VERDE,
        'B' => AZUL,
        'W' => u32::MAX,
        _ => 0,
    }
}

/// O estado de uma corrida do M9: o mapa e os tiles que a fixtura gravou.
fn m9_doc(e: &serde_json::Value) -> Doc {
    let mut d = Doc::novo(32, T);
    let tiles = e["tiles"].as_array().unwrap()[1..]
        .iter()
        .map(|t| {
            let linhas = t["img"].as_array().unwrap();
            let chars = linhas.iter().flat_map(|l| l.as_str().unwrap().chars());
            chars.flat_map(|c| cor(c).to_le_bytes()).collect()
        })
        .collect();
    let mut grelha = vec![TileRef::EMPTY; 16];
    for (k, id) in e["cells"].as_object().unwrap() {
        let (c, r) = k.split_once(',').unwrap();
        let (c, r): (usize, usize) = (c.parse().unwrap(), r.parse().unwrap());
        grelha[r * 4 + c] = TileRef::new(id.as_u64().unwrap() as u32, false, false, false);
    }
    let mapa = TileGrid {
        x: 0,
        y: 0,
        w: 4,
        h: 4,
        tiles: grelha,
        ..TileGrid::default()
    };
    let f = d.f;
    d.com_tiles(tiles, f, mapa);
    d
}

/// O traço que a corrida fez: o que mudou no canvas. Em Manual, as outras cópias entram também e
/// dão o mesmo tile.
fn m9_escritas(antes: &serde_json::Value, depois: &serde_json::Value) -> Vec<(i32, i32, u32)> {
    let linhas = |e: &serde_json::Value| -> Vec<Vec<char>> {
        let c = e["canvas"].as_array().unwrap();
        c.iter()
            .map(|l| l.as_str().unwrap().chars().collect())
            .collect()
    };
    let (a, b) = (linhas(antes), linhas(depois));
    let mut out = Vec::new();
    for y in 0..a.len() {
        for x in 0..a[y].len() {
            if a[y][x] != b[y][x] {
                out.push((x as i32, y as i32, cor(b[y][x])));
            }
        }
    }
    out
}

/// **M9 corrido**, caso a caso, sobre o que Pixelorama e Trial fazem igual. Ficam de fora os casos
/// onde discordam, e aí vale o Trial, no teste de baixo: apagar um tile inteiro em Manual e em Stack
/// (o Pixelorama deixa o tile intacto com o canvas vazio, e o mapa volta ao reabrir) e a união de um
/// traço em Manual (o Pixelorama perde a parte da direita).
#[test]
fn as_bordas_batem_com_o_m9_corrido() {
    let txt = include_str!("fixtures/oracle_tile_edges_m9.json");
    let v: serde_json::Value = serde_json::from_str(txt).unwrap();
    let corridas = v["pixelorama_q1_q2_q5"].as_object().unwrap();
    let mut conferidos = 0;
    for (tag, depois) in corridas {
        let [q, modo, caso, "depois"] = tag.split('/').collect::<Vec<_>>()[..] else {
            continue;
        };
        let m = match modo {
            "manual" => Modo::Manual,
            "auto" => Modo::Auto,
            "stack" => Modo::Stack,
            _ => continue,
        };
        let inteiro = caso.starts_with('b') || caso.starts_with('d');
        let discordam = match (q, m) {
            ("q1", Modo::Manual | Modo::Stack) => inteiro,
            ("q2", Modo::Manual) => true,
            ("q1" | "q2", _) => false,
            _ => true,
        };
        if discordam {
            continue;
        }
        let antes = &corridas[&format!("{q}/{modo}/{caso}/antes")];
        let mut d = m9_doc(antes);
        let original = d.s.clone();
        let escritas = m9_escritas(antes, depois);
        assert!(!escritas.is_empty(), "{tag}");
        d.traco(&escritas, m);
        let esperado = m9_doc(depois);
        let (nosso, deles) = (
            d.s.tileset(d.set).unwrap().tiles.clone(),
            esperado.s.tileset(esperado.set).unwrap().tiles.clone(),
        );
        assert_eq!(nosso.len(), deles.len(), "{tag}: número de tiles");
        assert_eq!(nosso, deles, "{tag}: imagens dos tiles");
        for r in 0..4 {
            for c in 0..4 {
                assert_eq!(d.id(c, r), esperado.id(c, r), "{tag}: célula {c},{r}");
            }
        }
        assert!(derived_is_consistent(&d.s), "{tag}");
        assert!(d.h.undo(&mut d.s), "{tag}");
        assert_eq!(d.s, original, "{tag}: um passo desfaz");
        conferidos += 1;
    }
    assert!(conferidos >= 20, "só {conferidos} casos conferidos");
    println!("M9: {conferidos} casos");
}

/// Onde os oráculos discordam, vale o Trial (M9): apagar tudo em Manual esvazia o tile e as cópias,
/// que continua no conjunto; em Stack a célula esvazia e o tile fica; a união em Manual.
#[test]
fn onde_os_oraculos_discordam_vale_o_trial() {
    let um_px = |d: &Doc, x, y| d.tile_com(&[(x, y, VERMELHO)]);
    let monta = |usos: &[(usize, usize)]| {
        let mut d = Doc::novo(32, T);
        let tile = um_px(&d, 1, 1);
        let mut g = vec![TileRef::EMPTY; 16];
        for &(c, r) in usos {
            g[r * 4 + c] = TileRef::new(1, false, false, false);
        }
        let mapa = TileGrid {
            x: 0,
            y: 0,
            w: 4,
            h: 4,
            tiles: g,
            ..TileGrid::default()
        };
        let f = d.f;
        d.com_tiles(vec![tile], f, mapa);
        d
    };
    let tres = [(0, 0), (1, 0), (3, 0)];
    let vazio_em = |c: i32| (c * i32::from(T) + 1, 1, 0);

    let mut d = monta(&tres);
    d.traco(&[vazio_em(0)], Modo::Manual);
    let set = d.s.tileset(d.set).unwrap();
    assert_eq!(set.tiles.len(), 2, "Manual: o tile fica");
    assert_eq!(set.tiles[1], set.tiles[0], "Manual: e fica vazio");
    assert_eq!(d.id(1, 0), 1, "as cópias continuam a apontar para ele");

    let mut d = monta(&tres);
    d.traco(&[vazio_em(0)], Modo::Stack);
    assert_eq!(d.id(0, 0), 0, "Stack: a célula esvazia");
    assert_eq!(d.id(1, 0), 1);
    assert_eq!(
        d.s.tileset(d.set).unwrap().tiles.len(),
        2,
        "Stack: o tile fica"
    );

    let mut d = monta(&tres[..2]);
    let t = i32::from(T);
    d.traco(&[(0, 0, AZUL), (t + 7, 0, AZUL)], Modo::Manual);
    let esperado = d.tile_com(&[(1, 1, VERMELHO), (0, 0, AZUL), (7, 0, AZUL)]);
    assert_eq!(d.s.tileset(d.set).unwrap().tiles[1], esperado, "a união");
}

/// **M9b corrido.** A (2×), B e C: a lista e os números, e quem fica com o original fora da horizontal.
/// Na diagonal os oráculos discordam, e vale o Trial: linha primeiro.
#[test]
fn a_lista_e_a_copia_que_fica_batem_com_o_m9b() {
    let monta = |a_em: &[(usize, usize)]| {
        let mut d = Doc::novo(32, T);
        let tiles = vec![
            d.tile_com(&[(1, 1, VERMELHO)]),
            d.tile_com(&[(2, 2, VERDE)]),
            d.tile_com(&[(3, 3, VERDE)]),
        ];
        let mut g = vec![TileRef::EMPTY; 16];
        for &(c, r) in a_em {
            g[r * 4 + c] = TileRef::new(1, false, false, false);
        }
        g[3 * 4 + 2] = TileRef::new(2, false, false, false);
        g[3 * 4 + 3] = TileRef::new(3, false, false, false);
        let mapa = TileGrid {
            x: 0,
            y: 0,
            w: 4,
            h: 4,
            tiles: g,
            ..TileGrid::default()
        };
        let f = d.f;
        d.com_tiles(tiles, f, mapa);
        d
    };
    let t = i32::from(T);
    // Dois toques no mesmo píxel do tile, um em cada célula.
    let igual = |c: i32, r: i32| (c * t, r * t, AZUL);
    let outro = |c: i32, r: i32| (c * t + 7, r * t + 7, AZUL);

    // 1. Iguais: A sai e os de cima descem; o novo vai para o fim.
    let mut d = monta(&[(0, 0), (1, 0)]);
    let (b, c) = {
        let set = d.s.tileset(d.set).unwrap();
        (set.tiles[2].clone(), set.tiles[3].clone())
    };
    d.traco(&[igual(0, 0), igual(1, 0)], Modo::Auto);
    let set = d.s.tileset(d.set).unwrap();
    assert_eq!(set.tiles.len(), 4);
    assert_eq!((&set.tiles[1], &set.tiles[2]), (&b, &c), "B=1, C=2");
    assert_eq!((d.id(0, 0), d.id(1, 0)), (3, 3), "novo=3");
    assert_eq!((d.id(2, 3), d.id(3, 3)), (1, 2));

    // 2. Diferentes na horizontal, 3. na vertical e 4. na diagonal: quem vem primeiro ganha o novo.
    for (nome, p, q) in [
        ("horizontal", (0, 0), (1, 0)),
        ("vertical", (0, 0), (0, 1)),
        ("diagonal, linha primeiro", (1, 0), (0, 1)),
    ] {
        let mut d = monta(&[(p.0 as usize, p.1 as usize), (q.0 as usize, q.1 as usize)]);
        d.traco(&[outro(q.0, q.1), igual(p.0, p.1)], Modo::Auto);
        assert_eq!(d.id(p.0, p.1), 4, "{nome}: o primeiro ganha o novo, no fim");
        assert_eq!(d.id(q.0, q.1), 1, "{nome}: o outro edita o original");
        assert_eq!((d.id(2, 3), d.id(3, 3)), (2, 3), "{nome}: a ordem fica");
    }
}
