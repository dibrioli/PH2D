//! **`Transform`** (decisão do dono, `P11_plano.md` §9.2; medido em G5): os píxeis da caixa seguem a
//! slice ao vivo no arrasto ([`previa`]) e vão com ela ao largar, só na cel da camada ativa no quadro atual, deixando buraco. Mudar o tamanho escala
//! pelo vizinho mais próximo, como o resto do módulo. Um passo só com a slice.

use crate::{Gesto, no_quadro};
use ph2d_pixel_core::selection::Mask;
use ph2d_pixel_core::transform::{Floating, Image, lift, resize_nearest, stamp};
use ph2d_pixel_core::{CelKey, Op, Rect, SliceId, SliceKey, SliceShape, Sprite, Stroke, apply};

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

/// Os ops dos píxeis de cada caixa, em sequência, para caixas que se sobrepõem verem o resultado da
/// anterior. `fundo` é o valor do buraco numa camada Background.
fn pixels(s: &Sprite, key: CelKey, caixas: &[(Rect, Rect)], fundo: u32) -> Vec<Op> {
    let mut copia = s.clone();
    let mut ops = Vec::new();
    for (de, para) in caixas {
        let Some(op) = um(&copia, key, *de, *para, fundo) else {
            continue;
        };
        if apply(&mut copia, op.clone()).is_ok() {
            ops.push(op);
        }
    }
    ops
}

/// A cel é LIGADA (§10): os píxeis mudam em cada quadro que a partilha, e a slice também, onde lá
/// caía na MESMA caixa que no quadro de `key`; um quadro com outra caixa não emoldurava esta arte.
fn nos_ligados(s: &Sprite, slices: Op, key: CelKey) -> Op {
    let (Op::Slices(novas), Some(aqui)) = (&slices, s.frame_index(key.1)) else {
        return slices;
    };
    let dono = s.owner(key);
    let ligados: Vec<u32> = (0..s.frames().len())
        .filter(|&i| i != aqui && s.entry((key.0, s.frames()[i].id)).is_some())
        .filter(|&i| s.owner((key.0, s.frames()[i].id)) == dono)
        .map(|i| i as u32)
        .collect();
    let mut out = novas.clone();
    for c in &mut out {
        let Some(velha) = s.slice(c.id) else { continue };
        let (Some(era), Some(fica)) = (velha.shape_at(aqui as u32), c.shape_at(aqui as u32)) else {
            continue;
        };
        for &f in ligados
            .iter()
            .filter(|&&f| era != fica && velha.shape_at(f) == Some(era))
        {
            match c.keys.binary_search_by_key(&f, |k| k.frame) {
                Ok(i) => c.keys[i].shape = fica,
                Err(i) => c.keys.insert(
                    i,
                    SliceKey {
                        frame: f,
                        shape: fica,
                    },
                ),
            }
        }
        c.keys.dedup_by(|b, a| a.shape == b.shape);
    }
    Op::Slices(out)
}

/// **O passo inteiro**: `slices` (também nos quadros que partilham a cel) e os píxeis de cada
/// caixa; sem caixas, `slices` só.
#[must_use]
pub fn com_pixels(s: &Sprite, slices: Op, key: CelKey, caixas: &[(Rect, Rect)], fundo: u32) -> Op {
    if caixas.is_empty() {
        return slices;
    }
    let mut ops = vec![nos_ligados(s, slices, key)];
    ops.extend(pixels(s, key, caixas, fundo));
    Op::Batch(ops)
}

/// **A arte a meio do arrasto** `g`: o traço com os MESMOS píxeis que [`com_pixels`] grava ao
/// largar (a pré-visualização é o passo); `None` sem caixas que mudem.
#[must_use]
pub fn previa(s: &Sprite, key: CelKey, g: &Gesto, fundo: u32) -> Option<Stroke> {
    let caixas = caixas_que_mudam(s, no_quadro(s, key.1), &g.previa());
    let op = Op::Batch((!caixas.is_empty()).then(|| pixels(s, key, &caixas, fundo))?);
    Some(Stroke::of_patches(key.0, key.1, &op, s.mode().bpp()))
}
