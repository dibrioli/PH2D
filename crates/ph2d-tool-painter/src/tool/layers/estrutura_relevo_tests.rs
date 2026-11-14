//! n.º 8 item 3 — apagar, duplicar e agrupar uma camada de Impasto: a figura muda só em tinta ∪ corpo (altura,
//! cobertura, material) da camada, com 1 texel de moldura (a diferença central da luz). O cache fica fora dessa janela e
//! a figura é, bit a bit, a da recomposição inteira (undo/redo também) — com e sem o papel texturizado, com o relevo nas
//! duas camadas, só na do gesto (tirar o ÚLTIMO relevo) e com uma altura numa orla de alfa 0 longe da tinta (o caso que
//! só a janela do corpo cobre). O controlo (`TROCA_RECOMPOE_SEMPRE`) recompõe tudo.

use super::super::*;
use super::cache::TROCA_RECOMPOE_SEMPRE;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase, RasterEditTool};
use std::sync::Arc;

const LADO: u32 = 48;
const N: usize = (LADO as usize) * (LADO as usize);
/// O texel da orla: alfa 0 na camada, altura posta à mão, longe de toda tinta (os traços vão de x 6 a 42).
const ORLA: (usize, usize) = (45, 45);

fn cp(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn traco(t: &mut PainterTool, y: f32) {
    t.set_brush_size_px(6.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    t.on_canvas_pointer(cp(6.0, y, PointerPhase::Down));
    for k in 1..=9 {
        t.on_canvas_pointer(cp(6.0 + 4.0 * k as f32, y, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(42.0, y, PointerPhase::Up));
    t.on_tick(1000.0 / 60.0);
}

fn na_tela(t: &mut PainterTool) -> Vec<u8> {
    match t.take_preview_arc() {
        Some((a, _, _)) => a.as_ref().clone(),
        None => t.composited.as_ref().expect("a figura").as_ref().clone(),
    }
}

fn confere(t: &mut PainterTool, caso: &str) {
    let mostrada = na_tela(t);
    t.pede_a_composicao_inteira();
    let certa = t.take_preview_arc().expect("a inteira").0.as_ref().clone();
    assert!(
        mostrada == certa,
        "{caso}: a figura difere da recomposição inteira em {} bytes",
        mostrada.iter().zip(&certa).filter(|(x, y)| x != y).count()
    );
}

/// O documento: A (pincel ou Impasto) e B (Impasto), com ou sem papel; B com uma altura na orla.
fn doc(papel: bool, a_relevo: bool, orla: bool) -> (PainterTool, RtLayerId, RtLayerId) {
    let mut t = PainterTool::default();
    if papel {
        t.agenda_papel_texturizado(7);
        t.abre_a_sprite_nova(7, vec![255u8; N * 4], LADO, LADO);
    } else {
        t.set_source(vec![255u8; N * 4], LADO, LADO);
    }
    if !t.brush_settings().impasto_show {
        t.toggle_impasto_show();
    }
    let a = t.add_raster_layer("A").expect("A");
    t.set_paint_tool_mode("brush");
    if a_relevo {
        t.set_paint_media(PaintMedia::Impasto);
    }
    traco(&mut t, 12.0);
    let b = t.add_raster_layer("B").expect("B");
    t.set_paint_media(PaintMedia::Impasto);
    traco(&mut t, 24.0);
    assert!(t.heights.contains_key(&b), "controlo: B tem relevo");
    t.select_layer(a);
    if orla {
        let i = ORLA.1 * LADO as usize + ORLA.0;
        assert_eq!(
            t.images.get(&b).expect("B fora da tela").rgba8[i * 4 + 3],
            0,
            "controlo: a orla é alfa 0"
        );
        Arc::make_mut(t.heights.get_mut(&b).expect("a altura de B"))[i] = 6.0;
        t.pede_a_composicao_inteira();
    }
    let _ = na_tela(&mut t);
    (t, a, b)
}

#[test]
fn apagar_duplicar_e_agrupar_o_impasto_recompoem_so_tinta_e_corpo() {
    type Gesto = fn(&mut PainterTool, RtLayerId);
    let gestos: [(&str, Gesto); 3] = [
        ("apagar", |t, b| assert!(t.delete_layer(b))),
        ("duplicar", |t, b| {
            t.duplicate_layer(b).expect("a cópia");
        }),
        ("grupo", |t, b| {
            t.select_layer(b);
            t.group_active().expect("o grupo");
        }),
    ];
    for (gesto, faz) in gestos {
        for papel in [false, true] {
            for (caso, a_relevo, orla) in [
                ("relevo nas duas", true, false),
                ("o último relevo", false, false),
                ("orla de relevo", true, true),
            ] {
                for antes in [false, true] {
                    let (mut t, _, b) = doc(papel, a_relevo, orla);
                    let rotulo = format!("{gesto} · papel={papel} · {caso} · controlo={antes}");
                    TROCA_RECOMPOE_SEMPRE.with(|x| x.set(antes));
                    faz(&mut t, b);
                    TROCA_RECOMPOE_SEMPRE.with(|x| x.set(false));
                    assert_eq!(t.composited_is_some(), !antes, "{rotulo}: o cache");
                    if !antes {
                        let r = t.dirty_rect.expect("a janela");
                        assert!(r.w < LADO || r.h < LADO, "{rotulo}: a tela inteira ({r:?})");
                    }
                    confere(&mut t, &rotulo);
                    assert!(t.undo_last(), "{rotulo}: desfaz");
                    confere(&mut t, &format!("{rotulo} · undo"));
                    assert!(t.redo_last(), "{rotulo}: refaz");
                    confere(&mut t, &format!("{rotulo} · redo"));
                }
            }
        }
    }
}

/// n.º 10 (B18.6): o caso da matriz do app — o gesto sobre a camada de Impasto que está ACTIVA (a nova onde se
/// pintou), não sobre uma de baixo. O contador é o próprio alcance: `Some` antes do gesto (o caminho confinado), e a
/// figura ao bit da recomposição inteira (undo/redo também).
#[test]
fn apagar_e_duplicar_o_impasto_activo_recompoem_so_tinta_e_corpo() {
    type Gesto = fn(&mut PainterTool, RtLayerId);
    let gestos: [(&str, Gesto); 2] = [
        ("apagar", |t, b| assert!(t.delete_layer(b))),
        ("duplicar", |t, b| {
            t.duplicate_layer(b).expect("a cópia");
        }),
    ];
    for (gesto, faz) in gestos {
        for papel in [false, true] {
            for a_relevo in [false, true] {
                let (mut t, _, b) = doc(papel, a_relevo, false);
                t.select_layer(b);
                let _ = na_tela(&mut t);
                let rotulo = format!("{gesto} · papel={papel} · a_relevo={a_relevo} · B activa");
                assert_eq!(t.layers.active(), Some(b), "{rotulo}: controlo, B activa");
                assert!(
                    t.alcance_da_tinta_de(b).is_some(),
                    "{rotulo}: o alcance da camada de Impasto activa não é provado (None = recompõe tudo)"
                );
                faz(&mut t, b);
                assert!(t.composited_is_some(), "{rotulo}: o cache");
                let r = t.dirty_rect.expect("a janela");
                assert!(r.w < LADO || r.h < LADO, "{rotulo}: a tela inteira ({r:?})");
                confere(&mut t, &rotulo);
                assert!(t.undo_last(), "{rotulo}: desfaz");
                confere(&mut t, &format!("{rotulo} · undo"));
                assert!(t.redo_last(), "{rotulo}: refaz");
                confere(&mut t, &format!("{rotulo} · redo"));
            }
        }
    }
}
