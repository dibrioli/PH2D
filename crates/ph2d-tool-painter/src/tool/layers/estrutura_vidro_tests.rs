//! n.º 8 (B18.6) — apagar, duplicar e agrupar com a aguada no papel de cor: a figura muda só em tinta ∪ vidro da
//! camada do gesto. O cache fica fora dessa janela e a figura é, bit a bit, a da recomposição inteira (undo/redo
//! também), com o vidro nas duas camadas, só na do gesto (tirar o ÚLTIMO vidro) e com um texel de vidro numa orla de
//! alfa 0 longe da tinta — o caso que só a janela do vidro cobre. O controlo (`TROCA_RECOMPOE_SEMPRE`) recompõe tudo.

use super::super::*;
use super::cache::TROCA_RECOMPOE_SEMPRE;
use crate::compositor::vidro::sela;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};
use std::sync::Arc;

const LADO: u32 = 48;
const N: usize = (LADO as usize) * (LADO as usize);
/// O texel da orla: alfa 0 na camada, vidro selado, longe de toda tinta (os traços vão de x 6 a 42).
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

/// O documento: papel de cor, A (pincel ou aguada) e B (aguada), seca; B com um texel de vidro na orla.
fn doc(a_aguada: bool, orla: bool) -> (PainterTool, RtLayerId, RtLayerId) {
    let mut t = PainterTool::default();
    t.agenda_papel_texturizado(7);
    t.abre_a_sprite_nova(7, vec![255u8; N * 4], LADO, LADO);
    let a = t.add_raster_layer("A").expect("A");
    t.set_paint_tool_mode("brush");
    if a_aguada {
        t.set_paint_media(PaintMedia::Watercolor);
        crate::tool::paint::tests::aquarela_neutra(&mut t);
    }
    traco(&mut t, 12.0);
    t.dry_session_now();
    let b = t.add_raster_layer("B").expect("B");
    t.set_paint_media(PaintMedia::Watercolor);
    crate::tool::paint::tests::aquarela_neutra(&mut t);
    traco(&mut t, 24.0);
    t.dry_session_now();
    if orla {
        t.select_layer(a);
        let i = ORLA.1 * LADO as usize + ORLA.0;
        let px = &t.images.get(&b).expect("B fora da tela").rgba8;
        assert_eq!(px[i * 4 + 3], 0, "controlo: a orla é alfa 0");
        let selo = [px[i * 4], px[i * 4 + 1], px[i * 4 + 2], 0];
        let plano = t.vidros.get_mut(&b).expect("o vidro de B");
        Arc::make_mut(plano)[i] = sela(selo, [220, 30, 30]);
        t.pede_a_composicao_inteira();
    }
    assert!(
        t.papel_atravessa_vidro(),
        "controlo: papel de cor com vidro"
    );
    t.select_layer(a);
    let _ = na_tela(&mut t);
    (t, a, b)
}

#[test]
fn apagar_duplicar_e_agrupar_com_a_aguada_no_papel_de_cor_recompoem_so_tinta_e_vidro() {
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
        for (caso, a_aguada, orla) in [
            ("vidro nas duas", true, false),
            ("o último vidro", false, false),
            ("orla de vidro", true, true),
        ] {
            for antes in [false, true] {
                let (mut t, _, b) = doc(a_aguada, orla);
                let rotulo = format!("{gesto} · {caso} · controlo={antes}");
                TROCA_RECOMPOE_SEMPRE.with(|x| x.set(antes));
                faz(&mut t, b);
                TROCA_RECOMPOE_SEMPRE.with(|x| x.set(false));
                assert_eq!(t.composited_is_some(), !antes, "{rotulo}: o cache");
                if !antes {
                    let r = t.dirty_rect.expect("a janela");
                    assert!(r.w < LADO || r.h < LADO, "{rotulo}: a tela inteira ({r:?})");
                    if orla {
                        let dentro = |(x, y): (usize, usize)| {
                            (r.x as usize..(r.x + r.w) as usize).contains(&x)
                                && (r.y as usize..(r.y + r.h) as usize).contains(&y)
                        };
                        assert!(dentro(ORLA), "{rotulo}: a orla fora da janela ({r:?})");
                    }
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
