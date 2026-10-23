//! O A/B do PIGMENTO da banda: a MESMA [`super::bands::stamp_band`] com o plano da LUT e sem ele.

use super::bands::{DabCtx, stamp_band};
use crate::height_film::{FilmAa, FilmLutPlan, film_lut_for};
use crate::{BrushSpec, Falloff};

const SIDE: u32 = 256;
const RADIUS: f32 = 100.0;
const CENTRE: [f32; 2] = [128.0, 128.0];

/// Carimba um dab inteiro pela `stamp_band` do PRODUTO, com o plano da LUT ou sem ele.
fn stamp_once(spec: &BrushSpec, with_lut: bool) -> Vec<u8> {
    let stride = (SIDE as usize) * 4;
    let mut dst = vec![0u8; stride * (SIDE as usize)];
    let film_aa = FilmAa::for_dab(spec, false, RADIUS);
    assert!(film_aa.is_some(), "o fixture tem de ter o AA ligado");
    let footprint = spec.dab_footprint([1.0, 0.0]);
    let held = with_lut.then(|| film_lut_for(spec, RADIUS)).flatten();
    assert_eq!(
        held.is_some(),
        with_lut,
        "o fixture tem de ser admissivel quando pede a LUT"
    );
    let lut = held
        .as_ref()
        .map(|l| FilmLutPlan::new(l, footprint, RADIUS, None));
    let ctx = DabCtx {
        spec,
        tex: None,
        image: None,
        shape: None,
        shape_cor: None,
        ramp: None,
        footprint,
        center: CENTRE,
        cx: CENTRE[0],
        cy: CENTRE[1],
        inv_radius: 1.0 / RADIUS,
        radius: RADIUS,
        coverage: 1.0,
        preserve_alpha: false,
        x0: 0,
        x1: i64::from(SIDE),
        stride,
        film_aa,
        lut,
        capsula: None,
        lei: crate::capsula::Lei::Uniao,
    };
    stamp_band(&ctx, &mut dst, None, None, 0);
    dst
}

/// **O A/B do PIGMENTO: a MESMA [`stamp_band`], com o plano e sem ele.**
///
/// ⚠️ A admissibilidade da LUT é uma afirmação sobre a IMAGEM (é a curvatura da silhueta que decide),
/// então **não existe alavanca que a desligue preservando o desenho** — a única A/B honesta é dirigir
/// a função do produto duas vezes, diferindo SÓ no campo `lut`, e comparar os bytes que ela escreve.
/// É isso que prova que o `w`/`d`/base que a fiação do pigmento passa descrevem a mesma geometria que
/// a cadeia de silhueta percorre.
///
/// A barra é o template do épsilon do passe de luz: **magnitude E contagem**, nunca uma só.
#[test]
fn the_pigment_band_paints_the_same_bytes_with_and_without_the_lut() {
    for falloff in [
        Falloff::Smooth,
        Falloff::Smoother,
        Falloff::Sphere,
        Falloff::Sharp,
        Falloff::Pow4,
        Falloff::Root,
    ] {
        let spec = BrushSpec {
            radius_px: RADIUS,
            falloff,
            impasto: true,
            impasto_depth: 0.5,
            impasto_smooth_edges: true,
            ..Default::default()
        };
        let with = stamp_once(&spec, true);
        let without = stamp_once(&spec, false);
        let (mut worst, mut differing) = (0i32, 0usize);
        for (a, b) in with.iter().zip(&without) {
            let d = i32::from(*a) - i32::from(*b);
            if d != 0 {
                differing += 1;
                worst = worst.max(d.abs());
            }
        }
        assert!(
            worst <= 1,
            "{falloff:?}: pior byte {worst} (limite 1) em {differing} bytes"
        );
        assert!(
            differing <= 256,
            "{falloff:?}: {differing} bytes divergem (limite 256), pior {worst}"
        );
        println!("[lut-pigment] {falloff:?}: {differing} bytes divergem, pior {worst}");
    }
}
