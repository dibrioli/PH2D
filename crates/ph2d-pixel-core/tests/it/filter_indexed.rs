//! **Invert e Gradient num sprite Indexed** — o NOSSO comportamento. A rodada 3 no Trial
//! (`oracle_trial_round3.json`) mediu falhas, não funções: `B1_invert_indexed` (o preview troca as
//! cores mas o OK não grava nada) e `B2_gradient_indexed` (o arrasto não pinta). Recusamos copiá-las:
//! Invert grava o índice mais próximo da cor invertida e o Gradient pinta (Palette ramp = degraus).

use ph2d_pixel_core::filter::adjust::Invert;
use ph2d_pixel_core::filter::{self, Channels};
use ph2d_pixel_core::gradient::{Gradient, Shape, palette_ramp};
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Op, Patch, Sprite, apply};
use serde_json::Value;

/// A paleta `IDX_PAL` da rodada 3 (16 cores).
fn idx_pal() -> Vec<[u8; 4]> {
    let d: Value = serde_json::from_str(include_str!("../fixtures/oracle_trial_round3.json"))
        .expect("fixtura");
    d["B1_invert_indexed"]["data"]["first_ok_only"]["palette"]
        .as_array()
        .expect("paleta")
        .iter()
        .map(|c| {
            std::array::from_fn(|k| {
                if k == 3 {
                    255
                } else {
                    c[k].as_u64().unwrap() as u8
                }
            })
        })
        .collect()
}

/// `w × h` Indexed com a paleta `IDX_PAL` e o índice `v(x, y)` em cada célula.
fn indexed(w: u32, h: u32, v: impl Fn(u32, u32) -> u8) -> Sprite {
    let mut s = Sprite::with_palette(
        NewSprite {
            width: w,
            height: h,
            mode: ColorMode::Indexed,
            background: Background::Transparent,
        },
        &idx_pal(),
    )
    .expect("sprite");
    let r = s.bounds();
    let (layer, frame) = (s.layers()[0].id, s.frames()[0].id);
    let pixels = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| v(x, y))
        .collect();
    apply(
        &mut s,
        Op::Patch(Patch {
            layer,
            frame,
            bbox: Some(r),
            rect: r,
            pixels,
        }),
    )
    .expect("patch");
    s
}

fn nearest(pal: &[[u8; 4]], skip: usize, c: [u8; 4]) -> u32 {
    let d = |p: &[u8; 4]| {
        (0..3)
            .map(|k| (i32::from(p[k]) - i32::from(c[k])).pow(2))
            .sum::<i32>()
    };
    (0..pal.len())
        .filter(|&i| i != skip)
        .min_by_key(|&i| (d(&pal[i]), i))
        .unwrap() as u32
}

/// `idx.ase` (16×1, célula i = índice i): o OK GRAVA — cada célula opaca vira o índice mais
/// próximo de `255 − cor`; o transparente fica transparente.
#[test]
fn invert_on_indexed_writes_the_nearest_index_of_the_inverted_colour() {
    let mut s = indexed(16, 1, |x, _| x as u8);
    let key = (s.layers()[0].id, s.frames()[0].id);
    let pal = s.palette().to_vec();
    let t = usize::from(s.transparent_index());
    let op = filter::apply(&s, &[key], Channels::ALL, &Invert).expect("Invert em Indexed grava");
    apply(&mut s, op).expect("aplica");
    let mut changed = 0;
    for x in 0..16 {
        let got = s.value_at(key, x, 0);
        if x as usize == t {
            assert_eq!(got, t as u32, "o transparente fica");
            continue;
        }
        let c = pal[x as usize];
        let want = nearest(&pal, t, [255 - c[0], 255 - c[1], 255 - c[2], 255]);
        assert_eq!(got, want, "célula {x}");
        changed += usize::from(got != x as u32);
    }
    assert!(changed >= 12, "Invert mudou só {changed} células");
}

/// `idx32.ase` (32×8, tudo índice 0), fg = índice 2, bg = índice 9, arrasto (0,4)→(31,4): pinta
/// a região inteira; com Palette ramp cada célula é um degrau 2..=9, pontas cravadas.
#[test]
fn gradient_paints_on_indexed_and_the_palette_ramp_steps_through_the_entries() {
    let s = indexed(32, 8, |_, _| 0);
    let pal = s.palette().to_vec();
    let g = Gradient {
        shape: Shape::Linear,
        from: (0, 4),
        to: (31, 4),
        fg: pal[2],
        bg: pal[9],
        dither: ph2d_pixel_core::dither::Dither::None,
    };
    let ramp = palette_ramp(&pal, 2, 9);
    let v = g.ramp_values(&s, &ramp);
    assert_eq!(v.len(), 32 * 8, "a região inteira");
    assert!(v.iter().all(|&(_, i)| (2..=9).contains(&i)), "{v:?}");
    let at = |p| v.iter().find(|q| q.0 == p).map(|q| q.1);
    assert_eq!(at((0, 4)), Some(2));
    assert_eq!(at((31, 4)), Some(9));
    let distinct: std::collections::BTreeSet<u32> = v.iter().map(|q| q.1).collect();
    assert_eq!(distinct.len(), 8, "os oito degraus aparecem");
    let solid = g.values(&s);
    assert_eq!(solid.len(), 32 * 8, "sem a rampa também pinta");
    assert!(
        solid
            .iter()
            .all(|&(_, i)| i != u32::from(s.transparent_index()))
    );
}
