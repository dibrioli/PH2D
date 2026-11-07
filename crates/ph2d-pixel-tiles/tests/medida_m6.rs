//! **M6**: editar um tile usado em toda a grelha, por todos os quadros, sem ligados. Corre à mão, em
//! `--profile smoke`, com `-- --ignored --nocapture --test-threads=1`.

use std::time::Instant;

use ph2d_pixel_core::tiles::new_tilemap_layer;
use ph2d_pixel_core::{
    Background, Cel, ColorMode, FrameId, History, LayerId, NewSprite, Op, Sprite, TileGrid,
    TileRef, TilesSet, TilesetId,
};
use ph2d_pixel_tiles::desenho::{Modo, aplica};

const TILE: u16 = 16;
const BPP: usize = 4;

fn monta(celulas: u32, quadros: u32) -> (Sprite, LayerId, FrameId, TilesetId) {
    let lado = celulas * u32::from(TILE);
    let mut s = Sprite::new(NewSprite {
        width: lado,
        height: lado,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .unwrap();
    let f0 = s.frames()[0].id;
    let acima = s.layers()[0].id;
    let (op, l, set) = new_tilemap_layer(&mut s, acima, TILE, TILE).unwrap();
    ph2d_pixel_core::apply(&mut s, op).unwrap();
    let mut quadros_ids = vec![f0];
    for _ in 1..quadros {
        let (op, f) = ph2d_pixel_core::frames::new_frame(
            &mut s,
            f0,
            ph2d_pixel_core::frames::NewFrame::Empty,
        )
        .unwrap();
        ph2d_pixel_core::apply(&mut s, op).unwrap();
        quadros_ids.push(f);
    }
    let g = TileGrid {
        x: 0,
        y: 0,
        w: celulas,
        h: celulas,
        tiles: vec![TileRef::new(1, false, false, false); (celulas * celulas) as usize],
        ..TileGrid::default()
    };
    let tile = vec![9; usize::from(TILE) * usize::from(TILE) * BPP];
    let t = TilesSet {
        counts: vec![(set, 2)],
        tiles: vec![(set, 1, tile)],
        cels: quadros_ids
            .iter()
            .map(|&f| ((l, f), Some(Cel::tilemap(g.clone()))))
            .collect(),
        ..TilesSet::default()
    };
    ph2d_pixel_core::apply(&mut s, Op::Tiles(Box::new(t))).unwrap();
    (s, l, f0, set)
}

fn mede(celulas: u32, quadros: u32, voltas: u32) {
    let px = u64::from(celulas) * u64::from(TILE);
    let gb = px * px * BPP as u64 * u64::from(quadros);
    println!(
        "M6 {celulas}×{celulas} células, tile {TILE}, {quadros} quadros: {} MB de imagens",
        gb >> 20
    );
    let t0 = Instant::now();
    let (mut s, l, f, _) = monta(celulas, quadros);
    println!("M6 montar: {:?}", t0.elapsed());
    let mut h = History::default();
    let (mut melhor_gesto, mut melhor_aplica, mut melhor_desfaz) =
        (u128::MAX, u128::MAX, u128::MAX);
    let mut melhor_antigo = u128::MAX;
    for i in 0..voltas {
        let t = Instant::now();
        px_a_px(&s, l, quadros);
        let antigo = t.elapsed().as_micros();
        println!("M6 volta {i}: píxel a píxel (o caminho de antes) {antigo} µs");
        melhor_antigo = melhor_antigo.min(antigo);
        let v = u32::from_le_bytes([i as u8 + 1, 2, 3, 255]);
        let t = Instant::now();
        let op = aplica(&s, l, f, &[(3, 3, v)], Modo::Manual).unwrap();
        let so_decide = t.elapsed().as_micros();
        let t = Instant::now();
        h.commit(&mut s, op).unwrap();
        let so_aplica = t.elapsed().as_micros();
        let t = Instant::now();
        assert!(h.undo(&mut s));
        let desfaz = t.elapsed().as_micros();
        assert!(h.redo(&mut s));
        println!(
            "M6 volta {i}: decidir {so_decide} µs · aplicar {so_aplica} µs · desfazer {desfaz} µs"
        );
        melhor_gesto = melhor_gesto.min(so_decide + so_aplica);
        melhor_aplica = melhor_aplica.min(so_aplica);
        melhor_desfaz = melhor_desfaz.min(desfaz);
    }
    println!(
        "M6 MÍNIMO de {voltas}: gesto {:.1} ms (aplicar {:.1} ms) · desfazer {:.1} ms · píxel a píxel {:.1} ms",
        melhor_gesto as f64 / 1000.0,
        melhor_aplica as f64 / 1000.0,
        melhor_desfaz as f64 / 1000.0,
        melhor_antigo as f64 / 1000.0
    );
}

/// **Um quadro de traço ao vivo**: decidir + aplicar + compor o sprite + repor, num núcleo.
fn mede_quadro_ao_vivo(celulas: u32, quadros: u32, voltas: u32) {
    let (mut s, l, f, _) = monta(celulas, quadros);
    let lado = s.width();
    let (mut melhor, mut compor) = (u128::MAX, u128::MAX);
    for i in 0..voltas {
        let v = u32::from_le_bytes([i as u8 + 1, 2, 3, 255]);
        let t = Instant::now();
        let op = aplica(&s, l, f, &[(3, 3, v)], Modo::Manual).unwrap();
        let inverso = ph2d_pixel_core::apply(&mut s, op).unwrap();
        let c = Instant::now();
        let px = ph2d_pixel_core::compose_window(
            &s,
            f,
            None,
            ph2d_pixel_core::Rect::new(0, 0, lado, lado),
            ph2d_pixel_core::Purpose::View,
        );
        let so_compor = c.elapsed().as_micros();
        std::hint::black_box(&px);
        ph2d_pixel_core::apply(&mut s, inverso).unwrap();
        melhor = melhor.min(t.elapsed().as_micros());
        compor = compor.min(so_compor);
    }
    println!(
        "AO VIVO {celulas}×{celulas} células ({lado} px), {quadros} quadros: MÍNIMO de {voltas}: {:.1} ms por quadro, dos quais compor {:.1} ms",
        melhor as f64 / 1000.0,
        compor as f64 / 1000.0
    );
}

#[test]
#[ignore = "medição à mão"]
fn m6_quadro_ao_vivo() {
    mede_quadro_ao_vivo(6, 1, 9);
    mede_quadro_ao_vivo(32, 1, 9);
    mede_quadro_ao_vivo(64, 8, 9);
    mede_quadro_ao_vivo(256, 64, 5);
}

/// O caminho de antes, no MESMO processo: refaz cada célula de cada quadro, píxel a píxel, num
/// buffer à parte (o que só o favorece, por caber melhor na cache).
fn px_a_px(s: &Sprite, l: LayerId, quadros: u32) {
    let (tw, th) = (u32::from(TILE), u32::from(TILE));
    let mut buf = Vec::new();
    for (k, c) in s.entries() {
        let Some(g) = c.tiles.as_deref().filter(|_| k.0 == l) else {
            continue;
        };
        let w = g.w as usize * usize::from(TILE);
        buf.resize(c.data.len(), 0u8);
        let src = vec![7u8; usize::from(TILE) * usize::from(TILE) * BPP];
        for (i, &t) in g.tiles.iter().enumerate() {
            let (col, row) = (i % g.w as usize, i / g.w as usize);
            for sy in 0..th {
                for sx in 0..tw {
                    let (dx, dy) = t.place(sx, sy, tw, th);
                    let a = (sy * tw + sx) as usize * BPP;
                    let d =
                        ((row * th as usize + dy as usize) * w + col * tw as usize + dx as usize)
                            * BPP;
                    buf[d..d + BPP].copy_from_slice(&src[a..a + BPP]);
                }
            }
        }
        std::hint::black_box(&buf);
    }
    std::hint::black_box(quadros);
}

#[test]
#[ignore = "medição: 4,3 GB de imagens"]
fn m6_256_celulas_64_quadros() {
    mede(256, 64, 5);
}

/// O lado de 1024×1024 com tile 16: 1 GiB por quadro. Um quadro só, para dar o custo por quadro.
#[test]
#[ignore = "medição: 1 GiB por quadro"]
fn m6_1024_celulas_1_quadro() {
    mede(1024, 1, 3);
}
