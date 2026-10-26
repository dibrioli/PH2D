//! **Edit ▸ Flip / Rotate / Shift e a área de transferência** (onda P6), leis do Trial em
//! `oracle_trial_selection.json`. Cada função devolve UM passo (`None` = nada a fazer); `bg` é o
//! valor (no modo do sprite) que uma área limpa recebe numa camada Background — a cor de fundo da
//! barra de cores.

use crate::affine::{Affine, Resample, transform_image};
use crate::compose::compose_rgba;
use crate::op::Op;
use crate::selection::Mask;
use crate::sprite::{CelKey, FrameId, Rect, Sprite};
use crate::transform::{
    Floating, Image, flip_h, flip_v, lift, offset_wrap, resize_nearest_centred, rot90ccw, rot90cw,
    rot180, stamp,
};

/// Um giro de múltiplo de 90°.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    Cw90,
    Ccw90,
    R180,
}

/// Onde cai o bloco `h × w` de um giro de 90° da caixa `r`: `(floor(cx − h/2), floor(cy − w/2))`,
/// `c` = centro da caixa (Trial `edit_rot90*_rect`, `xform_*_rot90`).
#[must_use]
pub fn rot90_origin(r: Rect) -> (i32, i32) {
    // 2·cx − h = 2x + w − h; o `floor` de metade é o `div_euclid` por 2.
    let x = (2 * i64::from(r.x) + i64::from(r.w) - i64::from(r.h)).div_euclid(2);
    let y = (2 * i64::from(r.y) + i64::from(r.h) - i64::from(r.w)).div_euclid(2);
    (x as i32, y as i32)
}

fn selection_after(s: &Sprite, f: &Floating) -> Option<Option<Mask>> {
    s.mask().map(|_| Some(f.mask(s.width(), s.height())))
}

/// Assenta `f` limpando primeiro `original` (a máscara de onde ele saiu); a selecção passa a ser
/// a forma de `f`. UM passo — o fecho de uma transformação.
#[must_use]
pub fn commit(s: &Sprite, key: CelKey, original: &Mask, f: &Floating, bg: u32) -> Option<Op> {
    stamp(
        s,
        key,
        f,
        Some(original),
        Some(Some(f.mask(s.width(), s.height()))),
        bg,
    )
}

fn lifted(s: &Sprite, key: CelKey) -> Option<(Mask, Floating)> {
    let m = s
        .mask()
        .cloned()
        .unwrap_or_else(|| Mask::full(s.width(), s.height()));
    let f = lift(s, key, Some(&m))?;
    Some((m, f))
}

/// **Edit ▸ Flip Horizontal/Vertical**: espelha dentro da caixa da selecção (sem selecção, o
/// sprite inteiro — igual ao Flip Canvas).
#[must_use]
pub fn flip(s: &Sprite, key: CelKey, horizontal: bool, bg: u32) -> Option<Op> {
    let (m, f) = lifted(s, key)?;
    let (image, keep) = f.permuted(if horizontal { flip_h } else { flip_v });
    let g = Floating { image, keep, ..f };
    stamp(s, key, &g, Some(&m), selection_after(s, &g), bg)
}

/// **Edit ▸ Rotate**: 180° no lugar; 90° de uma selecção `w × h` vira o bloco `h × w` em
/// [`rot90_origin`]; sem selecção, o conteúdo gira como o canvas giraria (`x' = H−1−y, y' = x` no
/// horário), com o tamanho do canvas mantido (cortado).
#[must_use]
pub fn rotate(s: &Sprite, key: CelKey, r: Rotation, bg: u32) -> Option<Op> {
    let (m, f) = lifted(s, key)?;
    let op = match r {
        Rotation::Cw90 => rot90cw,
        Rotation::Ccw90 => rot90ccw,
        Rotation::R180 => rot180,
    };
    let (image, keep) = f.permuted(op);
    let (x, y) = match (r, s.mask()) {
        (Rotation::R180, _) => (f.x, f.y),
        (_, Some(_)) => rot90_origin(f.rect()),
        (_, None) => (0, 0),
    };
    let g = Floating { x, y, image, keep };
    stamp(s, key, &g, Some(&m), selection_after(s, &g), bg)
}

/// **Edit ▸ Shift**: um passo de `(dx, dy)` com volta dentro da caixa da selecção (pede selecção).
#[must_use]
pub fn shift(s: &Sprite, key: CelKey, dx: i32, dy: i32, bg: u32) -> Option<Op> {
    let b = s.mask()?.bounds()?;
    let m = Mask::from_rect(s.width(), s.height(), b);
    let f = lift(s, key, Some(&m))?;
    let g = Floating {
        image: offset_wrap(&f.image, dx, dy),
        ..f
    };
    stamp(s, key, &g, Some(&m), None, bg)
}

/// **A escala pelos campos w/h da transformação** (Trial): fonte `floor((d + ½)·S/N)`, o bloco em
/// `floor(c − N/2)` com `c` = centro da caixa.
#[must_use]
pub fn scale_fields(f: &Floating, w: u32, h: u32) -> Floating {
    let keep = Image::from_values(
        f.image.w,
        f.image.h,
        1,
        &f.keep.iter().map(|&k| u32::from(k)).collect::<Vec<_>>(),
    );
    let x = (2 * i64::from(f.x) + i64::from(f.image.w) - i64::from(w)).div_euclid(2) as i32;
    let y = (2 * i64::from(f.y) + i64::from(f.image.h) - i64::from(h)).div_euclid(2) as i32;
    Floating {
        x,
        y,
        image: resize_nearest_centred(&f.image, w, h),
        keep: resize_nearest_centred(&keep, w, h)
            .values()
            .iter()
            .map(|&v| v != 0)
            .collect(),
    }
}

/// **A moldura afim** aplicada a células levantadas (a matriz é a do MVP, em células do
/// documento; `t` leva o canto ↖ de `f` à sua posição). `None` = espelha ou degenerou.
#[must_use]
pub fn transform(f: &Floating, t: &Affine, method: Resample, transparent: u32) -> Option<Floating> {
    let (r, image) = transform_image(&f.image, t, method, transparent, None)?;
    let keep_img = Image::from_values(
        f.image.w,
        f.image.h,
        1,
        &f.keep.iter().map(|&k| u32::from(k)).collect::<Vec<_>>(),
    );
    let (_, keep) = transform_image(&keep_img, t, method, 0, None)?;
    Some(Floating {
        x: r.x,
        y: r.y,
        image,
        keep: keep.values().iter().map(|&v| v != 0).collect(),
    })
}

/// **Edit ▸ Copy**: as células do cel activo sob a selecção (fora dela, transparente), na posição
/// de onde saíram. Pede selecção.
#[must_use]
pub fn copy(s: &Sprite, key: CelKey) -> Option<Floating> {
    lift(s, key, Some(s.mask()?))
}

/// **Edit ▸ Copy Merged**: o mesmo sobre as camadas visíveis compostas (no modo do sprite).
#[must_use]
pub fn copy_merged(s: &Sprite, frame: FrameId) -> Option<Floating> {
    let m = s.mask()?;
    let b = m.bounds()?;
    let mut rgba = Vec::new();
    compose_rgba(s, frame, None, &mut rgba);
    let w = s.width() as usize;
    let mut image = Image::new(b.w, b.h, s.mode().bpp(), s.transparent_value());
    let mut keep = vec![false; b.area()];
    for y in 0..b.h {
        for x in 0..b.w {
            let (cx, cy) = (b.x + x as i32, b.y + y as i32);
            if m.get(cx, cy) {
                let i = (cy as usize * w + cx as usize) * 4;
                let c = [rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]];
                image.set(x, y, s.value_of_rgba(c));
                keep[(y * b.w + x) as usize] = true;
            }
        }
    }
    Some(Floating {
        x: b.x,
        y: b.y,
        image,
        keep,
    })
}

/// **Edit ▸ Paste** assentado: as células guardadas em `f` (a posição copiada, ou para onde a
/// mão as levou) por cima do cel; a selecção passa a ser a forma colada.
#[must_use]
pub fn paste(s: &Sprite, key: CelKey, f: &Floating, bg: u32) -> Option<Op> {
    stamp(
        s,
        key,
        f,
        None,
        Some(Some(f.mask(s.width(), s.height()))),
        bg,
    )
}
