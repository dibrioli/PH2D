//! `PH2D_WETPAINT_SMOKE=3` — **a cena do FPS** (J13 §1): o app pinta SOZINHO, ao relógio de parede, e
//! cronometra cada quadro. Por rodada: um traço do Dabber (Real Oil por `PH2D_FPS_OLEO=0|1`, omissão 1 — por
//! PROCESSO: a grade que já viu óleo fica com a óptica dele), a espera, o Wet Drip por cima, a água a correr. O período é o tempo entre duas chamadas
//! seguidas de [`conduz`] (uma por quadro, de dentro da ponte); o ponteiro chega a `HZ` eventos por
//! segundo como um rato real. As faixas acumulam (a água das anteriores ainda corre): o relato é POR RODADA.
//! Tamanho da tela: `PH2D_FPS_TELA` = `<lado>` ou `<w>x<h>` (`3840x2160` = o 4K do produto), omissão 1024².

use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_tool_painter::PainterTool;
use std::time::Instant;

const HZ: f64 = 250.0;
const RODADAS: usize = 6;
/// `(nome, segundos, pincel)` — `pincel` = o preset que risca nesta fase (`None` = mãos paradas).
const FASES: [(&str, f64, Option<&str>); 4] = [
    (
        "dabber",
        2.5,
        Some("WetPainter/Oil/Dabber/Dabber.ph2d-brush.png"),
    ),
    ("seca", 2.0, None),
    (
        "wet drip",
        2.5,
        Some("WetPainter/Watercolor/Fluid/Wet Drip.ph2d-brush.png"),
    ),
    ("corre", 5.0, None),
];

use crate::wetpaint_smoke::tela_do_fps;

#[derive(Default)]
struct Estado {
    /// `rodada · FASES + fase`; `None` antes do primeiro quadro.
    passo: Option<usize>,
    inicio_fase: Option<Instant>,
    ultimo_quadro: Option<Instant>,
    eventos: usize,
    /// Por quadro da fase: `(período ms, ponteiro ms, eventos)`.
    quadros: Vec<(f64, f64, usize)>,
    /// `[fase]` → por rodada `(p50, p95, max, ponteiro p50, eventos/quadro)`.
    tabela: [Vec<[f64; 5]>; FASES.len()],
    feito: bool,
}

thread_local! {
    static ESTADO: std::cell::RefCell<Estado> = std::cell::RefCell::default();
}

fn preset(rel: &str) -> Option<ph2d_brush_presets::Preset> {
    let f = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/brushes")
        .join(rel);
    ph2d_brush_presets::ler(&std::fs::read(f).ok()?).ok()
}

fn quantil(v: &mut [f64], q: f64) -> f64 {
    v.sort_by(f64::total_cmp);
    v.get(((v.len().max(1) - 1) as f64 * q).round() as usize)
        .copied()
        .unwrap_or(0.0)
}

fn ponto(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 0.7,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um quadro da cena. Chamada pela ponte a cada quadro com o Painter activo; não faz nada fora do `=3`.
pub fn conduz(painter: &mut PainterTool) {
    if !crate::wetpaint_smoke::e_a_cena_do_fps() {
        return;
    }
    if let Some(rel) = ph2d_tool_painter::wetpaint_fps_primeiro::pedido() {
        return ph2d_tool_painter::wetpaint_fps_primeiro::quadro(rel, painter);
    }
    ESTADO.with(|e| quadro(&mut e.borrow_mut(), painter));
}

fn quadro(e: &mut Estado, painter: &mut PainterTool) {
    if e.feito {
        return;
    }
    let agora = Instant::now();
    let periodo = e.ultimo_quadro.map(|t| (agora - t).as_secs_f64() * 1e3);
    e.ultimo_quadro = Some(agora);
    let total = RODADAS * FASES.len();
    let passo = *e.passo.get_or_insert(0);
    let fase = passo % FASES.len();
    let (_, dur, pincel) = FASES[fase];
    if e.inicio_fase.is_none() {
        e.inicio_fase = Some(agora);
        e.eventos = 0;
        let _ = ph2d_tool_painter::wet_diag::take_otica();
        e.quadros.clear();
        if let Some(rel) = pincel {
            let Some(p) = preset(rel) else {
                eprintln!("[fps] o preset {rel} não foi encontrado");
                e.feito = true;
                return;
            };
            let _ = painter.preset_aplicar(&p);
            // A caixa Real Oil pela porta do painel (a gravada), nunca pelo setter interno.
            if fase == 0 && painter.wet_oleo_real().is_some() != oleo() {
                use ph2d_editor_core::Tool;
                painter.handle_panel_event(ph2d_editor_core::tool::PanelEvent::Click(
                    ph2d_tool_painter::ids::PAINTER_WETPAINT_OLEO_REAL,
                ));
            }
            painter.set_brush_size_px(30.0);
            painter.set_brush_color_srgb8(if fase == 0 {
                [40, 90, 220]
            } else {
                [230, 200, 30]
            });
        }
    }
    let t = (agora - e.inicio_fase.unwrap_or(agora)).as_secs_f64();
    let [w, h] = tela_do_fps().map(|v| v as f32);
    // As faixas sobem de baixo para cima: a água de uma faixa nova escorre para as já medidas.
    let y = h * (0.9 - 0.8 * (passo / FASES.len()) as f32 / RODADAS as f32);
    let x = |k: usize| w * 0.1 + w * 0.8 * ((k as f64 / HZ) / dur).min(1.0) as f32;
    let t0 = Instant::now();
    let mut n = 0;
    if pincel.is_some() {
        let alvo = ((t * HZ) as usize).min((dur * HZ) as usize);
        if e.eventos == 0 && alvo > 0 {
            painter.on_canvas_pointer(ponto(x(0), y, PointerPhase::Down));
        }
        while e.eventos < alvo {
            e.eventos += 1;
            n += 1;
            painter.on_canvas_pointer(ponto(x(e.eventos), y, PointerPhase::Move));
        }
    }
    let ponteiro = t0.elapsed().as_secs_f64() * 1e3;
    if let Some(p) = periodo {
        e.quadros.push((p, ponteiro, n));
    }
    if t < dur {
        return;
    }
    if pincel.is_some() {
        painter.on_canvas_pointer(ponto(x(e.eventos), y, PointerPhase::Up));
    }
    let mut per: Vec<f64> = e.quadros.iter().map(|q| q.0).collect();
    let mut pon: Vec<f64> = e.quadros.iter().map(|q| q.1).collect();
    let ev = e.quadros.iter().map(|q| q.2).sum::<usize>() as f64 / e.quadros.len().max(1) as f64;
    let max = per.iter().copied().fold(0.0, f64::max);
    // `quadros[0]` fecha a fase anterior; o `[1]` paga a troca de pincel da CENA (ler o preset do disco), o `[2]` o
    // pen-down. O máximo sem a troca é o que o artista sente.
    let q = |k: usize| e.quadros.get(k).map_or(0.0, |q| q.0);
    let max_sem_troca = per.iter().skip(2).copied().fold(0.0, f64::max);
    let linha = [
        quantil(&mut per, 0.5),
        quantil(&mut per, 0.95),
        max,
        quantil(&mut pon, 0.5),
        ev,
    ];
    // O compositor (ligado por omissão; `PH2D_WET_OTICA_FORA=0` desliga): imagens da água por segundo e o custo de cada uma, fora do fio da tela.
    let ((c_ms, c_max, c_n), (o_ms, o_max, o_n)) = ph2d_tool_painter::wet_diag::take_otica();
    let media = |soma: f64, n: u64| soma / n.max(1) as f64;
    eprintln!(
        "[fps] rodada {} {} {:<9}| quadros {:>4} | período p50 {:.1} p95 {:.1} max {:.1} ms | ponteiro p50 {:.2} ms | {:.1} eventos/quadro | imagens {:.1}/s óptica {:.1}/{:.1} ms cópia {:.2}/{:.2} ms | troca {:.1} pen-down {:.1} max sem a troca {:.1} ms",
        passo / FASES.len(),
        ["sem real", "real oil"][usize::from(oleo())],
        FASES[fase].0,
        e.quadros.len(),
        linha[0],
        linha[1],
        linha[2],
        linha[3],
        linha[4],
        o_n as f64 / t.max(1e-3),
        media(o_ms, o_n),
        o_max,
        media(c_ms, c_n),
        c_max,
        q(1),
        q(2),
        max_sem_troca
    );
    e.tabela[fase].push(linha);
    e.inicio_fase = None;
    e.passo = Some(passo + 1);
    if passo + 1 == total {
        e.feito = true;
        relatorio(e);
    }
}

fn oleo() -> bool {
    std::env::var("PH2D_FPS_OLEO").as_deref() != Ok("0")
}

fn relatorio(e: &Estado) {
    let nome = ["sem real", "real oil"][usize::from(oleo())];
    eprintln!(
        "[fps] === tela {}x{}, {nome}: período p50 (ms) por rodada ===",
        tela_do_fps()[0],
        tela_do_fps()[1]
    );
    for (f, (fase, ..)) in FASES.iter().enumerate() {
        let r: Vec<String> = e.tabela[f].iter().map(|x| format!("{:.1}", x[0])).collect();
        eprintln!("[fps] {nome} {fase:<9}| {}", r.join(" · "));
    }
}
