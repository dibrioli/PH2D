//! **P8 pelo caminho real** (`docs/PixelArt/P8_plano.md` §4): Edit ▸ Fill (`F`), Stroke (`S`),
//! Invert…, FX ▸ Outline… (`Shift+O`) e Despeckle… sobre a cena 9, pelo [`panel::frame`] — o
//! pedido do menu/tecla, a janelinha com Preview, o OK do diálogo (clique e Enter). A pergunta de
//! cada teste: *o que o canvas mostra, o que o documento grava, e é UM passo?*

use ph2d_app_pixel::Tool;
use ph2d_app_pixel::compose::Mods;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{ContextMenuKind, WidgetEvent};
use ph2d_editor_core::pixel_panel::{
    PixelDialogKind, PixelEditAction as E, PixelOutlineBg, PixelOutlineMatrix, PixelRequest,
};
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
    // Em tempo real: o canvas mostra o invertido, o documento não muda.
    assert_eq!(mostrado(&mut p, (102, 42)), [0, 255, 255, 255]);
    assert_eq!(*p.sprite(), antes, "o efeito ao vivo não grava");
    // Um canal desligado (G) volta ao original no que se mostra.
    carrega(&mut p, ids::PIXEL_FILTER_CHANNELS[1].1);
    assert_eq!(mostrado(&mut p, (102, 42)), [0, 0, 255, 255]);
    carrega(&mut p, ids::PIXEL_EDIT_OK);
    assert_eq!(aberto(&p), None);
    assert_eq!(p.rgba(102, 42), [0, 0, 255, 255], "o OK grava o que se via");
    assert_eq!(p.passos(), n + 1, "UM passo");
    p.tecla(KeyCode::KeyZ, true, false);
    assert_eq!(*p.sprite(), antes, "um Ctrl+Z desfaz o filtro inteiro");
}

/// Dono, 05/11: «os ajustes não funcionam em tempo real». A vista só recompõe quando o carimbo do
/// documento muda; `mostrado` lê o traço direto e passava com a tela parada.
#[test]
fn each_change_of_an_open_filter_tells_the_canvas_to_redraw() {
    let mut p = com_quadrado();
    let key = p.key;
    p.pede(PixelRequest::Edit(E::HueSaturation));
    p.quadro();
    let aberto = p.app.doc(key).map(|d| d.stamp);
    p.hero.store.set_number_value(ids::PIXEL_FILTER_HUE, 40.0);
    p.quadro();
    let mudou = p.app.doc(key).map(|d| d.stamp);
    assert_ne!(mudou, aberto, "o valor novo chega à tela");
    p.quadro();
    assert_eq!(
        p.app.doc(key).map(|d| d.stamp),
        mudou,
        "sem mudança, sem recompor"
    );
    carrega(&mut p, ids::PIXEL_EDIT_CANCEL);
    p.quadro();
    assert_ne!(
        p.app.doc(key).map(|d| d.stamp),
        mudou,
        "o Cancel tira o efeito da tela"
    );
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

/// O OK do Outline depois de clicar `escolhas` no diálogo.
fn outline_com(p: &mut Palco, escolhas: &[ph2d_editor_core::NodeId]) {
    p.pede(PixelRequest::Edit(E::Outline));
    for &id in escolhas {
        carrega(p, id);
    }
    carrega(p, ids::PIXEL_EDIT_OK);
    assert_eq!(aberto(p), None, "o OK fechou o diálogo");
}

fn id_do_fundo(b: PixelOutlineBg) -> ph2d_editor_core::NodeId {
    ids::PIXEL_FILTER_BACKGROUNDS
        .iter()
        .find(|e| e.0 == b)
        .expect("fundo")
        .1
}

#[test]
fn outline_with_a_background_colour_takes_the_background_swatch_as_the_background() {
    let mut p = com_quadrado();
    let antes = p.sprite().clone();
    p.app.fg = [0, 0, 255, 255];
    p.app.bg = RED;
    outline_com(&mut p, &[id_do_fundo(PixelOutlineBg::BackgroundColor)]);
    // Fundo = a cor de fundo (vermelho): o «fora» pinta as células VERMELHAS coladas ao que não é
    // vermelho — a borda do quadrado, não a volta transparente.
    assert_eq!(
        p.rgba(100, 42),
        [0, 0, 255, 255],
        "a borda vermelha ganha a cor"
    );
    assert_eq!(p.rgba(102, 42), RED, "o miolo não tem vizinho de outra cor");
    assert_eq!(p.rgba(99, 42), TRANSPARENT, "a volta transparente fica");
    p.tecla(KeyCode::KeyZ, true, false);
    assert_eq!(*p.sprite(), antes);
}

#[test]
fn outline_square_matrix_reaches_the_diagonal_corner_the_circle_leaves_empty() {
    let mut p = com_quadrado();
    let quadrada = ids::PIXEL_FILTER_MATRICES
        .iter()
        .find(|e| e.0 == PixelOutlineMatrix::Square)
        .expect("matriz")
        .1;
    p.app.fg = [0, 0, 255, 255];
    outline_com(&mut p, &[quadrada]);
    assert_eq!(p.rgba(99, 39), [0, 0, 255, 255], "o canto diagonal");
    assert_eq!(p.rgba(98, 39), TRANSPARENT, "a 2.ª volta não");
}

#[test]
fn outline_horizontal_and_vertical_presets_reach_only_their_own_axis() {
    for (preset, lado, cima) in [
        (
            PixelOutlineMatrix::Horizontal,
            [0, 0, 255, 255],
            TRANSPARENT,
        ),
        (PixelOutlineMatrix::Vertical, TRANSPARENT, [0, 0, 255, 255]),
    ] {
        let mut p = com_quadrado();
        let id = ids::PIXEL_FILTER_MATRICES
            .iter()
            .find(|e| e.0 == preset)
            .expect("matriz")
            .1;
        p.app.fg = [0, 0, 255, 255];
        outline_com(&mut p, &[id]);
        assert_eq!(p.rgba(99, 42), lado, "{preset:?}: ao lado do quadrado");
        assert_eq!(p.rgba(102, 39), cima, "{preset:?}: em cima do quadrado");
    }
}

#[test]
fn outline_tiled_wraps_around_the_sprite_edge() {
    let mut p = Palco::limpo();
    p.app.fg = RED;
    p.ferramenta(Tool::RectMarquee);
    p.pede(PixelRequest::Select(
        ph2d_editor_core::pixel_panel::PixelSelectAction::All,
    ));
    p.tecla(KeyCode::Delete, false, false);
    p.pede(PixelRequest::Select(
        ph2d_editor_core::pixel_panel::PixelSelectAction::Deselect,
    ));
    p.arrasta((0, 0), (1, 1), Mods::default());
    p.tecla(KeyCode::KeyF, false, false);
    p.pede(PixelRequest::Select(
        ph2d_editor_core::pixel_panel::PixelSelectAction::Deselect,
    ));
    assert_eq!(p.rgba(0, 0), RED);
    let (w, h) = (p.sprite().width() as i32, p.sprite().height() as i32);
    p.app.fg = [0, 0, 255, 255];
    let mut solto = Vec::new();
    outline_com(&mut p, &[]);
    solto.push((p.rgba(w - 1, 0), p.rgba(0, h - 1), p.rgba(2, 0)));
    p.tecla(KeyCode::KeyZ, true, false);
    outline_com(&mut p, &[ids::PIXEL_FILTER_TILED]);
    assert_eq!(
        solto[0],
        (TRANSPARENT, TRANSPARENT, [0, 0, 255, 255]),
        "sem Tiled"
    );
    assert_eq!(p.rgba(w - 1, 0), [0, 0, 255, 255], "Tiled: a volta do x");
    assert_eq!(p.rgba(0, h - 1), [0, 0, 255, 255], "Tiled: a volta do y");
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
