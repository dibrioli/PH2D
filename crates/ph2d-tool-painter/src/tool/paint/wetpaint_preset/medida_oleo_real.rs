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
    let mut t = pinta_folha();
    imprime_alturas(&mut t);
}

/// A folha do `tracos.py` sai em `$PH2D_OLEO_PNG/hoje.ppm` (o render do produto com o emboss do preset) e
/// `oleo.ppm` (o Óleo Real), sobre papel branco — para pôr ao lado de `capturas/A_base.jpg`.
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_folha_png() {
    let dir = std::path::PathBuf::from(std::env::var("PH2D_OLEO_PNG").expect("PH2D_OLEO_PNG=<pasta>"));
    let mut t = pinta_folha();
    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    let (gw, gh) = s.grid;
    let params = s.engine.sim.gather_params(&s.engine.tuning);
    let camadas: Vec<ph2d_wet_paint::render::RenderLayer<'_>> = s
        .engine
        .layers
        .iter()
        .map(|l| ph2d_wet_paint::render::RenderLayer { grid: &l.grid, opacity: l.opacity, visible: l.visible })
        .collect();
    let visual = ph2d_wet_paint::render::PigmentVisual { paper: false, relevo: true };
    let o = oleo_do_env();
    let grades: Vec<(&ph2d_wet_paint::grid::Grid, bool)> = camadas.iter().map(|c| (c.grid, c.visible)).collect();
    let campo = ph2d_wet_paint::oleo_real::CampoAltura::de(&grades, &o, usize::from(s.ratio), 1, 1, gw, gh);
    // Os termos em cinza (0..1 → 0..255; `e` e o brilho a ½ de escala), só onde há tinta.
    let mut termos = [(); 5].map(|()| vec![0u8; gw * gh]);
    let ativa = s.engine.active_grid();
    for cy in 1..=gh {
        for cx in 1..=gw {
            if ativa.sett[cx + cy * ativa.s] + ativa.susp[cx + cy * ativa.s] <= 0.0 {
                continue;
            }
            let t = ph2d_wet_paint::oleo_real::termos(&o, &campo, cx, cy);
            let k = (cx - 1) + (cy - 1) * gw;
            for (j, v) in [t.cav, t.luz, t.e * 0.5, t.espec * 0.5, t.ocl].into_iter().enumerate() {
                termos[j][k] = (v.clamp(0.0, 1.0) * 254.0) as u8 + 1;
            }
        }
    }
    for (nome, v) in ["t_cav", "t_luz", "t_e", "t_espec", "t_ocl"].iter().zip(&termos) {
        let mut pgm = format!("P5\n{gw} {gh}\n255\n").into_bytes();
        pgm.extend_from_slice(v);
        std::fs::write(dir.join(format!("{nome}.pgm")), pgm).expect("grava");
    }
    for (nome, com_oleo) in [("hoje", false), ("oleo", true)] {
        let mut px = vec![0u8; gw * gh * 4];
        for cy in 1..=gh {
            let linha = &mut px[(cy - 1) * gw * 4..cy * gw * 4];
            if com_oleo {
                ph2d_wet_paint::render::render_pigment_row_oleo(Some(&params), &camadas, visual, &o, &campo, cy, 1, gw, linha);
            } else {
                ph2d_wet_paint::render::render_pigment_row_visual(Some(&params), &camadas, visual, cy, 1, gw, linha);
            }
        }
        let mut ppm = format!("P6\n{gw} {gh}\n255\n").into_bytes();
        for p in px.chunks_exact(4) {
            let a = u32::from(p[3]);
            for c in 0..3 {
                ppm.push(((u32::from(p[c]) * a + 255 * (255 - a)) / 255) as u8);
            }
        }
        let f = dir.join(format!("{nome}.ppm"));
        std::fs::write(&f, ppm).expect("grava");
        println!("[oleo] {}", f.display());
    }
}

/// O Óleo Real de fábrica com o que `PH2D_OLEO_AF="campo=valor,…"` pedir (knobs em graus para a luz).
fn oleo_do_env() -> ph2d_wet_paint::oleo_real::OleoReal {
    let mut o = ph2d_wet_paint::oleo_real::OleoReal::default();
    for par in std::env::var("PH2D_OLEO_AF").unwrap_or_default().split(',').filter(|p| !p.is_empty()) {
        let (k, v) = par.split_once('=').unwrap_or_else(|| panic!("{par}: campo=valor"));
        let v: f64 = v.parse().unwrap_or_else(|_| panic!("{par}: número"));
        let af = &mut o.af;
        match k {
            "profundidade" => o.profundidade = v,
            "azimute" => o.azimute = v.to_radians(),
            "altitude" => o.altitude = v.to_radians(),
            "sombra" => o.sombra = v,
            "brilho" => o.brilho = v,
            "px_por_massa" => af.px_por_massa = v,
            "raio_suave" => af.raio_suave = v as usize,
            "grao" => af.grao = v,
            "fios" => af.fios = v,
            "limiar_cavidade" => af.limiar_cavidade = v,
            "fundo_cavidade" => af.fundo_cavidade = v,
            "s_fenda" => af.s_fenda = v,
            "k_fenda" => af.k_fenda = v,
            "ceu" => af.ceu = v,
            "penumbra" => af.penumbra = v,
            "rugosidade" => af.rugosidade = v,
            "brilho_max" => af.brilho_max = v,
            _ => panic!("{k}: campo desconhecido"),
        }
    }
    println!("[oleo] {o:?}");
    o
}

fn pinta_folha() -> PainterTool {
    let b = biblioteca();
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    pinta_folha_com(&mut t, &b);
    t
}

fn tracos() -> Vec<Traco> {
    let env = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_owned());
    let faca = env("PH2D_OLEO_FACA", "WetPainter/Oil/Dabber/Dabber.ph2d-brush.png");
    let faca_tex = env("PH2D_OLEO_FACA_TEX", &faca);
    let oily = env("PH2D_OLEO_OILY", "WetPainter/Oil/Flat/Filbert.ph2d-brush.png");
    let thick = env("PH2D_OLEO_THICK", "WetPainter/Oil/Bristle/Oil Bristle.ph2d-brush.png");
    vec![
        Traco { rotulo: "faca H azul", preset: faca.clone(), cor: AZ, a: [80.0, 110.0], b: [560.0, 110.0], diametro: 120.0 },
        Traco { rotulo: "faca V azul", preset: faca.clone(), cor: AZ, a: [700.0, 60.0], b: [700.0, 420.0], diametro: 120.0 },
        Traco { rotulo: "oily azul", preset: oily.clone(), cor: AZ, a: [850.0, 100.0], b: [1150.0, 100.0], diametro: 100.0 },
        Traco { rotulo: "thick azul", preset: thick.clone(), cor: AZ, a: [850.0, 260.0], b: [1150.0, 260.0], diametro: 100.0 },
        Traco { rotulo: "faca H branco", preset: faca.clone(), cor: BR, a: [80.0, 300.0], b: [560.0, 300.0], diametro: 120.0 },
        Traco { rotulo: "faca H escuro", preset: faca.clone(), cor: ES, a: [80.0, 480.0], b: [560.0, 480.0], diametro: 120.0 },
        Traco { rotulo: "faca-tex azul", preset: faca_tex.clone(), cor: AZ, a: [700.0, 560.0], b: [1150.0, 560.0], diametro: 120.0 },
        Traco { rotulo: "oily escuro", preset: oily.clone(), cor: ES, a: [850.0, 720.0], b: [1150.0, 720.0], diametro: 100.0 },
    ]
}

fn pinta_folha_com(t: &mut PainterTool, b: &Biblioteca) {
    for tr in &tracos() {
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
}

fn imprime_alturas(t: &mut PainterTool) {
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
    for tr in &tracos() {
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

// ── W0b: a espátula exprime-se com os knobs de hoje? ────────────────────────────────────────────────
// Um traço horizontal (tamanho 120, 450 px, azul, pressão 1, passo 3 px, relógio fixo, espera a água) por
// candidata, num tool novo. Mede a faixa: largura (m>10), rampa da borda 90→10 % do patamar, patamar p50,
// ruído célula-a-célula (m − média 3×3) no miolo 60 %, e orla (máx a ≤5 px da borda ÷ patamar).

const FW: u32 = 700;
const FH: u32 = 320;
const FY: f32 = 160.0;
const FX0: f32 = 100.0;
const FX1: f32 = 550.0;

type Ajuste = fn(&mut PainterTool);

fn lisa_dura(t: &mut PainterTool) {
    t.paint.brush.jitter = 0.0;
    t.set_brush_hardness(1.0);
    t.set_brush_falloff(ph2d_painter_brush::Falloff::Constant as u8);
    t.set_brush_shape_kind(ph2d_painter_brush::TextureKind::None.to_u8());
}

fn quadrada(t: &mut PainterTool) {
    lisa_dura(t);
    t.set_brush_dab_quina(1.0);
}

fn quadrada_sem_dente(t: &mut PainterTool) {
    quadrada(t);
    t.paint.wetpaint.knobs.set(Knob::PaperGate, 0.0);
}

/// A lâmina: quadrada, lisa, dura, sem dente, achatada ATRAVÉS do traço (o fio da faca perpendicular à marcha).
fn lamina(t: &mut PainterTool) {
    quadrada_sem_dente(t);
    t.set_brush_dab_flatten(0.8);
    t.set_brush_dab_angle(90.0);
}

/// A lâmina com a curva Custom de queda: miolo a 0,6, pico 1 a 0,85 do raio, 0 na borda (cantos Vector) — a orla
/// pela composição, sem lei nova.
fn lamina_com_orla(t: &mut PainterTool) {
    lamina(t);
    t.set_brush_hardness(0.0);
    t.set_brush_falloff(ph2d_painter_brush::Falloff::Custom as u8);
    t.set_brush_falloff_point(0, 0.0, 0.6);
    t.set_brush_falloff_point(1, 1.0, 0.0);
    let pico = t.add_brush_falloff_point_at(0.85, 1.0).expect("ponto");
    for id in [0, 1, pico] {
        t.set_brush_falloff_point_handle(id, 1);
    }
}

fn candidatas() -> Vec<(&'static str, &'static str, Ajuste)> {
    const DAB: &str = "WetPainter/Oil/Dabber/Dabber.ph2d-brush.png";
    const FIL: &str = "WetPainter/Oil/Flat/Filbert.ph2d-brush.png";
    const MRK: &str = "WetPainter/Marker/Marker/Marker Bullet.ph2d-brush.png";
    vec![
        ("A Dabber fabrica", DAB, |_| {}),
        ("B +jitter0", DAB, |t| t.paint.brush.jitter = 0.0),
        ("C +dura1 Constant", DAB, |t| {
            t.paint.brush.jitter = 0.0;
            t.set_brush_hardness(1.0);
            t.set_brush_falloff(ph2d_painter_brush::Falloff::Constant as u8);
        }),
        ("D C+sem cerdas", DAB, lisa_dura),
        ("E D+quina1", DAB, quadrada),
        ("F E+gate0", DAB, quadrada_sem_dente),
        ("G F+flat.8 ang0", DAB, |t| {
            quadrada_sem_dente(t);
            t.set_brush_dab_flatten(0.8);
            t.set_brush_dab_angle(0.0);
        }),
        ("H F+flat.8 ang90", DAB, |t| {
            quadrada_sem_dente(t);
            t.set_brush_dab_flatten(0.8);
            t.set_brush_dab_angle(90.0);
        }),
        ("I F+lev0 cap0", DAB, |t| {
            quadrada_sem_dente(t);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 0.0);
            t.paint.wetpaint.knobs.set(Knob::Capillary, 0.0);
        }),
        ("J F+lev2", DAB, |t| {
            quadrada_sem_dente(t);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 2.0);
            t.paint.wetpaint.knobs.set(Knob::LevelingOwnStroke, 1.0);
        }),
        ("K F+agua0", DAB, |t| {
            quadrada_sem_dente(t);
            t.paint.wetpaint.knobs.water = 0.0;
        }),
        ("L F+pig500", DAB, |t| {
            quadrada_sem_dente(t);
            t.paint.wetpaint.knobs.set(Knob::PigmentPerDab, 500.0);
        }),
        ("M F+drag0 pick0", DAB, |t| {
            quadrada_sem_dente(t);
            t.paint.wetpaint.knobs.set(Knob::Drag, 0.0);
            t.paint.wetpaint.knobs.set(Knob::Pickup, 0.0);
        }),
        ("H2 H+pig500", DAB, |t| {
            lamina(t);
            t.paint.wetpaint.knobs.set(Knob::PigmentPerDab, 500.0);
        }),
        ("H3 H+lev0", DAB, |t| {
            lamina(t);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 0.0);
        }),
        ("H4 H+edgeDark50", DAB, |t| {
            lamina(t);
            t.paint.wetpaint.knobs.set(Knob::EdgeDarkening, 50.0);
        }),
        ("H5 H+edgeDark200", DAB, |t| {
            lamina(t);
            t.paint.wetpaint.knobs.set(Knob::EdgeDarkening, 200.0);
        }),
        ("H6 H+heavyEdge1", DAB, |t| {
            lamina(t);
            t.set_wet_borda_que_pesa(1.0);
        }),
        ("H7 H+flat.95", DAB, |t| {
            lamina(t);
            t.set_brush_dab_flatten(0.95);
        }),
        ("R H sem quina", DAB, |t| {
            lamina(t);
            t.set_brush_dab_quina(0.0);
        }),
        ("S R+lev0", DAB, |t| {
            lamina(t);
            t.set_brush_dab_quina(0.0);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 0.0);
        }),
        ("T D+gate0 lev0", DAB, |t| {
            lisa_dura(t);
            t.paint.wetpaint.knobs.set(Knob::PaperGate, 0.0);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 0.0);
        }),
        ("U lamina curva-orla", DAB, lamina_com_orla),
        ("V U+lev0", DAB, |t| {
            lamina_com_orla(t);
            t.paint.wetpaint.knobs.set(Knob::Leveling, 0.0);
        }),
        ("N Filbert fabrica", FIL, |_| {}),
        ("O Filbert+F", FIL, quadrada_sem_dente),
        ("P Marker fabrica", MRK, |_| {}),
        ("Q Marker+F", MRK, quadrada_sem_dente),
    ]
}

fn mediana(mut v: Vec<f64>) -> f64 {
    pct(&mut v, 0.5)
}

/// Onde, andando de `de` para fora (passo `dir` = ±1 em linhas), a coluna `m` cruza `alvo`
/// pela primeira vez — em linhas fracionárias (interpolação linear), `None` se nunca.
fn cruza(m: &[f64], de: usize, dir: i64, alvo: f64) -> Option<f64> {
    let mut i = de as i64;
    loop {
        let j = i + dir;
        if j < 0 || j as usize >= m.len() {
            return None;
        }
        let (a, b) = (m[i as usize], m[j as usize]);
        if a >= alvo && b < alvo {
            let f = (a - alvo) / (a - b);
            return Some(i as f64 + dir as f64 * f);
        }
        i = j;
    }
}

#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_faca_candidatas() {
    let b = biblioteca();
    println!(
        "{:<20} {:>5} {:>7} {:>7} {:>7} {:>8} {:>8} {:>7} {:>6} {:>6}  perfil topo (m/P, passos de 1 célula da borda 10% p/ dentro)",
        "candidata", "razão", "larg", "rampT", "rampB", "P p50", "ruído", "ruído%", "orlaT", "orlaB"
    );
    for (nome, preset, ajuste) in candidatas() {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (FW * FH * 4) as usize], FW, FH);
        t.set_paint_media(PaintMedia::WetPaint);
        t.set_wet_relogio_fixo(true);
        let p = &b
            .entradas
            .iter()
            .find(|e| e.relativo == preset)
            .unwrap_or_else(|| panic!("preset {preset} não existe"))
            .preset;
        let erros = t.wet_preset_aplicar(p);
        assert!(erros.is_empty(), "{erros:?}");
        t.set_brush_size_px(60.0);
        t.set_brush_color_srgb8(AZ);
        ajuste(&mut t);
        let n = ((FX1 - FX0) / 3.0).ceil() as u32;
        t.on_canvas_pointer(ponto([FX0, FY], PointerPhase::Down));
        for k in 1..=n {
            let x = FX0 + (FX1 - FX0) * k as f32 / n as f32;
            t.on_canvas_pointer(ponto([x, FY], PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(ponto([FX1, FY], PointerPhase::Up));
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
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        s.bring_home();
        let r = f64::from(s.ratio);
        let (gw, gh) = s.grid;
        let g = s.engine.active_grid();
        let st = g.s;
        let m = |cx: usize, cy: usize| f64::from(g.sett[cx + cy * st]) + f64::from(g.susp[cx + cy * st]);
        // Colunas do meio do traço (px 200..450), em células.
        let c0 = (200.0 / r) as usize + 1;
        let c1 = ((450.0 / r) as usize + 1).min(gw);
        let yc = (f64::from(FY) / r) as usize + 1;
        let mut larg = vec![];
        let mut miolo = vec![];
        let mut cols: Vec<Vec<f64>> = vec![];
        for cx in c0..c1 {
            let col: Vec<f64> = (1..=gh).map(|cy| m(cx, cy)).collect();
            larg.push(col.iter().filter(|&&v| v > 10.0).count() as f64 * r);
            cols.push(col);
        }
        // O miolo 60 % da faixa (pela largura mediana), centrado no eixo do traço.
        let meia = mediana(larg.clone()) / r * 0.5;
        let (lo, hi) = ((yc as f64 - 0.6 * meia).ceil() as usize, (yc as f64 + 0.6 * meia).floor() as usize);
        let mut ruido = vec![];
        for cx in c0..c1 {
            for cy in lo.max(2)..=hi.min(gh - 1) {
                miolo.push(m(cx, cy));
                let mut soma = 0.0;
                for dy in [-1i64, 0, 1] {
                    for dx in [-1i64, 0, 1] {
                        soma += m((cx as i64 + dx) as usize, (cy as i64 + dy) as usize);
                    }
                }
                ruido.push(m(cx, cy) - soma / 9.0);
            }
        }
        let plat = mediana(miolo).max(1e-9);
        // A ondulação: o desvio das médias por COLUNA (ao longo do traço) e por LINHA (através), ÷ patamar.
        let dp = |v: &[f64]| {
            let n = v.len().max(1) as f64;
            let mu = v.iter().sum::<f64>() / n;
            (v.iter().map(|x| (x - mu) * (x - mu)).sum::<f64>() / n).sqrt()
        };
        let por_col: Vec<f64> = (c0..c1)
            .map(|cx| (lo.max(2)..=hi.min(gh - 1)).map(|cy| m(cx, cy)).sum::<f64>() / (hi + 1 - lo) as f64)
            .collect();
        let por_lin: Vec<f64> = (lo.max(2)..=hi.min(gh - 1))
            .map(|cy| (c0..c1).map(|cx| m(cx, cy)).sum::<f64>() / (c1 - c0) as f64)
            .collect();
        let (ond_x, ond_y) = (100.0 * dp(&por_col) / plat, 100.0 * dp(&por_lin) / plat);
        let nr = ruido.len().max(1) as f64;
        let mu = ruido.iter().sum::<f64>() / nr;
        let sd = (ruido.iter().map(|v| (v - mu) * (v - mu)).sum::<f64>() / nr).sqrt();
        // Rampa e orla por coluna (índice da coluna = cy − 1).
        let (mut rt, mut rb, mut ot, mut ob) = (vec![], vec![], vec![], vec![]);
        let i0 = yc - 1;
        let orla_cel = (5.0 / r).ceil() as i64;
        let mut perfil_topo: Vec<Vec<f64>> = vec![];
        for col in &cols {
            for (dir, rampa, orla) in [(-1i64, &mut rt, &mut ot), (1i64, &mut rb, &mut ob)] {
                // A borda: andar do eixo para fora até cair abaixo de 10 %; o 90 % procura-se a partir daí PARA DENTRO
                // (a última vez que estava acima de 90 % antes da borda), para a orla não confundir.
                let Some(e10) = cruza(col, i0, dir, 0.1 * plat) else { continue };
                let k10 = e10.round() as i64;
                let mut e90 = None;
                let mut i = k10;
                while (i - i0 as i64) * dir > 0 {
                    let v = col[i as usize];
                    if v >= 0.9 * plat {
                        let prox = col[(i + dir) as usize];
                        e90 = Some(i as f64 + dir as f64 * ((v - 0.9 * plat) / (v - prox).max(1e-9)).clamp(0.0, 1.0));
                        break;
                    }
                    i -= dir;
                }
                if let Some(e90) = e90 {
                    rampa.push((e10 - e90).abs() * r);
                }
                let mx = (0..=orla_cel)
                    .map(|d| col[(k10 - dir * d).clamp(0, col.len() as i64 - 1) as usize])
                    .fold(0.0f64, f64::max);
                orla.push(mx / plat);
                if dir == -1 {
                    perfil_topo.push((0..12).map(|d| col[(k10 + d).clamp(0, col.len() as i64 - 1) as usize] / plat).collect());
                }
            }
        }
        let perfil: Vec<String> = (0..12)
            .map(|d| format!("{:.2}", mediana(perfil_topo.iter().map(|p| p[d]).collect())))
            .collect();
        println!(
            "{:<20} {:>5} {:>7.1} {:>7.1} {:>7.1} {:>8.1} {:>8.1} {:>7.1} {:>6.2} {:>6.2}  {}  (ondX {:.1}% ondY {:.1}%, n rampa {}/{}, água {} ticks)",
            nome,
            r,
            mediana(larg),
            mediana(rt.clone()),
            mediana(rb.clone()),
            plat,
            sd,
            100.0 * sd / plat,
            mediana(ot),
            mediana(ob),
            perfil.join(" "),
            ond_x,
            ond_y,
            rt.len(),
            cols.len(),
            ticks
        );
    }
}
