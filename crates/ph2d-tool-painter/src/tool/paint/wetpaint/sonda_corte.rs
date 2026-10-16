//! SONDA (16/10, smoke do dono): «parecem dois dabs de cada vez, mas o segundo cortado» — o início de
//! cada traço do Wet Paint com uma borda VERTICAL recta. Mede, por combinação (Shape × Shape Color ×
//! espaçamento/Continuous × pressão), a maior BORDA VERTICAL da tinta: em cada coluna `x`, quantas linhas
//! passam de não-pintado (`x − 1`) a pintado (`x`) — ou o inverso, à direita. Um disco espalha a borda por
//! muitas colunas; um corte junta-a numa só. Na tela composta e na grade do motor (`susp + sett > 0`).
//! `PH2D_SONDA_OUT=<pasta>` grava `corte_<nome>.ppm`.
//! Medido e NEGATIVO (as sondas saíram, os números ficam): o worker em tempo real, um traço curto nos dois
//! relógios (tela = composite da folha inteira), a pressão, o Smooth Edges e os 12 pincéis de fábrica no
//! espaçamento deles — nenhum corta; o corte é a janela do rasto com a CORDA > o `spacing` do motor.
use super::shape_color_tests::{arma_rampa, wet_round};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::texture::TextureKind;
use std::sync::{Arc, Mutex};

const W: u32 = 480;
const H: u32 = 220;
const YC: f32 = 110.0;
const X0: f32 = 120.0;
const X1: f32 = 400.0;

fn vermelho_amarelo() -> ColorRamp {
    ColorRamp::new(
        vec![
            RampStop::new(0.0, [1.0, 0.0, 0.0, 1.0]),
            RampStop::new(1.0, [1.0, 1.0, 0.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    )
}

#[derive(Clone, Copy)]
pub(super) struct Combo {
    pub round: bool,
    pub voronoi: bool,
    pub cor: bool,
    /// `None` = o espaçamento de fábrica; `Some((s, continuo))`.
    pub espaco: Option<(f32, bool)>,
    /// A pressão sobe de 0,05 a 1 nos primeiros `n` eventos (a caneta a pousar); `0` = rato (1,0).
    pub rampa_pressao: u32,
    pub tamanho: f32,
    /// Pixels por evento do ponteiro.
    pub passo: f32,
}

pub(super) fn monta(c: Combo) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    if c.round {
        wet_round(&mut t);
    }
    t.set_brush_size_px(c.tamanho);
    t.set_brush_color_srgb8([30, 60, 220]);
    if c.voronoi {
        t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
        t.set_brush_shape_size(0, 4.0);
        t.set_brush_shape_size(1, 4.0);
    }
    if c.cor {
        arma_rampa(&mut t, Some((true, false)));
        t.set_shape_color_ramp(vermelho_amarelo());
        t.set_shape_falloff_ramp(vermelho_amarelo());
    }
    if let Some((s, continuo)) = c.espaco {
        t.set_brush_spacing(s);
        if continuo {
            t.toggle_brush_continuous_spacing();
        }
    }
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    t
}

fn ponteiro(pos: [f32; 2], pressure: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// O traço da esquerda para a direita; devolve os dabs que o motor viu `(x, r)` em células.
pub(super) fn pinta(t: &mut PainterTool, c: Combo) -> Vec<(f64, f64)> {
    let dabs: Arc<Mutex<Vec<(f64, f64)>>> = Arc::default();
    let pr = |k: u32| {
        if c.rampa_pressao == 0 {
            1.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            (0.05 + 0.95 * (k as f32 / c.rampa_pressao as f32)).min(1.0)
        }
    };
    t.on_canvas_pointer(ponteiro([X0, YC], pr(0), PointerPhase::Down));
    if let Some(sess) = t.paint.wetpaint.session.as_mut() {
        let d2 = dabs.clone();
        sess.engine.on_dab = Some(Box::new(move |_b, d| {
            d2.lock().unwrap().push((d.x, d.r));
        }));
    }
    t.on_tick(1000.0 / 60.0);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = ((X1 - X0) / c.passo).round() as u32;
    for k in 1..=n {
        #[allow(clippy::cast_precision_loss)]
        let x = X0 + c.passo * k as f32;
        t.on_canvas_pointer(ponteiro([x, YC], pr(k), PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro([X1, YC], 1.0, PointerPhase::Up));
    dabs.lock().unwrap().clone()
}

/// A maior borda vertical: `(linhas, x)` à esquerda (entra tinta) e à direita (sai tinta), e a altura pintada.
pub(super) fn borda_vertical(
    pint: &dyn Fn(i32, i32) -> bool,
) -> ((usize, i32), (usize, i32), usize) {
    let (mut esq, mut dir) = ((0usize, -1i32), (0usize, -1i32));
    for x in 1..W as i32 {
        let (mut e, mut d) = (0, 0);
        for y in 0..H as i32 {
            let (a, b) = (pint(x - 1, y), pint(x, y));
            if !a && b {
                e += 1;
            }
            if a && !b {
                d += 1;
            }
        }
        if e > esq.0 {
            esq = (e, x);
        }
        if d > dir.0 {
            dir = (d, x);
        }
    }
    let altura = (0..H as i32)
        .filter(|&y| (0..W as i32).any(|x| pint(x, y)))
        .count();
    (esq, dir, altura)
}

pub(super) fn tela_pintada(t: &PainterTool) -> impl Fn(i32, i32) -> bool + '_ {
    move |x: i32, y: i32| {
        if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
            return false;
        }
        let p = &t.canvas_rgba[((y as u32 * W + x as u32) * 4) as usize..][..3];
        255 - p[0].min(p[1]).min(p[2]) > 25
    }
}

fn grava(t: &PainterTool, nome: &str) {
    let Ok(dir) = std::env::var("PH2D_SONDA_OUT") else {
        return;
    };
    let mut ppm = format!("P6 {W} {H} 255\n").into_bytes();
    for k in 0..(W * H) as usize {
        ppm.extend_from_slice(&t.canvas_rgba[k * 4..k * 4 + 3]);
    }
    std::fs::write(format!("{dir}/corte_{nome}.ppm"), ppm).expect("grava");
}

/// Os 12 pincéis de fábrica × {lisa, Voronoi + Texture Color} × {rato, caneta} × {tamanho 30, 60}: a razão
/// borda-vertical ÷ altura (um disco ~0,2; um corte > 0,5).
#[test]
#[ignore = "sonda: o corte vertical nos pincéis de fábrica"]
fn sonda_corte_nos_pinceis_de_fabrica() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let bib = ph2d_brush_presets::Biblioteca::carregar(&raiz, None);
    for e in &bib.entradas {
        let nome_p = e.preset.parametros.name.clone();
        for (fnome, voronoi, cor) in [("lisa", false, false), ("vorocor", true, true)] {
            for rampa_pressao in [0u32, 6] {
                for (tamanho, passo) in [(30.0f32, 4.0f32), (60.0, 12.0)] {
                    let c = Combo {
                        round: false,
                        voronoi,
                        cor,
                        espaco: None,
                        rampa_pressao,
                        tamanho,
                        passo,
                    };
                    let mut t = PainterTool::default();
                    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
                    t.set_paint_media(PaintMedia::WetPaint);
                    t.set_wet_relogio_fixo(true);
                    assert!(t.wet_preset_aplicar(&e.preset).is_empty());
                    t.wet_preset_aplicar_volumes(&e.preset.parametros.volumes);
                    t.set_brush_size_px(tamanho);
                    if voronoi {
                        t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
                        t.set_brush_shape_size(0, 4.0);
                        t.set_brush_shape_size(1, 4.0);
                    }
                    if cor {
                        arma_rampa(&mut t, Some((true, false)));
                        t.set_shape_color_ramp(vermelho_amarelo());
                        t.set_shape_falloff_ramp(vermelho_amarelo());
                    }
                    let seed = t.paint.brush;
                    t.paint.brush_by_mode.fill(seed);
                    let dabs = pinta(&mut t, c);
                    let (te, td, th) = borda_vertical(&tela_pintada(&t));
                    let nome = format!(
                        "{}_{fnome}_p{rampa_pressao}_s{tamanho}",
                        nome_p.replace(' ', "")
                    );
                    grava(&t, &nome);
                    #[allow(clippy::cast_precision_loss)]
                    let q = te.0 as f64 / th.max(1) as f64;
                    eprintln!(
                        "{nome:>36}: esq {:>3}@{:<3} ({q:.2}) dir {:>3}@{:<3} alt {th:>3} · {} dabs · smooth {:.2} tilt {} · 1.ºs {:?}",
                        te.0,
                        te.1,
                        td.0,
                        td.1,
                        dabs.len(),
                        t.paint.wetpaint.smooth_edges,
                        t.paint.wetpaint.tilt_on,
                        dabs.iter()
                            .take(4)
                            .map(|(x, r)| format!("{x:.0}/{r:.0}"))
                            .collect::<Vec<_>>()
                    );
                }
            }
        }
    }
}

/// A janela do rasto refeita sobre os dabs que o motor viu (a lei do `trail.rs`: a âncora no 1.º dab da
/// janela, `half = max(⌈r + 4·sp⌉ + 2, 61)` só a crescer, `window_size = ⌊corda ÷ sp⌋ ≤ 2`, o dab que
/// nasce a lane pousa sozinho): `(dabs cortados à esquerda, à direita, o maior corte em células)`.
fn janela_refeita(dabs: &[(f64, f64)], sp: f64) -> (usize, usize, f64) {
    let (mut esq, mut dir, mut maior) = (0, 0, 0.0f64);
    let (mut half, mut anchor, mut count, mut wsize) = (61i32, 0i32, 0u32, 0u32);
    let mut prev: Option<f64> = None;
    for &(x, r) in dabs {
        if let Some(p) = prev {
            wsize = ((x - p).abs() / sp).floor().min(2.0) as u32;
        }
        prev = Some(x);
        if count == 0 {
            anchor = x.round() as i32;
            half = half.max(((r + 4.0 * sp).ceil() as i32 + 2).max(61));
        }
        let lo = f64::from(anchor - half);
        let hi = f64::from(anchor + half);
        if x - r < lo {
            esq += 1;
            maior = maior.max(lo - (x - r));
        }
        if x + r > hi {
            dir += 1;
            maior = maior.max(x + r - hi);
        }
        count += 1;
        if count > wsize {
            count = 0;
        }
    }
    (esq, dir, maior)
}

/// O espaçamento do pincel × o knob escondido `Spacing` do motor × o Smooth Edges, Wet Round, raio 60.
#[test]
#[ignore = "sonda: o corte vertical × espaçamentos e Smooth Edges"]
fn sonda_corte_espacamentos() {
    use ph2d_wet_paint::tuning::Knob;
    for (fnome, voronoi, cor) in [("lisa", false, false), ("vorocor", true, true)] {
        for host_sp in [0.05f32, 0.15, 0.3] {
            for knob_sp in [0.5f64, 2.0, 8.0] {
                for suave in [0.0f32, 1.0] {
                    let c = Combo {
                        round: true,
                        voronoi,
                        cor,
                        espaco: Some((host_sp, false)),
                        rampa_pressao: 0,
                        tamanho: 60.0,
                        passo: 12.0,
                    };
                    let mut t = monta(c);
                    t.paint.wetpaint.knobs.set(Knob::Spacing, knob_sp);
                    t.paint.wetpaint.smooth_edges = suave;
                    let dabs = pinta(&mut t, c);
                    for _ in 0..30 {
                        t.on_tick(1000.0 / 60.0);
                    }
                    let (te, td, th) = borda_vertical(&tela_pintada(&t));
                    let (je, jd, jm) = janela_refeita(&dabs, knob_sp);
                    let nome = format!("esp_{fnome}_h{host_sp}_k{knob_sp}_s{suave}");
                    grava(&t, &nome);
                    #[allow(clippy::cast_precision_loss)]
                    let (qe, qd) = (
                        te.0 as f64 / th.max(1) as f64,
                        td.0 as f64 / th.max(1) as f64,
                    );
                    let corda = dabs
                        .windows(2)
                        .map(|w| (w[1].0 - w[0].0).abs())
                        .fold(0.0f64, f64::max);
                    eprintln!(
                        "{nome:>32}: tela esq {:>3}@{:<3} ({qe:.2}) dir {:>3}@{:<3} ({qd:.2}) alt {th:>3} · {} dabs corda máx {corda:.1} · janela refeita: cortados esq {je} dir {jd} maior {jm:.1} cél",
                        te.0,
                        te.1,
                        td.0,
                        td.1,
                        dabs.len()
                    );
                }
            }
        }
    }
}

/// A ABLAÇÃO sem tocar no produto: um traço de PRIMEIRA com um pincel enorme FORA da tela cresce a janela
/// do rasto (o `fit_to` só cresce, e a janela vive entre traços), e o mesmo traço a seguir já não pode ser
/// cortado. A × B no mesmo processo: a tinta, as bordas verticais e os píxeis que mudam.
#[test]
#[ignore = "sonda: a ablação da janela do rasto"]
fn sonda_corte_ablacao_da_janela() {
    for (fnome, voronoi, cor) in [("lisa", false, false), ("vorocor", true, true)] {
        for host_sp in [0.05f32, 0.1, 0.15, 0.3] {
            for tamanho in [30.0f32, 60.0, 120.0] {
                let mut r = Vec::new();
                for larga in [false, true] {
                    let c = Combo {
                        round: true,
                        voronoi,
                        cor,
                        espaco: Some((host_sp, false)),
                        rampa_pressao: 0,
                        tamanho,
                        passo: 12.0,
                    };
                    let mut t = monta(c);
                    if larga {
                        t.set_brush_size_px(400.0);
                        let seed = t.paint.brush;
                        t.paint.brush_by_mode.fill(seed);
                        t.on_canvas_pointer(ponteiro([-3000.0, -3000.0], 1.0, PointerPhase::Down));
                        t.on_canvas_pointer(ponteiro([-2900.0, -3000.0], 1.0, PointerPhase::Move));
                        t.on_canvas_pointer(ponteiro([-2800.0, -3000.0], 1.0, PointerPhase::Move));
                        t.on_canvas_pointer(ponteiro([-2800.0, -3000.0], 1.0, PointerPhase::Up));
                        t.on_tick(1000.0 / 60.0);
                        t.set_brush_size_px(tamanho);
                        let seed = t.paint.brush;
                        t.paint.brush_by_mode.fill(seed);
                    }
                    let antes = super::shape_color_tests::impressao(&t.canvas_rgba);
                    let dabs = pinta(&mut t, c);
                    let half = {
                        let sess = t.paint.wetpaint.session.as_ref().unwrap();
                        sess.engine
                            .lane_trails
                            .iter()
                            .map(|tr| tr.window_half_for_measure())
                            .max()
                    };
                    let (te, td, th) = borda_vertical(&tela_pintada(&t));
                    let (je, jd, jm) = janela_refeita(&dabs, 2.0);
                    grava(&t, &format!("abl_{fnome}_h{host_sp}_s{tamanho}_{larga}"));
                    r.push((
                        t.canvas_rgba.clone(),
                        antes,
                        half,
                        te,
                        td,
                        th,
                        (je, jd, jm),
                        dabs.first().map(|d| d.1),
                    ));
                }
                let (a, b) = (&r[0], &r[1]);
                let muda =
                    a.0.chunks(4)
                        .zip(b.0.chunks(4))
                        .filter(|(p, q)| p != q)
                        .count();
                let tinta = |px: &[u8]| -> u64 { super::shape_color_tests::tinta(px) };
                eprintln!(
                    "{fnome:>7} h{host_sp:<4} s{tamanho:<3} r {:.0}: tela branca antes do B {} · half A {:?} B {:?} · janela refeita A esq/dir/maior {:?} · tinta A {} B {} (B/A {:.3}) · dir A {}@{} ({:.2}) B {}@{} ({:.2}) · esq A {}@{} B {}@{} · píxeis que mudam {muda}",
                    a.7.unwrap_or(0.0),
                    b.1 == a.1,
                    a.2,
                    b.2,
                    a.6,
                    tinta(&a.0),
                    tinta(&b.0),
                    tinta(&b.0) as f64 / tinta(&a.0).max(1) as f64,
                    a.4.0,
                    a.4.1,
                    a.4.0 as f64 / a.5.max(1) as f64,
                    b.4.0,
                    b.4.1,
                    b.4.0 as f64 / b.5.max(1) as f64,
                    a.3.0,
                    a.3.1,
                    b.3.0,
                    b.3.1,
                );
            }
        }
    }
}

/// O pincel do DONO (16/10): os dabs que o motor vê e a janela refeita sobre eles, × Continuous e Jitter.
#[test]
#[ignore = "sonda: o pincel do dono"]
fn sonda_corte_pincel_do_dono() {
    use super::janela_do_rasto_tests::{semeia, tela, traco};
    for continuo in [true, false] {
        for jitter in [0.1f32, 0.0] {
            for passo in [24.0f32, 6.0, 60.0] {
                let mut t = tela(1400, 720, 169.0, (true, true));
                t.set_brush_spacing(0.364);
                if continuo {
                    t.toggle_brush_continuous_spacing();
                }
                t.set_brush_jitter_norm(jitter);
                t.paint.wetpaint.knobs.water = 1.0;
                semeia(&mut t);
                let dabs: Arc<Mutex<Vec<(f64, f64)>>> = Arc::default();
                assert!(t.ensure_wet_session());
                {
                    let sess = t.paint.wetpaint.session.as_mut().unwrap();
                    sess.bring_home();
                    let d2 = dabs.clone();
                    sess.engine.on_dab = Some(Box::new(move |_b, d| {
                        d2.lock().unwrap().push((d.x, d.r));
                    }));
                }
                traco(&mut t, 360.0, 380.0, 1020.0, passo);
                let v = dabs.lock().unwrap().clone();
                let corda: Vec<f64> = v.windows(2).map(|w| w[1].0 - w[0].0).collect();
                let (je, jd, jm) = janela_refeita(&v, 2.0);
                eprintln!(
                    "cont {continuo} jitter {jitter} passo {passo}: {} dabs r {:.0} · corda min {:.1} máx {:.1} · cortados esq {je} dir {jd} maior {jm:.1} · 1.ºs {:?}",
                    v.len(),
                    v.first().map_or(0.0, |d| d.1),
                    corda.iter().copied().fold(f64::MAX, f64::min),
                    corda.iter().copied().fold(f64::MIN, f64::max),
                    v.iter()
                        .take(6)
                        .map(|d| format!("{:.0}", d.0))
                        .collect::<Vec<_>>()
                );
            }
        }
    }
}

/// O pincel do DONO com a CANETA a pousar: a pressão sobe de 0,05 a 1 nos primeiros `n` eventos. Os dabs,
/// a janela refeita, e a tela contra o oráculo (as janelas crescidas).
#[test]
#[ignore = "sonda: o pincel do dono com a pressão"]
fn sonda_corte_pincel_do_dono_com_pressao() {
    use super::janela_do_rasto_tests::{borda_vertical, cresce_a_janela, semeia, tela};
    for rampa in [0u32, 3, 8] {
        for passo in [6.0f32, 24.0] {
            let mut res = Vec::new();
            for crescida in [false, true] {
                let mut t = tela(1400, 720, 169.0, (true, true));
                t.set_brush_spacing(0.364);
                t.toggle_brush_continuous_spacing();
                t.set_brush_jitter_norm(0.1);
                t.paint.wetpaint.knobs.water = 1.0;
                semeia(&mut t);
                if crescida {
                    cresce_a_janela(&mut t);
                }
                let dabs: Arc<Mutex<Vec<(f64, f64)>>> = Arc::default();
                assert!(t.ensure_wet_session());
                {
                    let sess = t.paint.wetpaint.session.as_mut().unwrap();
                    sess.bring_home();
                    let d2 = dabs.clone();
                    sess.engine.on_dab = Some(Box::new(move |_b, d| {
                        d2.lock().unwrap().push((d.x, d.r));
                    }));
                }
                let pr = |k: u32| {
                    if rampa == 0 {
                        1.0
                    } else {
                        #[allow(clippy::cast_precision_loss)]
                        (0.05 + 0.95 * (k as f32 / rampa as f32)).min(1.0)
                    }
                };
                let (de, ate, y) = (380.0f32, 1020.0f32, 360.0f32);
                t.on_canvas_pointer(ponteiro([de, y], pr(0), PointerPhase::Down));
                t.on_tick(1000.0 / 60.0);
                // A caneta POUSA: nos `rampa` primeiros eventos anda 1 px enquanto a pressão sobe.
                let mut x = de;
                let mut k = 0u32;
                while x < ate {
                    k += 1;
                    x += if k <= rampa { 1.0 } else { passo };
                    t.on_canvas_pointer(ponteiro([x.min(ate), y], pr(k), PointerPhase::Move));
                    t.on_tick(1000.0 / 60.0);
                }
                t.on_canvas_pointer(ponteiro([ate, y], 1.0, PointerPhase::Up));
                let v = dabs.lock().unwrap().clone();
                let borda = borda_vertical(&t.canvas_rgba, 1400, 720);
                let tinta = super::shape_color_tests::tinta(&t.canvas_rgba) as f64;
                if let Ok(dir) = std::env::var("PH2D_SONDA_OUT") {
                    let mut ppm = b"P6 1400 720 255\n".to_vec();
                    for k in 0..1400 * 720 {
                        ppm.extend_from_slice(&t.canvas_rgba[k * 4..k * 4 + 3]);
                    }
                    std::fs::write(format!("{dir}/dono_p{rampa}_{passo}_{crescida}.ppm"), ppm)
                        .unwrap();
                }
                res.push((v, borda, tinta));
            }
            let (je, jd, jm) = janela_refeita(&res[0].0, 2.0);
            eprintln!(
                "rampa {rampa} passo {passo}: {} dabs · r {:?} · janela refeita: esq {je} dir {jd} maior {jm:.1} · borda {:.2} (oráculo {:.2}) · tinta {:.3} do oráculo",
                res[0].0.len(),
                res[0]
                    .0
                    .iter()
                    .take(8)
                    .map(|d| format!("{:.0}@{:.0}", d.1, d.0))
                    .collect::<Vec<_>>(),
                res[0].1,
                res[1].1,
                res[0].2 / res[1].2
            );
        }
    }
}
