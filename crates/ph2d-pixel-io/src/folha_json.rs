//! **P14e-2 — a folha que o Pixel exporta volta a ser o documento** (`docs/PixelArt/P14_plano.md`
//! §11 P14e-2): `folha.png` + `folha.json` (+ `folha_n.png`) ⇒ um [`Sprite`] com os quadros (com o
//! recorte desfeito), as durações, as tags, as slices (com `ph2d.collider` à parte), o pivô por
//! quadro (`meta.pivots`) e a camada Normal Map. Quem deriva o import é o MESMO de um `.ase`.

use ph2d_pixel_core::codec::SpriteParts;
use ph2d_pixel_core::{
    Cel, ColorMode, Frame, FrameId, Layer, LayerId, Op, Purpose, Rect, Slice, SliceId, SliceKey,
    SliceShape, Sprite, UserData, apply, compose_window, palette,
};

use crate::abrir::{ErroAbrir, abrir};
use crate::ase::Colliders;

/// RGBA de uma imagem PNG (indexada ou não), pelo leitor do File ▸ Open.
fn rgba_do_png(png: &[u8]) -> Result<(u32, u32, Vec<u8>), ErroAbrir> {
    let (s, _) = abrir(png, "png")?;
    let f = s.frames().first().ok_or(ErroAbrir::Vazio)?.id;
    let px = compose_window(&s, f, None, s.bounds(), Purpose::Export);
    Ok((s.width(), s.height(), px))
}

/// O quadro `q` do JSON desfeito do recorte: `sourceSize`, com o `frame` da folha colado em
/// `spriteSourceSize`.
fn quadro_de(q: &serde_json::Value, folha: &(u32, u32, Vec<u8>)) -> Option<(u32, u32, Vec<u8>)> {
    let n = |o: &serde_json::Value, k: &str| u32::try_from(o[k].as_u64()?).ok();
    let (fr, src, tam) = (&q["frame"], &q["spriteSourceSize"], &q["sourceSize"]);
    let (w, h) = (n(tam, "w")?, n(tam, "h")?);
    let (fx, fy, fw, fh) = (n(fr, "x")?, n(fr, "y")?, n(fr, "w")?, n(fr, "h")?);
    let (sx, sy, sw, sh) = (n(src, "x")?, n(src, "y")?, n(src, "w")?, n(src, "h")?);
    let (iw, ih, px) = folha;
    if fx + fw > *iw || fy + fh > *ih || sx + sw > w || sy + sh > h || sw > fw || sh > fh {
        return None;
    }
    // O `inner` (Inner padding) cerca a peça por igual: ela está no meio do `frame`.
    let (fx, fy) = (fx + (fw - sw) / 2, fy + (fh - sh) / 2);
    let mut d = vec![0_u8; w as usize * h as usize * 4];
    for y in 0..sh {
        let de = ((fy + y) as usize * *iw as usize + fx as usize) * 4;
        let para = ((sy + y) as usize * w as usize + sx as usize) * 4;
        d[para..para + sw as usize * 4].copy_from_slice(&px[de..de + sw as usize * 4]);
    }
    Some((w, h, d))
}

/// O cel apertado de uma imagem RGBA; `None` = só transparente.
fn cel(px: &[u8], w: u32, h: u32) -> Option<Cel> {
    crate::abrir::apertada(px, w, h, 4, &|c| c[3] == 0).map(|(b, d)| Cel::image(b, d))
}

/// **Abre a folha exportada**: o documento e as slices marcadas `ph2d.collider` (à parte, como no
/// `.ase`: o collider é do mundo).
///
/// # Errors
/// [`ErroAbrir`]: o JSON não se lê, a imagem não é a dele, ou os quadros não têm um tamanho só.
pub fn abrir_folha_json(
    png: &[u8],
    json: &str,
    normal_png: Option<&[u8]>,
) -> Result<(Sprite, Colliders), ErroAbrir> {
    let j: serde_json::Value =
        serde_json::from_str(json).map_err(|e| ErroAbrir::Ler(e.to_string()))?;
    let folha = rgba_do_png(png)?;
    let normal = normal_png.map(rgba_do_png).transpose()?;
    let mal = || ErroAbrir::Ler("sheet json".to_owned());
    let quadros = crate::abrir::quadros_do_json(&j);
    if quadros.is_empty() {
        return Err(ErroAbrir::Vazio);
    }
    let (cor, nor) = (LayerId(1), LayerId(2));
    let mut frames = Vec::with_capacity(quadros.len());
    let mut cels = Vec::new();
    let mut lados = None;
    for (i, q) in quadros.iter().enumerate() {
        let (w, h, px) = quadro_de(q, &folha).ok_or_else(mal)?;
        if *lados.get_or_insert((w, h)) != (w, h) {
            return Err(mal());
        }
        let id = FrameId(3 + i as u32);
        let ms = q["duration"].as_u64().and_then(|m| u32::try_from(m).ok());
        frames.push(Frame {
            id,
            duration_ms: ms.unwrap_or(ph2d_pixel_core::DEFAULT_FRAME_MS).max(1),
        });
        cels.extend(cel(&px, w, h).map(|c| ((cor, id), c)));
        if let Some(n) = &normal {
            let (_, _, npx) = quadro_de(q, n).ok_or_else(mal)?;
            cels.extend(cel(&npx, w, h).map(|c| ((nor, id), c)));
        }
    }
    let (w, h) = lados.ok_or_else(mal)?;
    let mut layers = vec![Layer::image(cor, "Layer 1")];
    if normal.is_some() {
        layers.push(Layer {
            normal: true,
            ..Layer::image(nor, "Normal Map")
        });
    }
    let primeiro = 3 + quadros.len() as u32;
    let mut colliders = Colliders::new();
    let lidas: Vec<_> = j["meta"]["slices"].as_array().map_or_else(Vec::new, |v| {
        v.iter()
            .filter_map(ph2d_sprite_sheet::aseprite::JsonSlice::ler)
            .collect()
    });
    // `meta.slices` sai na ordem inversa à do documento (F5).
    let slices = lidas
        .into_iter()
        .rev()
        .enumerate()
        .map(|(i, js)| {
            let id = SliceId(primeiro + i as u32);
            if js.collider {
                colliders.insert(id.0);
            }
            let r = |(x, y, w, h): (i32, i32, u32, u32)| Rect::new(x, y, w, h);
            Slice {
                id,
                name: js.name,
                user: UserData {
                    color: js.color.unwrap_or([0; 4]),
                    text: js.data,
                    ..UserData::default()
                },
                keys: js
                    .keys
                    .into_iter()
                    .map(|k| SliceKey {
                        frame: k.frame,
                        shape: SliceShape {
                            bounds: r(k.bounds),
                            center: k.center.map(r),
                            pivot: k.pivot,
                        },
                    })
                    .collect(),
            }
        })
        .collect();
    let mut s = Sprite::from_parts(SpriteParts {
        width: w,
        height: h,
        mode: Some(ColorMode::Rgba),
        palette: palette::DEFAULT.to_vec(),
        transparent: 0,
        layers,
        frames,
        cels,
        tilesets: Vec::new(),
        slices,
        user: UserData::default(),
        tags: Vec::new(),
    })
    .map_err(ErroAbrir::Documento)?;
    crate::abrir::poe_tags(&mut s, &j, quadros.len());
    poe_pivo(&mut s, &j);
    Ok((s, colliders))
}

/// O pivô de cada quadro (`meta.pivots`) volta como a slice do pivô: a primeira cujo pivô dá o de
/// TODOS os quadros. Sem `meta.pivots` (ou sem slice que o dê), o centro de fábrica.
fn poe_pivo(s: &mut Sprite, j: &serde_json::Value) {
    let Some(p) = j["meta"]["pivots"].as_array() else {
        return;
    };
    let quer: Vec<Option<(f64, f64)>> = p
        .iter()
        .map(|v| Some((v["x"].as_f64()?, v["y"].as_f64()?)))
        .collect();
    let ids: Vec<SliceId> = s.slices().iter().map(|c| c.id).collect();
    for id in ids {
        let mut t = s.clone();
        if apply(&mut t, Op::PivotSlice(Some(id))).is_err() {
            continue;
        }
        let da = (0..quer.len() as u32).map(|i| Some(ph2d_pixel_core::slice::pivot_px(&t, i)));
        if da.eq(quer.iter().copied()) {
            *s = t;
            return;
        }
    }
}
