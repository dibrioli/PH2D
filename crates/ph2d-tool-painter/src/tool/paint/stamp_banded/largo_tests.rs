//! B29.1 na BANDA: o lote do pincel liso leva o [`ph2d_painter_brush::TexelLargo`] e dá o mesmo byte a 1, 2 e 5 fios
//! (cada texel é composto pelos mesmos dabs, na mesma ordem); o `u8` continua a estancar com Flow baixo.

use super::{FIOS, banded_com_forca};
use ph2d_painter_brush::{BrushSpec, Dab, Tecto, TexelLargo};

const W: u32 = 96;
const H: u32 = 160;

/// Preto sobre branco, Strength 1, Flow 2 %, Accumulate OFF, sem cápsula: o pincel liso da banda.
fn liso() -> BrushSpec {
    BrushSpec {
        radius_px: 20.0,
        hardness: 1.0,
        color: [0.0, 0.0, 0.0],
        strength: 1.0,
        flow: 0.02,
        accumulate: false,
        continuous_spacing: false,
        ..BrushSpec::default()
    }
}

/// 60 idas e voltas numa coluna: cada banda corta a coluna e cada texel do meio recebe centenas de dabs.
#[allow(clippy::cast_precision_loss)]
fn coluna() -> Vec<Dab> {
    (0..60 * 36)
        .map(|i| {
            let k = i % 36;
            let y = 20.0 + 7.0 * if k < 18 { k } else { 35 - k } as f32;
            Dab {
                center: [48.0, y],
                radius_px: 20.0,
                coverage: 1.0,
                color: [0.0, 0.0, 0.0],
                color_shift: [0.0; 3],
                rotation: [1.0, 0.0],
                dir: [0.0, 1.0],
                arc_len: 0.0,
                stroke_radius_px: 20.0,
                step: [0.0, 0.0],
                ante: None,
                tilt: None,
            }
        })
        .collect()
}

fn carimba<M: Tecto + Default>(fios: Option<usize>) -> Vec<u8> {
    let mut buf = vec![255u8; (W * H * 4) as usize];
    let mut mask = vec![M::default(); (W * H) as usize];
    FIOS.with(|f| f.set(fios));
    let min_area = if fios.is_some() { 0 } else { usize::MAX };
    let planos = (Some(mask.as_mut_slice()), None);
    let _ = banded_com_forca(&mut buf, W, H, &coluna(), &liso(), false, planos, min_area);
    FIOS.with(|f| f.set(None));
    buf
}

/// ⭐ O tecto largo na banda: o mesmo byte a 1 (o laço serial) · 2 · 5 fios, e o texel do meio chega ao alvo (0) a
/// ≤ 1 nível; no MESMO teste o `u8` estanca (> 10) — e também dá o mesmo a 1 e 5 fios (a lei que shipou, ao bit).
#[test]
fn com_o_tecto_largo_a_banda_chega_ao_alvo_e_nao_depende_dos_fios() {
    let meio = ((80 * W + 48) * 4) as usize;
    let largo = carimba::<TexelLargo>(None);
    for n in [2, 5] {
        assert!(
            carimba::<TexelLargo>(Some(n)) == largo,
            "o tecto largo a {n} fios difere do serial"
        );
    }
    let byte = carimba::<u8>(None);
    assert!(
        carimba::<u8>(Some(5)) == byte,
        "o u8 a 5 fios difere do serial"
    );
    let leitura = format!("meio: largo {} · u8 {}", largo[meio], byte[meio]);
    eprintln!("B29.1 banda: {leitura}");
    assert!(
        largo[meio] <= 1,
        "o tecto largo devia chegar ao alvo: {leitura}"
    );
    assert!(
        byte[meio] > 10,
        "o u8 não estancou: a fixture não tem o fenómeno: {leitura}"
    );
}
