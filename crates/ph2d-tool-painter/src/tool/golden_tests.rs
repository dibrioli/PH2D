//! Golden-image stroke harness — the end-to-end verification that was missing.
//!
//! Por que este arquivo existe: o "compila mas nada funciona" do painter vinha
//! de testes que provavam `apply_ui_edit ↔ snapshot` ecoarem um valor enquanto
//! NENHUM teste dirigia `begin_stroke → queue_pointer → end_stroke` e olhava os
//! pixels pintados. Aqui pintamos um traço REAL pela API pública de stroke e
//! afirmamos assinaturas perceptuais (depósito, uniformidade da espinha / sem
//! scallop). Ver docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md §4.
//!
//! Estes testes pegam classes de bug que unit-test de wiring não pega:
//! Dilution-na-taxa (scallop), depósito morto, cobertura que não satura.

use super::*;
use crate::params::OklchColor;

/// Flat RGBA source canvas.
fn flat(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    for _ in 0..(w * h) {
        v.extend_from_slice(&rgba);
    }
    v
}

/// Rec.601-ish luma over straight sRGB bytes — só usado para ripple RELATIVO,
/// não para colorimetria. Suficiente para medir "quão escuro" + "quanto oscila".
#[inline]
fn luma(px: &[u8], idx: usize) -> f32 {
    0.299 * px[idx] as f32 + 0.587 * px[idx + 1] as f32 + 0.114 * px[idx + 2] as f32
}

/// Um traço wash opaco sobre branco DEVE escurecer o centro (o depósito
/// aconteceu de verdade) e manter o fundo opaco. Pega "nada foi pintado".
#[test]
fn golden_wash_stroke_deposits_color_at_center() {
    let mut t = PainterTool::default();
    t.params.size_px = 16.0;
    t.params.opacity = 1.0;
    t.params.active_color = OklchColor {
        l: 0.55,
        c: 0.18,
        h: 0.7,
        a: 1.0,
    };
    t.set_source(flat(32, 32, [255, 255, 255, 255]), 32, 32);
    let _ = t.current_preview(); // drena o dirty do set_source

    t.begin_stroke(7);
    t.queue_pointer(PointerSample {
        position: [16.0, 16.0],
        pressure: 1.0,
        tilt: 0.0,
    });
    t.end_stroke();

    let (px, w, _h) = t.current_preview().expect("traço deve marcar dirty");
    let c = ((16 * w + 16) * 4) as usize;
    let l = luma(px, c);
    assert!(l < 245.0, "centro não recebeu depósito: luma={l:.1}");
    assert_eq!(px[c + 3], 255, "fundo branco permanece opaco");
}

/// O sintoma EXATO do Enio: "dilution alto + charge alto parece spacing alto"
/// — um ripple periódico de luminância ao longo da espinha porque o depósito
/// nunca saturava (Dilution estava dobrada na TAXA de depósito). O fix moveu
/// Dilution para o cap de opacidade → a cobertura satura → espinha lisa.
///
/// Este teste falha se isso regredir (o ripple volta a ser uma fração grande
/// da profundidade do depósito). Ver 05_auditoria_algoritmos_wet_mix.md §5.
#[test]
fn golden_high_dilution_high_charge_stroke_has_no_scallop() {
    let (w, h) = (72u32, 24u32);
    let mut t = PainterTool::default();
    t.params.size_px = 12.0;
    t.params.opacity = 1.0;
    t.params.active_color = OklchColor {
        l: 0.45,
        c: 0.16,
        h: 0.9,
        a: 1.0,
    };
    t.set_source(flat(w, h, [255, 255, 255, 255]), w, h);
    let _ = t.current_preview();

    // Wet Mix ON, dilution alto + charge(load) alto, Pull/grain/jitter OFF —
    // assim a ÚNICA coisa que poderia ondular a espinha é a estrutura de dabs.
    t.brush.wet_mix.wet_mix_enabled = true;
    t.brush.wet_mix.dilution = 0.5;
    t.brush.wet_mix.load = 0.9; // "Charge"
    t.brush.wet_mix.pull = 0.0;
    t.brush.wet_mix.wetness_jitter = 0.0;
    t.brush.grain.grain_depth = 0.0;
    t.cached_brush_hash = None;

    let cy = (h / 2) as f32;
    t.begin_stroke(11);
    for x in 8..(w - 8) {
        t.queue_pointer(PointerSample {
            position: [x as f32, cy],
            pressure: 1.0,
            tilt: 0.0,
        });
    }
    t.end_stroke();

    let (px, pw, _ph) = t.current_preview().expect("traço deve marcar dirty");
    let row = h / 2;
    let (mut lo, mut hi, mut sum, mut n) = (f32::MAX, f32::MIN, 0.0f32, 0usize);
    for x in 16..(pw - 16) {
        let idx = ((row * pw + x) * 4) as usize;
        let l = luma(px, idx);
        lo = lo.min(l);
        hi = hi.max(l);
        sum += l;
        n += 1;
    }
    let mean = sum / n as f32;
    let depth = 255.0 - mean; // quanto o traço escureceu o branco
    let ripple = hi - lo; // oscilação de luminância ao longo da espinha
    eprintln!(
        "SCALLOP spine mean={mean:.1} depth={depth:.1} ripple={ripple:.1} ratio={:.3}",
        ripple / depth.max(1.0)
    );
    assert!(
        depth > 8.0,
        "traço fraco demais p/ julgar (depth={depth:.1})"
    );
    // Wash com cobertura saturada ondula pouco vs. sua profundidade; o bug
    // da-taxa fazia ripple ≈ depth (vales voltavam ao branco) → ratio ~1.0.
    assert!(
        ripple < 0.5 * depth,
        "scallop: ripple {ripple:.1} ≥ 0.5·depth {depth:.1} (cobertura não saturou)"
    );
}

/// SPIKE de viabilidade (Track C / ADR-0098): custo CPU de pintar num canvas 4K
/// com brushes pequeno→grande. É O número que decide se a migração foundational
/// p/ canvas GPU-residente (destravaria 4K/brush grande) se justifica, ou se o
/// CPU-first (ADR-0097) aguenta. Mede o caminho quente real begin→queue→end
/// (scheduler + apply_stamps_wash + composite trivial). #[ignore] + --release
/// (dev=opt0 mente, ver feedback_measure_perf_symptom_scale).
///
/// `cargo test --release -p ph2d-tool-painter spike_cpu_stroke_cost_4k -- --ignored --nocapture`
#[test]
#[ignore = "spike de perf — rode com --release --ignored"]
fn spike_cpu_stroke_cost_4k() {
    use std::time::Instant;
    let (w, h) = (4096u32, 4096u32);
    eprintln!("== SPIKE canvas {w}x{h} (CPU apply_stamps_wash, caminho vivo) ==");
    for size in [64.0f32, 256.0, 1024.0, 2048.0] {
        let mut t = PainterTool::default();
        t.params.size_px = size;
        t.params.opacity = 1.0;
        t.params.active_color = OklchColor {
            l: 0.45,
            c: 0.16,
            h: 0.9,
            a: 1.0,
        };
        t.set_source(flat(w, h, [255, 255, 255, 255]), w, h);
        let _ = t.current_preview();

        // Um traço curto no centro ≈ alguns frames de input p/ um brush grande.
        let (cx, cy) = ((w / 2) as f32, (h / 2) as f32);
        let len = (size * 2.0).clamp(96.0, 1024.0);
        let steps = 24;
        let t0 = Instant::now();
        t.begin_stroke(99);
        for i in 0..steps {
            let dx = (i as f32 / steps as f32) * len;
            t.queue_pointer(PointerSample {
                position: [cx + dx, cy],
                pressure: 1.0,
                tilt: 0.0,
            });
        }
        t.end_stroke();
        let dt = t0.elapsed().as_secs_f64() * 1e3;
        let _ = t.current_preview();
        eprintln!(
            "  brush={size:>6.0}px  stroke({steps} samples / {len:.0}px)  total={dt:>8.2}ms  ({:>6.2}ms/sample)",
            dt / steps as f64
        );
    }
    eprintln!(
        "Budget: 60fps=16.7ms/frame, 30fps=33.3ms. 1 frame ≈ 1-3 dabs. Se brush grande >> budget ⇒ GPU-residência justificada; senão CPU-first basta."
    );
}
