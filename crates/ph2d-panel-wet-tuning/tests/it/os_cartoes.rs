//! ⭐⭐ **O painel Wet Paint em cartões: nenhum controlo morreu.** O inventário vem da tabela que o corpo
//! de antes percorria; cada id é provado pelo gesto REAL (ponteiro → despacho → `apply_event` → ferramenta),
//! nos rects que a pintura registou.

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::Tool;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as eids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::cards::dicas;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_wet_tuning::{WetTuningPanel, cards, rows, set_current_brush, state};
use ph2d_tool_painter::ids as tid;
use ph2d_tool_painter::{PaintMedia, PainterTool, WetKnobs, WetTool};
use ph2d_ui_testkit::MockPanelHost;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 340.0,
    h: 5200.0,
};

fn ferramenta(na_mao: WetTool, oleo: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_oleo_real_ligado(oleo);
    t.handle_panel_event(PanelEvent::SelectOption(
        eids::PAINTER_PAINT_MODE,
        na_mao.fio().to_owned(),
    ));
    t
}

struct Mesa {
    host: MockPanelHost,
    st: state::WetTuningPanelState,
    rects: Vec<(NodeId, Rect)>,
    t: PainterTool,
    janela: Rect,
}

impl Mesa {
    fn nova(t: PainterTool) -> Self {
        Self::na_janela(t, JANELA)
    }

    fn na_janela(t: PainterTool, janela: Rect) -> Self {
        cards::force_cards_on_this_thread(Some(true));
        set_current_brush(Some(t.brush_settings()));
        let mut host = MockPanelHost::with_panel::<WetTuningPanel>();
        host.set_panel_visible(WetTuningPanel::ID, true);
        for s in rows::SECTIONS {
            host.store_mut().set_collapsed(s.header, false);
        }
        let mut mesa = Self {
            host,
            st: state::WetTuningPanelState,
            rects: Vec::new(),
            t,
            janela,
        };
        mesa.assenta();
        mesa
    }

    fn assenta(&mut self) {
        for _ in 0..60 {
            let (h, st, j) = (&mut self.host, &mut self.st, self.janela);
            self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || h.paint::<WetTuningPanel>(st, j),
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

    /// Os eventos pelo painel; o que ele manda à ferramenta chega lá, e a pintura seguinte vê o pincel.
    fn entrega(&mut self, eventos: Vec<WidgetEvent>) -> Vec<PanelEvent> {
        for ev in eventos {
            let _ = self
                .host
                .apply_panel_event::<WetTuningPanel>(&mut self.st, ev);
        }
        let mut pedidos = Vec::new();
        for a in self.host.drained_actions() {
            if let EditorAction::ToolPanelEvent(pe) = a {
                pedidos.push(pe.clone());
                self.t.handle_panel_event(pe);
            }
        }
        set_current_brush(Some(self.t.brush_settings()));
        pedidos
    }

    fn clica(&mut self, id: NodeId) -> Vec<PanelEvent> {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("{id:?} não pintado"));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(self.host.hit_at(cx, cy), Some(id), "outro id no centro");
        let ev = self.host.click_at(cx, cy);
        self.entrega(ev)
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
}

fn dona(row: &rows::TuneRow) -> WetTool {
    WetKnobs::dona_do_knob(ph2d_wet_paint::tuning::KNOB_DEFS[row.knob].knob)
        .unwrap_or(WetTool::Paint)
}

/// ⭐ Todo id do inventário registado e, salvo as barras, pintado com a ferramenta que o lê na mão.
#[test]
fn every_id_of_the_old_panel_is_registered_and_painted_in_cards() {
    let mut inv: Vec<NodeId> = vec![tid::WET_TUNING_KM_GLAZE, tid::WET_TUNING_CLOSE];
    let mut barras = Vec::new();
    for row in rows::rows() {
        inv.push(row.reset);
        match &row.options {
            Some(o) => inv.extend(o.iter().map(|(_, id)| *id)),
            None => {
                inv.push(row.chip);
                barras.push(row.slider);
            }
        }
    }
    for s in rows::SECTIONS {
        inv.extend([s.header, s.reset]);
    }
    inv.extend(tid::WET_TUNING_GROUP_HEADERS.iter().copied());
    inv.extend([
        tid::WET_TUNING_DRAG_HANDLE,
        tid::WET_TUNING_RESIZE_HANDLE,
        tid::WET_TUNING_RESIZE_HANDLE_BL,
    ]);
    let mut pintados = std::collections::BTreeSet::new();
    for na_mao in [
        WetTool::Paint,
        WetTool::Smear,
        WetTool::Blend,
        WetTool::Wet,
        WetTool::Dry,
        WetTool::Blow,
    ] {
        let mesa = Mesa::nova(ferramenta(na_mao, true));
        pintados.extend(mesa.rects.iter().map(|(id, _)| *id));
    }
    let mortos: Vec<NodeId> = inv
        .iter()
        .copied()
        .filter(|id| !pintados.contains(id))
        .collect();
    assert!(mortos.is_empty(), "nunca pintados: {mortos:?}");
    for b in barras {
        assert!(!pintados.contains(&b), "a barra {b:?} ainda se pinta");
    }
}

/// ⭐⭐ Cada número pelo SCRUB real: o gesto chega ao knob da linha, e só a esse; o Esc devolve-o.
#[test]
fn scrubbing_each_knob_reaches_the_tool_and_escape_gives_it_back() {
    use ph2d_editor_core::interaction::dispatch_key;
    for row in rows::rows().iter().filter(|r| r.options.is_none()) {
        let t = ferramenta(dona(row), true);
        let antes = t.brush_settings().wet_knobs.knobs;
        let dx = if row.track_of(antes[row.knob]) >= 0.5 {
            -60.0
        } else {
            60.0
        };
        let mut mesa = Mesa::nova(t);
        let ev = mesa.arrasta_a_pega(row.chip, dx);
        mesa.entrega(ev);
        let depois = mesa.t.brush_settings().wet_knobs.knobs;
        assert!(
            (depois[row.knob] - antes[row.knob]).abs() > 1e-4,
            "`{}` não se moveu: {} → {}",
            row.key,
            antes[row.knob],
            depois[row.knob]
        );
        for (i, (a, d)) in antes.iter().zip(depois.iter()).enumerate() {
            assert!(i == row.knob || a == d, "`{}` mexeu no knob {i}", row.key);
        }
        let arena = bumpalo::Bump::new();
        let esc = ph2d_host::KeyEvent {
            keycode: 0x1B,
            modifiers: ph2d_host::Modifiers::default(),
            kind: ph2d_host::KeyKind::Down,
            timestamp_ns: 0,
        };
        let ev = dispatch_key(mesa.host.store_mut(), esc, &arena).to_vec();
        mesa.entrega(ev);
        let voltou = mesa.t.brush_settings().wet_knobs.knobs[row.knob];
        assert!(
            (voltou - antes[row.knob]).abs() < row.step.max(1e-6) * 1.01,
            "`{}`: o Esc deixou {voltou} (era {})",
            row.key,
            antes[row.knob]
        );
    }
}

/// Duplo clique na pega volta ao valor de fábrica da linha. *Mutação: `default: None` no `prepare` ⇒
/// o duplo clique não faz nada e o knob fica onde o scrub o deixou.*
#[test]
fn double_clicking_the_handle_gives_the_factory_value_back() {
    let row = rows::rows()
        .iter()
        .find(|r| r.options.is_none())
        .expect("um número")
        .clone();
    let mut mesa = Mesa::nova(ferramenta(dona(&row), true));
    let ev = mesa.arrasta_a_pega(row.chip, 60.0);
    mesa.entrega(ev);
    let r = mesa.rect(row.chip).expect("campo");
    let (x, y) = (r.x + m::ICON_CELL * 0.5, r.y + r.h * 0.5);
    let ev = mesa.ponteiro(PointerKind::Up, x + 60.0, y);
    mesa.entrega(ev);
    mesa.assenta();
    let mexido = mesa.t.brush_settings().wet_knobs.knobs[row.knob];
    assert!((mexido - row.default).abs() > 1e-4, "o scrub tem de o tirar da fábrica");
    for _ in 0..2 {
        let ev = mesa.ponteiro(PointerKind::Down, x, y);
        mesa.entrega(ev);
        let ev = mesa.ponteiro(PointerKind::Up, x, y);
        mesa.entrega(ev);
    }
    let voltou = mesa.t.brush_settings().wet_knobs.knobs[row.knob];
    assert!(
        (voltou - row.default).abs() < 1e-4,
        "`{}`: {voltou}, a fábrica é {}",
        row.key,
        row.default
    );
}

/// O número escrito à mão chega ao knob.
#[test]
fn typing_a_number_reaches_the_tool() {
    let row = rows::rows()
        .iter()
        .find(|r| r.key == "textureSizeRef")
        .expect("a linha com unidade");
    let mut mesa = Mesa::nova(ferramenta(WetTool::Paint, true));
    let alvo = 500.0;
    let ev = mesa.host.type_into_number(row.chip, &format!("{alvo:.0}"));
    mesa.entrega(ev);
    let got = mesa.t.brush_settings().wet_knobs.knobs[row.knob];
    assert!((got - alvo).abs() <= row.step, "{got} em vez de {alvo}");
}

/// O repor de cada knob e o de cada grupo, a escolha, a caixa e o «×»: clique real ⇒ o efeito.
#[test]
fn every_reset_choice_check_and_close_answers_a_real_click() {
    for row in rows::rows() {
        let mut mesa = Mesa::nova(ferramenta(dona(row), true));
        let i = mesa.clica(row.reset);
        assert!(
            i.iter()
                .any(|e| matches!(e, PanelEvent::Click(id) if *id == row.reset)),
            "`{}`: o repor pediu {i:?}",
            row.key
        );
        if let Some(opcoes) = &row.options {
            for (n, (_, id)) in opcoes.iter().enumerate() {
                mesa.clica(*id);
                let v = mesa.t.brush_settings().wet_knobs.knobs[row.knob];
                assert!((v - n as f64).abs() < 1e-9, "`{}` opção {n}: {v}", row.key);
                mesa.assenta();
            }
        }
    }
    for s in rows::SECTIONS {
        let mut mesa = Mesa::nova(ferramenta(WetTool::Paint, true));
        let i = mesa.clica(s.reset);
        assert!(
            i.iter()
                .any(|e| matches!(e, PanelEvent::Click(id) if *id == s.reset)),
            "repor do grupo: {i:?}"
        );
    }
    let mut mesa = Mesa::nova(ferramenta(WetTool::Paint, true));
    let antes = mesa.t.brush_settings().wet_km_glaze;
    mesa.clica(tid::WET_TUNING_KM_GLAZE);
    assert_ne!(mesa.t.brush_settings().wet_km_glaze, antes, "a caixa vira");
    let i = mesa.clica(tid::WET_TUNING_CLOSE);
    assert!(
        i.iter()
            .any(|e| matches!(e, PanelEvent::Click(id) if *id == tid::PAINTER_WETPAINT_TUNING)),
        "o «×» é o mesmo Tuning da secção básica: {i:?}"
    );
}

/// A janela continua a mover-se pela faixa do título e a redimensionar pelos cantos.
#[test]
fn the_floating_list_moves_by_its_title_and_resizes_by_its_corner() {
    let viewport = Rect::new(0.0, 0.0, 1900.0, 1000.0);
    let mut mesa = Mesa::na_janela(ferramenta(WetTool::Paint, true), viewport);
    let cheio = eids::WET_TUNING_PANEL;
    // O encaixe aqui é a janela inteira: encolhe-a para o arrasto ter para onde ir.
    mesa.host
        .store_mut()
        .set_panel_resize_delta(cheio, -1500.0, -200.0);
    mesa.assenta();
    let faixa = mesa
        .rect(tid::WET_TUNING_DRAG_HANDLE)
        .expect("a faixa do título");
    let w0 = faixa.w;
    let (x, y) = (faixa.x + 10.0, faixa.y + faixa.h * 0.5);
    let _ = mesa.ponteiro(PointerKind::Down, x, y);
    let _ = mesa.ponteiro(PointerKind::Move, x + 200.0, y + 60.0);
    let _ = mesa.ponteiro(PointerKind::Up, x + 200.0, y + 60.0);
    mesa.assenta();
    let movida = mesa.rect(tid::WET_TUNING_DRAG_HANDLE).expect("a faixa");
    assert!(
        (movida.x - faixa.x - 200.0).abs() < 1.0,
        "{faixa:?} → {movida:?}"
    );
    assert!(
        (movida.y - faixa.y - 60.0).abs() < 1.0,
        "{faixa:?} → {movida:?}"
    );
    let canto = mesa.rect(tid::WET_TUNING_RESIZE_HANDLE).expect("o canto");
    let (x, y) = (canto.x + canto.w * 0.5, canto.y + canto.h * 0.5);
    let _ = mesa.ponteiro(PointerKind::Down, x, y);
    let _ = mesa.ponteiro(PointerKind::Move, x + 80.0, y);
    let _ = mesa.ponteiro(PointerKind::Up, x + 80.0, y);
    mesa.assenta();
    let larga = mesa.rect(tid::WET_TUNING_DRAG_HANDLE).expect("a faixa");
    assert!(
        (larga.w - w0 - 80.0).abs() < 1.0,
        "a lista alarga com o canto: {w0} → {}",
        larga.w
    );
    // A faixa não tapa o «×».
    let fecho = mesa.rect(tid::WET_TUNING_CLOSE).expect("o «×»");
    assert_eq!(
        mesa.host
            .hit_at(fecho.x + fecho.w * 0.5, fecho.y + fecho.h * 0.5),
        Some(tid::WET_TUNING_CLOSE)
    );
}

/// A dobra do cartão, a reordenação pela pega e o menu do botão direito.
#[test]
fn headers_fold_reorder_and_open_the_theme_menu() {
    use ph2d_editor_core::interaction::ContextMenuKind;
    let [a, b, ..] = [
        rows::SECTIONS[0].header,
        rows::SECTIONS[1].header,
        rows::SECTIONS[2].header,
    ];
    let mut mesa = Mesa::nova(ferramenta(WetTool::Paint, true));
    let r = mesa.rect(a).expect("cartão");
    let primeiro = rows::rows()
        .iter()
        .find(|x| x.group == rows::SECTIONS[0].group && x.options.is_none())
        .expect("uma linha")
        .chip;
    assert!(mesa.rect(primeiro).is_some());
    let ev = mesa
        .host
        .click_at(r.x + r.w * 0.3, r.y + m::CARD_HEAD_H * 0.5);
    assert!(
        mesa.entrega(ev).is_empty(),
        "dobrar não fala com a ferramenta"
    );
    assert!(mesa.host.store().is_collapsed(a));
    mesa.assenta();
    assert!(
        mesa.rect(primeiro).is_none(),
        "dobrado, o corpo não regista clique"
    );
    let ev = mesa
        .host
        .click_at(r.x + r.w * 0.3, r.y + m::CARD_HEAD_H * 0.5);
    let _ = mesa.entrega(ev);
    mesa.assenta();

    let ra = mesa.rect(a).expect("A");
    let rb = mesa.rect(b).expect("B");
    let (x, y0) = (ra.x + ra.w * 0.4, ra.y + m::CARD_HEAD_H * 0.5);
    let _ = mesa.ponteiro(PointerKind::Down, x, y0);
    let _ = mesa.ponteiro(PointerKind::Move, x, y0 + m::CARD_HEAD_H);
    mesa.assenta();
    let alvo = rb.y + rb.h * 2.0;
    let _ = mesa.ponteiro(PointerKind::Move, x, alvo);
    mesa.assenta();
    let _ = mesa.ponteiro(PointerKind::Up, x, alvo);
    mesa.assenta();
    let ordem = mesa.host.store().section_order().to_vec();
    let pos = |id| ordem.iter().position(|&k| k == id);
    assert!(pos(a) > pos(b), "o cartão não desceu: {ordem:?}");

    let r = mesa.rect(b).expect("B");
    let _ = mesa
        .host
        .right_click_at(r.x + r.w * 0.3, r.y + m::CARD_HEAD_H * 0.5);
    let menu = mesa.host.store().context_menu().map(|c| c.kind);
    assert!(
        matches!(menu, Some(ContextMenuKind::SectionOutline { section }) if section == b),
        "{menu:?}"
    );
}

/// Os avançados continuam a nascer dobrados; a lista rola até ao último e ele responde.
#[test]
fn the_advanced_groups_are_born_folded_and_the_list_scrolls() {
    cards::force_cards_on_this_thread(Some(true));
    set_current_brush(Some(ferramenta(WetTool::Paint, true).brush_settings()));
    let baixa = Rect::new(0.0, 0.0, 340.0, 500.0);
    let mut host = MockPanelHost::with_panel::<WetTuningPanel>();
    host.set_panel_visible(WetTuningPanel::ID, true);
    let dobrados: Vec<NodeId> = rows::SECTIONS
        .iter()
        .filter(|s| tid::WET_TUNING_GRUPOS_AVANCADOS.contains(&s.group))
        .map(|s| s.header)
        .collect();
    assert!(!dobrados.is_empty());
    for s in rows::SECTIONS {
        host.store_mut()
            .set_collapsed(s.header, dobrados.contains(&s.header));
    }
    let mut st = state::WetTuningPanelState;
    let mut pinta = |host: &mut MockPanelHost| {
        let mut r = Vec::new();
        for _ in 0..60 {
            r = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || host.paint::<WetTuningPanel>(&mut st, baixa),
            );
        }
        r
    };
    let antes = pinta(&mut host);
    let alvo = ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[4];
    let visivel = |rects: &[(NodeId, Rect)]| {
        rects
            .iter()
            .rev()
            .find(|(n, r)| *n == alvo && r.w > 0.0 && r.h > 0.0 && r.y + r.h < baixa.h)
            .map(|(_, r)| *r)
    };
    assert!(
        visivel(&antes).is_none(),
        "o Experimental começa abaixo da dobra"
    );
    let _ = host.wheel_at(baixa.w * 0.5, baixa.h * 0.5, -6000.0);
    let depois = pinta(&mut host);
    let r = visivel(&depois).expect("rolou até ao Experimental");
    let _ = host.click_at(r.x + r.w * 0.3, r.y + m::CARD_HEAD_H * 0.5);
    assert!(
        host.store().is_collapsed(alvo),
        "o clique chega ao que subiu"
    );
}

/// Todo ícone mudo tem dica; a linha inerte continua com a dica do que a acorda e editável.
#[test]
fn every_mute_control_has_a_tooltip_and_an_inert_row_says_what_wakes_it() {
    let mesa = Mesa::nova(ferramenta(WetTool::Paint, false));
    let store = mesa.host.store();
    let panel = eids::WET_TUNING_PANEL;
    let mudos: Vec<NodeId> = dicas::icon_only_ids(panel)
        .into_iter()
        .chain([
            tid::WET_TUNING_CLOSE,
            ph2d_editor_core::panel::cards::list_menu_of(panel),
        ])
        .filter(|id| store.tooltip_for(*id).is_none())
        .collect();
    assert!(mudos.is_empty(), "sem dica: {mudos:?}");
    let dica = ph2d_i18n::tr(ph2d_tool_painter::Dependente::MaterialSemOleo.dica());
    let row = rows::rows()
        .iter()
        .find(|r| r.key == "oilDepth")
        .expect("linha");
    assert_eq!(store.tooltip_for(row.chip), Some(dica));
    let mut mesa = mesa;
    let ev = mesa.arrasta_a_pega(row.chip, 60.0);
    let antes = mesa.t.brush_settings().wet_knobs.knobs[row.knob];
    mesa.entrega(ev);
    assert_ne!(
        mesa.t.brush_settings().wet_knobs.knobs[row.knob],
        antes,
        "inerte, mas editável"
    );
    assert!(
        dicas::field_hint(mesa.host.store(), row.chip).is_some_and(|d| d.contains("Depth")),
        "a dica do campo nomeia-o"
    );
}

/// Nome de controlo nunca vira «...»: o que não cabe em duas linhas de meia coluna ganha a linha toda.
/// *Contraprova: tirar o braço `cabe_em_duas_linhas` de `corpo_do_grupo` ⇒ «Dab rotation (none · follow…».*
#[test]
fn no_knob_label_is_cut_at_the_owners_width() {
    let mut mesa = Mesa::nova(ferramenta(WetTool::Paint, true));
    let medidos = mesa
        .host
        .medindo_a_pintura::<WetTuningPanel>(&mut mesa.st, JANELA);
    let cortados: Vec<&str> = medidos
        .iter()
        .filter(|m| m.pintado != m.texto && m.texto.chars().any(char::is_alphabetic))
        .map(|m| m.texto.as_str())
        .collect();
    assert!(!medidos.is_empty());
    assert!(cortados.is_empty(), "cortados: {cortados:?}");
}

/// A unidade mora no rótulo, nunca na caixa (D13).
#[test]
fn the_unit_lives_in_the_label_not_in_the_box() {
    let mesa = Mesa::nova(ferramenta(WetTool::Paint, true));
    let row = rows::rows()
        .iter()
        .find(|r| r.key == "textureSizeRef")
        .expect("linha");
    assert!(mesa.rect(row.chip).is_some());
    let d = dicas::field_hint(mesa.host.store(), row.chip).expect("marcado");
    assert!(d.contains("(px)"), "{d}");
}

/// `PH2D_CARDS=0`: o corpo de antes, com as barras e sem campos de cartão.
#[test]
fn cards_off_paints_the_old_body() {
    cards::force_cards_on_this_thread(Some(false));
    set_current_brush(Some(ferramenta(WetTool::Paint, true).brush_settings()));
    let mut host = MockPanelHost::with_panel::<WetTuningPanel>();
    host.set_panel_visible(WetTuningPanel::ID, true);
    let mut st = state::WetTuningPanelState;
    let rects = host.paint::<WetTuningPanel>(&mut st, JANELA);
    let row = rows::rows()
        .iter()
        .find(|r| r.options.is_none())
        .expect("linha");
    assert!(rects.iter().any(|(id, _)| *id == row.slider));
    assert!(host.store().card_field(row.chip).is_none());
    assert!(
        !host.store().is_floating_panel(eids::WET_TUNING_PANEL),
        "o corpo de antes publica o rect da janela"
    );
    cards::force_cards_on_this_thread(None);
}
