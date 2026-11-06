//! Quem não é traço nunca escreve no `data` de um cel de tilemap: recusa, ou anda com a grelha.

use ph2d_pixel_core::canvas::{self, Borders, SizeMethod};
use ph2d_pixel_core::edit::{self, Rotation};
use ph2d_pixel_core::filter::adjust::Invert;
use ph2d_pixel_core::filter::{self, Channels};
use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::selection::Mask;
use ph2d_pixel_core::tiles::new_tilemap_layer;
use ph2d_pixel_core::transform::{Floating, Image, lift, stamp};
use ph2d_pixel_core::{
    Background, Cel, ColorMode, FrameId, History, LayerId, NewSprite, Op, Paint, Patch, Rect,
    Sprite, Stroke, TileGrid, TileRef, TilesSet, TilesetId, compose_rgba, derived_is_consistent,
    layers,
};
use ph2d_pixel_tiles::converte::{para_imagem, para_tilemap};
use ph2d_pixel_tiles::desenho::{Modo, aplica};
use ph2d_pixel_tiles::mapa::carimba;

const T: u16 = 4;

struct Doc {
    s: Sprite,
    h: History,
    img: LayerId,
    tm: LayerId,
    set: TilesetId,
    f: FrameId,
}

/// Imagem em baixo, tilemap em cima, dois quadros (um ligado).
fn doc() -> Doc {
    let mut s = Sprite::new(NewSprite {
        width: 32,
        height: 32,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .unwrap();
    let mut h = History::default();
    let (img, f) = (s.layers()[0].id, s.frames()[0].id);
    let rect = Rect::new(1, 1, 2, 1);
    let p = Patch {
        layer: img,
        frame: f,
        bbox: Some(rect),
        rect,
        pixels: vec![9; 8],
    };
    h.commit(&mut s, Op::Patch(p)).unwrap();
    let (op, tm, set) = new_tilemap_layer(&mut s, img, T, T).unwrap();
    h.commit(&mut s, op).unwrap();
    let tile: Vec<u8> = (0..16_u8).flat_map(|i| [i + 1, 5, 5, 255]).collect();
    let r = TileRef::new(1, false, false, false);
    let g = TileGrid {
        x: 4,
        y: 8,
        w: 2,
        h: 1,
        tiles: vec![r, r],
    };
    let t = TilesSet {
        counts: vec![(set, 2)],
        tiles: vec![(set, 1, tile)],
        cels: vec![((tm, f), Some(Cel::tilemap(g)))],
        ..TilesSet::default()
    };
    h.commit(&mut s, Op::Tiles(Box::new(t))).unwrap();
    let (op, _) = frames::new_frame(&mut s, f, NewFrame::Linked).unwrap();
    h.commit(&mut s, op).unwrap();
    Doc {
        s,
        h,
        img,
        tm,
        set,
        f,
    }
}

fn composto(s: &Sprite, f: FrameId) -> Vec<u8> {
    let mut px = Vec::new();
    compose_rgba(s, f, None, &mut px);
    px
}

/// Cada um destes devolve `None` numa camada de tilemap: a lista que a tela deixa cinzenta.
#[test]
fn os_que_recusam_recusam_e_o_documento_fica_igual() {
    let mut d = doc();
    let (s, tm, f, img) = (d.s.clone(), d.tm, d.f, d.img);
    let key = (tm, f);
    let bg = s.transparent_value();
    let flo = Floating {
        x: 0,
        y: 0,
        image: Image::new(1, 1, 4, 7),
        keep: vec![true],
    };
    let m = Mask::full(32, 32);
    for (nome, r) in [
        ("rotate 90", canvas::rotate_canvas(&s, Rotation::Cw90)),
        ("rotate -90", canvas::rotate_canvas(&s, Rotation::Ccw90)),
        ("rotate 180", canvas::rotate_canvas(&s, Rotation::R180)),
        ("flip canvas h", canvas::flip_canvas(&s, true)),
        ("flip canvas v", canvas::flip_canvas(&s, false)),
        (
            "sprite size",
            canvas::sprite_size(&s, 64, 64, SizeMethod::Nearest),
        ),
        (
            "sprite size bilinear",
            canvas::sprite_size(&s, 64, 64, SizeMethod::Bilinear),
        ),
        (
            "sprite size rotsprite",
            canvas::sprite_size(&s, 64, 64, SizeMethod::RotSprite),
        ),
        (
            "canvas size fora",
            canvas::canvas_size(
                &s,
                Borders {
                    left: 1,
                    ..Borders::default()
                },
                bg,
            ),
        ),
        (
            "canvas size fora y",
            canvas::canvas_size(
                &s,
                Borders {
                    top: 2,
                    ..Borders::default()
                },
                bg,
            ),
        ),
        ("crop fora", canvas::crop(&s, Rect::new(1, 0, 16, 16))),
        ("filtro", filter::apply(&s, &[key], Channels::ALL, &Invert)),
        (
            "filtro ligado",
            filter::patch(&s, (tm, s.frames()[1].id), Channels::ALL, &Invert),
        ),
        ("edit flip", edit::flip(&s, key, true, bg)),
        ("edit rotate", edit::rotate(&s, key, Rotation::R180, bg)),
        ("edit paste", edit::paste(&s, key, &flo, bg)),
        ("edit commit", edit::commit(&s, key, &m, &flo, bg)),
        ("stamp", stamp(&s, key, &flo, Some(&m), None, bg)),
        ("merge down do tilemap", layers::merge_down(&s, tm)),
    ] {
        assert!(r.is_none(), "{nome}");
    }
    assert!(lift(&s, key, Some(&m)).is_some(), "ler continua a valer");
    // Traço de píxeis direto: sem op, e `writes` é a porta.
    let mut st = Stroke::new(tm, f, 7, Paint::default());
    st.start(&s, 4, 8);
    assert!(st.op(&s).is_none());
    assert_eq!(st.writes(&s), vec![((4, 8), 7)]);
    // Merge Down com o tilemap em baixo.
    let (op, sobre) = layers::new_layer(&mut d.s, tm).unwrap();
    d.h.commit(&mut d.s, op).unwrap();
    assert!(layers::merge_down(&d.s, sobre).is_none());
    // Layer via Cut abriria um buraco no derivado; via Copy só lê.
    let mut c = d.s.clone();
    assert!(layers::via(&mut c, tm, f, Rect::new(4, 8, 4, 4), true).is_none());
    let (op, nova) = layers::via(&mut c, tm, f, Rect::new(4, 8, 4, 4), false).unwrap();
    ph2d_pixel_core::apply(&mut c, op).unwrap();
    assert!(c.entry((nova, f)).is_some_and(|x| x.tiles.is_none()));
    assert!(derived_is_consistent(&c));
    // Um cel não muda para uma camada de outra espécie.
    let f2 = d.s.frames()[1].id;
    assert!(frames::move_cels(&d.s, &[((tm, f), (img, f2))], true).is_none());
    assert!(frames::move_cels(&d.s, &[((img, f), (tm, f2))], false).is_none());
    assert!(frames::move_cels_grow(&mut d.s, &[], &[((tm, f), (img, 0))], true).is_none());
    assert!(derived_is_consistent(&d.s));
}

/// Múltiplo do tile, a grelha anda. Fora dele, recusa (no teste de cima).
#[test]
fn canvas_size_crop_e_trim_andam_a_grelha_por_multiplos_do_tile() {
    let mut d = doc();
    let antes = d.s.clone();
    let bg = d.s.transparent_value();
    let visto = composto(&d.s, d.f);
    let op = canvas::canvas_size(
        &d.s,
        Borders {
            left: 8,
            top: 4,
            right: 0,
            bottom: 0,
        },
        bg,
    )
    .unwrap();
    d.h.commit(&mut d.s, op).unwrap();
    assert!(derived_is_consistent(&d.s));
    let g = d.s.entry((d.tm, d.f)).unwrap().tiles.clone().unwrap();
    assert_eq!((g.x, g.y), (12, 12));
    assert_eq!((d.s.width(), d.s.height()), (40, 36));
    let novo = composto(&d.s, d.f);
    for y in 0..32_usize {
        for x in 0..32_usize {
            let (a, b) = (y * 32 + x, (y + 4) * 40 + x + 8);
            assert_eq!(visto[a * 4..a * 4 + 4], novo[b * 4..b * 4 + 4], "{x},{y}");
        }
    }
    assert!(d.h.undo(&mut d.s));
    assert_eq!(d.s, antes, "desfazer ao byte");

    let op = canvas::crop(&d.s, Rect::new(4, 4, 16, 16)).unwrap();
    d.h.commit(&mut d.s, op).unwrap();
    assert!(derived_is_consistent(&d.s));
    let g = d.s.entry((d.tm, d.f)).unwrap().tiles.clone().unwrap();
    assert_eq!((g.x, g.y), (0, 4));
    assert!(d.h.undo(&mut d.s));
    assert_eq!(d.s, antes);

    // Trim: a imagem começa em (1,1), que não é múltiplo: recusa.
    assert!(canvas::trim(&d.s).is_none());
    // Sem esse píxel, o que fica é a caixa do mapa, múltipla: anda.
    let op = layers::delete(&d.s, d.img).unwrap();
    d.h.commit(&mut d.s, op).unwrap();
    let op = canvas::trim(&d.s).unwrap();
    d.h.commit(&mut d.s, op).unwrap();
    assert!(derived_is_consistent(&d.s));
    assert_eq!((d.s.width(), d.s.height()), (8, 4));
}

#[test]
fn flatten_da_imagem_com_o_composto_certo() {
    for so_visiveis in [false, true] {
        let mut d = doc();
        let quadros: Vec<_> = d.s.frames().iter().map(|f| f.id).collect();
        let vistos: Vec<_> = quadros.iter().map(|&f| composto(&d.s, f)).collect();
        let antes = d.s.clone();
        let op = layers::flatten(&mut d.s, so_visiveis).unwrap();
        let reservado = d.s.clone();
        d.h.commit(&mut d.s, op).unwrap();
        assert!(d.s.layers().iter().all(|l| l.tileset().is_none()));
        assert!(d.s.entries().all(|(_, c)| c.tiles.is_none()));
        for (f, v) in quadros.iter().zip(&vistos) {
            assert_eq!(&composto(&d.s, *f), v);
        }
        assert!(d.s.tileset(d.set).is_some(), "o tileset fica no sprite");
        assert!(d.h.undo(&mut d.s));
        assert_eq!(d.s, reservado);
        assert_eq!(d.s.layers(), antes.layers());
    }
}

#[test]
fn duplicar_leva_o_mesmo_tileset_e_o_mapa_copiado() {
    let mut d = doc();
    let (op, copia) = layers::duplicate(&mut d.s, d.tm).unwrap();
    d.h.commit(&mut d.s, op).unwrap();
    assert_eq!(d.s.layer(copia).unwrap().tileset(), Some(d.set));
    let (a, b) = (
        d.s.entry((d.tm, d.f)).unwrap(),
        d.s.entry((copia, d.f)).unwrap(),
    );
    assert_eq!(a.tiles, b.tiles);
    assert!(derived_is_consistent(&d.s));
    // O mapa é cópia: carimbar numa não muda a outra.
    let op = carimba(
        &d.s,
        copia,
        d.f,
        &[(0, 0)],
        TileRef::new(1, false, false, false),
    )
    .unwrap();
    d.h.commit(&mut d.s, op).unwrap();
    let (a, b) = (
        d.s.entry((d.tm, d.f)).unwrap(),
        d.s.entry((copia, d.f)).unwrap(),
    );
    assert_ne!(a.tiles, b.tiles);
    assert!(derived_is_consistent(&d.s));
}

/// Um gerador sem relógio: o mesmo teste, a mesma sequência.
struct Dado(u64);
impl Dado {
    fn n(&mut self, m: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) % m
    }
}

/// **`o_derivado_e_o_mapa_expandido`:** depois de cada op de uma sequência sorteada, `data` é o mapa
/// expandido; desfazer tudo volta ao byte, e refazer tudo também.
#[test]
fn o_derivado_e_o_mapa_expandido_depois_de_qualquer_sequencia() {
    for semente in 1..=12 {
        let mut d = doc();
        let mut dado = Dado(semente);
        let base = d.s.clone();
        let mut feitos = 0;
        for passo in 0..120 {
            let quadros: Vec<FrameId> = d.s.frames().iter().map(|f| f.id).collect();
            let f = quadros[dado.n(quadros.len() as u64) as usize];
            let camadas: Vec<LayerId> =
                d.s.layers()
                    .iter()
                    .filter(|l| !l.is_group())
                    .map(|l| l.id)
                    .collect();
            let l = camadas[dado.n(camadas.len() as u64) as usize];
            let (w, h) = (d.s.width() as i32, d.s.height() as i32);
            let (x, y) = (dado.n(w as u64) as i32, dado.n(h as u64) as i32);
            let cor = u32::from_le_bytes([dado.n(3) as u8 * 80, 5, 5, 255]);
            let v = if dado.n(4) == 0 { 0 } else { cor };
            let modo = [Modo::Manual, Modo::Auto, Modo::Stack][dado.n(3) as usize];
            let bg = d.s.transparent_value();
            let op = match dado.n(14) {
                0..=4 => {
                    let e: Vec<_> = (0..1 + dado.n(6) as i32)
                        .map(|i| ((x + i * 3).min(w - 1), y, v))
                        .collect();
                    aplica(&d.s, l, f, &e, modo)
                }
                5 => {
                    let n = dado.n(3) as u32;
                    carimba(
                        &d.s,
                        l,
                        f,
                        &[(x / 4, y / 4)],
                        TileRef::new(n, false, n == 2, n == 1),
                    )
                    .filter(|_| n < d.s.tilesets().first().map_or(0, |t| t.tiles.len() as u32))
                }
                6 => frames::new_frame(&mut d.s, f, NewFrame::Copy).map(|o| o.0),
                7 => frames::new_frame(&mut d.s, f, NewFrame::Linked).map(|o| o.0),
                8 => frames::delete(&d.s, &[f]),
                9 => frames::unlink(&d.s, (l, f)),
                10 => layers::duplicate(&mut d.s, l)
                    .map(|o| o.0)
                    .filter(|_| camadas.len() < 6),
                11 if w < 80 => canvas::canvas_size(
                    &d.s,
                    Borders {
                        left: 4,
                        top: 8,
                        right: 4,
                        bottom: 0,
                    },
                    bg,
                ),
                12 => frames::move_to(&d.s, f, dado.n(4) as usize),
                _ if dado.n(9) == 0 => para_imagem(&d.s, l),
                _ if dado.n(9) == 0 => para_tilemap(&mut d.s, l, T, T, passo % 2 == 0).map(|o| o.0),
                _ => None,
            };
            if let Some(op) = op
                && d.h.commit(&mut d.s, op).is_ok()
            {
                feitos += 1;
            }
            assert!(
                derived_is_consistent(&d.s),
                "semente {semente}, passo {passo}"
            );
        }
        assert!(
            feitos > 30,
            "semente {semente}: a sequência fez pouco ({feitos})"
        );
        let fim = d.s.clone();
        for _ in 0..feitos {
            assert!(
                d.h.undo(&mut d.s),
                "semente {semente}: o tecto do histórico comeu um passo"
            );
            assert!(derived_is_consistent(&d.s), "semente {semente}, a desfazer");
        }
        // Os ids reservados não voltam (nunca se reciclam); tudo o resto, ao byte.
        assert_eq!(d.s.layers(), base.layers(), "semente {semente}");
        assert_eq!(d.s.frames(), base.frames(), "semente {semente}");
        assert_eq!(d.s.tilesets(), base.tilesets(), "semente {semente}");
        assert!(d.s.entries().eq(base.entries()), "semente {semente}");
        assert_eq!((d.s.width(), d.s.height()), (base.width(), base.height()));
        for _ in 0..feitos {
            assert!(d.h.redo(&mut d.s));
        }
        assert_eq!(d.s, fim, "semente {semente}: refazer tudo");
    }
}
