//! ⭐ **A tira de Frames em cartões**, o método da Timeline: moldura sem cartão, as peças da lista, os
//! mesmos ids, e a célula com a altura de antes. Provado pelo gesto real nos rects pintados.

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{ferramenta, scroll_bar_of};
use ph2d_editor_core::tool::PanelEvent;
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

const NUMEROS: [NodeId; 5] = [
    ids::FLIP_FPS_NUM,
    ids::FLIP_GHOST_BEFORE_NUM,
    ids::FLIP_GHOST_AFTER_NUM,
    ids::FLIP_HOLD_NUM,
    ids::FLIP_TWEEN_NUM,
];

const LISTAS: [NodeId; 2] = [ids::FLIP_CYCLE_DD, ids::FLIP_TWEEN_EASE_DD];

const BOTOES: [NodeId; 18] = [
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
    ids::FLIP_KEY_RIGHT,
    ids::FLIP_KEY_DELETE,
    ids::FLIP_TWEEN_FADE,
    ids::FLIP_TWEEN_PAIRS,
    ids::FLIP_TWEEN_ADD,
];

/// ⭐ Todo botão da barra responde, com o mesmo id. *Mutação: tirar `barra_cartoes::paint` de `cards::paint`
/// ⇒ «não pintado».*
#[test]
fn every_toolbar_button_answers_a_real_click_in_the_card() {
    for id in BOTOES {
        let mut p = Palco::novo(true);
        let a = p.clica(id);
        let i = drain_flip_strip_intents();
        assert!(!a.is_empty() || !i.is_empty(), "{id:?} não pediu nada");
    }
}

/// ⭐⭐ A tira cabe na banda: nenhuma célula sai dela, a lista não rola, e a moldura sem cartão devolve à
/// célula a altura do painel de antes. *A régua da decisão de `docs/CartoesUI/14`: `novo` e `antes` na mensagem.*
#[test]
fn the_strip_still_fits_the_band_and_the_cell_keeps_its_old_height() {
    let antes = Palco::novo(false)
        .rect(ids::flip_cell_id(0))
        .expect("antes")
        .h;
    let p = Palco::novo(true);
    let c = p
        .rect(ids::flip_cell_id(0))
        .expect("a célula pinta-se na faixa");
    let banda = p
        .host
        .store()
        .panel_rect(ph2d_editor_core::ids::FLIP_STRIP_PANEL)
        .expect("a moldura publica o rect");
    assert!(
        c.y + c.h <= banda.y + banda.h,
        "a célula sai da banda: {c:?} {banda:?}"
    );
    assert!(
        c.y > banda.y + m::LIST_HEAD_H,
        "por cima da cabeça da lista: {c:?}"
    );
    eprintln!(
        "MEDIDA tira: antes {antes} · novo {} · banda {}",
        c.h, banda.h
    );
    assert!(
        p.rect(scroll_bar_of(ph2d_editor_core::ids::FLIP_STRIP_PANEL))
            .is_none(),
        "a lista rola"
    );
    assert!(c.h >= antes, "a tira perdeu altura: {} contra {antes}", c.h);
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

/// Lei 7: toda caixa da barra leva o nome inteiro, e toda peça tem a altura de um campo do cartão.
/// *Mutação: a largura da peça menor ⇒ a marca seguinte encosta ⇒ `sobrepõe`.*
#[test]
fn no_piece_of_the_bar_overlaps_or_leaves_the_field_height() {
    let p = Palco::novo(true);
    let mut pecas: Vec<(NodeId, Rect)> = BOTOES
        .iter()
        .chain(NUMEROS.iter())
        .chain(LISTAS.iter())
        .map(|&id| {
            (
                id,
                p.rect(id).unwrap_or_else(|| panic!("{id:?} não pintado")),
            )
        })
        .collect();
    for (id, r) in &pecas {
        assert!(r.h >= m::FIELD_H - 0.5, "{id:?} baixo: {r:?}");
    }
    pecas.sort_by(|a, b| (a.1.y, a.1.x).partial_cmp(&(b.1.y, b.1.x)).unwrap());
    for par in pecas.windows(2) {
        let (a, b) = (par[0].1, par[1].1);
        let mesma_fila = (a.y - b.y).abs() < m::FIELD_H * 0.5;
        assert!(
            !mesma_fila || a.x + a.w <= b.x + 0.5,
            "sobrepõe: {:?} {a:?} e {:?} {b:?}",
            par[0].0,
            par[1].0
        );
    }
}

/// A escolha das duas listas: abrir, clicar a opção, o pedido é o de sempre, com o id do dono.
/// *Mutação: tirar o `register_if_absent` de `prepara` ⇒ a lista não abre.*
#[test]
fn both_lists_open_and_pick_an_option() {
    for (dd, opcao) in [
        (ids::FLIP_CYCLE_DD, ids::flip_cycle_option_id(2)),
        (ids::FLIP_TWEEN_EASE_DD, ids::flip_tween_ease_option_id(1)),
    ] {
        let mut p = Palco::novo(true);
        p.clica(dd);
        p.assenta();
        let a = p.clica(opcao);
        assert!(
            a.iter().any(|x| matches!(
                x,
                EditorAction::ToolPanelEvent(PanelEvent::SelectOption(id, _)) if *id == dd
            )),
            "{dd:?}: {a:?}"
        );
    }
}

/// O número escrito chega ao barramento com o id dele. *Mutação: tirar o id de `NUMEROS` de `event.rs` ⇒ vermelho.*
#[test]
fn typing_each_number_asks_the_same_as_before() {
    for id in NUMEROS {
        let mut p = Palco::novo(true);
        for e in p.host.type_into_number(id, "3") {
            p.host.apply_panel_event::<FlipFramesPanel>(&mut p.st, e);
        }
        let a = p.host.drained_actions();
        assert!(
            a.iter().any(|x| matches!(
                x,
                EditorAction::ToolPanelEvent(PanelEvent::SetValue(n, v)) if *n == id && (*v - 3.0).abs() < 0.01
            )),
            "{id:?}: {a:?}"
        );
    }
}

/// O scrub na pega pede o novo valor, e o Esc devolve o do documento, no barramento e na caixa.
/// *Mutação: tirar `mark_card_field` de `prepara` ⇒ a pega não arrasta.*
#[test]
fn scrubbing_each_number_writes_it_and_escape_gives_it_back() {
    use ph2d_editor_core::widget::card_field::handle_rect;
    for id in NUMEROS {
        let mut p = Palco::novo(true);
        let antes = p.host.store().number_value(id).expect("o número");
        let h = handle_rect(p.rect(id).expect("o campo"));
        let (x, y) = (h.x + h.w * 0.5, h.y + h.h * 0.5);
        let ev = |x: f32, kind| PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: ph2d_host::PointerButton::Primary,
            timestamp_ns: 0,
        };
        p.host.dispatch_pointer_event(ev(x, PointerKind::Down));
        p.host
            .dispatch_pointer_event(ev(x + 8.0, PointerKind::Move));
        let mudou = p
            .host
            .dispatch_pointer_event(ev(x + 40.0, PointerKind::Move));
        assert!(
            mudou.contains(&WidgetEvent::ValueChanged(id)),
            "{id:?}: {mudou:?}"
        );
        let meio = p.host.store().number_value(id).unwrap();
        assert!((meio - antes).abs() > 0.5, "{id:?} não se moveu");
        let esc = p.host.key_down(0x1B);
        assert!(
            esc.contains(&WidgetEvent::ValueChanged(id)),
            "{id:?}: o Esc não avisou: {esc:?}"
        );
        assert_eq!(p.host.store().number_value(id), Some(antes), "{id:?}");
    }
}

/// O valor do documento aparece na caixa, e nenhum quadro o reescreve durante a escrita.
/// *Mutação: tirar `set_number_value` de `prepara` ⇒ o FPS fica no de fábrica.*
#[test]
fn the_field_shows_the_document_value() {
    let mut p = Palco::novo(true);
    let mut t = tira();
    t.fps = 12.0;
    t.ghost_before = 3;
    t.ghost_after = 2;
    t.tween_count = 5;
    t.cells[0].exposure = 7;
    set_current_flip_strip(t);
    p.assenta();
    for (id, v) in [
        (ids::FLIP_FPS_NUM, 12.0),
        (ids::FLIP_GHOST_BEFORE_NUM, 3.0),
        (ids::FLIP_GHOST_AFTER_NUM, 2.0),
        (ids::FLIP_HOLD_NUM, 7.0),
        (ids::FLIP_TWEEN_NUM, 5.0),
    ] {
        assert_eq!(p.host.store().number_value(id), Some(v), "{id:?}");
    }
}

/// Numa janela estreita a barra desce por grupos, nada se perde e as células continuam abaixo dela.
/// *Mutação: tirar o `desce` de dentro do grupo ⇒ uma peça passa da borda.*
#[test]
fn a_narrow_band_wraps_the_bar_and_loses_no_control() {
    const ESTREITA: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1000.0,
        h: 1040.0,
    };
    let mut p = Palco::novo(true);
    for _ in 0..60 {
        let (h, st) = (&mut p.host, &mut p.st);
        p.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || h.paint::<FlipFramesPanel>(st, ESTREITA),
        );
    }
    let banda = p
        .host
        .store()
        .panel_rect(ph2d_editor_core::ids::FLIP_STRIP_PANEL)
        .expect("o rect");
    let celula = p.rect(ids::flip_cell_id(0));
    for id in BOTOES.iter().chain(NUMEROS.iter()).chain(LISTAS.iter()) {
        let r = p.rect(*id).unwrap_or_else(|| panic!("{id:?} sumiu"));
        assert!(
            r.x + r.w <= banda.x + banda.w,
            "{id:?} sai: {r:?} {banda:?}"
        );
        if let Some(c) = celula {
            assert!(r.y + r.h <= c.y, "{id:?} por cima da célula: {r:?} {c:?}");
        }
    }
}

/// Todo ícone sem nome tem dica. *Mutação: tirar uma linha de `DICAS` ⇒ vermelho nesse id.*
#[test]
fn every_icon_of_the_bar_has_a_tooltip() {
    let p = Palco::novo(true);
    let icones = ph2d_editor_core::panel::cards::dicas::icon_only_ids(
        ph2d_editor_core::ids::FLIP_STRIP_PANEL,
    );
    assert!(icones.contains(&ids::FLIP_PLAY), "{icones:?}");
    for id in icones {
        assert!(p.host.store().tooltip_for(id).is_some(), "{id:?} sem dica");
    }
}
