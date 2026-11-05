//! **Sonda das manchas no miolo do Wet Liner** (Bug #62, fotos do dono 04/11): curva de ~65 px, sobre o «Textured Paper» ou a tela
//! lisa, com e sem Same as Paper, nas cores dele. Grava `PH2D_SONDA_OUT/<cena>_<quando>.ppm`; a régua é
//! `docs/Painter/handoffs/material_2026-11-05/manchas.py`.
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_wet_paint::tuning::Knob;

const W: u32 = 520;
const H: u32 = 360;

/// A curva em S do traço, `u` em 0..=1 (a régua refaz a mesma).
fn curva(u: f32) -> [f32; 2] {
    let (p0, p1, p2, p3) = ([70.0, 250.0], [190.0, 20.0], [330.0, 340.0], [450.0, 110.0]);
    let v = 1.0 - u;
    let f = |i: usize| {
        v * v * v * p0[i] + 3.0 * v * v * u * p1[i] + 3.0 * v * u * u * p2[i] + u * u * u * p3[i]
    };
    [f(0), f(1)]
}

fn ponto(pos: [f32; 2], pressure: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn grava(t: &PainterTool, arq: &str) {
    let mut b = format!("P6 {W} {H} 255\n").into_bytes();
    for p in t.canvas_rgba.chunks(4) {
        let a = u32::from(p[3]);
        for &c in &p[..3] {
            b.push(((u32::from(c) * a + 255 * (255 - a)) / 255) as u8);
        }
    }
    std::fs::write(arq, b).expect("gravar a foto");
}

/// O gesto da mão: pressão, velocidade e passadas. `cheia` = pressão 1, passo constante.
fn gesticula(t: &mut PainterTool, gesto: &str, n: usize) {
    let dt = 1000.0 / 60.0;
    let mut rng = 0x2545_f491_4f6c_dd1du64;
    let mut ruido = move || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        (rng >> 40) as f32 / (1u64 << 24) as f32 - 0.5
    };
    let apertada = |u: f32| {
        let (p0, p1, p2, p3) = (
            [170.0, 260.0],
            [240.0, -60.0],
            [290.0, 420.0],
            [350.0, 100.0],
        );
        let v = 1.0 - u;
        let f = |i: usize| {
            v * v * v * p0[i]
                + 3.0 * v * v * u * p1[i]
                + 3.0 * v * u * u * p2[i]
                + u * u * u * p3[i]
        };
        [f(0), f(1)]
    };
    let passada =
        |t: &mut PainterTool, ruido: &mut dyn FnMut() -> f32, de_lado: f32, volta: bool| {
            let c = |u: f32| {
                let u = if volta { 1.0 - u } else { u };
                let q = if gesto == "apertada" {
                    apertada(u)
                } else {
                    curva(u)
                };
                [q[0], q[1] + de_lado]
            };
            let (mut p, mut j, mut u) = (0.9f32, [0.0f32; 2], 0.0f32);
            t.on_canvas_pointer(ponto(c(0.0), 1.0, PointerPhase::Down));
            let mut pausas = [0.3f32, 0.5, 0.7].into_iter().peekable();
            while u < 1.0 {
                let mut du = 1.0 / n as f32;
                let mut pr = 1.0;
                let mut q = c(u.min(1.0));
                if gesto == "mao" {
                    // Baixa-passa: a mão treme devagar; desacelera a meio.
                    p = 0.9 * p + 0.1 * (0.8 + 0.6 * ruido());
                    j = [0.85 * j[0] + 0.5 * ruido(), 0.85 * j[1] + 0.5 * ruido()];
                    du *= 0.35 + 1.6 * (u * std::f32::consts::TAU * 2.0).sin().abs();
                    pr = p.clamp(0.2, 1.0);
                    q = [q[0] + j[0], q[1] + j[1]];
                }
                t.on_canvas_pointer(ponto(q, pr, PointerPhase::Move));
                if gesto == "real" {
                    std::thread::sleep(std::time::Duration::from_millis(8));
                }
                t.on_tick(dt);
                u += du;
                if gesto == "pausas" && pausas.peek().is_some_and(|x| u >= *x) {
                    pausas.next();
                    for _ in 0..40 {
                        t.on_canvas_pointer(ponto(q, pr, PointerPhase::Move));
                        t.on_tick(dt);
                    }
                }
            }
            t.on_canvas_pointer(ponto(c(1.0), 1.0, PointerPhase::Up));
        };
    passada(t, &mut ruido, 0.0, false);
    if gesto == "dois" {
        passada(t, &mut ruido, 25.0, false);
    }
    if gesto == "vaivem" {
        passada(t, &mut ruido, 0.0, true);
        passada(t, &mut ruido, 0.0, false);
    }
}

type Variante<'a> = (&'a str, Option<(Knob, f64)>);

fn pinta(
    dir: &str,
    (papel, igual_ao_papel, nome_cor, cor, gesto): (bool, bool, &str, [u8; 3], &str),
    (v, knob): Variante<'_>,
    pincel: &str,
) {
    let e = crate::tool::paint::wetpaint_preset::de_fabrica_ou_candidato(pincel);
    let mut t = PainterTool::default();
    if papel {
        let px = PainterTool::pixeis_do_papel_texturizado(W, H);
        t.agenda_papel_texturizado(1);
        t.abre_a_sprite_nova(1, px, W, H);
    } else {
        t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    }
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_semente_dos_tracos(0);
    assert!(t.wet_preset_aplicar(&e).is_empty());
    t.wet_preset_aplicar_volumes(&e.parametros.volumes);
    t.set_brush_size_px(33.0);
    t.set_brush_color_srgb8(cor);
    if igual_ao_papel {
        t.toggle_brush_grain_same_as_paper();
    }
    if let Some((k, x)) = knob {
        t.paint.wetpaint.knobs.set(k, x);
    }
    let n: usize = std::env::var("PH2D_SONDA_N")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(140);
    if gesto == "real" {
        t.set_wet_relogio_fixo(false);
    }
    gesticula(&mut t, gesto, n);
    let cena = format!(
        "{}_{}_{}_{}_{gesto}{n}_{v}",
        pincel.replace(' ', "_"),
        if papel { "papel" } else { "liso" },
        if igual_ao_papel { "same" } else { "sem" },
        nome_cor,
    );
    for _ in 0..240 {
        t.on_tick(1000.0 / 60.0);
    }
    grava(&t, &format!("{dir}/{cena}_molhado.ppm"));
    t.wetpaint_dry_canvas();
    for _ in 0..4 {
        t.on_tick(1000.0 / 60.0);
    }
    grava(&t, &format!("{dir}/{cena}_seco.ppm"));
    let (px, _, _) = (&mut t as &mut dyn RasterEditTool).run_full();
    let mut b = format!("P6 {W} {H} 255\n").into_bytes();
    for p in px.chunks(4) {
        b.extend_from_slice(&p[..3]);
    }
    std::fs::write(format!("{dir}/{cena}_produto.ppm"), b).expect("gravar a foto");
}

#[test]
#[ignore = "sonda: grava fotos em PH2D_SONDA_OUT"]
fn sonda_manchas_liner() {
    let dir = std::env::var("PH2D_SONDA_OUT").expect("PH2D_SONDA_OUT");
    let pinceis = std::env::var("PH2D_SONDA_PINCEIS").unwrap_or_else(|_| "Wet Liner".into());
    let variantes: Vec<Variante<'_>> = if std::env::var("PH2D_SONDA_BORDA").is_ok() {
        vec![
            ("fabrica", None),
            ("portao10", Some((Knob::PaperGate, 0.1))),
            ("portao25", Some((Knob::PaperGate, 0.25))),
            ("portao40", Some((Knob::PaperGate, 0.4))),
            ("portao60", Some((Knob::PaperGate, 0.6))),
            ("portao100", Some((Knob::PaperGate, 1.0))),
        ]
    } else {
        vec![
            ("fabrica", None),
            ("edge200", Some((Knob::EdgeDarkening, 200.0))), // o Wet Liner da foto do dono (antes da receita de 04/11)
        ]
    };
    let gestos = std::env::var("PH2D_SONDA_GESTOS").unwrap_or_else(|_| "cheia".into());
    for pincel in pinceis.split(',') {
        for gesto in gestos.split(',') {
            for igual in if std::env::var("PH2D_SONDA_BORDA").is_ok() {
                vec![false]
            } else {
                vec![false, true]
            } {
                for (nc, c) in [("vermelho", [190, 30, 30]), ("preto", [15, 15, 20])] {
                    for v in &variantes {
                        pinta(&dir, (true, igual, nc, c, gesto), *v, pincel);
                    }
                }
            }
        }
    }
}
