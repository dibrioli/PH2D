//! ⭐⭐ **O painel Vector em cartões, fatia G3** (`docs/CartoesUI/12_VECTOR_PLANO.md`): Effects, Filters,
//! a rampa, o painel à esquerda deles e o painel do traço da barra ②. F43 é o layout aprovado: muda a
//! cara. Cada id é provado pelo gesto REAL, nos rects pintados, e o que chega ao barramento é o MESMO do
//! painel de antes.

use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, dispatch_key};
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::widget::key_mark::{KeyMark, key_mark_id, set_key_marks};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{self, VectorPanelState};
use ph2d_panel_vector::{
    FalloffRole, FilterKindView, FilterRowView, FxParamView, FxRowView, VectorPanel, cards,
    ids as p,
};
use ph2d_tool_vector::ids as t;
use ph2d_ui_testkit::MockPanelHost;

const COLUNA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1340.0,
    h: 3200.0,
};

const SOMBRA: usize = 0;
const RUIDO: usize = 1;
const AJUSTE: usize = 2;
const DUOTONE: usize = 3;
const RAMPA: usize = 4;
const CRESCE: usize = 5;

fn tipos() -> Vec<FilterKindView> {
    let vazio = FilterKindView {
        name: "Blur",
        radius_label: None,
        offset_labels: None,
        color_label: None,
        color_b_label: None,
        modes: &[],
        takes_blend: false,
        takes_ramp: false,
        noise_labels: None,
        grow_label: None,
        adjust_labels: None,
    };
    vec![
        FilterKindView {
            name: "Drop Shadow",
            radius_label: Some("Radius"),
            offset_labels: Some(("Offset X", "Offset Y")),
            color_label: Some("Color"),
            takes_blend: true,
            ..vazio
        },
        FilterKindView {
            name: "Noise",
            noise_labels: Some(("Size", "Detail", "Seed")),
            modes: &["Displace", "Grain"],
            ..vazio
        },
        FilterKindView {
            name: "Color Adjust",
            adjust_labels: Some(("Hue", "Saturation", "Brightness")),
            ..vazio
        },
        FilterKindView {
            name: "Duotone",
            color_label: Some("Dark"),
            color_b_label: Some("Light"),
            ..vazio
        },
        FilterKindView {
            name: "Gradient Map",
            takes_ramp: true,
            ..vazio
        },
        FilterKindView {
            name: "Grow",
            grow_label: Some("Amount"),
            ..vazio
        },
    ]
}

fn degrau(kind: u8) -> FilterRowView {
    let mut stop_pos = [0.0; 8];
    stop_pos[1] = 0.5;
    stop_pos[2] = 1.0;
    let mut stop_colors = [[0; 4]; 8];
    stop_colors[1] = [10, 200, 30, 255];
    FilterRowView {
        kind,
        mode: 0,
        enabled: true,
        radius: 0.12,
        offx: 0.25,
        offy: -0.25,
        color: [200, 10, 10, 255],
        color_b: [10, 10, 200, 255],
        opacity: 0.5,
        blend: 0,
        scale: 0.25,
        detail: 3,
        seed: 4,
        grow: 0.1,
        hue: 20.0,
        sat: 0.25,
        bright: -0.25,
        stop_pos,
        stop_colors,
        stop_count: 3,
        ramp_preview: [[128; 3]; ph2d_panel_vector::RAMP_PREVIEW_N],
    }
}

fn efeito(label: &'static str) -> FxRowView {
    let num = |name, max, integer, value| FxParamView {
        name,
        min: 0.0,
        max,
        toggle: false,
        integer,
        value,
    };
    FxRowView {
        label,
        enabled: true,
        params: vec![
            num("Size", 100.0, false, 12.0),
            FxParamView {
                toggle: true,
                ..num("Smooth", 1.0, false, 0.0)
            },
            num("Ridges", 128.0, true, 8.0),
            num("Bend", 1.0, false, 0.25),
        ],
        falloff_role: FalloffRole::NotFalloff,
    }
}

/// Uma forma com tudo à vista: dois efeitos e um degrau de cada espécie.
fn forma() {
    state::set_length_suffix("px");
    ph2d_panel_vector::set_current_effects(
        true,
        false,
        ph2d_vec_scene::effect::PathEffect::KINDS,
        vec![efeito("Zig Zag"), efeito("Wave")],
    );
    ph2d_panel_vector::set_current_filter_can_add(true);
    ph2d_panel_vector::set_filter_kinds(tipos());
    ph2d_panel_vector::set_filter_blend_names(vec!["Normal", "Multiply", "Screen"]);
    ph2d_panel_vector::set_current_filters(
        [SOMBRA, RUIDO, AJUSTE, DUOTONE, RAMPA, CRESCE]
            .map(|k| degrau(k as u8))
            .to_vec(),
    );
    set_key_marks(BTreeMap::new());
}

struct Mesa {
    host: MockPanelHost,
    st: VectorPanelState,
    rects: Vec<(NodeId, Rect)>,
}

impl Mesa {
    fn nova(cartoes: bool) -> Self {
        cards::force_cards_on_this_thread(Some(cartoes));
        forma();
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

    /// A pilha com lugar: cheia (as seis espécies), o «+» dos filtros apaga-se.
    fn com_lugar_nos_filtros(mut self) -> Self {
        ph2d_panel_vector::set_current_filters(vec![degrau(0)]);
        self.assenta();
        self
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
        self.aplica(ev)
    }

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

/// `(chip, barra)` dos números que o tipo da linha `row` oferece, incluída a opacidade.
fn numeros_do_degrau(row: usize) -> Vec<(NodeId, NodeId)> {
    let mut v = match row {
        SOMBRA => vec![
            (p::filter_radius_num_id(row), p::filter_radius_id(row)),
            (p::filter_offx_num_id(row), p::filter_offx_id(row)),
            (p::filter_offy_num_id(row), p::filter_offy_id(row)),
        ],
        RUIDO => vec![
            (p::filter_scale_num_id(row), p::filter_scale_id(row)),
            (p::filter_detail_num_id(row), p::filter_detail_id(row)),
            (p::filter_seed_num_id(row), p::filter_seed_id(row)),
        ],
        AJUSTE => vec![
            (p::filter_hue_num_id(row), p::filter_hue_id(row)),
            (p::filter_sat_num_id(row), p::filter_sat_id(row)),
            (p::filter_bright_num_id(row), p::filter_bright_id(row)),
        ],
        CRESCE => vec![(p::filter_grow_num_id(row), p::filter_grow_id(row))],
        _ => vec![],
    };
    v.push((p::filter_opacity_num_id(row), p::filter_opacity_id(row)));
    v
}

fn cliques_do_degrau(row: usize) -> Vec<NodeId> {
    let mut v = vec![
        p::FILTER_PROPS_CLOSE,
        p::filter_hide_id(row),
        p::filter_remove_id(row),
    ];
    if row > 0 {
        v.push(p::filter_up_id(row));
    }
    if row < CRESCE {
        v.push(p::filter_down_id(row));
    }
    match row {
        RUIDO => v.extend([p::filter_mode_id(row, 0), p::filter_mode_id(row, 1)]),
        RAMPA => v.extend([p::filter_stop_add_id(row), p::filter_stop_remove_id(row)]),
        _ => {}
    }
    v
}

fn numeros_do_efeito(row: usize) -> Vec<(NodeId, NodeId)> {
    [0, 2, 3]
        .map(|k| {
            (
                t::vector_fx_param_num_id(row, k),
                t::vector_fx_param_id(row, k),
            )
        })
        .to_vec()
}

/// O inventário: tudo registado, e tudo pintado nos dois painéis.
#[test]
fn every_id_of_the_slice_is_painted_in_cards_and_in_the_old_panel() {
    for cartoes in [true, false] {
        let mut mesa = Mesa::nova(cartoes);
        let mut lista = vec![t::VECTOR_FX_ADD_MENU, t::VECTOR_FX_APPLY];
        for r in 0..2 {
            lista.extend([
                t::vector_fx_row_id(r),
                t::vector_fx_hide_id(r),
                t::vector_fx_remove_id(r),
            ]);
        }
        for r in 0..6 {
            lista.extend([
                p::filter_row_id(r),
                p::filter_hide_id(r),
                p::filter_remove_id(r),
            ]);
        }
        for id in lista {
            assert!(mesa.rect(id).is_some(), "{id:?} (cartões {cartoes})");
        }
        mesa.abre(t::vector_fx_row_id(1));
        let mut d = vec![
            t::VECTOR_FX_PROPS_CLOSE,
            t::vector_fx_up_id(1),
            t::vector_fx_toggle_id(1, 1),
        ];
        d.extend(numeros_do_efeito(1).iter().map(|n| n.0));
        for id in d {
            assert!(mesa.rect(id).is_some(), "{id:?} (cartões {cartoes})");
        }
        for r in 0..6 {
            let mut mesa = Mesa::nova(cartoes);
            mesa.abre(p::filter_row_id(r));
            let mut ids: Vec<NodeId> = numeros_do_degrau(r).iter().map(|n| n.0).collect();
            ids.extend(cliques_do_degrau(r));
            match r {
                SOMBRA => ids.extend([p::filter_color_id(r), p::filter_blend_id(r)]),
                DUOTONE => ids.extend([p::filter_color_id(r), p::filter_color_b_id(r)]),
                RAMPA => ids.extend([
                    p::filter_stop_color_id(r),
                    p::filter_stop_id(r, 0),
                    p::filter_stop_id(r, 2),
                ]),
                _ => {}
            }
            for id in ids {
                assert!(mesa.rect(id).is_some(), "degrau {r}: {id:?} ({cartoes})");
            }
        }
    }
}

/// Nos cartões as barras não se pintam: o campo é a cara delas.
/// *Mutação: pôr um `slider_row` num corpo de G3 ⇒ vermelho.*
#[test]
fn no_old_bar_is_painted_in_the_slice() {
    for r in 0..6 {
        let mut mesa = Mesa::nova(true);
        mesa.abre(p::filter_row_id(r));
        for (_, barra) in numeros_do_degrau(r) {
            assert!(mesa.rect(barra).is_none(), "degrau {r}: barra {barra:?}");
        }
    }
    let mut mesa = Mesa::nova(true);
    mesa.abre(t::vector_fx_row_id(0));
    for (_, barra) in numeros_do_efeito(0) {
        assert!(mesa.rect(barra).is_none(), "efeito: barra {barra:?}");
    }
}

/// ⭐⭐ **Mesmo id, mesmo evento**: o clique real produz no barramento o que produz no painel de antes.
/// *Mutação: trocar `hide` e `remove` na linha ⇒ vermelho.*
#[test]
fn every_click_sends_the_same_action_as_the_old_panel() {
    let grava = |cartoes: bool, abrir: Option<NodeId>, ids: &[NodeId]| -> Vec<Vec<String>> {
        ids.iter()
            .map(|id| {
                let mut mesa = Mesa::nova(cartoes);
                if abrir == Some(p::FILTER_ADD_MENU) {
                    mesa = mesa.com_lugar_nos_filtros();
                }
                if let Some(a) = abrir {
                    mesa.abre(a);
                }
                mesa.clica(*id)
            })
            .collect()
    };
    let mut casos: Vec<(Option<NodeId>, Vec<NodeId>)> = vec![
        (
            None,
            vec![
                t::vector_fx_hide_id(1),
                t::vector_fx_remove_id(1),
                t::VECTOR_FX_APPLY,
                p::filter_hide_id(2),
                p::filter_remove_id(2),
            ],
        ),
        (
            Some(t::vector_fx_row_id(1)),
            vec![
                t::VECTOR_FX_PROPS_CLOSE,
                t::vector_fx_up_id(1),
                t::vector_fx_toggle_id(1, 1),
            ],
        ),
        (
            Some(t::VECTOR_FX_ADD_MENU),
            vec![t::vector_fx_add_id(0), t::vector_fx_add_id(3)],
        ),
        (
            Some(p::FILTER_ADD_MENU),
            vec![p::filter_add_id(0), p::filter_add_id(2)],
        ),
    ];
    casos.extend((0..6).map(|r| (Some(p::filter_row_id(r)), cliques_do_degrau(r))));
    for (abrir, ids) in casos {
        let (novo, velho) = (grava(true, abrir, &ids), grava(false, abrir, &ids));
        assert_eq!(novo, velho, "o barramento mudou ao abrir {abrir:?}");
    }
    // O painel aberto com a lista das leis por cima dele.
    let opcao = |cartoes: bool| {
        let mut mesa = Mesa::nova(cartoes);
        mesa.abre(p::filter_row_id(SOMBRA));
        mesa.abre(p::filter_blend_id(SOMBRA));
        mesa.clica(p::filter_blend_option_id(SOMBRA, 1))
    };
    let accoes = opcao(true);
    assert_eq!(accoes, opcao(false));
    assert!(!accoes.is_empty(), "a lei não chegou");
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

/// ⭐ **Scrub de cada número**: o campo muda, a ferramenta ouve pela barra de sempre, a semente a meio
/// não o puxa, e o Esc devolve. *Mutação: tirar o `mark_card_field` ou a semente ⇒ vermelho.*
#[test]
fn scrubbing_each_number_reaches_the_tool_and_escape_gives_it_back() {
    let mut casos: Vec<(NodeId, Vec<(NodeId, NodeId)>)> = (0..6)
        .map(|r| (p::filter_row_id(r), numeros_do_degrau(r)))
        .collect();
    casos.push((t::vector_fx_row_id(0), numeros_do_efeito(0)));
    for (abrir, numeros) in casos {
        for (chip, barra) in numeros {
            let mut mesa = Mesa::nova(true);
            mesa.abre(abrir);
            let antes = mesa.host.store().number_value(chip).expect("semeado");
            let ev = mesa.arrasta_a_pega(chip, 120.0);
            assert!(
                ev.contains(&WidgetEvent::ValueChanged(barra)),
                "{chip:?} não falou pela barra: {ev:?}"
            );
            let accoes = mesa.aplica(ev);
            assert!(!valores(&accoes, barra).is_empty(), "{chip:?}: {accoes:?}");
            let durante = mesa.host.store().number_value(chip).expect("campo");
            assert!((durante - antes).abs() > 1e-6, "{chip:?} parado em {antes}");
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
    }
}

/// Escrever o número manda à ferramenta o MESMO que no painel de antes.
#[test]
fn typing_a_number_sends_the_same_as_the_old_panel() {
    let casos = [
        (
            p::filter_row_id(SOMBRA),
            p::filter_radius_num_id(SOMBRA),
            "0.5",
        ),
        (
            p::filter_row_id(SOMBRA),
            p::filter_offx_num_id(SOMBRA),
            "-1",
        ),
        (p::filter_row_id(RUIDO), p::filter_detail_num_id(RUIDO), "2"),
        (p::filter_row_id(AJUSTE), p::filter_hue_num_id(AJUSTE), "45"),
        (
            p::filter_row_id(RAMPA),
            p::filter_opacity_num_id(RAMPA),
            "0.75",
        ),
        (
            t::vector_fx_row_id(0),
            t::vector_fx_param_num_id(0, 0),
            "40",
        ),
        (
            t::vector_fx_row_id(0),
            t::vector_fx_param_num_id(0, 2),
            "16",
        ),
    ];
    for (abrir, chip, texto) in casos {
        let grava = |cartoes: bool| {
            let mut mesa = Mesa::nova(cartoes);
            mesa.abre(abrir);
            let ev = mesa.host.type_into_number(chip, texto);
            mesa.aplica(ev)
        };
        let novo = grava(true);
        assert!(!novo.is_empty(), "{chip:?}: nada chegou");
        assert_eq!(novo, grava(false), "{chip:?}");
    }
}

/// O par de offset é uma grandeza: X e Y na mesma linha, mesmo com um número ímpar antes dele.
/// *Mutação: juntar `soltos` e `offset` no mesmo `chunks` ⇒ vermelho.*
#[test]
fn the_offset_pair_stays_on_one_row() {
    let mut mesa = Mesa::nova(true);
    mesa.abre(p::filter_row_id(SOMBRA));
    let x = mesa.rect(p::filter_offx_num_id(SOMBRA)).expect("X");
    let y = mesa.rect(p::filter_offy_num_id(SOMBRA)).expect("Y");
    assert!((x.y - y.y).abs() < 0.5, "{x:?} × {y:?}");
    assert!(x.x + x.w <= y.x, "{x:?} × {y:?}");
}

/// Cada número traz o passo e o valor de fábrica dele: a seta e o duplo clique na pega leem-nos.
/// *Mutação: tirar o `mark_card_field` de `prepare` ou de `prepara_numero` ⇒ vermelho.*
#[test]
fn each_field_declares_its_step_and_factory_value() {
    let mesa = Mesa::nova(true);
    let s = mesa.host.store();
    let spec = |id: NodeId| {
        s.card_field(id)
            .unwrap_or_else(|| panic!("{id:?} sem marca"))
    };
    assert_eq!(spec(t::vector_fx_param_num_id(0, 2)).step, 1.0, "contagem");
    assert!(spec(t::vector_fx_param_num_id(0, 3)).step < 1.0, "fração");
    assert_eq!(spec(p::filter_hue_num_id(AJUSTE)).default, Some(0.0));
    assert_eq!(spec(p::filter_opacity_num_id(RAMPA)).default, Some(1.0));
    assert_eq!(spec(p::filter_offx_num_id(SOMBRA)).default, Some(0.0));
    assert_eq!(spec(p::filter_radius_num_id(SOMBRA)).default, None);
}

/// F43: o olho à esquerda do «−», e o nome à esquerda dos dois.
/// *Mutação: trocar `hide` e `remove` na linha ⇒ vermelho.*
#[test]
fn the_row_keeps_the_order_of_f43() {
    let mesa = Mesa::nova(true);
    for (linha, olho, menos) in [
        (
            t::vector_fx_row_id(0),
            t::vector_fx_hide_id(0),
            t::vector_fx_remove_id(0),
        ),
        (
            p::filter_row_id(0),
            p::filter_hide_id(0),
            p::filter_remove_id(0),
        ),
    ] {
        let [l, o, m] = [linha, olho, menos].map(|i| mesa.rect(i).expect("pintado"));
        assert!(l.x + l.w <= o.x && o.x + o.w <= m.x, "{l:?} {o:?} {m:?}");
    }
}

/// O que o campo mostra é o do documento, não o do trilho. *Mutação: tirar a semente ⇒ vermelho.*
#[test]
fn the_fields_show_the_document_value() {
    let mesa = Mesa::nova(true);
    let s = mesa.host.store();
    for (id, v) in [
        (t::vector_fx_param_num_id(0, 0), 12.0),
        (t::vector_fx_param_num_id(1, 2), 8.0),
        (p::filter_radius_num_id(SOMBRA), 0.12),
        (p::filter_offy_num_id(SOMBRA), -0.25),
        (p::filter_detail_num_id(RUIDO), 3.0),
        (p::filter_seed_num_id(RUIDO), 4.0),
        (p::filter_hue_num_id(AJUSTE), 20.0),
        (p::filter_bright_num_id(AJUSTE), -0.25),
        (p::filter_grow_num_id(CRESCE), 0.1),
        (p::filter_opacity_num_id(RAMPA), 0.5),
    ] {
        assert_eq!(s.number_value(id), Some(v), "{id:?}");
    }
}

/// ⭐ F43 no cartão: a linha `[glifo nome] [👁]◇ [−]`, a caixa é que abre, e o painel nasce à esquerda da
/// lista, no topo da linha. *Mutação: tirar `anota_props` ⇒ não abre.*
#[test]
fn a_row_opens_its_card_to_the_left_of_the_list() {
    for (linha, campo, fecha) in [
        (
            t::vector_fx_row_id(1),
            t::vector_fx_param_num_id(1, 0),
            t::VECTOR_FX_PROPS_CLOSE,
        ),
        (
            p::filter_row_id(SOMBRA),
            p::filter_radius_num_id(SOMBRA),
            p::FILTER_PROPS_CLOSE,
        ),
    ] {
        let mut mesa = Mesa::nova(true);
        assert!(mesa.rect(campo).is_none(), "fechado, nenhum campo à mostra");
        let l = mesa.rect(linha).expect("a linha");
        mesa.abre(linha);
        let lista = mesa
            .host
            .store()
            .panel_rect(core_ids::VECTOR_PANEL)
            .expect("a lista");
        let r = mesa.rect(campo).expect("o campo");
        assert!(r.x + r.w <= lista.x, "{r:?} × {lista:?}");
        let fundo = mesa.rect(core_ids::RAIL_BACKDROP).expect("o fundo");
        assert!(
            (fundo.y - l.y).abs() < 0.5,
            "o topo na linha: {fundo:?} × {l:?}"
        );
        assert!(fundo.x + fundo.w <= lista.x, "a moldura sobre a lista");
        mesa.clica(fecha);
        mesa.assenta();
        assert!(mesa.rect(campo).is_none(), "o X não fechou");
    }
}

/// O «+» abre a lista dos tipos PARA DENTRO do cartão, e a escolha fecha-a. Um cartão dobrado continua a
/// abrir. *Mutação: voltar a ancorar no corpo ⇒ o dobrado não abre.*
#[test]
fn the_plus_opens_inward_and_a_folded_card_still_opens_it() {
    for (mais, opcao, seccao) in [
        (
            t::VECTOR_FX_ADD_MENU,
            t::vector_fx_add_id(0),
            t::VECTOR_SECTION_EFFECTS,
        ),
        (
            p::FILTER_ADD_MENU,
            p::filter_add_id(0),
            p::VECTOR_SECTION_FILTERS,
        ),
    ] {
        for dobrado in [false, true] {
            let mut mesa = Mesa::nova(true).com_lugar_nos_filtros();
            if dobrado {
                mesa.host.set_collapsed(seccao, true);
                mesa.assenta();
            }
            mesa.abre(mais);
            let (chip, l) = (
                mesa.rect(mais).expect("o +"),
                mesa.rect(opcao).expect("a linha do +"),
            );
            assert!(
                l.x + l.w <= chip.x + chip.w + 0.5,
                "a lista passa da borda do «+»: {l:?} × {chip:?}"
            );
            let lista = mesa
                .host
                .store()
                .panel_rect(core_ids::VECTOR_PANEL)
                .expect("a lista");
            assert!(l.x >= lista.x, "a lista sai do cartão: {l:?} × {lista:?}");
            mesa.clica(opcao);
            mesa.assenta();
            assert_eq!(mesa.host.dropdown_is_open(mais), Some(false));
        }
    }
}

/// O punho da rampa arrasta-se na barra do cartão, e o que chega é o índice e a posição de sempre.
/// *Mutação: tirar `register_ramp_handles` do ramo dos cartões ⇒ vermelho.*
#[test]
fn a_ramp_handle_drags_on_the_card_bar() {
    let mut mesa = Mesa::nova(true);
    mesa.abre(p::filter_row_id(RAMPA));
    let punho = mesa.rect(p::filter_stop_id(RAMPA, 1)).expect("o punho");
    let (x, y) = (punho.x + punho.w * 0.5, punho.y + punho.h * 0.5);
    let mut ev = mesa.ponteiro(PointerKind::Down, x, y);
    ev.extend(mesa.ponteiro(PointerKind::Move, x + 30.0, y));
    let accoes = mesa.aplica(ev);
    assert!(
        accoes.iter().any(|a| a.contains("\"4:1:")),
        "a rampa não ouviu: {accoes:?}"
    );
    assert_eq!(
        ph2d_panel_vector::selected_stop(RAMPA),
        1,
        "arrastar escolhe"
    );
}

/// O punho responde já no quadro em que o painel abre: o passe de sementes corre ANTES de o painel
/// publicar a barra, e nesse quadro só o registo de `fx_side` dá o punho ao store.
/// *Mutação: tirar `register_ramp_handles` de `fx_side` ⇒ vermelho.*
#[test]
fn a_ramp_handle_answers_in_the_very_frame_the_panel_opens() {
    let mut mesa = Mesa::nova(true);
    mesa.clica(p::filter_row_id(RAMPA));
    // Dois quadros: o 1.º pinta a linha aberta e anota-a; o 2.º abre o painel com a barra (`seed` ainda
    // sem ela).
    for _ in 0..2 {
        let (h, st) = (&mut mesa.host, &mut mesa.st);
        mesa.rects = h.paint::<VectorPanel>(st, COLUNA);
        if mesa
            .rects
            .iter()
            .any(|(n, _)| *n == p::filter_stop_id(RAMPA, 1))
        {
            break;
        }
    }
    let punho = mesa.rect(p::filter_stop_id(RAMPA, 1)).expect("o punho");
    let (x, y) = (punho.x + punho.w * 0.5, punho.y + punho.h * 0.5);
    let mut ev = mesa.ponteiro(PointerKind::Down, x, y);
    ev.extend(mesa.ponteiro(PointerKind::Move, x + 30.0, y));
    let accoes = mesa.aplica(ev);
    assert!(
        accoes.iter().any(|a| a.contains("\"4:1:")),
        "o punho não ouviu no 1.º quadro: {accoes:?}"
    );
}

/// A marca de animação de cada campo mora na barra, e a do olho na linha.
/// *Mutação: tirar `.key` da fila ⇒ vermelho; tirar o `paint_decorator_for` da linha ⇒ vermelho.*
#[test]
fn the_animation_marks_stay() {
    let mut mesa = Mesa::nova(true);
    set_key_marks(BTreeMap::from([
        (p::filter_radius_id(SOMBRA), KeyMark::Chave),
        (p::filter_opacity_id(SOMBRA), KeyMark::Chave),
        (p::filter_blend_id(SOMBRA), KeyMark::Chave),
        (p::filter_color_id(SOMBRA), KeyMark::Chave),
        (p::filter_hide_id(0), KeyMark::Chave),
        (t::vector_fx_hide_id(0), KeyMark::Chave),
    ]));
    mesa.abre(p::filter_row_id(SOMBRA));
    for id in [
        p::filter_radius_id(SOMBRA),
        p::filter_opacity_id(SOMBRA),
        p::filter_blend_id(SOMBRA),
        p::filter_color_id(SOMBRA),
        p::filter_hide_id(0),
        t::vector_fx_hide_id(0),
    ] {
        assert!(mesa.rect(key_mark_id(id)).is_some(), "sem marca: {id:?}");
    }
    set_key_marks(BTreeMap::new());
}

/// Forma presa a ossos: a frase, sem linhas nem Apply. E sem efeitos, nada de Apply.
#[test]
fn a_bound_shape_says_why_and_an_empty_stack_has_no_apply() {
    let mut mesa = Mesa::nova(true);
    ph2d_panel_vector::set_current_effects(
        true,
        true,
        ph2d_vec_scene::effect::PathEffect::KINDS,
        vec![efeito("Zig Zag")],
    );
    mesa.assenta();
    assert!(mesa.rect(t::vector_fx_row_id(0)).is_none(), "linha presa");
    assert!(mesa.rect(t::VECTOR_FX_APPLY).is_none(), "Apply preso");
    assert!(mesa.rect(t::VECTOR_FX_ADD_MENU).is_none(), "+ preso");
    ph2d_panel_vector::set_current_effects(
        true,
        false,
        ph2d_vec_scene::effect::PathEffect::KINDS,
        Vec::new(),
    );
    mesa.assenta();
    assert!(mesa.rect(t::VECTOR_FX_APPLY).is_none(), "Apply vazio");
    assert!(
        mesa.rect(t::VECTOR_FX_ADD_MENU).is_some(),
        "+ com a pilha vazia"
    );
}

/// Abre o painel do traço pelo store (a barra pinta-se no hero; aqui só a âncora).
fn abre_o_traco(mesa: &mut Mesa) {
    mesa.host.store_mut().set_tool_row_slots(vec![(
        p::VECTOR_BAR_STROKE,
        Rect::new(400.0, 40.0, 80.0, 24.0),
    )]);
    if let Some(InteractiveState::Dropdown { open, .. }) =
        mesa.host.store_mut().get_mut(p::VECTOR_BAR_STROKE)
    {
        *open = true;
    }
    mesa.assenta();
}

/// ⭐ **O painel do traço da barra ② em cartão**: a pega de um campo marcado faz scrub porque está
/// pintada, e o clique no valor foca. No painel de antes não há pega: o clique à esquerda foca.
/// *Mutação: o ramo dos cartões fora de `paint_stroke_popover` ⇒ a pega deixa de existir.*
#[test]
fn the_stroke_popover_of_the_bar_paints_real_handles() {
    let dash = t::VECTOR_DASH_IDS[0];
    let mut mesa = Mesa::nova(true);
    abre_o_traco(&mut mesa);
    let r = mesa.rect(dash).expect("o tracejado no painel do traço");
    assert_eq!(
        ph2d_editor_core::widget::card_field::painted_letter(dash).as_deref(),
        Some(ph2d_i18n::tr("card.vector.glyph.dash")),
        "a pega não se pintou"
    );
    let antes = mesa.host.store().number_value(dash).expect("semeado");
    let ev = mesa.arrasta_a_pega(dash, 40.0);
    assert!(ev.contains(&WidgetEvent::ValueChanged(dash)), "{ev:?}");
    mesa.esc();
    assert_eq!(mesa.host.store().number_value(dash), Some(antes));
    let ev = mesa
        .host
        .click_at(r.x + r.w - m::FIELD_PAD_R, r.y + r.h * 0.5);
    mesa.aplica(ev);
    assert_eq!(
        mesa.host.store().focus_id(),
        Some(dash),
        "o valor não focou"
    );

    let mut velho = Mesa::nova(false);
    abre_o_traco(&mut velho);
    assert!(velho.host.store().card_field(dash).is_none(), "marcado");
    let r = velho.rect(dash).expect("o tracejado de antes");
    let ev = velho.host.click_at(r.x + 4.0, r.y + r.h * 0.5);
    velho.aplica(ev);
    assert_eq!(
        velho.host.store().focus_id(),
        Some(dash),
        "antes: não focou"
    );
}

/// Com a aba do painel atrás o traço abre na mesma: alguém tem de semear os campos do cartão dele. O
/// mock força o painel visível; este caminho só existe no hero.
/// *Mutação: tirar o `prepare` do ramo do painel escondido ⇒ vermelho.*
#[test]
fn the_popover_works_with_the_panel_tab_behind() {
    use ph2d_editor_core::panel::{ErasedPanel, PanelRegistry};
    use ph2d_editor_core::screens::hero::{HeroScreen, paint_hero_screen};
    static INIT: std::sync::Once = std::sync::Once::new();
    ph2d_editor_core::test_support::ensure_panel_registry();
    INIT.call_once(|| {
        let mut reg = PanelRegistry::new_empty();
        reg.push(ErasedPanel::new::<VectorPanel>());
        let _ = ph2d_editor_core::panel::install_panel_registry(reg);
    });
    cards::force_cards_on_this_thread(Some(true));
    forma();
    state::set_current_appearance(Some(state::Appearance {
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        layers: vec![state::PaintRow {
            align: ph2d_vec_scene::StrokeAlign::Centre,
            ramp: None,
            is_fill: false,
            color: [0x33, 0x66, 0xCC, 0xFF],
            width: 2.0,
            enabled: true,
            opacity: 0.5,
            blend: ph2d_vec_scene::BlendMode::Normal,
            offset: [0.0, 0.0],
            dilate: 0.0,
            dilate_join: 0,
            mixed: Default::default(),
        }],
        mixed_opacity: false,
        mixed_blend: false,
        layers_differ: false,
        order: Vec::new(),
    }));
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(VectorPanel::ID, false);
    let janela = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    let mut pinta = |hero: &mut HeroScreen| {
        let mut scene = ph2d_vector::VectorScene::new();
        let mut text = ph2d_text::TextSystem::without_system_fonts();
        paint_hero_screen(hero, janela, &mut scene, &mut text);
    };
    pinta(&mut hero);
    hero.store.set_tool_row_slots(vec![(
        p::VECTOR_BAR_STROKE,
        Rect::new(400.0, 40.0, 80.0, 24.0),
    )]);
    if let Some(InteractiveState::Dropdown { open, .. }) = hero.store.get_mut(p::VECTOR_BAR_STROKE)
    {
        *open = true;
    }
    pinta(&mut hero);
    let dash = t::VECTOR_DASH_IDS[0];
    assert!(
        hero.hit_index.rect_for(dash).is_some(),
        "o painel não abriu"
    );
    assert!(hero.store.card_field(dash).is_some(), "o campo sem marca");
    assert_eq!(
        hero.store.number_value(p::vector_paint_row_opacity_id(0)),
        Some(50.0),
        "sem semente com a aba atrás"
    );
    state::set_current_appearance(None);
}

/// O chip numérico da fila ② partilha o id com um campo de cartão, mas não tem pega: o clique à
/// esquerda foca, como no resto dele. *Mutação: tirar o `tool_row_slot` do `dispatch_down` ⇒ vermelho.*
#[test]
fn a_number_chip_of_the_bar_has_no_handle() {
    let mut mesa = Mesa::nova(true);
    let id = t::VECTOR_WIDTH_NUM;
    assert!(
        mesa.host.store().card_field(id).is_some(),
        "o id é de cartão"
    );
    let chip = Rect::new(400.0, 10.0, 80.0, 24.0);
    mesa.host.store_mut().set_tool_row_slots(vec![(id, chip)]);
    mesa.host.hit_index_mut().register(id, chip);
    let ev = mesa.host.click_at(chip.x + 4.0, chip.y + chip.h * 0.5);
    mesa.aplica(ev);
    assert_eq!(mesa.host.store().focus_id(), Some(id), "a fila não focou");
}
