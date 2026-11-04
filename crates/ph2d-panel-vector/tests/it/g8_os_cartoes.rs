//! ⭐⭐ **O painel Vector em cartões, fatia G8** (`docs/CartoesUI/12_VECTOR_PLANO.md`): States e Morph
//! States, e o fim do corpo antigo em cartões. Cada id é provado pelo gesto REAL, nos rects pintados, e
//! o que chega ao barramento é o MESMO do painel de antes.

use std::collections::BTreeMap;

use super::g4_os_cartoes::{COLUNA, Mesa};
use super::g5_os_cartoes::sem_acusacao;
use ph2d_a11y::NodeId;
use ph2d_anim::{Easing, EasingFamily, EasingMode};
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::cards::{censo, grip_of};
use ph2d_editor_core::widget::key_mark::set_key_marks;
use ph2d_panel_vector::ids as p;
use ph2d_panel_vector::state::{
    self, ComponentState, LayoutItem, MorphShapeRow, MorphStatesState, UiStatesState,
    WidgetSkinState,
};
use ph2d_tool_vector::ids as t;

fn curva_com_direcao() -> Easing {
    Easing {
        family: EasingFamily::ALL[1],
        mode: EasingMode::ALL[1],
    }
}

/// Dois papéis gravados, um por gravar, dois sinais, e o interruptor da preview desligado.
fn poses() -> UiStatesState {
    UiStatesState {
        host: Some("Menu".into()),
        recorded: [true, true, false, false],
        role_labels: ["Default", "Hover", "Pressed", "Disabled"].map(String::from),
        live: None,
        duration_s: 0.5,
        spring: None,
        preview: Some(false),
        move_all: Some(false),
        easing: curva_com_direcao(),
        bindings: vec![("open".into(), 1), ("close".into(), 2)],
    }
}

fn maquina() -> MorphStatesState {
    MorphStatesState {
        rows: vec![
            MorphShapeRow {
                to: "Wide".into(),
                when: String::new(),
                live: true,
            },
            MorphShapeRow {
                to: "Tall".into(),
                when: "jump".into(),
                live: false,
            },
        ],
        actions: vec!["jump".into(), "crouch".into()],
        current: Some("Wide".into()),
        can_make: 0,
        preview: false,
    }
}

fn base() {
    ph2d_panel_vector::set_current_text_visible(false);
    ph2d_panel_vector::set_current_selection_count(1);
    state::set_frame_present(false);
    state::set_frame_clip(None);
    state::set_layout_flow(None);
    state::set_layout_item(None);
    state::set_component_state(None);
    state::set_widget_skin_state(None, 0);
    state::set_ui_states_state(Some(poses()));
    state::set_morph_states_state(Some(maquina()));
    set_key_marks(BTreeMap::new());
}

fn com_mola() {
    base();
    state::set_ui_states_state(Some(UiStatesState {
        spring: Some((20.0, 0.5)),
        ..poses()
    }));
}

fn em_preview() {
    base();
    state::set_ui_states_state(Some(UiStatesState {
        preview: Some(true),
        live: Some(1),
        ..poses()
    }));
    state::set_morph_states_state(Some(MorphStatesState {
        preview: true,
        ..maquina()
    }));
}

fn para_criar() {
    base();
    state::set_morph_states_state(Some(MorphStatesState {
        rows: Vec::new(),
        can_make: 3,
        ..maquina()
    }));
}

fn na_interface(cartoes: bool, semente: fn()) -> Mesa {
    let mut mesa = Mesa::com(cartoes, semente);
    mesa.clica(p::VECTOR_TAB_INTERFACE);
    mesa.assenta();
    mesa
}

fn cliques() -> Vec<NodeId> {
    let mut v = vec![
        p::VECTOR_STATE_PREVIEW,
        p::VECTOR_STATE_MOVE_ALL,
        p::VECTOR_STATE_SPRING,
        p::VECTOR_STATE_SIGNAL_ADD,
        p::VECTOR_MORPH_PREVIEW,
        p::VECTOR_MORPH_DISSOLVE,
    ];
    for i in 0..2 {
        v.extend([
            p::vector_state_record_id(i),
            p::vector_state_apply_id(i),
            p::vector_state_clear_id(i),
            p::vector_state_signal_remove_id(i),
            p::morph_shape_play_id(i),
            p::morph_shape_disconnect_id(i),
        ]);
        v.extend((0..4).map(|r| p::vector_state_signal_role_id(i, r)));
    }
    v.extend([p::vector_state_record_id(2), p::vector_state_record_id(3)]);
    v.extend((0..EasingFamily::ALL.len()).map(p::vector_easing_family_id));
    v.extend((0..EasingMode::ALL.len()).map(p::vector_easing_mode_id));
    v
}

#[test]
fn every_id_of_the_slice_is_painted_in_cards_and_in_the_old_panel() {
    for cartoes in [true, false] {
        let mesa = na_interface(cartoes, base);
        for id in cliques() {
            assert!(mesa.rect(id).is_some(), "{id:?} (cartões {cartoes})");
        }
        for id in [
            p::VECTOR_STATE_DURATION_NUM,
            p::vector_state_signal_name_id(0),
            p::vector_state_signal_name_id(1),
            p::morph_shape_key_button_id(0),
            p::morph_shape_key_button_id(1),
        ] {
            assert!(mesa.rect(id).is_some(), "{id:?} (cartões {cartoes})");
        }
    }
}

/// ⭐⭐ **Mesmo id, mesmo evento**. *Mutação: trocar dois ids num corpo ⇒ vermelho.*
#[test]
fn every_click_sends_the_same_action_as_the_old_panel() {
    for id in cliques() {
        let novo = na_interface(true, base).clica(id);
        assert_eq!(novo, na_interface(false, base).clica(id), "{id:?}");
    }
    let id = p::VECTOR_MORPH_STATES_MAKE;
    let novo = na_interface(true, para_criar).clica(id);
    assert!(!novo.is_empty(), "Make mudo");
    assert_eq!(novo, na_interface(false, para_criar).clica(id));
}

/// O ponteiro chega mesmo aos verbos: um id pintado e morto sob o rato já custou caro neste painel.
#[test]
fn no_click_is_silent() {
    for id in [
        p::VECTOR_STATE_PREVIEW,
        p::VECTOR_STATE_MOVE_ALL,
        p::VECTOR_STATE_SPRING,
        p::vector_state_record_id(2),
        p::vector_state_clear_id(0),
        p::vector_state_signal_remove_id(1),
        p::morph_shape_play_id(1),
        p::morph_shape_disconnect_id(0),
        p::VECTOR_MORPH_DISSOLVE,
    ] {
        assert!(
            !na_interface(true, base).clica(id).is_empty(),
            "{id:?} mudo"
        );
    }
}

/// ⭐ **Scrub de cada número**, e o Esc devolve. *Mutação: tirar `mark_card_field` ⇒ vermelho.*
#[test]
fn scrubbing_each_number_speaks_and_escape_gives_it_back() {
    for (semente, id) in [
        (base as fn(), p::VECTOR_STATE_DURATION_NUM),
        (com_mola, p::VECTOR_STATE_STIFFNESS_NUM),
        (com_mola, p::VECTOR_STATE_DAMPING_NUM),
    ] {
        let mut mesa = na_interface(true, semente);
        let antes = mesa.host.store().number_value(id).expect("semeado");
        let ev = mesa.arrasta_a_pega(id, 60.0);
        assert!(!ev.is_empty(), "{id:?} mudo");
        let durante = mesa.host.store().number_value(id).expect("campo");
        assert!((durante - antes).abs() > 1e-6, "{id:?} parado em {antes}");
        mesa.esc();
        let voltou = mesa.host.store().number_value(id).expect("campo");
        assert!((voltou - antes).abs() < 1e-6, "{id:?}: o Esc deu {voltou}");
    }
}

/// O campo mostra o documento. *Mutação: tirar a semente ⇒ vermelho.*
#[test]
fn each_field_shows_the_document_value() {
    let tem = |mesa: &Mesa, id| mesa.host.store().number_value(id).expect("campo");
    let mesa = na_interface(true, base);
    assert!((tem(&mesa, p::VECTOR_STATE_DURATION_NUM) - 0.5).abs() < 1e-6);
    let mesa = na_interface(true, com_mola);
    assert!((tem(&mesa, p::VECTOR_STATE_STIFFNESS_NUM) - 20.0).abs() < 1e-6);
    assert!((tem(&mesa, p::VECTOR_STATE_DAMPING_NUM) - 0.5).abs() < 1e-6);
}

/// Cada número sabe o passo e o valor de fábrica dele. *Mutação: tirar `mark_card_field` ⇒ vermelho:* o
/// campo marca-se sozinho ao pintar, sem fábrica, e o scrub continua a andar.
#[test]
fn the_numbers_know_their_factory_value() {
    let mesa = na_interface(true, com_mola);
    for (id, fabrica) in [
        (p::VECTOR_STATE_STIFFNESS_NUM, 12.0),
        (p::VECTOR_STATE_DAMPING_NUM, 1.0),
    ] {
        let spec = mesa.host.store().card_field(id).expect("marcado");
        assert_eq!(spec.default, Some(fabrica), "{id:?}");
    }
    let mesa = na_interface(true, base);
    let spec = mesa.host.store().card_field(p::VECTOR_STATE_DURATION_NUM);
    let fabrica = f64::from(ph2d_tokens::Duration::Fast.secs());
    assert_eq!(spec.expect("marcado").default, Some(fabrica));
}

/// Escrever dá o mesmo que no painel de antes.
#[test]
fn typing_sends_the_same_action_as_the_old_panel() {
    for (semente, id, texto) in [
        (base as fn(), p::VECTOR_STATE_DURATION_NUM, "1.5"),
        (com_mola, p::VECTOR_STATE_STIFFNESS_NUM, "30"),
        (com_mola, p::VECTOR_STATE_DAMPING_NUM, "1.5"),
    ] {
        let grava = |cartoes: bool| {
            let mut mesa = na_interface(cartoes, semente);
            let ev = mesa.host.type_into_number(id, texto);
            mesa.aplica(ev)
        };
        let novo = grava(true);
        assert!(!novo.is_empty(), "{id:?} mudo");
        assert_eq!(novo, grava(false), "{id:?}");
    }
}

/// O nome do sinal é o texto do documento, e o Enter leva-o, como no painel de antes.
#[test]
fn the_signal_name_commits_like_the_old_panel() {
    let id = p::vector_state_signal_name_id(0);
    let mesa = na_interface(true, base);
    let texto = match mesa.host.store().get(id) {
        Some(InteractiveState::TextInput { text, .. }) => text.clone(),
        outro => panic!("o campo é {outro:?}"),
    };
    assert_eq!(texto, "open", "o campo não mostra o nome do documento");
    let grava = |cartoes: bool| {
        let mut mesa = na_interface(cartoes, base);
        let ev = mesa.host.type_keys_then_enter(id, "x");
        mesa.aplica(ev)
    };
    let novo = grava(true);
    assert!(!novo.is_empty(), "o nome não chegou a ninguém");
    assert_eq!(novo, grava(false));
}

/// A tecla abre a lista dela, e cada opção manda o mesmo que antes.
#[test]
fn the_key_box_opens_its_list_and_each_option_speaks() {
    let id = p::morph_shape_key_button_id(1);
    let opcao = p::morph_shape_key_option_id(1, 2);
    let escolhe = |cartoes: bool| {
        let mut mesa = na_interface(cartoes, base);
        mesa.clica(id);
        mesa.assenta();
        assert_eq!(mesa.host.dropdown_is_open(id), Some(true), "{cartoes}");
        assert!(mesa.rect(opcao).is_some(), "lista por pintar ({cartoes})");
        mesa.clica(opcao)
    };
    let novo = escolhe(true);
    assert!(!novo.is_empty(), "a opção é muda");
    assert_eq!(novo, escolhe(false));
}

/// ⭐ **O que aparece responde às perguntas de antes.** Cada controlo só onde tem sujeito.
#[test]
fn what_appears_answers_the_old_questions() {
    // Com a mola, a duração e a curva saem, porque respondem à mesma pergunta.
    let mesa = na_interface(true, com_mola);
    assert!(mesa.rect(p::VECTOR_STATE_STIFFNESS_NUM).is_some());
    assert!(mesa.rect(p::VECTOR_STATE_DURATION_NUM).is_none());
    assert!(mesa.rect(p::vector_easing_family_id(0)).is_none());
    // Sem a mola, ela sai.
    let mesa = na_interface(true, base);
    assert!(mesa.rect(p::VECTOR_STATE_STIFFNESS_NUM).is_none());
    // Show e Clear só no gravado; Rec em todos.
    assert!(mesa.rect(p::vector_state_apply_id(2)).is_none());
    assert!(mesa.rect(p::vector_state_clear_id(3)).is_none());
    // Em preview, a autoria fecha inteira.
    let mesa = na_interface(true, em_preview);
    for id in [
        p::vector_state_record_id(0),
        p::vector_state_apply_id(0),
        p::VECTOR_STATE_SPRING,
        p::VECTOR_STATE_DURATION_NUM,
        p::vector_easing_family_id(0),
        p::VECTOR_STATE_SIGNAL_ADD,
    ] {
        assert!(mesa.rect(id).is_none(), "{id:?} em preview");
    }
    assert!(mesa.rect(p::VECTOR_STATE_PREVIEW).is_some());
    // Sem máquina, só a face que a cria.
    let mesa = na_interface(true, para_criar);
    assert!(mesa.rect(p::VECTOR_MORPH_STATES_MAKE).is_some());
    assert!(mesa.rect(p::VECTOR_MORPH_DISSOLVE).is_none());
    assert!(mesa.rect(p::VECTOR_MORPH_PREVIEW).is_none());
}

/// A curva sem direção não oferece direção: seriam três opções que desenham a mesma coisa.
#[test]
fn the_direction_only_where_the_curve_uses_it() {
    let sem = *EasingFamily::ALL
        .iter()
        .find(|f| !f.uses_mode())
        .expect("uma família sem modo");
    let mut mesa = na_interface(true, base);
    assert!(mesa.rect(p::vector_easing_mode_id(0)).is_some());
    state::set_ui_states_state(Some(UiStatesState {
        easing: Easing {
            family: sem,
            mode: EasingMode::ALL[0],
        },
        ..poses()
    }));
    mesa.assenta();
    assert!(mesa.rect(p::vector_easing_mode_id(0)).is_none());
}

/// A face das frases: sem hospedeiro, formas a menos e a mais, e a lista além do tecto.
#[test]
fn the_sentence_faces_have_no_controls_and_grow_the_card() {
    let sem_hospedeiro = || {
        base();
        state::set_ui_states_state(Some(UiStatesState {
            host: None,
            ..poses()
        }));
    };
    let mesa = na_interface(true, sem_hospedeiro);
    assert!(mesa.rect(p::VECTOR_STATE_PREVIEW).is_none());
    assert!(mesa.rect(p::vector_state_record_id(0)).is_none());
    // As duas frases ocupam lugar: o cartão aberto é mais alto que o dobrado.
    let do_cartao = |mesa: &Mesa| {
        let y = |id| mesa.rect(grip_of(id)).expect("título").y;
        y(p::VECTOR_SECTION_MORPH_STATES) - y(p::VECTOR_SECTION_STATES)
    };
    let aberto = do_cartao(&mesa);
    let mut mesa = mesa;
    mesa.host.set_collapsed(p::VECTOR_SECTION_STATES, true);
    mesa.assenta();
    assert!(
        aberto > do_cartao(&mesa) + 20.0,
        "as frases não se pintaram"
    );

    for can_make in [1, p::MAX_MORPH_STATES + 1] {
        let mut mesa = na_interface(true, base);
        state::set_morph_states_state(Some(MorphStatesState {
            rows: Vec::new(),
            can_make,
            ..maquina()
        }));
        mesa.assenta();
        assert!(
            mesa.rect(p::VECTOR_MORPH_STATES_MAKE).is_none(),
            "{can_make}"
        );
    }

    let mut mesa = na_interface(true, base);
    let linha = |i: usize| MorphShapeRow {
        to: format!("S{i}"),
        when: String::new(),
        live: false,
    };
    state::set_morph_states_state(Some(MorphStatesState {
        rows: (0..p::MAX_MORPH_STATES + 1).map(linha).collect(),
        ..maquina()
    }));
    mesa.assenta();
    // 118 linhas passam da coluna: o que está fora da janela não regista alvo.
    assert!(mesa.rect(p::morph_shape_play_id(60)).is_some());
}

/// Um nome longo sobe a rótulo e não se corta; a tecla continua na linha.
#[test]
fn a_long_shape_name_is_not_cut() {
    let longa = || {
        base();
        state::set_morph_states_state(Some(MorphStatesState {
            rows: vec![MorphShapeRow {
                to: "A very long shape name that no column can hold".into(),
                when: "jump".into(),
                live: false,
            }],
            ..maquina()
        }));
    };
    sem_acusacao("nome longo", || {
        let mesa = na_interface(true, longa);
        assert!(mesa.rect(p::morph_shape_key_button_id(0)).is_some());
        assert!(mesa.rect(p::morph_shape_play_id(0)).is_some());
    });
}

/// ⭐ **O aceso segue o documento.** *Mutação: acender sempre, ou nunca ⇒ vermelho.* O tamanho da
/// pintura não distingue: é a tinta que muda, por isso compara os bytes.
#[test]
fn what_is_lit_follows_the_document() {
    let pinta = |semente: &dyn Fn()| {
        let mut mesa = na_interface(true, base);
        semente();
        mesa.assenta();
        let (h, st) = (&mut mesa.host, &mut mesa.st);
        h.paint_draw_data::<ph2d_panel_vector::VectorPanel>(st, COLUNA)
    };
    let com = |muda: fn(&mut UiStatesState)| {
        move || {
            let mut s = poses();
            muda(&mut s);
            state::set_ui_states_state(Some(s));
        }
    };
    let nada = com(|_| {});
    let casos: [(&str, Box<dyn Fn()>); 7] = [
        ("Preview", Box::new(com(|s| s.preview = Some(true)))),
        ("Move All", Box::new(com(|s| s.move_all = Some(true)))),
        (
            "Curve",
            Box::new(com(|s| s.easing.family = EasingFamily::ALL[2])),
        ),
        (
            "Direction",
            Box::new(com(|s| s.easing.mode = EasingMode::ALL[2])),
        ),
        ("Papel do sinal", Box::new(com(|s| s.bindings[0].1 = 3))),
        ("Papel à vista", Box::new(com(|s| s.live = Some(0)))),
        (
            "Papel gravado",
            Box::new(com(|s| {
                s.recorded = [true, false, false, false];
                s.live = Some(3);
            })),
        ),
    ];
    for (nome, caso) in casos {
        assert_ne!(pinta(&nada), pinta(&*caso), "{nome}");
    }
    assert_ne!(
        pinta(&|| state::set_morph_states_state(Some(maquina()))),
        pinta(&|| state::set_morph_states_state(Some(MorphStatesState {
            preview: true,
            ..maquina()
        }))),
        "Preview do Morph"
    );
}

/// Os ícones mudos têm dica. *Mutação: tirar a `dica` ⇒ vermelho.*
#[test]
fn the_icon_verbs_explain_themselves() {
    let mesa = na_interface(true, base);
    for id in [
        p::vector_state_signal_remove_id(0),
        p::morph_shape_play_id(0),
        p::morph_shape_disconnect_id(0),
    ] {
        let dica = mesa.host.store().tooltip_for(id);
        assert!(dica.is_some_and(|d| !d.is_empty()), "{id:?}");
    }
}

/// ⭐⭐ **A lei da compactação** sobre cada face que a varredura do registo não semeia.
#[test]
fn the_slice_passes_the_compaction_census() {
    for (nome, semente) in [
        ("base", base as fn()),
        ("mola", com_mola),
        ("preview", em_preview),
        ("criar", para_criar),
    ] {
        sem_acusacao(nome, || {
            let _ = na_interface(true, semente);
        });
    }
}

fn vazio() {
    base();
    state::set_ui_states_state(None);
    state::set_morph_states_state(None);
    ph2d_panel_vector::set_current_selection_count(0);
}

fn interface() {
    base();
    state::set_frame_present(true);
    state::set_frame_clip(Some(false));
    state::set_component_state(Some(ComponentState::default()));
    state::set_widget_skin_state(
        Some(WidgetSkinState {
            kinds: vec!["Button".into()],
            selected: None,
            drives: None,
            icon: None,
            unknown: false,
        }),
        0,
    );
}

fn item() {
    vazio();
    ph2d_panel_vector::set_current_selection_count(1);
    state::set_layout_item(Some(LayoutItem {
        absolute: false,
        in_flow: false,
        grow: 0.0,
        shrink: 1.0,
        parent_is_grid: false,
    }));
}

fn texto() {
    vazio();
    ph2d_panel_vector::set_current_text_visible(true);
    ph2d_panel_vector::set_current_text_axes(Vec::new());
}

fn eixos() {
    texto();
    ph2d_panel_vector::set_current_text_axes(vec![ph2d_panel_vector::TextAxisSlot {
        name: "Width".into(),
        min: 0.0,
        max: 1.0,
        value: 0.5,
    }]);
}

/// O painel de antes e os cartões respondem o mesmo: que secções há, estado a estado e aba a aba.
/// *Mutação: trocar uma pergunta na tabela de presença ⇒ vermelho.*
#[test]
fn presence_answers_like_the_old_panel() {
    let semeados: [(&str, fn()); 6] = [
        ("vazio", vazio),
        ("base", base),
        ("interface", interface),
        ("item", item),
        ("texto", texto),
        ("eixos", eixos),
    ];
    let abas = [p::VECTOR_TAB_DRAWING, p::VECTOR_TAB_INTERFACE];
    for (nome, semente) in semeados {
        for aba in abas {
            let cartoes = {
                let mut mesa = Mesa::com(true, semente);
                mesa.clica(aba);
                mesa.assenta();
                p::VECTOR_SECTIONS
                    .iter()
                    .filter(|s| mesa.rect(grip_of(**s)).is_some() || mesa.rect(**s).is_some())
                    .copied()
                    .collect::<Vec<_>>()
            };
            let antes = {
                let mut mesa = Mesa::com(false, semente);
                mesa.clica(aba);
                mesa.assenta();
                p::VECTOR_SECTIONS
                    .iter()
                    .filter(|s| mesa.rect(**s).is_some())
                    .copied()
                    .collect::<Vec<_>>()
            };
            assert_eq!(cartoes, antes, "{nome}, aba {aba:?}");
        }
    }
}

fn fontes(prefixo: &str) -> Vec<(String, String)> {
    let pasta = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
    let mut v: Vec<(String, String)> = std::fs::read_dir(pasta)
        .expect("src")
        .filter_map(|e| {
            let e = e.ok()?;
            let nome = e.file_name().into_string().ok()?;
            nome.starts_with(prefixo)
                .then(|| (nome, std::fs::read_to_string(e.path()).unwrap_or_default()))
        })
        .collect();
    v.sort();
    v
}

/// ⭐⭐ **Zero corpo antigo em cartões**, no código: nenhum corpo de cartão chama um pintor antigo, e a
/// sonda e o hospedeiro não existem. *Mutação: chamar um `*_section` num cartão ⇒ vermelho.*
#[test]
fn no_card_runs_an_old_painter() {
    let cartoes = fontes("cards");
    assert!(cartoes.len() >= 10, "o censo não leu os cartões");
    for (nome, texto) in &cartoes {
        for proibido in [
            "_section(",
            ".paint_body(",
            "uma_seccao",
            "hospeda(",
            "absorb_next_header",
            "sonda",
        ] {
            assert!(!texto.contains(proibido), "{nome} usa {proibido}");
        }
    }
    let (_, presenca) = cartoes
        .iter()
        .find(|(n, _)| n == "cards_presenca.rs")
        .expect("a tabela");
    // A lista do corpo antigo só responde à aba e ao escopo: as tarefas dela nunca correm aqui.
    let chamadas = presenca.matches("corpo(snap)").count();
    let compactado: String = presenca.split_whitespace().collect();
    assert_eq!(chamadas, 1);
    assert!(compactado.contains("corpo(snap).de_topo_a_vista()"));
    for (_, texto) in fontes("paint_body_plan") {
        assert!(!texto.contains("fn tira"), "o pintor antigo saía da lista");
        assert!(!texto.contains("fn membros"), "os membros antigos");
    }
}

/// O título de cada cartão é o da secção de antes. Pelo código: o pintor antigo diz o par id + chave.
/// *Mutação: trocar dois títulos na tabela ⇒ vermelho.*
#[test]
fn each_card_title_is_the_old_section_title() {
    let sem_espaco = |s: &str| s.split_whitespace().collect::<String>();
    let antigo = sem_espaco(
        &fontes("paint")
            .into_iter()
            .map(|(_, t)| t)
            .collect::<String>(),
    );
    let mut pares = 0;
    for ficheiro in ["cards_presenca.rs", "cards_avancado.rs"] {
        let (_, texto) = fontes(ficheiro).pop().expect("a tabela");
        for pedaco in sem_espaco(&texto).split("VECTOR_SECTION_").skip(1) {
            let nome: String = pedaco
                .chars()
                .take_while(|c| c.is_ascii_uppercase() || *c == '_')
                .collect();
            let Some(i) = pedaco.find("\"panel.vector.section.") else {
                continue;
            };
            let chave = &pedaco[i..=pedaco[i + 1..].find('"').map_or(i, |f| i + 1 + f)];
            let par = [
                format!("VECTOR_SECTION_{nome},tr({chave})"),
                format!("VECTOR_SECTION_{nome},{chave}"),
            ];
            assert!(
                par.iter().any(|p| antigo.contains(p.as_str())),
                "{nome} com {chave} em {ficheiro}: o painel de antes diz outra coisa"
            );
            pares += 1;
        }
    }
    assert!(pares >= 30, "o censo leu só {pares} pares");
}

/// Os membros do Advanced em cartões são os do grupo, e estão todos no cartão.
#[test]
fn the_advanced_members_stay_in_their_card() {
    let mesa = Mesa::com(true, || {
        base();
        ph2d_panel_vector::set_current_selection_count(1);
    });
    for id in [
        t::VECTOR_SECTION_BLEND,
        t::VECTOR_SECTION_MORPH,
        t::VECTOR_SECTION_ENVELOPE,
    ] {
        assert!(mesa.rect(id).is_some(), "{id:?}");
    }
}

/// A tinta de um estado, sem a ordem: trocar qual opção acende não muda o multiconjunto, e acender duas
/// ou nenhuma muda.
fn tinta(semente: fn(), aba: NodeId, muda: &dyn Fn()) -> (Vec<u32>, Vec<u32>) {
    let mut mesa = Mesa::com(true, semente);
    mesa.clica(aba);
    muda();
    mesa.assenta();
    let (h, st) = (&mut mesa.host, &mut mesa.st);
    let pintura = h.paint_draw_data::<ph2d_panel_vector::VectorPanel>(st, COLUNA);
    let mut ordenada = pintura.clone();
    ordenada.sort_unstable();
    (pintura, ordenada)
}

/// ⭐ **Em cada escolha acende UMA opção, a do documento.** Os seis sobreviventes da G6 e da G7: uma
/// opção acesa fixa muda só a contagem do que acende. *Mutação: `quina == 0` → `true` ⇒ vermelho.*
#[test]
fn every_choice_lights_exactly_one_option() {
    let (d, i) = (p::VECTOR_TAB_DRAWING, p::VECTOR_TAB_INTERFACE);
    let tudo = super::g6_os_cartoes::tudo as fn();
    let contorno = |quina, lado| {
        ph2d_panel_vector::set_current_contour(
            true,
            6.0,
            0.125,
            2.0,
            quina,
            lado,
            [16, 32, 48, 255],
        );
    };
    let vinculo = |flip| {
        ph2d_panel_vector::set_current_patternpath(true, 0.25, 0.75, 1.5, 0.5, flip, 30.0);
    };
    let gaiola = |m| ph2d_panel_vector::set_current_envelope_mode(m);
    let direcao = |id: Option<NodeId>| {
        state::set_frame_present(true);
        state::set_frame_clip(Some(false));
        state::set_layout_flow(id.map(|dir| state::LayoutFlow {
            size: [p::VECTOR_LAYOUT_SIZE_W_FIXED, p::VECTOR_LAYOUT_SIZE_H_HUG],
            min: [1.0, 2.0],
            max: [3.0, 4.0],
            dir,
            gap: [0.5, 0.25],
            pad: [0.125; 4],
            align: p::VECTOR_LAYOUT_ALIGN_CENTER,
            justify: p::VECTOR_LAYOUT_JUSTIFY_END,
            columns: 3.0,
        }));
    };
    let com_curva = |familia: usize, modo: usize| {
        state::set_ui_states_state(Some(UiStatesState {
            easing: Easing {
                family: EasingFamily::ALL[familia],
                mode: EasingMode::ALL[modo],
            },
            ..poses()
        }));
    };
    type Casos<'a> = Vec<(
        &'a str,
        fn(),
        NodeId,
        Box<dyn Fn() + 'a>,
        Box<dyn Fn() + 'a>,
    )>;
    let casos: Casos<'_> = vec![
        (
            "Cage",
            tudo,
            d,
            Box::new(move || gaiola(0)),
            Box::new(move || gaiola(1)),
        ),
        (
            "Corner 0-1",
            tudo,
            d,
            Box::new(move || contorno(0, 0)),
            Box::new(move || contorno(1, 0)),
        ),
        (
            "Corner 1-2",
            tudo,
            d,
            Box::new(move || contorno(1, 0)),
            Box::new(move || contorno(2, 0)),
        ),
        (
            "Side 0-1",
            tudo,
            d,
            Box::new(move || contorno(0, 0)),
            Box::new(move || contorno(0, 1)),
        ),
        (
            "Side 1-2",
            tudo,
            d,
            Box::new(move || contorno(0, 1)),
            Box::new(move || contorno(0, 2)),
        ),
        (
            "Flip",
            tudo,
            d,
            Box::new(move || vinculo(false)),
            Box::new(move || vinculo(true)),
        ),
        (
            "Direction",
            base,
            i,
            Box::new(move || direcao(Some(p::VECTOR_LAYOUT_DIR_ROW))),
            Box::new(move || direcao(Some(p::VECTOR_LAYOUT_DIR_COL))),
        ),
        (
            "Curve",
            base,
            i,
            Box::new(move || com_curva(1, 0)),
            Box::new(move || com_curva(2, 0)),
        ),
        (
            "Curve dir",
            base,
            i,
            Box::new(move || com_curva(1, 0)),
            Box::new(move || com_curva(1, 2)),
        ),
    ];
    for (nome, semente, aba, a, b) in casos {
        let (pa, oa) = tinta(semente, aba, &*a);
        let (pb, ob) = tinta(semente, aba, &*b);
        assert_ne!(pa, pb, "{nome}: o aceso não segue o documento");
        assert_eq!(oa, ob, "{nome}: acendeu outra contagem de opções");
    }
}

/// A lista dos ícones tem a largura do cartão: os nomes dos glifos não cabiam na caixa, que divide a
/// linha com Drives. *Mutação: ancorar na caixa ⇒ vermelho.*
#[test]
fn the_icon_list_is_as_wide_as_the_card() {
    let mut mesa = na_interface(true, || {
        base();
        state::set_widget_skin_state(
            Some(WidgetSkinState {
                kinds: vec!["Button".into()],
                selected: Some(0),
                drives: Some(None),
                icon: Some(None),
                unknown: false,
            }),
            0,
        );
    });
    let caixa = mesa.rect(p::VECTOR_WIDGET_ICON_DD).expect("caixa");
    mesa.clica(p::VECTOR_WIDGET_ICON_DD);
    mesa.assenta();
    let linha = mesa
        .rect(p::vector_widget_icon_option_id(1))
        .expect("opção");
    assert!(
        linha.w > caixa.w * 1.5,
        "lista {} sobre caixa {}",
        linha.w,
        caixa.w
    );
}

/// Quantas frases o cartão `secao` escreve neste estado: uma linha sem células, no censo.
fn frases(semente: fn(), secao: NodeId) -> usize {
    let mut mesa = na_interface(true, semente);
    let ((), censo) = censo::grava(|| mesa.assenta());
    let mut topos: Vec<i32> = censo
        .linhas
        .iter()
        .filter(|l| l.secao == secao && l.celulas.is_empty())
        .map(|l| l.topo.round() as i32)
        .collect();
    topos.sort_unstable();
    topos.dedup();
    topos.len()
}

fn morph_com(rows: usize, can_make: usize) {
    base();
    let linha = |i: usize| MorphShapeRow {
        to: format!("S{i}"),
        when: String::new(),
        live: false,
    };
    state::set_morph_states_state(Some(MorphStatesState {
        rows: (0..rows).map(linha).collect(),
        can_make,
        ..maquina()
    }));
}

/// ⭐ **As frases estão lá**: de quem são as poses, o que falta, como se sai e o que ficou de fora.
/// *Mutação: tirar qualquer `c.hint` deste cartão ⇒ vermelho.*
#[test]
fn each_face_writes_its_sentences() {
    let (st, mo) = (p::VECTOR_SECTION_STATES, p::VECTOR_SECTION_MORPH_STATES);
    let sem_hospedeiro = || {
        base();
        state::set_ui_states_state(Some(UiStatesState {
            host: None,
            ..poses()
        }));
    };
    assert_eq!(frases(base, st), 1, "de quem são as poses");
    assert_eq!(frases(sem_hospedeiro, st), 2, "sem hospedeiro");
    assert_eq!(
        frases(em_preview, st),
        3,
        "dono, saída e o que está à vista"
    );
    assert_eq!(frases(base, mo), 0);
    assert_eq!(frases(em_preview, mo), 1, "a saída do teclado");
    assert_eq!(frases(para_criar, mo), 1, "a promessa do botão");
    assert_eq!(frases(|| morph_com(0, 1), mo), 1, "formas a menos");
    assert_eq!(
        frases(|| morph_com(0, p::MAX_MORPH_STATES + 1), mo),
        1,
        "formas a mais"
    );
    assert_eq!(
        frases(|| morph_com(p::MAX_MORPH_STATES + 1, 0), mo),
        1,
        "além do tecto"
    );
}
