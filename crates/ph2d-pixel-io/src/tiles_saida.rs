//! **File ▸ Export ▸ Export Tileset**: os tiles em PNG e o mapa em JSON. O Godot corrido remonta-os píxel
//! a píxel (`docs/PixelArt/P10_oraculos.md`, M8; `docs/PixelArt/ferramentas/oraculo_godot_remonta.gd`).

use ph2d_pixel_core::{Sprite, TileRef, Tileset, TilesetId};
use serde_json::{Value, json};

/// O pedido da janela.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportTiles {
    pub conjunto: TilesetId,
    /// Tiles por linha da imagem.
    pub colunas: u32,
}

/// As colunas de partida: as do gabarito, onde um terreno sai inteiro em 6 linhas.
pub const COLUNAS_INICIAIS: u32 = 8;

/// A imagem dos tiles: o tile `i` na célula `(i % colunas, i / colunas)`, com o vazio no lugar 0.
#[must_use]
pub fn atlas(s: &Sprite, set: &Tileset, colunas: u32) -> (u32, u32, Vec<u8>) {
    let colunas = colunas.max(1);
    let (tw, th) = (u32::from(set.tile_w), u32::from(set.tile_h));
    let linhas = (set.tiles.len() as u32).div_ceil(colunas);
    let (w, h) = (colunas * tw, linhas * th);
    let bpp = s.mode().bpp();
    let mut rgba = vec![0; w as usize * h as usize * 4];
    for (i, t) in set.tiles.iter().enumerate() {
        let (cx, cy) = (i as u32 % colunas * tw, i as u32 / colunas * th);
        for (p, px) in t.chunks_exact(bpp).enumerate() {
            let mut v = [0; 4];
            v[..bpp].copy_from_slice(px);
            let (x, y) = (cx + p as u32 % tw, cy + p as u32 / tw);
            let d = (y as usize * w as usize + x as usize) * 4;
            rgba[d..d + 4].copy_from_slice(&s.rgba_of_value(u32::from_le_bytes(v)));
        }
    }
    (w, h, rgba)
}

/// As camadas deste tileset, de baixo para cima, com o mapa de cada quadro.
fn camadas(s: &Sprite, set: &Tileset) -> Vec<Value> {
    let (tw, th) = (i32::from(set.tile_w), i32::from(set.tile_h));
    s.layers()
        .iter()
        .filter(|l| l.tileset() == Some(set.id))
        .map(|l| {
            let cels: Vec<Value> = s
                .frames()
                .iter()
                .enumerate()
                .filter_map(|(n, f)| {
                    let g = s.entry(s.owner((l.id, f.id)))?.tiles.as_deref()?;
                    let ((col, row), fase) = g.ancora(set.forma, tw, th);
                    let tiles: Vec<u32> = g.tiles.iter().map(|t| t.0).collect();
                    Some(json!({
                        "frame": n,
                        "col": col,
                        "row": row,
                        "w": g.w,
                        "h": g.h,
                        "origin": [fase.0, fase.1],
                        "tiles": tiles,
                    }))
                })
                .collect();
            json!({
                "name": l.name,
                "visible": l.visible,
                "opacity": l.opacity,
                "cels": cels,
            })
        })
        .collect()
}

/// O texto do `.json`: a imagem, a forma da grelha, os flips e o mapa de cada camada e quadro. A célula
/// `i` de um cel está em `(col + i % w, row + i / w)`, e `origin` é onde começa a caixa da `(0, 0)`.
#[must_use]
pub fn json_do_mapa(s: &Sprite, set: &Tileset, imagem: &str, colunas: u32) -> String {
    let duracoes: Vec<Value> = s
        .frames()
        .iter()
        .map(|f| json!({ "duration_ms": f.duration_ms }))
        .collect();
    let terrenos: Vec<Value> = set
        .terrains
        .iter()
        .map(|t| json!({ "name": t.name, "first": t.first }))
        .collect();
    let v = json!({
        "format": "ph2d-tilemap",
        "version": 1,
        "image": imagem,
        "columns": colunas.max(1),
        "tile_count": set.tiles.len(),
        "tile_w": set.tile_w,
        "tile_h": set.tile_h,
        "shape": set.forma.nome(),
        "flip_bits": {
            "x": TileRef::FLIP_X,
            "y": TileRef::FLIP_Y,
            "d": TileRef::FLIP_D,
            "id_mask": TileRef::ID_MASK,
        },
        "canvas": [s.width(), s.height()],
        "frames": duracoes,
        "terrains": terrenos,
        "layers": camadas(s, set),
    });
    serde_json::to_string_pretty(&v).unwrap_or_default()
}
