//! ⭐⭐ **O painel Design Tokens em cartões: nenhum controlo morreu.** O inventário vem das tabelas
//! (`ColorToken::ALL`, `NumToken::ALL`); cada id é provado pelo gesto REAL nos rects pintados. As
//! camadas de override são globais por thread: cada teste limpa as suas no início.

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::cards::dicas;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_tokens::state::{TokensIntent, TokensPanelState, drain_intents};
use ph2d_panel_tokens::{TokensPanel, cards, ids, ids_das_familias};
use ph2d_tokens::num_overrides::{NumValue, clear_num_overrides, set_num_override};
use ph2d_tokens::overrides::{TokenValue, clear_color_overrides, set_color_override};
use ph2d_tokens::{ColorToken, NumToken, Theme};
use ph2d_ui_testkit::MockPanelHost;

const COLUNA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 340.0,
    h: 16000.0,
};

struct Mesa {
    host: MockPanelHost,
    st: TokensPanelState,
    rects: Vec<(NodeId, Rect)>,
}

impl Mesa {
    /// As famílias abertas: o inventário quer vê-las.
    fn nova() -> Self {
        cards::force_cards_on_this_thread(Some(true));
        let _ = drain_intents();
        let mut host = MockPanelHost::with_panel::<TokensPanel>();
        host.set_panel_visible(TokensPanel::ID, true);
        for f in ids_das_familias().chain([ids::TOKENS_SEC_SCALE]) {
            host.set_collapsed(f, false);
        }
        let mut mesa = Self {
            host,
            st: TokensPanelState::default(),
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
                || h.paint::<TokensPanel>(st, COLUNA),
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

    fn aplica(&mut self, eventos: Vec<WidgetEvent>) -> Vec<TokensIntent> {
        for e in eventos {
            self.host.apply_panel_event::<TokensPanel>(&mut self.st, e);
        }
        drain_intents()
    }

    fn clica(&mut self, id: NodeId) -> Vec<TokensIntent> {
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
}

fn limpa() {
    clear_color_overrides();
    clear_num_overrides();
}

/// O tema em que o painel pinta (o das camadas que os testes autoram).
fn tema() -> Theme {
    Theme::default()
}

/// ⭐ Todo id do inventário pintado num estado alcançável: as caixas e o elo sempre, o repor quando a
/// linha está autorada, a fórmula quando o `f(x)` a abre.
#[test]
fn every_id_of_the_old_panel_is_painted_in_some_reachable_state() {
    limpa();
    let mut mesa = Mesa::nova();
    let mut pintados: std::collections::BTreeSet<NodeId> =
        mesa.rects.iter().map(|(id, _)| *id).collect();
    for row in 0..ColorToken::ALL.len() {
        for id in [ids::tokens_swatch_id(row), ids::tokens_link_id(row)] {
            assert!(pintados.contains(&id), "cor {row}: {id:?}");
        }
    }
    for row in 0..NumToken::ALL.len() {
        for id in [ids::tokens_num_chip_id(row), ids::tokens_num_link_id(row)] {
            assert!(pintados.contains(&id), "escala {row}: {id:?}");
        }
    }
    for id in [
        ids::TOKENS_CLOSE,
        ids::TOKENS_DTCG_EXPORT,
        ids::TOKENS_DTCG_IMPORT,
    ] {
        assert!(pintados.contains(&id), "{id:?}");
    }
    assert!(
        !pintados.contains(&ids::TOKENS_RESET_ALL),
        "num modo de fábrica o repor geral é um clique que não faz nada"
    );
    let theme = tema();
    for &t in ColorToken::ALL.iter() {
        set_color_override(theme, t, Some(TokenValue::Literal(t.resolve(theme)))).unwrap();
    }
    for &t in NumToken::ALL.iter() {
        set_num_override(theme, t, Some(NumValue::Literal(t.px(theme)))).unwrap();
    }
    mesa.assenta();
    pintados.extend(mesa.rects.iter().map(|(id, _)| *id));
    for row in 0..ColorToken::ALL.len() {
        assert!(pintados.contains(&ids::tokens_reset_id(row)), "repor {row}");
    }
    for row in 0..NumToken::ALL.len() {
        assert!(
            pintados.contains(&ids::tokens_num_reset_id(row)),
            "repor {row}"
        );
    }
    assert!(pintados.contains(&ids::TOKENS_RESET_ALL));
    limpa();
}

/// O clique na amostra abre o selector NESSE token (o anel marca-a); nunca passa pela fila.
#[test]
fn clicking_a_swatch_opens_the_picker_on_that_token() {
    limpa();
    let mut mesa = Mesa::nova();
    let id = ids::tokens_swatch_id(0);
    let i = mesa.clica(id);
    assert!(i.is_empty(), "a amostra não enfileira: {i:?}");
    assert_eq!(mesa.host.store().picker_target(), Some(id));
}

/// A máquina do elo: armar, apontar; o elo fica aceso enquanto armado.
#[test]
fn the_link_gesture_arms_then_closes_on_the_clicked_row() {
    limpa();
    let mut mesa = Mesa::nova();
    let (a, b) = (0, 1);
    assert!(
        mesa.clica(ids::tokens_link_id(a)).is_empty(),
        "armar não escreve"
    );
    assert_eq!(mesa.st.armed(), Some(a));
    mesa.assenta();
    let i = mesa.clica(ids::tokens_link_id(b));
    assert!(
        matches!(i[..], [TokensIntent::Link { from, to }] if from == a && to == b),
        "{i:?}"
    );
    assert_eq!(mesa.st.armed(), None);
    let i = mesa.clica(ids::tokens_num_link_id(0));
    assert!(i.is_empty());
    assert_eq!(mesa.st.armed_num(), Some(0));
    let i = mesa.clica(ids::tokens_num_link_id(1));
    assert!(
        matches!(i[..], [TokensIntent::NumLink { from: 0, to: 1 }]),
        "{i:?}"
    );
}

/// O repor de uma linha autorada e o geral; os dois do DTCG; o «×» fecha.
#[test]
fn every_verb_answers_a_real_click() {
    limpa();
    let theme = tema();
    set_color_override(
        theme,
        ColorToken::ALL[2],
        Some(TokenValue::Literal(ColorToken::ALL[2].resolve(theme))),
    )
    .unwrap();
    set_num_override(theme, NumToken::ALL[3], Some(NumValue::Literal(7.0))).unwrap();
    let mut mesa = Mesa::nova();
    let i = mesa.clica(ids::tokens_reset_id(2));
    assert!(matches!(i[..], [TokensIntent::Reset(2)]), "{i:?}");
    let i = mesa.clica(ids::tokens_num_reset_id(3));
    assert!(matches!(i[..], [TokensIntent::NumReset(3)]), "{i:?}");
    let i = mesa.clica(ids::TOKENS_RESET_ALL);
    assert!(matches!(i[..], [TokensIntent::ResetAll]), "{i:?}");
    let i = mesa.clica(ids::TOKENS_DTCG_EXPORT);
    assert!(matches!(i[..], [TokensIntent::ExportDtcg]), "{i:?}");
    let i = mesa.clica(ids::TOKENS_DTCG_IMPORT);
    assert!(matches!(i[..], [TokensIntent::ImportDtcg]), "{i:?}");
    let _ = mesa.clica(ids::TOKENS_CLOSE);
    assert!(!mesa.host.panel_visible(TokensPanel::ID), "o «×» fecha");
    limpa();
}

/// O número: o scrub real na pega e o escrito à mão, cada um pede `NumSet` da SUA linha; o Esc devolve.
#[test]
fn scrubbing_and_typing_a_scale_value_ask_for_that_row() {
    use ph2d_editor_core::interaction::dispatch_key;
    limpa();
    for row in [0, NumToken::ALL.len() - 1] {
        let mut mesa = Mesa::nova();
        let chip = ids::tokens_num_chip_id(row);
        let antes = NumToken::ALL[row].px(tema());
        let r = mesa.rect(chip).expect("campo");
        let (x, y) = (r.x + m::ICON_CELL * 0.5, r.y + r.h * 0.5);
        let mut ev = mesa.ponteiro(PointerKind::Down, x, y);
        ev.extend(mesa.ponteiro(PointerKind::Move, x + 30.0, y));
        ev.extend(mesa.ponteiro(PointerKind::Move, x + 60.0, y));
        let i = mesa.aplica(ev);
        let px = i.iter().rev().find_map(|t| match t {
            TokensIntent::NumSet { row: r, px } if *r == row => Some(*px),
            _ => None,
        });
        let px = px.unwrap_or_else(|| panic!("linha {row}: {i:?}"));
        assert!((px - antes).abs() > 1e-3, "linha {row} não se moveu");
        let arena = bumpalo::Bump::new();
        let esc = ph2d_host::KeyEvent {
            keycode: 0x1B,
            modifiers: ph2d_host::Modifiers::default(),
            kind: ph2d_host::KeyKind::Down,
            timestamp_ns: 0,
        };
        let ev = dispatch_key(mesa.host.store_mut(), esc, &arena).to_vec();
        let i = mesa.aplica(ev);
        assert!(
            matches!(i.last(), Some(TokensIntent::NumSet { px, .. }) if (*px - antes).abs() < 1e-3),
            "o Esc: {i:?}"
        );
        let _ = mesa.ponteiro(PointerKind::Up, x + 60.0, y);
        mesa.assenta();
        let ev = mesa.host.type_into_number(chip, "12");
        let i = mesa.aplica(ev);
        assert!(
            matches!(i.last(), Some(TokensIntent::NumSet { row: r, px }) if *r == row && (*px - 12.0).abs() < 1e-3),
            "escrever: {i:?}"
        );
    }
}

/// Dobrar, reordenar pelo cabeçalho e o botão direito no título.
#[test]
fn headers_fold_reorder_and_open_the_theme_menu() {
    use ph2d_editor_core::interaction::ContextMenuKind;
    limpa();
    let mut mesa = Mesa::nova();
    let (a, b) = (ids::TOKENS_SEC_MODE, ids::TOKENS_SEC_APP);
    let r = mesa.rect(a).expect("Mode");
    assert!(mesa.rect(ids::TOKENS_DTCG_EXPORT).is_some());
    let _ = mesa
        .host
        .click_at(r.x + r.w * 0.3, r.y + m::CARD_HEAD_H * 0.5);
    assert!(mesa.host.store().is_collapsed(a));
    mesa.assenta();
    assert!(
        mesa.rect(ids::TOKENS_DTCG_EXPORT).is_none(),
        "dobrado não regista clique"
    );
    let _ = mesa
        .host
        .click_at(r.x + r.w * 0.3, r.y + m::CARD_HEAD_H * 0.5);
    mesa.assenta();

    mesa.host.set_collapsed(b, true);
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
    assert!(pos(a) > pos(b), "não desceu: {ordem:?}");

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

/// Todo ícone mudo tem dica, e as famílias nascem dobradas como antes.
#[test]
fn every_mute_control_has_a_tooltip_and_data_families_are_born_folded() {
    limpa();
    let mesa = Mesa::nova();
    let store = mesa.host.store();
    let panel = ph2d_editor_core::ids::TOKENS_PANEL;
    let mudos: Vec<NodeId> = dicas::icon_only_ids(panel)
        .into_iter()
        .chain([
            ids::TOKENS_CLOSE,
            ph2d_editor_core::panel::cards::list_menu_of(panel),
        ])
        .filter(|id| store.tooltip_for(*id).is_none())
        .collect();
    assert!(mudos.is_empty(), "sem dica: {mudos:?}");

    cards::force_cards_on_this_thread(Some(true));
    let mut host = MockPanelHost::with_panel::<TokensPanel>();
    host.set_panel_visible(TokensPanel::ID, true);
    let mut st = TokensPanelState::default();
    let _ = host.paint::<TokensPanel>(&mut st, COLUNA);
    for f in ids_das_familias().chain([ids::TOKENS_SEC_SCALE]) {
        assert!(host.store().is_collapsed(f), "{f:?} nasce dobrada");
    }
}

/// A amostra mostra a cor do TEMA DO APP: pintar um token de outra cor muda o que ela mostra.
#[test]
fn the_swatch_follows_the_authored_colour() {
    limpa();
    let theme = tema();
    let token = ColorToken::ALL[0];
    let outra = ph2d_tokens::Color {
        r: 1,
        g: 2,
        b: 3,
        a: 255,
    };
    set_color_override(theme, token, Some(TokenValue::Literal(outra))).unwrap();
    let mesa = Mesa::nova();
    assert!(
        mesa.rect(ids::tokens_reset_id(0)).is_some(),
        "autorada, tem repor"
    );
    assert_eq!(token.resolve(theme), outra);
    limpa();
}

/// A fórmula: o `f(x)` abre o campo onde estava o número; o Enter fecha e pede a fórmula.
#[test]
fn the_formula_button_swaps_the_field_for_a_text_box() {
    limpa();
    ph2d_tokens::num_expr::install_math(ph2d_tokens::num_expr::MathHost {
        deps: |_| Ok(Vec::new()),
        eval: |_, _| Ok(1.0),
    });
    let mut mesa = Mesa::nova();
    let row = 0;
    let fx = ids::tokens_num_fx_id(row);
    let campo = ids::tokens_num_formula_id(row);
    assert!(mesa.rect(campo).is_none());
    let i = mesa.clica(fx);
    assert!(i.is_empty(), "abrir não escreve");
    mesa.assenta();
    assert!(mesa.rect(campo).is_some(), "o campo abriu");
    assert!(
        mesa.rect(ids::tokens_num_chip_id(row)).is_none(),
        "um slot, um editor: o número vira leitura"
    );
    let i = mesa.aplica(vec![WidgetEvent::Submit(campo)]);
    assert!(
        matches!(i[..], [TokensIntent::NumFormula { row: 0, .. }]),
        "{i:?}"
    );
    assert_eq!(mesa.st.fx_open(), None);
}

/// Quem segue outro DIZ quem, por inteiro, na linha de baixo: a seguinte desce a altura dessa frase.
/// *Contraprova: tirar o `c.hint` de `linha_de_escala` ⇒ a linha seguinte fica onde estava.*
#[test]
fn a_scale_row_that_follows_another_says_which_in_full() {
    limpa();
    let seguinte = ids::tokens_num_chip_id(1);
    let antes = Mesa::nova().rect(seguinte).expect("linha").y;
    let (a, b) = (NumToken::ALL[0], NumToken::ALL[2]);
    set_num_override(tema(), a, Some(NumValue::Alias(b))).unwrap();
    let depois = Mesa::nova().rect(seguinte).expect("linha").y;
    assert!(depois > antes + m::LABEL_PX * 0.5, "{antes} → {depois}");
    limpa();
}

/// `PH2D_CARDS=0`: o corpo de antes, sem campos de cartão.
#[test]
fn cards_off_paints_the_old_body() {
    limpa();
    cards::force_cards_on_this_thread(Some(false));
    let mut host = MockPanelHost::with_panel::<TokensPanel>();
    host.set_panel_visible(TokensPanel::ID, true);
    let mut st = TokensPanelState::default();
    let rects = host.paint::<TokensPanel>(&mut st, COLUNA);
    assert!(rects.iter().any(|(id, _)| *id == ids::tokens_swatch_id(0)));
    assert!(
        host.store()
            .card_field(ids::tokens_num_chip_id(0))
            .is_none()
    );
    assert!(
        !host
            .store()
            .is_floating_panel(ph2d_editor_core::ids::TOKENS_PANEL)
    );
}
