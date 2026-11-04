//! ⭐⭐ **O painel Vector em cartões, fatia G6** (`docs/CartoesUI/12_VECTOR_PLANO.md`): os cinco membros
//! do Advanced. Cada id é provado pelo gesto REAL, nos rects pintados, e o que chega ao barramento é o
//! MESMO do painel de antes.

use std::collections::BTreeMap;

use super::g4_os_cartoes::Mesa;
use super::g5_os_cartoes::sem_acusacao;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::cards::dicas::field_hint;
use ph2d_editor_core::widget::key_mark::{KeyMark, key_mark_id, set_key_marks};
use ph2d_panel_vector::ids as p;
use ph2d_tool_vector::ids as t;

const PRESETS: &[&str] = &["Arc", "Bulge", "Wave"];

/// Forma selecionada, sem envelope, vínculo nem contour: só as portas de entrada.
fn sem_nada() {
    ph2d_panel_vector::set_current_text_visible(false);
    ph2d_panel_vector::set_current_selection_count(2);
    ph2d_panel_vector::set_current_has_envelope(false);
    ph2d_panel_vector::set_current_envelope_mode(0);
    ph2d_panel_vector::set_current_envelope_presets(PRESETS, None, 0.5);
    ph2d_panel_vector::set_current_patternpath(false, 0.0, 1.0, 1.0, 0.0, false, 0.0);
    ph2d_panel_vector::set_current_patternpath_can_link(true);
    ph2d_panel_vector::set_current_patternpath_can_pick(false);
    ph2d_panel_vector::set_current_contour(false, 4.0, 0.0, 1.0, 0, 0, [255; 4]);
    ph2d_panel_vector::set_current_contour_can_add(true);
    set_key_marks(BTreeMap::new());
}

/// Tudo vivo: envelope com preset, vínculo, contour.
fn tudo() {
    sem_nada();
    ph2d_panel_vector::set_current_has_envelope(true);
    ph2d_panel_vector::set_current_envelope_presets(PRESETS, Some(1), 0.25);
    ph2d_panel_vector::set_current_patternpath(true, 0.25, 0.75, 1.5, 0.5, false, 30.0);
    ph2d_panel_vector::set_current_contour(true, 6.0, 0.125, 2.0, 1, 2, [16, 32, 48, 255]);
}

fn pinos() {
    tudo();
    ph2d_panel_vector::set_current_envelope_mode(2);
}

fn cliques() -> Vec<NodeId> {
    let mut v = vec![
        t::VECTOR_MODE_PICKBLEND,
        p::VECTOR_BLEND_RUN,
        p::VECTOR_BLEND_RESET_SPINE,
        p::VECTOR_BLEND_EXPAND,
        p::VECTOR_BLEND_RELEASE,
        p::VECTOR_MORPH_RUN,
        p::VECTOR_ENVELOPE_RUN,
        p::VECTOR_ENVELOPE_PERSPECTIVE,
        p::VECTOR_ENVELOPE_MESH,
        p::VECTOR_ENVELOPE_PINS,
        p::VECTOR_ENVELOPE_EXPAND,
        p::VECTOR_ENVELOPE_RELEASE,
        p::VECTOR_PATTERNPATH_FLIP_OFF,
        p::VECTOR_PATTERNPATH_FLIP,
        p::VECTOR_PATTERNPATH_DETACH,
        p::VECTOR_CONTOUR_TO,
        p::VECTOR_CONTOUR_JOIN_MITER,
        p::VECTOR_CONTOUR_JOIN_ROUND,
        p::VECTOR_CONTOUR_JOIN_BEVEL,
        p::VECTOR_CONTOUR_SIDE_OUTER,
        p::VECTOR_CONTOUR_SIDE_INNER,
        p::VECTOR_CONTOUR_SIDE_BOTH,
        p::VECTOR_CONTOUR_EXPAND,
        p::VECTOR_CONTOUR_REMOVE,
    ];
    v.extend((0..PRESETS.len()).map(p::vector_envelope_preset_id));
    v
}

/// `(chip, quem fala por ele)`: a barra, salvo a Accel, que fala pelo próprio chip.
fn numeros() -> [(NodeId, NodeId); 12] {
    [
        (t::VECTOR_BLEND_STEPS_NUM, p::VECTOR_BLEND_STEPS),
        (t::VECTOR_MORPH_T_NUM, p::VECTOR_MORPH_T),
        (t::VECTOR_ENVELOPE_BEND_NUM, p::VECTOR_ENVELOPE_BEND),
        (
            p::VECTOR_PATTERNPATH_SPACING_NUM,
            p::VECTOR_PATTERNPATH_SPACING,
        ),
        (p::VECTOR_PATTERNPATH_SLIDE_NUM, p::VECTOR_PATTERNPATH_SLIDE),
        (p::VECTOR_PATTERNPATH_START_NUM, p::VECTOR_PATTERNPATH_START),
        (p::VECTOR_PATTERNPATH_END_NUM, p::VECTOR_PATTERNPATH_END),
        (
            p::VECTOR_PATTERNPATH_OFFSET_NUM,
            p::VECTOR_PATTERNPATH_OFFSET,
        ),
        (
            p::VECTOR_PATTERNPATH_ROTATION_NUM,
            p::VECTOR_PATTERNPATH_ROTATION,
        ),
        (p::VECTOR_CONTOUR_STEPS_NUM, p::VECTOR_CONTOUR_STEPS),
        (p::VECTOR_CONTOUR_OFFSET_NUM, p::VECTOR_CONTOUR_OFFSET),
        (p::VECTOR_CONTOUR_ACCEL_NUM, p::VECTOR_CONTOUR_ACCEL_NUM),
    ]
}

#[test]
fn every_id_of_the_slice_is_painted_in_cards_and_in_the_old_panel() {
    for cartoes in [true, false] {
        let mesa = Mesa::com(cartoes, tudo);
        for id in cliques() {
            assert!(mesa.rect(id).is_some(), "{id:?} (cartões {cartoes})");
        }
        for (chip, _) in numeros() {
            assert!(mesa.rect(chip).is_some(), "{chip:?} (cartões {cartoes})");
        }
        let vazia = Mesa::com(cartoes, sem_nada);
        for id in [
            p::VECTOR_ENVELOPE_RUN,
            p::VECTOR_PATTERNPATH_LINK,
            p::VECTOR_CONTOUR_ADD,
        ] {
            assert!(vazia.rect(id).is_some(), "{id:?} (cartões {cartoes})");
        }
    }
}

/// ⭐⭐ **A metade esquerda deixou de ficar vazia**: cada número começa na coluna do cartão ou na 2.ª
/// da grelha, e nenhuma barra antiga se pinta. *Mutação: voltar ao corpo antigo ⇒ vermelho.*
#[test]
fn the_left_half_is_used_and_no_old_bar_is_painted() {
    let mesa = Mesa::com(true, tudo);
    let r = |id: NodeId| mesa.rect(id).unwrap_or_else(|| panic!("{id:?}"));
    let esquerda = r(t::VECTOR_BLEND_STEPS_NUM).x;
    let direita = r(p::VECTOR_BLEND_RESET_SPINE).x;
    assert!(direita > esquerda);
    for (chip, barra) in numeros() {
        let x = r(chip).x;
        assert!(
            (x - esquerda).abs() < 1.0 || (x - direita).abs() < 1.0,
            "{chip:?} fora da grelha: {x}"
        );
        if chip != barra {
            assert!(mesa.rect(barra).is_none(), "barra {barra:?}");
        }
    }
    assert!((r(p::VECTOR_CONTOUR_ACCEL_NUM).x - esquerda).abs() < 1.0);
}

/// ⭐⭐ **Mesmo id, mesmo evento**. *Mutação: trocar dois ids num corpo ⇒ vermelho.*
#[test]
fn every_click_sends_the_same_action_as_the_old_panel() {
    for id in cliques() {
        let novo = Mesa::com(true, tudo).clica(id);
        assert_eq!(novo, Mesa::com(false, tudo).clica(id), "{id:?}");
    }
    for id in [
        p::VECTOR_ENVELOPE_RUN,
        p::VECTOR_PATTERNPATH_LINK,
        p::VECTOR_CONTOUR_ADD,
    ] {
        let novo = Mesa::com(true, sem_nada).clica(id);
        assert!(!novo.is_empty(), "{id:?} mudo");
        assert_eq!(novo, Mesa::com(false, sem_nada).clica(id), "{id:?}");
    }
    let novo = Mesa::com(true, pinos).clica(p::VECTOR_ENVELOPE_CLEAR_PINS);
    assert!(!novo.is_empty(), "Clear Pins mudo");
    assert_eq!(
        novo,
        Mesa::com(false, pinos).clica(p::VECTOR_ENVELOPE_CLEAR_PINS)
    );
}

/// ⭐ **Scrub de cada número**: fala pela barra, a semente a meio não o puxa, e o Esc devolve.
/// *Mutação: tirar `mark_card_field` ou `semeia` ⇒ vermelho.*
#[test]
fn scrubbing_each_number_reaches_the_tool_and_escape_gives_it_back() {
    for (chip, fala) in numeros() {
        let mut mesa = Mesa::com(true, tudo);
        let antes = mesa.host.store().number_value(chip).expect("semeado");
        let ev = mesa.arrasta_a_pega(chip, 60.0);
        assert!(
            ev.contains(&WidgetEvent::ValueChanged(fala)),
            "{chip:?} não falou por {fala:?}: {ev:?}"
        );
        let durante = mesa.host.store().number_value(chip).expect("campo");
        assert!((durante - antes).abs() > 1e-6, "{chip:?} parado em {antes}");
        mesa.assenta();
        let segue = mesa.host.store().number_value(chip).expect("campo");
        assert!(
            (segue - durante).abs() < 1e-6,
            "{chip:?}: a semente puxou a meio do scrub"
        );
        mesa.esc();
        let voltou = mesa.host.store().number_value(chip).expect("campo");
        assert!(
            (voltou - antes).abs() < 1e-6,
            "{chip:?}: o Esc deu {voltou}, era {antes}"
        );
    }
}

/// O campo mostra o valor do DOCUMENTO, em cada unidade. *Mutação: tirar uma semente ⇒ vermelho.*
#[test]
fn each_field_shows_the_document_value() {
    let mesa = Mesa::com(true, tudo);
    for (id, valor) in [
        (t::VECTOR_ENVELOPE_BEND_NUM, 0.25),
        (p::VECTOR_PATTERNPATH_SPACING_NUM, 1.5),
        (p::VECTOR_PATTERNPATH_START_NUM, 0.25),
        (p::VECTOR_PATTERNPATH_END_NUM, 0.75),
        (p::VECTOR_PATTERNPATH_SLIDE_NUM, 0.5),
        (p::VECTOR_PATTERNPATH_OFFSET_NUM, 0.5),
        (p::VECTOR_PATTERNPATH_ROTATION_NUM, 30.0),
        (p::VECTOR_CONTOUR_STEPS_NUM, 6.0),
        (p::VECTOR_CONTOUR_OFFSET_NUM, 12.5),
        (p::VECTOR_CONTOUR_ACCEL_NUM, 2.0),
    ] {
        let tem = mesa.host.store().number_value(id).expect("campo");
        assert!((tem - valor).abs() < 1e-6, "{id:?}: {tem}, era {valor}");
    }
}

/// Passos do Blend e `t` do Morph vivem na barra, que a shell move: o campo segue-a.
/// *Mutação: tirar a semente ⇒ vermelho.*
#[test]
fn blend_steps_and_morph_t_follow_their_bars() {
    use ph2d_editor_core::panel::PanelHostInternal;
    use ph2d_tool_vector::params::{blend_steps_from_track, blend_steps_to_track};
    let mut mesa = Mesa::com(true, tudo);
    let trilho = blend_steps_to_track(12);
    mesa.host
        .store_mut()
        .set_slider_value(p::VECTOR_BLEND_STEPS, trilho);
    mesa.host
        .store_mut()
        .set_slider_value(p::VECTOR_MORPH_T, 0.25);
    mesa.assenta();
    let tem = |id: NodeId| mesa.host.store().number_value(id).expect("campo");
    let passos = blend_steps_from_track(f64::from(trilho));
    assert_eq!(tem(t::VECTOR_BLEND_STEPS_NUM), f64::from(passos));
    assert_eq!(passos, 12);
    assert!((tem(t::VECTOR_MORPH_T_NUM) - 0.25).abs() < 1e-6);
}

/// Escrever um número dá o mesmo que no painel de antes.
#[test]
fn typing_sends_the_same_action_as_the_old_panel() {
    for (id, texto) in [
        (t::VECTOR_BLEND_STEPS_NUM, "9"),
        (t::VECTOR_MORPH_T_NUM, "0.4"),
        (t::VECTOR_ENVELOPE_BEND_NUM, "0.2"),
        (p::VECTOR_PATTERNPATH_SPACING_NUM, "2"),
        (p::VECTOR_PATTERNPATH_ROTATION_NUM, "45"),
        (p::VECTOR_CONTOUR_STEPS_NUM, "3"),
        (p::VECTOR_CONTOUR_ACCEL_NUM, "2.5"),
    ] {
        let grava = |cartoes: bool| {
            let mut mesa = Mesa::com(cartoes, tudo);
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
    let mesa = Mesa::com(true, sem_nada);
    for id in [
        p::VECTOR_ENVELOPE_PERSPECTIVE,
        p::VECTOR_ENVELOPE_EXPAND,
        p::vector_envelope_preset_id(0),
        p::VECTOR_PATTERNPATH_PICK,
        p::VECTOR_PATTERNPATH_DETACH,
        p::VECTOR_PATTERNPATH_SPACING_NUM,
        p::VECTOR_CONTOUR_STEPS_NUM,
        p::VECTOR_CONTOUR_TO,
        p::VECTOR_CONTOUR_REMOVE,
    ] {
        assert!(mesa.rect(id).is_none(), "{id:?} sem sujeito");
    }

    // Um caminho só: a porta é o Picker, e a auto-ligação some.
    let mut mesa = Mesa::com(true, sem_nada);
    ph2d_panel_vector::set_current_patternpath_can_link(false);
    ph2d_panel_vector::set_current_patternpath_can_pick(true);
    mesa.assenta();
    assert!(mesa.rect(p::VECTOR_PATTERNPATH_PICK).is_some(), "Pick");
    assert!(mesa.rect(p::VECTOR_PATTERNPATH_LINK).is_none(), "Link");

    // Nenhuma porta: o cabeçalho nem sobe.
    ph2d_panel_vector::set_current_patternpath_can_pick(false);
    ph2d_panel_vector::set_current_contour_can_add(false);
    mesa.assenta();
    assert!(mesa.rect(p::VECTOR_SECTION_PATTERNPATH).is_none(), "PP");
    assert!(mesa.rect(p::VECTOR_SECTION_CONTOUR).is_none(), "Contour");

    // Pinos: uma saída, sem preset nem Bend.
    let mesa = Mesa::com(true, pinos);
    assert!(mesa.rect(p::VECTOR_ENVELOPE_CLEAR_PINS).is_some());
    assert!(mesa.rect(p::vector_envelope_preset_id(0)).is_none());
    assert!(mesa.rect(t::VECTOR_ENVELOPE_BEND_NUM).is_none());

    // Sem preset ativo, o Bend não tem o que re-carimbar.
    let mut mesa = Mesa::com(true, tudo);
    assert!(mesa.rect(p::VECTOR_ENVELOPE_CLEAR_PINS).is_none());
    ph2d_panel_vector::set_current_envelope_presets(PRESETS, None, 0.5);
    mesa.assenta();
    assert!(mesa.rect(t::VECTOR_ENVELOPE_BEND_NUM).is_none(), "Bend");
    assert!(mesa.rect(p::vector_envelope_preset_id(0)).is_some());
}

/// O que se pinta muda só com o estado escolhido: a opção acesa sai do DOCUMENTO, e nenhum rect a
/// distingue. *Mutação: acender sempre a 1.ª ⇒ vermelho.*
#[test]
fn the_lit_option_follows_the_document() {
    let pinta = |semente: &dyn Fn()| {
        let mut mesa = Mesa::com(true, tudo);
        semente();
        mesa.assenta();
        let (h, st) = (&mut mesa.host, &mut mesa.st);
        h.paint_draw_data::<ph2d_panel_vector::VectorPanel>(st, super::g4_os_cartoes::COLUNA)
    };
    let contorno = |quina: u8, lado: u8| {
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
    let vinculo = |flip: bool| {
        ph2d_panel_vector::set_current_patternpath(true, 0.25, 0.75, 1.5, 0.5, flip, 30.0);
    };
    let gaiola = |m: u8| ph2d_panel_vector::set_current_envelope_mode(m);
    assert_ne!(
        pinta(&|| contorno(0, 0)),
        pinta(&|| contorno(1, 0)),
        "Corner"
    );
    assert_ne!(pinta(&|| contorno(0, 0)), pinta(&|| contorno(0, 1)), "Side");
    assert_ne!(pinta(&|| vinculo(false)), pinta(&|| vinculo(true)), "Flip");
    assert_ne!(pinta(&|| gaiola(0)), pinta(&|| gaiola(1)), "Cage");
    // Sempre UMA acesa: o tamanho do que se pinta não muda com a escolha. Sem isto, uma opção acesa
    // sempre, ou nunca, continuava a dar pinturas diferentes.
    let n = |semente: &dyn Fn()| pinta(semente).len();
    assert_eq!(n(&|| gaiola(0)), n(&|| gaiola(1)), "Cage: uma acesa");
    assert_eq!(
        n(&|| contorno(0, 0)),
        n(&|| contorno(1, 0)),
        "Corner: uma acesa"
    );
    assert_eq!(
        n(&|| contorno(0, 0)),
        n(&|| contorno(0, 1)),
        "Side: uma acesa"
    );
    assert_eq!(
        n(&|| vinculo(false)),
        n(&|| vinculo(true)),
        "Flip: uma acesa"
    );
}

/// A amostra pinta a cor do último anel, a do documento. *Mutação: cor fixa ⇒ vermelho.*
#[test]
fn the_ring_colour_is_the_document() {
    let pinta = |cor: [u8; 4]| {
        let mut mesa = Mesa::com(true, tudo);
        ph2d_panel_vector::set_current_contour(true, 6.0, 0.125, 2.0, 1, 2, cor);
        mesa.assenta();
        let (h, st) = (&mut mesa.host, &mut mesa.st);
        h.paint_draw_data::<ph2d_panel_vector::VectorPanel>(st, super::g4_os_cartoes::COLUNA)
    };
    assert_ne!(pinta([16, 32, 48, 255]), pinta([200, 32, 48, 255]));
}

/// O Pick Shapes acende com o modo armado: é o aviso de que o próximo clique escolhe.
/// *Mutação: `primary: false` ⇒ vermelho.*
#[test]
fn pick_shapes_lights_while_armed() {
    use ph2d_tool_vector::params::DrawMode;
    let pinta = |modo: DrawMode| {
        let mut mesa = Mesa::com(true, tudo);
        ph2d_panel_vector::set_current_vector_style(Some(ph2d_tool_vector::VectorStyleSnapshot {
            mode: modo,
            ..Default::default()
        }));
        mesa.assenta();
        let (h, st) = (&mut mesa.host, &mut mesa.st);
        h.paint_draw_data::<ph2d_panel_vector::VectorPanel>(st, super::g4_os_cartoes::COLUNA)
    };
    assert_ne!(pinta(DrawMode::PickBlend), pinta(DrawMode::Select));
}

/// O canal fica no campo. *Mutação: tirar um `.key` ⇒ vermelho.*
#[test]
fn each_channel_keeps_its_animation_mark() {
    let marcados = [
        p::VECTOR_BLEND_STEPS,
        p::VECTOR_MORPH_T,
        p::VECTOR_ENVELOPE_BEND,
        p::VECTOR_ENVELOPE_PERSPECTIVE,
        p::VECTOR_PATTERNPATH_SPACING,
        p::VECTOR_PATTERNPATH_SLIDE,
        p::VECTOR_PATTERNPATH_START,
        p::VECTOR_PATTERNPATH_END,
        p::VECTOR_PATTERNPATH_OFFSET,
        p::VECTOR_PATTERNPATH_ROTATION,
        p::VECTOR_PATTERNPATH_FLIP_OFF,
        p::VECTOR_CONTOUR_STEPS,
        p::VECTOR_CONTOUR_OFFSET,
        p::VECTOR_CONTOUR_ACCEL,
        p::VECTOR_CONTOUR_TO,
        p::VECTOR_CONTOUR_JOIN_MITER,
        p::VECTOR_CONTOUR_SIDE_OUTER,
    ];
    let mut mesa = Mesa::com(true, tudo);
    set_key_marks(marcados.iter().map(|id| (*id, KeyMark::Chave)).collect());
    mesa.assenta();
    for id in marcados {
        assert!(mesa.rect(key_mark_id(id)).is_some(), "sem marca: {id:?}");
    }
    set_key_marks(BTreeMap::new());
}

/// O passo de cada campo. *Mutação: tirar uma linha do `prepare` ⇒ vermelho.*
#[test]
fn each_field_has_its_step_and_factory_value() {
    let mesa = Mesa::com(true, tudo);
    let spec = |id: NodeId| {
        mesa.host
            .store()
            .card_field(id)
            .unwrap_or_else(|| panic!("{id:?} sem marca"))
    };
    assert_eq!(
        spec(p::VECTOR_CONTOUR_STEPS_NUM).step,
        1.0,
        "anéis inteiros"
    );
    assert_eq!(spec(p::VECTOR_PATTERNPATH_ROTATION_NUM).step, 1.0, "graus");
    assert_eq!(spec(p::VECTOR_PATTERNPATH_START_NUM).step, 0.01, "fração");
    // A seta anda o passo do `populate`, que não é o do arrasto.
    assert_eq!(spec(p::VECTOR_PATTERNPATH_OFFSET_NUM).step, 0.05);
    assert_eq!(spec(p::VECTOR_PATTERNPATH_OFFSET_NUM).scrub, 0.01);
    assert_eq!(spec(p::VECTOR_CONTOUR_OFFSET_NUM).step, 0.5);
    assert_eq!(spec(p::VECTOR_CONTOUR_OFFSET_NUM).scrub, 0.1);
    assert_eq!(spec(p::VECTOR_CONTOUR_ACCEL_NUM).default, Some(1.0));
    assert_eq!(spec(p::VECTOR_PATTERNPATH_END_NUM).default, Some(1.0));
    assert_eq!(spec(t::VECTOR_ENVELOPE_BEND_NUM).default, Some(0.5));
    assert_eq!(
        spec(p::VECTOR_PATTERNPATH_SLIDE_NUM).default,
        None,
        "derivado"
    );
}

/// ⭐⭐ **Compacto como o Figma** (05 §4). *Mutação: separar um par ⇒ vermelho.*
#[test]
fn siblings_share_the_line_and_nothing_is_glued() {
    let mesa = Mesa::com(true, tudo);
    let r = |id: NodeId| mesa.rect(id).unwrap_or_else(|| panic!("{id:?}"));
    for (a, b) in [
        (t::VECTOR_MODE_PICKBLEND, p::VECTOR_BLEND_RUN),
        (t::VECTOR_BLEND_STEPS_NUM, p::VECTOR_BLEND_RESET_SPINE),
        (p::VECTOR_BLEND_EXPAND, p::VECTOR_BLEND_RELEASE),
        (p::VECTOR_MORPH_RUN, t::VECTOR_MORPH_T_NUM),
        (p::VECTOR_ENVELOPE_EXPAND, p::VECTOR_ENVELOPE_RELEASE),
        (
            p::VECTOR_PATTERNPATH_SPACING_NUM,
            p::VECTOR_PATTERNPATH_SLIDE_NUM,
        ),
        (
            p::VECTOR_PATTERNPATH_START_NUM,
            p::VECTOR_PATTERNPATH_END_NUM,
        ),
        (
            p::VECTOR_PATTERNPATH_OFFSET_NUM,
            p::VECTOR_PATTERNPATH_ROTATION_NUM,
        ),
        (p::VECTOR_CONTOUR_STEPS_NUM, p::VECTOR_CONTOUR_OFFSET_NUM),
        (p::VECTOR_CONTOUR_ACCEL_NUM, p::VECTOR_CONTOUR_TO),
        (p::VECTOR_CONTOUR_EXPAND, p::VECTOR_CONTOUR_REMOVE),
    ] {
        let (ra, rb) = (r(a), r(b));
        assert!(
            // O segmento tem a folga da pílula.
            (ra.y - rb.y).abs() <= 2.0,
            "{a:?} e {b:?} em linhas diferentes"
        );
        assert!(ra.x + ra.w <= rb.x, "{a:?} por cima de {b:?}");
    }
    let (a, b) = (r(p::VECTOR_BLEND_RUN), r(p::VECTOR_BLEND_RELEASE));
    assert!(b.y > a.y + a.h, "linhas do Blend coladas");
    // O preset ímpar fica sozinho com a linha inteira: nenhum botão morto.
    let (par, impar) = (
        r(p::vector_envelope_preset_id(0)),
        r(p::vector_envelope_preset_id(2)),
    );
    assert!(impar.y >= par.y + par.h + m::FIELD_ROW_GAP - 0.5);
    assert!(impar.w > par.w * 1.5, "o preset ímpar não ocupou a linha");
}

/// Rótulo com a unidade, nunca na caixa. *Mutação: tirar `com_unidade` ⇒ vermelho.*
#[test]
fn each_number_says_its_unit_in_the_label() {
    let mesa = Mesa::com(true, tudo);
    let tem = |id: NodeId, nome: &str| {
        let dica = field_hint(mesa.host.store(), id).unwrap_or_else(|| panic!("{id:?} sem dica"));
        assert!(dica.starts_with(nome), "{id:?}: «{dica}» sem «{nome}»");
    };
    tem(p::VECTOR_PATTERNPATH_SPACING_NUM, "Spacing (×)");
    tem(p::VECTOR_PATTERNPATH_OFFSET_NUM, "Offset (m)");
    tem(p::VECTOR_PATTERNPATH_ROTATION_NUM, "Rotation (°)");
    tem(p::VECTOR_CONTOUR_OFFSET_NUM, "Offset (%)");
    tem(t::VECTOR_BLEND_STEPS_NUM, "Steps");
}

/// ⭐ A dobra de cada membro continua a dele, e o grupo é um cartão só.
#[test]
fn each_member_still_folds_on_its_own() {
    let mut mesa = Mesa::com(true, tudo);
    mesa.host.set_collapsed(t::VECTOR_SECTION_BLEND, true);
    mesa.assenta();
    assert!(
        mesa.rect(t::VECTOR_SECTION_BLEND).is_some(),
        "o título fica"
    );
    assert!(mesa.rect(p::VECTOR_BLEND_RUN).is_none(), "o corpo some");
    assert!(mesa.rect(p::VECTOR_MORPH_RUN).is_some(), "o vizinho fica");
}

/// ⭐⭐ **A lei da compactação sobre os estados que a varredura do registo não semeia.**
#[test]
fn the_slice_passes_the_compaction_census() {
    for (nome, semente) in [
        ("sem nada", sem_nada as fn()),
        ("tudo", tudo),
        ("pinos", pinos),
    ] {
        let mut mesa = Mesa::com(true, semente);
        sem_acusacao(nome, || mesa.assenta());
    }
}

/// `PH2D_CARDS=0` pinta o corpo de antes: a barra volta. *Mutação: tirar o `card_body()` ⇒ vermelho.*
#[test]
fn cards_off_paints_the_old_body() {
    let mesa = Mesa::com(false, tudo);
    for id in [
        p::VECTOR_BLEND_STEPS,
        p::VECTOR_MORPH_T,
        p::VECTOR_ENVELOPE_BEND,
        p::VECTOR_PATTERNPATH_SPACING,
        p::VECTOR_CONTOUR_STEPS,
    ] {
        assert!(mesa.rect(id).is_some(), "barra {id:?}");
    }
}
