//! O gate-lei das pontas que ESBATEM ao comprido (J6t, [`super::esbate`]): no recto de 40 px (raio do motor 66,9) o depósito
//! cai para os dois centros das pontas e enche um raio para dentro — na SOMA com as pontas rectas (o Square Hard), no
//! FILME (a aguarela) e na soma redonda — e o miolo pousa a MESMA tinta.

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_wet_paint::tuning::Knob;

const W: usize = 520;
const H: usize = 120;
const Y: f32 = 60.0;
const X0: f32 = 60.0;
const X1: f32 = 460.0;
/// O raio do MOTOR do pincel sem preset a tamanho 40 (`largura.rs`: a largura medida é a prometida).
const R: f32 = 66.9;

#[derive(Clone, Copy, Debug)]
enum Caso {
    Rectas,
    Filme,
    Redondo,
}

fn recto(caso: Caso, e: f32) -> Vec<f64> {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([30, 60, 200]);
    match caso {
        // A ponta DURA e quadrada do pastel (o Square Hard): a pegada chata ao longo, que a carga do fecho supõe.
        Caso::Rectas => {
            t.toggle_brush_pontas_rectas();
            t.set_brush_hardness(1.0);
            t.set_brush_dab_quina(1.0);
        }
        Caso::Filme => t.paint.wetpaint.knobs.set(Knob::StrokeFilm, 1.0),
        Caso::Redondo => {}
    }
    t.set_brush_pontas_esbate(e);
    let p = |x: f32, phase| CanvasPointer {
        pos: [x, Y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(X0, PointerPhase::Down));
    let n = ((X1 - X0) / 3.0) as usize;
    for k in 1..=n {
        t.on_canvas_pointer(p(X0 + 3.0 * k as f32, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(p(X0 + 3.0 * n as f32, PointerPhase::Up));
    t.on_tick(1000.0 / 60.0);
    t.canvas_rgba
        .chunks(4)
        .map(|px| f64::from(255 - px[0].min(px[1]).min(px[2])) / 255.0)
        .collect()
}

/// O perfil das duas pontas: a escuridão do eixo (± 8 px) ÷ a do meio a `0 · ¼ · ½ · 1 · 1½` raios para DENTRO das
/// pontas do traço de controlo `ref` (onde ele passa metade do meio) — `[início…, fim…]` — e a tinta do miolo (as
/// colunas a mais de um raio e meio dessas pontas).
const PASSOS: [f32; 5] = [0.0, 0.25, 0.5, 1.0, 1.5];

fn perfil(c: &[f64], r#ref: &[f64]) -> ([[f64; 5]; 2], f64) {
    let col = |c: &[f64], x: usize| {
        (Y as usize - 8..=Y as usize + 8)
            .map(|y| c[y * W + x])
            .sum::<f64>()
            / 17.0
    };
    let meio_de = |c: &[f64]| (220..300).map(|x| col(c, x)).sum::<f64>() / 80.0;
    let m_ref = meio_de(r#ref);
    let p0 = (0..W).find(|&x| col(r#ref, x) >= 0.5 * m_ref).unwrap() as f32;
    let p1 = (0..W)
        .rev()
        .find(|&x| col(r#ref, x) >= 0.5 * m_ref)
        .unwrap() as f32;
    let meio = meio_de(c);
    let q = [
        PASSOS.map(|k| col(c, (p0 + k * R).round() as usize) / meio),
        PASSOS.map(|k| col(c, (p1 - k * R).round() as usize) / meio),
    ];
    let (xa, xb) = ((p0 + 1.5 * R) as usize, (p1 - 1.5 * R) as usize);
    let tinta = (0..H)
        .flat_map(|y| (xa..xb).map(move |x| (y, x)))
        .map(|(y, x)| c[y * W + x])
        .sum();
    (q, tinta)
}

/// ⭐ O GATE: com a lei as duas pontas esbatem — perto da ponta (até meio raio) o eixo fica claramente abaixo do traço
/// sem ela, e um raio e meio para dentro é o mesmo — nos três depósitos (a soma com as pontas rectas, o filme, a soma
/// redonda), e o miolo pousa a mesma tinta (± 3 %).
#[test]
fn as_pontas_esbatem_ao_comprido_e_o_miolo_pousa_a_mesma_tinta() {
    for caso in [Caso::Rectas, Caso::Filme, Caso::Redondo] {
        let c0 = recto(caso, 0.0);
        let (q0, t0) = perfil(&c0, &c0);
        // A queda mínima perto da ponta: cresce com `e` (a escuridão satura: o depósito a 0,8 lê ~0,93).
        for (e, minima) in [(1.6f32, 0.06), (4.0, 0.14)] {
            let (q, t) = perfil(&recto(caso, e), &c0);
            eprintln!(
                "{caso:?} e {e}: sem {q0:.3?} · com {q:.3?}; tinta {:.4}",
                t / t0
            );
            for (lado, ponta) in [(0, "início"), (1, "fim")] {
                let queda = (0..3)
                    .map(|k| q0[lado][k] - q[lado][k])
                    .fold(f64::MIN, f64::max);
                assert!(
                    queda >= minima,
                    "{caso:?} e {e}: a ponta do {ponta} tem de esbater: queda {queda:.3} ({:.3?} contra {:.3?})",
                    q[lado],
                    q0[lado]
                );
                assert!(
                    q[lado][4] >= q0[lado][4] - 0.03,
                    "{caso:?} e {e}: um raio e meio para dentro do {ponta} é o traço de sempre: {:.3?} contra {:.3?}",
                    q[lado],
                    q0[lado]
                );
            }
            assert!(
                (0.97..=1.03).contains(&(t / t0)),
                "{caso:?} e {e}: o miolo pousa a tinta de sempre: {:.4}",
                t / t0
            );
        }
    }
}

/// As contas da lei: o alvo e a carga do fecho (a soma com a cauda viva da metade de trás dá o alvo).
#[test]
fn as_contas_do_esbate_fecham() {
    use super::esbate::{alvo, carga_do_fecho};
    assert_eq!(alvo(0.0, -0.7), 1.0);
    assert!((alvo(1.6, 0.0) - 0.8).abs() < 1e-6 && alvo(1.6, 1.0) == 1.0 && alvo(2.0, -1.0) == 0.0);
    for e in [0.3f32, 1.0, 1.6, 2.0] {
        // A cauda a `a` raios para dentro do fim: `a` da mão + o fecho de `d ∈ (0, 1 − a]`.
        for a in [0.0f32, 0.3, 0.7] {
            let m = 2000;
            let f: f32 = (0..m)
                .map(|i| {
                    carga_do_fecho(e, (1.0 - a) * (i as f32 + 0.5) / m as f32, 1.0) * (1.0 - a)
                        / m as f32
                })
                .sum();
            assert!(
                (a + f - alvo(e, a)).abs() < 1e-3,
                "e {e} a {a}: {} contra {}",
                a + f,
                alvo(e, a)
            );
        }
    }
}
