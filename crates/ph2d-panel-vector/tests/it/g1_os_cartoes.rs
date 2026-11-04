//! ⭐⭐ **O painel Vector em cartões, fatia G1** (`docs/CartoesUI/12_VECTOR_PLANO.md`): a lista, as abas,
//! o hospedeiro dos corpos por converter e os corpos nativos de Transform, Snap, Constraints, Frame e
//! Clip. Cada id é provado pelo gesto REAL, nos rects pintados, e o que chega ao barramento é o MESMO
//! do painel de antes.

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{WidgetEvent, dispatch_key};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::dicas;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{AnchorState, VectorPanelState};
use ph2d_panel_vector::{VectorPanel, cards, ids as p, state};
use ph2d_tool_vector::frames::DEVICE_PRESETS;
use ph2d_tool_vector::ids as t;
use ph2d_ui_testkit::MockPanelHost;

const COLUNA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1340.0,
    h: 3200.0,
};

fn publica() {
    state::set_length_suffix("px");
    state::set_current_transform(Some([10.0, 20.0, 100.0, 40.0]));
    state::set_resize_box(Some(false));
    state::set_anchor_state(Some(AnchorState {
        h: Some(p::VECTOR_ANCHOR_H_START),
        v: Some(p::VECTOR_ANCHOR_V_END),
    }));
    state::set_frame_present(true);
    state::set_frame_clip(Some(false));
    state::set_frame_panel_open(false);
    state::set_current_snap(false);
}

struct Mesa {
    host: MockPanelHost,
    st: VectorPanelState,
    rects: Vec<(NodeId, Rect)>,
}

impl Mesa {
    fn nova(cartoes: bool) -> Self {
        cards::force_cards_on_this_thread(Some(cartoes));
        publica();
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        host.open_all_sections();
        let mut mesa = Self {
            host,
            st: VectorPanelState,
            rects: Vec::new(),
        };
        mesa.assenta();
        mesa
    }

    fn assenta(&mut self) {
        for _ in 0..60 {
            let (h, st) = (&mut self.host, &mut self.st);
            self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || h.paint::<VectorPanel>(st, COLUNA),
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

    fn aplica(&mut self, eventos: Vec<WidgetEvent>) -> Vec<String> {
        let _ = self.host.drained_actions();
        for e in eventos {
            self.host.apply_panel_event::<VectorPanel>(&mut self.st, e);
        }
        self.host
            .drained_actions()
            .iter()
            .map(|a| format!("{a:?}"))
            .collect()
    }

    fn clica(&mut self, id: NodeId) -> Vec<String> {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("{id:?} não pintado"));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(self.host.hit_at(cx, cy), Some(id), "outro id no centro");
        let ev = self.host.click_at(cx, cy);
        assert!(
            ev.contains(&WidgetEvent::Click(id)),
            "{id:?} não emitiu Click: {ev:?}"
        );
        self.aplica(ev)
    }

    fn ponteiro(&mut self, kind: PointerKind, x: f32, y: f32) -> Vec<WidgetEvent> {
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

    /// O scrub real: premir na PEGA e arrastar `dx`, sem largar.
    fn arrasta_a_pega(&mut self, id: NodeId, dx: f32) -> Vec<WidgetEvent> {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("{id:?} não pintado"));
        let (x, y) = (r.x + m::ICON_CELL * 0.5, r.y + r.h * 0.5);
        let mut ev = self.ponteiro(PointerKind::Down, x, y);
        ev.extend(self.ponteiro(PointerKind::Move, x + dx * 0.5, y));
        ev.extend(self.ponteiro(PointerKind::Move, x + dx, y));
        ev
    }

    fn abre_interface(&mut self) {
        self.clica(p::VECTOR_TAB_INTERFACE);
        self.assenta();
    }

    fn abre_drawing(&mut self) {
        self.clica(p::VECTOR_TAB_DRAWING);
        self.assenta();
    }
}

/// Os verbos e as opções (um clique) de cada corpo nativo, por aba.
fn cliques_drawing() -> Vec<NodeId> {
    vec![
        p::VECTOR_TRANSFORM_RESIZE_BOX,
        p::VECTOR_SNAP_OFF,
        core_ids::VECTOR_SNAP_ON,
        p::VECTOR_SNAP_PATH_OFF,
        p::VECTOR_SNAP_PATH_ON,
        p::VECTOR_SNAP_CROSS_OFF,
        p::VECTOR_SNAP_CROSS_ON,
        p::VECTOR_SNAP_GUIDES_OFF,
        p::VECTOR_SNAP_GUIDES_ON,
        p::VECTOR_PIVOT_EDIT,
    ]
}

fn cliques_interface() -> Vec<NodeId> {
    let mut v = vec![
        p::VECTOR_ANCHOR_H_START,
        p::VECTOR_ANCHOR_H_CENTER,
        p::VECTOR_ANCHOR_H_END,
        p::VECTOR_ANCHOR_H_STRETCH,
        p::VECTOR_ANCHOR_V_START,
        p::VECTOR_ANCHOR_V_CENTER,
        p::VECTOR_ANCHOR_V_END,
        p::VECTOR_ANCHOR_V_STRETCH,
        p::VECTOR_FRAME_PANEL_OFF,
        p::VECTOR_FRAME_PANEL_ON,
        p::VECTOR_FRAME_CLIP_OFF,
        p::VECTOR_FRAME_CLIP_ON,
    ];
    v.extend(DEVICE_PRESETS.iter().map(|d| d.id));
    v
}

const NUMEROS: [NodeId; 5] = [
    t::VECTOR_TRANSFORM_X,
    t::VECTOR_TRANSFORM_Y,
    t::VECTOR_TRANSFORM_W,
    t::VECTOR_TRANSFORM_H,
    p::VECTOR_TRANSFORM_R,
];

/// O inventário, derivado do que o corpo de antes regista: cada id tem de estar no store e pintado.
#[test]
fn every_id_of_the_slice_is_registered_and_painted_in_cards() {
    let mut mesa = Mesa::nova(true);
    for id in NUMEROS.into_iter().chain(cliques_drawing()) {
        assert!(mesa.host.store().get(id).is_some(), "{id:?} sem registo");
        assert!(mesa.rect(id).is_some(), "{id:?} não pintado em Drawing");
    }
    mesa.abre_interface();
    for id in cliques_interface() {
        assert!(mesa.host.store().get(id).is_some(), "{id:?} sem registo");
        assert!(mesa.rect(id).is_some(), "{id:?} não pintado em Interface");
    }
    // O nome vai com o id: Left à esquerda de Stretch, Top à esquerda de Stretch.
    for (primeiro, ultimo) in [
        (p::VECTOR_ANCHOR_H_START, p::VECTOR_ANCHOR_H_STRETCH),
        (p::VECTOR_ANCHOR_V_START, p::VECTOR_ANCHOR_V_STRETCH),
    ] {
        let (a, b) = (mesa.rect(primeiro), mesa.rect(ultimo));
        assert!(a.zip(b).is_some_and(|(a, b)| a.x < b.x), "{a:?} × {b:?}");
    }
}

/// ⭐⭐ **Mesmo id, mesmo evento**: o clique real em cada opção e verbo produz no barramento exactamente
/// o que produz no painel de antes. *Mutação: trocar dois ids num `Seg` ⇒ vermelho.*
#[test]
fn every_click_sends_the_same_action_as_the_old_panel() {
    let grava = |cartoes: bool, ids: Vec<NodeId>, interface: bool| -> Vec<(NodeId, Vec<String>)> {
        let mut mesa = Mesa::nova(cartoes);
        if interface {
            mesa.abre_interface();
        }
        let out: Vec<(NodeId, Vec<String>)> =
            ids.into_iter().map(|id| (id, mesa.clica(id))).collect();
        if interface {
            mesa.abre_drawing();
        }
        out
    };
    for (ids, interface) in [(cliques_drawing(), false), (cliques_interface(), true)] {
        let novo = grava(true, ids.clone(), interface);
        let velho = grava(false, ids, interface);
        assert_eq!(novo, velho, "o barramento mudou entre os dois painéis");
        assert!(
            novo.iter().any(|(_, a)| !a.is_empty()),
            "nada chegou ao barramento"
        );
    }
}

fn valores_enviados(accoes: &[String], id: NodeId) -> Vec<f64> {
    let alvo = format!(
        "{:?}",
        EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, 0.0))
    );
    let cabeca = alvo.split(", ").next().unwrap_or_default().to_string();
    accoes
        .iter()
        .filter_map(|a| {
            let resto = a.strip_prefix(cabeca.as_str())?.strip_prefix(", ")?;
            resto.trim_end_matches(&[')', ' '][..]).parse().ok()
        })
        .collect()
}

fn valor_enviado(accoes: &[String], id: NodeId) -> Option<f64> {
    valores_enviados(accoes, id).last().copied()
}

/// ⭐ **Scrub de cada número**: chega ao barramento pelo id dele, e o Esc devolve-o. A rotação é
/// relativa: os deltas somam a distância arrastada, e o re-semear não a pode zerar a meio.
/// *Mutação: tirar o `scrub != Some(id)` do `seed_number_fields` ⇒ a rotação soma a mais.*
#[test]
fn scrubbing_each_number_reaches_the_tool_and_escape_gives_it_back() {
    for id in [
        t::VECTOR_TRANSFORM_X,
        t::VECTOR_TRANSFORM_Y,
        t::VECTOR_TRANSFORM_W,
        t::VECTOR_TRANSFORM_H,
    ] {
        let mut mesa = Mesa::nova(true);
        let antes = mesa.host.store().number_value(id).expect("semeado");
        let ev = mesa.arrasta_a_pega(id, 60.0);
        assert!(
            ev.contains(&WidgetEvent::ValueChanged(id)),
            "{id:?} não avisou: {ev:?}"
        );
        let accoes = mesa.aplica(ev);
        let v = valor_enviado(&accoes, id).unwrap_or_else(|| panic!("{id:?}: {accoes:?}"));
        assert!(
            (v - antes).abs() > 1.0,
            "{id:?} não se moveu: {antes} → {v}"
        );
        // Um quadro a meio do gesto: a semente do documento não pode puxar o número de volta.
        mesa.host.paint::<VectorPanel>(&mut mesa.st, COLUNA);
        let no_campo = mesa.host.store().number_value(id).expect("campo");
        assert!(
            (no_campo - v).abs() < 1e-6,
            "{id:?}: a semente puxou {v} → {no_campo}"
        );
        let arena = bumpalo::Bump::new();
        let esc = ph2d_host::KeyEvent {
            keycode: 0x1B,
            modifiers: ph2d_host::Modifiers::default(),
            kind: ph2d_host::KeyKind::Down,
            timestamp_ns: 0,
        };
        let ev = dispatch_key(mesa.host.store_mut(), esc, &arena).to_vec();
        let accoes = mesa.aplica(ev);
        let voltou = valor_enviado(&accoes, id).unwrap_or_else(|| panic!("Esc: {accoes:?}"));
        assert!((voltou - antes).abs() < 1e-6, "{id:?}: o Esc deu {voltou}");
    }
}

#[test]
fn a_rotation_scrub_sends_deltas_that_sum_to_the_distance() {
    let id = p::VECTOR_TRANSFORM_R;
    let mut mesa = Mesa::nova(true);
    let r = mesa.rect(id).expect("pintado");
    let (x, y) = (r.x + m::ICON_CELL * 0.5, r.y + r.h * 0.5);
    let mut total = 0.0;
    let mut ev = mesa.ponteiro(PointerKind::Down, x, y);
    ev.extend(mesa.ponteiro(PointerKind::Move, x + 10.0, y));
    for passo in [20.0_f32, 30.0, 40.0] {
        ev.extend(mesa.ponteiro(PointerKind::Move, x + passo, y));
        total += valores_enviados(&mesa.aplica(std::mem::take(&mut ev)), id)
            .iter()
            .sum::<f64>();
        // Um quadro entre dois movimentos: é aqui que a semente zerava o campo arrastado.
        mesa.host.paint::<VectorPanel>(&mut mesa.st, COLUNA);
    }
    let no_campo = mesa.host.store().number_value(id).expect("campo");
    assert!(
        no_campo > 0.0,
        "a semente zerou o campo a meio do arrasto: total {total}"
    );
    assert!(
        (total - no_campo).abs() < 1e-6,
        "soma dos deltas {total} ≠ o que o campo acumulou {no_campo}"
    );
}

/// Escrever um número chega pelo mesmo id.
#[test]
fn typing_a_number_reaches_the_tool() {
    let id = t::VECTOR_TRANSFORM_W;
    let mut mesa = Mesa::nova(true);
    let ev = mesa.host.type_into_number(id, "250");
    let accoes = mesa.aplica(ev);
    assert_eq!(valor_enviado(&accoes, id), Some(250.0), "{accoes:?}");
}

/// Cada cabeçalho dobra pelo clique real.
#[test]
fn every_native_header_folds_its_card() {
    let mut mesa = Mesa::nova(true);
    for id in [t::VECTOR_SECTION_TRANSFORM, t::VECTOR_SECTION_SNAP] {
        let r = mesa
            .host
            .registos_da_ultima_pintura()
            .into_iter()
            .filter(|(n, r)| *n == cards_grip(id) && r.w > 0.0)
            .map(|(_, r)| r)
            .last()
            .expect("a pega do cartão");
        let antes = mesa.host.store().is_collapsed(id);
        let ev = mesa.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        mesa.aplica(ev);
        assert_ne!(
            mesa.host.store().is_collapsed(id),
            antes,
            "{id:?} não dobrou"
        );
        mesa.assenta();
    }
}

fn cards_grip(id: NodeId) -> NodeId {
    ph2d_editor_core::panel::cards::grip_of(id)
}

/// Arrastar pelo título reordena, na ordem de sempre.
#[test]
fn dragging_a_card_by_its_header_reorders_the_list() {
    let mut mesa = Mesa::nova(true);
    let de = t::VECTOR_SECTION_TRANSFORM;
    let g = mesa.rect(cards_grip(de)).expect("a pega");
    let (x, y0) = (g.x + g.w * 0.5, g.y + g.h * 0.5);
    let _ = mesa.ponteiro(PointerKind::Down, x, y0);
    for i in 1..=30 {
        let _ = mesa.ponteiro(PointerKind::Move, x, y0 + (i * 20) as f32);
        mesa.host.paint::<VectorPanel>(&mut mesa.st, COLUNA);
    }
    let _ = mesa.ponteiro(PointerKind::Up, x, y0 + 600.0);
    mesa.assenta();
    let ordem = mesa.host.store().section_order().to_vec();
    let pos = |k: NodeId| ordem.iter().position(|o| *o == k);
    assert!(
        pos(de) > pos(t::VECTOR_SECTION_SNAP),
        "o Transform devia ter passado o Snap: {ordem:?}"
    );
}

/// Soltar um cartão fora da coluna: fica solto e o campo responde lá.
#[test]
fn a_card_dragged_out_of_the_column_comes_loose_and_keeps_working() {
    let mut mesa = Mesa::nova(true);
    let sec = t::VECTOR_SECTION_TRANSFORM;
    let coluna = mesa
        .host
        .store()
        .panel_rect(core_ids::VECTOR_PANEL)
        .expect("a lista")
        .x;
    let g = mesa.rect(cards_grip(sec)).expect("a pega");
    let (x, y0) = (g.x + g.w * 0.5, g.y + g.h * 0.5);
    let _ = mesa.ponteiro(PointerKind::Down, x, y0);
    for i in 1..=20 {
        let _ = mesa.ponteiro(PointerKind::Move, x - (i * 30) as f32, y0 + 40.0);
        mesa.host.paint::<VectorPanel>(&mut mesa.st, COLUNA);
    }
    let _ = mesa.ponteiro(PointerKind::Up, x - 600.0, y0 + 40.0);
    mesa.assenta();
    assert!(
        mesa.host.store().is_loose_in(sec, core_ids::VECTOR_PANEL),
        "o cartão não se soltou"
    );
    let id = t::VECTOR_TRANSFORM_X;
    let r = mesa.rect(id).expect("o campo do solto");
    assert!(r.x < coluna, "o campo devia estar fora da coluna: {r:?}");
    let ev = mesa.arrasta_a_pega(id, 60.0);
    assert!(
        valor_enviado(&mesa.aplica(ev), id).is_some(),
        "o solto morreu"
    );
}

/// As abas moram no cabeçalho da lista, trocam o que se vê, e o «×» cancela a ferramenta.
#[test]
fn the_tabs_live_in_the_list_head_and_switch_the_cards() {
    let mut mesa = Mesa::nova(true);
    let lista = mesa
        .host
        .store()
        .panel_rect(core_ids::VECTOR_PANEL)
        .expect("a lista publicou o rect");
    assert_eq!(
        mesa.host.store().floating_panel_rects().next(),
        Some(lista),
        "a lista flutua"
    );
    let aba = mesa.rect(p::VECTOR_TAB_INTERFACE).expect("a aba");
    assert!(
        aba.y >= lista.y && aba.y + aba.h <= lista.y + m::LIST_HEAD_H,
        "aba {aba:?} fora do cabeçalho da lista {lista:?}"
    );
    assert!(mesa.rect(p::VECTOR_ANCHOR_H_START).is_none());
    assert!(
        mesa.rect(t::VECTOR_TRANSFORM_X).is_some(),
        "Ambas: em Drawing"
    );
    mesa.abre_interface();
    assert!(mesa.rect(p::VECTOR_ANCHOR_H_START).is_some());
    assert!(
        mesa.rect(t::VECTOR_TRANSFORM_X).is_some(),
        "Ambas: em Interface"
    );
    let accoes = mesa.clica(core_ids::VECTOR_CLOSE);
    assert!(!accoes.is_empty(), "o «×» não fez nada");
    mesa.clica(p::VECTOR_TAB_DRAWING);
}

/// O corpo por converter mora num cartão, com o título absorvido e o «+» no cabeçalho, e o clique
/// neste abre a mesma lista de sempre. O painel à esquerda nasce à esquerda da LISTA.
#[test]
fn an_unconverted_section_lives_in_a_card_and_its_plus_still_opens() {
    let mut mesa = Mesa::nova(true);
    let sec = t::VECTOR_SECTION_FILL;
    let pega = mesa
        .rect(cards_grip(sec))
        .expect("Fill & Stroke é um cartão");
    let mais = mesa.rect(p::VECTOR_PAINT_ADD_MENU);
    // Sem sujeito o «+» está apagado e não se regista; com sujeito está no título.
    if let Some(r) = mais {
        assert!(
            r.y >= pega.y && r.y < pega.y + pega.h,
            "o «+» fora do título"
        );
    }
    let colunas: Vec<NodeId> = mesa
        .rects
        .iter()
        .filter(|(n, _)| *n == sec)
        .map(|(n, _)| *n)
        .collect();
    assert!(
        colunas.len() <= 1,
        "o cabeçalho absorvido registou-se por cima do título do cartão"
    );
}

/// O grupo Advanced é um cartão só: os membros moram dentro, e não na lista.
#[test]
fn the_advanced_group_is_one_card_with_its_members_inside() {
    let mesa = Mesa::nova(true);
    assert!(mesa.rect(cards_grip(p::VECTOR_SECTION_ADVANCED)).is_some());
    let dentro = mesa.rect(t::VECTOR_SECTION_BLEND).expect("Blend lá dentro");
    let card = mesa
        .rect(cards_grip(p::VECTOR_SECTION_ADVANCED))
        .expect("Advanced");
    assert!(dentro.y > card.y, "o Blend mora dentro do cartão");
    // Dobrado o grupo, o membro some com ele.
    let mut mesa = mesa;
    mesa.host.set_collapsed(p::VECTOR_SECTION_ADVANCED, true);
    mesa.assenta();
    assert!(
        mesa.rect(t::VECTOR_SECTION_BLEND).is_none(),
        "o Blend ficou de fora"
    );
}

/// Todo ícone mudo tem dica, e o campo nomeia-se com a unidade.
#[test]
fn every_mute_control_in_the_native_cards_has_a_tooltip() {
    let mesa = Mesa::nova(true);
    let store = mesa.host.store();
    let panel = core_ids::VECTOR_PANEL;
    let mudos: Vec<NodeId> = dicas::icon_only_ids(panel)
        .into_iter()
        .chain([
            core_ids::VECTOR_CLOSE,
            ph2d_editor_core::panel::cards::list_menu_of(panel),
        ])
        .filter(|id| store.tooltip_for(*id).is_none())
        .collect();
    assert!(mudos.is_empty(), "sem dica: {mudos:?}");
    let dica = dicas::field_hint(store, t::VECTOR_TRANSFORM_W).expect("campo marcado");
    assert!(dica.contains("(px)"), "a dica nomeia a unidade: {dica}");
}

/// `PH2D_CARDS=0`: nenhuma lista, o corpo de antes.
#[test]
fn cards_off_paints_the_old_body() {
    let mesa = Mesa::nova(false);
    assert!(
        mesa.host.store().floating_panel_rects().next().is_none(),
        "com PH2D_CARDS=0 não há lista flutuante"
    );
    assert!(mesa.rect(t::VECTOR_TRANSFORM_X).is_some());
    assert!(mesa.rect(p::VECTOR_SNAP_OFF).is_some());
}

/// A sonda não pinta nem regista nada: a lista vem da condição de cada secção, e sem seleção o
/// Transform sai.
#[test]
fn a_section_without_a_subject_has_no_card() {
    let mut mesa = Mesa::nova(true);
    assert!(mesa.rect(cards_grip(t::VECTOR_SECTION_TRANSFORM)).is_some());
    state::set_current_transform(None);
    mesa.assenta();
    assert!(
        mesa.rect(cards_grip(t::VECTOR_SECTION_TRANSFORM)).is_none(),
        "o Transform sem forma não tem cartão"
    );
}

/// A subida da altura de abertura (`219 → 250`) é só moldura: abre com as mesmas três secções.
#[test]
fn the_panel_opens_with_the_same_three_sections() {
    cards::force_cards_on_this_thread(Some(true));
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let rects = host.paint::<VectorPanel>(&mut st, COLUNA);
    let tem = |id: NodeId| rects.iter().any(|(n, r)| *n == cards_grip(id) && r.w > 0.0);
    for id in [
        t::VECTOR_SECTION_SNAP,
        t::VECTOR_SECTION_FILL,
        p::VECTOR_SECTION_ADVANCED,
    ] {
        assert!(tem(id), "{id:?} devia abrir");
    }
    for id in [t::VECTOR_SECTION_TRANSFORM, p::VECTOR_SECTION_APPEARANCE] {
        assert!(!tem(id), "{id:?} não tem sujeito ao abrir");
    }
    assert!(host.store().is_collapsed(t::VECTOR_SECTION_SNAP));
    assert!(host.store().is_collapsed(p::VECTOR_SECTION_ADVANCED));
    assert!(!host.store().is_collapsed(t::VECTOR_SECTION_FILL));
}

fn forma_com_tinta() {
    let camada = state::PaintRow {
        align: ph2d_vec_scene::StrokeAlign::Centre,
        ramp: None,
        is_fill: true,
        color: [0x33, 0x66, 0xCC, 0xFF],
        width: 2.0,
        enabled: true,
        opacity: 0.5,
        blend: ph2d_vec_scene::BlendMode::Normal,
        offset: [0.0, 0.0],
        dilate: 0.0,
        dilate_join: 1,
        mixed: Default::default(),
    };
    state::set_current_appearance(Some(state::Appearance {
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        layers: vec![camada],
        mixed_opacity: false,
        mixed_blend: false,
        layers_differ: false,
        order: Vec::new(),
    }));
    state::set_base_paints(Some([state::BasePaint::Shown; 2]));
    state::set_current_fill(Some(ph2d_panel_vector::FillKind::Linear), None);
}

/// ⭐ **F44 no cartão**: o «+» mora no título do cartão e abre a lista de sempre; a linha abre o painel
/// À ESQUERDA da lista. *Mutação: passar o rect do slot, e não o da lista, aos passes diferidos.*
#[test]
fn the_fill_row_opens_left_of_the_list_and_the_plus_lives_in_the_title() {
    forma_com_tinta();
    let mut mesa = Mesa::nova(true);
    let lista = mesa
        .host
        .store()
        .panel_rect(core_ids::VECTOR_PANEL)
        .expect("a lista");
    let pega = mesa
        .rect(cards_grip(t::VECTOR_SECTION_FILL))
        .expect("Fill & Stroke é um cartão");
    let mais = mesa.rect(p::VECTOR_PAINT_ADD_MENU).expect("o «+» vivo");
    assert!(
        mais.y >= pega.y && mais.y + mais.h <= pega.y + pega.h,
        "o «+» fora do título: {mais:?} × {pega:?}"
    );
    let (cx, cy) = (mais.x + mais.w * 0.5, mais.y + mais.h * 0.5);
    assert_eq!(mesa.host.hit_at(cx, cy), Some(p::VECTOR_PAINT_ADD_MENU));
    let ev = mesa.host.click_at(cx, cy);
    mesa.aplica(ev);
    assert_eq!(
        mesa.host.dropdown_is_open(p::VECTOR_PAINT_ADD_MENU),
        Some(true)
    );
    let ev = mesa.host.click_at(cx, cy);
    mesa.aplica(ev);
    mesa.assenta();

    let linha = mesa.rect(p::VECTOR_FILL_ROW).expect("a caixa do Fill");
    let ev = mesa
        .host
        .click_at(linha.x + linha.w * 0.5, linha.y + linha.h * 0.5);
    mesa.aplica(ev);
    assert_eq!(mesa.host.dropdown_is_open(p::VECTOR_FILL_ROW), Some(true));
    mesa.assenta();
    let chip = mesa
        .rect(t::VECTOR_FILL_KIND_LINEAR)
        .expect("o tipo no painel");
    assert!(
        chip.x + chip.w <= lista.x,
        "o painel abre à ESQUERDA da lista: {chip:?} × {lista:?}"
    );
}

/// O painel à esquerda fica à MESMA distância da lista a que ficava do painel de antes.
/// *Mutação: passar o rect do slot, e não o da lista, aos passes diferidos.*
#[test]
fn the_left_panel_keeps_its_distance_from_the_list() {
    let vao = |cartoes: bool| {
        forma_com_tinta();
        let mut mesa = Mesa::nova(cartoes);
        let linha = mesa.rect(p::VECTOR_FILL_ROW).expect("a caixa do Fill");
        let ev = mesa
            .host
            .click_at(linha.x + linha.w * 0.5, linha.y + linha.h * 0.5);
        mesa.aplica(ev);
        mesa.assenta();
        let fecho = mesa
            .rect(p::VECTOR_FILL_PROPS_CLOSE)
            .expect("o fecho do painel");
        let painel = mesa
            .host
            .store()
            .panel_rect(core_ids::VECTOR_PANEL)
            .expect("o rect");
        painel.x - (fecho.x + fecho.w)
    };
    let (novo, velho) = (vao(true), vao(false));
    assert!((novo - velho).abs() < 0.5, "cartões {novo} × antes {velho}");
}
