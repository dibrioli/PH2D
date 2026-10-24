//! SONDA DESCARTÁVEL (24/10, pergunta do dono): porque a borracha a Strength 1 precisa de várias passadas e
//! alguns pincéis Digital a Strength 1 não pousam opacidade total. Não é gate. Corre-se à mão:
//! `cargo test -p ph2d-tool-painter --lib sonda_opacidade -- --ignored --nocapture`

use crate::tool::PainterTool;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::Falloff;

const W: usize = 256;
const H: usize = 128;
const Y: f32 = 64.0;
const X0: f32 = 30.0;
const X1: f32 = 226.0;
const PX: usize = 128;
const DIAM: f32 = 40.0;

fn cp(x: f32, p: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, Y],
        pressure: p,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// pressão: `None` = rato (1,0); `Some(p)` = caneta constante `p`.
fn passada(t: &mut PainterTool, p: f32) {
    t.on_canvas_pointer(cp(X0, p, PointerPhase::Down));
    let mut x = X0;
    while x < X1 {
        x += 3.0;
        t.on_canvas_pointer(cp(x, p, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp(X1, p, PointerPhase::Up));
}

fn presets() -> Biblioteca {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    Biblioteca::carregar(&raiz, None)
}

/// «quanto o traço fez» em `(PX, Y+dy)`: borracha = alfa removido; pincel = escurecimento sobre branco.
fn feito(t: &PainterTool, dy: i32, apaga: bool) -> u8 {
    let y = (Y as i32 + dy) as usize;
    let i = (y * W + PX) * 4;
    let c = &t.canvas_rgba;
    if apaga { 255 - c[i + 3] } else { 255 - c[i] }
}

struct Caso {
    nome: &'static str,
    preset: Option<&'static str>,
    borracha: bool,
}

fn monta(bib: &Biblioteca, c: &Caso) -> PainterTool {
    let mut t = PainterTool::default();
    let fundo: [u8; 4] = if c.borracha {
        [200, 50, 50, 255]
    } else {
        [255; 4]
    };
    t.set_source(fundo.repeat(W * H), W as u32, H as u32);
    if let Some(n) = c.preset {
        let p = &bib
            .entradas
            .iter()
            .find(|e| e.preset.parametros.name == n)
            .unwrap_or_else(|| panic!("preset {n}"))
            .preset;
        let f = t.preset_aplicar(p).expect("meio");
        if !f.is_empty() {
            println!("  ({n}: falhas {f:?})");
        }
    }
    if c.borracha {
        t.set_paint_tool_mode("eraser");
    }
    t.set_brush_size_px(DIAM);
    t.set_brush_color_srgb8([0, 0, 0]);
    t.set_brush_strength(1.0);
    t
}

type Mexe = (&'static str, fn(&mut PainterTool));

fn fatores() -> Vec<Mexe> {
    vec![
        ("base", |_| {}),
        ("acc ON", |t| t.paint.brush.accumulate = true),
        ("acc OFF", |t| t.paint.brush.accumulate = false),
        ("flow 1", |t| t.paint.brush.flow = 1.0),
        ("duro", |t| {
            t.paint.brush.falloff = Falloff::Constant;
            t.paint.brush.hardness = 1.0;
        }),
        ("hard1", |t| t.paint.brush.hardness = 1.0),
        ("spc .02", |t| t.paint.brush.spacing = 0.02),
        ("p->opac0", |t| t.paint.dynamics.strength_pressure = false),
        ("pen neutra", |t| t.paint.dynamics.pen = Default::default()),
        ("tecto0", |t| {
            t.paint.brush.traco = ph2d_painter_brush::traco::TracoComposto::NEUTRO
        }),
        ("borda0", |t| t.paint.brush.borda_px = 0.0),
        ("smoothE0", |t| t.paint.brush.smooth_edges = false),
        ("jit0", |t| t.paint.brush.dab_jitter = Default::default()),
        ("taper0", |t| t.paint.brush.taper = Default::default()),
        ("grain1", |t| t.paint.brush.grain_depth = 0.0),
    ]
}

#[test]
#[ignore = "sonda à mão"]
fn sonda_opacidade_centro_e_fatores() {
    let bib = presets();
    let casos = [
        Caso {
            nome: "borracha de fabrica (rail)",
            preset: None,
            borracha: true,
        },
        Caso {
            nome: "pincel de fabrica (rail)",
            preset: None,
            borracha: false,
        },
        Caso {
            nome: "Hard Eraser",
            preset: Some("Hard Eraser"),
            borracha: false,
        },
        Caso {
            nome: "Hard Eraser + rail eraser",
            preset: Some("Hard Eraser"),
            borracha: true,
        },
        Caso {
            nome: "Kneaded Eraser",
            preset: Some("Kneaded Eraser"),
            borracha: false,
        },
        Caso {
            nome: "Hard Round + rail eraser",
            preset: Some("Hard Round"),
            borracha: true,
        },
        Caso {
            nome: "Soft Round + rail eraser",
            preset: Some("Soft Round"),
            borracha: true,
        },
        Caso {
            nome: "Hard Round",
            preset: Some("Hard Round"),
            borracha: false,
        },
        Caso {
            nome: "Soft Round",
            preset: Some("Soft Round"),
            borracha: false,
        },
        Caso {
            nome: "Flat Block-in",
            preset: Some("Flat Block-in"),
            borracha: false,
        },
        Caso {
            nome: "Glaze Round",
            preset: Some("Glaze Round"),
            borracha: false,
        },
        Caso {
            nome: "Pixel 1px",
            preset: Some("Pixel 1px"),
            borracha: false,
        },
        Caso {
            nome: "Mapping Pen",
            preset: Some("Mapping Pen"),
            borracha: false,
        },
        Caso {
            nome: "Ballpoint",
            preset: Some("Ballpoint"),
            borracha: false,
        },
        Caso {
            nome: "Marker Liner",
            preset: Some("Marker Liner"),
            borracha: false,
        },
        Caso {
            nome: "Halftone",
            preset: Some("Halftone"),
            borracha: false,
        },
        Caso {
            nome: "Stars",
            preset: Some("Stars"),
            borracha: false,
        },
        Caso {
            nome: "Bokeh",
            preset: Some("Bokeh"),
            borracha: false,
        },
    ];
    for c in &casos {
        let apaga = c.borracha || c.preset.is_some_and(|n| n.contains("Eraser"));
        let t0 = monta(&bib, c);
        let b = t0.paint.brush;
        let d = t0.paint.dynamics;
        println!(
            "\n### {} — media {:?} eraser {} | acc {} str {:.2} flow {:.3} hard {:.2} fall {:?} spc {:.3} borda {:.1} smoothE {} tecto {:.2}/{} p->op {} min {:.2} curve {:.2} pen fl {:.2} ce {:.2} jit.op {:.2} cover_wanted {}",
            c.nome,
            t0.paint_media(),
            t0.paint.eraser,
            b.accumulate,
            b.strength,
            b.flow,
            b.hardness,
            b.falloff,
            b.spacing,
            b.borda_px,
            b.smooth_edges,
            b.traco.tecto,
            b.traco.tecto_por_dab,
            d.strength_pressure,
            d.strength_min,
            d.strength_curve,
            d.pen.floor,
            d.pen.ceiling,
            b.dab_jitter.opacity,
            t0.stroke_cover_wanted(&b),
        );
        // passadas × pressão, centro + perfil
        for &p in &[1.0f32, 0.6] {
            let mut t = monta(&bib, c);
            let mut linha = format!("  p={p:.1} centro por passada:");
            for _ in 0..3 {
                passada(&mut t, p);
                linha.push_str(&format!(" {:3}", feito(&t, 0, apaga)));
            }
            let mut t1 = monta(&bib, c);
            passada(&mut t1, p);
            linha.push_str("  | perfil 1 passada dy=0..20:");
            for dy in (0..=20).step_by(4) {
                linha.push_str(&format!(" {:3}", feito(&t1, dy, apaga)));
            }
            println!("{linha}");
        }
        // fatores, 1 passada, rato
        let mut linha = String::from("  fatores (centro, 1 passada, p=1 | p=0.6):");
        for (n, f) in fatores() {
            let mut a = monta(&bib, c);
            f(&mut a);
            passada(&mut a, 1.0);
            let mut b2 = monta(&bib, c);
            f(&mut b2);
            passada(&mut b2, 0.6);
            linha.push_str(&format!(
                " {n}={}|{}",
                feito(&a, 0, apaga),
                feito(&b2, 0, apaga)
            ));
        }
        println!("{linha}");
    }
}

/// Passadas até apagar/cobrir, por pressão de caneta constante (a mão leve do dono: 0,24–0,86).
#[test]
#[ignore = "sonda à mão"]
fn sonda_opacidade_passadas_por_pressao() {
    let bib = presets();
    let casos = [
        Caso {
            nome: "borracha de fabrica (rail)",
            preset: None,
            borracha: true,
        },
        Caso {
            nome: "Hard Eraser",
            preset: Some("Hard Eraser"),
            borracha: true,
        },
        Caso {
            nome: "Kneaded Eraser",
            preset: Some("Kneaded Eraser"),
            borracha: true,
        },
        Caso {
            nome: "Soft Round + rail eraser",
            preset: Some("Soft Round"),
            borracha: true,
        },
        Caso {
            nome: "Marker Liner + rail eraser",
            preset: Some("Marker Liner"),
            borracha: true,
        },
        Caso {
            nome: "Hard Round",
            preset: Some("Hard Round"),
            borracha: false,
        },
        Caso {
            nome: "Soft Round",
            preset: Some("Soft Round"),
            borracha: false,
        },
        Caso {
            nome: "Ballpoint",
            preset: Some("Ballpoint"),
            borracha: false,
        },
        Caso {
            nome: "Marker Liner",
            preset: Some("Marker Liner"),
            borracha: false,
        },
    ];
    for c in &casos {
        let apaga = c.borracha || c.preset.is_some_and(|n| n.contains("Eraser"));
        let mut linha = format!("{:<28}", c.nome);
        for &p in &[0.3f32, 0.5, 0.86, 1.0] {
            let mut t = monta(&bib, c);
            linha.push_str(&format!(" p={p:.2}:"));
            for _ in 0..4 {
                passada(&mut t, p);
                linha.push_str(&format!("{:4}", feito(&t, 0, apaga)));
            }
        }
        // a mesma, sem pressão→opacidade (só a Kneaded/Hard Eraser e os pincéis que a têm)
        let mut t = monta(&bib, c);
        t.paint.dynamics.strength_pressure = false;
        passada(&mut t, 0.3);
        linha.push_str(&format!(" | p=0.3 sem p->op:{:4}", feito(&t, 0, apaga)));
        let mut t = monta(&bib, c);
        t.paint.brush.borda_px = 0.0;
        t.paint.brush.traco = ph2d_painter_brush::traco::TracoComposto::NEUTRO;
        t.paint.brush.accumulate = true;
        passada(&mut t, 0.3);
        linha.push_str(&format!(" acc ON:{:4}", feito(&t, 0, apaga)));
        println!("{linha}");
    }
}

/// O RATO (pressão `1.0`, tilt 0 — `ph2d-pen/src/ponteiro.rs` `do_rato`) em TODOS os presets de fábrica, no
/// meio dele e no Digital: centro e máximo de UMA passada, e o factor que o limita (cada um desligado à vez).
#[test]
#[ignore = "sonda à mão"]
fn sonda_opacidade_com_o_rato_por_pincel() {
    use super::PaintMedia;
    let bib = presets();
    println!("presets: {}  erros: {}", bib.entradas.len(), bib.erros.len());
    let fatores: Vec<Mexe> = vec![
        ("flow1", |t| t.paint.brush.flow = 1.0),
        ("accON", |t| t.paint.brush.accumulate = true),
        ("tecto0", |t| {
            t.paint.brush.traco = ph2d_painter_brush::traco::TracoComposto::NEUTRO
        }),
        ("grain0", |t| t.paint.brush.grain_depth = 0.0),
        ("tex0", |t| t.paint.brush.texture = Default::default()),
        ("shape0", |t| t.paint.brush.shape = Default::default()),
        ("dual0", |t| t.paint.brush.dual = None),
        ("hard1", |t| t.paint.brush.hardness = 1.0),
        ("duro", |t| {
            t.paint.brush.falloff = Falloff::Constant;
            t.paint.brush.hardness = 1.0;
        }),
        ("spc.02", |t| t.paint.brush.spacing = 0.02),
        ("jit0", |t| t.paint.brush.dab_jitter = Default::default()),
        ("spcAtt0", |t| t.paint.brush.space_attenuation = false),
        ("p->op0", |t| t.paint.dynamics.strength_pressure = false),
        ("pen0", |t| t.paint.dynamics.pen = Default::default()),
        ("vel0", |t| t.paint.dynamics.speed_opacity = 0.0),
        ("borda0", |t| t.paint.brush.borda_px = 0.0),
        ("taper0", |t| t.paint.brush.taper = Default::default()),
        ("opac1", |t| t.paint.brush.opacity = 1.0),
    ];
    let mut linhas: Vec<(u8, String)> = Vec::new();
    for e in &bib.entradas {
        let nome = e.preset.parametros.name.clone();
        let monta_em = |digital: bool| -> PainterTool {
            let mut t = PainterTool::default();
            t.set_source([255u8; 4].repeat(W * H), W as u32, H as u32);
            let _ = t.preset_aplicar(&e.preset);
            if digital {
                t.set_paint_media(PaintMedia::Digital);
            }
            let apaga = t.paint.eraser || nome.contains("Eraser");
            if apaga {
                t.set_source([200u8, 50, 50, 255].repeat(W * H), W as u32, H as u32);
            }
            t.set_brush_size_px(DIAM);
            t.set_brush_color_srgb8([0, 0, 0]);
            t.set_brush_strength(1.0);
            t
        };
        let proprio = monta_em(false).paint_media();
        let meios: Vec<bool> = if proprio == PaintMedia::Digital {
            vec![true]
        } else {
            vec![false, true]
        };
        for digital in meios {
            let medir = |t: &PainterTool| -> (u8, u8) {
                let apaga = t.paint.eraser || nome.contains("Eraser");
                let c = feito(t, 0, apaga);
                let mut mx = 0u8;
                for dy in -20i32..=20 {
                    let y = (Y as i32 + dy) as usize;
                    for x in (X0 as usize + 30)..(X1 as usize - 30) {
                        let i = (y * W + x) * 4;
                        let v = if apaga {
                            255 - t.canvas_rgba[i + 3]
                        } else {
                            255 - t.canvas_rgba[i]
                        };
                        mx = mx.max(v);
                    }
                }
                (c, mx)
            };
            let mut t = monta_em(digital);
            let b = t.paint.brush;
            let d = t.paint.dynamics;
            let meio = format!("{:?}", t.paint_media());
            passada(&mut t, 1.0);
            let (c, mx) = medir(&t);
            let mut fs = String::new();
            for (n, f) in &fatores {
                let mut a = monta_em(digital);
                f(&mut a);
                passada(&mut a, 1.0);
                let (ca, _) = medir(&a);
                if ca.abs_diff(c) >= 4 {
                    fs.push_str(&format!(" {n}={ca}"));
                }
            }
            let mut tudo = monta_em(digital);
            for (_, f) in &fatores {
                f(&mut tudo);
            }
            passada(&mut tudo, 1.0);
            let (ct, _) = medir(&tudo);
            linhas.push((
                c,
                format!(
                    "{:<22} {:<10} c={:3} max={:3} | flow {:.3} acc {} er {} p->op {} min {:.2} pen {:.2}/{:.2} velOp {:.2} grain {:.2} tecto {:.2} spcAtt {} spc {:.3} hard {:.2} {:?} jitOp {:.2} | TUDO={ct} |{fs}",
                    nome,
                    meio,
                    c,
                    mx,
                    b.flow,
                    b.accumulate,
                    t.paint.eraser,
                    d.strength_pressure,
                    d.strength_min,
                    d.pen.floor,
                    d.pen.ceiling,
                    d.speed_opacity,
                    b.grain_depth,
                    b.traco.tecto,
                    b.space_attenuation,
                    b.spacing,
                    b.hardness,
                    b.falloff,
                    b.dab_jitter.opacity,
                ),
            ));
        }
    }
    linhas.sort_by_key(|(c, _)| *c);
    for (_, l) in &linhas {
        println!("{l}");
    }
    // a pergunta aberta: Soft Round, Accumulate ON não sobe o centro a p=0,3
    let c = Caso {
        nome: "Soft Round",
        preset: Some("Soft Round"),
        borracha: false,
    };
    for &p in &[0.3f32, 1.0] {
        let mut s = format!("Soft Round p={p}:");
        for (n, f) in [
            ("base", (|_| {}) as fn(&mut PainterTool)),
            ("accON", |t| t.paint.brush.accumulate = true),
            ("accOFF", |t| t.paint.brush.accumulate = false),
            ("accON+tecto0", |t| {
                t.paint.brush.accumulate = true;
                t.paint.brush.traco = ph2d_painter_brush::traco::TracoComposto::NEUTRO;
            }),
            ("accON+tecto0+borda0", |t| {
                t.paint.brush.accumulate = true;
                t.paint.brush.borda_px = 0.0;
                t.paint.brush.traco = ph2d_painter_brush::traco::TracoComposto::NEUTRO;
            }),
            ("accOFF+tecto0", |t| {
                t.paint.brush.accumulate = false;
                t.paint.brush.traco = ph2d_painter_brush::traco::TracoComposto::NEUTRO;
            }),
            ("accON+spc.02", |t| {
                t.paint.brush.accumulate = true;
                t.paint.brush.spacing = 0.02;
            }),
            ("accON+tecto0+spc.02", |t| {
                t.paint.brush.accumulate = true;
                t.paint.brush.spacing = 0.02;
                t.paint.brush.traco = ph2d_painter_brush::traco::TracoComposto::NEUTRO;
            }),
        ] {
            let mut t = monta(&bib, &c);
            f(&mut t);
            passada(&mut t, p);
            s.push_str(&format!(" {n}={}", feito(&t, 0, false)));
        }
        let t = monta(&bib, &c);
        s.push_str(&format!(
            " | cover_wanted {} tecto {:.2}/{} flow {:.3} spc {:.3}",
            t.stroke_cover_wanted(&t.paint.brush),
            t.paint.brush.traco.tecto,
            t.paint.brush.traco.tecto_por_dab,
            t.paint.brush.flow,
            t.paint.brush.spacing
        ));
        println!("{s}");
    }
}
