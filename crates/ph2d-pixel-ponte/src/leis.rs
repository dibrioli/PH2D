//! Leis puras que a família do Pixel Art usa: nada aqui vê o estado dela.

use ph2d_editor_core::pixel_panel::{
    ColorModeChoice, NewSpriteMode, PIXEL_BRUSH_THUMB_MAX, PixelBrushThumb, PixelDither,
    PixelSortKey, PixelToGray,
};
use ph2d_pixel_core::color::ToGray;
use ph2d_pixel_core::convert::{Bayer, Convert, Dither};
use ph2d_pixel_core::layers::{self};
use ph2d_pixel_core::palette::SortKey;
use ph2d_pixel_core::selection::Mask;
use ph2d_pixel_core::transform::{Floating, Image};
use ph2d_pixel_core::{ColorMode, LayerId, Op, Patch, Sprite, Stroke};
use ph2d_pixel_io::brush_file::BrushSlot;

/// *Layer via Copy/Cut* com a selecção de VERDADE (P6): só as células seleccionadas vão para a
/// camada nova (as outras ficam transparentes nela); no Cut o original perde-as (Background: a cor
/// de fundo). UM passo — o Cut é um lote (camada nova + buraco), os dois contra o estado de antes.
pub fn via_selection(
    s: &mut Sprite,
    active: LayerId,
    frame: ph2d_pixel_core::FrameId,
    cut: bool,
    bg: u32,
) -> Option<(Op, LayerId)> {
    use ph2d_pixel_core::transform::{Floating, Image, lift, stamp};
    let l = s.layer(active)?;
    if l.is_group() || (cut && l.tileset().is_some()) {
        return None;
    }
    let key = (active, frame);
    let m = s.mask()?.clone();
    let f = lift(s, key, Some(&m))?;
    let clip = layers::Clip {
        mode: s.mode(),
        palette: s.palette().to_vec(),
        transparent: s.transparent_index(),
        rect: f.rect(),
        data: f.image.data,
    };
    let buraco = if cut {
        let nada = Floating {
            x: 0,
            y: 0,
            image: Image::new(0, 0, s.mode().bpp(), s.transparent_value()),
            keep: Vec::new(),
        };
        stamp(s, key, &nada, Some(&m), None, bg)
    } else {
        None
    };
    let (op, id) = layers::paste_as_new(s, active, frame, &clip, false)?;
    let op = match buraco {
        Some(b) => Op::Batch(vec![b, op]),
        None => op,
    };
    Some((op, id))
}

/// Os valores que o op escreve, como um traço de valores prontos.
pub fn traco(o: &Op, cel: ph2d_pixel_core::CelKey) -> Stroke {
    let mut values = Vec::new();
    let mut alvo = None;
    let mut patch = |p: &Patch| {
        alvo.get_or_insert((p.layer, p.frame));
        let bpp = p.pixels.len() / (p.rect.w as usize * p.rect.h as usize).max(1);
        for y in 0..p.rect.h {
            for x in 0..p.rect.w {
                let i = (y as usize * p.rect.w as usize + x as usize) * bpp;
                let mut b = [0u8; 4];
                b[..bpp].copy_from_slice(&p.pixels[i..i + bpp]);
                values.push((
                    (p.rect.x + x as i32, p.rect.y + y as i32),
                    u32::from_le_bytes(b),
                ));
            }
        }
    };
    match o {
        Op::Patch(p) => patch(p),
        Op::Batch(ops) => ops.iter().for_each(|o| {
            if let Op::Patch(p) = o {
                patch(p);
            }
        }),
        _ => {}
    }
    let (layer, frame) = alvo.unwrap_or(cel);
    Stroke::with_values(layer, frame, values)
}

/// As células de `m` no cel `ck` com o valor `v` (`None` = nada muda).
pub fn pinta(
    s: &ph2d_pixel_core::Sprite,
    ck: ph2d_pixel_core::CelKey,
    m: &Mask,
    v: u32,
) -> Option<Op> {
    let ck = s.owner(ck);
    let rect = m.bounds()?;
    let bpp = s.mode().bpp();
    let before = s.read_rect(ck, rect);
    let mut pixels = before.clone();
    for (x, y) in m.cells() {
        let i = ((y - rect.y) as usize * rect.w as usize + (x - rect.x) as usize) * bpp;
        pixels[i..i + bpp].copy_from_slice(&v.to_le_bytes()[..bpp]);
    }
    (pixels != before).then(|| {
        Op::Patch(Patch {
            layer: ck.0,
            frame: ck.1,
            bbox: Some(s.cel(ck).map_or(rect, |c| c.bbox.union(rect))),
            rect,
            pixels,
        })
    })
}

/// A cobertura `fl` (1 byte, já no sítio) pintada em `fg` e cortada ao sprite e à selecção: as células
/// no modo do sprite, prontas a assentar.
pub fn na_cor(s: &Sprite, fl: &Floating, fg: [u8; 4]) -> Floating {
    let (w, h) = (fl.image.w, fl.image.h);
    let mut image = Image::new(w, h, s.mode().bpp(), s.transparent_value());
    let mut keep = vec![false; w as usize * h as usize];
    for y in 0..h {
        for x in 0..w {
            let (cx, cy) = (fl.x + x as i32, fl.y + y as i32);
            let i = (y * w + x) as usize;
            let c = u8::try_from(fl.image.get(x, y)).unwrap_or(u8::MAX);
            if fl.keep[i] && c > 0 && s.bounds().contains(cx, cy) && s.in_selection(cx, cy) {
                image.set(x, y, s.value_of_rgba(fg));
                keep[i] = true;
            }
        }
    }
    Floating {
        x: fl.x,
        y: fl.y,
        image,
        keep,
    }
}

/// O padrão `w × h` amostrado por vizinho até [`PIXEL_BRUSH_THUMB_MAX`] de lado.
pub fn miniatura((w, h): (u32, u32), cells: &[Option<Option<[u8; 4]>>]) -> PixelBrushThumb {
    let lado = w.max(h).max(1);
    let m = PIXEL_BRUSH_THUMB_MAX;
    let (tw, th) = if lado <= m {
        (w, h)
    } else {
        ((w * m).div_ceil(lado), (h * m).div_ceil(lado))
    };
    let cells = (0..th)
        .flat_map(|y| (0..tw).map(move |x| (x, y)))
        .map(|(x, y)| {
            let (sx, sy) = (x * w / tw.max(1), y * h / th.max(1));
            let i = usize::try_from(sy * w + sx).unwrap_or(usize::MAX);
            cells.get(i).copied().flatten().flatten()
        })
        .collect();
    PixelBrushThumb {
        w: tw,
        h: th,
        cells,
    }
}

/// `guardado` com os parâmetros do pincel que ELE guarda tirados de `agora` (as cores ficam).
pub fn funde(guardado: &BrushSlot, agora: &BrushSlot) -> BrushSlot {
    let p = guardado.params;
    let mut s = guardado.clone();
    if p.kind {
        s.kind = agora.kind.clone();
    }
    if p.size {
        s.size = agora.size;
    }
    if p.angle {
        s.angle = agora.angle;
    }
    if p.ink {
        s.ink = agora.ink;
    }
    if p.pixel_perfect {
        s.pixel_perfect = agora.pixel_perfect;
    }
    s
}

/// As escolhas do diálogo *Color Mode* no vocabulário do núcleo.
#[must_use]
pub fn conversao(c: ColorModeChoice) -> (ColorMode, Convert) {
    let mode = match c.target {
        NewSpriteMode::Rgba => ColorMode::Rgba,
        NewSpriteMode::Gray => ColorMode::Gray,
        NewSpriteMode::Indexed => ColorMode::Indexed,
    };
    let dither = match c.dither {
        PixelDither::None => Dither::None,
        PixelDither::Ordered2 => Dither::Ordered(Bayer::M2),
        PixelDither::Ordered4 => Dither::Ordered(Bayer::M4),
        PixelDither::Ordered8 => Dither::Ordered(Bayer::M8),
        PixelDither::Old2 => Dither::Old(Bayer::M2),
        PixelDither::Old4 => Dither::Old(Bayer::M4),
        PixelDither::Old8 => Dither::Old(Bayer::M8),
        PixelDither::FloydSteinberg => Dither::FloydSteinberg,
    };
    let to_gray = match c.gray {
        PixelToGray::Luminance => ToGray::Luminance,
        PixelToGray::Hsv => ToGray::Hsv,
        PixelToGray::Hsl => ToGray::Hsl,
    };
    (mode, Convert { dither, to_gray })
}

pub fn sort_key(k: PixelSortKey) -> SortKey {
    match k {
        PixelSortKey::Hue => SortKey::Hue,
        PixelSortKey::Saturation => SortKey::Saturation,
        PixelSortKey::Brightness => SortKey::Brightness,
        PixelSortKey::Luminance => SortKey::Luminance,
        PixelSortKey::Red => SortKey::Red,
        PixelSortKey::Green => SortKey::Green,
        PixelSortKey::Blue => SortKey::Blue,
        PixelSortKey::Alpha => SortKey::Alpha,
    }
}

/// Os índices que alguma célula usa (Indexed), em ordem de primeira aparição.
pub fn usados(s: &Sprite) -> Vec<usize> {
    let mut vistos = [false; ph2d_pixel_core::palette::MAX_COLORS];
    let mut out = Vec::new();
    for (_, c) in s.entries() {
        for &v in &c.data {
            if !std::mem::replace(&mut vistos[usize::from(v)], true) {
                out.push(usize::from(v));
            }
        }
    }
    out
}
