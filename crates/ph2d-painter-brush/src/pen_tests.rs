//! As leis da caneta no pincel (W6, [`crate::pen`]): cada neutro é a lei de antes AO BIT, e cada lei faz o
//! que o doc 47_medicao mediu.

use crate::pen::{DabRotation, DabTilt, Pen, Tilt, pulled_string_step};
use crate::{BrushSpec, Dab, Dynamics, Stroke, StrokePoint};

fn traco(
    spec: BrushSpec,
    dynamics: Dynamics,
    tilt: Option<[f32; 2]>,
    p: impl Fn(f32) -> f32,
) -> Vec<Dab> {
    let mut s = Stroke::new(spec, dynamics, 7);
    let (mut out, mut todos) = (Vec::new(), Vec::new());
    if let Some(t) = tilt {
        s.set_tilt(t);
    }
    s.begin(
        StrokePoint {
            pos: [10.0, 50.0],
            pressure: p(0.0),
        },
        &mut out,
    );
    todos.append(&mut out);
    for i in 1..=100 {
        let f = i as f32 / 100.0;
        s.extend(
            StrokePoint {
                pos: [10.0 + 3.0 * i as f32, 50.0],
                pressure: p(f),
            },
            &mut out,
        );
        todos.append(&mut out);
    }
    s.finish(&mut out);
    todos.append(&mut out);
    todos
}

fn rampa(f: f32) -> f32 {
    0.15 + 0.85 * (f * std::f32::consts::PI).sin()
}

#[test]
fn the_neutral_pen_reads_the_hand_bit_for_bit() {
    let pen = Pen::default();
    for p in [0.0f32, 0.013, 0.15, 0.5, 0.999, 1.0] {
        assert_eq!(pen.read(p).to_bits(), p.to_bits());
        assert_eq!(pen.smooth(0.3, p).to_bits(), p.to_bits());
    }
    let d = Dynamics::default();
    for p in [0.0f32, 0.2, 0.46, 1.0] {
        assert_eq!(d.radius_scale(p).to_bits(), (0.1 + 0.9 * p).to_bits());
    }
}

#[test]
fn the_floor_and_the_ceiling_map_the_hand_onto_the_brush() {
    let pen = Pen {
        floor: 0.1,
        ceiling: 0.6,
        ..Pen::default()
    };
    assert_eq!(pen.read(0.05), 0.0);
    assert_eq!(pen.read(0.6), 1.0);
    assert_eq!(pen.read(0.9), 1.0);
    assert!((pen.read(0.35) - 0.5).abs() < 1e-6);
    // O rato (pressão 1) continua 1 com qualquer piso/tecto.
    assert_eq!(pen.read(1.0), 1.0);
}

#[test]
fn the_size_curve_is_the_opacity_curve_on_the_radius() {
    let lin = Dynamics {
        size_min: 0.2,
        ..Default::default()
    };
    let sq = Dynamics {
        size_curve: 2.0,
        strength_pressure: true,
        strength_min: 0.2,
        strength_curve: 2.0,
        ..lin
    };
    for p in [0.0f32, 0.15, 0.46, 1.0] {
        assert!((sq.radius_scale(p) - (0.2 + 0.8 * p * p)).abs() < 1e-6);
        assert_eq!(
            sq.radius_scale(p).to_bits(),
            sq.coverage_scale(p).to_bits(),
            "a MESMA lei"
        );
        assert_eq!(lin.radius_scale(p).to_bits(), (0.2 + 0.8 * p).to_bits());
    }
}

/// ⭐ **A curva em S** (J3, `docs/Painter/Pinceis/exames/canetas.md` 2.ª onda): `0` é a recta do piso/tecto ao
/// bit; `1` é o smoothstep entre eles — sem quina no pé nem no ombro (a inclinação vai a 0 nas duas pontas, o
/// que a recta não faz), monótona, e com o raio do Studio Pen (`pressureSize 0,93`, piso 0,14, tecto 0,86) é a
/// rampa do G-pen do Krita medida na foto da bateria (`d__Ink-3_Gpen_gestos.png`, a largura ÷ a cheia) a ± 0,04.
#[test]
fn the_s_curve_softens_the_floor_and_the_ceiling_into_the_g_pen_ramp() {
    let recta = Pen {
        floor: 0.14,
        ceiling: 0.86,
        ..Pen::default()
    };
    for i in 0..=100 {
        let p = i as f32 / 100.0;
        assert_eq!(
            Pen {
                s_curve: 0.0,
                ..recta
            }
            .read(p)
            .to_bits(),
            recta.read(p).to_bits()
        );
    }
    let s = Pen {
        s_curve: 1.0,
        ..recta
    };
    let (e, lo, hi) = (1e-3f32, 0.14f32, 0.86f32);
    assert!(
        (s.read(lo + e) - s.read(lo)) / e < 0.05,
        "o pé ainda tem quina"
    );
    assert!(
        (s.read(hi) - s.read(hi - e)) / e < 0.05,
        "o ombro ainda tem quina"
    );
    assert!(
        (recta.read(hi) - recta.read(hi - e)) / e > 1.0,
        "a recta tem a quina que o S tira"
    );
    let mut antes = 0.0f32;
    for i in 0..=100 {
        let v = s.read(i as f32 / 100.0);
        assert!(
            v >= antes && (0.0..=1.0).contains(&v),
            "não monótona a {i} %"
        );
        antes = v;
    }
    let d = Dynamics {
        size_min: 0.07,
        pen: s,
        ..Default::default()
    };
    const GPEN: [(f32, f32); 7] = [
        (0.2, 0.126),
        (0.3, 0.202),
        (0.4, 0.353),
        (0.5, 0.555),
        (0.6, 0.756),
        (0.7, 0.908),
        (0.8, 0.958),
    ];
    for (p, w) in GPEN {
        let v = d.radius_scale(s.read(p));
        assert!((v - w).abs() <= 0.04, "a {p}: {v:.3} contra o G-pen {w}");
    }
}

#[test]
fn the_smoothing_follows_the_hand_with_a_lag_of_s_over_one_minus_s_samples() {
    let pen = Pen {
        smoothing: 0.5,
        ..Pen::default()
    };
    let mut lida = 0.0;
    for _ in 0..30 {
        lida = pen.smooth(lida, 1.0);
    }
    assert!((lida - 1.0).abs() < 1e-6, "converge");
    assert!((pen.smooth(0.2, 0.6) - 0.4).abs() < 1e-6);
}

#[test]
fn the_string_only_pulls_when_the_hand_is_farther_than_it() {
    assert_eq!(pulled_string_step([0.0, 0.0], [3.0, 4.0], 5.0), [0.0, 0.0]);
    let p = pulled_string_step([0.0, 0.0], [6.0, 8.0], 5.0);
    assert!((p[0] - 3.0).abs() < 1e-6 && (p[1] - 4.0).abs() < 1e-6);
}

/// Com a corda de 30 px, cada dab fica a ≥ 30 px − 1 passo atrás da mão, e o pen-up não corre atrás dela.
#[test]
fn a_pulled_string_stroke_ends_where_the_brush_is() {
    let spec = BrushSpec {
        stabilizer: 0.0,
        ..BrushSpec::default()
    };
    let corda = Dynamics {
        pen: Pen {
            pulled_string_px: 30.0,
            ..Pen::default()
        },
        ..Default::default()
    };
    let livre = traco(spec, Dynamics::default(), None, |_| 1.0);
    let puxado = traco(spec, corda, None, |_| 1.0);
    let fim = |d: &[Dab]| d.iter().map(|d| d.center[0]).fold(f32::MIN, f32::max);
    assert!(
        (fim(&livre) - 310.0).abs() < 2.0,
        "sem corda o traço chega à mão: {}",
        fim(&livre)
    );
    assert!(
        (fim(&puxado) - 280.0).abs() < 2.0,
        "com corda acaba 30 px atrás: {}",
        fim(&puxado)
    );
}

#[test]
fn the_tilt_composes_both_axes_like_the_real_pen() {
    let (a, az) = Tilt::polar([55.0 / 90.0, 0.0]);
    assert!((a - 55.0).abs() < 1e-3, "{a}");
    assert!((az[0] - 1.0).abs() < 1e-6 && az[1].abs() < 1e-6);
    let (a, az) = Tilt::polar([0.0, -0.5]);
    assert!((a - 45.0).abs() < 0.1 && (az[1] + 1.0).abs() < 1e-6);
    assert_eq!(Tilt::polar([0.0, 0.0]), (0.0, [1.0, 0.0]));
}

#[test]
fn below_the_start_the_pen_tilt_changes_nothing() {
    let t = Tilt {
        start_deg: 37.5,
        full_deg: 45.0,
        width: 1.2,
        length: 3.3,
        ..Tilt::default()
    };
    assert_eq!(t.amount(30.0), 0.0);
    assert_eq!(t.amount(50.0), 1.0);
    assert!((t.amount(41.25) - 0.5).abs() < 1e-6);
}

/// A ponta deitada do lápis (doc 47 §3): ao longo do azimute 3,3 raios, através 1,2 — o eixo maior, o
/// referencial e o menor saem da decomposição; um pincel redondo sem caneta fica redondo.
#[test]
fn the_lying_tip_is_an_ellipse_along_the_azimuth() {
    let along = DabTilt {
        azimuth: [1.0, 0.0],
        length: 3.3,
        width: 1.2,
        smooth: 0.0,
        ..DabTilt::NEUTRA
    };
    let (s1, f, m) = along.compose([1.0, 0.0], 1.0);
    assert!((s1 - 3.3).abs() < 1e-5 && f[0].abs() > 0.9999 && (m - 1.2 / 3.3).abs() < 1e-5);
    let across = DabTilt {
        azimuth: [0.0, 1.0],
        ..along
    };
    let (s1, f, m) = across.compose([1.0, 0.0], 1.0);
    assert!((s1 - 3.3).abs() < 1e-5 && f[1].abs() > 0.9999 && (m - 1.2 / 3.3).abs() < 1e-5);
    // Um pincel CHATO (menor 0,4) deitado através: o eixo maior é o da caneta, o menor o produto.
    let (s1, _, m) = across.compose([1.0, 0.0], 0.4);
    assert!(
        (s1 - 3.3 * 0.4).abs() < 1e-5 || (s1 - 1.2).abs() < 1e-5,
        "{s1}"
    );
    assert!(m <= 1.0);
}

#[test]
fn the_side_of_the_tip_smooths_the_grain_only_when_dragged_lengthwise() {
    let t = Tilt {
        start_deg: 30.0,
        full_deg: 40.0,
        smooth: 0.4,
        ..Tilt::default()
    };
    let ao_longo = t.effect([55.0 / 90.0, 0.0], [1.0, 0.0]).unwrap();
    let de_lado = t.effect([0.0, 55.0 / 90.0], [1.0, 0.0]).unwrap();
    assert!((ao_longo.smooth - 0.4).abs() < 1e-6);
    assert!(de_lado.smooth.abs() < 1e-6);
    assert!(t.effect([0.0, 0.0], [1.0, 0.0]).is_none(), "de pé: nenhum");
}

/// ⭐ O neutro: a inclinação chega e nenhuma lei a lê — os dabs são os de sem caneta, AO BIT; e o rato
/// (`[0, 0]`) com a lei LIGADA também.
#[test]
fn a_tilt_nobody_reads_paints_the_stroke_of_before() {
    let spec = BrushSpec::default();
    let antes = traco(spec, Dynamics::default(), None, rampa);
    assert_eq!(
        antes,
        traco(spec, Dynamics::default(), Some([0.6, -0.3]), rampa)
    );
    let ligada = BrushSpec {
        pen_tilt: Tilt {
            width: 1.2,
            length: 3.3,
            smooth: 0.4,
            rotation: DabRotation::PenTilt,
            lag_px: 170.0,
            ..Tilt::default()
        },
        ..spec
    };
    assert_eq!(
        antes,
        traco(ligada, Dynamics::default(), Some([0.0, 0.0]), rampa)
    );
}

/// O lápis deitado AO LONGO do traço: o raio cresce até ao eixo maior, e a pegada é a elipse ao longo dele.
#[test]
fn a_lying_pen_stretches_the_dab() {
    let spec = BrushSpec {
        pen_tilt: Tilt {
            start_deg: 37.5,
            full_deg: 45.0,
            width: 1.2,
            length: 3.3,
            ..Tilt::default()
        },
        ..BrushSpec::default()
    };
    let de_pe = traco(spec, Dynamics::default(), Some([30.0 / 90.0, 0.0]), |_| 1.0);
    let deitado = traco(spec, Dynamics::default(), Some([55.0 / 90.0, 0.0]), |_| 1.0);
    let r = spec.clamped_radius();
    assert!(
        de_pe.iter().all(|d| (d.radius_px - r).abs() < 1e-4),
        "abaixo do início nada muda"
    );
    let d = deitado.last().unwrap();
    assert!(
        (d.radius_px - 3.3 * r).abs() < 1e-3,
        "{} vs {}",
        d.radius_px,
        3.3 * r
    );
    let fp = spec.dab_footprint_of(d);
    assert!((fp.minor_fraction() - 1.2 / 3.3).abs() < 1e-4);
}

/// Os 105 toques dos 180 s do dono (17/10, `docs/Painter/ferramentas/caneta/medicoes_2026-10-17/`): a pressão de
/// cada amostra, de 5 em 5 ms.
fn toques_do_dono() -> Vec<Vec<f32>> {
    include_str!(
        "../../../docs/Painter/ferramentas/caneta/medicoes_2026-10-17/pressao_por_toque.txt"
    )
    .lines()
    .filter(|l| !l.starts_with('#') && !l.is_empty())
    .map(|l| {
        l.split(' ')
            .map(|s| s.split_once(':').unwrap().1.parse().unwrap())
            .collect()
    })
    .collect()
}

/// ⭐ **A suavização no ORÁCULO dos 180 s** (W6): a sequência REAL de pressão entra, e a lida sai sem degraus
/// (nenhum salto maior que os da mão, menos saltos de um nível do libinput) e com o atraso da LEI — o centróide
/// de cada toque atrasa no máximo `s/(1 − s)` amostras (truncado no pen-up, nunca mais). No tecto
/// [`crate::pen::MAX_SMOOTHING`] o atraso é ≤ 45 ms, o tempo mediano do dono até ao pico.
#[test]
fn the_smoothing_on_the_owners_strokes_adds_no_step_and_no_lag_beyond_the_law() {
    let toques = toques_do_dono();
    assert_eq!(toques.len(), 105, "os toques do oráculo");
    for s in [0.3f32, 0.6, crate::pen::MAX_SMOOTHING] {
        let pen = Pen {
            smoothing: s,
            ..Pen::default()
        };
        let (mut saltos_mao, mut saltos_lida, mut n) = (0usize, 0usize, 0usize);
        let mut atraso_max = 0.0f32;
        for t in &toques {
            let mut lida = Vec::with_capacity(t.len());
            for (k, &p) in t.iter().enumerate() {
                let p = pen.read(p);
                lida.push(if k == 0 {
                    p
                } else {
                    pen.smooth(lida[k - 1], p)
                });
            }
            let maior = |v: &[f32]| {
                v.windows(2)
                    .map(|w| (w[1] - w[0]).abs())
                    .fold(0.0f32, f32::max)
            };
            assert!(
                maior(&lida) <= maior(t) + 1e-6,
                "um degrau que a mão não deu (s {s})"
            );
            saltos_mao += t
                .windows(2)
                .filter(|w| (w[1] - w[0]).abs() >= 0.0099)
                .count();
            saltos_lida += lida
                .windows(2)
                .filter(|w| (w[1] - w[0]).abs() >= 0.0099)
                .count();
            n += t.len();
            let centro = |v: &[f32]| {
                let m: f32 = v.iter().sum();
                v.iter().enumerate().map(|(i, p)| i as f32 * p).sum::<f32>() / m.max(1e-6)
            };
            atraso_max = atraso_max.max(centro(&lida) - centro(t));
        }
        assert!(
            saltos_lida < saltos_mao,
            "s {s}: {saltos_lida} saltos contra {saltos_mao} da mão ({n} amostras)"
        );
        let lei = s / (1.0 - s);
        assert!(
            atraso_max <= lei + 1e-3,
            "s {s}: atraso {atraso_max:.2} amostras > a lei {lei:.2}"
        );
    }
    let lei = crate::pen::MAX_SMOOTHING / (1.0 - crate::pen::MAX_SMOOTHING);
    assert!(
        lei * 5.0 <= 45.0 + 1e-3,
        "no tecto a lida atrasa {} ms",
        lei * 5.0
    );
}

/// O pincel de um dab deitado a alisar tem o grão mais raso (`grain_depth × (1 − smooth)`); sem caneta é o de
/// sempre, ao bit.
#[test]
fn the_brush_of_a_smoothing_dab_has_a_shallower_grain() {
    let spec = BrushSpec {
        grain_depth: 0.8,
        ..BrushSpec::default()
    };
    let mut d = traco(spec, Dynamics::default(), None, |_| 1.0)[3];
    assert_eq!(
        spec.at_dab(&d),
        BrushSpec {
            radius_px: d.radius_px,
            ..spec
        }
    );
    d.tilt = Some(DabTilt {
        azimuth: [1.0, 0.0],
        length: 1.0,
        width: 1.0,
        smooth: 0.5,
        ..DabTilt::NEUTRA
    });
    assert!((spec.at_dab(&d).grain_depth - 0.4).abs() < 1e-6);
}

/// O PenTilt roda o carimbo para o azimute da caneta (sem esticar nada); `None` não roda.
#[test]
fn pen_tilt_rotation_turns_the_dab_to_the_azimuth() {
    let roda = |rotation| BrushSpec {
        pen_tilt: Tilt {
            rotation,
            start_deg: 0.0,
            full_deg: 45.0,
            ..Tilt::default()
        },
        ..BrushSpec::default()
    };
    let d = *traco(
        roda(DabRotation::PenTilt),
        Dynamics::default(),
        Some([0.0, 0.5]),
        |_| 1.0,
    )
    .last()
    .unwrap();
    assert!(
        d.rotation[0].abs() < 1e-5 && (d.rotation[1] - 1.0).abs() < 1e-5,
        "{:?}",
        d.rotation
    );
    let d = *traco(
        roda(DabRotation::None),
        Dynamics::default(),
        Some([0.0, 0.5]),
        |_| 1.0,
    )
    .last()
    .unwrap();
    assert_eq!(d.rotation, [1.0, 0.0]);
}

/// O ATRASO da inclinação (o Rebelle: ~170 px): de pé e depois deitado a meio do traço, a ponta cresce aos poucos
/// pelo caminho; sem atraso, de uma vez.
#[test]
fn the_tilt_lag_grows_the_lying_tip_along_the_path() {
    let deitado = |lag_px| {
        let spec = BrushSpec {
            pen_tilt: Tilt {
                start_deg: 0.0,
                full_deg: 45.0,
                length: 3.0,
                lag_px,
                ..Tilt::default()
            },
            ..BrushSpec::default()
        };
        let mut s = Stroke::new(spec, Dynamics::default(), 7);
        let mut out = Vec::new();
        s.begin(
            StrokePoint {
                pos: [10.0, 50.0],
                pressure: 1.0,
            },
            &mut out,
        );
        let mut raios = Vec::new();
        for i in 1..=60 {
            s.set_tilt(if i <= 10 { [0.0, 0.0] } else { [0.5, 0.0] });
            s.extend(
                StrokePoint {
                    pos: [10.0 + 3.0 * i as f32, 50.0],
                    pressure: 1.0,
                },
                &mut out,
            );
            raios.extend(out.iter().map(|d| d.radius_px));
        }
        (spec.clamped_radius(), raios)
    };
    let (r, sem) = deitado(0.0);
    let (_, com) = deitado(170.0);
    assert!((sem.last().unwrap() - 3.0 * r).abs() < 1e-3);
    let meio = com[com.len() / 2];
    assert!(
        meio > 1.05 * r && meio < 2.5 * r,
        "a meio do atraso: {meio} (r {r})"
    );
    assert!(
        com.windows(2).all(|w| w[1] >= w[0] - 1e-4),
        "cresce sem voltar atrás"
    );
}
