//! ⭐ **A tira de Frames, só o cromo em cartão**: os pintores de sempre dentro do cartão único, com os
//! mesmos ids, e a tira continua a caber na banda. Provado pelo gesto real nos rects pintados.

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{ferramenta, scroll_bar_of};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerEvent, PointerKind, PointerSource};
use ph2d_panel_flip_frames::state::FlipStripState;
use ph2d_panel_flip_frames::{
    FlipCell, FlipFramesPanel, FlipStripIntent, FlipStripSnapshot, drain_flip_strip_intents, ids,
    set_current_flip_strip,
};
use ph2d_ui_testkit::MockPanelHost;

/// A janela das fotos: o painel fica na banda que o layout lhe dá, no encaixe de baixo.
const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1930.0,
    h: 1040.0,
};

fn tira() -> FlipStripSnapshot {
    let cell = |key: i32| FlipCell {
        key,
        exposure: 4,
        breakdown: false,
        instanced: false,
        selected: false,
        pinned: false,
        weight: 1.0,
    };
    FlipStripSnapshot {
        has_layer: true,
        layer_name: "Ink".to_owned(),
        cells: vec![cell(0), cell(4), cell(8)],
        current_key: Some(0),
        ..Default::default()
    }
}

struct Palco {
    host: MockPanelHost,
    st: FlipStripState,
    rects: Vec<(NodeId, Rect)>,
}

impl Palco {
    fn novo(em_cartoes: bool) -> Self {
        ferramenta::force_cards_on_this_thread(Some(em_cartoes));
        set_current_flip_strip(tira());
        let _ = drain_flip_strip_intents();
        let mut p = Self {
            host: MockPanelHost::with_panel::<FlipFramesPanel>(),
            st: FlipStripState::default(),
            rects: Vec::new(),
        };
        p.assenta();
        p
    }

    fn assenta(&mut self) {
        for _ in 0..60 {
            let (h, st) = (&mut self.host, &mut self.st);
            self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || h.paint::<FlipFramesPanel>(st, JANELA),
            );
        }
    }

    fn rect(&self, id: NodeId) -> Option<Rect> {
        self.rects
            .iter()
            .rev()
            .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| *r)
    }

    fn clica(&mut self, id: NodeId) -> Vec<EditorAction> {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("{id:?} não pintado"));
        let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(self.host.hit_at(x, y), Some(id), "outro id no centro");
        let ev = self.host.click_at(x, y);
        for e in ev {
            self.host
                .apply_panel_event::<FlipFramesPanel>(&mut self.st, e);
        }
        self.host.drained_actions()
    }
}

const BOTOES: [NodeId; 17] = [
    ids::FLIP_PREV_DRAWING,
    ids::FLIP_PLAY,
    ids::FLIP_NEXT_DRAWING,
    ids::FLIP_GHOST,
    ids::FLIP_KEY_PIN,
    ids::FLIP_AUTOKEY,
    ids::FLIP_FALLOFF,
    ids::FLIP_ADDITIVE,
    ids::FLIP_KEY_ADD,
    ids::FLIP_KEY_DUP,
    ids::FLIP_KEY_INSTANCE,
    ids::FLIP_KEY_UNLINK,
    ids::FLIP_KEY_LEFT,
    ids::FLIP_KEY_DELETE,
    ids::FLIP_TWEEN_FADE,
    ids::FLIP_TWEEN_PAIRS,
    ids::FLIP_TWEEN_ADD,
];

/// ⭐ Todo botão da barra responde no cartão, com o mesmo id. *Mutação: tirar o `paint_toolbar::paint` de
/// `cards::paint` ⇒ «não pintado».*
#[test]
fn every_toolbar_button_answers_a_real_click_in_the_card() {
    for id in BOTOES {
        let mut p = Palco::novo(true);
        let a = p.clica(id);
        let i = drain_flip_strip_intents();
        assert!(!a.is_empty() || !i.is_empty(), "{id:?} não pediu nada");
    }
}

/// ⭐⭐ A tira cabe na banda: nenhuma célula nem campo sai dela, e a lista não rola. Mede o que o cromo
/// custa à tira, contra o corpo de antes. *A régua da decisão «só o cromo»: `novo` e `antes` na mensagem.*
#[test]
fn the_strip_still_fits_the_band_and_the_list_does_not_scroll() {
    let antes = Palco::novo(false)
        .rect(ids::flip_cell_id(0))
        .expect("antes")
        .h;
    let p = Palco::novo(true);
    let c = p
        .rect(ids::flip_cell_id(0))
        .expect("a célula pinta-se no cartão");
    let banda = p
        .host
        .store()
        .panel_rect(ph2d_editor_core::ids::FLIP_STRIP_PANEL)
        .expect("a lista publica o rect");
    assert!(
        c.y + c.h <= banda.y + banda.h,
        "a célula sai da banda: {c:?} {banda:?}"
    );
    assert!(
        c.y > banda.y + m::LIST_HEAD_H,
        "por cima da cabeça da lista: {c:?}"
    );
    eprintln!(
        "MEDIDA tira: antes {antes} · no cartão {} · banda {}",
        c.h, banda.h
    );
    assert!(
        p.rect(scroll_bar_of(ph2d_editor_core::ids::FLIP_STRIP_PANEL))
            .is_none(),
        "a lista rola: o cromo não bate certo"
    );
    assert!(
        c.h >= antes * 0.75,
        "a tira perdeu demais: {} contra {antes} de antes",
        c.h
    );
}

/// ⭐ Arrastar a célula dentro do cartão pede mover a chave: o gesto da tira não passou pelo cartão.
#[test]
fn dragging_a_cell_in_the_card_still_moves_its_key() {
    let mut p = Palco::novo(true);
    let c = p.rect(ids::flip_cell_id(1)).expect("a célula");
    let (x, y) = (c.x + c.w * 0.5, c.y + c.h * 0.5);
    let at = |x: f32, kind| PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: ph2d_host::PointerButton::Primary,
        timestamp_ns: 0,
    };
    p.host.dispatch_pointer_event(at(x, PointerKind::Down));
    p.host
        .dispatch_pointer_event(at(x + c.w, PointerKind::Move));
    p.host.dispatch_pointer_event(at(x + c.w, PointerKind::Up));
    p.assenta();
    let i = drain_flip_strip_intents();
    assert!(
        matches!(i.as_slice(), [FlipStripIntent::MoveKey { from: 4, to }] if *to > 4),
        "{i:?}"
    );
}

/// O «×» da lista fecha como o de antes; a lista diz a camada que se anima.
#[test]
fn the_list_close_works() {
    let mut p = Palco::novo(true);
    let a = p.clica(ids::FLIP_STRIP_CLOSE);
    let fechou = !p.host.panel_visible("flip_frames")
        || a.iter()
            .any(|x| matches!(x, EditorAction::CancelActiveTool));
    assert!(fechou, "{a:?}");
}

/// `PH2D_CARDS=0`: o corpo de antes, sem o cartão.
#[test]
fn cards_off_paints_the_old_body() {
    let p = Palco::novo(false);
    assert!(p.rect(ids::FLIP_STRIP_SEC).is_none());
    assert!(p.rect(ids::FLIP_PLAY).is_some());
}
