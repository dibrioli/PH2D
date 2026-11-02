//! **P8 pelo caminho real** (`docs/PixelArt/P8_plano.md` §4): Edit ▸ Fill (`F`), Stroke (`S`),
//! Invert…, FX ▸ Outline… (`Shift+O`) e Despeckle… sobre a cena 9, pelo [`panel::frame`] — o
//! pedido do menu/tecla, a janelinha com Preview, o OK do diálogo (clique e Enter). A pergunta de
//! cada teste: *o que o canvas mostra, o que o documento grava, e é UM passo?*

use ph2d_app_pixel::Tool;
use ph2d_app_pixel::compose::Mods;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{ContextMenuKind, WidgetEvent};
use ph2d_editor_core::pixel_panel::{PixelDialogKind, PixelEditAction as E, PixelRequest};
use ph2d_pixel_core::selection::Mask;
use winit::keyboard::KeyCode;

use crate::selecao::Palco;

const RED: [u8; 4] = [255, 0, 0, 255];
const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];

/// Um palco limpo com um quadrado vermelho opaco em `(100..=104, 40..=44)` sobre uma volta
/// transparente, e sem selecção.
fn com_quadrado() -> Palco {
    let mut p = Palco::limpo();
    p.app.fg = RED;
    p.ferramenta(Tool::RectMarquee);
    // A volta do quadrado limpa-se primeiro (Delete), para o contorno só ver o quadrado.
    p.arrasta((90, 30), (114, 54), Mods::default());
    p.tecla(KeyCode::Delete, false, false);
    assert_eq!(p.rgba(99, 39), TRANSPARENT, "a volta limpa");
    p.arrasta((100, 40), (104, 44), Mods::default());
    p.tecla(KeyCode::KeyF, false, false);
    p.pede(PixelRequest::Select(
        ph2d_editor_core::pixel_panel::PixelSelectAction::Deselect,
    ));
    p
}

fn aberto(p: &Palco) -> Option<PixelDialogKind> {
    let open = p
        .hero
        .store
        .context_menu()
        .is_some_and(|r| r.kind == ContextMenuKind::PixelEditDialog);
    open.then(|| p.hero.store.pixel_edit_dialog().kind)
}

/// O clique real num controlo do diálogo, e o quadro seguinte da família.
fn carrega(p: &mut Palco, id: ph2d_editor_core::NodeId) {
    p.hero.apply_event(WidgetEvent::Click(id));
    p.quadro();
}

fn mostrado(p: &mut Palco, c: (i32, i32)) -> [u8; 4] {
    let key = p.key;
    p.app.shown_rgba(key, c.0, c.1).expect("doc")
}

#[test]
fn fill_and_stroke_paint_the_selection_and_its_inner_edge_as_one_step() {
    let mut p = Palco::limpo();
    p.app.fg = RED;
    p.ferramenta(Tool::RectMarquee);
    p.arrasta((2, 2), (8, 6), Mods::default());
    let m = p.mascara().expect("selecção");
    p.um_passo_que_desfaz("Fill (F)", |p| p.tecla(KeyCode::KeyF, false, false));
    p.tecla(KeyCode::KeyF, false, false);
    for (x, y) in m.cells() {
        assert_eq!(p.rgba(x, y), RED, "Fill: ({x}, {y})");
    }
    p.tecla(KeyCode::KeyZ, true, false);
    p.um_passo_que_desfaz("Stroke (S)", |p| p.tecla(KeyCode::KeyS, false, false));
    let antes = p.sprite().clone();
    p.tecla(KeyCode::KeyS, false, false);
    let borda = m.edge4();
    for (x, y) in m.cells() {
        let quer = if borda.get(x, y) {
            RED
        } else {
            let d = &antes;
            d.rgba_of_value(d.value_at((d.layers()[0].id, d.frames()[0].id), x, y))
        };
        assert_eq!(p.rgba(x, y), quer, "Stroke: ({x}, {y})");
    }
    assert!(!borda.get(5, 4), "o miolo não é borda");
    // O menu Edit pede o mesmo (o botão emite este pedido).
    p.tecla(KeyCode::KeyZ, true, false);
    p.pede(PixelRequest::Edit(E::Fill));
    assert_eq!(p.rgba(5, 4), RED);
}

#[test]
fn without_a_selection_fill_and_stroke_do_nothing() {
    let mut p = Palco::limpo();
    p.app.fg = RED;
    let antes = p.sprite().clone();
    let n = p.passos();
    p.tecla(KeyCode::KeyF, false, false);
    p.tecla(KeyCode::KeyS, false, false);
    assert_eq!(*p.sprite(), antes);
    assert_eq!(p.passos(), n);
}

#[test]
fn invert_previews_without_writing_and_ok_is_one_step_that_undo_restores() {
    let mut p = com_quadrado();
    let antes = p.sprite().clone();
    let n = p.passos();
    p.pede(PixelRequest::Edit(E::Invert));
    assert_eq!(aberto(&p), Some(PixelDialogKind::Invert));
    // Preview ligado por omissão: o canvas mostra o invertido, o documento não muda.
    assert_eq!(mostrado(&mut p, (102, 42)), [0, 255, 255, 255]);
    assert_eq!(*p.sprite(), antes, "o Preview não grava");
    // Um canal desligado (G) volta ao original no que se mostra.
    carrega(&mut p, ids::PIXEL_FILTER_CHANNELS[1].1);
    assert_eq!(mostrado(&mut p, (102, 42)), [0, 0, 255, 255]);
    // Preview desligado: o canvas mostra o documento.
    carrega(&mut p, ids::PIXEL_FILTER_PREVIEW);
    assert_eq!(mostrado(&mut p, (102, 42)), RED);
    carrega(&mut p, ids::PIXEL_FILTER_PREVIEW);
    carrega(&mut p, ids::PIXEL_EDIT_OK);
    assert_eq!(aberto(&p), None);
    assert_eq!(p.rgba(102, 42), [0, 0, 255, 255], "o OK grava o que se via");
    assert_eq!(p.passos(), n + 1, "UM passo");
    p.tecla(KeyCode::KeyZ, true, false);
    assert_eq!(*p.sprite(), antes, "um Ctrl+Z desfaz o filtro inteiro");
}

#[test]
fn cancel_and_esc_leave_the_document_byte_for_byte() {
    for fecha in ["cancel", "esc"] {
        let mut p = com_quadrado();
        let antes = p.sprite().clone();
        let n = p.passos();
        p.tecla(KeyCode::KeyO, false, true);
        assert_eq!(aberto(&p), Some(PixelDialogKind::Outline), "Shift+O");
        assert_eq!(
            mostrado(&mut p, (99, 42)),
            RED,
            "o contorno pré-visualizado"
        );
        if fecha == "cancel" {
            carrega(&mut p, ids::PIXEL_EDIT_CANCEL);
        } else {
            // O que o Esc do diálogo faz (`dispatch/key.rs`; a tecla prova-se no editor-core).
            p.hero.store.close_context_menu();
            p.quadro();
        }
        assert_eq!(aberto(&p), None, "{fecha} fecha");
        assert_eq!(*p.sprite(), antes, "{fecha}: o documento ao byte");
        assert_eq!(p.passos(), n, "{fecha}: nenhum passo");
        assert_eq!(
            mostrado(&mut p, (99, 42)),
            TRANSPARENT,
            "{fecha}: a pré-visualização sai"
        );
    }
}

#[test]
fn outline_ok_by_enter_draws_the_circle_ring_outside_with_the_foreground_color() {
    let mut p = com_quadrado();
    let antes = p.sprite().clone();
    p.app.fg = [0, 0, 255, 255];
    p.pede(PixelRequest::Edit(E::Outline));
    // O que o Enter do diálogo faz (`dispatch/key.rs`; a tecla prova-se no editor-core).
    p.hero.store.request_pixel_edit();
    p.quadro();
    assert_eq!(aberto(&p), None, "o Enter é o OK");
    // Círculo (4 vizinhos), fora: o anel cola nos lados, os cantos diagonais ficam vazios.
    assert_eq!(p.rgba(99, 42), [0, 0, 255, 255]);
    assert_eq!(p.rgba(102, 45), [0, 0, 255, 255]);
    assert_eq!(p.rgba(99, 39), TRANSPARENT, "canto diagonal");
    assert_eq!(p.rgba(102, 42), RED, "o miolo fica");
    p.tecla(KeyCode::KeyZ, true, false);
    assert_eq!(*p.sprite(), antes, "um Ctrl+Z desfaz o contorno inteiro");
}

#[test]
fn despeckle_from_the_menu_removes_a_lonely_cell_only_inside_the_selection() {
    let mut p = Palco::limpo();
    p.app.fg = RED;
    p.ferramenta(Tool::Pencil);
    for c in [(30, 30), (40, 30)] {
        p.press(c, Mods::default());
        p.release();
    }
    assert_eq!(p.rgba(30, 30), RED, "o lápis pôs o ponto");
    p.ferramenta(Tool::RectMarquee);
    p.arrasta((25, 25), (35, 35), Mods::default());
    assert!(
        p.mascara()
            .is_some_and(|m: Mask| m.get(30, 30) && !m.get(40, 30))
    );
    p.pede(PixelRequest::Edit(E::Despeckle));
    assert_eq!(aberto(&p), Some(PixelDialogKind::Despeckle));
    assert_eq!(
        mostrado(&mut p, (30, 30)),
        TRANSPARENT,
        "a mediana 3×3 apaga o ponto solto"
    );
    p.um_passo_que_desfaz("Despeckle", |p| carrega(p, ids::PIXEL_EDIT_OK));
    p.pede(PixelRequest::Edit(E::Despeckle));
    carrega(&mut p, ids::PIXEL_EDIT_OK);
    assert_eq!(p.rgba(30, 30), TRANSPARENT);
    assert_eq!(p.rgba(40, 30), RED, "fora da selecção nada muda");
}
