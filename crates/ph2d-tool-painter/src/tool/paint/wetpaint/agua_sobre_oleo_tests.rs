//! **A água sobre o óleo** (smoke do dono 30/10): o pincel molhado pintado sobre óleo com papel de COR.

use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase};

pub(super) const W: usize = 240;
pub(super) const H: usize = 160;
pub(super) const PAPEL: [u8; 3] = [150, 130, 100];

pub(super) fn preset(rel: &str) -> ph2d_brush_presets::Preset {
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/brushes/WetPainter")
        .join(rel);
    ph2d_brush_presets::ler(&std::fs::read(&p).expect("o preset")).expect("o preset lê-se")
}

pub(super) fn ponteiro(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A tela: papel de cor, um traço de óleo (Dabber) em `y = 90`, assentado.
pub(super) fn com_oleo() -> PainterTool {
    com_oleo_em(false)
}

/// [`com_oleo`] com o traço de óleo na horizontal (`y = 90`) ou na VERTICAL (`x = 120`, de `y = 20` a `140`).
pub(super) fn com_oleo_em(vertical: bool) -> PainterTool {
    com_oleo_sobre(vertical, PAPEL)
}

/// [`com_oleo_em`] sobre o papel `papel` (o branco: a cena do Rebelle, que não recebe a cor do papel pela porta).
pub(super) fn com_oleo_sobre(vertical: bool, papel: [u8; 3]) -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", papel[0], papel[1], papel[2]),
    ));
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o meio");
    t.set_brush_size_px(30.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    let ponto = |k: usize| {
        if vertical {
            (120.0, 20.0 + 3.0 * k as f32)
        } else {
            (40.0 + 4.0 * k as f32, 90.0)
        }
    };
    let (x, y) = ponto(0);
    t.on_canvas_pointer(ponteiro(x, y, PointerPhase::Down));
    for k in 1..=40 {
        let (x, y) = ponto(k);
        t.on_canvas_pointer(ponteiro(x, y, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    let (x, y) = ponto(40);
    t.on_canvas_pointer(ponteiro(x, y, PointerPhase::Up));
    for _ in 0..120 {
        t.on_tick(1000.0 / 60.0);
    }
    t
}

/// A imagem MOSTRADA (camadas sobre o papel, com a luz).
pub(super) fn mostrada(t: &mut PainterTool) -> Vec<u8> {
    t.invalidate_composite();
    let (px, _, _) = t.take_preview_arc().expect("a imagem mostrada");
    px.to_vec()
}

fn grava(nome: &str, px: &[u8]) {
    let dir = std::env::var("PH2D_SONDA_DIR").unwrap_or_default();
    if dir.is_empty() {
        return;
    }
    let mut v = format!("P6\n{W} {H}\n255\n").into_bytes();
    for p in px.chunks(4) {
        v.extend_from_slice(&p[..3]);
    }
    std::fs::write(format!("{dir}/{nome}.ppm"), v).unwrap();
}

pub(super) fn relato(l: String) {
    eprintln!("{l}");
    let dir = std::env::var("PH2D_SONDA_DIR").unwrap_or_default();
    if !dir.is_empty() {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(format!("{dir}/relato.txt"))
            .unwrap();
        writeln!(f, "{l}").unwrap();
    }
}

fn despeja(t: &mut PainterTool, x: usize, y: usize, mostrada: &[u8]) {
    t.wet_bring_home();
    let camada = t.canvas_rgba[(y * W + x) * 4..(y * W + x) * 4 + 4].to_vec();
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
    let r = usize::from(s.ratio);
    let (gw, _) = s.grid;
    let (cx, cy) = (x / r + 1, y / r + 1);
    let pg = (cy - 1) * gw * 4 + (cx - 1) * 4;
    let base = s.base[(y * W + x) * 4..(y * W + x) * 4 + 4].to_vec();
    relato(format!(
        "({x},{y}) razão {r} célula ({cx},{cy}): mostrada {:?} camada {:?} base {:?} pigmento {:?} sombra {}",
        &mostrada[(y * W + x) * 4..(y * W + x) * 4 + 4],
        camada,
        base,
        &s.pigment[pg..pg + 4],
        s.sombra.get((cy - 1) * gw + cx - 1).copied().unwrap_or(0)
    ));
    {
        let layers: Vec<ph2d_wet_paint::render::RenderLayer<'_>> = s
            .engine
            .layers
            .iter()
            .map(|l| ph2d_wet_paint::render::RenderLayer {
                grid: &l.grid,
                opacity: l.opacity,
                visible: l.visible,
            })
            .collect();
        let params = s.engine.sim.gather_params(&s.engine.tuning);
        let mut row = vec![0u8; gw * 4];
        ph2d_wet_paint::render::render_pigment_row_visual(
            Some(&params),
            &layers,
            ph2d_wet_paint::render::PigmentVisual {
                paper: true,
                relevo: true,
            },
            cy,
            cx,
            cx,
            &mut row,
        );
        relato(format!(
            "   sem a óptica do óleo: {:?}",
            &row[(cx - 1) * 4..cx * 4]
        ));
    }
    for (k, l) in s.engine.layers.iter().enumerate() {
        let g = &l.grid;
        let i = cx + cy * g.s;
        relato(format!(
            "   camada {k} vis {} op {}: sett {:.0} {:?} susp {:.0} {:?} film {:.3} wet {} óleo {:?}",
            l.visible,
            l.opacity,
            g.sett[i],
            g.sett_rgb[i],
            g.susp[i],
            g.susp_rgb[i],
            g.film[i],
            g.wet.get(i).copied().unwrap_or(0),
            g.oleo.get(i).map(|o| o.oleo)
        ));
    }
}

fn lum(p: &[u8]) -> f32 {
    0.2126 * f32::from(p[0]) + 0.7152 * f32::from(p[1]) + 0.0722 * f32::from(p[2])
}

/// O Wet Flat a meio do traço sobre o óleo de [`com_oleo`] (`k` passos do rato, sem soltar).
pub(super) fn molhado_a_meio(t: &mut PainterTool, k: usize) {
    t.preset_aplicar(&preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png"))
        .expect("o meio");
    t.set_brush_size_px(30.0);
    t.set_brush_color_srgb8([240, 130, 60]);
    t.on_canvas_pointer(ponteiro(20.0, 70.0, PointerPhase::Down));
    for j in 1..=k {
        t.on_canvas_pointer(ponteiro(20.0 + 4.0 * j as f32, 70.0, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
}

/// ⭐ **O molhado sobre tinta fina não clareia além das tintas** (smoke do dono 30/10: manchas brancas do Wet Flat
/// sobre a orla do óleo, num papel de COR, que somem ao assentar). Mecanismo: a lei `Cobre` decide o ponto sobre o
/// BRANCO e a cor que saía para a tela passava da mais clara das duas tintas (`[239,194,168]` de azul fino sob
/// laranja); só o papel branco escondia o branco embutido. A meio do traço, cada célula com tinta assentada E água
/// tem a cor da camada (sem a óptica do óleo) dentro da caixa das duas tintas, canal a canal. Medido antes da cura:
/// 476 píxeis da imagem a meio mais claros que o óleo, o papel e o fim (+20); depois, 65 (os reflexos do óleo).
#[test]
fn o_molhado_sobre_tinta_fina_nao_clareia_alem_das_tintas() {
    let mut t = com_oleo();
    molhado_a_meio(&mut t, 30);
    t.wet_bring_home();
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
    let (gw, gh) = s.grid;
    let layers: Vec<ph2d_wet_paint::render::RenderLayer<'_>> = s
        .engine
        .layers
        .iter()
        .map(|l| ph2d_wet_paint::render::RenderLayer {
            grid: &l.grid,
            opacity: l.opacity,
            visible: l.visible,
        })
        .collect();
    assert_eq!(layers.len(), 1, "a cena é de uma camada do motor");
    let params = s.engine.sim.gather_params(&s.engine.tuning);
    assert!(
        matches!(params.ecra, ph2d_wet_paint::assenta::Ecra::Cobre { .. }),
        "o Wet Flat pinta pela lei Cobre"
    );
    let g = layers[0].grid;
    let (mut mistas, mut fora, mut pior) = (0, 0, (0.0f32, 0, 0));
    let mut row = vec![0u8; gw * 4];
    for cy in 1..=gh {
        ph2d_wet_paint::render::render_pigment_row_visual(
            Some(&params),
            &layers,
            ph2d_wet_paint::render::PigmentVisual {
                paper: true,
                relevo: false,
            },
            cy,
            1,
            gw,
            &mut row,
        );
        for cx in 1..=gw {
            let i = cx + cy * g.s;
            if g.sett[i] <= 0.0 || g.susp[i] <= 0.0 {
                continue;
            }
            mistas += 1;
            let px = &row[(cx - 1) * 4..cx * 4];
            for (k, &v) in px.iter().take(3).enumerate() {
                let excesso = f32::from(v) - g.sett_rgb[i][k].max(g.susp_rgb[i][k]);
                if excesso > 1.0 {
                    fora += 1;
                    if excesso > pior.0 {
                        pior = (excesso, cx, cy);
                    }
                    break;
                }
            }
        }
    }
    assert!(
        mistas > 500,
        "a cena tem só {mistas} células assentado + água"
    );
    assert_eq!(
        fora,
        0,
        "{fora} de {mistas} células com a cor mais clara que as duas tintas (pior +{:.0} em {:?})",
        pior.0,
        (pior.1, pior.2)
    );
}

/// SONDA — o Wet Flat sobre o óleo: a imagem a meio do traço contra a assentada.
#[test]
#[ignore = "diagnóstico"]
fn diag_a_agua_sobre_o_oleo_a_meio_do_traco() {
    let mut t = com_oleo();
    let oleo = mostrada(&mut t);
    grava("0_oleo", &oleo);
    t.preset_aplicar(&preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png"))
        .expect("o meio");
    t.set_brush_size_px(30.0);
    t.set_brush_color_srgb8([240, 130, 60]);
    t.on_canvas_pointer(ponteiro(20.0, 70.0, PointerPhase::Down));
    let mut meio = Vec::new();
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(20.0 + 4.0 * k as f32, 70.0, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
        if k == 30 {
            meio = mostrada(&mut t);
            for (x, y) in [(82usize, 97usize), (130, 57)] {
                despeja(&mut t, x, y, &meio);
            }
        }
    }
    let antes_do_soltar = mostrada(&mut t);
    t.on_canvas_pointer(ponteiro(180.0, 70.0, PointerPhase::Up));
    for _ in 0..600 {
        t.on_tick(1000.0 / 60.0);
    }
    let fim = mostrada(&mut t);
    grava("1_meio", &meio);
    grava("2_antes_do_soltar", &antes_do_soltar);
    grava("3_fim", &fim);
    let papel = lum(&[PAPEL[0], PAPEL[1], PAPEL[2]]);
    for (nome, img) in [("meio", &meio), ("antes do soltar", &antes_do_soltar)] {
        let mut n = 0;
        let mut pior = (0.0f32, 0, 0);
        for i in 0..W * H {
            let m = lum(&img[i * 4..i * 4 + 4]);
            let teto = lum(&fim[i * 4..i * 4 + 4])
                .max(lum(&oleo[i * 4..i * 4 + 4]))
                .max(papel);
            if m > teto + 20.0 {
                n += 1;
                if m - teto > pior.0 {
                    pior = (m - teto, i % W, i / W);
                }
            }
        }
        let (x, y) = (pior.1, pior.2);
        let i = (y * W + x) * 4;
        relato(format!(
            "{nome}: {n} píxeis mais claros que o fim/óleo/papel (+20); pior +{:.0} em ({x},{y}): {:?} fim {:?} óleo {:?}",
            pior.0,
            &img[i..i + 4],
            &fim[i..i + 4],
            &oleo[i..i + 4]
        ));
    }
}

/// SONDA — as Grain Colors (azul → amarelo, Clouds) no Wet Flat sobre o papel de cor (dono 30/10: «a pintura também
/// fica esbranquiçada»): dois traços que se cruzam, a imagem a meio do 2.º e assentada.
#[test]
#[ignore = "diagnóstico"]
fn diag_grain_colors_no_papel_de_cor() {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", PAPEL[0], PAPEL[1], PAPEL[2]),
    ));
    t.preset_aplicar(&preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png"))
        .expect("o meio");
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([40, 120, 230]);
    t.set_brush_texture_kind(ph2d_painter_brush::TextureKind::Clouds.to_u8());
    t.set_texture_ramp(ph2d_color::ColorRamp::new(
        vec![
            // As cores do dono (sRGB [40,120,230] e [250,220,40]) em LINEAR, como o seletor as guarda.
            ph2d_color::RampStop::new(0.0, [0.021, 0.188, 0.791, 1.0]),
            ph2d_color::RampStop::new(0.304, [0.955, 0.716, 0.021, 1.0]),
        ],
        ph2d_color::RampColorMode::Rgb,
        ph2d_color::RampInterp::Linear,
    ));
    t.set_texture_ramp_enabled(std::env::var("PH2D_SONDA_SEM_RAMPA").is_err());
    if let Some(r) = t.rampa_da_aguada() {
        for g in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
            relato(format!(
                "rampa no grão {g}: {:?}",
                r.cor(None, Some(g), 1.0)
            ));
        }
    }
    let traco = |t: &mut PainterTool, y0: f32, y1: f32, ate: usize| {
        t.on_canvas_pointer(ponteiro(30.0, y0, PointerPhase::Down));
        for k in 1..=ate {
            let f = k as f32 / 40.0;
            t.on_canvas_pointer(ponteiro(
                30.0 + 180.0 * f,
                y0 + (y1 - y0) * f,
                PointerPhase::Move,
            ));
            t.on_tick(1000.0 / 60.0);
        }
    };
    traco(&mut t, 60.0, 100.0, 40);
    t.on_canvas_pointer(ponteiro(210.0, 100.0, PointerPhase::Up));
    for _ in 0..60 {
        t.on_tick(1000.0 / 60.0);
    }
    traco(&mut t, 100.0, 60.0, 30);
    let meio = mostrada(&mut t);
    {
        t.wet_bring_home();
        let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
        let g = &s.engine.layers[0].grid;
        for (x, y) in [(60usize, 75usize), (100, 80), (140, 85), (180, 80)] {
            let i = (x + 1) + (y + 1) * g.s;
            relato(format!(
                "({x},{y}) susp {:.0} {:?} sett {:.0} {:?} camada {:?}",
                g.susp[i],
                g.susp_rgb[i],
                g.sett[i],
                g.sett_rgb[i],
                &t.canvas_rgba[(y * W + x) * 4..(y * W + x) * 4 + 4]
            ));
        }
    }
    t.on_canvas_pointer(ponteiro(165.0, 70.0, PointerPhase::Up));
    for _ in 0..600 {
        t.on_tick(1000.0 / 60.0);
    }
    let fim = mostrada(&mut t);
    grava("g_meio", &meio);
    grava("g_fim", &fim);
    let papel = lum(&[PAPEL[0], PAPEL[1], PAPEL[2]]);
    let claros = |img: &[u8]| {
        (0..W * H)
            .filter(|&i| lum(&img[i * 4..i * 4 + 4]) > papel + 40.0)
            .count()
    };
    relato(format!(
        "grain colors: píxeis mais claros que o papel (+40) — a meio {} · assentado {}",
        claros(&meio),
        claros(&fim)
    ));
}

/// O que a água pousou DENTRO do óleo: com o óleo de [`com_oleo`], um traço aguado só no papel acima da parede (o pé a
/// `folga` px da orla). Devolve `(massa nova dentro, células dentro com > 50 de massa nova, células do miolo)`; o
/// miolo = células com óleo (massa > 1000) a ≥ 4 px da orla de cima da sua coluna.
pub(super) fn agua_que_sobe(folga: f32) -> (f64, usize, usize) {
    let mut t = com_oleo();
    t.wet_bring_home();
    let antes: Vec<f32> = {
        let g = &t
            .paint
            .wetpaint
            .session
            .as_ref()
            .expect("a sessão")
            .engine
            .layers[0]
            .grid;
        (0..g.cells).map(|i| g.sett[i] + g.susp[i]).collect()
    };
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
    let gs = s.engine.layers[0].grid.s;
    let (gw, gh) = s.grid;
    // A orla de cima de cada coluna (a 1.ª célula com massa > 1000) e o topo do óleo no meio.
    let orla: Vec<Option<usize>> = (1..=gw)
        .map(|cx| (1..=gh).find(|&cy| antes[cx + cy * gs] > 1000.0))
        .collect();
    let topo = orla[120].expect("o óleo no meio") as f32 - 1.0;
    t.preset_aplicar(&preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png"))
        .expect("o meio");
    t.set_brush_size_px(20.0);
    t.set_brush_color_srgb8([240, 130, 60]);
    let y = topo - 10.0 - folga;
    t.on_canvas_pointer(ponteiro(40.0, y, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(40.0 + 4.0 * k as f32, y, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro(200.0, y, PointerPhase::Up));
    for _ in 0..600 {
        t.on_tick(1000.0 / 60.0);
    }
    t.wet_bring_home();
    let g = &t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessão")
        .engine
        .layers[0]
        .grid;
    let (mut massa, mut celulas, mut miolo) = (0.0f64, 0, 0);
    for (k, o) in orla.iter().enumerate() {
        let Some(o) = *o else { continue };
        let cx = k + 1;
        for cy in o + 4..=gh {
            let i = cx + cy * gs;
            if antes[i] <= 1000.0 {
                continue;
            }
            miolo += 1;
            let nova = f64::from(g.sett[i] + g.susp[i] - antes[i]).max(0.0);
            massa += nova;
            if nova > 50.0 {
                celulas += 1;
            }
        }
    }
    (massa, celulas, miolo)
}

/// SONDA — a água sobe a ladeira do óleo? (Bug 2 do smoke do dono 30/10.)
#[test]
#[ignore = "diagnóstico"]
fn diag_a_agua_sobe_a_ladeira_do_oleo() {
    for folga in [0.0f32, 4.0] {
        let (m, c, n) = agua_que_sobe(folga);
        relato(format!(
            "folga {folga} px: massa nova no miolo do óleo {m:.0} · {c} de {n} células com > 50"
        ));
    }
}

/// ⭐ **A água não sobe a ladeira do óleo** (Bug 2 do smoke do dono 30/10): um traço aguado (Wet Flat) pintado só no
/// papel, ao pé de um Oil ▸ Dabber assentado, num papel de cor. Mecanismo: o nivelamento corria só pela altura da
/// água e a do óleo não entrava em passe nenhum (`solver::ladeira_do_oleo`). Medido antes da cura, massa nova no
/// miolo do óleo (≥ 4 px da orla): encostado 1 003 372 (4 341 células > 50), a 4 px 685 617 (4 014); depois 40 511
/// (116, onde o próprio pincel pousa) e 3 468 (10).
#[test]
fn a_agua_nao_sobe_a_ladeira_do_oleo() {
    let (encostado, celulas, miolo) = agua_que_sobe(0.0);
    assert!(miolo > 5000, "o miolo do óleo tem só {miolo} células");
    assert!(
        encostado < 100_000.0 && celulas < 400,
        "encostado à parede: {encostado:.0} de massa nova no miolo do óleo, {celulas} células > 50"
    );
    let (folga, celulas, _) = agua_que_sobe(4.0);
    assert!(
        folga < 34_000.0 && celulas < 50,
        "a 4 px da parede: {folga:.0} de massa nova no miolo do óleo, {celulas} células > 50"
    );
    // O eixo `x` (a parede de LADO; a de cima só prova o `y` — a prova de mutação deixou o `x` vivo): a 4 px, sem a
    // ladeira 588 227, sem o eixo `x` 400 279, com ela 4 975.
    let (lado, celulas, _) = agua_que_sobe_de_lado(4.0);
    assert!(
        lado < 40_000.0 && celulas < 100,
        "de lado, a 4 px da parede: {lado:.0} de massa nova no miolo do óleo, {celulas} células > 50"
    );
}

/// [`agua_que_sobe`] DE LADO: o óleo vertical de [`com_oleo_em`] e o traço aguado vertical à esquerda da parede (o pé a
/// `folga` px da orla); o miolo = células com óleo a ≥ 4 px da orla da esquerda da sua linha. Prova o eixo `x` da ladeira.
pub(super) fn agua_que_sobe_de_lado(folga: f32) -> (f64, usize, usize) {
    let mut t = com_oleo_em(true);
    t.wet_bring_home();
    let antes: Vec<f32> = {
        let g = &t
            .paint
            .wetpaint
            .session
            .as_ref()
            .expect("a sessão")
            .engine
            .layers[0]
            .grid;
        (0..g.cells).map(|i| g.sett[i] + g.susp[i]).collect()
    };
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
    let gs = s.engine.layers[0].grid.s;
    let (gw, gh) = s.grid;
    let orla: Vec<Option<usize>> = (1..=gh)
        .map(|cy| (1..=gw).find(|&cx| antes[cx + cy * gs] > 1000.0))
        .collect();
    let esquerda = orla[80].expect("o óleo no meio") as f32 - 1.0;
    t.preset_aplicar(&preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png"))
        .expect("o meio");
    t.set_brush_size_px(20.0);
    t.set_brush_color_srgb8([240, 130, 60]);
    let x = esquerda - 10.0 - folga;
    t.on_canvas_pointer(ponteiro(x, 30.0, PointerPhase::Down));
    for k in 1..=30 {
        t.on_canvas_pointer(ponteiro(x, 30.0 + 3.0 * k as f32, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro(x, 120.0, PointerPhase::Up));
    for _ in 0..600 {
        t.on_tick(1000.0 / 60.0);
    }
    t.wet_bring_home();
    let g = &t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessão")
        .engine
        .layers[0]
        .grid;
    let (mut massa, mut celulas, mut miolo) = (0.0f64, 0, 0);
    for (k, o) in orla.iter().enumerate() {
        let Some(o) = *o else { continue };
        let cy = k + 1;
        for cx in o + 4..=gw {
            let i = cx + cy * gs;
            if antes[i] <= 1000.0 {
                continue;
            }
            miolo += 1;
            let nova = f64::from(g.sett[i] + g.susp[i] - antes[i]).max(0.0);
            massa += nova;
            if nova > 50.0 {
                celulas += 1;
            }
        }
    }
    (massa, celulas, miolo)
}

/// SONDA — o eixo `x` da ladeira (a parede de lado).
#[test]
#[ignore = "diagnóstico"]
fn diag_a_agua_sobe_a_ladeira_de_lado() {
    for folga in [0.0f32, 4.0] {
        let (m, c, n) = agua_que_sobe_de_lado(folga);
        relato(format!(
            "de lado, folga {folga} px: massa nova no miolo {m:.0} · {c} de {n} células com > 50"
        ));
    }
}

/// SONDA — a aguada amarela POR CIMA do óleo azul (dono 30/10: a mistura do main, sem branco nem bege), no papel
/// de cor (`p_*`) e no branco (`b_*`, a cena do Rebelle): a imagem a meio e assentada, e o óleo antes.
#[test]
#[ignore = "diagnóstico"]
fn diag_aguada_por_cima_do_oleo() {
    for (pre, papel) in [("p", PAPEL), ("b", [255; 3])] {
        let mut t = com_oleo_sobre(false, papel);
        grava(&format!("{pre}_antes"), &mostrada(&mut t));
        let (meio, fim) = aguada_por_cima_do_oleo_sobre(papel);
        grava(&format!("{pre}_meio"), &meio);
        grava(&format!("{pre}_fim"), &fim);
        relato(format!(
            "papel {papel:?}: claros a meio {} · assentada {}",
            mais_claros(&meio, papel),
            mais_claros(&fim, papel)
        ));
    }
}

/// Os píxeis da aguada mais CLAROS que o papel e a aguada juntos (o branco/creme): azul acima dos dois e luz acima do papel.
pub(super) fn mais_claros(px: &[u8], papel: [u8; 3]) -> usize {
    let lum =
        |p: &[u8]| 0.2126 * f32::from(p[0]) + 0.7152 * f32::from(p[1]) + 0.0722 * f32::from(p[2]);
    let lp = lum(&papel);
    (0..H)
        .flat_map(|y| (95..145).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let p = &px[(y * W + x) * 4..(y * W + x) * 4 + 3];
            let [r, b] = [i32::from(p[0]), i32::from(p[2])];
            b > i32::from(papel[2].max(40)) + 25 && r > 150 && lum(p) > lp + 15.0 && r > b + 10
        })
        .count()
}

/// A aguada amarela do Wet Flat por cima do óleo de [`com_oleo_sobre`], com o ponteiro ainda em baixo.
fn aguada_a_meio(papel: [u8; 3]) -> PainterTool {
    let mut t = com_oleo_sobre(false, papel);
    t.preset_aplicar(&preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png"))
        .expect("o meio");
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([220, 200, 40]);
    t.on_canvas_pointer(ponteiro(120.0, 30.0, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(120.0, 30.0 + 2.8 * k as f32, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t
}

/// SONDA — as células da faixa creme abaixo do óleo, a meio da aguada (papel de cor).
#[test]
#[ignore = "diagnóstico"]
fn diag_a_faixa_abaixo_do_oleo() {
    let mut t = aguada_a_meio(PAPEL);
    let m = mostrada(&mut t);
    for y in [121, 124, 127] {
        despeja(&mut t, 120, y, &m);
    }
}

/// A aguada amarela do Wet Flat por cima do óleo azul de [`com_oleo_sobre`]: (a meio, assentada).
pub(super) fn aguada_por_cima_do_oleo_sobre(papel: [u8; 3]) -> (Vec<u8>, Vec<u8>) {
    let mut t = aguada_a_meio(papel);
    let meio = mostrada(&mut t);
    t.on_canvas_pointer(ponteiro(120.0, 142.0, PointerPhase::Up));
    for _ in 0..600 {
        t.on_tick(1000.0 / 60.0);
    }
    let fim = mostrada(&mut t);
    (meio, fim)
}
