//! ⭐⭐ **O painel Vector em cartões, fatia G2** (`docs/CartoesUI/12_VECTOR_PLANO.md`): Appearance, Fill &
//! Stroke e os painéis à esquerda. F44/F45 são o layout aprovado: muda a cara, ficam as linhas, as
//! colunas, os ids e os eventos. Cada id é provado pelo gesto REAL, nos rects pintados, e o que chega ao
//! barramento é o MESMO do painel de antes.

use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{WidgetEvent, dispatch_key};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::key_mark::{KeyMark, key_mark_id, set_key_marks};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{self, Appearance, BasePaint, PaintRow, VectorPanelState};
use ph2d_panel_vector::{FillKind, VectorPanel, cards, ids as p};
use ph2d_tool_vector::ids as t;
use ph2d_ui_testkit::MockPanelHost;
use ph2d_vec_scene::{BlendMode, StackItem, StrokeAlign};

/// A coluna à direita e larga: o painel à esquerda tem de caber dentro da região.
const COLUNA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1340.0,
    h: 3200.0,
};

fn camada(is_fill: bool) -> PaintRow {
    PaintRow {
        align: StrokeAlign::Centre,
        ramp: None,
        is_fill,
        color: [0x33, 0x66, 0xCC, 0xFF],
        width: 2.0,
        enabled: true,
        opacity: 0.5,
        blend: BlendMode::Normal,
        offset: [3.0, 4.0],
        dilate: 2.0,
        dilate_join: 1,
        mixed: Default::default(),
    }
}

/// Uma forma com tudo à vista: a tinta e o traço de base e uma camada de cada espécie.
fn forma(tipo: FillKind) {
    state::set_length_suffix("px");
    state::set_current_appearance(Some(Appearance {
        opacity: 0.25,
        blend: BlendMode::Normal,
        layers: vec![camada(true), camada(false)],
        mixed_opacity: false,
        mixed_blend: false,
        layers_differ: false,
        order: Vec::new(),
    }));
    state::set_base_paints(Some([BasePaint::Shown; 2]));
    state::set_current_fill(Some(tipo), Some(30.0));
    state::set_current_fill_rule(Some(ph2d_panel_vector::state::PathFillRule::NonZero));
    state::set_current_grad_influence(Some(2.0));
    state::set_current_grad_jitter(Some(0.5));
    state::set_stroke_present(Some(true));
    state::set_stroke_above_effects(Some(false));
    state::set_stroke_paint_kind(Some(state::StrokePaintKind::Solid));
    state::set_token_bindings(Some(Default::default()));
    state::set_current_selection_count(1);
    state::close_open_layer();
    set_key_marks(BTreeMap::new());
}

struct Mesa {
    host: MockPanelHost,
    st: VectorPanelState,
    rects: Vec<(NodeId, Rect)>,
    coluna: Rect,
}

impl Mesa {
    fn nova(cartoes: bool, tipo: FillKind) -> Self {
        Self::na_coluna(cartoes, tipo, COLUNA)
    }

    fn na_coluna(cartoes: bool, tipo: FillKind, coluna: Rect) -> Self {
        cards::force_cards_on_this_thread(Some(cartoes));
        forma(tipo);
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        host.open_all_sections();
        let mut mesa = Self {
            host,
            st: VectorPanelState,
            rects: Vec::new(),
            coluna,
        };
        mesa.assenta();
        mesa
    }

    fn assenta(&mut self) {
        for _ in 0..60 {
            let (h, st, coluna) = (&mut self.host, &mut self.st, self.coluna);
            self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || h.paint::<VectorPanel>(st, coluna),
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
        self.aplica(ev)
    }

    /// Abre o `Dropdown` `id` pelo clique e pinta: os painéis à esquerda e os menus.
    fn abre(&mut self, id: NodeId) {
        self.clica(id);
        self.assenta();
        assert_eq!(
            self.host.dropdown_is_open(id),
            Some(true),
            "{id:?} não abriu"
        );
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

    fn esc(&mut self) -> Vec<String> {
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

/// O que cada lugar mostra. Os mesmos ids com os cartões e no painel de antes (salvo as barras, que
/// nos cartões só existem no store: o campo faz o trabalho delas).
fn da_lista() -> Vec<NodeId> {
    let mut v = vec![
        p::VECTOR_OBJ_OPACITY_NUM,
        p::VECTOR_OBJ_BLEND,
        p::VECTOR_PAINT_ADD_MENU,
        t::VECTOR_FILL_SWATCH,
        t::VECTOR_FILL_OPACITY_NUM,
        p::VECTOR_FILL_ROW,
        p::VECTOR_FILL_EYE,
        p::VECTOR_FILL_PRESENT,
        t::VECTOR_STROKE_SWATCH,
        t::VECTOR_STROKE_OPACITY_NUM,
        t::VECTOR_WIDTH_NUM,
        p::VECTOR_STROKE_SETTINGS,
        p::VECTOR_STROKE_EYE,
        p::VECTOR_STROKE_PRESENT,
    ];
    for i in 0..2 {
        v.extend([
            p::vector_paint_swatch_id(i),
            p::vector_paint_row_opacity_id(i),
            p::vector_paint_row_id(i),
            p::vector_paint_eye_id(i),
            p::vector_paint_del_id(i),
        ]);
    }
    v.push(p::vector_paint_row_width_id(1));
    v
}

fn do_fill_linear() -> Vec<NodeId> {
    vec![
        p::VECTOR_FILL_RULE_NONZERO,
        p::VECTOR_FILL_RULE_EVENODD,
        t::VECTOR_FILL_KIND_SOLID,
        t::VECTOR_FILL_KIND_LINEAR,
        t::VECTOR_FILL_KIND_RADIAL,
        t::VECTOR_FILL_KIND_MULTI,
        t::VECTOR_FILL_KIND_PATTERN,
        p::VECTOR_GRAD_ADD_STOP,
        p::VECTOR_GRAD_REMOVE_STOP,
        p::VECTOR_TOKEN_FILL,
        p::VECTOR_FILL_PROPS_CLOSE,
    ]
}

fn cliques_do_traco() -> Vec<NodeId> {
    let mut v = vec![
        t::VECTOR_ALIGN_INNER,
        t::VECTOR_ALIGN_CENTRE,
        t::VECTOR_ALIGN_OUTER,
        t::VECTOR_STROKE_KIND_SOLID,
        t::VECTOR_STROKE_KIND_PATTERN,
        t::VECTOR_STROKE_KIND_BRUSH,
        t::VECTOR_CAP_BUTT,
        t::VECTOR_CAP_ROUND,
        t::VECTOR_CAP_SQUARE,
        t::VECTOR_JOIN_MITER,
        t::VECTOR_JOIN_ROUND,
        t::VECTOR_JOIN_BEVEL,
        p::VECTOR_STROKE_ABOVE_EFFECTS,
        t::VECTOR_MARKER_BOTH,
        t::VECTOR_EXPAND_POWER_STROKE,
    ];
    v.extend(
        (0..ph2d_vec_scene::WIDTH_PRESETS
            .len()
            .min(p::MAX_WIDTH_PRESETS))
            .map(p::vector_width_preset_id),
    );
    v
}

fn numeros_do_traco() -> Vec<NodeId> {
    vec![
        t::VECTOR_DASH_IDS[0],
        t::VECTOR_GAP_IDS[0],
        t::VECTOR_MARKER_SCALE,
        t::VECTOR_MARKER_SCALE_END,
        t::VECTOR_MARKER_ROUND,
        t::VECTOR_EXPAND_W_START_NUM,
        t::VECTOR_EXPAND_W_MID_NUM,
        t::VECTOR_EXPAND_W_END_NUM,
        t::VECTOR_EXPAND_W_POS_NUM,
    ]
}

fn menus_do_traco() -> Vec<NodeId> {
    vec![
        p::VECTOR_TOKEN_WIDTH,
        p::VECTOR_TOKEN_STROKE,
        t::VECTOR_MARKER_START_DD,
        t::VECTOR_MARKER_END_DD,
        p::VECTOR_STROKE_PROPS_CLOSE,
    ]
}

fn da_camada() -> Vec<NodeId> {
    vec![
        p::VECTOR_PAINT_ALIGN_INNER,
        p::VECTOR_PAINT_ALIGN_CENTRE,
        p::VECTOR_PAINT_ALIGN_OUTER,
        p::VECTOR_PAINT_DX,
        p::VECTOR_PAINT_DY,
        p::VECTOR_PAINT_DILATE,
        p::VECTOR_PAINT_JOIN_MITER,
        p::VECTOR_PAINT_JOIN_ROUND,
        p::VECTOR_PAINT_JOIN_BEVEL,
        p::VECTOR_PAINT_BLEND,
        p::VECTOR_PAINT_PROPS_CLOSE,
    ]
}

/// O inventário: tudo registado, e tudo pintado nos dois painéis, no mesmo estado alcançável.
#[test]
fn every_id_of_the_slice_is_painted_in_cards_and_in_the_old_panel() {
    for cartoes in [true, false] {
        let confere = |mesa: &Mesa, ids: &[NodeId], onde: &str| {
            for (i, id) in ids.iter().enumerate() {
                let n = format!("{onde} [{i}] (cartões {cartoes})");
                let s = mesa.host.store();
                assert!(
                    s.get(*id).is_some() || s.is_picker_swatch(*id),
                    "sem registo: {n}"
                );
                assert!(mesa.rect(*id).is_some(), "não pintado: {n}");
            }
        };
        let mut mesa = Mesa::nova(cartoes, FillKind::Linear);
        confere(&mesa, &da_lista(), "a lista");
        mesa.abre(p::VECTOR_FILL_ROW);
        confere(&mesa, &do_fill_linear(), "o Fill linear");
        confere(&mesa, &[t::VECTOR_GRAD_ANGLE_NUM], "o ângulo");

        let mut mesa = Mesa::nova(cartoes, FillKind::MultiPoint);
        mesa.abre(p::VECTOR_FILL_ROW);
        confere(
            &mesa,
            &[
                p::VECTOR_GRAD_ADD_POINT,
                p::VECTOR_GRAD_REMOVE_POINT,
                t::VECTOR_GRAD_INFLUENCE_NUM,
                t::VECTOR_GRAD_JITTER_NUM,
            ],
            "o Fill multi",
        );

        let mut mesa = Mesa::nova(cartoes, FillKind::Linear);
        mesa.abre(p::VECTOR_STROKE_SETTINGS);
        confere(&mesa, &cliques_do_traco(), "Stroke settings");
        confere(&mesa, &numeros_do_traco(), "os números do traço");
        confere(&mesa, &menus_do_traco(), "os menus do traço");

        let mut mesa = Mesa::nova(cartoes, FillKind::Linear);
        mesa.abre(p::vector_paint_row_id(1));
        confere(&mesa, &da_camada(), "a camada de traço");
    }
}

/// Nos cartões as barras não se pintam: o campo é a cara delas (05, a pega faz o trabalho).
/// *Mutação: pôr um `slider_row` num corpo de G2 ⇒ vermelho.*
#[test]
fn no_old_bar_is_painted_in_the_slice() {
    let barras = [
        p::VECTOR_OBJ_OPACITY,
        p::VECTOR_GRAD_ANGLE,
        p::VECTOR_GRAD_INFLUENCE,
        p::VECTOR_GRAD_JITTER,
        p::VECTOR_EXPAND_W_START,
        p::VECTOR_EXPAND_W_MID,
        p::VECTOR_EXPAND_W_END,
        p::VECTOR_EXPAND_W_POS,
    ];
    for (tipo, abrir) in [
        (FillKind::Linear, p::VECTOR_FILL_ROW),
        (FillKind::MultiPoint, p::VECTOR_FILL_ROW),
        (FillKind::Linear, p::VECTOR_STROKE_SETTINGS),
    ] {
        let mut mesa = Mesa::nova(true, tipo);
        mesa.abre(abrir);
        for b in barras {
            assert!(mesa.rect(b).is_none(), "a barra {b:?} pintou-se");
        }
    }
}

/// ⭐⭐ **Mesmo id, mesmo evento**: o clique real em cada opção e verbo produz no barramento exactamente
/// o que produz no painel de antes. *Mutação: trocar dois ids num `Seg` ⇒ vermelho.*
#[test]
fn every_click_sends_the_same_action_as_the_old_panel() {
    let grava = |cartoes: bool, abrir: NodeId, ids: &[NodeId]| -> Vec<(NodeId, Vec<String>)> {
        ids.iter()
            .map(|id| {
                // Uma mesa por clique: um verbo pode fechar o painel (⚙ de camada, X).
                let mut mesa = Mesa::nova(cartoes, FillKind::Linear);
                mesa.abre(abrir);
                (*id, mesa.clica(*id))
            })
            .collect()
    };
    let simples: Vec<NodeId> = do_fill_linear()
        .into_iter()
        .filter(|i| *i != p::VECTOR_TOKEN_FILL)
        .collect();
    for (abrir, ids) in [
        (p::VECTOR_FILL_ROW, simples),
        (p::VECTOR_STROKE_SETTINGS, cliques_do_traco()),
        (p::vector_paint_row_id(1), {
            let mut v = da_camada();
            v.retain(|i| *i != p::VECTOR_PAINT_DX && *i != p::VECTOR_PAINT_DY);
            v.retain(|i| *i != p::VECTOR_PAINT_DILATE && *i != p::VECTOR_PAINT_BLEND);
            v
        }),
    ] {
        let novo = grava(true, abrir, &ids);
        let velho = grava(false, abrir, &ids);
        assert_eq!(novo, velho, "o barramento mudou entre os dois painéis");
        assert!(novo.iter().any(|(_, a)| !a.is_empty()), "nada chegou");
    }
    // Os botões da linha: olho e «−», da base e da camada.
    let linha = [
        p::VECTOR_FILL_EYE,
        p::VECTOR_FILL_PRESENT,
        p::VECTOR_STROKE_EYE,
        p::VECTOR_STROKE_PRESENT,
        p::vector_paint_eye_id(0),
        p::vector_paint_del_id(0),
    ];
    let grava = |cartoes: bool| -> Vec<Vec<String>> {
        linha
            .iter()
            .map(|id| Mesa::nova(cartoes, FillKind::Linear).clica(*id))
            .collect()
    };
    assert_eq!(grava(true), grava(false));
}

fn valores(accoes: &[String], id: NodeId) -> Vec<f64> {
    use ph2d_editor_core::action_bus::EditorAction;
    use ph2d_editor_core::tool::PanelEvent;
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

/// `(campo, o painel a abrir, o id que a ferramenta ouve)`: um campo ligado a uma barra fala pela barra.
fn numeros() -> Vec<(NodeId, Option<NodeId>, NodeId)> {
    let traco = Some(p::VECTOR_STROKE_SETTINGS);
    let camada = Some(p::vector_paint_row_id(1));
    let fill = Some(p::VECTOR_FILL_ROW);
    vec![
        (p::VECTOR_OBJ_OPACITY_NUM, None, p::VECTOR_OBJ_OPACITY),
        (t::VECTOR_FILL_OPACITY_NUM, None, t::VECTOR_FILL_OPACITY),
        (t::VECTOR_STROKE_OPACITY_NUM, None, t::VECTOR_STROKE_OPACITY),
        (t::VECTOR_WIDTH_NUM, None, t::VECTOR_WIDTH),
        (t::VECTOR_GRAD_ANGLE_NUM, fill, p::VECTOR_GRAD_ANGLE),
        (t::VECTOR_DASH_IDS[0], traco, t::VECTOR_DASH_IDS[0]),
        (t::VECTOR_GAP_IDS[0], traco, t::VECTOR_GAP_IDS[0]),
        (t::VECTOR_MARKER_SCALE, traco, t::VECTOR_MARKER_SCALE),
        (
            t::VECTOR_MARKER_SCALE_END,
            traco,
            t::VECTOR_MARKER_SCALE_END,
        ),
        (t::VECTOR_MARKER_ROUND, traco, t::VECTOR_MARKER_ROUND),
        (p::VECTOR_PAINT_DX, camada, p::VECTOR_PAINT_DX),
        (p::VECTOR_PAINT_DY, camada, p::VECTOR_PAINT_DY),
        (p::VECTOR_PAINT_DILATE, camada, p::VECTOR_PAINT_DILATE),
    ]
}

/// ⭐ **Scrub de cada número**: o campo muda, a ferramenta ouve no id de sempre, uma semente a meio não
/// o puxa, e o Esc devolve. *Mutação: tirar o guarda da mão de `paint_dashes::seed` ⇒ vermelho.*
#[test]
fn scrubbing_each_number_reaches_the_tool_and_escape_gives_it_back() {
    for (k, (id, abrir, ouve)) in numeros().into_iter().enumerate() {
        let mut mesa = Mesa::nova(true, FillKind::Linear);
        if let Some(a) = abrir {
            mesa.abre(a);
        }
        let antes = mesa.host.store().number_value(id).expect("semeado");
        // Um número no tecto só anda para baixo.
        let dx = if antes >= 100.0 { -40.0 } else { 40.0 };
        let ev = mesa.arrasta_a_pega(id, dx);
        assert!(
            ev.contains(&WidgetEvent::ValueChanged(id)),
            "{id:?} não avisou: {ev:?}"
        );
        let ev2 = ev.clone();
        let accoes = mesa.aplica(ev);
        assert!(
            !valores(&accoes, ouve).is_empty(),
            "{id:?} [{k}]: a ferramenta não ouviu {ouve:?}: {accoes:?} (eventos {ev2:?})"
        );
        let durante = mesa.host.store().number_value(id).expect("campo");
        assert!(
            (durante - antes).abs() > 1e-6,
            "{id:?} não se moveu: {antes} → {durante}"
        );
        mesa.host.paint::<VectorPanel>(&mut mesa.st, COLUNA);
        let depois = mesa.host.store().number_value(id).expect("campo");
        assert!(
            (depois - durante).abs() < 1e-6,
            "{id:?}: a semente puxou {durante} → {depois}"
        );
        mesa.esc();
        let voltou = mesa.host.store().number_value(id).expect("campo");
        assert!((voltou - antes).abs() < 1e-6, "{id:?}: o Esc deu {voltou}");
    }
}

/// Os números do perfil autoram pela barra deles, que a shell lê: o scrub tem de a mover.
#[test]
fn a_profile_number_moves_its_bar() {
    for (chip, barra) in [
        (t::VECTOR_EXPAND_W_START_NUM, p::VECTOR_EXPAND_W_START),
        (t::VECTOR_EXPAND_W_MID_NUM, p::VECTOR_EXPAND_W_MID),
        (t::VECTOR_EXPAND_W_END_NUM, p::VECTOR_EXPAND_W_END),
        (t::VECTOR_EXPAND_W_POS_NUM, p::VECTOR_EXPAND_W_POS),
    ] {
        let mut mesa = Mesa::nova(true, FillKind::Linear);
        mesa.abre(p::VECTOR_STROKE_SETTINGS);
        let antes = mesa.host.store().slider(barra).expect("a barra").1;
        let ev = mesa.arrasta_a_pega(chip, 30.0);
        assert!(ev.contains(&WidgetEvent::ValueChanged(barra)), "{ev:?}");
        mesa.aplica(ev);
        let depois = mesa.host.store().slider(barra).expect("a barra").1;
        assert!((depois - antes).abs() > 1e-4, "{antes} → {depois}");
    }
}

/// A opacidade e o Weight da LINHA da camada falam pela porta da camada, apontando-a como a aberta.
#[test]
fn a_layer_line_speaks_through_the_layer_door() {
    for (id, ouve) in [
        (p::vector_paint_row_opacity_id(1), p::VECTOR_PAINT_OPACITY),
        (p::vector_paint_row_width_id(1), p::VECTOR_PAINT_WIDTH),
    ] {
        let mut mesa = Mesa::nova(true, FillKind::Linear);
        let ev = mesa.arrasta_a_pega(id, 40.0);
        let accoes = mesa.aplica(ev);
        assert!(!valores(&accoes, ouve).is_empty(), "{id:?}: {accoes:?}");
        assert_eq!(
            state::open_layer_index(),
            Some(1),
            "{id:?} não apontou a camada"
        );
    }
}

/// O que a linha mostra é o da forma, e não o de fábrica: o corpo de antes recebia o valor ao pintar.
/// *Mutação: tirar `semeia_as_linhas` do `prepare` ⇒ vermelho.*
#[test]
fn the_fields_show_the_published_shape_and_a_mix_reads_mixed() {
    let mesa = Mesa::nova(true, FillKind::Linear);
    let s = mesa.host.store();
    assert_eq!(s.number_value(p::VECTOR_OBJ_OPACITY_NUM), Some(25.0));
    assert_eq!(
        s.number_value(p::vector_paint_row_opacity_id(1)),
        Some(50.0)
    );
    assert_eq!(s.number_value(p::vector_paint_row_width_id(1)), Some(2.0));

    let mut a = state::current_appearance().expect("a forma");
    a.layers[1].mixed.opacity = true;
    a.mixed_opacity = true;
    state::set_current_appearance(Some(a));
    let mut mesa = mesa;
    mesa.assenta();
    let s = mesa.host.store();
    assert!(s.number_input_mixed(p::vector_paint_row_opacity_id(1)));
    assert!(s.number_input_mixed(p::VECTOR_OBJ_OPACITY_NUM));
    assert!(!s.number_input_mixed(p::vector_paint_row_width_id(1)));
}

/// ⭐ **F44c no cartão**: todas as linhas têm as mesmas colunas, a cor é um quadrado, e o Weight de `3`
/// fica numa fila na largura de fábrica. *Mutação: voltar os botões a `row_h` ⇒ a linha parte.*
#[test]
fn all_lines_share_their_columns_and_a_small_weight_stays_on_one_row() {
    let estreita = Rect {
        x: 0.0,
        y: 0.0,
        w: 1340.0,
        h: 1600.0,
    };
    let mesa = Mesa::na_coluna(true, FillKind::Linear, estreita);
    let lista = mesa
        .host
        .store()
        .panel_rect(core_ids::VECTOR_PANEL)
        .expect("a lista");
    assert!(lista.w < 320.0, "a largura de fábrica: {lista:?}");
    let [fs, ss] = [t::VECTOR_FILL_SWATCH, t::VECTOR_STROKE_SWATCH].map(|i| mesa.rect(i).unwrap());
    let [fe, se] = [p::VECTOR_FILL_EYE, p::VECTOR_STROKE_EYE].map(|i| mesa.rect(i).unwrap());
    let [fp, sp] =
        [t::VECTOR_FILL_OPACITY_NUM, t::VECTOR_STROKE_OPACITY_NUM].map(|i| mesa.rect(i).unwrap());
    let peso = mesa.rect(t::VECTOR_WIDTH_NUM).unwrap();
    assert!(
        (fs.x - ss.x).abs() < 0.5 && (fs.w - fs.h).abs() < 0.5,
        "a cor"
    );
    assert!((fe.x - se.x).abs() < 0.5, "o olho: {fe:?} × {se:?}");
    assert!((fp.x - sp.x).abs() < 0.5, "o %: {fp:?} × {sp:?}");
    assert!((sp.y - peso.y).abs() < 0.5, "numa fila: {sp:?} × {peso:?}");
    assert!((sp.y - ss.y).abs() < 0.5, "numa fila com a cor: {ss:?}");
    assert!(peso.x > ss.x + ss.w, "o Weight depois da cor");
    assert!(
        (fs.h - m::FIELD_H).abs() < 0.5,
        "a linha tem a altura do campo"
    );
}

/// Dois dígitos ainda cabem numa fila (só o `100` do dono parte, 02/11).
/// *Mutação: voltar os botões a `row_h` ⇒ a linha parte.*
#[test]
fn a_two_digit_weight_still_fits_one_row() {
    let estreita = Rect {
        x: 0.0,
        y: 0.0,
        w: 1340.0,
        h: 1600.0,
    };
    let mut mesa = Mesa::na_coluna(true, FillKind::Linear, estreita);
    let mut a = state::current_appearance().expect("a forma");
    a.layers[1].width = 12.0;
    state::set_current_appearance(Some(a));
    mesa.assenta();
    let peso = mesa.rect(p::vector_paint_row_width_id(1)).unwrap();
    let cor = mesa.rect(p::vector_paint_swatch_id(1)).unwrap();
    assert!((peso.y - cor.y).abs() < 0.5, "{peso:?} × {cor:?}");
}

/// O nome vai com o id: cada opção no lugar da tabela, da esquerda para a direita.
/// *Mutação: trocar dois ids num `Seg` ⇒ vermelho.*
#[test]
fn each_option_sits_in_the_order_of_its_table() {
    let mut mesa = Mesa::nova(true, FillKind::Linear);
    mesa.abre(p::VECTOR_FILL_ROW);
    let cresce = |mesa: &Mesa, ids: &[NodeId]| {
        let xs: Vec<f32> = ids
            .iter()
            .map(|i| mesa.rect(*i).expect("pintado").x)
            .collect();
        assert!(xs.windows(2).all(|w| w[0] < w[1]), "fora de ordem: {xs:?}");
    };
    cresce(
        &mesa,
        &[
            t::VECTOR_FILL_KIND_SOLID,
            t::VECTOR_FILL_KIND_LINEAR,
            t::VECTOR_FILL_KIND_RADIAL,
            t::VECTOR_FILL_KIND_MULTI,
            t::VECTOR_FILL_KIND_PATTERN,
        ],
    );
    let mut mesa = Mesa::nova(true, FillKind::Linear);
    mesa.abre(p::VECTOR_STROKE_SETTINGS);
    for trio in [
        [
            t::VECTOR_ALIGN_INNER,
            t::VECTOR_ALIGN_CENTRE,
            t::VECTOR_ALIGN_OUTER,
        ],
        [
            t::VECTOR_CAP_BUTT,
            t::VECTOR_CAP_ROUND,
            t::VECTOR_CAP_SQUARE,
        ],
        [
            t::VECTOR_JOIN_MITER,
            t::VECTOR_JOIN_ROUND,
            t::VECTOR_JOIN_BEVEL,
        ],
        [
            t::VECTOR_STROKE_KIND_SOLID,
            t::VECTOR_STROKE_KIND_PATTERN,
            t::VECTOR_STROKE_KIND_BRUSH,
        ],
    ] {
        cresce(&mesa, &trio);
    }
}

/// Dono 02/11: só quando o valor não cabe a linha passa a duas filas, e o número nunca se corta.
#[test]
fn a_weight_that_does_not_fit_breaks_the_line_and_is_never_cut() {
    let estreita = Rect {
        x: 0.0,
        y: 0.0,
        w: 1340.0,
        h: 1600.0,
    };
    let mut mesa = Mesa::na_coluna(true, FillKind::Linear, estreita);
    let mut a = state::current_appearance().expect("a forma");
    a.layers[1].width = 1234.0;
    state::set_current_appearance(Some(a));
    mesa.assenta();
    let peso = mesa.rect(p::vector_paint_row_width_id(1)).unwrap();
    let cor = mesa.rect(p::vector_paint_swatch_id(1)).unwrap();
    assert!(
        peso.y > cor.y + cor.h - 0.5,
        "duas filas: {peso:?} × {cor:?}"
    );
    assert!(peso.w >= m::ICON_CELL + m::FIELD_PAD_R + 20.0, "{peso:?}");
}

/// O arrasto pelo ⚙ (F45) segue vivo no cartão: a linha voa, e o largar pede o Move.
/// *Mutação: não publicar os ⚙ no passe diferido ⇒ vermelho.*
#[test]
fn dragging_the_gear_reorders_the_stack_in_the_card() {
    let mut mesa = Mesa::nova(true, FillKind::Linear);
    let gear = mesa.rect(p::VECTOR_FILL_ROW).expect("o ⚙ do Fill de base");
    let topo = mesa.rect(p::vector_paint_row_id(1)).expect("o ⚙ de cima").y;
    let (x, y) = (gear.x + gear.w * 0.5, gear.y + gear.h * 0.5);
    let ev = mesa.ponteiro(PointerKind::Down, x, y);
    mesa.aplica(ev);
    let ev = mesa.ponteiro(PointerKind::Move, x, topo + 1.0);
    mesa.aplica(ev);
    mesa.assenta();
    assert!(mesa.rect(p::VECTOR_FILL_ROW).is_none(), "a agarrada voa");
    let _ = mesa.host.drained_actions();
    let ev = mesa.ponteiro(PointerKind::Up, x, topo + 1.0);
    for e in ev {
        mesa.host.apply_panel_event::<VectorPanel>(&mut mesa.st, e);
    }
    let lei = state::take_paint_move().expect("o largar pediu o Move");
    assert_eq!(lei.0, StackItem::BaseFill);
}

/// O «+» mora no título e abre a lista de sempre; a escolha chega como antes.
#[test]
fn the_plus_opens_the_menu_and_its_rows_answer() {
    for (linha, ouvido) in [
        (p::VECTOR_PAINT_ADD_FILL, "ADD_FILL"),
        (p::VECTOR_PAINT_ADD_STROKE, "ADD_STROKE"),
    ] {
        let grava = |cartoes| {
            let mut mesa = Mesa::nova(cartoes, FillKind::Linear);
            mesa.abre(p::VECTOR_PAINT_ADD_MENU);
            if cartoes {
                // A lista cresce para dentro do cartão: o «+» está no canto dele.
                let lista = mesa
                    .host
                    .store()
                    .panel_rect(core_ids::VECTOR_PANEL)
                    .unwrap();
                let opcao = mesa.rect(linha).expect("a linha da lista");
                assert!(
                    opcao.x + opcao.w <= lista.x + lista.w,
                    "{opcao:?} × {lista:?}"
                );
            }
            let accoes = mesa.clica(linha);
            mesa.assenta();
            assert_eq!(
                mesa.host.dropdown_is_open(p::VECTOR_PAINT_ADD_MENU),
                Some(false),
                "{ouvido}: o menu não fechou"
            );
            accoes
        };
        let novo = grava(true);
        assert!(!novo.is_empty(), "{ouvido}: nada chegou");
        assert_eq!(novo, grava(false), "{ouvido}");
    }
}

/// Os menus de Appearance e do painel: abrem, a lista cobre o corpo, e a linha escolhida chega como
/// antes. *Mutação: não pôr o pendente em `aberto` ⇒ as opções não se pintam.*
#[test]
fn the_blend_and_marker_menus_open_and_their_rows_answer() {
    for (abrir, botao, opcao) in [
        (None, p::VECTOR_OBJ_BLEND, p::vector_obj_blend_option_id(2)),
        (
            Some(p::vector_paint_row_id(1)),
            p::VECTOR_PAINT_BLEND,
            p::vector_paint_blend_option_id(2),
        ),
        (
            Some(p::VECTOR_STROKE_SETTINGS),
            t::VECTOR_MARKER_END_DD,
            t::vector_marker_option_id(1, 2),
        ),
        (
            Some(p::VECTOR_STROKE_SETTINGS),
            p::VECTOR_TOKEN_STROKE,
            p::vector_token_option_id(p::token_slot_of(p::VECTOR_TOKEN_STROKE).unwrap().code, 1),
        ),
    ] {
        let grava = |cartoes| {
            let mut mesa = Mesa::nova(cartoes, FillKind::Linear);
            if let Some(a) = abrir {
                mesa.abre(a);
            }
            mesa.abre(botao);
            let chip = mesa.rect(botao).expect("a caixa");
            let r = mesa.rect(opcao).expect("a linha da lista");
            assert!(r.w >= 1.0 && chip.w >= 1.0);
            mesa.clica(opcao)
        };
        let novo = grava(true);
        assert!(!novo.is_empty(), "{opcao:?}: nada chegou");
        assert_eq!(novo, grava(false), "{opcao:?}");
    }
}

/// ⭐ **O painel à esquerda é um cartão**: fica à esquerda da lista, com a largura dela, à mesma
/// distância do painel de antes; o X fecha-o; um clique no fundo não vaza; o tipo é o do cartão.
/// *Mutação: passar `theme` em vez da tinta do cartão, ou o rect do slot ⇒ vermelho.*
#[test]
fn the_left_panel_is_a_card_left_of_the_list_and_its_x_closes_it() {
    for (abrir, x, dentro) in [
        (
            p::VECTOR_FILL_ROW,
            p::VECTOR_FILL_PROPS_CLOSE,
            t::VECTOR_FILL_KIND_LINEAR,
        ),
        (
            p::VECTOR_STROKE_SETTINGS,
            p::VECTOR_STROKE_PROPS_CLOSE,
            t::VECTOR_CAP_ROUND,
        ),
        (
            p::vector_paint_row_id(1),
            p::VECTOR_PAINT_PROPS_CLOSE,
            p::VECTOR_PAINT_DX,
        ),
    ] {
        let mut mesa = Mesa::nova(true, FillKind::Linear);
        mesa.abre(abrir);
        let lista = mesa
            .host
            .store()
            .panel_rect(core_ids::VECTOR_PANEL)
            .expect("a lista");
        let fundo = mesa.rect(core_ids::RAIL_BACKDROP).expect("o fundo");
        assert!(fundo.x + fundo.w <= lista.x, "{fundo:?} × {lista:?}");
        assert!((fundo.w - lista.w).abs() < 0.5, "a largura da lista");
        let fecho = mesa.rect(x).expect("o X");
        assert!(
            (fecho.w - m::HEAD_ICON_BOX).abs() < 0.5,
            "o X tem a caixa do cabeçalho do cartão: {fecho:?}"
        );
        let campo = mesa.rect(dentro).expect("o corpo");
        assert!(
            campo.x >= fundo.x + m::CARD_PAD_X - 0.5,
            "{campo:?} × {fundo:?}"
        );
        assert!(campo.h >= m::FIELD_H - m::SEG_PAD * 2.0 - 0.5, "{campo:?}");
        mesa.clica(x);
        mesa.assenta();
        assert_eq!(mesa.host.dropdown_is_open(abrir), Some(false), "{x:?}");
        assert!(mesa.rect(dentro).is_none(), "{dentro:?} ficou pintado");
    }
}

/// Um ecrã baixo: o painel encolhe para caber, rola pela roda, e o que estava em baixo vem à vista.
#[test]
fn a_tall_left_panel_scrolls_inside_a_short_screen() {
    let baixa = Rect {
        x: 0.0,
        y: 0.0,
        w: 1340.0,
        h: 560.0,
    };
    let mut mesa = Mesa::na_coluna(true, FillKind::Linear, baixa);
    mesa.abre(p::VECTOR_STROKE_SETTINGS);
    let fundo = mesa.rect(core_ids::RAIL_BACKDROP).expect("o fundo");
    assert!(fundo.h <= baixa.h, "{fundo:?}");
    let topo = mesa.rect(t::VECTOR_CAP_ROUND).expect("Cap").y;
    let (cx, cy) = (fundo.x + fundo.w * 0.5, fundo.y + fundo.h * 0.5);
    mesa.host.wheel_at(cx, cy, -120.0);
    mesa.assenta();
    let depois = mesa.rect(t::VECTOR_CAP_ROUND).map_or(f32::MIN, |r| r.y);
    assert!(depois < topo, "a roda não rolou: {topo} → {depois}");
}

/// Todo campo e opção da fatia tem a sua marca de animação, com o canal de sempre.
/// *Mutação: tirar `.key(CAP)` ⇒ vermelho.*
#[test]
fn every_animated_control_keeps_its_mark() {
    let abre_e_confere = |abrir: Option<NodeId>, canais: &[NodeId]| {
        let mut mesa = Mesa::nova(true, FillKind::Linear);
        ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
        let mapa: BTreeMap<NodeId, KeyMark> = canais.iter().map(|c| (*c, KeyMark::Chave)).collect();
        set_key_marks(mapa);
        if let Some(a) = abrir {
            mesa.abre(a);
        } else {
            mesa.assenta();
        }
        for (i, c) in canais.iter().enumerate() {
            assert!(
                mesa.rect(key_mark_id(*c)).is_some(),
                "sem marca: {abrir:?} [{i}]"
            );
        }
        set_key_marks(BTreeMap::new());
    };
    abre_e_confere(
        None,
        &[
            p::VECTOR_OBJ_OPACITY_NUM,
            p::VECTOR_OBJ_BLEND,
            t::VECTOR_FILL_SWATCH,
            t::VECTOR_STROKE_SWATCH,
            t::VECTOR_WIDTH,
            p::VECTOR_FILL_EYE,
            p::VECTOR_STROKE_EYE,
            p::vector_paint_swatch_id(1),
            p::vector_paint_row_opacity_id(1),
            p::vector_paint_row_width_id(1),
            p::vector_paint_eye_id(1),
        ],
    );
    abre_e_confere(
        Some(p::VECTOR_FILL_ROW),
        &[
            p::VECTOR_FILL_RULE_NONZERO,
            t::VECTOR_FILL_KIND_SOLID,
            p::VECTOR_GRAD_ANGLE,
        ],
    );
    abre_e_confere(
        Some(p::VECTOR_STROKE_SETTINGS),
        &[
            t::VECTOR_ALIGN_INNER,
            t::VECTOR_STROKE_KIND_SOLID,
            t::VECTOR_CAP_BUTT,
            t::VECTOR_JOIN_MITER,
            p::VECTOR_STROKE_ABOVE_EFFECTS,
            t::VECTOR_DASH_IDS[0],
            t::VECTOR_MARKER_START_DD,
            t::VECTOR_MARKER_END_DD,
            t::VECTOR_MARKER_SCALE,
            t::VECTOR_MARKER_ROUND,
            p::VECTOR_EXPAND_W_START,
        ],
    );
    abre_e_confere(
        Some(p::vector_paint_row_id(1)),
        &[
            p::VECTOR_PAINT_ALIGN_INNER,
            p::VECTOR_PAINT_DX,
            p::VECTOR_PAINT_DILATE,
            p::VECTOR_PAINT_JOIN_MITER,
            p::VECTOR_PAINT_BLEND,
        ],
    );
}

/// Todo ícone mudo de uma linha e dos painéis tem dica. *Mutação: tirar uma linha de `prepare` ⇒ vermelho.*
#[test]
fn every_mute_control_of_the_slice_has_a_tooltip() {
    let mut mesa = Mesa::nova(true, FillKind::Linear);
    mesa.abre(p::vector_paint_row_id(1));
    let store = mesa.host.store();
    let mudos: Vec<NodeId> = [
        p::VECTOR_FILL_ROW,
        p::VECTOR_STROKE_SETTINGS,
        t::VECTOR_FILL_SWATCH,
        t::VECTOR_STROKE_SWATCH,
        p::VECTOR_FILL_EYE,
        p::VECTOR_FILL_PRESENT,
        p::VECTOR_STROKE_EYE,
        p::VECTOR_STROKE_PRESENT,
        p::VECTOR_FILL_PROPS_CLOSE,
        p::VECTOR_STROKE_PROPS_CLOSE,
        p::VECTOR_PAINT_PROPS_CLOSE,
        p::vector_paint_row_id(1),
        p::vector_paint_swatch_id(1),
        p::vector_paint_eye_id(1),
        p::vector_paint_del_id(1),
        p::vector_paint_up_id(1),
        p::vector_paint_down_id(1),
        t::VECTOR_CAP_BUTT,
        t::VECTOR_JOIN_ROUND,
        t::VECTOR_ALIGN_OUTER,
        p::VECTOR_PAINT_ALIGN_INNER,
        p::VECTOR_PAINT_JOIN_BEVEL,
    ]
    .into_iter()
    .filter(|id| store.tooltip_for(*id).is_none())
    .collect();
    assert!(mudos.is_empty(), "sem dica: {mudos:?}");
}

/// `PH2D_CARDS=0`: a barra de antes pinta-se, e o painel à esquerda tem a caixa de antes.
#[test]
fn cards_off_paints_the_old_faces() {
    let mut mesa = Mesa::nova(false, FillKind::Linear);
    assert!(
        mesa.rect(p::VECTOR_OBJ_OPACITY).is_some(),
        "a barra de antes"
    );
    mesa.abre(p::VECTOR_FILL_ROW);
    assert!(
        mesa.rect(p::VECTOR_GRAD_ANGLE).is_some(),
        "o ângulo de antes"
    );
    let fecho = mesa.rect(p::VECTOR_FILL_PROPS_CLOSE).expect("o X de antes");
    assert!((fecho.w - m::HEAD_ICON_BOX).abs() > 0.5, "{fecho:?}");
}

/// ⭐⭐ A lei da compactação (05 §4) nas linhas e nos painéis à esquerda, respeitando F44: os tokens do
/// traço e a Angle com o token dividem a linha. *Mutação: separar um par ⇒ vermelho.*
#[test]
fn the_slice_passes_the_compaction_census() {
    for (tipo, abrir, onde) in [
        (FillKind::Linear, p::VECTOR_FILL_ROW, "Fill linear"),
        (FillKind::MultiPoint, p::VECTOR_FILL_ROW, "Fill multi"),
        (
            FillKind::Solid,
            p::VECTOR_STROKE_SETTINGS,
            "Stroke settings",
        ),
        (FillKind::Solid, p::vector_paint_row_id(1), "camada"),
    ] {
        let mut mesa = Mesa::nova(true, tipo);
        mesa.abre(abrir);
        super::g5_os_cartoes::sem_acusacao(onde, || mesa.assenta());
    }
}

#[test]
fn siblings_share_the_line_in_the_left_panels() {
    let r = |mesa: &Mesa, id: NodeId| mesa.rect(id).unwrap_or_else(|| panic!("{id:?}"));
    let mut mesa = Mesa::nova(true, FillKind::Linear);
    mesa.abre(p::VECTOR_FILL_ROW);
    let (a, b) = (
        r(&mesa, t::VECTOR_GRAD_ANGLE_NUM),
        r(&mesa, p::VECTOR_TOKEN_FILL),
    );
    assert!((a.y - b.y).abs() < 1.0, "Angle e token: {a:?} {b:?}");
    let mut mesa = Mesa::nova(true, FillKind::Solid);
    mesa.abre(p::VECTOR_STROKE_SETTINGS);
    let (a, b) = (
        r(&mesa, p::VECTOR_TOKEN_WIDTH),
        r(&mesa, p::VECTOR_TOKEN_STROKE),
    );
    assert!((a.y - b.y).abs() < 1.0, "os dois tokens: {a:?} {b:?}");
    let mut mesa = Mesa::nova(true, FillKind::Solid);
    mesa.abre(p::vector_paint_row_id(1));
    let (a, b) = (
        r(&mesa, p::VECTOR_PAINT_DILATE),
        r(&mesa, p::VECTOR_PAINT_JOIN_MITER),
    );
    // O segmento senta-se dentro da pílula: mesma linha é sobrepor em y, não o mesmo y.
    assert!(
        a.y < b.y + b.h && b.y < a.y + a.h,
        "Grow e a quina: {a:?} {b:?}"
    );
}
