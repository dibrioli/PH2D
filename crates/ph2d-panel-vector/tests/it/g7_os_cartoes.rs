//! ⭐⭐ **O painel Vector em cartões, fatia G7** (`docs/CartoesUI/12_VECTOR_PLANO.md`): Component, Layout
//! e Widget, na aba Interface. Cada id é provado pelo gesto REAL, nos rects pintados, e o que chega ao
//! barramento é o MESMO do painel de antes.

use std::collections::BTreeMap;

use super::g4_os_cartoes::Mesa;
use super::g5_os_cartoes::sem_acusacao;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::cards::dicas::field_hint;
use ph2d_editor_core::widget::key_mark::{KeyMark, key_mark_id, set_key_marks};
use ph2d_panel_vector::ids as p;
use ph2d_panel_vector::state::{self, ComponentState, LayoutFlow, LayoutItem, WidgetSkinState};

fn fluxo(dir: NodeId) -> LayoutFlow {
    LayoutFlow {
        size: [p::VECTOR_LAYOUT_SIZE_W_FIXED, p::VECTOR_LAYOUT_SIZE_H_HUG],
        min: [1.0, 2.0],
        max: [3.0, 4.0],
        dir,
        gap: [0.5, 0.25],
        pad: [0.125, 0.25, 0.375, 0.625],
        align: p::VECTOR_LAYOUT_ALIGN_CENTER,
        justify: p::VECTOR_LAYOUT_JUSTIFY_END,
        columns: 3.0,
    }
}

fn pele() -> WidgetSkinState {
    WidgetSkinState {
        kinds: ["Button", "Toggle", "Slider"].map(String::from).to_vec(),
        selected: Some(2),
        drives: Some(Some("Knob".to_string())),
        icon: Some(None),
        unknown: false,
    }
}

fn base() {
    ph2d_panel_vector::set_current_text_visible(false);
    ph2d_panel_vector::set_current_selection_count(1);
    state::set_frame_present(true);
    state::set_frame_clip(Some(false));
    state::set_layout_flow(Some(fluxo(p::VECTOR_LAYOUT_DIR_WRAP)));
    state::set_layout_item(Some(LayoutItem {
        absolute: false,
        in_flow: true,
        grow: 2.0,
        shrink: 0.5,
        parent_is_grid: false,
    }));
    state::set_token_bindings(Some(Default::default()));
    state::set_component_state(Some(ComponentState {
        is_instance: true,
        has_overrides: true,
        can_make_variant: true,
        can_edit_prefab: true,
        ..ComponentState::default()
    }));
    state::set_widget_skin_state(Some(pele()), 0);
    set_key_marks(BTreeMap::new());
}

fn mestre() {
    base();
    state::set_component_state(Some(ComponentState {
        is_main: true,
        ..ComponentState::default()
    }));
}

fn na_interface(cartoes: bool, semente: fn()) -> Mesa {
    let mut mesa = Mesa::com(cartoes, semente);
    mesa.clica(p::VECTOR_TAB_INTERFACE);
    mesa.assenta();
    mesa
}

fn cliques() -> Vec<NodeId> {
    vec![
        p::VECTOR_COMPONENT_CREATE,
        p::VECTOR_COMPONENT_EDIT,
        p::VECTOR_COMPONENT_SWAP,
        p::VECTOR_COMPONENT_DETACH,
        p::VECTOR_COMPONENT_UPDATE_MAIN,
        p::VECTOR_COMPONENT_RESET,
        p::VECTOR_LAYOUT_DIR_OFF,
        p::VECTOR_LAYOUT_DIR_ROW,
        p::VECTOR_LAYOUT_DIR_COL,
        p::VECTOR_LAYOUT_DIR_WRAP,
        p::VECTOR_LAYOUT_DIR_GRID,
        p::VECTOR_LAYOUT_SIZE_W_FIXED,
        p::VECTOR_LAYOUT_SIZE_W_HUG,
        p::VECTOR_LAYOUT_SIZE_H_FIXED,
        p::VECTOR_LAYOUT_SIZE_H_HUG,
        p::VECTOR_LAYOUT_PAD_ALL_MODE,
        p::VECTOR_LAYOUT_PAD_EACH_MODE,
        p::VECTOR_LAYOUT_ALIGN_START,
        p::VECTOR_LAYOUT_ALIGN_CENTER,
        p::VECTOR_LAYOUT_ALIGN_END,
        p::VECTOR_LAYOUT_ALIGN_STRETCH,
        p::VECTOR_LAYOUT_JUSTIFY_START,
        p::VECTOR_LAYOUT_JUSTIFY_CENTER,
        p::VECTOR_LAYOUT_JUSTIFY_END,
        p::VECTOR_LAYOUT_JUSTIFY_BETWEEN,
        p::VECTOR_LAYOUT_JUSTIFY_AROUND,
        p::VECTOR_LAYOUT_ITEM_ABSOLUTE,
        p::vector_widget_kind_id(0),
        p::vector_widget_kind_id(1),
        p::vector_widget_kind_id(2),
        p::VECTOR_WIDGET_BIND,
        p::VECTOR_WIDGET_UNBIND,
        p::VECTOR_WIDGET_REMOVE,
    ]
}

/// Os números do Wrap, e o que cada um mostra do documento.
fn numeros() -> [(NodeId, f64); 11] {
    [
        (p::VECTOR_LAYOUT_MIN_W, 1.0),
        (p::VECTOR_LAYOUT_MAX_W, 3.0),
        (p::VECTOR_LAYOUT_MIN_H, 2.0),
        (p::VECTOR_LAYOUT_MAX_H, 4.0),
        (p::VECTOR_LAYOUT_GAP_MAIN, 0.5),
        (p::VECTOR_LAYOUT_GAP_CROSS, 0.25),
        (p::VECTOR_LAYOUT_PAD_ALL, 0.125),
        (p::VECTOR_LAYOUT_ITEM_GROW, 2.0),
        (p::VECTOR_LAYOUT_ITEM_SHRINK, 0.5),
        (p::VECTOR_LAYOUT_GAP_MAIN, 0.5),
        (p::VECTOR_LAYOUT_PAD_ALL, 0.125),
    ]
}

#[test]
fn every_id_of_the_slice_is_painted_in_cards_and_in_the_old_panel() {
    for cartoes in [true, false] {
        let mesa = na_interface(cartoes, base);
        for id in cliques() {
            assert!(mesa.rect(id).is_some(), "{id:?} (cartões {cartoes})");
        }
        for (id, _) in numeros() {
            assert!(mesa.rect(id).is_some(), "{id:?} (cartões {cartoes})");
        }
        assert!(
            mesa.rect(p::VECTOR_WIDGET_ICON_DD).is_some(),
            "ícone (cartões {cartoes})"
        );
        assert!(
            mesa.rect(p::VECTOR_TOKEN_GAP_MAIN).is_some(),
            "token (cartões {cartoes})"
        );
    }
}

/// ⭐⭐ **Mesmo id, mesmo evento**. *Mutação: trocar dois ids num corpo ⇒ vermelho.*
#[test]
fn every_click_sends_the_same_action_as_the_old_panel() {
    for id in cliques() {
        let novo = na_interface(true, base).clica(id);
        assert_eq!(novo, na_interface(false, base).clica(id), "{id:?}");
    }
    for id in [p::VECTOR_COMPONENT_PLACE, p::VECTOR_COMPONENT_PLACE_LINKED] {
        let novo = na_interface(true, mestre).clica(id);
        assert!(!novo.is_empty(), "{id:?} mudo");
        assert_eq!(novo, na_interface(false, mestre).clica(id), "{id:?}");
    }
}

/// ⭐ **Scrub de cada número**, e o Esc devolve. *Mutação: tirar `mark_card_field` ⇒ vermelho.*
#[test]
fn scrubbing_each_number_speaks_and_escape_gives_it_back() {
    for (id, _) in numeros() {
        let mut mesa = na_interface(true, base);
        let antes = mesa.host.store().number_value(id).expect("semeado");
        let ev = mesa.arrasta_a_pega(id, 60.0);
        assert!(
            ev.contains(&WidgetEvent::ValueChanged(id)),
            "{id:?} mudo: {ev:?}"
        );
        let durante = mesa.host.store().number_value(id).expect("campo");
        assert!((durante - antes).abs() > 1e-6, "{id:?} parado em {antes}");
        mesa.esc();
        let voltou = mesa.host.store().number_value(id).expect("campo");
        assert!((voltou - antes).abs() < 1e-6, "{id:?}: o Esc deu {voltou}");
    }
}

/// O campo mostra o documento: as sementes de sempre chegam aos cartões. *Mutação: tirar `seed` ⇒ vermelho.*
#[test]
fn each_field_shows_the_document_value() {
    let mesa = na_interface(true, base);
    for (id, valor) in numeros() {
        let tem = mesa.host.store().number_value(id).expect("campo");
        assert!((tem - valor).abs() < 1e-6, "{id:?}: {tem}, era {valor}");
    }
}

/// Escrever dá o mesmo que no painel de antes.
#[test]
fn typing_sends_the_same_action_as_the_old_panel() {
    for (id, texto) in [
        (p::VECTOR_LAYOUT_GAP_MAIN, "1.5"),
        (p::VECTOR_LAYOUT_PAD_ALL, "2"),
        (p::VECTOR_LAYOUT_MIN_W, "3"),
        (p::VECTOR_LAYOUT_ITEM_GROW, "4"),
    ] {
        let grava = |cartoes: bool| {
            let mut mesa = na_interface(cartoes, base);
            let ev = mesa.host.type_into_number(id, texto);
            mesa.aplica(ev)
        };
        let novo = grava(true);
        assert!(!novo.is_empty(), "{id:?} mudo");
        assert_eq!(novo, grava(false), "{id:?}");
    }
}

/// ⭐ As perguntas de antes continuam a mandar. *Mutação: tirar cada `if` ⇒ vermelho.*
#[test]
fn what_cannot_be_used_is_not_offered() {
    // Componente: uma forma comum só promove, e sem diferenças não há Apply nem Revert.
    let mut mesa = na_interface(true, base);
    state::set_component_state(Some(ComponentState::default()));
    mesa.assenta();
    assert!(mesa.rect(p::VECTOR_COMPONENT_CREATE).is_some());
    for id in [p::VECTOR_COMPONENT_PLACE, p::VECTOR_COMPONENT_DETACH] {
        assert!(mesa.rect(id).is_none(), "{id:?} numa forma comum");
    }
    state::set_component_state(Some(ComponentState {
        is_instance: true,
        ..ComponentState::default()
    }));
    mesa.assenta();
    assert!(mesa.rect(p::VECTOR_COMPONENT_DETACH).is_some());
    assert!(
        mesa.rect(p::VECTOR_COMPONENT_RESET).is_none(),
        "sem diferenças"
    );
    assert!(mesa.rect(p::VECTOR_COMPONENT_EDIT).is_none(), "sem receita");
    state::set_component_state(None);
    mesa.assenta();
    assert!(
        mesa.rect(p::VECTOR_SECTION_COMPONENT).is_none(),
        "sem assunto"
    );

    // Layout: parado só a direção; Cross só no Wrap; Cols só na grade, e sem repartir.
    let mut mesa = na_interface(true, base);
    state::set_layout_flow(None);
    mesa.assenta();
    assert!(mesa.rect(p::VECTOR_LAYOUT_DIR_OFF).is_some());
    assert!(mesa.rect(p::VECTOR_LAYOUT_GAP_MAIN).is_none(), "Gap parado");
    state::set_layout_flow(Some(fluxo(p::VECTOR_LAYOUT_DIR_ROW)));
    mesa.assenta();
    assert!(
        mesa.rect(p::VECTOR_LAYOUT_GAP_CROSS).is_none(),
        "Cross em linha"
    );
    assert!(
        mesa.rect(p::VECTOR_TOKEN_GAP_CROSS).is_none(),
        "token em linha"
    );
    assert!(
        mesa.rect(p::VECTOR_LAYOUT_COLUMNS).is_none(),
        "Cols em linha"
    );
    state::set_layout_flow(Some(fluxo(p::VECTOR_LAYOUT_DIR_GRID)));
    mesa.assenta();
    assert!(
        mesa.rect(p::VECTOR_LAYOUT_COLUMNS).is_some(),
        "Cols na grade"
    );
    assert!(
        mesa.rect(p::VECTOR_LAYOUT_JUSTIFY_BETWEEN).is_none(),
        "grade"
    );
    assert!(mesa.rect(p::VECTOR_LAYOUT_JUSTIFY_END).is_some());

    // O filho: absoluto ou na grade não reparte; sem fluxo no pai, nenhum controlo.
    let item = |absolute, in_flow, parent_is_grid| {
        state::set_layout_item(Some(LayoutItem {
            absolute,
            in_flow,
            grow: 0.0,
            shrink: 1.0,
            parent_is_grid,
        }));
    };
    item(true, true, false);
    mesa.assenta();
    assert!(mesa.rect(p::VECTOR_LAYOUT_ITEM_GROW).is_none(), "absoluto");
    item(false, true, true);
    mesa.assenta();
    assert!(
        mesa.rect(p::VECTOR_LAYOUT_ITEM_GROW).is_none(),
        "pai em grade"
    );
    item(false, false, false);
    mesa.assenta();
    assert!(
        mesa.rect(p::VECTOR_LAYOUT_ITEM_ABSOLUTE).is_none(),
        "sem fluxo"
    );

    // Widget: sem pele, só vestir; o tipo sem face nem valor não oferece ícone nem Bind.
    let mut mesa = na_interface(true, base);
    state::set_widget_skin_state(
        Some(WidgetSkinState {
            drives: Some(None),
            ..pele()
        }),
        0,
    );
    mesa.assenta();
    assert!(mesa.rect(p::VECTOR_WIDGET_BIND).is_some());
    assert!(
        mesa.rect(p::VECTOR_WIDGET_UNBIND).is_none(),
        "nada a soltar"
    );
    state::set_widget_skin_state(
        Some(WidgetSkinState {
            drives: None,
            icon: None,
            ..pele()
        }),
        0,
    );
    mesa.assenta();
    assert!(mesa.rect(p::VECTOR_WIDGET_BIND).is_none(), "não dirige");
    assert!(mesa.rect(p::VECTOR_WIDGET_ICON_DD).is_none(), "sem face");
    state::set_widget_skin_state(
        Some(WidgetSkinState {
            selected: None,
            ..pele()
        }),
        0,
    );
    mesa.assenta();
    assert!(mesa.rect(p::VECTOR_WIDGET_WEAR).is_some());
    assert!(mesa.rect(p::VECTOR_WIDGET_REMOVE).is_none(), "nada vestido");
    state::set_widget_skin_state(
        Some(WidgetSkinState {
            selected: None,
            unknown: true,
            ..pele()
        }),
        0,
    );
    mesa.assenta();
    assert!(
        mesa.rect(p::VECTOR_WIDGET_REMOVE).is_some(),
        "tipo do futuro"
    );
    assert!(mesa.rect(p::VECTOR_WIDGET_WEAR).is_none(), "tipo do futuro");
}

/// All troca os campos: Each dá os quatro lados aos pares. O modo é local e não vai ao barramento.
/// *Mutação: tirar o ramo `each` ⇒ vermelho.*
#[test]
fn padding_each_gives_four_sides_two_by_two() {
    let mut mesa = na_interface(true, base);
    assert!(
        mesa.rect(p::VECTOR_LAYOUT_PAD_T).is_none(),
        "All tem um campo"
    );
    mesa.clica(p::VECTOR_LAYOUT_PAD_EACH_MODE);
    mesa.assenta();
    let r = |id: NodeId| mesa.rect(id).unwrap_or_else(|| panic!("{id:?}"));
    assert!(mesa.rect(p::VECTOR_LAYOUT_PAD_ALL).is_none(), "All em Each");
    assert!((r(p::VECTOR_LAYOUT_PAD_T).y - r(p::VECTOR_LAYOUT_PAD_R).y).abs() < 1.0);
    assert!((r(p::VECTOR_LAYOUT_PAD_B).y - r(p::VECTOR_LAYOUT_PAD_L).y).abs() < 1.0);
    assert!(r(p::VECTOR_LAYOUT_PAD_B).y > r(p::VECTOR_LAYOUT_PAD_T).y);
    for (id, valor) in [
        (p::VECTOR_LAYOUT_PAD_T, 0.125),
        (p::VECTOR_LAYOUT_PAD_R, 0.25),
        (p::VECTOR_LAYOUT_PAD_B, 0.375),
        (p::VECTOR_LAYOUT_PAD_L, 0.625),
    ] {
        let tem = mesa.host.store().number_value(id).expect("campo");
        assert!((tem - valor).abs() < 1e-6, "{id:?}: {tem}");
    }
    let tem = field_hint(mesa.host.store(), p::VECTOR_LAYOUT_PAD_L).expect("dica");
    assert!(tem.starts_with("Left (m)"), "«{tem}»");
    mesa.clica(p::VECTOR_LAYOUT_PAD_ALL_MODE);
    mesa.assenta();
}

/// Cada caixa de token abre a lista dela, e o token presa risca o número que governa.
/// *Mutação: tirar `popover_de_token` ou a rachura ⇒ vermelho.*
#[test]
fn a_bound_token_slashes_its_number_and_its_box_opens() {
    let pinta = |preso: Option<&str>| {
        let mut mesa = na_interface(true, base);
        let b = state::TokenBindings {
            gap_main: preso.map(String::from),
            ..state::TokenBindings::default()
        };
        state::set_token_bindings(Some(b));
        mesa.assenta();
        let (h, st) = (&mut mesa.host, &mut mesa.st);
        h.paint_draw_data::<ph2d_panel_vector::VectorPanel>(st, super::g4_os_cartoes::COLUNA)
    };
    assert_ne!(pinta(None), pinta(Some("accent")));
    let mut mesa = na_interface(true, base);
    let fechada = mesa.rects.len();
    mesa.clica(p::VECTOR_TOKEN_GAP_MAIN);
    mesa.assenta();
    assert_eq!(
        mesa.host.dropdown_is_open(p::VECTOR_TOKEN_GAP_MAIN),
        Some(true)
    );
    assert!(mesa.rects.len() > fechada, "a lista do token não se pintou");
}

/// A caixa do ícone abre a lista dela, e deixa o recado para o passe diferido.
#[test]
fn the_icon_box_opens_its_list() {
    let mut mesa = na_interface(true, base);
    let fechada = mesa.rects.len();
    mesa.clica(p::VECTOR_WIDGET_ICON_DD);
    mesa.assenta();
    assert_eq!(
        mesa.host.dropdown_is_open(p::VECTOR_WIDGET_ICON_DD),
        Some(true)
    );
    assert!(
        mesa.rects.len() > fechada,
        "a lista de ícones não se pintou"
    );
}

/// A opção acesa sai do documento. *Mutação: acender sempre a mesma ⇒ vermelho.*
#[test]
fn the_lit_option_follows_the_document() {
    let pinta = |semente: &dyn Fn()| {
        let mut mesa = na_interface(true, base);
        semente();
        mesa.assenta();
        let (h, st) = (&mut mesa.host, &mut mesa.st);
        h.paint_draw_data::<ph2d_panel_vector::VectorPanel>(st, super::g4_os_cartoes::COLUNA)
    };
    let fluxo_com = |f: fn(&mut LayoutFlow)| {
        let mut v = fluxo(p::VECTOR_LAYOUT_DIR_WRAP);
        f(&mut v);
        state::set_layout_flow(Some(v));
    };
    let a = pinta(&|| fluxo_com(|_| {}));
    assert_ne!(
        a,
        pinta(&|| fluxo_com(|f| f.size[0] = p::VECTOR_LAYOUT_SIZE_W_HUG)),
        "Width"
    );
    assert_ne!(
        a,
        pinta(&|| fluxo_com(|f| f.size[1] = p::VECTOR_LAYOUT_SIZE_H_FIXED)),
        "Height"
    );
    assert_ne!(
        a,
        pinta(&|| fluxo_com(|f| f.align = p::VECTOR_LAYOUT_ALIGN_END)),
        "Align"
    );
    assert_ne!(
        a,
        pinta(&|| fluxo_com(|f| f.justify = p::VECTOR_LAYOUT_JUSTIFY_AROUND)),
        "Distribute"
    );
    assert_ne!(
        a,
        pinta(&|| state::set_widget_skin_state(
            Some(WidgetSkinState {
                selected: Some(1),
                ..pele()
            }),
            0
        )),
        "Widget"
    );
}

/// Sempre UMA opção acesa: o tamanho do que se pinta não muda com a escolha. Sem isto, uma opção acesa
/// sempre, ou nunca, continuava a dar pinturas diferentes. *Mutação: `aceso` fixo ⇒ vermelho.*
#[test]
fn exactly_one_option_is_lit() {
    let n = |semente: &dyn Fn()| {
        let mut mesa = na_interface(true, base);
        semente();
        mesa.assenta();
        let (h, st) = (&mut mesa.host, &mut mesa.st);
        h.paint_draw_data::<ph2d_panel_vector::VectorPanel>(st, super::g4_os_cartoes::COLUNA)
            .len()
    };
    let com = |f: fn(&mut LayoutFlow)| {
        let mut v = fluxo(p::VECTOR_LAYOUT_DIR_WRAP);
        f(&mut v);
        state::set_layout_flow(Some(v));
    };
    let base_n = n(&|| com(|_| {}));
    assert_eq!(
        base_n,
        n(&|| com(|f| f.align = p::VECTOR_LAYOUT_ALIGN_END)),
        "Align"
    );
    assert_eq!(
        base_n,
        n(&|| com(|f| f.justify = p::VECTOR_LAYOUT_JUSTIFY_AROUND)),
        "Distribute"
    );
    assert_eq!(
        base_n,
        n(&|| com(|f| f.size[0] = p::VECTOR_LAYOUT_SIZE_W_HUG)),
        "Width"
    );
    assert_eq!(
        base_n,
        n(&|| state::set_widget_skin_state(
            Some(WidgetSkinState {
                selected: Some(1),
                ..pele()
            }),
            0
        )),
        "Type"
    );
    // Direction: Off acende sem fluxo, mas sem fluxo o resto some. Compara-se Row com Column.
    let direcao = |dir: NodeId| state::set_layout_flow(Some(fluxo(dir)));
    assert_eq!(
        n(&|| direcao(p::VECTOR_LAYOUT_DIR_ROW)),
        n(&|| direcao(p::VECTOR_LAYOUT_DIR_COL)),
        "Direction"
    );
}

/// O que só fala às vezes: órfã, excedente de tipos, Swap armado, moldura ausente. Cada um é um `if`.
#[test]
fn what_speaks_only_sometimes_speaks_when_it_has_a_subject() {
    let pinta = |semente: &dyn Fn()| {
        let mut mesa = na_interface(true, base);
        semente();
        mesa.assenta();
        let (h, st) = (&mut mesa.host, &mut mesa.st);
        h.paint_draw_data::<ph2d_panel_vector::VectorPanel>(st, super::g4_os_cartoes::COLUNA)
    };
    let copia = |main_missing: bool| {
        state::set_component_state(Some(ComponentState {
            is_instance: true,
            main_missing,
            ..ComponentState::default()
        }));
    };
    assert_ne!(pinta(&|| copia(false)), pinta(&|| copia(true)), "órfã");
    assert_ne!(
        pinta(&|| state::set_widget_skin_state(Some(pele()), 0)),
        pinta(&|| state::set_widget_skin_state(Some(pele()), 3)),
        "excedente"
    );

    // Armado, o botão diz o que o próximo clique faz, sem cortar: desce de linha.
    let mut mesa = na_interface(true, base);
    state::set_component_state(Some(ComponentState {
        is_instance: true,
        swap_armed: true,
        ..ComponentState::default()
    }));
    let ((), c) = ph2d_editor_core::panel::cards::censo::grava(|| mesa.assenta());
    let nome = ph2d_i18n::tr("panel.vector.component.swap_armed");
    assert!(c.textos.iter().any(|t| t.texto == nome), "«{nome}»");
    assert_eq!(c.textos_cortados(), Vec::<String>::new());

    // Um mestre não troca nem solta.
    let mesa = na_interface(true, mestre);
    assert!(
        mesa.rect(p::VECTOR_COMPONENT_SWAP).is_none(),
        "Swap no mestre"
    );
    assert!(
        mesa.rect(p::VECTOR_COMPONENT_PLACE).is_some(),
        "Instantiate"
    );

    // Filho de fluxo sem moldura: o Layout fala só do filho.
    let mut mesa = na_interface(true, base);
    state::set_frame_clip(None);
    mesa.assenta();
    assert!(mesa.rect(p::VECTOR_LAYOUT_DIR_OFF).is_none(), "sem moldura");
    assert!(
        mesa.rect(p::VECTOR_LAYOUT_ITEM_GROW).is_some(),
        "o filho fica"
    );
}

/// O canal fica no campo. *Mutação: tirar um `.key` ⇒ vermelho.*
#[test]
fn each_channel_keeps_its_animation_mark() {
    let marcados = [
        p::VECTOR_LAYOUT_DIR_OFF,
        p::VECTOR_LAYOUT_SIZE_W_FIXED,
        p::VECTOR_LAYOUT_SIZE_H_FIXED,
        p::VECTOR_LAYOUT_MIN_W,
        p::VECTOR_LAYOUT_MAX_W,
        p::VECTOR_LAYOUT_MIN_H,
        p::VECTOR_LAYOUT_MAX_H,
        p::VECTOR_LAYOUT_GAP_MAIN,
        p::VECTOR_LAYOUT_GAP_CROSS,
        p::VECTOR_LAYOUT_PAD_ALL,
        p::VECTOR_LAYOUT_ALIGN_START,
        p::VECTOR_LAYOUT_JUSTIFY_START,
        p::VECTOR_LAYOUT_ITEM_GROW,
        p::VECTOR_LAYOUT_ITEM_SHRINK,
        p::VECTOR_LAYOUT_ITEM_ABSOLUTE,
        p::VECTOR_WIDGET_ICON_DD,
    ];
    let mut mesa = na_interface(true, base);
    set_key_marks(marcados.iter().map(|id| (*id, KeyMark::Chave)).collect());
    mesa.assenta();
    for id in marcados {
        assert!(mesa.rect(key_mark_id(id)).is_some(), "sem marca: {id:?}");
    }
    set_key_marks(BTreeMap::new());
}

/// ⭐⭐ **Compacto como o Figma** (05 §4). *Mutação: separar um par ⇒ vermelho.*
#[test]
fn siblings_share_the_line() {
    let mesa = na_interface(true, base);
    let r = |id: NodeId| mesa.rect(id).unwrap_or_else(|| panic!("{id:?}"));
    for (a, b) in [
        (p::VECTOR_COMPONENT_CREATE, p::VECTOR_COMPONENT_EDIT),
        (p::VECTOR_COMPONENT_SWAP, p::VECTOR_COMPONENT_DETACH),
        (p::VECTOR_COMPONENT_UPDATE_MAIN, p::VECTOR_COMPONENT_RESET),
        (p::VECTOR_LAYOUT_SIZE_W_FIXED, p::VECTOR_LAYOUT_SIZE_H_FIXED),
        (p::VECTOR_LAYOUT_MIN_W, p::VECTOR_LAYOUT_MAX_W),
        (p::VECTOR_LAYOUT_MIN_H, p::VECTOR_LAYOUT_MAX_H),
        (p::VECTOR_LAYOUT_GAP_MAIN, p::VECTOR_LAYOUT_GAP_CROSS),
        (p::VECTOR_TOKEN_GAP_MAIN, p::VECTOR_TOKEN_GAP_CROSS),
        (p::VECTOR_LAYOUT_PAD_ALL_MODE, p::VECTOR_LAYOUT_PAD_ALL),
        (p::VECTOR_LAYOUT_ITEM_GROW, p::VECTOR_LAYOUT_ITEM_SHRINK),
        (p::VECTOR_WIDGET_BIND, p::VECTOR_WIDGET_UNBIND),
    ] {
        let (ra, rb) = (r(a), r(b));
        // O segmento tem a folga da pílula.
        assert!((ra.y - rb.y).abs() <= 2.0, "{a:?} e {b:?}: {ra:?} {rb:?}");
        assert!(ra.x + ra.w <= rb.x, "{a:?} por cima de {b:?}");
    }
}

/// Distância de mundo diz o metro; razão não tem unidade. *Mutação: tirar `com_unidade` ⇒ vermelho.*
#[test]
fn each_number_says_its_unit_in_the_label() {
    let mesa = na_interface(true, base);
    let tem = |id: NodeId, nome: &str| {
        let dica = field_hint(mesa.host.store(), id).unwrap_or_else(|| panic!("{id:?} sem dica"));
        assert!(dica.starts_with(nome), "{id:?}: «{dica}» sem «{nome}»");
    };
    tem(p::VECTOR_LAYOUT_GAP_MAIN, "Gap (m)");
    tem(p::VECTOR_LAYOUT_GAP_CROSS, "Cross (m)");
    tem(p::VECTOR_LAYOUT_MIN_W, "Min W (m)");
    tem(p::VECTOR_LAYOUT_MAX_H, "Max H (m)");
    tem(p::VECTOR_LAYOUT_PAD_ALL, "All (m)");
    tem(p::VECTOR_LAYOUT_ITEM_GROW, "Grow");
}

/// ⭐⭐ **A lei da compactação sobre os estados que a varredura do registo não semeia.**
#[test]
fn the_slice_passes_the_compaction_census() {
    for (nome, semente) in [("instância", base as fn()), ("mestre", mestre)] {
        let mut mesa = na_interface(true, semente);
        sem_acusacao(nome, || mesa.assenta());
    }
    for dir in [
        p::VECTOR_LAYOUT_DIR_ROW,
        p::VECTOR_LAYOUT_DIR_COL,
        p::VECTOR_LAYOUT_DIR_GRID,
    ] {
        let mut mesa = na_interface(true, base);
        state::set_layout_flow(Some(fluxo(dir)));
        sem_acusacao("direção", || mesa.assenta());
    }
    let mut mesa = na_interface(true, base);
    mesa.clica(p::VECTOR_LAYOUT_PAD_EACH_MODE);
    sem_acusacao("each", || mesa.assenta());
    mesa.clica(p::VECTOR_LAYOUT_PAD_ALL_MODE);
    mesa.assenta();
}

/// `PH2D_CARDS=0` continua no corpo de antes: T, R, B, L sem unidade, e o mesmo Direction.
#[test]
fn cards_off_paints_the_old_body() {
    let mesa = na_interface(false, base);
    assert!(mesa.rect(p::VECTOR_LAYOUT_DIR_ROW).is_some());
    assert!(field_hint(mesa.host.store(), p::VECTOR_LAYOUT_GAP_MAIN).is_none());
}
