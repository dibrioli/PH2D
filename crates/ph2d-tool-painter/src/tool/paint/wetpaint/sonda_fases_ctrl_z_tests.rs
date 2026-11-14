//! **Sonda (ignorada): as fases do Ctrl+Z/redo da tinta viva a 3840×2160** — pela porta do app (`undo_last` /
//! `redo_last`), cada fase pelo cronómetro de [`super::undo_vivo::fases`]; undo e redo intercalados, um quadro entre
//! eles, o resumo é o MÍNIMO (a mediana ao lado). Medição, nunca barra.
use super::agua_sobre_oleo_tests::{PAPEL, ponteiro, preset};
use super::undo_tinta_morre_tests::{AGUADA, DABBER};
use super::undo_vivo::fases;
use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase};
use std::collections::BTreeMap;
use std::time::Instant;

const W: usize = 3840;
const H: usize = 2160;

/// A tela como no app (as razões de grade de fábrica), Wet com o relógio fixo.
fn tela(meio: &str, tamanho: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", PAPEL[0], PAPEL[1], PAPEL[2]),
    ));
    t.preset_aplicar(&preset(meio)).expect("o meio");
    t.set_brush_size_px(tamanho);
    t.set_brush_color_srgb8([40, 90, 220]);
    t
}

/// Um traço de lado a lado: 40 movimentos, um quadro cada, e dois depois do Up.
fn traco(t: &mut PainterTool, y: f32) {
    let (x0, x1) = (200.0, W as f32 - 200.0);
    t.on_canvas_pointer(ponteiro(x0, y, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(
            x0 + (x1 - x0) * k as f32 / 40.0,
            y,
            PointerPhase::Move,
        ));
        t.ticks_em_blocos(1);
    }
    t.on_canvas_pointer(ponteiro(x1, y, PointerPhase::Up));
    t.ticks_em_blocos(2);
}

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg").unwrap_or_default()
}

fn min_med(v: &mut [f64]) -> (f64, f64) {
    v.sort_by(f64::total_cmp);
    (v.first().copied().unwrap_or(0.0), v[v.len() / 2])
}

#[test]
#[ignore = "sonda: --profile smoke --ignored --nocapture --test-threads=1"]
fn sonda_as_fases_do_ctrl_z_wet_a_4k() {
    let rodadas: usize = std::env::var("PH2D_SONDA_RODADAS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7);
    eprintln!("carga antes: {}", carga().trim());
    for (meio, tam) in [(AGUADA, 240.0), (DABBER, 120.0)] {
        let mut t = tela(meio, tam);
        for i in 0..3 {
            traco(&mut t, 400.0 + 600.0 * i as f32);
        }
        t.wet_bring_home();
        let mut ordem: Vec<&'static str> = Vec::new();
        let mut tempos: BTreeMap<(&'static str, bool), Vec<f64>> = BTreeMap::new();
        for _ in 0..rodadas {
            for refaz in [false, true] {
                fases::comeca();
                let a = Instant::now();
                let feito = if refaz { t.redo_last() } else { t.undo_last() };
                fases::marca("pintor depois do volta");
                let total = a.elapsed().as_secs_f64() * 1e3;
                assert!(feito, "refaz = {refaz}");
                let f = fases::tira();
                assert!(
                    f.iter().any(|e| e.0 == "volta: repoe"),
                    "o undo vivo não correu (refaz = {refaz})"
                );
                for (nome, d) in f.into_iter().chain([("TOTAL", std::time::Duration::ZERO)]) {
                    if !ordem.contains(&nome) {
                        ordem.push(nome);
                    }
                    let ms = if nome == "TOTAL" {
                        total
                    } else {
                        d.as_secs_f64() * 1e3
                    };
                    tempos.entry((nome, refaz)).or_default().push(ms);
                }
                t.ticks_em_blocos(1);
            }
        }
        // O censo do registo trocado (somado nas rodadas): o plano que já é igual à grade não precisava de troca.
        let censo = ph2d_wet_paint::sonda_fases::censo();
        let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
        eprintln!(
            "== {W}×{H} {meio} tamanho {tam}: grade {}×{} · {rodadas} rodadas · mínimo / mediana (ms)",
            s.grid.0, s.grid.1
        );
        eprintln!(
            "{:<38} {:>9} {:>9} {:>9} {:>9}",
            "fase", "undo min", "undo med", "redo min", "redo med"
        );
        for nome in ordem {
            let mut u = tempos.remove(&(nome, false)).unwrap_or_default();
            let mut r = tempos.remove(&(nome, true)).unwrap_or_default();
            let (um, ud) = if u.is_empty() {
                (0.0, 0.0)
            } else {
                min_med(&mut u)
            };
            let (rm, rd) = if r.is_empty() {
                (0.0, 0.0)
            } else {
                min_med(&mut r)
            };
            eprintln!("{nome:<38} {um:>9.2} {ud:>9.2} {rm:>9.2} {rd:>9.2}");
        }
        eprintln!(
            "censo (undo e redo somados): plano · tiles iguais ao bit à grade / tiles com o plano"
        );
        for (plano, iguais, tem) in censo {
            eprintln!(
                "  {plano:<10} {iguais:>7} / {tem:<7} ({:.0} %)",
                100.0 * iguais as f64 / tem.max(1) as f64
            );
        }
    }
    eprintln!("carga depois: {}", carga().trim());
}
