//! AUDITORIA lente 2 (costura), 15/10 — sonda do smoke do dono P39-3 (Impasto + Spacing 0,25 + Continuous
//! pela PORTA do painel). Não é portão: mede, por variante, o campo e a tela acesa contra o alvo denso.
//! `cargo test -p ph2d-tool-painter --lib auditoria_l2 -- --ignored --nocapture`

use super::capsula::verde_laranja;
use super::*;
use crate::ids;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_brush::TextureKind;

const LW: u32 = 640;
const LH: u32 = 320;

#[derive(Clone, Copy, Debug)]
struct Caso {
    nome: &'static str,
    r: f32,
    /// inclinação do traço (dy total sobre 480 px de dx); 0 = recto horizontal (o portão).
    dy: f32,
    /// curvatura (amplitude de um seno ao longo do traço, px).
    curva: f32,
    /// pressão variável (caneta).
    caneta: bool,
    /// rampa: true = a de fábrica (as duas camadas); false = a verde-laranja do portão.
    fabrica: bool,
    /// ordem: 0 = Spacing→Continuous; 1 = Continuous→Spacing; 2 = Continuous no Digital e depois Impasto;
    /// 3 = Continuous e depois Digital→Impasto; 4 = Continuous, ferramenta knife e volta a brush;
    /// 5 = campo DIRECTO + fill dos slots (a receita do portão).
    ordem: u8,
    kind: TextureKind,
    eventos: usize,
    follow: u8,
    fino: bool,
}

fn ev(t: &mut PainterTool, e: PanelEvent) {
    t.handle_panel_event(e);
}

fn monta(c: Caso, sp: f32, cont: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (LW * LH * 4) as usize], LW, LH);
    let media = |t: &mut PainterTool, v: &str| {
        ev(
            t,
            PanelEvent::SelectOption(ids::PAINTER_BRUSH_MEDIA, v.into()),
        )
    };
    if c.ordem == 2 && cont {
        ev(
            &mut t,
            PanelEvent::Click(ids::PAINTER_BRUSH_CONTINUOUS_SPACING),
        );
    }
    media(&mut t, "2");
    t.set_brush_color_srgb8([20, 30, 120]);
    t.set_wet_relogio_fixo(true);
    t.set_brush_shape_kind(c.kind.to_u8());
    t.set_brush_size_px(c.r * 2.0);
    if !c.fabrica {
        t.set_shape_color_ramp(verde_laranja());
    }
    t.set_shape_ramp_enabled(true);
    if c.fino {
        t.paint.brush.shape.size = [3.0; 2];
    }
    if c.follow > 0 {
        ev(
            &mut t,
            PanelEvent::SelectOption(ids::PAINTER_SHAPE_FOLLOW, c.follow.to_string()),
        );
    }
    match c.ordem {
        5 => {
            t.set_brush_spacing(sp);
            t.paint.brush.continuous_spacing = cont;
            let s = t.paint.brush;
            t.paint.brush_by_mode.fill(s);
        }
        1 => {
            if cont {
                ev(
                    &mut t,
                    PanelEvent::Click(ids::PAINTER_BRUSH_CONTINUOUS_SPACING),
                );
            }
            ev(
                &mut t,
                PanelEvent::SetValue(ids::PAINTER_BRUSH_SPACING, f64::from(sp)),
            );
        }
        _ => {
            ev(
                &mut t,
                PanelEvent::SetValue(ids::PAINTER_BRUSH_SPACING, f64::from(sp)),
            );
            if cont && c.ordem != 2 {
                ev(
                    &mut t,
                    PanelEvent::Click(ids::PAINTER_BRUSH_CONTINUOUS_SPACING),
                );
            }
            if c.ordem == 3 {
                media(&mut t, "0");
                media(&mut t, "2");
            }
            if c.ordem == 4 {
                t.set_paint_tool_mode("knife");
                t.set_paint_tool_mode("brush");
            }
        }
    }
    t
}

fn acesa_wh(t: &PainterTool, cru: &[u8], w: u32, h: u32) -> Vec<u8> {
    let mut b = cru.to_vec();
    t.apply_impasto_light(&mut b, Region { x: 0, y: 0, w, h });
    b
}

fn caminho(c: Caso, s: f32) -> [f32; 2] {
    let x = 80.0 + 480.0 * s;
    let y = LH as f32 / 2.0 - c.dy / 2.0 + c.dy * s + c.curva * (s * std::f32::consts::TAU).sin();
    [x, y]
}

fn risca(t: &mut PainterTool, c: Caso) -> Vec<u8> {
    let n = c.eventos;
    let p = |k: usize| {
        let s = k as f32 / n as f32;
        let pr = if c.caneta {
            0.75 + 0.25 * (s * 9.0).sin()
        } else {
            1.0
        };
        (caminho(c, s), pr)
    };
    let mk = |pos, pressure, phase| CanvasPointer {
        pos,
        pressure,
        tilt: [0.0, 0.0],
        phase,
    };
    let (p0, pr0) = p(0);
    t.on_canvas_pointer(mk(p0, pr0, PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=n {
        let (pk, pr) = p(k);
        t.on_canvas_pointer(mk(pk, pr, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    let (pn, prn) = p(n);
    t.on_canvas_pointer(mk(pn, prn, PointerPhase::Up));
    for _ in 0..30 {
        t.on_tick(1000.0 / 60.0);
    }
    if c.ordem == 6 {
        // o artista mexe no cartão Body DEPOIS do traço (re-derivação viva do último traço).
        let d = t.paint.brush.impasto_depth;
        t.set_brush_impasto_depth(d * 0.8);
        let b = t.paint.brush.impasto_body;
        t.set_brush_impasto_body((b - 0.2).max(0.0));
        for _ in 0..5 {
            t.on_tick(1000.0 / 60.0);
        }
    }
    t.canvas_rgba.to_vec()
}

/// Os píxeis a menos de `0,6 r` do eixo, longe das pontas.
fn miolo(c: Caso) -> Vec<usize> {
    let mut v = Vec::new();
    for k in 0..=400 {
        let s = 0.2 + 0.6 * k as f32 / 400.0;
        let [x, y] = caminho(c, s);
        for dy in -(c.r * 0.6) as i32..=(c.r * 0.6) as i32 {
            let yy = (y as i32 + dy) as usize;
            v.push(yy * LW as usize + x as usize);
        }
    }
    v.sort_unstable();
    v.dedup();
    v
}

fn dif(a: &[u8], b: &[u8], m: &[usize]) -> f32 {
    let s: f32 = m
        .iter()
        .map(|&i| {
            (0..3)
                .map(|k| (f32::from(a[i * 4 + k]) - f32::from(b[i * 4 + k])).abs())
                .sum::<f32>()
                / 3.0
        })
        .sum();
    s / m.len() as f32 / 255.0
}

fn relevo(t: &PainterTool) -> Vec<f32> {
    t.layers
        .active()
        .and_then(|id| t.layer_height_view(id))
        .unwrap_or_default()
}

fn dif_rel(a: &[f32], b: &[f32], m: &[usize]) -> f32 {
    if a.is_empty() || b.is_empty() {
        return f32::NAN;
    }
    let st: f32 = m.iter().map(|&i| b[i].abs()).sum();
    m.iter().map(|&i| (a[i] - b[i]).abs()).sum::<f32>() / st.max(1e-9)
}

macro_rules! linha {
    ($($a:tt)*) => {{
        let l = format!($($a)*);
        println!("{l}");
        if let Ok(f) = std::env::var("PH2D_AUD_OUT") {
            use std::io::Write;
            let mut h = std::fs::OpenOptions::new().create(true).append(true).open(f).expect("out");
            writeln!(h, "{l}").expect("w");
        }
    }};
}

#[test]
#[ignore = "sonda de auditoria (lente 2): Continuous pela porta do painel no Impasto"]
fn auditoria_l2_continuous_pela_porta() {
    let base = Caso {
        nome: "portão (r20 recto, verde-laranja)",
        r: 20.0,
        dy: 0.0,
        curva: 0.0,
        caneta: false,
        fabrica: false,
        ordem: 0,
        kind: TextureKind::Voronoi,
        eventos: 96,
        follow: 0,
        fino: false,
    };
    let casos = [
        Caso {
            ordem: 5,
            nome: "receita do portão (campo directo + fill)",
            ..base
        },
        base,
        Caso {
            ordem: 1,
            nome: "Continuous→Spacing",
            ..base
        },
        Caso {
            ordem: 2,
            nome: "Continuous no Digital → Impasto",
            ..base
        },
        Caso {
            ordem: 3,
            nome: "Continuous, Digital→Impasto",
            ..base
        },
        Caso {
            ordem: 4,
            nome: "Continuous, knife→brush",
            ..base
        },
        Caso {
            fabrica: true,
            nome: "rampa de fábrica",
            ..base
        },
        Caso {
            dy: 40.0,
            nome: "inclinado (dy 40)",
            ..base
        },
        Caso {
            curva: 25.0,
            nome: "curvo (amp 25)",
            ..base
        },
        Caso {
            caneta: true,
            nome: "caneta (pressão 0,5–1)",
            ..base
        },
        Caso {
            r: 50.0,
            nome: "r 50",
            ..base
        },
        Caso {
            r: 50.0,
            dy: 40.0,
            curva: 15.0,
            fabrica: true,
            caneta: true,
            nome: "dono: r50 curvo fábrica caneta",
            ..base
        },
        Caso {
            kind: TextureKind::Checker,
            dy: 40.0,
            nome: "Checker inclinado",
            ..base
        },
        Caso {
            kind: TextureKind::Noise,
            dy: 40.0,
            nome: "Noise inclinado",
            ..base
        },
        Caso {
            ordem: 6,
            nome: "Depth e Body mexidos DEPOIS do traço",
            ..base
        },
        Caso {
            eventos: 12,
            nome: "12 eventos (40 px por evento)",
            ..base
        },
        Caso {
            eventos: 6,
            curva: 15.0,
            nome: "6 eventos curvo",
            ..base
        },
        Caso {
            fino: true,
            nome: "Shape Size 3",
            ..base
        },
        Caso {
            follow: 1,
            curva: 25.0,
            nome: "Follow Rake curvo",
            ..base
        },
        Caso {
            follow: 2,
            curva: 25.0,
            nome: "Follow Flow curvo",
            ..base
        },
        Caso {
            r: 50.0,
            dy: 40.0,
            curva: 15.0,
            fabrica: true,
            eventos: 12,
            nome: "dono sem caneta: r50 curvo fábrica 12 ev",
            ..base
        },
    ];
    linha!(
        "| caso | cont. no campo | spacing | lays_capsules | Δacesa 0,25+C vs denso | Δacesa 0,25 sem C vs denso | Δrelevo 0,25+C | Δ vs receita do portão |"
    );
    linha!("|---|---|---:|---|---:|---:|---:|---:|");
    for c in casos {
        let m = miolo(c);
        let mut ta = monta(c, 0.025, false);
        let ca = risca(&mut ta, c);
        let la = acesa_wh(&ta, &ca, LW, LH);
        let ha = relevo(&ta);
        let mut tt = monta(c, 0.25, true);
        let (campo, sp, caps) = (
            tt.paint.brush.continuous_spacing,
            tt.paint.brush.spacing,
            tt.stroke_spec().lays_capsules(),
        );
        let ct = risca(&mut tt, c);
        let lt = acesa_wh(&tt, &ct, LW, LH);
        let ht = relevo(&tt);
        let mut ts = monta(c, 0.25, false);
        let cs = risca(&mut ts, c);
        let ls = acesa_wh(&ts, &cs, LW, LH);
        let mut tg = monta(Caso { ordem: 5, ..c }, 0.25, true);
        let cg = risca(&mut tg, c);
        let lg = acesa_wh(&tg, &cg, LW, LH);
        linha!(
            "| {} | {campo} | {sp} | {caps} | {:.4} | {:.4} | {:.4} | {:.5} |",
            c.nome,
            dif(&lt, &la, &m),
            dif(&ls, &la, &m),
            dif_rel(&ht, &ha, &m),
            dif(&lt, &lg, &m),
        );
        if let Ok(d) = std::env::var("PH2D_SONDA_OUT") {
            for (s, px) in [("denso", &la), ("cont", &lt), ("sem", &ls)] {
                let mut ppm = format!("P6 {LW} {LH} 255\n").into_bytes();
                for k in 0..(LW * LH) as usize {
                    ppm.extend_from_slice(&px[k * 4..k * 4 + 3]);
                }
                let n = c.nome.replace([' ', '(', ')', ',', '→', '+'], "_");
                std::fs::write(format!("{d}/{n}_{s}.ppm"), ppm).expect("grava");
            }
        }
    }
}
