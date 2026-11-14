//! n.º 10 F5 — a troca PURA com a tinta viva do Wet (sem traço por drenar) não muda píxel nenhum da figura: adormecer
//! e acordar não escrevem píxeis. A sonda (64², com e sem papel) mediu 0 píxeis numa janela marcada de 64×44.

use super::super::*;
use super::cache::TINTA_MARCA_SEMPRE;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase, RasterEditTool};

const LADO: u32 = 64;
const N: usize = (LADO as usize) * (LADO as usize);

fn cp(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn traco(t: &mut PainterTool, y: f32) {
    t.set_brush_size_px(8.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    t.on_canvas_pointer(cp(8.0, y, PointerPhase::Down));
    for k in 1..=10 {
        t.on_canvas_pointer(cp(8.0 + 4.5 * k as f32, y, PointerPhase::Move));
        ph2d_editor_core::Tool::on_tick(t, 1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(54.0, y, PointerPhase::Up));
    ph2d_editor_core::Tool::on_tick(t, 1000.0 / 60.0);
}

fn inteira(t: &mut PainterTool) -> Vec<u8> {
    t.pede_a_composicao_inteira();
    t.take_preview_arc().expect("inteira").0.as_ref().clone()
}

/// **A troca pura com a tinta viva não marca nada e a figura mostrada é a da recomposição inteira**, ao bit, em cada
/// passo (com e sem papel), e o desfazer/refazer também. Controlo (`TINTA_MARCA_SEMPRE`, o caminho de antes): marca a
/// janela da tinta e dá a mesma figura.
#[test]
fn a_troca_pura_com_a_tinta_viva_nao_marca_nada_e_a_figura_fica() {
    for (papel, antes) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; N * 4], LADO, LADO);
        t.set_wet_relogio_fixo(true);
        let a = t.add_raster_layer("A").expect("A");
        let b = t.add_raster_layer("B").expect("B");
        if papel {
            t.set_substrate_depth(1.0);
        }
        t.set_paint_tool_mode("wetpaint");
        t.select_layer(a);
        traco(&mut t, 20.0);
        t.select_layer(b);
        traco(&mut t, 40.0);
        for (k, alvo) in [a, b, a, b].into_iter().enumerate() {
            let caso = format!("papel={papel} antes={antes} passo {k}");
            let figura = inteira(&mut t);
            assert!(t.tinta_viva(), "{caso}: controlo, viva");
            TINTA_MARCA_SEMPRE.with(|x| x.set(antes));
            t.select_layer(alvo);
            TINTA_MARCA_SEMPRE.with(|x| x.set(false));
            assert!(t.composited_is_some(), "{caso}: o cache fica");
            if antes {
                assert!(
                    t.dirty_rect.is_some(),
                    "{caso}: controlo, o de antes marca a janela"
                );
            } else {
                assert_eq!(t.dirty_rect, None, "{caso}: nada a marcar");
            }
            let mostrada = match t.take_preview_arc() {
                Some((x, _, _)) => x.as_ref().clone(),
                None => t.composited.as_ref().expect("figura").as_ref().clone(),
            };
            assert!(mostrada == figura, "{caso}: a figura mostrada mudou");
            assert!(
                mostrada == inteira(&mut t),
                "{caso}: a figura difere da recomposição inteira"
            );
            for (rotulo, ok) in [("undo", t.undo_last()), ("redo", t.redo_last())] {
                assert!(ok, "{caso}: {rotulo}");
                let m = match t.take_preview_arc() {
                    Some((x, _, _)) => x.as_ref().clone(),
                    None => t.composited.as_ref().expect("figura").as_ref().clone(),
                };
                assert!(
                    m == inteira(&mut t),
                    "{caso} · {rotulo}: a figura difere da recomposição inteira"
                );
            }
        }
    }
}
