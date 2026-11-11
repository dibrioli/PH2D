//! **Arrastar a letra de um campo muda o valor** (dono 07/11: «as caixas de texto ainda estão sem a
//! capacidade de modificar arrastando o mouse sobre a letra»). A lei é a do campo de cartão.

use super::*;
use ph2d_editor_core::interaction::drag::NUMBER_INPUT_DRAG_THRESHOLD_PX;

/// A mão sobre a letra do campo `r`.
fn letra(r: Rect) -> (f32, f32) {
    centre(doc::Layout::handle(r))
}

fn mover(m: &mut Mesa, (x, y): (f32, f32), mods: Modifiers) {
    let e = m.e;
    let mut ts = std::mem::replace(&mut m.ts, TextSystem::without_system_fonts());
    let input = PointerInput {
        text: &mut ts,
        mods,
        now_ns: 0,
        primary_held: true,
    };
    m.docs().pointer_move(e, input, x, y);
    m.ts = ts;
}

/// Down na letra e o limiar passado: daqui em diante cada pixel conta.
fn pega(m: &mut Mesa, r: Rect) -> (f32, f32) {
    let (x, y) = letra(r);
    m.pointer(PointerKind::Down, (x, y));
    let x = x + NUMBER_INPUT_DRAG_THRESHOLD_PX;
    mover(m, (x, y), Modifiers::default());
    assert_eq!(m.pending(), [], "o limiar não salta o valor");
    (x, y)
}

/// *Mutação: o `Drag::Scrub` não emitir nada no `motion`, ou o clique sem limiar não abrir o campo.*
#[test]
fn arrastar_a_letra_de_cols_muda_a_grelha_e_clicar_nela_abre_a_escrita() {
    let mut m = Mesa::new();
    let (c, r) = (m.doc().snap.hframes, m.doc().snap.vframes);
    let cols = m.doc().layout.cols;
    let (x, y) = pega(&mut m, cols);
    mover(&mut m, (x + 3.0, y), Modifiers::default()); // LITERAL-PX-OK: três pixels
    m.pointer(PointerKind::Up, (x + 3.0, y)); // LITERAL-PX-OK: três pixels
    assert_eq!(m.pending(), [AnimFieldEdit::Grid(c + 3, r)]);
    assert!(m.doc().editing.is_none(), "o arrasto não abre a escrita");
    m.frame();
    assert_eq!(m.doc().snap.hframes, c + 3, "chega à cena");
    m.click(letra(cols));
    assert_eq!(m.pending(), [], "o clique não muda o valor");
    assert!(matches!(m.doc().editing, Some((doc::Field::Cols, _))));
}

/// Um quadro por segundo por pixel; Shift ×10; e o mesmo valor não se reenvia.
#[test]
fn frames_por_segundo_andam_um_por_pixel_e_o_shift_dez() {
    let mut m = Mesa::new();
    let fps = m.doc().layout.fps;
    let (x, y) = pega(&mut m, fps);
    mover(&mut m, (x + 2.0, y), Modifiers::default()); // LITERAL-PX-OK: dois pixels
    mover(&mut m, (x + 2.0, y), Modifiers::default()); // LITERAL-PX-OK: dois pixels
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    mover(&mut m, (x + 3.0, y), shift); // LITERAL-PX-OK: mais um pixel
    let doze = ph2d_ecs::ms_of_fps(12.0);
    let vinte_e_dois = ph2d_ecs::ms_of_fps(22.0);
    assert_eq!(
        m.pending(),
        [
            AnimFieldEdit::FrameMs(0, doze),
            AnimFieldEdit::FrameMs(0, vinte_e_dois)
        ]
    );
}

/// O Esc a meio devolve o valor do Down; puxar para lá do fim pára no modelo (nunca 0 colunas).
#[test]
fn esc_devolve_o_valor_e_o_arrasto_para_no_limite() {
    let mut m = Mesa::new();
    let fps = m.doc().layout.fps;
    let (x, y) = pega(&mut m, fps);
    mover(&mut m, (x + 5.0, y), Modifiers::default()); // LITERAL-PX-OK: cinco pixels
    m.key(Key::Escape, None);
    let antes = m.pending();
    assert_eq!(antes.last(), Some(&AnimFieldEdit::FrameMs(0, 100)));
    m.frame();
    let r = m.doc().snap.vframes;
    let cols = m.doc().layout.cols;
    let (x, y) = pega(&mut m, cols);
    mover(&mut m, (x - 500.0, y), Modifiers::default()); // LITERAL-PX-OK: bem para lá do fim
    assert_eq!(m.pending(), [AnimFieldEdit::Grid(1, r)]);
}

/// O multiplicador do quadro escolhido anda 0,1 por pixel.
#[test]
fn o_multiplicador_do_quadro_anda_pela_letra() {
    let mut m = Mesa::new();
    let quadro = centre(m.doc().layout.frames[1].0);
    m.click(quadro);
    m.frame();
    let this = m.doc().layout.this_frame;
    let (x, y) = pega(&mut m, this);
    mover(&mut m, (x + 5.0, y), Modifiers::default()); // LITERAL-PX-OK: cinco pixels
    assert_eq!(m.pending(), [AnimFieldEdit::FrameMsAt(0, 1, 150)]);
}

/// Dono, 07/11: «a seta do hover não apareceu». A ↔ sobre a letra e durante o arrasto, nunca no valor.
/// *Mutação: `pointer_look` devolver sempre `None`.*
#[test]
fn a_letra_mostra_a_seta_de_arrastar() {
    use ph2d_editor_core::doc_surface::PointerLook;
    let mut m = Mesa::new();
    let e = m.e;
    let l = m.doc().layout.clone();
    for r in [l.fps, l.cols, l.rows] {
        let (x, y) = letra(r);
        assert_eq!(m.docs().pointer_look(e, x, y), Some(PointerLook::ResizeEw));
        let valor = (r.x + r.w - 2.0, y); // LITERAL-PX-OK: dentro da borda direita
        assert_eq!(m.docs().pointer_look(e, valor.0, valor.1), None);
    }
    let (x, y) = letra(l.this_frame);
    assert_eq!(
        m.docs().pointer_look(e, x, y),
        None,
        "sem quadro escolhido não arrasta"
    );
    let (x, y) = pega(&mut m, l.cols);
    let longe = (x + 200.0, y + 200.0); // LITERAL-PX-OK: a mão saiu da letra
    mover(&mut m, longe, Modifiers::default());
    assert_eq!(
        m.docs().pointer_look(e, longe.0, longe.1),
        Some(PointerLook::ResizeEw),
        "a seta segue durante o arrasto"
    );
}
