//! Os gates do [`super`] — o campo de deslocamento do esfregão. Cortados do módulo pelo tecto de LOC
//! (2026-10-01), quando as linhas de um pingo passaram a compor-se por faixas.

use super::*;
use crate::height::HeightDab;

fn dab_at(center: [f32; 2], radius: f32) -> HeightDab<'static> {
    HeightDab {
        center,
        radius,
        coverage: 1.0,
        footprint: crate::footprint::FootprintDeform::identity(),
        prev_center: None,
        shape: None,
        grain: None,
        grain_image: None,
    }
}

fn spec(radius: f32) -> BrushSpec {
    BrushSpec {
        radius_px: radius,
        flow: 1.0,
        strength: 1.0,
        ..Default::default()
    }
}

/// ⛔⛔ **O INSTRUMENTO DA RECUSA GRAMPEIA, E O PRODUTO NÃO** — as duas metades, porque
/// cada uma sozinha mente.
///
/// O cabeçalho deste módulo afirma que *«o `disp` de um texel só cresce enquanto o cursor
/// está a menos de um raio dele»*, e a composição `D(p) = v + D(p − v)` **não o cumpre**: um
/// texel herda o mapa de quem está atrás, e a corrente alcança arbitrariamente longe. O tecto
/// que honra a frase existe e foi **medido e recusado** — ver
/// [`TECTO_MEDIDO_E_RECUSADO_EM_RAIOS`], porque ele mata o transporte longo que o dono exigiu.
///
/// ⚠️ Este gate não escolhe: ele afirma que o instrumento **funciona** (com o tecto, o
/// transporte pára no alcance do dab) e que o produto **não o usa** (sem ele, a corrente
/// alcança muito mais). *Sem a segunda metade, alguém leria a recusa como já aplicada.*
///
/// **Mutações que sangram:** apagar o grampo · `let k = 1.0` · trocar o `>` por `<`.
#[test]
fn o_instrumento_da_recusa_grampeia_e_o_produto_nao() {
    let (w, h) = (64u32, 64u32);
    let n = (w * h) as usize;
    let raio = 8.0f32;
    let s = spec(raio);
    let anda = |tecto: f32| -> f32 {
        let mut sc = SmearScratch::default();
        let mut disp = vec![[0.0f32; 2]; n];
        // Um traço LONGO de propósito: 48 passos de 1 px, seis vezes o diâmetro do dab.
        for k in 0..48u32 {
            let x = 8.0 + k as f32;
            let _ = accumulate_dab_smear(
                SmearOut {
                    disp: &mut disp,
                    scratch: &mut sc,
                },
                Transporte {
                    step: [1.0, 0.0],
                    tecto_em_raios: tecto,
                    arco: None,
                },
                None,
                w,
                h,
                &s,
                &dab_at([x, 32.0], raio),
            );
        }
        disp.iter().map(|d| d[0].hypot(d[1])).fold(0.0, f32::max)
    };
    let com = anda(TECTO_MEDIDO_E_RECUSADO_EM_RAIOS);
    let sem = anda(SEM_TECTO);
    let alcance = 2.0 * raio;
    assert!(
        com <= alcance + 1e-3,
        "o transporte passou o alcance do dab: {com:.2} px contra {alcance:.2}"
    );
    // A metade do PRODUTO: ele passa `SEM_TECTO`, logo a corrente tem de disparar. Isto é
    // ao mesmo tempo o controlo positivo da fixtura e a afirmação de que a recusa não foi
    // aplicada às escondidas.
    assert!(
        sem > 2.0 * alcance,
        "o produto passou a grampear (ou a fixtura não contém o fenómeno): sem tecto a \
         corrente só chegou a {sem:.2} px, contra o alcance de {alcance:.2}"
    );
}

/// **The law the whole fix rests on: transport is a SUM, so it does not depend on how finely the
/// motion was sampled.** Walk the same 32 px with 32 one-pixel steps and with 8 four-pixel steps —
/// the displacement that lands on the axis must be the same distance, not a different one.
///
/// The kernel this replaces fails exactly here: `h·wⁿ` depends on `n`.
///
/// **Mutation that must bleed:** make the sink `disp[i] = step·add` (assign, not add) — the coarse
/// walk then reports one step's worth and the two disagree by 4×.
#[test]
fn transport_is_a_sum_so_the_dab_spacing_cannot_change_it() {
    let (w, h) = (64u32, 64u32);
    let n = (w * h) as usize;
    let s = spec(8.0);
    let walk = |stride: f32| {
        let mut sc = SmearScratch::default();
        let mut disp = vec![[0.0f32; 2]; n];
        let steps = (32.0 / stride) as u32;
        for k in 0..steps {
            let x = 16.0 + stride * (k as f32 + 1.0);
            let _ = accumulate_dab_smear(
                SmearOut {
                    disp: &mut disp,
                    scratch: &mut sc,
                },
                Transporte {
                    step: [stride, 0.0],
                    tecto_em_raios: TECTO_MEDIDO_E_RECUSADO_EM_RAIOS,
                    arco: None,
                },
                None,
                w,
                h,
                &s,
                &dab_at([x, 32.0], 8.0),
            );
        }
        disp
    };
    let fine = walk(1.0);
    let coarse = walk(4.0);
    // On the axis, mid-trail: the brush centre passed over this texel, so the weight is ~1 and the
    // displacement should be ~the distance travelled while it was under the brush.
    let i = (32 * w + 32) as usize;
    assert!(
        fine[i][0] > 8.0,
        "the fine walk must actually transport (got {})",
        fine[i][0]
    );
    let ratio = coarse[i][0] / fine[i][0];
    assert!(
        (0.8..1.25).contains(&ratio),
        "same 32 px of motion, 4× the sampling: fine displaced {} px, coarse {} px (ratio {ratio}). \
         Transport that depends on the spacing is the product law this kernel exists to replace",
        fine[i][0],
        coarse[i][0]
    );
}

/// **The trail has the brush's WIDTH.** Off the drag axis the falloff makes the displacement
/// smaller, but it must not make it *zero* — that is the filament. Across the trail, the band of
/// texels that moved at all should span roughly the brush's diameter.
///
/// **Mutation that must bleed:** restore a per-step lerp toward the previous result — the off-axis
/// column collapses to the centre texel.
#[test]
fn the_displaced_band_is_as_wide_as_the_brush() {
    let (w, h) = (96u32, 96u32);
    let n = (w * h) as usize;
    let s = spec(10.0);
    let mut sc = SmearScratch::default();
    let mut disp = vec![[0.0f32; 2]; n];
    for k in 0..48u32 {
        let x = 20.0 + k as f32;
        let _ = accumulate_dab_smear(
            SmearOut {
                disp: &mut disp,
                scratch: &mut sc,
            },
            Transporte {
                step: [1.0, 0.0],
                tecto_em_raios: TECTO_MEDIDO_E_RECUSADO_EM_RAIOS,
                arco: None,
            },
            None,
            w,
            h,
            &s,
            &dab_at([x, 48.0], 10.0),
        );
    }
    // Cut across the trail at mid-drag and count what moved by a visible amount.
    let moved = (0..h)
        .filter(|&y| disp[(y * w + 44) as usize][0] > 0.5)
        .count();
    assert!(
        moved >= 14,
        "the knife is 20 px across but only {moved} px of the cross-section moved — the transport \
         narrowed to a filament"
    );
}

/// A dab that does not move writes nothing at all — and, in particular, does not dirty a rect.
#[test]
fn a_still_dab_transports_nothing() {
    let (w, h) = (32u32, 32u32);
    let mut disp = vec![[0.0f32; 2]; (w * h) as usize];
    assert!(
        accumulate_dab_smear(
            SmearOut {
                disp: &mut disp,
                scratch: &mut SmearScratch::default()
            },
            Transporte {
                step: [0.0, 0.0],
                tecto_em_raios: TECTO_MEDIDO_E_RECUSADO_EM_RAIOS,
                arco: None,
            },
            None,
            w,
            h,
            &spec(6.0),
            &dab_at([16.0, 16.0], 6.0)
        )
        .is_none()
    );
    assert!(disp.iter().all(|d| *d == [0.0, 0.0]));
}

/// The Selection attenuates the transport where it is partial — the knife cannot drag paint out of a
/// region the artist masked off.
#[test]
fn the_selection_attenuates_the_transport() {
    let (w, h) = (48u32, 48u32);
    let n = (w * h) as usize;
    let s = spec(8.0);
    // Left half fully selected, right half not at all.
    let mut mask = vec![0u8; n];
    for y in 0..h {
        for x in 0..(w / 2) {
            mask[(y * w + x) as usize] = 255;
        }
    }
    let mut sc = SmearScratch::default();
    let mut disp = vec![[0.0f32; 2]; n];
    for k in 0..24u32 {
        let _ = accumulate_dab_smear(
            SmearOut {
                disp: &mut disp,
                scratch: &mut sc,
            },
            Transporte {
                step: [1.0, 0.0],
                tecto_em_raios: TECTO_MEDIDO_E_RECUSADO_EM_RAIOS,
                arco: None,
            },
            Some(&mask),
            w,
            h,
            &s,
            &dab_at([12.0 + k as f32, 24.0], 8.0),
        );
    }
    let inside = disp[(24 * w + 12) as usize][0];
    let outside = disp[(24 * w + 40) as usize][0];
    assert!(
        inside > 0.5,
        "inside the selection the knife drags ({inside})"
    );
    assert_eq!(
        outside, 0.0,
        "outside the selection nothing moves (got {outside})"
    );
}

/// ⭐⭐ **As FAIXAS de um pingo dão o campo da rota em SÉRIE — ao bit** (2026-10-01). Um pingo grande
/// (raio `128`, a pegada passa o piso de uma divisão) num traço que CURVA — o ramo do arco — e com uma
/// Selecção de bordo suave; a rota em faixas contra a mesma rota com `ablate::SERIAL`, que é a de
/// sempre.
///
/// ⚠️ CONTROLO: a pegada tem de pedir mais do que uma faixa (senão as duas rotas são a mesma) e o
/// campo tem de ter mexido.
#[test]
fn as_faixas_do_pingo_dao_o_campo_da_serie() {
    let (w, h) = (400u32, 400u32);
    let n = (w * h) as usize;
    let raio = 128.0f32;
    let lado = (2.0 * raio) as usize;
    assert!(
        crate::dab::band_count(lado * lado, lado, crate::dab::PARALLEL_MIN_AREA) > 1,
        "CONTROLO: a pegada não pede faixas — as duas rotas seriam a mesma"
    );
    let s = spec(raio);
    let selecao: Vec<u8> = (0..n).map(|i| ((i * 7) % 256) as u8).collect();
    let corre = |serial: bool| -> Vec<[f32; 2]> {
        crate::ablate::set(if serial { crate::ablate::SERIAL } else { 0 });
        let mut sc = SmearScratch::default();
        let mut disp = vec![[0.0f32; 2]; n];
        let mut ant = [140.0f32, 200.0];
        for k in 1..=12u32 {
            let a = k as f32 * 0.25;
            let c = [140.0 + 60.0 * a.sin(), 200.0 + 60.0 * (1.0 - a.cos())];
            let arco = (k % 2 == 0).then_some(Arco {
                centro: [140.0, 260.0],
                dtheta: 0.25,
            });
            let _ = accumulate_dab_smear(
                SmearOut {
                    disp: &mut disp,
                    scratch: &mut sc,
                },
                Transporte {
                    step: [c[0] - ant[0], c[1] - ant[1]],
                    tecto_em_raios: SEM_TECTO,
                    arco,
                },
                (k % 3 == 0).then_some(selecao.as_slice()),
                w,
                h,
                &s,
                &dab_at(c, raio),
            );
            ant = c;
        }
        crate::ablate::set(0);
        disp
    };
    let (serie, faixas) = (corre(true), corre(false));
    let mexidos = serie.iter().filter(|d| d[0] != 0.0 || d[1] != 0.0).count();
    assert!(
        mexidos > 10_000,
        "CONTROLO: o campo mexeu só {mexidos} texels"
    );
    let iguais = serie
        .iter()
        .zip(&faixas)
        .all(|(a, b)| a[0].to_bits() == b[0].to_bits() && a[1].to_bits() == b[1].to_bits());
    assert!(iguais, "as faixas mudaram o campo contra a rota em série");
}

/// SONDA (relógio, `--release`, à mão): o custo por texel do pingo em SÉRIE, recto contra ARCO.
#[test]
#[ignore = "sonda de relógio"]
fn diag_custo_do_pingo_recto_contra_arco() {
    let (w, h) = (1024u32, 1024u32);
    let n = (w * h) as usize;
    let raio = 128.8f32;
    let s = spec(raio);
    for (arco_on, serie) in [(false, true), (true, true), (false, false)] {
        crate::ablate::set(if serie { crate::ablate::SERIAL } else { 0 });
        let mut sc = SmearScratch::default();
        let mut disp = vec![[0.0f32; 2]; n];
        let t = std::time::Instant::now();
        let mut ant = [512.0f32, 512.0];
        let reps = 200u32;
        for k in 1..=reps {
            let a = k as f32 * 0.02;
            let c = [512.0 + 200.0 * a.sin(), 512.0 + 200.0 * (1.0 - a.cos())];
            let arco = arco_on.then_some(Arco {
                centro: [512.0, 712.0],
                dtheta: 0.02,
            });
            let _ = accumulate_dab_smear(
                SmearOut {
                    disp: &mut disp,
                    scratch: &mut sc,
                },
                Transporte {
                    step: [c[0] - ant[0], c[1] - ant[1]],
                    tecto_em_raios: SEM_TECTO,
                    arco,
                },
                None,
                w,
                h,
                &s,
                &dab_at(c, raio),
            );
            ant = c;
        }
        crate::ablate::set(0);
        let us = t.elapsed().as_secs_f64() * 1e6 / f64::from(reps);
        let texels = std::f64::consts::PI * f64::from(raio) * f64::from(raio);
        println!(
            "arco={arco_on} serie={serie}: {us:.0} µs por pingo · {:.1} ns por texel",
            us * 1e3 / texels
        );
    }
}
