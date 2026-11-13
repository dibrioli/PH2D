//! **O vidro da aguada segue a base que o borrão move** (B18.4, Watercolor Basic a 4K): com Smudge, o borrão
//! escreve a base no lugar e o `make_mut` move-a para um `Arc` novo; o `Weak` do vidro congelado morria, o
//! vidro recongelava a partir do plano JÁ escrito e copiava-o inteiro em todo quadro (58 MB a 3840×2160).
//! O oráculo é o caminho de antes (`VIDRO_SEGUE_A_BASE = false`) no mesmo processo.

use super::watercolor_vidro::VIDRO_SEGUE_A_BASE;
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use std::sync::Arc;

const W: u32 = 256;
const H: u32 = 160;

fn cp(x: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, 80.0],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um traço com Smudge: os píxeis da tela depois de cada tique e quantas vezes o plano do vidro mudou de `Arc`.
fn traco(segue: bool) -> (Vec<Vec<u8>>, usize, usize) {
    VIDRO_SEGUE_A_BASE.set(segue);
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::Watercolor);
    crate::tool::paint::tests::aquarela_neutra(&mut t);
    crate::tool::paint::tests::watercolor_basic_de_antes(&mut t);
    t.set_brush_size_px(10.0);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.paint.brush.wet_smudge = 0.4;
    let mut telas = Vec::new();
    let (mut planos, mut trocas) = (None, 0usize);
    let mut ev = |t: &mut PainterTool, x: f32, phase| {
        t.on_canvas_pointer(cp(x, phase));
        t.on_tick(1000.0 / 60.0);
        telas.push(t.canvas_rgba.to_vec());
        let p = t
            .layers
            .active()
            .and_then(|c| t.vidros.get(&c))
            .map(Arc::as_ptr);
        if p.is_some() && planos.is_some() && p != planos {
            trocas += 1;
        }
        planos = p;
    };
    ev(&mut t, 30.0, PointerPhase::Down);
    let moves = 14;
    for k in 1..=moves {
        ev(&mut t, 30.0 + 12.0 * k as f32, PointerPhase::Move);
    }
    VIDRO_SEGUE_A_BASE.set(true);
    (telas, trocas, moves)
}

#[test]
fn o_vidro_segue_a_base_que_o_borrao_move_sem_copiar_o_plano() {
    let (antes, trocas_antes, moves) = traco(false);
    let (cura, trocas_cura, _) = traco(true);
    assert!(
        trocas_antes >= moves - 2,
        "controlo: o caminho de antes recopia o vidro em quase todo quadro ({trocas_antes} de {moves})"
    );
    assert!(
        trocas_cura <= 1,
        "o plano do vidro foi recopiado {trocas_cura} vezes num traço (só o 1.º quadro o pode copiar)"
    );
    assert_eq!(
        antes, cura,
        "os píxeis da tela são os mesmos nos dois caminhos (o vidro só pesa no papel de cor)"
    );
}

/// O 1.º traço de uma camada sem vidro não COPIA o plano que acabou de nascer zerado (a 4K, 58 MB tocados no
/// 1.º toque); o 2.º traço copia (o desfazer segura o plano): a régua vê a cópia quando ela existe.
#[test]
fn o_primeiro_traco_da_camada_nao_copia_o_vidro_que_nasce_zerado() {
    use super::watercolor_vidro::COPIAS_DO_VIDRO;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::Watercolor);
    crate::tool::paint::tests::aquarela_neutra(&mut t);
    crate::tool::paint::tests::watercolor_basic_de_antes(&mut t);
    t.set_brush_size_px(10.0);
    let mut traco = |t: &mut PainterTool, y: f32| {
        COPIAS_DO_VIDRO.set(0);
        let p = |x: f32, phase| CanvasPointer {
            pos: [x, y],
            ..cp(x, phase)
        };
        t.on_canvas_pointer(p(30.0, PointerPhase::Down));
        t.on_tick(1000.0 / 60.0);
        for k in 1..=6 {
            t.on_canvas_pointer(p(30.0 + 12.0 * k as f32, PointerPhase::Move));
            t.on_tick(1000.0 / 60.0);
        }
        t.on_canvas_pointer(p(110.0, PointerPhase::Up));
        for _ in 0..3 {
            t.on_tick(1000.0 / 60.0);
        }
        COPIAS_DO_VIDRO.get()
    };
    let primeiro = traco(&mut t, 50.0);
    let segundo = traco(&mut t, 110.0);
    assert!(
        segundo >= 1,
        "controlo: o 2.º traço copia o vidro que o desfazer segura ({segundo})"
    );
    assert_eq!(
        primeiro, 0,
        "o 1.º traço copiou o vidro recém-nascido {primeiro} vez(es)"
    );
}
