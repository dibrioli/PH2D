//! ⭐⭐ **O painel Painter em cartões, fatia H1** (`docs/CartoesUI/13_PAINTER_PLANO.md`): a lista, a faixa
//! Brush/Layers, a captura dos corpos por converter e os nativos Mixing, Symmetry e Tiling. Tudo pelo gesto real,
//! nos rects pintados, e o que chega ao barramento é o MESMO do painel de antes.

use super::censo_dos_controlos::Bancada;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{WidgetEvent, dispatch_key};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{dicas, grip_of, list_menu_of};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_painter_layers::cards::force_cards_on_this_thread;
use ph2d_panel_painter_layers::state::{
    PainterLayersPanelState, set_current_brush, set_current_dock_shows_layers,
};
use ph2d_panel_painter_layers::{FALLBACK_BRUSH, PainterLayersPanel};
use ph2d_tool_painter::ids as p;
use ph2d_tool_painter::{BrushSettings, PaintMedia};
use ph2d_ui_testkit::MockPanelHost;
use std::collections::BTreeSet;

/// A coluna mostra o painel INTEIRO sem rolar: no Wet Paint ele passou de 6000 px (B35.13 pôs as 17 cerdas no Shape e
/// o n.º 8 o cartão Performance; o Tiling caía em y 5972), por isso 9000.
const COLUNA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 9000.0,
};

pub(super) const PAINEL: NodeId = core_ids::PAINTER_LAYERS_PANEL;

pub(super) struct Mesa {
    pub(super) host: MockPanelHost,
    pub(super) st: PainterLayersPanelState,
    pub(super) rects: Vec<(NodeId, Rect)>,
    pub(super) janela: Rect,
}

impl Mesa {
    pub(super) fn nova(cartoes: bool, brush: BrushSettings, camadas: bool) -> Self {
        Self::na_janela(cartoes, brush, camadas, COLUNA)
    }

    pub(super) fn na_janela(
        cartoes: bool,
        brush: BrushSettings,
        camadas: bool,
        janela: Rect,
    ) -> Self {
        force_cards_on_this_thread(Some(cartoes));
        set_current_brush(Some(brush));
        set_current_dock_shows_layers(camadas);
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let _ = host.paint::<PainterLayersPanel>(&mut st, janela);
        host.open_all_sections();
        let mut mesa = Self {
            host,
            st,
            rects: Vec::new(),
            janela,
        };
        mesa.assenta();
        mesa
    }

    pub(super) fn assenta(&mut self) {
        for _ in 0..60 {
            self.quadro();
        }
    }

    pub(super) fn quadro(&mut self) {
        let (h, st, j) = (&mut self.host, &mut self.st, self.janela);
        self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || h.paint::<PainterLayersPanel>(st, j),
        );
    }

    pub(super) fn ids(&self) -> BTreeSet<NodeId> {
        self.rects
            .iter()
            .filter(|(_, r)| r.w > 0.0 && r.h > 0.0)
            .map(|(i, _)| *i)
            .collect()
    }

    pub(super) fn rect(&self, id: NodeId) -> Option<Rect> {
        self.rects
            .iter()
            .rev()
            .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| *r)
    }

    pub(super) fn aplica(&mut self, eventos: Vec<WidgetEvent>) -> Vec<String> {
        let _ = self.host.drained_actions();
        for e in eventos {
            self.host
                .apply_panel_event::<PainterLayersPanel>(&mut self.st, e);
        }
        self.host
            .drained_actions()
            .iter()
            .map(|a| format!("{a:?}"))
            .collect()
    }

    pub(super) fn clica(&mut self, id: NodeId) -> Vec<String> {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("{id:?} não pintado"));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(self.host.hit_at(cx, cy), Some(id), "outro id no centro");
        let ev = self.host.click_at(cx, cy);
        self.aplica(ev)
    }

    pub(super) fn ponteiro(&mut self, kind: PointerKind, x: f32, y: f32) -> Vec<WidgetEvent> {
        self.host.dispatch_pointer_event(PointerEvent {
            kind,
            x,
            y,
            button: PointerButton::Primary,
            source: PointerSource::Mouse,
            pressure: 1.0,
            timestamp_ns: 0,
        })
    }

    pub(super) fn arrasta_a_pega(&mut self, id: NodeId, dx: f32) -> Vec<WidgetEvent> {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("{id:?} não pintado"));
        let (x, y) = (r.x + m::ICON_CELL * 0.5, r.y + r.h * 0.5);
        let mut ev = self.ponteiro(PointerKind::Down, x, y);
        ev.extend(self.ponteiro(PointerKind::Move, x + dx * 0.5, y));
        ev.extend(self.ponteiro(PointerKind::Move, x + dx, y));
        ev
    }

    /// Dobra todos os cartões menos estes: numa lista alta, o que passa da janela não regista alvo.
    pub(super) fn so_abertos(&mut self, abertos: &[NodeId]) {
        for s in TODAS {
            self.host
                .store_mut()
                .set_collapsed_if_unchosen(*s, !abertos.contains(s));
            let dobrado = self.host.store().is_collapsed(*s);
            if dobrado == abertos.contains(s) {
                self.host.store_mut().toggle_collapsed(*s);
            }
        }
        self.assenta();
    }
}

/// O pincel de uma ferramenta num meio, pela porta gravada do painel.
pub(super) fn pincel(meio: PaintMedia, modo: &str) -> (BrushSettings, BTreeSet<NodeId>) {
    let mut b = Bancada::nova(meio, &[]);
    b.no_modo(modo);
    b.host.open_all_sections();
    b.no_modo(modo);
    (b.tool.brush_settings(), b.ids().into_iter().collect())
}

pub(super) const TODAS: &[NodeId] = &[
    p::PAINTER_CARD_TOPO,
    p::PAINTER_CARD_BASICOS,
    p::PAINTER_BRUSH_MIX_SECTION,
    p::PAINTER_MASK_SECTION,
    p::PAINTER_WATERCOLOR_SECTION,
    p::PAINTER_IMPASTO_SECTION,
    p::PAINTER_WETPAINT_SECTION,
    p::PAINTER_BRUSH_RANDOMIZE_SECTION,
    p::PAINTER_SHAPE_SECTION,
    p::PAINTER_SHAPE_RAMP_SECTION,
    p::PAINTER_WATERCOLOR_PAPER_SECTION,
    p::PAINTER_BRUSH_TEXTURE_SECTION,
    p::PAINTER_BRUSH_STROKE_SECTION,
    p::PAINTER_PEN_SECTION,
    p::PAINTER_BRUSH_SYMMETRY_SECTION,
    p::PAINTER_BRUSH_TILING_SECTION,
];

/// O que o painel de antes pinta e a lista já não: as barras dos nativos e a barra de rolagem antiga. Nada mais.
pub(super) fn saiu_de_proposito(id: NodeId) -> bool {
    id == p::PAINTER_BRUSH_SYMMETRY_SEGMENTS
        || id == p::PAINTER_BRUSH_SIZE_SLIDER
        || id == p::PAINTER_BRUSH_STRENGTH_SLIDER
        || id == p::PAINTER_BRUSH_FLOW_SLIDER
        || p::PAINTER_BRUSH_RANDOMIZE_SLIDERS.contains(&id)
        || super::h7_os_cartoes::BARRAS.contains(&id)
        || super::h8_os_cartoes::barras().contains(&id)
        || id == ph2d_editor_core::widget::PAINTER_LAYERS_SCROLLBAR_ID
}

/// ⭐ O INVENTÁRIO, medido: cada id que o painel de antes pinta, em cada ferramenta e meio, os cartões pintam.
#[test]
fn every_old_id_is_still_painted_in_each_tool_and_medium() {
    let casos = [
        (PaintMedia::Digital, "brush"),
        (PaintMedia::Watercolor, "brush"),
        (PaintMedia::Impasto, "brush"),
        (PaintMedia::WetPaint, "brush"),
        (PaintMedia::Digital, "eraser"),
        (PaintMedia::Digital, "blur"),
        (PaintMedia::Digital, "smear"),
        (PaintMedia::Digital, "clone"),
        (PaintMedia::Digital, "mask"),
        (PaintMedia::Digital, "inpaint"),
        (PaintMedia::Digital, "selection"),
        (PaintMedia::Digital, "sculpt"),
    ];
    let mut contados = 0;
    for (meio, modo) in casos {
        let (brush, antes) = pincel(meio, modo);
        let agora = Mesa::nova(true, brush, false).ids();
        let faltam: Vec<_> = antes
            .iter()
            .filter(|i| !agora.contains(i) && !saiu_de_proposito(**i))
            .collect();
        assert!(faltam.is_empty(), "{modo} {meio:?}: sumiram {faltam:?}");
        contados += antes.len();
    }
    // A régua não mede o vazio: 12 estados, com 34 a 170 ids cada.
    assert!(contados > 900, "o inventário encolheu: {contados}");
}

/// O topo da vista Layers: as duas barras e o Apply, com a mesma acção do painel de antes.
#[test]
fn the_layers_view_lives_in_one_card_and_its_verbs_send_the_same_action() {
    for id in [
        p::PAINTER_LAYERS_ADD,
        p::PAINTER_LAYERS_GROUP,
        p::PAINTER_LAYERS_DUPLICATE,
        p::PAINTER_LAYERS_DELETE,
        p::PAINTER_LAYERS_ADD_TEXTURE,
        p::PAINTER_APPLY,
    ] {
        let velho = Mesa::nova(false, FALLBACK_BRUSH, true).clica(id);
        let novo = Mesa::nova(true, FALLBACK_BRUSH, true).clica(id);
        assert!(!velho.is_empty(), "{id:?}: o de antes não fez nada");
        assert_eq!(velho, novo, "{id:?}");
    }
}

/// O mesmo clique dá a mesma acção, nos corpos capturados e nos nativos.
#[test]
fn every_click_sends_the_same_action_as_the_old_panel() {
    let mut ligado = FALLBACK_BRUSH;
    ligado.symmetry_enabled = true;
    let mut circular = ligado;
    circular.symmetry_circular = true;
    let mut linha = ligado;
    linha.symmetry_axis = 2;
    let casos = [
        (FALLBACK_BRUSH, p::PAINTER_BRUSH_SYMMETRY_USE),
        (FALLBACK_BRUSH, p::PAINTER_BRUSH_TILING_X),
        (FALLBACK_BRUSH, p::PAINTER_BRUSH_TILING_Y),
        (FALLBACK_BRUSH, p::PAINTER_BRUSH_REPEAT_IMAGE),
        (FALLBACK_BRUSH, p::PAINTER_BRUSH_SYNC),
        (FALLBACK_BRUSH, p::PAINTER_BRUSH_ACCUMULATE),
        (FALLBACK_BRUSH, p::PAINTER_BRUSH_TILING_RESET),
        (FALLBACK_BRUSH, p::PAINTER_BRUSH_SYMMETRY_RESET),
        (FALLBACK_BRUSH, p::PAINTER_BRUSH_STROKE_RESET),
        (FALLBACK_BRUSH, p::PAINTER_SHAPE_RESET),
        (FALLBACK_BRUSH, p::PAINTER_SIDEBAR_TOGGLE_DOCK),
        (FALLBACK_BRUSH, p::PAINTER_LAYERS_TOGGLE_DOCK),
        (ligado, p::PAINTER_BRUSH_SYMMETRY_CIRCULAR),
        (ligado, p::PAINTER_BRUSH_SYMMETRY_AXIS_X),
        (ligado, p::PAINTER_BRUSH_SYMMETRY_AXIS_Y),
        (ligado, p::PAINTER_BRUSH_SYMMETRY_AXIS_CUSTOM),
        (circular, p::PAINTER_BRUSH_SYMMETRY_PICK_CENTER),
        (linha, p::PAINTER_BRUSH_SYMMETRY_DRAW_LINE),
    ];
    for (brush, id) in casos {
        let velho = Mesa::nova(false, brush, false).clica(id);
        let novo = Mesa::nova(true, brush, false).clica(id);
        assert!(!novo.is_empty(), "{id:?}: nada no barramento");
        assert_eq!(velho, novo, "{id:?}");
    }
}

pub(super) fn valor_enviado(accoes: &[String], id: NodeId) -> Option<f64> {
    let marca = format!("SetValue({id:?}, ");
    accoes.iter().rev().find_map(|a| {
        let resto = a.split(&marca).nth(1)?;
        resto.split(')').next()?.trim().parse().ok()
    })
}

pub(super) fn esc(mesa: &mut Mesa) -> Vec<String> {
    let arena = bumpalo::Bump::new();
    let tecla = ph2d_host::KeyEvent {
        keycode: 0x1B,
        modifiers: ph2d_host::Modifiers::default(),
        kind: ph2d_host::KeyKind::Down,
        timestamp_ns: 0,
    };
    let ev = dispatch_key(mesa.host.store_mut(), tecla, &arena).to_vec();
    mesa.aplica(ev)
}

/// O Pigment nativo: o scrub chega, um quadro a meio não o puxa de volta, e o Esc devolve.
#[test]
fn scrubbing_pigment_reaches_the_tool_and_escape_gives_it_back() {
    let id = p::PAINTER_WATERCOLOR_MIX;
    let mut brush = FALLBACK_BRUSH;
    brush.pigment_offered = true;
    brush.pigment = true;
    brush.pigment_mix = 0.25;
    let mut mesa = Mesa::nova(true, brush, false);
    mesa.so_abertos(&[p::PAINTER_BRUSH_MIX_SECTION]);
    let antes = mesa.host.store().number_value(id).expect("semeado");
    assert!((antes - 0.25).abs() < 1e-6, "semente {antes}");
    let ev = mesa.arrasta_a_pega(id, 60.0);
    assert!(ev.contains(&WidgetEvent::ValueChanged(id)), "{ev:?}");
    let no_gesto = mesa.host.store().number_value(id).expect("campo");
    assert!((no_gesto - antes).abs() > 1e-3, "não se moveu: {no_gesto}");
    let _ = mesa.aplica(ev);
    // Um quadro com o pincel ainda antigo: a semente não pode puxar o número de volta.
    mesa.quadro();
    let a_meio = mesa.host.store().number_value(id).expect("campo");
    assert!(
        (a_meio - no_gesto).abs() < 1e-6,
        "a semente puxou: {a_meio}"
    );
    let _ = esc(&mut mesa);
    let voltou = mesa.host.store().number_value(id).expect("campo");
    assert!((voltou - antes).abs() < 1e-6, "o Esc deu {voltou}");
}

/// Segments nativo: escrever chega pela barra de sempre, com o mesmo número do painel de antes.
#[test]
fn typing_segments_sends_the_same_action_as_the_old_panel() {
    let mut brush = FALLBACK_BRUSH;
    brush.symmetry_enabled = true;
    brush.symmetry_circular = true;
    let chip = p::PAINTER_BRUSH_SYMMETRY_SEGMENTS_CHIP;
    let escreve = |cartoes: bool| {
        let mut mesa = Mesa::nova(cartoes, brush, false);
        let ev = mesa.host.type_into_number(chip, "9");
        mesa.aplica(ev)
    };
    let (velho, novo) = (escreve(false), escreve(true));
    assert!(!novo.is_empty(), "nada no barramento");
    assert_eq!(velho, novo);
}

/// O corpo capturado: Paint Mode abre o popover debaixo da caixa, e a opção envia o mesmo.
#[test]
fn a_dropdown_opens_under_its_chip_and_its_option_sends_the_same() {
    let id = p::PAINTER_BRUSH_MEDIA;
    let opcao = p::painter_brush_media_option_id(1);
    let escolhe = |cartoes: bool| {
        let mut mesa = Mesa::nova(cartoes, FALLBACK_BRUSH, false);
        let chip = mesa.rect(id).expect("a caixa");
        let _ = mesa.clica(id);
        mesa.assenta();
        let r = mesa.rect(opcao).expect("a opção aberta");
        assert!(
            r.y >= chip.y && (r.x - chip.x).abs() < 200.0,
            "a lista abriu longe: {chip:?} → {r:?}"
        );
        mesa.clica(opcao)
    };
    let (velho, novo) = (escolhe(false), escolhe(true));
    assert!(!novo.is_empty());
    assert_eq!(velho, novo);
}

/// O Reset de uma secção mora no cabeçalho do cartão, acima do corpo, e tem dica.
#[test]
fn the_reset_of_a_section_lives_in_the_card_head_and_has_a_tooltip() {
    let mesa = Mesa::nova(true, FALLBACK_BRUSH, false);
    let sec = p::PAINTER_BRUSH_STROKE_SECTION;
    let reset = mesa.rect(p::PAINTER_BRUSH_STROKE_RESET).expect("o Reset");
    let pega = mesa.rect(grip_of(sec)).expect("a pega");
    assert!(
        reset.y < pega.y + pega.h && reset.x >= pega.x + pega.w - 1.0,
        "fora do cabeçalho: {reset:?} × {pega:?}"
    );
    assert!(
        mesa.host
            .store()
            .tooltip_for(p::PAINTER_BRUSH_STROKE_RESET)
            .is_some()
    );
}

/// Cada cabeçalho dobra, e o corpo sai com ele: num capturado e num nativo.
#[test]
fn every_header_folds_its_card_and_takes_the_body_with_it() {
    for (sec, de_dentro) in [
        (
            p::PAINTER_BRUSH_STROKE_SECTION,
            core_ids::PAINTER_BRUSH_STROKE_METHOD,
        ),
        (p::PAINTER_BRUSH_TILING_SECTION, p::PAINTER_BRUSH_TILING_X),
    ] {
        let mut mesa = Mesa::nova(true, FALLBACK_BRUSH, false);
        assert!(mesa.rect(de_dentro).is_some(), "{de_dentro:?} sem rect");
        let g = mesa.rect(grip_of(sec)).expect("a pega");
        let ev = mesa.host.click_at(g.x + g.w * 0.5, g.y + g.h * 0.5);
        let _ = mesa.aplica(ev);
        assert!(mesa.host.store().is_collapsed(sec), "{sec:?} não dobrou");
        mesa.assenta();
        assert!(mesa.rect(de_dentro).is_none(), "{de_dentro:?} ficou vivo");
    }
}

/// O botão direito no título abre o menu de tema da secção.
#[test]
fn right_click_on_a_card_head_opens_its_menu() {
    let mut mesa = Mesa::nova(true, FALLBACK_BRUSH, false);
    let g = mesa
        .rect(grip_of(p::PAINTER_BRUSH_TILING_SECTION))
        .expect("a pega");
    let sec = p::PAINTER_BRUSH_TILING_SECTION;
    let ev = mesa
        .host
        .right_click_at(g.x + g.w * 0.3, g.y + m::CARD_HEAD_H * 0.5);
    let _ = mesa.aplica(ev);
    let menu = mesa.host.store().context_menu().map(|c| c.kind);
    assert!(
        matches!(menu, Some(ph2d_editor_core::interaction::ContextMenuKind::SectionOutline { section }) if section == sec),
        "{menu:?}"
    );
}

/// Arrastar pelo título reordena, na ordem de sempre.
#[test]
fn dragging_a_card_by_its_header_reorders_the_list() {
    let mut mesa = Mesa::nova(true, FALLBACK_BRUSH, false);
    let (de, passa) = (
        p::PAINTER_BRUSH_SYMMETRY_SECTION,
        p::PAINTER_BRUSH_TILING_SECTION,
    );
    let g = mesa.rect(grip_of(de)).expect("a pega");
    let (x, y0) = (g.x + g.w * 0.5, g.y + g.h * 0.5);
    let _ = mesa.ponteiro(PointerKind::Down, x, y0);
    for i in 1..=30 {
        let _ = mesa.ponteiro(PointerKind::Move, x, y0 + (i * 20) as f32);
        mesa.quadro();
        // A voar, o corpo não regista clique.
        if i == 15 {
            assert!(
                mesa.rect(p::PAINTER_BRUSH_SYMMETRY_USE).is_none(),
                "o corpo do que voa ficou clicável"
            );
        }
    }
    let _ = mesa.ponteiro(PointerKind::Up, x, y0 + 600.0);
    mesa.assenta();
    let ordem = mesa.host.store().section_order().to_vec();
    let pos = |k: NodeId| ordem.iter().position(|o| *o == k);
    assert!(pos(de) > pos(passa), "não passou: {ordem:?}");
}

/// Soltar fora da coluna: um corpo CAPTURADO continua vivo lá, com o id no sítio novo.
#[test]
fn a_card_dragged_out_of_the_column_comes_loose_and_keeps_working() {
    let mut mesa = Mesa::nova(true, FALLBACK_BRUSH, false);
    let sec = p::PAINTER_BRUSH_STROKE_SECTION;
    let id = core_ids::PAINTER_BRUSH_STROKE_METHOD;
    let coluna = mesa.rect(id).expect("na lista").x;
    let g = mesa.rect(grip_of(sec)).expect("a pega");
    let (x, y0) = (g.x + g.w * 0.5, g.y + g.h * 0.5);
    let _ = mesa.ponteiro(PointerKind::Down, x, y0);
    for i in 1..=20 {
        let _ = mesa.ponteiro(PointerKind::Move, x - (i * 30) as f32, y0 + 40.0);
        mesa.quadro();
    }
    let _ = mesa.ponteiro(PointerKind::Up, x - 600.0, y0 + 40.0);
    mesa.assenta();
    assert!(
        mesa.host.store().is_loose_in(sec, PAINEL),
        "o cartão não se soltou"
    );
    let r = mesa.rect(id).expect("o controlo do solto");
    assert!(r.x < coluna - 100.0, "devia estar fora: {r:?}");
    assert_eq!(
        mesa.host.hit_at(r.x + r.w * 0.5, r.y + r.h * 0.5),
        Some(id),
        "o solto morreu"
    );
}

/// Rolar leva os alvos capturados com a cena, no próprio quadro; e o que sai da janela não fica clicável.
#[test]
fn scrolling_moves_the_targets_with_the_list() {
    let janela = Rect::new(0.0, 0.0, 1600.0, 700.0);
    let mut mesa = Mesa::na_janela(true, FALLBACK_BRUSH, false, janela);
    let id = p::PAINTER_BRUSH_BLEND;
    let antes = mesa.rect(id).expect("à vista");
    let lista = mesa.host.store().panel_rect(PAINEL).expect("a lista");
    let ev = mesa
        .host
        .wheel_at(lista.x + lista.w * 0.5, lista.y + 200.0, -40.0);
    let _ = mesa.aplica(ev);
    mesa.assenta();
    assert!(mesa.host.store().panel_scroll(PAINEL) > 0.0, "não rolou");
    let depois = mesa.rect(id).expect("ainda à vista");
    let rolou = mesa.host.store().panel_scroll(PAINEL);
    assert!(
        (antes.y - rolou - depois.y).abs() < 1.0,
        "{antes:?} − {rolou} ≠ {depois:?}"
    );
    // Uma secção lá de baixo ainda não pinta alvos fora da janela.
    if let Some(r) = mesa.rect(p::PAINTER_BRUSH_TILING_X) {
        assert!(r.y + r.h <= 700.0 + 1.0, "fora da janela: {r:?}");
    }
}

/// As linhas da vista Layers publicam-se para o arrasto, como antes, e o clique numa linha é o mesmo.
#[test]
fn the_layers_card_publishes_its_rows_and_a_row_click_is_the_same() {
    use ph2d_editor_core::interaction::PanelRowFamily;
    use ph2d_tool_painter::ids::{PainterLayerWidget, painter_layer_widget_id};
    let clica_a_linha = |cartoes: bool| {
        let mut mesa = Mesa::nova(cartoes, FALLBACK_BRUSH, true);
        let mut pilha = ph2d_tool_painter::LayerStack::new();
        let l = pilha.add_raster("One", 8, 8).expect("a camada nasce");
        let _ = pilha.add_raster("Two", 8, 8).expect("a camada nasce");
        ph2d_panel_painter_layers::state::set_current_layers(Some(pilha));
        mesa.assenta();
        let linha = painter_layer_widget_id(l.0, PainterLayerWidget::Row);
        assert_eq!(
            mesa.host.store().panel_row_family(linha),
            Some(PanelRowFamily::PainterLayer),
            "a linha não está publicada"
        );
        let accoes = mesa.clica(linha);
        ph2d_panel_painter_layers::state::set_current_layers(None);
        accoes
    };
    let (velho, novo) = (clica_a_linha(false), clica_a_linha(true));
    assert!(!novo.is_empty());
    assert_eq!(velho, novo);
}

/// Trocar de vista troca os cartões, e o «×» é o de sempre.
#[test]
fn the_strip_and_the_close_button_are_the_same_ids() {
    let mut mesa = Mesa::nova(true, FALLBACK_BRUSH, false);
    assert!(mesa.rect(p::PAINTER_SIDEBAR_TOGGLE_DOCK).is_some());
    assert!(mesa.rect(p::PAINTER_LAYERS_TOGGLE_DOCK).is_some());
    let velho = Mesa::nova(false, FALLBACK_BRUSH, false).clica(p::PAINTER_LAYERS_CLOSE);
    let novo = mesa.clica(p::PAINTER_LAYERS_CLOSE);
    assert_eq!(velho, novo);
}

/// Todo ícone sem texto tem dica.
#[test]
fn every_mute_control_has_a_tooltip() {
    let mesa = Mesa::nova(true, FALLBACK_BRUSH, false);
    let store = mesa.host.store();
    let mudos: Vec<NodeId> = dicas::icon_only_ids(PAINEL)
        .into_iter()
        .chain([p::PAINTER_LAYERS_CLOSE, list_menu_of(PAINEL)])
        .filter(|id| store.tooltip_for(*id).is_none())
        .collect();
    assert!(mudos.is_empty(), "sem dica: {mudos:?}");
}

/// `PH2D_CARDS=0`: o corpo antigo, sem lista, com o cabeçalho antigo.
#[test]
fn cards_off_paints_the_old_body() {
    let mesa = Mesa::nova(false, FALLBACK_BRUSH, false);
    assert!(mesa.rect(grip_of(p::PAINTER_BRUSH_MIX_SECTION)).is_none());
    assert!(mesa.rect(p::PAINTER_BRUSH_SYMMETRY_RESET).is_some());
    assert_eq!(mesa.host.store().floating_panel_rects().count(), 0);
}

/// A linha esmaecida ainda não chegou aos nativos: o dia em que um destes ids ganhar uma lei, o corpo tem de a pintar.
#[test]
fn no_native_id_has_a_dimmed_law() {
    let nativos = [
        p::PAINTER_WATERCOLOR_MIX,
        p::PAINTER_BRUSH_SYMMETRY_USE,
        p::PAINTER_BRUSH_SYMMETRY_CIRCULAR,
        p::PAINTER_BRUSH_SYMMETRY_SEGMENTS_CHIP,
        p::PAINTER_BRUSH_SYMMETRY_AXIS_X,
        p::PAINTER_BRUSH_SYMMETRY_AXIS_Y,
        p::PAINTER_BRUSH_SYMMETRY_AXIS_CUSTOM,
        p::PAINTER_BRUSH_SYMMETRY_PICK_CENTER,
        p::PAINTER_BRUSH_SYMMETRY_DRAW_LINE,
        p::PAINTER_BRUSH_TILING_X,
        p::PAINTER_BRUSH_TILING_Y,
        p::PAINTER_BRUSH_REPEAT_IMAGE,
    ];
    let leis = ph2d_tool_painter::Dependente::TODOS.len();
    assert!(leis > 5, "a tabela das leis encolheu: {leis}");
    for id in nativos {
        for d in ph2d_tool_painter::Dependente::TODOS {
            assert!(
                !d.cobre(id),
                "{id:?} ganhou a lei {d:?}: pinte-a no corpo nativo"
            );
        }
    }
}

/// Cada caixa nativa mostra o SEU valor: trocar qual está ligada muda o desenho.
#[test]
fn each_native_box_shows_its_own_value() {
    let desenho = |x: bool, y: bool, repete: bool, simetria: bool| {
        let mut b = FALLBACK_BRUSH;
        b.tiling = [x, y];
        b.repeat_image = repete;
        b.symmetry_enabled = simetria;
        let mut mesa = Mesa::nova(true, b, false);
        let (h, st) = (&mut mesa.host, &mut mesa.st);
        h.paint_draw_data::<PainterLayersPanel>(st, COLUNA)
    };
    let base = desenho(false, false, false, false);
    let x = desenho(true, false, false, false);
    let y = desenho(false, true, false, false);
    let repete = desenho(false, false, true, false);
    assert_ne!(base, x, "Tiling X não se vê");
    assert_ne!(base, y, "Tiling Y não se vê");
    assert_ne!(x, y, "X e Y desenham o mesmo");
    assert_ne!(base, repete, "Repeat Image não se vê");
    assert_ne!(
        base,
        desenho(false, false, false, true),
        "Use Symmetry não se vê"
    );
}

/// O Segments semeia-se do pincel: o campo nativo não tem barra que o leve.
#[test]
fn the_segments_field_shows_the_brush_value() {
    let mut brush = FALLBACK_BRUSH;
    brush.symmetry_enabled = true;
    brush.symmetry_circular = true;
    brush.symmetry_segments = 11;
    let mesa = Mesa::nova(true, brush, false);
    let chip = p::PAINTER_BRUSH_SYMMETRY_SEGMENTS_CHIP;
    assert_eq!(mesa.host.store().number_value(chip), Some(11.0));
}

/// Deform é dono do corpo inteiro, num cartão: os ids dele continuam lá, e nada do pincel.
#[test]
fn deform_owns_the_whole_body_in_one_card() {
    let mut brush = FALLBACK_BRUSH;
    brush.is_deform = true;
    let antes = Mesa::nova(false, brush, false).ids();
    let mesa = Mesa::nova(true, brush, false);
    let agora = mesa.ids();
    let faltam: Vec<_> = antes
        .iter()
        .filter(|i| !agora.contains(i) && !saiu_de_proposito(**i))
        .collect();
    assert!(faltam.is_empty(), "sumiram {faltam:?}");
    assert!(agora.contains(&grip_of(p::PAINTER_CARD_EXCLUSIVO)));
    assert!(!agora.contains(&p::PAINTER_BRUSH_SYNC));
}

/// A vista Layers fecha o seletor da cor do pincel, como no painel de antes.
#[test]
fn the_layers_view_closes_the_brush_colour_picker() {
    let mut mesa = Mesa::nova(true, FALLBACK_BRUSH, true);
    mesa.host
        .store_mut()
        .set_picker_target(Some(core_ids::PAINTER_COLOR_THUMB));
    mesa.quadro();
    assert_eq!(mesa.host.store().picker_target(), None);
}

/// A medição: as duas pinturas intercaladas, o mínimo de cada, gravado no ficheiro de `PH2D_CUSTO_TXT`.
#[test]
#[ignore = "mede: --ignored, em --profile smoke"]
fn custo_por_quadro_das_duas_caras() {
    let (brush, _) = pincel(PaintMedia::WetPaint, "brush");
    let mut velho = Mesa::nova(false, brush, false);
    let mut novo = Mesa::nova(true, brush, false);
    let (mut a, mut b) = (u128::MAX, u128::MAX);
    for _ in 0..30 {
        force_cards_on_this_thread(Some(false));
        let t = std::time::Instant::now();
        velho.quadro();
        a = a.min(t.elapsed().as_micros());
        force_cards_on_this_thread(Some(true));
        let t = std::time::Instant::now();
        novo.quadro();
        b = b.min(t.elapsed().as_micros());
    }
    let linha = format!("cartoes {b} us · antes {a} us\n");
    if let Ok(f) = std::env::var("PH2D_CUSTO_TXT") {
        std::fs::write(f, &linha).expect("grava");
    }
    println!("{linha}");
}

/// As mesmas secções abrem: os cartões não trocam o que o dono decidiu deixar recolhido.
#[test]
fn the_panel_opens_with_the_same_sections() {
    let abre = |cartoes: bool| {
        force_cards_on_this_thread(Some(cartoes));
        set_current_brush(Some(FALLBACK_BRUSH));
        set_current_dock_shows_layers(false);
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let mut r = Vec::new();
        for _ in 0..40 {
            r = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || host.paint::<PainterLayersPanel>(&mut st, COLUNA),
            );
        }
        r.into_iter()
            .filter(|(_, r)| r.w > 0.0 && r.h > 0.0)
            .map(|(i, _)| i)
            .collect::<BTreeSet<NodeId>>()
    };
    let (antes, agora) = (abre(false), abre(true));
    let faltam: Vec<_> = antes
        .iter()
        .filter(|i| !agora.contains(i) && !saiu_de_proposito(**i))
        .collect();
    assert!(faltam.is_empty(), "sumiram {faltam:?}");
    // Dos corpos recolhidos, nada aparece: Randomize e Tiling continuam dobrados.
    assert!(!agora.contains(&p::PAINTER_BRUSH_TILING_X));
    assert!(!antes.contains(&p::PAINTER_BRUSH_TILING_X));
    assert!(agora.contains(&p::PAINTER_BRUSH_SYNC));
}
