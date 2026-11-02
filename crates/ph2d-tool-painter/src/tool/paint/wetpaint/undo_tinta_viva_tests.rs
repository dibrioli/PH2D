//! **O undo mata a tinta viva** (smoke do dono, 30/10): desfazer um traço deixa a tinta que FICOU na tela sem a água,
//! o assentado e o óleo dela — o traço seguinte por cima não a re-molha nem a arrasta como faria sem o undo.
//!
//! Cena A = o óleo solúvel, um traço a mais LONGE dele, o undo desse traço e a aguada por cima; cena B = sem o traço
//! a mais. O undo devolve a tela de antes do traço: a grade e a imagem depois da aguada têm de ser as mesmas.

use super::agua_sobre_oleo_tests::{H, PAPEL, W, mostrada, ponteiro, preset};
use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase};
use ph2d_wet_paint::tuning::Knob;

fn tick(t: &mut PainterTool, n: usize) {
    for _ in 0..n {
        t.on_tick(1000.0 / 60.0);
    }
}

/// Um traço horizontal de `x0` a `x0 + 160` na linha `y`.
fn traco(t: &mut PainterTool, x0: f32, y: f32) {
    t.on_canvas_pointer(ponteiro(x0, y, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(x0 + 4.0 * k as f32, y, PointerPhase::Move));
        tick(t, 1);
    }
    t.on_canvas_pointer(ponteiro(x0 + 160.0, y, PointerPhase::Up));
}

fn base(meio: &str, cor: [u8; 3]) -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", PAPEL[0], PAPEL[1], PAPEL[2]),
    ));
    t.preset_aplicar(&preset(meio)).expect("o meio");
    t.paint.wetpaint.knobs.set(Knob::Solubility, 1.0);
    t.set_brush_size_px(30.0);
    t.set_brush_color_srgb8(cor);
    t
}

/// Os campos da tinta viva das células (sett/susp/film/wet/óleo), para comparar as duas cenas.
type Grade = (
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<u8>,
    Vec<ph2d_wet_paint::grid::oleo_celula::OleoCelula>,
);

fn grade(t: &mut PainterTool) -> Option<Grade> {
    t.wet_bring_home();
    let g = &t.paint.wetpaint.session.as_ref()?.engine.layers[0].grid;
    Some((
        g.sett.clone(),
        g.susp.clone(),
        g.film.clone(),
        g.wet.clone(),
        g.oleo.clone(),
    ))
}

/// O 1.º traço (`meio`/`cor`), o extra (só na cena A, desfeito) e a grade antes do 2.º traço.
fn ate_a_aguada(
    meio: &str,
    cor: [u8; 3],
    com_undo: bool,
    assenta: usize,
) -> (PainterTool, Option<Grade>) {
    let mut t = base(meio, cor);
    traco(&mut t, 40.0, 90.0);
    tick(&mut t, assenta);
    if com_undo {
        traco(&mut t, 40.0, 150.0);
        assert!(t.undo_last(), "havia o traço extra para desfazer");
    }
    let g = grade(&mut t);
    (t, g)
}

fn aguada(t: &mut PainterTool, meio: &str, cor: [u8; 3]) {
    t.preset_aplicar(&preset(meio)).expect("o meio");
    t.paint.wetpaint.knobs.set(Knob::Dissolve, 0.35);
    t.paint.wetpaint.knobs.set(Knob::RewetDrag, 0.15);
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8(cor);
    t.on_canvas_pointer(ponteiro(120.0, 30.0, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(120.0, 30.0 + 2.8 * k as f32, PointerPhase::Move));
        tick(t, 1);
    }
    t.on_canvas_pointer(ponteiro(120.0, 142.0, PointerPhase::Up));
    tick(t, 600);
}

fn compara(meio1: &str, cor1: [u8; 3], meio2: &str, cor2: [u8; 3], assenta: usize) {
    let (mut b, gb) = ate_a_aguada(meio1, cor1, false, assenta);
    let (mut a, ga) = ate_a_aguada(meio1, cor1, true, assenta);
    assert!(
        gb.is_some(),
        "a fixture não tem a tinta viva: a sessão da cena B morreu sem undo"
    );
    assert!(
        ga == gb,
        "{meio1}: depois do undo do traço extra a grade da tinta viva NÃO é a de sem o traço (sessão viva: {})",
        ga.is_some()
    );
    aguada(&mut b, meio2, cor2);
    aguada(&mut a, meio2, cor2);
    let (ib, ia) = (mostrada(&mut b), mostrada(&mut a));
    let dif = ib.iter().zip(&ia).filter(|(x, y)| x != y).count();
    assert_eq!(
        dif, 0,
        "{meio1} → {meio2}: o traço por cima depois do undo deu outra imagem ({dif} bytes)"
    );
}

/// ⭐ Óleo solúvel (Dabber, Solubility 1) + undo de outro traço + aguada (Re-wet 0,35, Drag 0,15) = sem o undo.
#[test]
fn undo_de_outro_traco_nao_mata_o_oleo_soluvel() {
    compara(
        "Oil/Dabber/Dabber.ph2d-brush.png",
        [40, 90, 220],
        "Watercolor/Fluid/Wet Flat.ph2d-brush.png",
        [220, 200, 40],
        120,
    );
}

/// ⭐ Aguarela × aguarela: Wet Flat azul, undo de outro traço, Wet Flat amarelo por cima ainda molhado.
#[test]
fn undo_de_outro_traco_nao_seca_a_aguarela_molhada() {
    compara(
        "Watercolor/Fluid/Wet Flat.ph2d-brush.png",
        [40, 90, 220],
        "Watercolor/Fluid/Wet Flat.ph2d-brush.png",
        [220, 200, 40],
        10,
    );
}

/// O traço A (Dabber) e o B; `refaz` = desfaz e refaz o B. A grade e a aguada por cima têm de ser as de sem o par.
fn depois_do_redo(refaz: bool) -> (PainterTool, Option<Grade>) {
    let mut t = base("Oil/Dabber/Dabber.ph2d-brush.png", [40, 90, 220]);
    traco(&mut t, 40.0, 90.0);
    tick(&mut t, 120);
    traco(&mut t, 40.0, 120.0);
    if refaz {
        assert!(t.undo_last(), "havia o B para desfazer");
        assert!(t.redo_last(), "havia o B para refazer");
    }
    let g = grade(&mut t);
    (t, g)
}

/// ⭐ O REDO não mata a tinta viva: A, B, undo, redo e a aguada por cima = A, B e a aguada.
#[test]
fn redo_nao_mata_a_tinta_viva() {
    let (mut b, gb) = depois_do_redo(false);
    let (mut a, ga) = depois_do_redo(true);
    assert!(gb.is_some(), "a fixture não tem a tinta viva");
    assert!(
        ga == gb,
        "depois do redo a grade da tinta viva NÃO é a de antes do undo (sessão viva: {})",
        ga.is_some()
    );
    aguada(
        &mut b,
        "Watercolor/Fluid/Wet Flat.ph2d-brush.png",
        [220, 200, 40],
    );
    aguada(
        &mut a,
        "Watercolor/Fluid/Wet Flat.ph2d-brush.png",
        [220, 200, 40],
    );
    let (ib, ia) = (mostrada(&mut b), mostrada(&mut a));
    let dif = ib.iter().zip(&ia).filter(|(x, y)| x != y).count();
    assert_eq!(
        dif, 0,
        "o traço por cima depois do redo deu outra imagem ({dif} bytes)"
    );
}

/// O Dabber, depois o Wet Flat com o Real Oil na mão: com `extra`, um traço a mais e o seu undo.
fn ladeira_depois(extra: bool) -> Option<Vec<f32>> {
    let mut t = base("Oil/Dabber/Dabber.ph2d-brush.png", [40, 90, 220]);
    traco(&mut t, 40.0, 90.0);
    tick(&mut t, 30);
    t.preset_aplicar(&preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png"))
        .expect("o meio");
    t.set_wet_oleo_real_ligado(true);
    if extra {
        traco(&mut t, 40.0, 130.0);
        assert!(t.undo_last(), "havia o traço extra para desfazer");
    }
    t.wet_bring_home();
    let s = t.paint.wetpaint.session.as_ref()?;
    Some(s.engine.layers[0].grid.oleo_antes.clone())
}

/// ⭐ O undo devolve a ladeira do óleo de ANTES do passo (`Grid::oleo_antes`), não a do traço desfeito.
#[test]
fn undo_devolve_a_ladeira_do_oleo_de_antes() {
    let b = ladeira_depois(false);
    let a = ladeira_depois(true);
    assert!(b.is_some() && a.is_some(), "a sessão morreu");
    assert!(
        a == b,
        "a ladeira do óleo depois do undo ({} células) não é a de sem o traço ({} células)",
        a.map_or(0, |v| v.len()),
        b.map_or(0, |v| v.len())
    );
}

/// SONDA (c): os bytes e o tempo da fotografia de UM passo a 4096², e quantos passos cabem no orçamento.
#[test]
#[ignore = "sonda: corre por nome com --ignored"]
fn diag_bytes_da_foto_por_passo() {
    for lado in [1024usize, 2048, 4096] {
        let e = ph2d_wet_paint::painter::Engine::new(lado, lado);
        let t0 = std::time::Instant::now();
        let f = super::undo_vivo::FotoDoMotor::de_para_sonda(&e);
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        let teto = super::undo_vivo::orcamento_das_fotos(lado, lado);
        eprintln!(
            "{lado}²: foto {} MB ({:.1} B/célula), {ms:.0} ms; orçamento {} MB = {} passos",
            f.bytes() >> 20,
            f.bytes() as f64 / ((lado + 2) * (lado + 2)) as f64,
            teto >> 20,
            teto / f.bytes()
        );
    }
}

/// SONDA (a): que campos do motor diferem entre a cena com o undo e a sem, antes da aguada.
#[test]
#[ignore = "sonda: corre por nome com --ignored"]
fn diag_o_que_difere_depois_do_undo() {
    let m = "Watercolor/Fluid/Wet Flat.ph2d-brush.png";
    let (mut b, _) = ate_a_aguada(m, [40, 90, 220], false, 10);
    let (mut a, _) = ate_a_aguada(m, [40, 90, 220], true, 10);
    fn h<T: std::fmt::Debug>(v: &T) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut s = std::collections::hash_map::DefaultHasher::new();
        format!("{v:?}").hash(&mut s);
        s.finish()
    }
    let dump = |t: &mut PainterTool| {
        t.wet_bring_home();
        let s = t.paint.wetpaint.session.as_ref().expect("sessão");
        let g = &s.engine.layers[0].grid;
        let e = &s.engine;
        vec![
            ("susp_rgb", h(&g.susp_rgb)),
            ("sett_rgb", h(&g.sett_rgb)),
            ("vel", h(&(&g.vel_x, &g.vel_y))),
            ("flow", h(&(&g.flow_x, &g.flow_y))),
            ("frow", h(&(&g.frow_lo, &g.frow_hi))),
            ("row", h(&(&g.row_lo, &g.row_hi))),
            ("active", h(&g.active)),
            ("bloom", h(&g.bloom)),
            ("bbox", h(&(g.bx0, g.by0, g.bx1, g.by1, g.has_fluid))),
            ("relevo", h(&g.relevo)),
            ("vela", h(&g.vela)),
            ("oleo_antes", h(&g.oleo_antes)),
            (
                "sim",
                h(&(
                    e.sim.frame,
                    e.sim.dry_every,
                    e.sim.semente_do_po,
                    e.sim.step_pending(),
                )),
            ),
            ("pigment", h(&s.planos.pigment)),
            ("sombra", h(&s.planos.sombra)),
            ("grao", h(&s.planos.grao)),
            ("lanes", h(&s.lanes.len())),
            ("adiados", h(&s.adiados.len())),
            ("lane_trails", h(&e.lane_trails.len())),
            ("tool_rng", h(&(t.paint.seed, t.paint.tex_rng))),
        ]
    };
    let (db, da) = (dump(&mut b), dump(&mut a));
    for ((n, x), (_, y)) in db.iter().zip(&da) {
        eprintln!("{n}: {}", if x == y { "igual" } else { "DIFERE" });
    }
}

/// O tecto (c): a conta sem tirar a fotografia é a da fotografia tirada (a que decide se cabe).
#[test]
fn a_conta_da_foto_e_a_da_foto_tirada() {
    let e = ph2d_wet_paint::painter::Engine::new(64, 48);
    let f = super::undo_vivo::FotoDoMotor::de_para_sonda(&e);
    assert_eq!(
        super::undo_vivo::FotoDoMotor::bytes_de_para_sonda(&e),
        f.bytes()
    );
}

/// SONDA (d): o custo da fotografia da ladeira do óleo no início de um traço (a grade inteira), por lado.
#[test]
#[ignore = "sonda: corre por nome com --ignored"]
fn diag_custo_da_ladeira_no_inicio_do_traco() {
    for lado in [1024usize, 2048, 4096] {
        let mut e = ph2d_wet_paint::painter::Engine::new(lado, lado);
        e.sim.oleo = true;
        let g = &mut e.layers[0].grid;
        g.oleo = vec![ph2d_wet_paint::grid::oleo_celula::OleoCelula::NEUTRA; g.cells];
        g.sett.fill(1.0);
        let mut melhor = f64::MAX;
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            e.begin_direct_stroke(0, 10.0, 10.0);
            melhor = melhor.min(t0.elapsed().as_secs_f64() * 1e3);
            e.end_direct_stroke();
        }
        eprintln!(
            "{lado}²: fotografia da ladeira {melhor:.1} ms (mín de 5), {} MB",
            (e.layers[0].grid.oleo_antes.len() * 4) >> 20
        );
    }
}

/// Queixa do dono 31/10 («undo/redo voltou a matar a tinta»): A, B, e `passo` (0 nada · 1 desfaz o B · 2 desfaz e
/// refaz o B), com o hospedeiro a compor por quadro como o app. Devolve a ferramenta logo depois.
fn viva_depois(passo: u8, quadro: bool) -> PainterTool {
    let mut t = base("Watercolor/Fluid/Wet Flat.ph2d-brush.png", [40, 90, 220]);
    t.set_compor_por_quadro(quadro);
    traco(&mut t, 40.0, 60.0);
    tick(&mut t, 5);
    if passo != 1 {
        traco(&mut t, 40.0, 120.0);
    } else {
        traco(&mut t, 40.0, 120.0);
        assert!(t.undo_last(), "havia o B para desfazer");
    }
    if passo == 2 {
        assert!(t.undo_last(), "havia o B para desfazer");
        assert!(t.redo_last(), "havia o B para refazer");
    }
    t
}

/// O que o artista vê sem forçar a recomposição (a tela que o quadro mostra).
fn vista(t: &mut PainterTool) -> Vec<u8> {
    t.take_preview_arc()
        .map(|(p, _, _)| p.to_vec())
        .unwrap_or_default()
}

fn estado(t: &mut PainterTool) -> (Option<Grade>, bool) {
    let g = grade(t);
    let f = t
        .paint
        .wetpaint
        .session
        .as_ref()
        .is_some_and(|s| s.engine.layers[0].grid.has_fluid);
    (g, f)
}

/// ⭐ Depois de desfazer/refazer a tinta molhada continua VIVA: a sessão sobrevive, ainda tem fluido, a água
/// continua a andar nos quadros seguintes, e tela e grade são as da linha do tempo sem o par desfaz/refaz.
#[test]
fn undo_redo_deixa_a_tinta_viva_a_andar() {
    for quadro in [false, true] {
        let mut sem = viva_depois(0, quadro);
        let mut com = viva_depois(2, quadro);
        assert!(
            com.paint.wetpaint.session.is_some(),
            "quadro={quadro}: o redo acabou a sessão (tinta morta)"
        );
        let (g0, f0) = estado(&mut com);
        assert!(f0, "quadro={quadro}: depois do redo a grade não tem fluido");
        assert_eq!(
            estado(&mut sem),
            (g0.clone(), f0),
            "quadro={quadro}: o estado logo depois do redo difere"
        );
        let _ = (vista(&mut sem), vista(&mut com));
        tick(&mut sem, 30);
        tick(&mut com, 30);
        let (g1, _) = estado(&mut com);
        assert!(
            g1 != g0,
            "quadro={quadro}: a água não andou em 30 quadros depois do redo (tinta morta)"
        );
        assert!(
            estado(&mut sem).0 == g1,
            "quadro={quadro}: a água depois do redo evolui diferente da sem undo"
        );
        let (vs, vc) = (vista(&mut sem), vista(&mut com));
        let dif = vs.iter().zip(&vc).filter(|(a, b)| a != b).count();
        assert_eq!(
            dif, 0,
            "quadro={quadro}: a tela mostrada depois do redo difere ({dif} bytes)"
        );
        // E o undo do B: a tinta do A volta viva e continua a andar.
        let mut u = viva_depois(1, quadro);
        assert!(
            u.paint.wetpaint.session.is_some(),
            "quadro={quadro}: o undo acabou a sessão"
        );
        let (h0, fu) = estado(&mut u);
        assert!(fu, "quadro={quadro}: depois do undo a grade não tem fluido");
        let _ = vista(&mut u);
        tick(&mut u, 30);
        assert!(
            estado(&mut u).0 != h0,
            "quadro={quadro}: a água do A não andou depois do undo"
        );
    }
}

/// O mesmo com a água no WORKER (relógio do app, não o fixo): a tinta continua viva depois de desfazer/refazer.
#[test]
fn undo_redo_com_o_worker_deixa_a_tinta_viva() {
    fn real(t: &mut PainterTool, n: usize) {
        for _ in 0..n {
            std::thread::sleep(std::time::Duration::from_millis(16));
            t.on_tick(1000.0 / 60.0);
        }
    }
    for passo in [1u8, 2] {
        let mut t = base("Watercolor/Fluid/Wet Flat.ph2d-brush.png", [40, 90, 220]);
        t.set_wet_relogio_fixo(false);
        t.set_compor_por_quadro(true);
        traco(&mut t, 40.0, 60.0);
        real(&mut t, 5);
        traco(&mut t, 40.0, 120.0);
        real(&mut t, 3);
        assert!(t.undo_last(), "havia o B");
        if passo == 2 {
            real(&mut t, 3);
            assert!(t.redo_last(), "havia o B para refazer");
        }
        assert!(
            t.paint.wetpaint.session.is_some(),
            "passo {passo}: a sessão morreu no undo/redo"
        );
        let (g0, f0) = estado(&mut t);
        assert!(f0, "passo {passo}: sem fluido depois do undo/redo");
        real(&mut t, 20);
        assert!(
            t.paint.wetpaint.session.is_some(),
            "passo {passo}: a sessão morreu depois"
        );
        let (g1, _) = estado(&mut t);
        assert!(
            g1 != g0,
            "passo {passo}: a água não andou depois do undo/redo (tinta morta)"
        );
    }
}

/// ⭐ Regressão 31/10 («undo/redo voltou a matar a tinta»): a fotografia de UM passo tem de caber no orçamento até
/// 4096², ou o undo acaba a sessão. Medido com a grade inteira (63 B/célula): 4096×2560 662 MB × 604 MB, não cabia.
/// O pior caso conta o óleo (a ladeira `oleo_antes` cheia).
#[test]
fn a_foto_do_passo_cabe_ate_4096() {
    let mut e = ph2d_wet_paint::painter::Engine::new(256, 256);
    let n = e.layers[0].grid.cells;
    e.layers[0].grid.oleo_antes = vec![0.0; n];
    let bpc = super::undo_vivo::FotoDoMotor::bytes_de_para_sonda(&e) as f64 / n as f64;
    for (w, h) in [
        (1920usize, 1080usize),
        (3840, 2160),
        (4096, 2560),
        (4096, 4096),
    ] {
        let foto = bpc * ((w + 2) * (h + 2)) as f64;
        let teto = super::undo_vivo::orcamento_das_fotos(w, h) as f64;
        // antes + depois de UM passo, e ainda um segundo passo: três fotografias.
        assert!(
            3.0 * foto <= teto,
            "{w}x{h}: 3 fotos {:.0} MB > orçamento {:.0} MB ({bpc:.1} B/célula)",
            3.0 * foto / 1e6,
            teto / 1e6
        );
    }
}

/// **Um undo que não se aplica não mexe no ritmo da água**: soltar a sessão para o undo e voltar com
/// `aplicou = false` deixa o passo em voo onde estava (antes, o soltar drenava-o à cabeça).
#[test]
fn undo_que_nao_se_aplica_nao_drena_o_passo_em_voo() {
    let mut t = base("Watercolor/Fluid/Wet Flat.ph2d-brush.png", [40, 60, 160]);
    traco(&mut t, 40.0, 90.0);
    tick(&mut t, 20);
    traco(&mut t, 40.0, 150.0); // o 1.º traço abre a sessão: só o 2.º tem fotografia
    t.wet_bring_home();
    let e = &mut t
        .paint
        .wetpaint
        .session
        .as_mut()
        .expect("a sessão viva")
        .engine;
    e.drain_step();
    assert!(e.step_stage().is_none(), "o passo completou num estágio");
    let estagio = e.sim.stage_in_flight();
    assert!(estagio.is_some());
    let solta = t
        .wet_solta_para_o_undo(false)
        .expect("o passo tem fotografia");
    t.wet_volta_do_undo(solta, false);
    let e = &t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessão volta")
        .engine;
    assert_eq!(e.sim.stage_in_flight(), estagio, "o undo falhado drenou");
}
