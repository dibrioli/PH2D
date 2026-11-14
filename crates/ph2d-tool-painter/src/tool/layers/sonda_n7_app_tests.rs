//! N7 onda 4: a cena da matriz do APP a 3840×2160 (Textured Paper, traço na base, «+», traço na nova) e os gestos
//! de camada com as fases cronometradas — o que o gesto paga além da recomposição. `--ignored --nocapture`,
//! `--profile smoke`.

use super::super::*;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};

const W: u32 = 3840;
const H: u32 = 2160;

fn ms(t0: std::time::Instant) -> f64 {
    t0.elapsed().as_secs_f64() * 1e3
}

fn cp(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn traco(t: &mut PainterTool, y: f32) {
    t.sonda_raio(24.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    t.on_canvas_pointer(cp(600.0, y, PointerPhase::Down));
    for k in 1..=30 {
        t.on_canvas_pointer(cp(600.0 + 30.0 * k as f32, y, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(1500.0, y, PointerPhase::Up));
    t.on_tick(1000.0 / 60.0);
}

fn cena(modo: &str) -> (PainterTool, RtLayerId, RtLayerId) {
    let mut t = PainterTool::default();
    t.agenda_papel_texturizado(7);
    t.abre_a_sprite_nova(7, vec![255u8; (W * H * 4) as usize], W, H);
    t.set_wet_relogio_fixo(true);
    t.set_paint_tool_mode(modo);
    let base = t.layers.active().expect("a base");
    traco(&mut t, 300.0);
    let nova = t.add_raster_layer("Nova").expect("+");
    traco(&mut t, 600.0);
    t.select_layer(base);
    traco(&mut t, 900.0);
    t.select_layer(nova);
    traco(&mut t, 1200.0);
    let _ = t.take_preview_arc();
    (t, base, nova)
}

/// A simulação viva do app marca o próprio sujo a ~40 imagens/s: no clique há quase sempre um sujo pendente.
fn sujo_da_simulacao(t: &mut PainterTool) {
    t.mark_dirty(crate::compositor::Region {
        x: 600,
        y: 1180,
        w: 900,
        h: 40,
    });
}

fn gesto(t: &mut PainterTool, nome: &str, f: impl FnOnce(&mut PainterTool)) {
    let t0 = std::time::Instant::now();
    f(t);
    let g = ms(t0);
    let ficou = t.composited_is_some() || !t.gpu_lane_stale;
    let r = t.dirty_rect;
    let t0 = std::time::Instant::now();
    let _ = t.take_preview_arc();
    let q = ms(t0);
    println!(
        "[N7 app 4K] {nome:<28} gesto {g:6.2} ms | drenagem {q:6.2} ms | cache ficou={ficou} sujo={r:?}"
    );
}

#[test]
#[ignore = "sonda de tempo"]
fn sonda_gestos_de_camada_na_cena_do_app_a_4k() {
    let _ = crate::raio_da_sonda::perfil_da_sonda();
    for modo in ["brush", "wetpaint"] {
        for pendente in [false, true] {
            let (mut t, base, nova) = cena(modo);
            println!(
                "--- {modo} sujo pendente={pendente} viva={} papel_atravessa_vidro={}",
                t.tinta_viva(),
                t.papel_atravessa_vidro()
            );
            let p = |t: &mut PainterTool| {
                if pendente {
                    sujo_da_simulacao(t)
                }
            };
            p(&mut t);
            gesto(&mut t, "trocar", |t| t.select_layer(base));
            p(&mut t);
            gesto(&mut t, "voltar", |t| t.select_layer(nova));
            p(&mut t);
            let t0 = std::time::Instant::now();
            let s = t.snapshot_model();
            let snap = ms(t0);
            drop(s);
            let t0 = std::time::Instant::now();
            let al = t.alcance_da_tinta_de(nova);
            let alc = ms(t0);
            println!("    snapshot_model {snap:.2} ms · alcance_da_tinta_de {alc:.2} ms → {al:?}");
            let mut dup = None;
            gesto(&mut t, "duplicar", |t| dup = t.duplicate_layer(nova));
            p(&mut t);
            gesto(&mut t, "apagar", |t| {
                assert!(t.delete_layer(dup.expect("dup")))
            });
            p(&mut t);
            gesto(&mut t, "grupo", |t| {
                t.group_active().expect("grupo");
            });
        }
    }
}
