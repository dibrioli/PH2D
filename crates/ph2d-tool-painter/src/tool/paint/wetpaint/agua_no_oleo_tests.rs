//! SONDA J11 (smoke do dono J10): a água × o óleo seco num bloco — (1) a água entra no corpo do óleo mais depressa
//! que escorre no papel? (2) Re-wet × Solubility dissolve e BAIXA o relevo? (3) o pixelado da orla. Dabber azul seco,
//! Wet Drip d0,9 por cima (a cena de `escorre_contorna_tests`). Medição, nunca portão.

use super::agua_sobre_oleo_tests::{H, PAPEL, W, grava, mostrada, preset, relato};
use super::escorre_contorna_tests::{ObstaculoA, grid, ticks, traco};
use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_wet_paint::tuning::Knob;

/// `(nome, com A, Solubility de A, Re-wet de B, Drag de B, passadas de B)`.
pub(super) type Variante = (&'static str, bool, f64, f64, f64, u32);

/// `true` = a fase B ao relógio REAL (o worker do app), 16 ms por tique.
static REAL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

const VARIANTES: [Variante; 7] = [
    ("papel", false, 0.0, 0.0, 0.0, 1),
    ("papel6", false, 0.0, 0.0, 0.0, 6),
    ("hoje", true, 0.0, 0.0, 0.0, 1),
    ("hoje6", true, 0.0, 0.0, 0.0, 6),
    ("s1rw035", true, 1.0, 0.35, 0.0, 1),
    ("s1rw035x6", true, 1.0, 0.35, 0.0, 6),
    ("s1rw1", true, 1.0, 1.0, 0.0, 1),
];

/// As colunas de B e de A (células, já com o anel de pad): A de x 40–200, B de 60–180; mede em 70..=170.
const X0: usize = 71;
const X1: usize = 171;

pub(super) fn linha(v: Variante) -> String {
    let (nome, com_a, sol, rw, drag, vezes) = v;
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", PAPEL[0], PAPEL[1], PAPEL[2]),
    ));
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o Dabber");
    t.paint.wetpaint.knobs.set(Knob::Solubility, sol);
    t.set_brush_size_px(30.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    // Sem A, um toque de água limpa longe da cena só para abrir a sessão (a grade).
    if com_a {
        traco(&mut t, 40.0, 200.0, 90.0);
    } else {
        t.paint.wetpaint.knobs.set(Knob::Dilution, 1.0);
        traco(&mut t, 2.0, 4.0, 150.0);
    }
    ticks(&mut t, 1200);
    t.wet_bring_home();
    let (antes, sett0, s, gh) = {
        let g = grid(&t);
        let antes: Vec<f32> = (0..g.cells).map(|i| g.sett[i] + g.susp[i]).collect();
        (antes, g.sett.clone(), g.s, g.cells / g.s - 2)
    };
    let top_a: Vec<Option<usize>> = (X0..=X1)
        .map(|cx| (1..=gh).find(|&cy| antes[cx + cy * s] > 500.0))
        .collect();
    let bot_a: Vec<Option<usize>> = (X0..=X1)
        .map(|cx| (1..=gh).rev().find(|&cy| antes[cx + cy * s] > 500.0))
        .collect();

    t.preset_aplicar(&preset("Watercolor/Fluid/Wet Drip.ph2d-brush.png"))
        .expect("o Wet Drip");
    t.paint.wetpaint.knobs.set(Knob::Dilution, 0.9);
    t.paint.wetpaint.knobs.set(Knob::Medium, 0.0);
    t.paint.wetpaint.knobs.set(Knob::Dissolve, rw);
    t.paint.wetpaint.knobs.set(Knob::RewetDrag, drag);
    t.set_brush_size_px(20.0);
    t.set_brush_color_srgb8([240, 220, 40]);
    let real = REAL.load(std::sync::atomic::Ordering::Relaxed);
    if real {
        t.set_wet_relogio_fixo(false);
    }
    for _ in 0..vezes {
        traco_vendo(&mut t, 60.0, 180.0, 55.0);
    }
    t.wet_bring_home();
    let chao_fim_traco = {
        let g = grid(&t);
        g.chao.iter().copied().fold(0f32, f32::max)
    };
    // A frente de B por tempo: a linha mais funda (média das colunas) com massa nova > 50, e a velocidade média
    // da água nas células molhadas sobre o óleo (A > 500 antes) e sobre o papel.
    let mut serie = String::new();
    for passo in 0..=12u32 {
        if passo > 0 {
            ticks_vendo(&mut t, if passo <= 8 { 15 } else { 150 });
        }
        t.wet_bring_home();
        let g = grid(&t);
        let mut fr = 0f64;
        let mut fr_n = 0usize;
        let (mut vo, mut no, mut vp, mut np) = (0f64, 0usize, 0f64, 0usize);
        for cx in X0..=X1 {
            if let Some(cy) = (1..=gh).rev().find(|&cy| {
                let i = cx + cy * s;
                f64::from(g.sett[i] + g.susp[i] - antes[i]) > 50.0
            }) {
                fr += cy as f64;
                fr_n += 1;
            }
            for cy in 1..=gh {
                let i = cx + cy * s;
                if g.film[i] <= 0.01 {
                    continue;
                }
                let v = f64::from(g.vel_x[i]).hypot(f64::from(g.vel_y[i]));
                if antes[i] > 500.0 {
                    vo += v;
                    no += 1;
                } else {
                    vp += v;
                    np += 1;
                }
            }
        }
        serie += &format!(
            " | {}:{:.1} vo{:.3}/{} vp{:.3}/{}",
            passo,
            fr / fr_n.max(1) as f64,
            vo / no.max(1) as f64,
            no,
            vp / np.max(1) as f64,
            np
        );
    }
    // A tela INCREMENTAL (o que o app mostra) × a recomposta inteira: um bloco velho é um rectângulo de diferença.
    t.wet_bring_home();
    ve(&mut t);
    let inc = ULTIMA.with(|u| u.borrow_mut().take());
    let img = mostrada(&mut t);
    let velho = inc.map_or("sem tela".to_string(), |inc| {
        let (mut n, mut mx, mut x0, mut y0, mut x1, mut y1) = (0usize, 0u8, W, H, 0, 0);
        for (k, (a, b)) in inc.chunks(4).zip(img.chunks(4)).enumerate() {
            let d = (0..3).map(|c| a[c].abs_diff(b[c])).max().unwrap_or(0);
            if d > 8 {
                n += 1;
                mx = mx.max(d);
                let (x, y) = (k % W, k / W);
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
            }
        }
        if n > 0 {
            grava(&format!("j11_{nome}_incremental"), &inc);
        }
        format!("velho {n} px max {mx} caixa {x0},{y0}-{x1},{y1}")
    });
    let suf = SUFIXO.lock().expect("o sufixo").clone();
    grava(&format!("j11_{nome}{suf}"), &img);
    grava_zoom(&format!("j11_{nome}{suf}_zoom"), &img, 60, 40, 120, 90, 4);
    t.wet_bring_home();
    let g = grid(&t);
    // A dissolução: o assentado de A no que B cobriu (linhas 60..=80) depois / antes; a profundidade do verde no
    // corpo de A: linhas abaixo da orla de cima de A com massa nova > 50 (média por coluna).
    let (mut s0, mut s1, mut entra, mut n, mut abaixo) = (0f64, 0f64, 0f64, 0usize, 0f64);
    for (k, cx) in (X0..=X1).enumerate() {
        let (Some(top), Some(bot)) = (top_a[k], bot_a[k]) else {
            continue;
        };
        for cy in top..=bot.min(top + 20) {
            let i = cx + cy * s;
            s0 += f64::from(sett0[i]);
            s1 += f64::from(g.sett[i]);
        }
        let fundo = (top..=bot)
            .rev()
            .find(|&cy| {
                let i = cx + cy * s;
                f64::from(g.sett[i] + g.susp[i] - antes[i]) > 50.0
            })
            .map_or(0, |cy| cy + 1 - top);
        entra += fundo as f64;
        for cy in bot + 1..=gh {
            let i = cx + cy * s;
            abaixo += f64::from(g.sett[i] + g.susp[i] - antes[i]).max(0.0);
        }
        n += 1;
    }
    format!(
        "{nome:<11}| chao fim do traço {chao_fim_traco:.2} | sett orla A {:.3} | verde no corpo {:.1} linhas | abaixo {abaixo:.0} | {velho}{serie}",
        s1 / s0.max(1e-9),
        entra / n.max(1) as f64
    )
}

/// Como o app: um tique, e a tela drenada (composição INCREMENTAL) a cada tique.
fn ticks_vendo(t: &mut PainterTool, n: u32) {
    for _ in 0..n {
        espera();
        t.on_tick(1000.0 / 60.0);
        ve(t);
    }
}

thread_local! {
    /// A última tela drenada (a que o app mostra agora).
    static ULTIMA: std::cell::RefCell<Option<Vec<u8>>> = const { std::cell::RefCell::new(None) };
}

fn ve(t: &mut PainterTool) {
    if let Some((px, _, _)) = t.take_preview_arc() {
        ULTIMA.with(|u| *u.borrow_mut() = Some(px.to_vec()));
    }
}

fn espera() {
    if REAL.load(std::sync::atomic::Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

/// O traço de [`traco`] com a tela drenada a cada passo.
fn traco_vendo(t: &mut PainterTool, x0: f32, x1: f32, y: f32) {
    use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
    let pt = super::agua_sobre_oleo_tests::ponteiro;
    t.on_canvas_pointer(pt(x0, y, PointerPhase::Down));
    for k in 1..=40 {
        let x = x0 + (x1 - x0) * k as f32 / 40.0;
        t.on_canvas_pointer(pt(x, y, PointerPhase::Move));
        espera();
        t.on_tick(1000.0 / 60.0);
        ve(t);
    }
    t.on_canvas_pointer(pt(x1, y, PointerPhase::Up));
}

/// Um recorte `(x, y, w, h)` ampliado `k` vezes (vizinho mais próximo): o pixelado à vista.
fn grava_zoom(nome: &str, px: &[u8], x0: usize, y0: usize, w: usize, h: usize, k: usize) {
    let dir = std::env::var("PH2D_SONDA_DIR").unwrap_or_default();
    if dir.is_empty() {
        return;
    }
    let mut v = format!("P6\n{} {}\n255\n", w * k, h * k).into_bytes();
    for y in 0..h * k {
        for x in 0..w * k {
            let i = ((y0 + y / k) * W + x0 + x / k) * 4;
            v.extend_from_slice(&px[i..i + 3]);
        }
    }
    std::fs::write(format!("{dir}/{nome}.ppm"), v).expect("grava");
    let _ = H;
}

#[test]
#[ignore = "sonda J11"]
fn diag_j11_agua_no_oleo() {
    use std::sync::atomic::Ordering::Relaxed;
    // (atrito, taxa da água parada): `< 0` = desligado, `0` = o produto — a varredura da J11 em `J11_agua_no_oleo.md`.
    REAL.store(std::env::var("PH2D_J11_REAL").is_ok(), Relaxed);
    let (a, tx) = (0.0f32, 0.0f32);
    let _a = ph2d_wet_paint::solver::regua::Regua::poe(
        &ph2d_wet_paint::solver::CHAO_ATRITO,
        a.to_bits(),
    );
    let _t = ph2d_wet_paint::solver::regua::Regua::poe(
        &ph2d_wet_paint::solver::REMOLHA_TAXA,
        tx.to_bits(),
    );
    *SUFIXO.lock().expect("o sufixo") = format!("_a{a}t{tx}");
    bloco();
}

static SUFIXO: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

fn bloco() {
    let suf = SUFIXO.lock().expect("o sufixo").clone();
    let linhas: Vec<String> = std::thread::scope(|s| {
        let h: Vec<_> = VARIANTES
            .iter()
            .map(|&v| {
                std::thread::Builder::new()
                    .stack_size(64 << 20)
                    .spawn_scoped(s, move || linha(v))
                    .expect("a thread")
            })
            .collect();
        let reguas: Vec<_> = [
            (ObstaculoA::Dabber, 1u32),
            (ObstaculoA::DabberCurto, 1),
            (ObstaculoA::Dabber, 6),
        ]
        .into_iter()
        .map(|(a, n)| {
            std::thread::Builder::new()
                .stack_size(64 << 20)
                .spawn_scoped(s, move || {
                    format!(
                        "regua x{n} {}",
                        super::escorre_contorna_tests::linha(a, 1200, 0.0, 0, n)
                    )
                })
                .expect("a thread")
        })
        .collect();
        h.into_iter()
            .chain(reguas)
            .map(|h| h.join().unwrap_or_else(|_| "REBENTOU".into()))
            .collect()
    });
    for l in linhas {
        relato(format!("{suf:<6}{l}"));
    }
}

/// SONDA J11 — o FPS com o Real Oil: quem marca o sujo do motor a cada tique depois de um traço (o app: tique + a tela
/// drenada). `(pincel, Dilution)`; a tabela soma os 300 tiques depois do traço.
#[test]
#[ignore = "sonda J11"]
fn diag_j11_quem_suja() {
    use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
    let pt = super::agua_sobre_oleo_tests::ponteiro;
    for (rel, dil) in [
        ("Oil/Dabber/Dabber.ph2d-brush.png", None),
        ("Oil/Flat/Filbert.ph2d-brush.png", Some(0.9)),
    ] {
        let mut t = PainterTool::default();
        t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
        t.set_paint_tool_mode("wetpaint");
        t.set_wet_relogio_fixo(true);
        t.preset_aplicar(&preset(rel)).expect("o pincel");
        if let Some(d) = dil {
            t.paint.wetpaint.knobs.set(Knob::Dilution, d);
        }
        t.set_brush_size_px(24.0);
        t.set_brush_color_srgb8([40, 90, 220]);
        t.on_canvas_pointer(pt(30.0, 60.0, PointerPhase::Down));
        for k in 1..=40 {
            t.on_canvas_pointer(pt(30.0 + 4.5 * k as f32, 60.0, PointerPhase::Move));
            t.on_tick(1000.0 / 60.0);
            let _ = t.take_preview_arc();
        }
        t.on_canvas_pointer(pt(210.0, 60.0, PointerPhase::Up));
        let _ = ph2d_wet_paint::painter::conta_sujo::tira();
        let _ = super::composite_contagem::take();
        ph2d_wet_paint::painter::conta_sujo::CONTA_SUJO
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let mut comp = (0u64, 0u64, 0u64);
        let mut caixa = 0i64;
        for _ in 0..300 {
            t.on_tick(1000.0 / 60.0);
            let _ = t.take_preview_arc();
            let c = super::composite_contagem::take();
            comp = (comp.0 + c.0, comp.1 + c.1, comp.2 + c.2);
            t.wet_bring_home();
            let g = grid(&t);
            if g.has_fluid {
                caixa += i64::from(g.bx1 - g.bx0 + 1) * i64::from(g.by1 - g.by0 + 1);
            }
        }
        ph2d_wet_paint::painter::conta_sujo::CONTA_SUJO
            .store(false, std::sync::atomic::Ordering::Relaxed);
        let (oleo_real, campo) = {
            let g = grid(&t);
            (g.oleo_real, !g.oleo.is_empty())
        };
        relato(format!(
            "{rel} d{dil:?}: composição {comp:?} | caixa molhada média {} células | Grid::oleo_real {oleo_real} campo {campo} | wet_oleo_real {}",
            caixa / 300,
            t.wet_oleo_real().is_some()
        ));
        for (k, n, c) in ph2d_wet_paint::painter::conta_sujo::tira() {
            relato(format!("   {k:<70} {n:>6} chamadas {c:>10} células"));
        }
    }
}

/// SONDA J11 — o quadro do app com um traço de Dabber numa tela grande (1200×800): ms por tique de `on_tick` e da tela
/// drenada, no traço e nos 120 tiques depois, Real Oil ligado × desligado, intercalados (o MÍNIMO de 3).
#[test]
#[ignore = "sonda J11"]
fn diag_j11_fps_do_dabber() {
    use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
    use std::time::Instant;
    let (w, h) = (1200usize, 800usize);
    let pt = super::agua_sobre_oleo_tests::ponteiro;
    let mut res: std::collections::BTreeMap<&str, Vec<[f64; 6]>> = Default::default();
    for _ in 0..3 {
        for (nome, real) in [("real oil", true), ("sem real oil", false)] {
            let mut t = PainterTool::default();
            t.abre_a_sprite(vec![255u8; w * h * 4], w as u32, h as u32);
            t.set_paint_tool_mode("wetpaint");
            t.set_wet_relogio_fixo(true);
            t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
                .expect("o Dabber");
            t.set_wet_oleo_real_ligado(real);
            t.set_brush_size_px(30.0);
            t.set_brush_color_srgb8([40, 90, 220]);
            let mut m = [0f64; 5];
            let mut quadro = |t: &mut PainterTool, k: usize| {
                let t0 = Instant::now();
                t.on_tick(1000.0 / 60.0);
                let t1 = Instant::now();
                let _ = t.take_preview_arc();
                m[k] += (t1 - t0).as_secs_f64() * 1e3;
                m[k + 1] += t1.elapsed().as_secs_f64() * 1e3;
            };
            t.on_canvas_pointer(pt(200.0, 400.0, PointerPhase::Down));
            let mut ponteiro_ms = 0.0;
            for k in 1..=60 {
                let t0 = Instant::now();
                t.on_canvas_pointer(pt(200.0 + 12.0 * k as f32, 400.0, PointerPhase::Move));
                ponteiro_ms += t0.elapsed().as_secs_f64() * 1e3;
                quadro(&mut t, 0);
            }
            t.on_canvas_pointer(pt(920.0, 400.0, PointerPhase::Up));
            for _ in 0..120 {
                quadro(&mut t, 2);
            }
            let px = t.take_preview_arc().map(|(p, _, _)| p.to_vec());
            let _ = px;
            t.invalidate_composite();
            let pintados = t
                .take_preview_arc()
                .map_or(0, |(p, _, _)| p.chunks(4).filter(|c| c[0] < 200).count());
            m[4] = pintados as f64;
            res.entry(nome).or_default().push([
                m[0] / 60.0,
                m[1] / 60.0,
                m[2] / 120.0,
                m[3] / 120.0,
                ponteiro_ms / 60.0,
                m[4],
            ]);
        }
    }
    for (nome, v) in res {
        let min = |k: usize| v.iter().map(|x| x[k]).fold(f64::MAX, f64::min);
        relato(format!(
            "{nome:<13}| no traço: ponteiro {:.2} ms tique {:.2} ms tela {:.2} ms | depois: tique {:.2} ms tela {:.2} ms | pintados {:.0}",
            min(4),
            min(0),
            min(1),
            min(2),
            min(3),
            min(5)
        ));
    }
}
