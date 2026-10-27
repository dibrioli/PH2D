//! **Arrastar um número do cartão Real Oil** (smoke do dono 27/10: «arrastando os números está calculando mil
//! vezes por segundo e fica travando»): cada `SetValue` do arrasto chega à ferramenta por evento do rato, e o
//! que ele pede à tela tem de caber no quadro — a recomposição do óleo corre no TICK, com o último valor.

use super::composite_contagem as contagem;
use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase};

/// O Specular do cartão (`PAINTER_WETPAINT_OLEO_FIELDS[2]`, `0..1`): só muda o sombreado.
const BRILHO: usize = 2;

/// Uma tela `w × h` em Wet Paint com o Óleo Real ligado e dois traços que se cruzam, a água parada.
fn tela(w: usize, h: usize) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![240u8; w * h * 4], w as u32, h as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_wet_oleo_real_ligado(true);
    t.set_brush_size_px(14.0);
    let traco = |t: &mut PainterTool, a: [f32; 2], b: [f32; 2], cor: [u8; 3]| {
        t.set_brush_color_srgb8(cor);
        let p = |k: f32, phase| CanvasPointer {
            pos: [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k],
            pressure: 0.9,
            tilt: [0.0, 0.0],
            phase,
        };
        t.on_canvas_pointer(p(0.0, PointerPhase::Down));
        for i in 1..=30 {
            t.on_canvas_pointer(p(i as f32 / 30.0, PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(p(1.0, PointerPhase::Up));
    };
    let (fw, fh) = (w as f32, h as f32);
    traco(
        &mut t,
        [0.1 * fw, 0.3 * fh],
        [0.9 * fw, 0.45 * fh],
        [180, 40, 30],
    );
    traco(
        &mut t,
        [0.3 * fw, 0.9 * fh],
        [0.6 * fw, 0.1 * fh],
        [30, 60, 190],
    );
    t.on_tick(0.0);
    assert!(t.paint.wetpaint.session.is_some(), "a cena perdeu a sessão");
    t
}

fn arrasta(t: &mut PainterTool, n: usize, de: f64) {
    for i in 0..n {
        t.handle_panel_event(PanelEvent::SetValue(
            crate::ids::PAINTER_WETPAINT_OLEO_FIELDS[BRILHO],
            de + 0.3 * i as f64 / n as f64,
        ));
    }
}

fn arrasta_suave(t: &mut PainterTool, n: usize, de: f64) {
    for i in 0..n {
        t.handle_panel_event(PanelEvent::SetValue(
            crate::ids::PAINTER_WETPAINT_SMOOTH_EDGES,
            de + 0.3 * i as f64 / n as f64,
        ));
    }
}

/// Os ticks até a varredura fechar (tecto: a tela nunca fica pendente para sempre).
fn varre(t: &mut PainterTool) -> usize {
    let mut q = 0;
    while t.oleo_por_compor() {
        t.on_tick(0.0);
        q += 1;
        assert!(q < 10_000, "a varredura não fecha");
    }
    q
}

/// **N eventos de arrasto num quadro → zero recomposições nos eventos e UMA no tick** (o Specular do Real Oil e o
/// Smooth Edges, os dois números que recompõem a tela). Antes da cura (`no_evento`) eram N.
#[test]
fn arrastar_um_numero_do_oleo_recompoe_uma_vez_por_quadro() {
    let mut t = tela(160, 120);
    oleo_no_quadro::faixa_fixa(Some(usize::MAX));
    for (nome, f) in [
        ("Specular", arrasta as fn(&mut PainterTool, usize, f64)),
        ("Smooth Edges", arrasta_suave),
    ] {
        let _ = contagem::take();
        f(&mut t, 30, 0.2);
        let (nos_eventos, ..) = contagem::take();
        t.on_tick(0.0);
        let (no_tick, ..) = contagem::take();
        assert_eq!(
            (nos_eventos, no_tick),
            (0, 1),
            "{nome}: 30 eventos de arrasto num quadro — (recomposições nos eventos, no tick)"
        );
        assert!(
            !t.oleo_por_compor(),
            "{nome}: a tela inteira cabia numa faixa"
        );
    }
    oleo_no_quadro::faixa_fixa(None);
}

/// **A tela depois da varredura é a recomposição inteira, ao byte** — em faixas de 7 linhas, com o arrasto
/// a recomeçar a meio da varredura e o Smooth Edges (que lê pixels vizinhos) ligado. A tela é ALTA (640 px): uma
/// faixa saltada tem de ficar mais longe que o alcance do óleo para a régua a ver.
#[test]
fn a_tela_depois_da_varredura_e_a_recomposicao_inteira_ao_byte() {
    let mut a = tela(200, 640);
    let mut b = tela(200, 640);
    assert_eq!(
        a.canvas_rgba, b.canvas_rgba,
        "as duas cenas nasceram diferentes"
    );
    for t in [&mut a, &mut b] {
        t.set_wet_smooth_edges(0.4);
    }
    oleo_no_quadro::faixa_fixa(Some(7));
    arrasta(&mut a, 25, 0.1);
    for _ in 0..3 {
        a.on_tick(0.0);
    }
    assert!(
        a.oleo_por_compor(),
        "a varredura acabou antes de o arrasto recomeçar"
    );
    arrasta_suave(&mut a, 10, 0.5);
    arrasta(&mut a, 10, 0.6);
    let quadros = varre(&mut a);
    oleo_no_quadro::faixa_fixa(None);
    assert!(
        quadros > 3,
        "a varredura não se fez em faixas ({quadros} quadros)"
    );
    b.set_wet_smooth_edges(a.wet_smooth_edges());
    b.set_wet_oleo_real(a.wet_oleo_real());
    let difs = a
        .canvas_rgba
        .iter()
        .zip(b.canvas_rgba.iter())
        .filter(|(x, y)| x != y)
        .count();
    assert_eq!(
        difs, 0,
        "{difs} bytes diferentes entre a varredura e a recomposição inteira"
    );
    assert_ne!(
        a.wet_oleo_real(),
        Some(Default::default()),
        "o arrasto não chegou ao óleo"
    );
}

/// **A sonda do defeito**, antes × depois intercalados no mesmo processo: `N` eventos de arrasto num quadro — os
/// ms nos EVENTOS e no pior TICK, as recomposições — à tela do smoke (1024²) e a 4K.
/// `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_arrasto_do_oleo -- --ignored --nocapture`
#[test]
#[ignore = "sonda de custo — corre-se à mão no perfil smoke"]
fn sonda_arrasto_do_oleo() {
    for (w, h) in [(1024, 1024), (3840, 2160)] {
        let mut t = tela(w, h);
        let n = 16;
        for ronda in 0..4 {
            let antes = ronda % 2 == 0;
            oleo_no_quadro::no_evento(antes);
            let _ = (
                contagem::take(),
                contagem::take_tempos(),
                contagem::take_campo(),
            );
            let t0 = std::time::Instant::now();
            arrasta(&mut t, n, 0.2 + 0.05 * ronda as f64);
            let ev = t0.elapsed().as_secs_f64() * 1e3;
            let (c_ev, ..) = contagem::take();
            let mut quadros = Vec::new();
            loop {
                let q = std::time::Instant::now();
                t.on_tick(0.0);
                quadros.push(q.elapsed().as_secs_f64() * 1e3);
                if !t.oleo_por_compor() {
                    break;
                }
            }
            let (c_tk, _, cel) = contagem::take();
            let (tp, ts) = contagem::take_tempos();
            let tc = contagem::take_campo();
            let pior = quadros.iter().copied().fold(0.0, f64::max);
            let soma: f64 = quadros.iter().sum();
            let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
            println!(
                "[sonda] {w}x{h} {}: {n} eventos {ev:.1} ms ({c_ev} composites) · tick {soma:.1} ms em {} quadros, \
                 pior {pior:.1} ms ({c_tk} composites, {cel} células) · campo {tc:.1} · pigmento {tp:.1} · sombra \
                 {ts:.1} ms · load {}",
                if antes { "ANTES " } else { "DEPOIS" },
                quadros.len(),
                load.trim()
            );
        }
        oleo_no_quadro::no_evento(false);
    }
}
