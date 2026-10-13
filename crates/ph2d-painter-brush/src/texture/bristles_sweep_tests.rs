//! **As cerdas que ARRASTAM** (doc 31 §3, prompt 32): o Digital/Impasto varre cada pelo pelo passo do
//! dab (`Dab::step`), e o traço vira riscas em vez de filas de pontos. O Wet Paint (`bristles_at`) não varre.

use super::*;
use crate::footprint::FootprintDeform;
use crate::texture::{ShapeFrame, TexDabBasis, TextureKind, TextureMapping, shape_basis};
use crate::{BrushSpec, Dynamics, ShapeInput, Stroke, StrokePoint, stamp_dab_textured};

fn cerdas() -> TextureSettings {
    let mut p = [0.5; MAX_TEX_PARAMS];
    p[..4].copy_from_slice(&BristleKnobs::default().to_unit());
    TextureSettings {
        kind: TextureKind::Bristles,
        params: p,
        ..TextureSettings::default()
    }
}

fn base(s: &TextureSettings, step: [f32; 2]) -> TexDabBasis {
    shape_basis(
        s,
        &mut 0u64,
        [256.0, 256.0],
        FootprintDeform::identity(),
        ShapeFrame::Stroke {
            arc_len: 30.0,
            unit_px: 20.0,
            step,
        },
    )
}

/// ⭐ **A lei, ao bit:** com um passo, o Digital lê a MÉDIA de `tile(round(q) + off_k)` sobre os `máx|D|`
/// texels do varrimento centrado (a referência aqui é escrita à parte); sem passo, Tiled e Flow, a leitura de sempre; o
/// `bristles_at` (o Wet Paint) varre pelo máximo.
#[test]
fn o_arrasto_e_a_media_ao_longo_do_passo_centrado() {
    let s = cerdas();
    let tile = build_bristle_texture(BristleKnobs::default());
    let c = [100.25f32, 80.75];
    let (com, sem) = (base(&s, [9.0, 4.0]), base(&s, [0.0, 0.0]));
    let mut mudou = 0;
    for py in 60..100i64 {
        for px in 80..120i64 {
            let (dx, dy) = (px as f64 + 0.5 - 100.25, py as f64 + 0.5 - 80.75);
            let (x, y) = (bristle_round(dx), bristle_round(dy));
            let (mut soma, mut maximo) = (0.0f32, f32::NEG_INFINITY);
            for k in 0..9 {
                let o = [k - 4, ((k * 4) as f64 / 9.0).round() as i32 - 2];
                soma += tile[bristle_index(x + o[0], y + o[1])];
                maximo = maximo.max(tile[bristle_index(x + o[0], y + o[1])]);
            }
            let m = soma / 9.0;
            let v = sample_dab(&s, &com, px, py, c, 20.0);
            assert_eq!(v.to_bits(), (m.clamp(0.0, 1.0)).to_bits(), "({px}, {py})");
            let liso = sample_dab(&s, &sem, px, py, c, 20.0);
            assert_eq!(
                liso.to_bits(),
                (sample_bristle(&tile, dx, dy).clamp(0.0, 1.0) as f32).to_bits()
            );
            mudou += usize::from(v != liso);
            // O Wet Paint varre pelo MÁXIMO (o portão de papel do depósito dele apagaria a média), sem o
            // clamp do carimbo; sem passo lê o de antes.
            let d = [dx, dy];
            let p = [px as f64 + 0.5, py as f64 + 0.5];
            assert_eq!(
                bristles_at(&s, &com, d, p, 1.0, 20.0).to_bits(),
                f64::from(maximo).to_bits()
            );
            assert_eq!(
                bristles_at(&s, &sem, d, p, 1.0, 20.0).to_bits(),
                sample_bristle(&tile, dx, dy).to_bits()
            );
        }
    }
    assert!(mudou > 200, "o controlo não mede: {mudou}");
    for ancorado in [
        TextureSettings {
            mapping: TextureMapping::Tiled,
            ..s
        },
        TextureSettings { flow: true, ..s },
    ] {
        for (px, py) in [(90i64, 70i64), (101, 81), (115, 95)] {
            assert_eq!(
                sample_dab(&ancorado, &base(&ancorado, [9.0, 4.0]), px, py, c, 20.0).to_bits(),
                sample_dab(&ancorado, &base(&ancorado, [0.0, 0.0]), px, py, c, 20.0).to_bits(),
                "{:?} flow {}",
                ancorado.mapping,
                ancorado.flow
            );
        }
    }
}

const W: u32 = 300;
const H: u32 = 96;

/// Um traço recto de 220 px pelo motor de verdade (Stroke → dabs → carimbo por píxel), raio 20, com o
/// passo de cada dab (`varre`) ou com ele zerado (o carimbo de antes desta janela).
fn traco(spacing: f32, varre: bool, curva: bool) -> Vec<u8> {
    let spec = BrushSpec {
        radius_px: 20.0,
        spacing,
        color: [0.0, 0.0, 0.0],
        stabilizer: 0.0,
        shape: cerdas(),
        continuous_spacing: true,
        ..BrushSpec::default()
    };
    let dynamics = Dynamics {
        size_pressure: false,
        strength_pressure: false,
        ..Dynamics::default()
    };
    let mut stroke = Stroke::new(spec, dynamics, 1);
    let ponto = |k: usize| {
        let x = 40.0 + 5.0 * k as f32;
        let y = if curva {
            48.0 + 22.0 * (x / 40.0).sin()
        } else {
            48.0
        };
        StrokePoint {
            pos: [x, y],
            pressure: 1.0,
        }
    };
    let (mut dabs, mut out) = (Vec::new(), Vec::new());
    stroke.begin(ponto(0), &mut out);
    dabs.extend_from_slice(&out);
    for k in 1..=44 {
        stroke.extend(ponto(k), &mut out);
        dabs.extend_from_slice(&out);
    }
    let mut buf = vec![255u8; (W * H * 4) as usize];
    for d in &dabs {
        let fp = spec.dab_footprint(spec.dab_rotor(d));
        let sb = shape_basis(
            &spec.shape,
            &mut 0u64,
            [W as f32, H as f32],
            fp,
            ShapeFrame::Stroke {
                arc_len: d.arc_len,
                unit_px: d.stroke_radius_px,
                step: if varre { d.step } else { [0.0, 0.0] },
            },
        );
        let _ = stamp_dab_textured(
            &mut buf,
            W,
            H,
            d.center,
            &BrushSpec {
                radius_px: d.radius_px,
                ..spec
            },
            d.coverage,
            false,
            None,
            None,
            Some(ShapeInput {
                basis: &sb,
                image: None,
                ramp_lut: None,
            }),
        );
    }
    buf
}

/// O coeficiente de variação da tinta AO LONGO das linhas do miolo (`|y − 48| ≤ 9`, x 90–230), médio —
/// a régua do doc 31 §1.
fn cv_ao_longo(buf: &[u8]) -> f64 {
    let mut cvs = Vec::new();
    for y in 39..=57usize {
        let linha: Vec<f64> = (90..230usize)
            .map(|x| {
                let p = &buf[(y * W as usize + x) * 4..][..3];
                f64::from(255 - p[0].min(p[1]).min(p[2]))
            })
            .collect();
        let m = linha.iter().sum::<f64>() / linha.len() as f64;
        if m > 1.0 {
            let v = linha.iter().map(|a| (a - m) * (a - m)).sum::<f64>() / linha.len() as f64;
            cvs.push(v.sqrt() / m);
        }
    }
    cvs.iter().sum::<f64>() / cvs.len().max(1) as f64
}

fn tinta(buf: &[u8]) -> u64 {
    buf.as_chunks::<4>()
        .0
        .iter()
        .map(|p| u64::from(255 - p[0].min(p[1]).min(p[2])))
        .sum()
}

/// ⭐ **As cerdas do Digital riscam, não pontilham** (o relato do dono, 13/10): com o arrasto o CV ao
/// longo a Spacing 0,1 e 0,25 cai abaixo de 0,2 e de ¼ do de hoje; CONTROLO, o passo zerado (o carimbo
/// de antes) pontilha a esses Spacings. A tinta é a de hoje (± 3 %), e a 0,025 (passo ≈ 1 px) a MESMA.
#[test]
fn as_cerdas_do_digital_riscam_em_vez_de_pontilhar() {
    for sp in [0.1f32, 0.25] {
        let (com, sem) = (traco(sp, true, false), traco(sp, false, false));
        let (cc, cs) = (cv_ao_longo(&com), cv_ao_longo(&sem));
        let (tc, ts) = (tinta(&com) as f64, tinta(&sem) as f64);
        println!("spacing {sp}: CV com arrasto {cc:.3} · sem {cs:.3} · tinta {tc} contra {ts}");
        assert!(cs > 0.3, "o controlo não pontilha a {sp}: {cs:.3}");
        assert!(
            cc < 0.2 && cc < cs / 4.0,
            "spacing {sp}: CV {cc:.3} (sem arrasto {cs:.3})"
        );
        assert!(
            (tc - ts).abs() / ts < 0.03,
            "spacing {sp}: a tinta mudou ({tc} contra {ts})"
        );
    }
    let (a, b) = (
        tinta(&traco(0.025, true, false)),
        tinta(&traco(0.025, false, false)),
    );
    let rel = (a as f64 - b as f64).abs() / b as f64;
    println!("spacing 0,025: tinta {a} contra {b} ({:.2} %)", rel * 100.0);
    assert_eq!(a, b, "a 0,025 o passo é 1 texel: nada muda");
}

/// **O custo** (MEDIR_VELOCIDADE: as variantes no MESMO processo, intercaladas em blocos rodados, o
/// MÍNIMO): o traço com o arrasto contra o passo zerado, recto e em curva, a cada Spacing.
/// `cargo test -p ph2d-painter-brush --profile smoke --lib measure_custo_do_arrasto -- --ignored --nocapture`
#[test]
#[ignore = "medição: o custo do arrasto das cerdas"]
fn measure_custo_do_arrasto() {
    use std::time::Instant;
    println!("| traço | spacing | sem arrasto (ms) | com arrasto (ms) | razão |");
    println!("|---|---:|---:|---:|---:|");
    for curva in [false, true] {
        for sp in [0.025f32, 0.1, 0.25, 0.5] {
            let mut min = [f64::MAX; 2];
            for bloco in 0..9 {
                for i in 0..2 {
                    let v = (i + bloco) % 2;
                    let t0 = Instant::now();
                    std::hint::black_box(traco(sp, v == 1, curva));
                    min[v] = min[v].min(t0.elapsed().as_secs_f64() * 1e3);
                }
            }
            println!(
                "| {} | {sp} | {:.2} | {:.2} | {:.3} |",
                if curva { "curva" } else { "recto" },
                min[0],
                min[1],
                min[1] / min[0]
            );
        }
    }
}

/// ⭐ **A opção é uma OPÇÃO** (dono 13/10: *«não pode substituir a antiga»*): desligada (a omissão), nenhum
/// dab leva passo — o traço de antes, ao bit. Ligada com as cerdas, todos levam, o de ABERTURA também (o
/// aquecimento dá-lhe um passo nominal na direcção assente; sem isso os pontos dele saltavam à vista).
/// CONTROLO: ligada sem as cerdas, nada a ler o passo, nada a aquecer.
#[test]
fn o_passo_continuo_e_uma_opcao_desligada_por_omissao() {
    let dabs = |continuo: bool, kind: TextureKind| {
        let spec = BrushSpec {
            radius_px: 20.0,
            spacing: 0.25,
            stabilizer: 0.0,
            continuous_spacing: continuo,
            shape: TextureSettings { kind, ..cerdas() },
            ..BrushSpec::default()
        };
        let mut stroke = Stroke::new(spec, Dynamics::default(), 1);
        let (mut todos, mut out) = (Vec::new(), Vec::new());
        let p = |x: f32| StrokePoint {
            pos: [x, 48.0],
            pressure: 1.0,
        };
        stroke.begin(p(40.0), &mut out);
        todos.extend_from_slice(&out);
        for k in 1..=30 {
            stroke.extend(p(40.0 + 5.0 * k as f32), &mut out);
            todos.extend_from_slice(&out);
        }
        todos
    };
    assert!(!BrushSpec::default().continuous_spacing);
    let off = dabs(false, TextureKind::Bristles);
    assert!(off.len() > 10 && off.iter().all(|d| d.step == [0.0, 0.0]));
    let on = dabs(true, TextureKind::Bristles);
    assert!(
        on.iter().all(|d| d.step[0] > 5.0 && d.step[1].abs() < 1e-3),
        "{:?}",
        on.iter().map(|d| d.step).collect::<Vec<_>>()
    );
    let liso = dabs(true, TextureKind::None);
    assert_eq!(
        liso.iter().map(|d| d.center).collect::<Vec<_>>(),
        dabs(false, TextureKind::None)
            .iter()
            .map(|d| d.center)
            .collect::<Vec<_>>()
    );
}
