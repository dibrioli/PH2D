//! **Medição W0(c) do Óleo Real** (doc `49_realshader/plano.md`): a escala de altura do Wet Paint nas pinceladas
//! equivalentes às sondas do Rebelle (`49_realshader/ferramentas/tracos.py`). Imprime, nunca barra:
//! `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib medida_oleo_real_altura -- --ignored --nocapture`.
use crate::{PaintMedia, PainterTool};
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_wet_paint::tuning::Knob;

const W: u32 = 1200;
const H: u32 = 800;
const AZ: [u8; 3] = [20, 150, 230];
const BR: [u8; 3] = [240, 240, 236];
const ES: [u8; 3] = [30, 25, 60];

fn ponto(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn biblioteca() -> Biblioteca {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    Biblioteca::carregar(&raiz, None)
}

/// Os knobs do relevo de cada pincel de fábrica do Wet Paint, depois de aplicado.
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_altura_knobs_de_fabrica() {
    let b = biblioteca();
    println!(
        "{:<44} {:>7} {:>7} {:>7} {:>7} {:>7} {:>8}",
        "preset", "Emboss", "EmbExp", "EmbRef", "BrHgt", "BrRem", "Pigment"
    );
    for e in b.entradas.iter().filter(|e| e.partes()[0] == "WetPainter") {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
        t.set_paint_media(PaintMedia::WetPaint);
        let _ = t.wet_preset_aplicar(&e.preset);
        let k = &t.paint.wetpaint.knobs;
        println!(
            "{:<44} {:>7.3} {:>7.3} {:>7.1} {:>7.3} {:>7.3} {:>8.1}",
            e.relativo,
            k.get(Knob::Emboss),
            k.get(Knob::EmbossSizeExp),
            k.get(Knob::EmbossSizeRef),
            k.get(Knob::BristleHeight),
            k.get(Knob::BristleRemove),
            k.get(Knob::PigmentPerDab),
        );
    }
}

struct Traco {
    rotulo: &'static str,
    preset: String,
    cor: [u8; 3],
    a: [f32; 2],
    b: [f32; 2],
    diametro: f32,
}

fn pct(v: &mut [f64], p: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    let i = ((v.len() - 1) as f64 * p).round() as usize;
    v[i]
}

/// As pinceladas do `tracos.py` com os NOSSOS pincéis de fábrica, numa tela 1200×800, como no Rebelle.
/// Presets por env (`PH2D_OLEO_FACA`, `PH2D_OLEO_FACA_TEX`, `PH2D_OLEO_OILY`, `PH2D_OLEO_THICK`: o `relativo`).
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_altura_tracos() {
    let env = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_owned());
    let faca = env("PH2D_OLEO_FACA", "WetPainter/Oil/Dabber/Dabber.ph2d-brush.png");
    let faca_tex = env("PH2D_OLEO_FACA_TEX", &faca);
    let oily = env("PH2D_OLEO_OILY", "WetPainter/Oil/Flat/Filbert.ph2d-brush.png");
    let thick = env("PH2D_OLEO_THICK", "WetPainter/Oil/Bristle/Oil Bristle.ph2d-brush.png");
    let tracos = [
        Traco { rotulo: "faca H azul", preset: faca.clone(), cor: AZ, a: [80.0, 110.0], b: [560.0, 110.0], diametro: 120.0 },
        Traco { rotulo: "faca V azul", preset: faca.clone(), cor: AZ, a: [700.0, 60.0], b: [700.0, 420.0], diametro: 120.0 },
        Traco { rotulo: "oily azul", preset: oily.clone(), cor: AZ, a: [850.0, 100.0], b: [1150.0, 100.0], diametro: 100.0 },
        Traco { rotulo: "thick azul", preset: thick.clone(), cor: AZ, a: [850.0, 260.0], b: [1150.0, 260.0], diametro: 100.0 },
        Traco { rotulo: "faca H branco", preset: faca.clone(), cor: BR, a: [80.0, 300.0], b: [560.0, 300.0], diametro: 120.0 },
        Traco { rotulo: "faca H escuro", preset: faca.clone(), cor: ES, a: [80.0, 480.0], b: [560.0, 480.0], diametro: 120.0 },
        Traco { rotulo: "faca-tex azul", preset: faca_tex.clone(), cor: AZ, a: [700.0, 560.0], b: [1150.0, 560.0], diametro: 120.0 },
        Traco { rotulo: "oily escuro", preset: oily.clone(), cor: ES, a: [850.0, 720.0], b: [1150.0, 720.0], diametro: 100.0 },
    ];
    let b = biblioteca();
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    for tr in &tracos {
        let p = &b
            .entradas
            .iter()
            .find(|e| e.relativo == tr.preset)
            .unwrap_or_else(|| panic!("preset {} não existe", tr.preset))
            .preset;
        let erros = t.wet_preset_aplicar(p);
        assert!(erros.is_empty(), "{erros:?}");
        t.set_brush_size_px(tr.diametro / 2.0);
        t.set_brush_color_srgb8(tr.cor);
        // O passo de 3 px do tracos.py.
        let n = ((tr.b[0] - tr.a[0]).hypot(tr.b[1] - tr.a[1]) / 3.0).ceil() as u32;
        t.on_canvas_pointer(ponto(tr.a, PointerPhase::Down));
        for k in 1..=n {
            let f = k as f32 / n as f32;
            let pos = [tr.a[0] + (tr.b[0] - tr.a[0]) * f, tr.a[1] + (tr.b[1] - tr.a[1]) * f];
            t.on_canvas_pointer(ponto(pos, PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(ponto(tr.b, PointerPhase::Up));
        let mut ticks = 0;
        loop {
            t.on_tick(1000.0 / 60.0);
            ticks += 1;
            let s = t.paint.wetpaint.session.as_mut().expect("sessão");
            s.bring_home();
            if !s.engine.active_grid().has_fluid || ticks >= 60 * 60 {
                break;
            }
        }
        println!("[oleo] {} ({}): água parou após {} ticks", tr.rotulo, tr.preset, ticks);
    }
    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    let ratio = s.ratio;
    let (gw, gh) = s.grid;
    let g = s.engine.active_grid();
    let st = g.s;
    let tem_altura = !g.altura.is_empty();
    println!(
        "[oleo] razão (px de tela por célula) = {ratio} · grade {gw}×{gh} · altura preenchida: {tem_altura} \
         (não-zero: {})",
        g.altura.iter().filter(|&&h| h != 0.0).count()
    );
    let m = |i: usize| f64::from(g.sett[i]) + f64::from(g.susp[i]);
    let h = |i: usize| if tem_altura { f64::from(g.altura[i]) } else { 0.0 };
    let emb_cru = |i: usize| -> (f64, f64, f64) {
        let k = f64::from(g.relevo[i]);
        let e_m = ((m(i + 1) - m(i - 1)) * 0.5 + (m(i + st) - m(i - st)) * 1.0) * 0.008 * k;
        let e_h = ((h(i + 1) - h(i - 1)) * 0.5 + (h(i + st) - h(i - st)) * 1.0)
            * ph2d_wet_paint::bristles_relevo::LUZ
            * k;
        (e_m + e_h, e_m, e_h)
    };
    let r = f32::from(ratio);
    let cel = |x: f32, y: f32| -> usize {
        let (cx, cy) = ((x / r) as usize + 1, (y / r) as usize + 1);
        cy * st + cx
    };
    println!(
        "{:<14} {:>6} | {:>8} {:>8} {:>8} {:>8} | {:>6} {:>6} {:>6} {:>6} | {:>6} {:>6} {:>6} {:>6} | {:>6} {:>6} | {:>6}",
        "traço", "células", "m p50", "m p90", "m p99", "m max", "h p50", "h p90", "h p99", "h max",
        "|e|p50", "|e|p90", "|e|p99", "|e|max", "|em|99", "|eh|99", "clamp%"
    );
    for tr in &tracos {
        let rad = tr.diametro / 2.0 + 10.0;
        let (x0, x1) = (tr.a[0].min(tr.b[0]) - rad, tr.a[0].max(tr.b[0]) + rad);
        let (y0, y1) = (tr.a[1].min(tr.b[1]) - rad, tr.a[1].max(tr.b[1]) + rad);
        let (mut vm, mut vh, mut ve, mut vem, mut veh) = (vec![], vec![], vec![], vec![], vec![]);
        let mut morde = 0usize;
        let mut relevos = vec![];
        let mut y = y0.max(0.0);
        while y < y1.min(H as f32) {
            let mut x = x0.max(0.0);
            while x < x1.min(W as f32) {
                let i = cel(x, y);
                if m(i) > 0.0 {
                    vm.push(m(i));
                    vh.push(h(i));
                    let (e, em, eh) = emb_cru(i);
                    if e.abs() > 40.0 {
                        morde += 1;
                    }
                    ve.push(e.abs());
                    vem.push(em.abs());
                    veh.push(eh.abs());
                    relevos.push(f64::from(g.relevo[i]));
                }
                x += r;
            }
            y += r;
        }
        let n = vm.len();
        println!(
            "{:<14} {:>6} | {:>8.1} {:>8.1} {:>8.1} {:>8.1} | {:>6.3} {:>6.3} {:>6.3} {:>6.3} | {:>6.2} {:>6.2} {:>6.2} {:>6.2} | {:>6.2} {:>6.2} | {:>6.2}  relevo p50 {:.3}",
            tr.rotulo, n,
            pct(&mut vm, 0.5), pct(&mut vm, 0.9), pct(&mut vm, 0.99), pct(&mut vm, 1.0),
            pct(&mut vh, 0.5), pct(&mut vh, 0.9), pct(&mut vh, 0.99), pct(&mut vh, 1.0),
            pct(&mut ve, 0.5), pct(&mut ve, 0.9), pct(&mut ve, 0.99), pct(&mut ve, 1.0),
            pct(&mut vem, 0.99), pct(&mut veh, 0.99),
            100.0 * morde as f64 / n.max(1) as f64,
            pct(&mut relevos, 0.5),
        );
    }
    // O perfil da borda DIREITA da faca vertical, a meio (y = 240): 40 amostras de 2 px centradas na borda.
    let yc = 240.0f32;
    let borda = (700..W as i32)
        .map(|x| x as f32)
        .take_while(|&x| m(cel(x, yc)) > 10.0)
        .last()
        .unwrap_or(700.0);
    println!("[oleo] perfil faca V em y={yc}: borda direita (última célula com m>10) em x={borda}");
    println!("{:>6} {:>9} {:>8} {:>8}", "x", "m", "altura", "emb");
    for k in -20..20 {
        let x = borda + 2.0 * k as f32;
        let i = cel(x, yc);
        println!("{:>6} {:>9.1} {:>8.3} {:>8.2}", x, m(i), h(i), emb_cru(i).0);
    }
}
