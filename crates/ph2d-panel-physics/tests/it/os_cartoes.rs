//! ⭐⭐ **O painel Physics em cartões: nenhum controlo morreu.** O inventário vem das tabelas que o
//! corpo de antes percorria; cada id é provado pelo gesto REAL (ponteiro → despacho → `apply_event`
//! → intent), nos rects que a pintura registou.

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::dicas;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_physics::state::{PhysicsPanelState, PhysicsSnapshot};
use ph2d_panel_physics::{
    PhysicsIntent, PhysicsPanel, cards, drain_intents, ids, interact, rows, set_current_physics,
};
use ph2d_physics_ecs::{
    HoldMode, InteractionSettings, InteractionTool, JointTool, LayerMatrix, PhysicsSettings,
};
use ph2d_ui_testkit::MockPanelHost;

const COLUNA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 340.0,
    h: 2600.0,
};

struct Mesa {
    host: MockPanelHost,
    st: PhysicsPanelState,
    rects: Vec<(NodeId, Rect)>,
}

impl Mesa {
    fn nova(snap: PhysicsSnapshot) -> Self {
        cards::force_cards_on_this_thread(Some(true));
        set_current_physics(Some(snap));
        let _ = drain_intents();
        let mut mesa = Self {
            host: MockPanelHost::with_panel::<PhysicsPanel>(),
            st: PhysicsPanelState,
            rects: Vec::new(),
        };
        // O Debug nasce dobrado; o inventário quer vê-lo aberto (a escolha do artista ganha à semente).
        mesa.host.set_collapsed(ids::PHYSICS_SEC_DEBUG, false);
        mesa.assenta();
        mesa
    }

    /// Os quadros que a mola das alturas pede para os cartões assentarem.
    fn assenta(&mut self) {
        for _ in 0..60 {
            let (h, st) = (&mut self.host, &mut self.st);
            self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || h.paint::<PhysicsPanel>(st, COLUNA),
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

    fn aplica(&mut self, eventos: Vec<WidgetEvent>) -> Vec<PhysicsIntent> {
        for e in eventos {
            self.host.apply_panel_event::<PhysicsPanel>(&mut self.st, e);
        }
        drain_intents()
    }

    fn clica(&mut self, id: NodeId) -> Vec<PhysicsIntent> {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("{id:?} não pintado"));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(self.host.hit_at(cx, cy), Some(id), "outro id no centro");
        let ev = self.host.click_at(cx, cy);
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

    /// O scrub real: premir na PEGA (a célula do ícone) e arrastar `dx`, sem largar.
    fn arrasta_a_pega(&mut self, chip: NodeId, dx: f32) -> Vec<WidgetEvent> {
        let r = self
            .rect(chip)
            .unwrap_or_else(|| panic!("{chip:?} não pintado"));
        let (x, y) = (r.x + m::ICON_CELL * 0.5, r.y + r.h * 0.5);
        let mut ev = self.ponteiro(PointerKind::Down, x, y);
        ev.extend(self.ponteiro(PointerKind::Move, x + dx * 0.5, y));
        ev.extend(self.ponteiro(PointerKind::Move, x + dx, y));
        ev
    }

    fn solta(&mut self, chip: NodeId, dx: f32) -> Vec<WidgetEvent> {
        let r = self.rect(chip).expect("pintado");
        self.ponteiro(
            PointerKind::Up,
            r.x + m::ICON_CELL * 0.5 + dx,
            r.y + r.h * 0.5,
        )
    }
}

fn mundo(settings: PhysicsSettings, interaction: InteractionSettings) -> PhysicsSnapshot {
    PhysicsSnapshot {
        settings,
        vivo: settings,
        interaction,
        ..PhysicsSnapshot::default()
    }
}

fn ultimo_mundo(intents: &[PhysicsIntent]) -> Option<PhysicsSettings> {
    intents.iter().rev().find_map(|i| match i {
        PhysicsIntent::SetSettings(s, _) => Some(*s),
        _ => None,
    })
}

fn estados() -> Vec<InteractionSettings> {
    let mut out = Vec::new();
    for tool in InteractionTool::ALL {
        for hold in HoldMode::ALL {
            for joint in JointTool::ALL {
                out.push(InteractionSettings {
                    tool,
                    hold,
                    joint,
                    ..InteractionSettings::default()
                });
            }
        }
    }
    out
}

/// O inventário de ANTES: tudo o que o `populate` regista, das mesmas tabelas.
fn inventario() -> Vec<NodeId> {
    let mut v: Vec<NodeId> = Vec::new();
    for r in rows::rows() {
        v.extend([r.slider, r.chip]);
    }
    for r in interact::IROWS {
        v.extend([r.slider, r.chip]);
    }
    v.extend(ids::PHYSICS_INTERACT_TOOL_OPT);
    v.extend(ids::PHYSICS_HOLD_MODE_OPT);
    v.extend(ids::PHYSICS_IK_ANGLE_OPT);
    v.extend(ids::PHYSICS_JOINT_TOOL_OPT);
    v.extend(ids::PHYSICS_LAYER_CELL);
    v.extend(rows::section_header_ids());
    v.extend([
        ids::PHYSICS_SHOW_COLLIDERS,
        ids::PHYSICS_SLEEP_SPIN,
        ids::PHYSICS_CLEAR_RUN,
        ids::PHYSICS_RESTORE_RUN,
        ids::PHYSICS_RESET_DEFAULTS,
        ids::PHYSICS_CLOSE,
    ]);
    v
}

/// ⭐ Todo id do inventário: registado no store e, salvo as barras (a pega do chip faz-lhes o
/// trabalho), pintado em ALGUM estado alcançável dos rádios e da fita.
#[test]
fn every_id_of_the_old_panel_is_registered_and_painted_in_cards() {
    let barras: Vec<NodeId> = rows::rows()
        .map(|r| r.slider)
        .chain(interact::IROWS.iter().map(|r| r.slider))
        .collect();
    let mut pintados = std::collections::BTreeSet::new();
    let fitas = [(0.0, 0.0), (2.0, 0.0), (0.0, 2.0)];
    for (i, it) in estados().into_iter().enumerate() {
        let (gravada, descartada) = fitas[i % fitas.len()];
        let mesa = Mesa::nova(PhysicsSnapshot {
            recorded_run_seconds: gravada,
            discarded_run_seconds: descartada,
            ..mundo(PhysicsSettings::default(), it)
        });
        pintados.extend(mesa.rects.iter().map(|(id, _)| *id));
        for id in inventario() {
            assert!(mesa.host.store().get(id).is_some() || rows::is_section_header(id));
        }
    }
    assert!(!pintados.is_empty());
    let mortos: Vec<NodeId> = inventario()
        .into_iter()
        .filter(|id| !barras.contains(id) && !pintados.contains(id))
        .collect();
    assert!(mortos.is_empty(), "nunca pintados: {mortos:?}");
    for b in barras {
        assert!(
            !pintados.contains(&b),
            "a barra {b:?} ainda se pinta no cartão"
        );
    }
}

/// ⭐⭐ **Cada número do mundo pelo SCRUB real**: o gesto na pega muda o campo da linha, e só esse;
/// o Esc devolve-o no mundo, não só na caixa. *Mutação: tirar o espelho do `escape_scrub` ⇒ o Esc.*
#[test]
fn scrubbing_each_world_field_writes_its_setting_and_escape_gives_it_back() {
    use ph2d_editor_core::interaction::dispatch_key;
    for row in rows::rows() {
        let antes = PhysicsSettings::default();
        let mut mesa = Mesa::nova(mundo(antes, InteractionSettings::default()));
        let dx = if (row.get)(&antes) >= row.max {
            -60.0
        } else {
            60.0
        };
        let ev = mesa.arrasta_a_pega(row.chip, dx);
        assert!(
            ev.contains(&WidgetEvent::ValueChanged(row.slider)),
            "`{}`: o chip tem de avisar pela barra: {ev:?}",
            row.label
        );
        let got = ultimo_mundo(&mesa.aplica(ev)).expect("o scrub escreveu no mundo");
        assert!(
            ((row.get)(&got) - (row.get)(&antes)).abs() > 1e-4,
            "`{}` não se moveu",
            row.label
        );
        let mut so_esta = got;
        (row.set)(&mut so_esta, (row.get)(&antes));
        assert_eq!(so_esta, antes, "`{}` mexeu noutro campo", row.label);

        let arena = bumpalo::Bump::new();
        let esc = ph2d_host::KeyEvent {
            keycode: 0x1B,
            modifiers: ph2d_host::Modifiers::default(),
            kind: ph2d_host::KeyKind::Down,
            timestamp_ns: 0,
        };
        let ev = dispatch_key(mesa.host.store_mut(), esc, &arena).to_vec();
        let voltou = ultimo_mundo(&mesa.aplica(ev)).expect("o Esc tem de escrever no mundo");
        assert!(
            ((row.get)(&voltou) - (row.get)(&antes)).abs() < 1e-3,
            "`{}`: o Esc mostrou o valor velho e o mundo ficou com o arrastado",
            row.label
        );
    }
}

/// O número escrito à mão e as linhas da interacção: o mesmo caminho, destino `SetInteraction`.
#[test]
fn typing_and_scrubbing_reach_the_tool_rows() {
    let mut mesa = Mesa::nova(mundo(
        PhysicsSettings::default(),
        InteractionSettings::default(),
    ));
    let ev = mesa.host.type_into_number(ids::PHYSICS_GRAVITY_X_NUM, "3");
    let got = ultimo_mundo(&mesa.aplica(ev)).expect("escrever chega ao mundo");
    assert!((got.gravity_x - 3.0).abs() < 1e-3, "{}", got.gravity_x);

    for row in interact::IROWS {
        let it = estados()
            .into_iter()
            .find(|s| (row.shown)(s))
            .expect("a linha é alcançável");
        let mut mesa = Mesa::nova(mundo(PhysicsSettings::default(), it));
        let dx = if (row.get)(&it) >= row.max {
            -60.0
        } else {
            60.0
        };
        let ev = mesa.arrasta_a_pega(row.chip, dx);
        let intents = mesa.aplica(ev);
        let got = intents
            .iter()
            .rev()
            .find_map(|i| match i {
                PhysicsIntent::SetInteraction(s) => Some(*s),
                _ => None,
            })
            .unwrap_or_else(|| panic!("`{}` não chegou à ferramenta: {intents:?}", row.label));
        assert!(
            ((row.get)(&got) - (row.get)(&it)).abs() > 1e-4,
            "`{}` não se moveu",
            row.label
        );
        let _ = mesa.solta(row.chip, dx);
    }
}

/// Os rádios, as caixas, os verbos, o fecho e a matriz: clique real no rect pintado ⇒ o efeito.
#[test]
fn every_choice_check_verb_and_cell_answers_a_real_click() {
    let base = mundo(PhysicsSettings::default(), InteractionSettings::default());
    let mut mesa = Mesa::nova(base);
    for (n, &id) in ids::PHYSICS_INTERACT_TOOL_OPT.iter().enumerate() {
        let i = mesa.clica(id);
        assert!(
            matches!(i[..], [PhysicsIntent::SetInteraction(s)] if s.tool == InteractionTool::ALL[n]),
            "ferramenta {n}: {i:?}"
        );
    }
    for (n, &id) in ids::PHYSICS_HOLD_MODE_OPT.iter().enumerate() {
        let i = mesa.clica(id);
        assert!(
            matches!(i[..], [PhysicsIntent::SetInteraction(s)] if s.hold == HoldMode::ALL[n]),
            "pega {n}: {i:?}"
        );
    }
    for (n, &id) in ids::PHYSICS_JOINT_TOOL_OPT.iter().enumerate() {
        let i = mesa.clica(id);
        assert!(
            matches!(i[..], [PhysicsIntent::SetInteraction(s)] if s.joint == JointTool::ALL[n]),
            "junta {n}: {i:?}"
        );
    }
    let i = mesa.clica(ids::PHYSICS_SHOW_COLLIDERS);
    assert!(matches!(i[..], [PhysicsIntent::ToggleColliders]), "{i:?}");
    let i = mesa.clica(ids::PHYSICS_SLEEP_SPIN);
    let dorme = ultimo_mundo(&i).expect("o sono escreve").sleep_enabled();
    assert_eq!(
        dorme,
        !base.vivo.sleep_enabled(),
        "a caixa vira o que se via"
    );

    let matriz = LayerMatrix::from_rows(base.vivo.layer_matrix);
    for a in 0..ph2d_physics_ecs::MAX_LAYERS {
        for b in 0..=a {
            let id = ids::PHYSICS_LAYER_CELL[ids::physics_layer_cell_index(a, b)];
            let got = ultimo_mundo(&mesa.clica(id)).expect("a célula escreve");
            let got = LayerMatrix::from_rows(got.layer_matrix);
            assert_eq!(got.collides(a, b), !matriz.collides(a, b), "par ({a},{b})");
        }
    }
    let i = mesa.clica(ids::PHYSICS_RESET_DEFAULTS);
    assert_eq!(ultimo_mundo(&i), Some(PhysicsSettings::default()));

    let mut gravada = Mesa::nova(PhysicsSnapshot {
        recorded_run_seconds: 2.0,
        ..base
    });
    let i = gravada.clica(ids::PHYSICS_CLEAR_RUN);
    assert!(matches!(i[..], [PhysicsIntent::ClearRun]), "{i:?}");
    let mut descartada = Mesa::nova(PhysicsSnapshot {
        discarded_run_seconds: 2.0,
        ..base
    });
    let i = descartada.clica(ids::PHYSICS_RESTORE_RUN);
    assert!(matches!(i[..], [PhysicsIntent::RestoreRun]), "{i:?}");

    let mut m2 = Mesa::nova(base);
    let _ = m2.clica(ids::PHYSICS_CLOSE);
    assert!(
        !m2.host.panel_visible("physics"),
        "o «×» da lista fecha o painel"
    );
}

/// O arrasto que pinta a matriz continua a funcionar dentro do cartão.
#[test]
fn dragging_across_the_matrix_paints_each_crossed_cell() {
    let mut mesa = Mesa::nova(mundo(
        PhysicsSettings::default(),
        InteractionSettings::default(),
    ));
    let centro = |mesa: &Mesa, a, b| {
        let r = mesa
            .rect(ids::PHYSICS_LAYER_CELL[ids::physics_layer_cell_index(a, b)])
            .expect("célula pintada");
        (r.x + r.w * 0.5, r.y + r.h * 0.5)
    };
    let (x0, y0) = centro(&mesa, 7, 0);
    let (x1, y1) = centro(&mesa, 7, 7);
    let ev = mesa.host.drag_at(x0, y0, x1, y1);
    let got = ultimo_mundo(&mesa.aplica(ev)).expect("o arrasto escreve");
    let got = LayerMatrix::from_rows(got.layer_matrix);
    for j in 0..8 {
        assert!(!got.collides(7, j), "o par (7,{j}) ficou por pintar");
    }
    assert!(got.collides(6, 0), "uma célula fora do caminho não muda");
}

/// O cabeçalho dobra pelo despacho da casa; dobrado o corpo sai, e o mundo não se mexe.
#[test]
fn every_header_folds_its_card_without_touching_the_world() {
    let mut mesa = Mesa::nova(mundo(
        PhysicsSettings::default(),
        InteractionSettings::default(),
    ));
    assert!(mesa.rect(ids::PHYSICS_GRAVITY_X_NUM).is_some());
    for id in rows::section_header_ids() {
        assert!(!mesa.host.store().is_collapsed(id));
        let r = mesa.rect(id).expect("o cabeçalho pinta-se");
        let ev = mesa
            .host
            .click_at(r.x + r.w * 0.3, r.y + m::CARD_HEAD_H * 0.5);
        let i = mesa.aplica(ev);
        assert!(i.is_empty(), "dobrar pediu {i:?}");
        assert!(mesa.host.store().is_collapsed(id), "{id:?} não dobrou");
        mesa.assenta();
    }
    assert!(
        mesa.rect(ids::PHYSICS_GRAVITY_X_NUM).is_none(),
        "dobrado, o corpo não regista clique"
    );
}

/// Arrastar o cabeçalho reordena (a mesma pilha do Kanban) e grava a ordem das secções.
#[test]
fn dragging_a_card_by_its_header_reorders_the_list() {
    let mut mesa = Mesa::nova(mundo(
        PhysicsSettings::default(),
        InteractionSettings::default(),
    ));
    let a = mesa.rect(ids::PHYSICS_SEC_WORLD).expect("World");
    let b = mesa.rect(ids::PHYSICS_SEC_AIR).expect("Air");
    let (x, y0) = (a.x + a.w * 0.4, a.y + m::CARD_HEAD_H * 0.5);
    let _ = mesa.ponteiro(PointerKind::Down, x, y0);
    let _ = mesa.ponteiro(PointerKind::Move, x, y0 + m::CARD_HEAD_H);
    mesa.assenta();
    let alvo = b.y + b.h * 2.0;
    let _ = mesa.ponteiro(PointerKind::Move, x, alvo);
    mesa.assenta();
    let _ = mesa.ponteiro(PointerKind::Up, x, alvo);
    mesa.assenta();
    let ordem = mesa.host.store().section_order().to_vec();
    let pos = |id| ordem.iter().position(|&k| k == id);
    assert!(
        pos(ids::PHYSICS_SEC_WORLD) > pos(ids::PHYSICS_SEC_SOLVER),
        "World não desceu: {ordem:?}"
    );
    assert!(ultimo_mundo(&drain_intents()).is_none());
}

/// Todo ícone e toda célula muda têm dica; o campo leva a do arrasto com o rótulo dele.
#[test]
fn every_mute_control_in_the_cards_has_a_tooltip() {
    let mesa = Mesa::nova(mundo(
        PhysicsSettings::default(),
        InteractionSettings::default(),
    ));
    let store = mesa.host.store();
    let panel = ph2d_editor_core::ids::PHYSICS_PANEL;
    let mudos: Vec<NodeId> = dicas::icon_only_ids(panel)
        .into_iter()
        .chain(ids::PHYSICS_LAYER_CELL)
        .chain([
            ids::PHYSICS_CLOSE,
            ph2d_editor_core::panel::cards::list_menu_of(panel),
        ])
        .filter(|id| store.tooltip_for(*id).is_none())
        .collect();
    assert!(mudos.is_empty(), "sem dica: {mudos:?}");
    let dica = dicas::field_hint(store, ids::PHYSICS_SLEEP_DELAY_NUM).expect("campo marcado");
    assert!(
        dica.contains("(s)"),
        "a dica nomeia o campo com a unidade: {dica}"
    );
}

/// `PH2D_CARDS=0`: o corpo de antes, com as barras pintadas e sem lista flutuante.
#[test]
fn cards_off_paints_the_old_body() {
    cards::force_cards_on_this_thread(Some(false));
    set_current_physics(Some(PhysicsSnapshot::default()));
    let mut host = MockPanelHost::with_panel::<PhysicsPanel>();
    let mut st = PhysicsPanelState;
    let rects = host.paint::<PhysicsPanel>(&mut st, COLUNA);
    assert!(rects.iter().any(|(id, _)| *id == ids::PHYSICS_GRAVITY_X));
    assert!(rects.iter().any(|(id, _)| *id == ids::PHYSICS_CLOSE));
    assert!(
        host.store()
            .card_field(ids::PHYSICS_GRAVITY_X_NUM)
            .is_none(),
        "o corpo de antes não marca campos de cartão"
    );
}

/// A ordem natural dos cartões é a lista que o `populate` regista e o `event` despacha.
#[test]
fn the_natural_order_of_the_cards_is_the_section_table() {
    let mesa = Mesa::nova(PhysicsSnapshot::default());
    let tops: Vec<(NodeId, f32)> = rows::section_header_ids()
        .map(|id| (id, mesa.rect(id).expect("cartão").y))
        .collect();
    assert!(
        tops.windows(2).all(|w| w[0].1 < w[1].1),
        "fora da ordem: {tops:?}"
    );
    assert_eq!(
        ph2d_panel_physics::state::last_painted_section_headers(),
        rows::section_header_ids().collect::<Vec<_>>()
    );
}

/// O Debug nasce dobrado com o resumo, e a escolha do artista ganha à semente.
#[test]
fn the_debug_card_is_born_folded_and_a_click_opens_it() {
    cards::force_cards_on_this_thread(Some(true));
    set_current_physics(Some(PhysicsSnapshot::default()));
    let mut mesa = Mesa {
        host: MockPanelHost::with_panel::<PhysicsPanel>(),
        st: PhysicsPanelState,
        rects: Vec::new(),
    };
    mesa.assenta();
    assert!(mesa.host.store().is_collapsed(ids::PHYSICS_SEC_DEBUG));
    assert!(mesa.rect(ids::PHYSICS_RESET_DEFAULTS).is_none());
    let r = mesa.rect(ids::PHYSICS_SEC_DEBUG).expect("cabeçalho");
    let _ = mesa
        .host
        .click_at(r.x + r.w * 0.3, r.y + m::CARD_HEAD_H * 0.5);
    mesa.assenta();
    assert!(
        mesa.rect(ids::PHYSICS_RESET_DEFAULTS).is_some(),
        "o clique abriu e a semente não o fechou de novo"
    );
}

/// O botão direito no cabeçalho abre o menu do tema/contorno da secção, como no Bones.
#[test]
fn right_clicking_a_card_header_opens_its_theme_menu() {
    use ph2d_editor_core::interaction::ContextMenuKind;
    let mut mesa = Mesa::nova(PhysicsSnapshot::default());
    let r = mesa.rect(ids::PHYSICS_SEC_AIR).expect("cartão");
    let ev = mesa
        .host
        .right_click_at(r.x + r.w * 0.3, r.y + m::CARD_HEAD_H * 0.5);
    assert!(ultimo_mundo(&mesa.aplica(ev)).is_none());
    let menu = mesa.host.store().context_menu().map(|c| c.kind);
    assert!(
        matches!(menu, Some(ContextMenuKind::SectionOutline { section }) if section == ids::PHYSICS_SEC_AIR),
        "{menu:?}"
    );
}

/// Arrastar o cartão para fora da coluna solta-o, e os ids do corpo continuam a responder lá.
#[test]
fn a_card_dragged_out_of_the_column_comes_loose_and_keeps_working() {
    let panel = ph2d_editor_core::ids::PHYSICS_PANEL;
    let coluna = Rect::new(1200.0, 0.0, 340.0, 1600.0);
    cards::force_cards_on_this_thread(Some(true));
    set_current_physics(Some(PhysicsSnapshot::default()));
    let _ = drain_intents();
    let mut host = MockPanelHost::with_panel::<PhysicsPanel>();
    let mut st = PhysicsPanelState;
    let mut pinta = |host: &mut MockPanelHost, n: usize| {
        let mut r = Vec::new();
        for _ in 0..n {
            r = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || host.paint::<PhysicsPanel>(&mut st, coluna),
            );
        }
        r
    };
    let rects = pinta(&mut host, 60);
    let cab = rects
        .iter()
        .rev()
        .find(|(id, _)| *id == ids::PHYSICS_SEC_WORLD)
        .expect("World")
        .1;
    let ev = |kind, x, y| PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: 0,
    };
    let (x0, y0) = (cab.x + 30.0, cab.y + m::CARD_HEAD_H * 0.5);
    let _ = host.dispatch_pointer_event(ev(PointerKind::Down, x0, y0));
    pinta(&mut host, 2);
    for i in 1..=8 {
        let t = i as f32 / 8.0;
        let _ = host.dispatch_pointer_event(ev(PointerKind::Move, x0 - 800.0 * t, y0 + 200.0 * t));
        pinta(&mut host, 2);
    }
    let _ = host.dispatch_pointer_event(ev(PointerKind::Up, x0 - 800.0, y0 + 200.0));
    let rects = pinta(&mut host, 30);
    assert!(
        host.store().is_loose_in(ids::PHYSICS_SEC_WORLD, panel),
        "o cartão não ficou solto"
    );
    let campo = rects
        .iter()
        .rev()
        .find(|(id, _)| *id == ids::PHYSICS_GRAVITY_X_NUM)
        .expect("o campo pinta-se no solto")
        .1;
    assert!(campo.x < coluna.x, "o campo saiu com o cartão: {campo:?}");
    let (x, y) = (campo.x + m::ICON_CELL * 0.5, campo.y + campo.h * 0.5);
    let mut evs = host.dispatch_pointer_event(ev(PointerKind::Down, x, y));
    evs.extend(host.dispatch_pointer_event(ev(PointerKind::Move, x + 30.0, y)));
    evs.extend(host.dispatch_pointer_event(ev(PointerKind::Move, x + 60.0, y)));
    for e in evs {
        host.apply_panel_event::<PhysicsPanel>(&mut st, e);
    }
    let got = ultimo_mundo(&drain_intents()).expect("o scrub do solto escreve no mundo");
    assert!(got.gravity_x.abs() > 1e-4, "{}", got.gravity_x);
}

/// A lista mais alta que o encaixe rola; o que estava abaixo da dobra sobe e responde ao clique.
#[test]
fn a_list_taller_than_its_slot_scrolls_to_the_lower_cards() {
    let baixa = Rect::new(0.0, 0.0, 340.0, 500.0);
    cards::force_cards_on_this_thread(Some(true));
    set_current_physics(Some(PhysicsSnapshot::default()));
    let _ = drain_intents();
    let mut host = MockPanelHost::with_panel::<PhysicsPanel>();
    let mut st = PhysicsPanelState;
    let mut pinta = |host: &mut MockPanelHost| {
        let mut r = Vec::new();
        for _ in 0..60 {
            r = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || host.paint::<PhysicsPanel>(&mut st, baixa),
            );
        }
        r
    };
    let antes = pinta(&mut host);
    let visivel = |rects: &[(NodeId, Rect)], id| {
        rects
            .iter()
            .rev()
            .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0 && r.y + r.h < baixa.h)
            .map(|(_, r)| *r)
    };
    let alvo = ids::PHYSICS_SEC_DEBUG;
    assert!(
        visivel(&antes, alvo).is_none(),
        "o Debug começa abaixo da dobra"
    );
    let _ = host.wheel_at(baixa.w * 0.5, baixa.h * 0.5, -3000.0);
    let depois = pinta(&mut host);
    let r = visivel(&depois, alvo).expect("rolou até ao Debug");
    let _ = host.click_at(r.x + r.w * 0.3, r.y + m::CARD_HEAD_H * 0.5);
    assert!(
        !host.store().is_collapsed(alvo),
        "o clique no cartão que subiu chega-lhe: o Debug nasce dobrado e abre"
    );
}

/// O Auto Key: o que é canal do mundo mantém a marca clicável com o MESMO id; o que é de sessão ou
/// da vista não a leva. *O id do chip é o que `ph2d_app_physics::marcas_mundo::CAMPOS` nomeia.*
#[test]
fn world_channels_keep_a_clickable_mark_and_session_settings_have_none() {
    use ph2d_editor_core::widget::key_mark::{KeyMark, key_mark_target, set_key_marks};
    let marca_de = |id: NodeId| -> bool {
        set_key_marks(std::collections::BTreeMap::from([(id, KeyMark::Chave)]));
        let mesa = Mesa::nova(PhysicsSnapshot::default());
        let tem = mesa
            .rects
            .iter()
            .filter_map(|(alvo, _)| key_mark_target(*alvo))
            .any(|c| c.fields.contains(&id));
        set_key_marks(std::collections::BTreeMap::new());
        tem
    };
    let canais = rows::rows()
        .map(|r| r.chip)
        .chain([ids::PHYSICS_SLEEP_SPIN, ids::PHYSICS_LAYER_CELL[0]]);
    for id in canais {
        assert!(marca_de(id), "{id:?} perdeu a marca de animação no cartão");
    }
    for id in [ids::PHYSICS_HOLD_STIFFNESS_NUM, ids::PHYSICS_SHOW_COLLIDERS] {
        assert!(!marca_de(id), "{id:?} não é canal e ganhou marca");
    }
}
