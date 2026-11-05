//! ⭐⭐ **O painel Vector em cartões, fatia G4** (`docs/CartoesUI/12_VECTOR_PLANO.md`): Text, Font,
//! Paragraph, Axes e Text on Path. Cada id é provado pelo gesto REAL, nos rects pintados, e o que chega
//! ao barramento é o MESMO do painel de antes.

use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::interaction::{WidgetEvent, dispatch_key};
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::widget::key_mark::{KeyMark, key_mark_id, set_key_marks};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{self, VectorPanelState};
use ph2d_panel_vector::{FontPreview, TextAxisSlot, VectorPanel, cards, ids as p};
use ph2d_tool_vector::ids as t;
use ph2d_ui_testkit::MockPanelHost;
use ph2d_vec_text::TextAlign;

pub(crate) const COLUNA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1340.0,
    h: 3200.0,
};

fn eixo(name: &str) -> TextAxisSlot {
    TextAxisSlot {
        name: name.to_owned(),
        min: 0.0,
        max: 100.0,
        value: 25.0,
    }
}

fn fonte(nome: &str) -> FontPreview {
    FontPreview {
        family: None,
        display: nome.to_owned(),
        outline: ph2d_vector::BezPath::new(),
        advance_em: 0.0,
    }
}

/// Um texto solto com tudo à vista: fonte variável, caixa Fixed, e a selecção permite prender.
fn texto() {
    state::set_length_suffix("px");
    ph2d_panel_vector::set_current_text_visible(true);
    ph2d_panel_vector::set_current_text(Some("Hello\nworld".to_owned()));
    ph2d_panel_vector::set_current_text_font(Some("Inter".to_owned()));
    ph2d_panel_vector::set_current_text_has_weight(true);
    ph2d_panel_vector::set_current_text_align(Some(TextAlign::Center));
    ph2d_panel_vector::set_current_text_wrap(Some(6.0));
    ph2d_panel_vector::set_current_text_axes(vec![
        eixo("Width"),
        eixo("Slant"),
        eixo("Optical Size"),
    ]);
    ph2d_panel_vector::set_current_text_font_previews(vec![fonte("Inter"), fonte("Noto Sans")]);
    ph2d_panel_vector::set_current_textpath(false, 0.0, false);
    ph2d_panel_vector::set_current_textpath_can_link(true);
    set_key_marks(BTreeMap::new());
}

pub(crate) struct Mesa {
    pub(crate) host: MockPanelHost,
    pub(crate) st: VectorPanelState,
    pub(crate) rects: Vec<(NodeId, Rect)>,
}

impl Mesa {
    pub(crate) fn nova(cartoes: bool) -> Self {
        Self::com(cartoes, texto)
    }

    /// A mesa com o estado que `semente` publica.
    pub(crate) fn com(cartoes: bool, semente: fn()) -> Self {
        cards::force_cards_on_this_thread(Some(cartoes));
        semente();
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

    pub(crate) fn presa(cartoes: bool) -> Self {
        let mut mesa = Self::nova(cartoes);
        ph2d_panel_vector::set_current_textpath(true, 0.25, false);
        mesa.assenta();
        mesa
    }

    pub(crate) fn assenta(&mut self) {
        for _ in 0..60 {
            ph2d_panel_vector::set_current_text_seed(Some([2.0, 700.0, 1.5, 0.1]));
            let (h, st) = (&mut self.host, &mut self.st);
            self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || h.paint::<VectorPanel>(st, COLUNA),
            );
        }
    }

    pub(crate) fn rect(&self, id: NodeId) -> Option<Rect> {
        self.rects
            .iter()
            .rev()
            .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| *r)
    }

    pub(crate) fn aplica(&mut self, eventos: Vec<WidgetEvent>) -> Vec<String> {
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

    pub(crate) fn clica(&mut self, id: NodeId) -> Vec<String> {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("{id:?} não pintado"));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(self.host.hit_at(cx, cy), Some(id), "outro id no centro");
        let ev = self.host.click_at(cx, cy);
        self.aplica(ev)
    }

    pub(crate) fn ponteiro(&mut self, kind: PointerKind, x: f32, y: f32) -> Vec<WidgetEvent> {
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

    pub(crate) fn arrasta_a_pega(&mut self, id: NodeId, dx: f32) -> Vec<WidgetEvent> {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("{id:?} não pintado"));
        let (x, y) = (r.x + m::ICON_CELL * 0.5, r.y + r.h * 0.5);
        let mut ev = self.ponteiro(PointerKind::Down, x, y);
        ev.extend(self.ponteiro(PointerKind::Move, x + dx * 0.5, y));
        ev.extend(self.ponteiro(PointerKind::Move, x + dx, y));
        ev
    }

    pub(crate) fn esc(&mut self) -> Vec<String> {
        let arena = bumpalo::Bump::new();
        let esc = ph2d_host::KeyEvent {
            keycode: 0x1B,
            modifiers: ph2d_host::Modifiers::default(),
            kind: ph2d_host::KeyKind::Down,
            timestamp_ns: 0,
        };
        let ev = dispatch_key(self.host.store_mut(), esc, &arena).to_vec();
        self.aplica(ev)
    }
}

/// `(chip, barra)` dos números ligados a uma barra do texto solto.
fn numeros() -> [(NodeId, NodeId); 5] {
    [
        (p::VECTOR_TEXT_SIZE_NUM, p::VECTOR_TEXT_SIZE),
        (p::VECTOR_TEXT_WEIGHT_NUM, p::VECTOR_TEXT_WEIGHT),
        (p::VECTOR_TEXT_LINE_HEIGHT_NUM, p::VECTOR_TEXT_LINE_HEIGHT),
        (p::VECTOR_TEXT_TRACKING_NUM, p::VECTOR_TEXT_TRACKING),
        (p::VECTOR_TEXT_WRAP_W_NUM, p::VECTOR_TEXT_WRAP_W),
    ]
}

fn cliques_solto() -> Vec<NodeId> {
    vec![
        p::VECTOR_TEXT_FONT_PREV,
        p::VECTOR_TEXT_FONT_NEXT,
        p::VECTOR_TEXT_FONT_IMPORT,
        p::VECTOR_TEXT_ALIGN_LEFT,
        p::VECTOR_TEXT_ALIGN_CENTER,
        p::VECTOR_TEXT_ALIGN_RIGHT,
        p::VECTOR_TEXT_WRAP_AUTO,
        p::VECTOR_TEXT_WRAP_FIXED,
        p::VECTOR_TEXTPATH_LINK,
        p::VECTOR_TEXTPATH_PICK,
    ]
}

fn cliques_preso() -> Vec<NodeId> {
    vec![
        p::VECTOR_TEXTPATH_FLIP_OFF,
        p::VECTOR_TEXTPATH_FLIP,
        p::VECTOR_TEXTPATH_DETACH,
    ]
}

/// O inventário: tudo pintado nos dois painéis, solto e preso.
#[test]
fn every_id_of_the_slice_is_painted_in_cards_and_in_the_old_panel() {
    for cartoes in [true, false] {
        let mesa = Mesa::nova(cartoes);
        let mut lista = cliques_solto();
        lista.extend(numeros().map(|n| n.0));
        lista.push(p::VECTOR_TEXT_FONT_DD);
        lista.extend((0..3).map(p::vector_text_axis_id));
        for id in lista {
            assert!(mesa.rect(id).is_some(), "solto {id:?} (cartões {cartoes})");
        }
        let mesa = Mesa::presa(cartoes);
        let mut lista = cliques_preso();
        lista.push(p::VECTOR_TEXTPATH_OFFSET_NUM);
        for id in lista {
            assert!(mesa.rect(id).is_some(), "preso {id:?} (cartões {cartoes})");
        }
    }
}

/// Nos cartões as barras não se pintam: o campo é a cara delas.
/// *Mutação: pôr um `slider_row` num corpo de G4 ⇒ vermelho.*
#[test]
fn no_old_bar_is_painted_in_the_slice() {
    let mesa = Mesa::nova(true);
    for (_, barra) in numeros() {
        assert!(mesa.rect(barra).is_none(), "barra {barra:?}");
    }
    let mesa = Mesa::presa(true);
    assert!(mesa.rect(p::VECTOR_TEXTPATH_OFFSET).is_none(), "offset");
}

/// ⭐⭐ **Mesmo id, mesmo evento**: o clique real produz no barramento o que produz no painel de antes.
/// *Mutação: trocar dois ids num corpo ⇒ vermelho.*
#[test]
fn every_click_sends_the_same_action_as_the_old_panel() {
    for id in cliques_solto() {
        let novo = Mesa::nova(true).clica(id);
        assert_eq!(novo, Mesa::nova(false).clica(id), "{id:?}");
        assert!(!novo.is_empty(), "{id:?} mudo");
    }
    for id in cliques_preso() {
        let novo = Mesa::presa(true).clica(id);
        assert_eq!(novo, Mesa::presa(false).clica(id), "{id:?}");
        assert!(!novo.is_empty(), "{id:?} mudo");
    }
}

/// A caixa abre a lista de sempre, por cima, e a linha escolhe e fecha como no painel de antes.
/// *Mutação: tirar `set_pending_font_dd` ⇒ vermelho.*
#[test]
fn the_font_box_opens_the_list_and_a_row_picks_and_closes() {
    let grava = |cartoes: bool| {
        let mut mesa = Mesa::nova(cartoes);
        mesa.clica(p::VECTOR_TEXT_FONT_DD);
        mesa.assenta();
        assert_eq!(
            mesa.host.dropdown_is_open(p::VECTOR_TEXT_FONT_DD),
            Some(true)
        );
        let accoes = mesa.clica(p::vector_text_font_option_id(1));
        mesa.assenta();
        assert_eq!(
            mesa.host.dropdown_is_open(p::VECTOR_TEXT_FONT_DD),
            Some(false),
            "não fechou"
        );
        accoes
    };
    let novo = grava(true);
    assert!(!novo.is_empty(), "a escolha não chegou");
    assert_eq!(novo, grava(false));
}

/// A lista tem a largura do nome mais largo COMO SE PINTA, não a da caixa: cada família no contorno dela.
/// *Mutação: voltar à porta sem a medida ⇒ vermelho.*
#[test]
fn the_font_list_is_as_wide_as_the_widest_painted_name() {
    let mut mesa = Mesa::nova(true);
    let mut longa = fonte("Fira Sans Compressed Extra Light");
    longa.outline.move_to((0.0, 0.0));
    longa.outline.line_to((30.0, 1.0));
    longa.advance_em = 30.0;
    ph2d_panel_vector::set_current_text_font_previews(vec![fonte("Inter"), longa]);
    mesa.clica(p::VECTOR_TEXT_FONT_DD);
    mesa.assenta();
    let caixa = mesa.rect(p::VECTOR_TEXT_FONT_DD).expect("a caixa");
    let linha = mesa
        .rect(p::vector_text_font_option_id(1))
        .expect("a linha");
    assert!(
        linha.w >= 30.0 * caixa.h * 0.62,
        "{linha:?} × a caixa {caixa:?}"
    );
}

/// ⭐ **Scrub de cada número**: o campo muda, a ferramenta ouve pela barra de sempre, a semente a meio
/// não o puxa, e o Esc devolve. *Mutação: tirar o `mark_card_field` ⇒ vermelho.*
#[test]
fn scrubbing_each_number_reaches_the_tool_and_escape_gives_it_back() {
    for (chip, barra) in numeros() {
        let mut mesa = Mesa::nova(true);
        let antes = mesa.host.store().number_value(chip).expect("semeado");
        let ev = mesa.arrasta_a_pega(chip, 60.0);
        assert!(
            ev.contains(&WidgetEvent::ValueChanged(barra)),
            "{chip:?} não falou pela barra: {ev:?}"
        );
        let accoes = mesa.aplica(ev);
        assert!(!accoes.is_empty(), "{chip:?} mudo");
        let durante = mesa.host.store().number_value(chip).expect("campo");
        assert!((durante - antes).abs() > 1e-6, "{chip:?} parado em {antes}");
        ph2d_panel_vector::set_current_text_seed(Some([2.0, 700.0, 1.5, 0.1]));
        mesa.host.paint::<VectorPanel>(&mut mesa.st, COLUNA);
        let depois = mesa.host.store().number_value(chip).expect("campo");
        assert!((depois - durante).abs() < 1e-6, "{chip:?}: a semente puxou");
        mesa.esc();
        let voltou = mesa.host.store().number_value(chip).expect("campo");
        assert!(
            (voltou - antes).abs() < 1e-6,
            "{chip:?}: o Esc deu {voltou}"
        );
    }
    // O offset do caminho e um eixo, as duas outras espécies.
    let mut mesa = Mesa::presa(true);
    let antes = mesa
        .host
        .store()
        .number_value(p::VECTOR_TEXTPATH_OFFSET_NUM)
        .expect("semeado");
    assert!((antes - 0.25).abs() < 1e-6, "semeou {antes}");
    let ev = mesa.arrasta_a_pega(p::VECTOR_TEXTPATH_OFFSET_NUM, 20.0);
    assert!(ev.contains(&WidgetEvent::ValueChanged(p::VECTOR_TEXTPATH_OFFSET)));
    let mut mesa = Mesa::nova(true);
    let eixo = p::vector_text_axis_id(1);
    let ev = mesa.arrasta_a_pega(eixo, 20.0);
    assert!(ev.contains(&WidgetEvent::ValueChanged(eixo)), "{ev:?}");
    assert!(!mesa.aplica(ev).is_empty(), "o eixo não chegou");
}

/// Escrever o número manda à ferramenta o MESMO que no painel de antes.
#[test]
fn typing_a_number_sends_the_same_as_the_old_panel() {
    for (chip, texto) in [
        (p::VECTOR_TEXT_SIZE_NUM, "3"),
        (p::VECTOR_TEXT_WEIGHT_NUM, "500"),
        (p::VECTOR_TEXT_LINE_HEIGHT_NUM, "2"),
        (p::VECTOR_TEXT_TRACKING_NUM, "0.2"),
        (p::VECTOR_TEXT_WRAP_W_NUM, "9"),
        (p::vector_text_axis_id(0), "50"),
    ] {
        let grava = |cartoes: bool| {
            let mut mesa = Mesa::nova(cartoes);
            let ev = mesa.host.type_into_number(chip, texto);
            mesa.aplica(ev)
        };
        let novo = grava(true);
        assert!(!novo.is_empty(), "{chip:?} mudo");
        assert_eq!(novo, grava(false), "{chip:?}");
    }
}

/// ⭐ As perguntas de antes continuam a mandar. *Mutação: tirar cada `if` ⇒ vermelho.*
#[test]
fn what_the_text_cannot_use_is_not_offered() {
    let mut mesa = Mesa::nova(true);
    ph2d_panel_vector::set_current_text_has_weight(false);
    mesa.assenta();
    assert!(
        mesa.rect(p::VECTOR_TEXT_WEIGHT_NUM).is_none(),
        "Weight numa fonte estática"
    );
    assert!(mesa.rect(p::VECTOR_TEXT_SIZE_NUM).is_some(), "Size");

    let mut mesa = Mesa::nova(true);
    ph2d_panel_vector::set_current_text_wrap(None);
    mesa.assenta();
    assert!(
        mesa.rect(p::VECTOR_TEXT_WRAP_W_NUM).is_none(),
        "largura em Auto"
    );
    assert!(mesa.rect(p::VECTOR_TEXT_WRAP_FIXED).is_some(), "Fixed");

    let mesa = Mesa::presa(true);
    for id in [
        p::VECTOR_TEXT_WRAP_AUTO,
        p::VECTOR_TEXT_WRAP_FIXED,
        p::VECTOR_TEXT_WRAP_W_NUM,
        p::VECTOR_TEXTPATH_LINK,
        p::VECTOR_TEXTPATH_PICK,
    ] {
        assert!(mesa.rect(id).is_none(), "{id:?} num texto preso");
    }
    assert!(mesa.rect(p::VECTOR_TEXT_ALIGN_LEFT).is_some(), "Align fica");

    let mut mesa = Mesa::nova(true);
    assert!(
        mesa.rect(p::VECTOR_TEXTPATH_DETACH).is_none(),
        "Detach solto"
    );
    ph2d_panel_vector::set_current_textpath_can_link(false);
    mesa.assenta();
    assert!(
        mesa.rect(p::VECTOR_TEXTPATH_LINK).is_none(),
        "prender sem par"
    );
    assert!(
        mesa.rect(p::VECTOR_TEXTPATH_PICK).is_some(),
        "o Picker fica"
    );

    // Sem eixos não há cartão; com eixos a mais, só até ao tecto do registo.
    let mut mesa = Mesa::nova(true);
    ph2d_panel_vector::set_current_text_axes(Vec::new());
    mesa.assenta();
    assert!(mesa.rect(t::VECTOR_SECTION_AXES).is_none(), "Axes vazio");
    let n = p::MAX_TEXT_VARIATION_AXES;
    ph2d_panel_vector::set_current_text_axes(
        vec![eixo("E"); 0]
            .into_iter()
            .chain((0..=n).map(|_| eixo("E")))
            .collect(),
    );
    mesa.assenta();
    assert!(
        mesa.rect(p::vector_text_axis_id(n - 1)).is_some(),
        "o último"
    );
    assert!(
        mesa.rect(p::vector_text_axis_id(n)).is_none(),
        "acima do tecto"
    );
}

/// O passo e o valor de fábrica de cada campo. *Mutação: tirar uma linha do `prepare` ⇒ vermelho.*
#[test]
fn each_field_has_its_step_and_factory_value() {
    use ph2d_tool_vector::params as q;
    let mesa = Mesa::presa(true);
    let spec = |id: NodeId| {
        mesa.host
            .store()
            .card_field(id)
            .unwrap_or_else(|| panic!("{id:?} sem marca"))
    };
    for (id, fabrica) in [
        (p::VECTOR_TEXT_SIZE_NUM, q::DEFAULT_TEXT_SIZE),
        (p::VECTOR_TEXT_WEIGHT_NUM, q::DEFAULT_TEXT_WEIGHT),
        (p::VECTOR_TEXT_LINE_HEIGHT_NUM, q::DEFAULT_TEXT_LINE_HEIGHT),
        (p::VECTOR_TEXT_TRACKING_NUM, q::DEFAULT_TEXT_TRACKING),
        (p::VECTOR_TEXT_WRAP_W_NUM, q::DEFAULT_TEXT_WRAP),
        (p::VECTOR_TEXTPATH_OFFSET_NUM, 0.0),
    ] {
        assert_eq!(spec(id).default, Some(fabrica), "{id:?}");
    }
    assert_eq!(
        spec(p::VECTOR_TEXT_WEIGHT_NUM).scrub,
        1.0,
        "o peso anda inteiro"
    );
    assert!(
        spec(p::VECTOR_TEXT_SIZE_NUM).scrub < 1.0,
        "o tamanho é fino"
    );
    assert_eq!(spec(p::vector_text_axis_id(0)).step, 2.5, "faixa ÷ 40");
}

/// A marca de animação de cada campo mora na barra, como no corpo de antes.
/// *Mutação: tirar `.key` de uma linha ⇒ vermelho.*
#[test]
fn the_animation_marks_stay() {
    let marcados = [
        p::VECTOR_TEXT_SIZE,
        p::VECTOR_TEXT_WEIGHT,
        p::VECTOR_TEXT_LINE_HEIGHT,
        p::VECTOR_TEXT_TRACKING,
        p::VECTOR_TEXT_WRAP_W,
        p::VECTOR_TEXT_FONT_DD,
        p::VECTOR_TEXT_ALIGN_LEFT,
        p::VECTOR_TEXT_WRAP_AUTO,
    ];
    let mut mesa = Mesa::nova(true);
    set_key_marks(marcados.iter().map(|id| (*id, KeyMark::Chave)).collect());
    mesa.assenta();
    for id in marcados {
        assert!(mesa.rect(key_mark_id(id)).is_some(), "sem marca: {id:?}");
    }
    let mut mesa = Mesa::presa(true);
    let marcados = [p::VECTOR_TEXTPATH_OFFSET, p::VECTOR_TEXTPATH_FLIP_OFF];
    set_key_marks(marcados.iter().map(|id| (*id, KeyMark::Chave)).collect());
    mesa.assenta();
    for id in marcados {
        assert!(mesa.rect(key_mark_id(id)).is_some(), "sem marca: {id:?}");
    }
    set_key_marks(BTreeMap::new());
}

/// Todo ícone sem texto tem dica. *Mutação: tirar uma linha da tabela de dicas ⇒ vermelho.*
#[test]
fn the_icon_only_controls_explain_themselves() {
    let mesa = Mesa::nova(true);
    for id in [
        p::VECTOR_TEXT_FONT_PREV,
        p::VECTOR_TEXT_FONT_NEXT,
        p::VECTOR_TEXT_ALIGN_LEFT,
        p::VECTOR_TEXT_ALIGN_CENTER,
        p::VECTOR_TEXT_ALIGN_RIGHT,
    ] {
        assert!(
            mesa.host.store().tooltip_for(id).is_some(),
            "{id:?} sem dica"
        );
    }
}

/// Dobrar um cartão de texto esconde o corpo e o título devolve-o.
#[test]
fn folding_a_text_card_hides_its_body() {
    let mut mesa = Mesa::nova(true);
    mesa.host.set_collapsed(t::VECTOR_SECTION_PARAGRAPH, true);
    mesa.assenta();
    assert!(mesa.rect(p::VECTOR_TEXT_LINE_HEIGHT_NUM).is_none());
    assert!(
        mesa.rect(p::VECTOR_TEXT_SIZE_NUM).is_some(),
        "os outros ficam"
    );
    mesa.host.set_collapsed(t::VECTOR_SECTION_PARAGRAPH, false);
    mesa.assenta();
    assert!(mesa.rect(p::VECTOR_TEXT_LINE_HEIGHT_NUM).is_some());
}

/// `PH2D_CARDS=0`: o corpo de antes, com as barras.
#[test]
fn with_cards_off_the_old_bars_come_back() {
    let mesa = Mesa::nova(false);
    for (_, barra) in numeros() {
        assert!(mesa.rect(barra).is_some(), "barra {barra:?}");
    }
}

/// ⭐ **O «+» de Fill & Stroke com o cartão dobrado**: o «+» continua no título, e a lista abre.
/// O corpo dobrado não corre: a âncora não pode depender dele.
/// *Mutação: ancorar outra vez no corpo ⇒ vermelho.*
#[test]
fn the_plus_of_fill_and_stroke_opens_on_a_folded_card() {
    let mut mesa = Mesa::nova(true);
    state::set_stroke_present(Some(false));
    state::set_base_paints(Some([state::BasePaint::Absent; 2]));
    mesa.host.set_collapsed(t::VECTOR_SECTION_FILL, true);
    mesa.assenta();
    mesa.clica(p::VECTOR_PAINT_ADD_MENU);
    mesa.assenta();
    assert_eq!(
        mesa.host.dropdown_is_open(p::VECTOR_PAINT_ADD_MENU),
        Some(true)
    );
    let opcao = mesa.rect(p::VECTOR_STROKE_PRESENT);
    assert!(opcao.is_some(), "a lista não abriu com o cartão dobrado");
    let lista = mesa
        .host
        .store()
        .panel_rect(ph2d_editor_core::ids::VECTOR_PANEL)
        .expect("a lista");
    let o = opcao.unwrap();
    assert!(o.x + o.w <= lista.x + lista.w, "{o:?} × {lista:?}");
    assert!(!mesa.clica(p::VECTOR_STROKE_PRESENT).is_empty(), "mudo");
}

/// ⭐⭐ A lei da compactação (05 §4) no texto solto e no preso. Offset e Side tentam dividir a linha; a
/// 300 px os dois lados não cabem inteiros na metade, e a escolha desce sem cortar nome.
#[test]
fn the_slice_passes_the_compaction_census() {
    let mut mesa = Mesa::nova(true);
    super::g5_os_cartoes::sem_acusacao("solto", || mesa.assenta());
    let mut mesa = Mesa::presa(true);
    super::g5_os_cartoes::sem_acusacao("preso", || mesa.assenta());
}

/// Lei da compactação: Offset e Side dividem a linha na largura de fábrica da lista, com os dois lados
/// inteiros. *Mutação: os nomes longos do painel de antes ⇒ a escolha desce.*
#[test]
fn offset_e_side_dividem_a_linha_na_largura_de_fabrica() {
    let mut mesa = Mesa::presa(true);
    let janela = Rect::new(0.0, 0.0, 1930.0, 3200.0);
    for _ in 0..60 {
        let (h, st) = (&mut mesa.host, &mut mesa.st);
        mesa.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || h.paint::<VectorPanel>(st, janela),
        );
    }
    let campo = mesa.rect(p::VECTOR_TEXTPATH_OFFSET_NUM).expect("Offset");
    let lado = mesa.rect(p::VECTOR_TEXTPATH_FLIP).expect("Other");
    let coluna = lado.x + lado.w - campo.x;
    assert!((250.0..=262.0).contains(&coluna), "a coluna mede {coluna}");
    assert!(
        lado.y < campo.y + campo.h && campo.y < lado.y + lado.h,
        "{campo:?} × {lado:?}"
    );
}
