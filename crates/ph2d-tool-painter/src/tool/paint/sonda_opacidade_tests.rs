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
    println!(
        "presets: {}  erros: {}",
        bib.entradas.len(),
        bib.erros.len()
    );
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

/// Um traço recto de `a` a `b` (passo 3 px) à pressão `p`.
fn recta(t: &mut PainterTool, a: [f32; 2], b: [f32; 2], p: f32) {
    let ev = |pos, phase| CanvasPointer {
        pos,
        pressure: p,
        tilt: [0.0, 0.0],
        phase,
    };
    let n = ((b[0] - a[0]).hypot(b[1] - a[1]) / 3.0).ceil().max(1.0) as u32;
    t.on_canvas_pointer(ev(a, PointerPhase::Down));
    for k in 1..=n {
        let f = k as f32 / n as f32;
        let pos = [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f];
        t.on_canvas_pointer(ev(pos, PointerPhase::Move));
    }
    t.on_canvas_pointer(ev(b, PointerPhase::Up));
}

/// Os presets de fábrica de cada ferramenta (24/10), aplicados pela porta do navegador (a ferramenta vem na mão),
/// a 40 px: o que UMA passada faz com o rato (1) e com a mão do dono (0,24 · 0,5 · 0,86).
#[test]
#[ignore = "sonda à mão"]
fn sonda_ferramentas_de_fabrica() {
    let bib = presets();
    let monta = |nome: &str, fundo: &dyn Fn(usize, usize) -> [u8; 4]| {
        let mut t = PainterTool::default();
        let px: Vec<u8> = (0..H)
            .flat_map(|y| (0..W).flat_map(move |x| fundo(x, y)))
            .collect();
        t.set_source(px, W as u32, H as u32);
        let p = &bib
            .entradas
            .iter()
            .find(|e| e.preset.parametros.name == nome)
            .unwrap_or_else(|| panic!("preset {nome}"))
            .preset;
        assert!(t.preset_aplicar(p).expect("meio").is_empty());
        t.set_brush_size_px(DIAM * 0.5);
        t
    };
    let px = |t: &PainterTool, x: usize, y: usize| {
        let i = (y * W + x) * 4;
        [
            t.canvas_rgba[i],
            t.canvas_rgba[i + 1],
            t.canvas_rgba[i + 2],
            t.canvas_rgba[i + 3],
        ]
    };
    let cheio = |_: usize, _: usize| [200u8, 50, 50, 255];
    for nome in ["Hard Eraser", "Kneaded Eraser", "Texture Eraser"] {
        let mut linha = format!("{nome:<15}");
        for p in [1.0f32, 0.24, 0.5, 0.86] {
            let mut t = monta(nome, &cheio);
            passada(&mut t, p);
            let apagado = |x, y| 255 - px(&t, x, y)[3];
            let largura = (0..H).filter(|&y| apagado(PX, y) >= 128).count();
            let v: Vec<f64> = (60..196)
                .flat_map(|x| (Y as usize - 6..Y as usize + 6).map(move |y| (x, y)))
                .map(|(x, y)| f64::from(apagado(x, y)))
                .collect();
            let m = v.iter().sum::<f64>() / v.len() as f64;
            let dp = (v.iter().map(|a| (a - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt();
            linha.push_str(&format!(
                " | p {p:.2}: centro {:3} largura {largura:2} miolo {m:5.1}±{dp:4.1}",
                apagado(PX, Y as usize)
            ));
        }
        println!("{linha}");
    }
    // Blur e Smear: a metade de cima vermelha, a de baixo branca (a orla em y = Y).
    let metade = |_: usize, y: usize| {
        if y < Y as usize {
            [200u8, 50, 50, 255]
        } else {
            [255; 4]
        }
    };
    let verde_lim = |g: u8| f64::from(255 - g) / 205.0;
    for p in [1.0f32, 0.5] {
        let mut t = monta("Blur Basic", &metade);
        passada(&mut t, p);
        let orla = (Y as usize - 30..Y as usize + 30)
            .filter(|&y| (0.1..0.9).contains(&verde_lim(px(&t, PX, y)[1])))
            .count();
        let mut t = monta("Smear Basic", &metade);
        recta(&mut t, [PX as f32, Y - 30.0], [PX as f32, Y + 50.0], p);
        let arrasto: f64 = (Y as usize + 1..H)
            .map(|y| verde_lim(px(&t, PX, y)[1]))
            .sum();
        let alcance = (Y as usize + 1..H)
            .filter(|&y| verde_lim(px(&t, PX, y)[1]) > 0.1)
            .count();
        println!(
            "p {p:.2}: Blur Basic orla 10–90 % {orla} px (antes 0) · Smear Basic vermelho arrastado {arrasto:.1} px, alcance {alcance} px"
        );
    }
    // Clone: uma barra vermelha em x 40..80; a fonte no meio dela, o traço de 150 a 226 copia-a para 150..190.
    let barra = |x: usize, _: usize| {
        if (40..80).contains(&x) {
            [200u8, 50, 50, 255]
        } else {
            [255; 4]
        }
    };
    for p in [1.0f32, 0.5] {
        let mut t = monta("Clone Basic", &barra);
        t.set_clone_source([60.0, Y]);
        recta(&mut t, [150.0, Y], [226.0, Y], p);
        let y = Y as usize;
        let iguais = (150..226)
            .filter(|&x| {
                let (a, b) = (px(&t, x, y), barra(x - 90, y));
                (0..3).all(|k| a[k].abs_diff(b[k]) <= 8)
            })
            .count();
        let vermelhos = (150..226).filter(|&x| px(&t, x, y)[1] < 128).count();
        println!(
            "p {p:.2}: Clone Basic {iguais}/76 px iguais à fonte (x − 90), {vermelhos} px vermelhos (a barra tem 40)"
        );
    }
}
