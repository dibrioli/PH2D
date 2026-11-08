//! **P12 H: a Undo History pelo caminho real** (Aseprite O11–O14, R2): três traços com Pencil, Line e
//! Paint Bucket e um comando de menu viram linhas com o nome do que o artista fez; a janela pede
//! `GoTo(n)` pelo canal dela e o quadro do app anda até lá; um traço novo corta os desfeitos.

use ph2d_app_pixel::Tool;
use ph2d_app_pixel::compose::Mods;
use ph2d_editor_core::pixel_panel::{PixelEditAction as E, PixelLayerAction as L, PixelRequest};
use ph2d_panel_pixel_history::canal::{self, HistoryRequest};
use ph2d_pixel_core::Sprite;
use ph2d_pixel_core::StepState::{Done, Undone};
use winit::keyboard::KeyCode;

use crate::edit::born;
use crate::selecao::Palco;

/// Um Pixel Sprite nascido pelo Add (história vazia), em Draw, com o quadro do painel.
fn palco() -> Palco {
    let (app, sim, cam, bits) = born();
    let key = ph2d_app_pixel::input::key_of(sim.world(), bits).expect("marcador");
    let mut hero = ph2d_editor_core::screens::hero::HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo
        .mode
        .enter(bits, ph2d_editor_core::object_mode::ObjectMode::Draw);
    let mut p = Palco {
        app,
        sim,
        cam,
        hero,
        toasts: ph2d_editor_core::ToastQueue::new(),
        bits,
        key,
    };
    p.quadro();
    p
}

const NADA: Mods = Mods {
    alt: false,
    shift: false,
    ctrl: false,
};

fn rotulos(p: &mut Palco) -> Vec<&'static str> {
    p.quadro();
    canal::view().rows.iter().map(|r| r.label).collect()
}

fn traco(p: &mut Palco, t: Tool, a: (i32, i32), b: (i32, i32)) -> Sprite {
    p.ferramenta(t);
    p.arrasta(a, b, NADA);
    if p.app.live_shape().is_some() {
        p.tecla(KeyCode::Enter, false, false);
    }
    p.sprite().clone()
}

fn com_tres_tracos_e_uma_camada() -> (Palco, Sprite) {
    let mut p = palco();
    assert_eq!(
        rotulos(&mut p),
        ["pixel.history.initial"],
        "o palco nasce sem história"
    );
    p.app.fg = [200, 30, 30, 255];
    let primeiro = traco(&mut p, Tool::Pencil, (2, 2), (8, 2));
    p.app.fg = [30, 200, 30, 255];
    traco(&mut p, Tool::Line, (2, 6), (12, 9));
    p.app.fg = [30, 30, 200, 255];
    traco(&mut p, Tool::Bucket, (3, 14), (3, 14));
    p.pede(PixelRequest::LayerAction(L::NewLayer));
    (p, primeiro)
}

#[test]
fn history_rows_name_each_step_like_aseprite() {
    let (mut p, _) = com_tres_tracos_e_uma_camada();
    assert_eq!(
        rotulos(&mut p),
        [
            "pixel.history.initial",
            "pixel.history.pencil",
            "pixel.history.line",
            "pixel.history.paint_bucket",
            "pixel.keys.layer.new",
        ]
    );
    let palavras: Vec<_> = canal::view()
        .rows
        .iter()
        .map(|r| ph2d_i18n::tr(r.label))
        .collect();
    assert_eq!(
        palavras,
        [
            "Initial State",
            "Pencil Tool",
            "Line Tool",
            "Paint Bucket Tool",
            "New Layer"
        ]
    );
    let v = canal::view();
    assert_eq!(v.current, 4);
    assert!(v.bytes > 0);
    let titulo = canal::title(v.bytes);
    assert!(
        titulo.starts_with("Undo History (") && titulo.ends_with(')'),
        "{titulo}"
    );
}

#[test]
fn history_goto_through_the_window_channel_walks_the_document() {
    let (mut p, primeiro) = com_tres_tracos_e_uma_camada();
    let camada_do_traco = p.sprite().layers()[0].id;
    p.quadro();
    let _ = canal::take_requests();
    // O que a linha 1 da janela faz ao clique (`event::apply_event`), e o quadro do app a encaminha.
    canal::request(HistoryRequest::GoTo(1));
    p.quadro();
    // O `next_id` não recua (ids não se reusam, como no Aseprite): compara-se o que se vê.
    let (s, k) = (
        p.sprite(),
        (primeiro.layers()[0].id, primeiro.frames()[0].id),
    );
    assert_eq!(
        s.layers(),
        primeiro.layers(),
        "as camadas do primeiro traço"
    );
    assert_eq!(s.frames(), primeiro.frames());
    assert_eq!(s.cel(k), primeiro.cel(k), "as células do primeiro traço");
    let v = canal::view();
    assert_eq!(v.current, 1);
    let estados: Vec<_> = v.rows.iter().map(|r| r.state).collect();
    assert_eq!(estados, [Done, Done, Undone, Undone, Undone]);
    let d = p.app.doc(p.key).expect("doc");
    assert_eq!(
        d.layer, camada_do_traco,
        "a camada activa é a que o traço tocou"
    );
    assert_eq!(d.frame, p.sprite().frames()[0].id);

    traco(&mut p, Tool::Pencil, (4, 12), (9, 12));
    assert_eq!(
        rotulos(&mut p),
        [
            "pixel.history.initial",
            "pixel.history.pencil",
            "pixel.history.pencil",
        ],
        "o traço novo corta os desfeitos (R2)"
    );
}

#[test]
fn edit_menu_rows_open_undo_history_and_keyboard_shortcuts() {
    use ph2d_panel_pixel_tools::ids::EDIT_ACTIONS;
    let linha = |e: E| {
        EDIT_ACTIONS
            .iter()
            .find(|(_, r, _)| *r == PixelRequest::Edit(e))
            .map(|(_, r, nome)| (r.clone(), *nome))
            .expect("a linha do menu Edit")
    };
    let (historia, nome) = linha(E::UndoHistory);
    assert_eq!(ph2d_i18n::tr(nome), "Undo History");
    let (atalhos, nome) = linha(E::KeyboardShortcuts);
    assert_eq!(ph2d_i18n::tr(nome), "Keyboard Shortcuts…");

    // O pedido da linha do menu, pelo quadro do app (o clique do botão -> pedido mede-se no
    // `ph2d-panel-pixel-tools`).
    let mut p = palco();
    assert!(!canal::abrir_pendente());
    p.pede(historia);
    assert!(canal::abrir_pendente(), "Edit ▸ Undo History pede a janela");
    // A janela dos atalhos não expõe o pedido pendente (`ph2d-panel-pixel-keys`, outra linha).
    p.pede(atalhos);
    assert!(
        p.app.pending().is_empty(),
        "o quadro consumiu Keyboard Shortcuts…"
    );
}
