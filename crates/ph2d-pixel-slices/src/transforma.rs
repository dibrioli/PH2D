//! **`Transform`** (decisão do dono, `P11_plano.md` §9.2; medido em G5): ao largar, os píxeis da caixa
//! vão com a slice, só na cel da camada ativa no quadro atual, deixando buraco. Mudar o tamanho escala
//! pelo vizinho mais próximo, como o resto do módulo. Um passo só com a slice.

use ph2d_pixel_core::selection::Mask;
use ph2d_pixel_core::transform::{Floating, Image, lift, resize_nearest, stamp};
use ph2d_pixel_core::{CelKey, Op, Rect, SliceId, SliceShape, Sprite, apply};

/// O par de caixas (antes, depois) das slices que mudaram de caixa neste gesto.
#[must_use]
pub fn caixas_que_mudam(
    s: &Sprite,
    frame: u32,
    previa: &[(SliceId, SliceShape)],
) -> Vec<(Rect, Rect)> {
    previa
        .iter()
        .filter_map(|(id, k)| {
            let antes = s.slice(*id)?.shape_at(frame)?.bounds;
            (antes != k.bounds).then_some((antes, k.bounds))
        })
        .collect()
}

/// Os píxeis de `de` em `para`: `None` se nada muda. Fora do canvas não há píxeis: a parte visível
/// fica no seu lugar dentro da caixa, e o que sai do canvas ao largar perde-se.
fn um(s: &Sprite, key: CelKey, de: Rect, para: Rect, fundo: u32) -> Option<Op> {
    let m = Mask::from_rect(s.width(), s.height(), de);
    let f = lift(s, key, Some(&m))?;
    let mut caixa = Image::new(de.w, de.h, s.mode().bpp(), s.transparent_value());
    for y in 0..f.image.h {
        for x in 0..f.image.w {
            let at = ((f.x - de.x) as u32 + x, (f.y - de.y) as u32 + y);
            caixa.set(at.0, at.1, f.image.get(x, y));
        }
    }
    let image = if (de.w, de.h) == (para.w, para.h) {
        caixa
    } else {
        resize_nearest(&caixa, para.w, para.h)
    };
    let solta = Floating {
        x: para.x,
        y: para.y,
        keep: vec![true; para.area()],
        image,
    };
    stamp(s, key, &solta, Some(&m), None, fundo)
}

/// **O passo inteiro**: `slices` e os píxeis de cada caixa, em sequência, para caixas que se sobrepõem
/// verem o resultado da anterior; sem caixas, `slices` só. `fundo` é o valor do buraco numa camada
/// Background.
#[must_use]
pub fn com_pixels(s: &Sprite, slices: Op, key: CelKey, caixas: &[(Rect, Rect)], fundo: u32) -> Op {
    if caixas.is_empty() {
        return slices;
    }
    let mut copia = s.clone();
    let mut ops = vec![slices];
    for (de, para) in caixas {
        let Some(op) = um(&copia, key, *de, *para, fundo) else {
            continue;
        };
        if apply(&mut copia, op.clone()).is_ok() {
            ops.push(op);
        }
    }
    Op::Batch(ops)
}
