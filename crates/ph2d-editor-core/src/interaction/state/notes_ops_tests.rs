//! Gates das operações de nota — a lei da permutação (o texto viaja com a nota).

use super::*;
use crate::ids;

fn store() -> WidgetStore {
    let mut s = WidgetStore::with_capacity(64);
    for id in NOTE_TITLE_IDS.iter().chain(NOTE_BODY_IDS.iter()) {
        s.register(
            *id,
            InteractiveState::TextInput {
                state: TextInputState::Normal,
                text: String::new(),
                caret: 0,
                selection_anchor: None,
            },
        );
    }
    s
}

fn escreve(s: &mut WidgetStore, id: NodeId, t: &str) {
    if let Some(InteractiveState::TextInput { text, .. }) = s.get_mut(id) {
        *text = t.to_string();
    }
}

fn le(s: &WidgetStore, id: NodeId) -> String {
    match s.get(id) {
        Some(InteractiveState::TextInput { text, .. }) => text.clone(),
        _ => panic!("sem caixa"),
    }
}

const P: NodeId = ids::INSP_PANEL;
const A: NodeId = ids::INSP_LIVE_TRANSFORM_SECTION;
const B: NodeId = ids::INSP_LIVE_RENDER_SECTION;

/// Três notas com títulos `n0`, `n1`, `n2` — as duas primeiras em `A`, a terceira em `B`.
fn tres() -> WidgetStore {
    let mut s = store();
    for (i, sec) in [(0, A), (1, A), (2, B)] {
        assert_eq!(s.notes_push(P, i as u8, Some(sec)), Some(i));
        escreve(&mut s, NOTE_TITLE_IDS[i], &format!("n{i}"));
        escreve(&mut s, NOTE_BODY_IDS[i], &format!("b{i}"));
    }
    s
}

fn titulos(s: &WidgetStore) -> Vec<String> {
    (0..s.notes_for_panel(P).len())
        .map(|i| le(s, NOTE_TITLE_IDS[i]))
        .collect()
}

/// ⭐⭐ **Apagar a nota de cima faz a de baixo subir COM o texto dela, e a última ranhura fica
/// vazia.** *Mutação: não permutar ⇒ a nota 1 herda o título `n0`.*
#[test]
fn apagar_sobe_as_de_baixo_com_o_texto_delas() {
    let mut s = tres();
    s.note_delete(P, 0);
    assert_eq!(s.notes_for_panel(P).len(), 2);
    assert_eq!(titulos(&s), vec!["n1", "n2"]);
    assert_eq!(le(&s, NOTE_BODY_IDS[1]), "b2");
    assert_eq!(
        le(&s, NOTE_TITLE_IDS[2]),
        "",
        "a ranhura que sobrou guardou texto velho"
    );
    assert_eq!(s.notes_for_panel(P)[0].color_idx, 1);
}

/// ⭐⭐ **Duplicar põe a cópia logo a seguir, com a cor, a secção e o TEXTO do original.**
/// *Mutação: a cópia no fim ⇒ ela sai da secção `A`, que é onde o original está.*
#[test]
fn duplicar_poe_a_copia_logo_a_seguir_com_o_texto() {
    let mut s = tres();
    s.note_set_color(P, 0, 7);
    assert_eq!(s.note_duplicate(P, 0), Some(1));
    assert_eq!(titulos(&s), vec!["n0", "n0", "n1", "n2"]);
    let notas = s.notes_for_panel(P);
    assert_eq!(notas[1].color_idx, 7);
    assert_eq!(notas[1].section, Some(A));
    assert_eq!(notas[3].section, Some(B));
}

/// Com o painel cheio, duplicar não faz nada — e não destrói nenhum texto.
#[test]
fn duplicar_com_o_painel_cheio_nao_faz_nada() {
    let mut s = store();
    for _ in 0..NOTES_PER_PANEL {
        s.notes_push(P, 0, None);
    }
    escreve(&mut s, NOTE_TITLE_IDS[NOTES_PER_PANEL - 1], "ultima");
    assert_eq!(s.note_duplicate(P, 0), None);
    assert_eq!(s.notes_for_panel(P).len(), NOTES_PER_PANEL);
    assert_eq!(le(&s, NOTE_TITLE_IDS[NOTES_PER_PANEL - 1]), "ultima");
}

/// ⭐⭐ **Mover a nota para OUTRA secção muda a âncora e leva o texto.** A nota `n0` vai para o
/// topo de `B` — fica ANTES da `n2`, e a lista ordena-se para isso. *Mutação: não mudar a
/// `section` ⇒ ela continua a pintar-se em `A`.*
#[test]
fn mover_para_outra_seccao_muda_a_ancora_e_leva_o_texto() {
    let mut s = tres();
    s.note_move(P, 0, Some(B), 0);
    let notas = s.notes_for_panel(P);
    assert_eq!(titulos(&s), vec!["n1", "n0", "n2"]);
    assert_eq!(notas[1].section, Some(B));
    assert_eq!(notas[0].section, Some(A));
    assert_eq!(le(&s, NOTE_BODY_IDS[1]), "b0");
}

/// Mover dentro da mesma secção reordena; um `rank` além do fim põe-na a última da secção.
#[test]
fn mover_dentro_da_seccao_reordena() {
    let mut s = tres();
    s.note_move(P, 0, Some(A), 9);
    assert_eq!(titulos(&s), vec!["n1", "n0", "n2"]);
    assert_eq!(s.notes_for_panel(P)[1].section, Some(A));
    // e mover para onde já está não mexe em nada
    s.note_move(P, 1, Some(A), 1);
    assert_eq!(titulos(&s), vec!["n1", "n0", "n2"]);
}

/// ⚠️ Um campo de nota com o teclado larga-o quando as notas se permutam.
#[test]
fn permutar_larga_o_teclado_de_um_campo_de_nota() {
    let mut s = tres();
    s.set_focus(Some(NOTE_TITLE_IDS[1]));
    s.note_delete(P, 0);
    assert_eq!(s.focus_id(), None);
}

/// ⭐ O arrasto de nota só fica ACTIVO depois do limiar — um clique parado na pega não move.
#[test]
fn o_arrasto_de_nota_arma_depois_do_limiar() {
    let mut s = store();
    s.begin_note_drag(P, 0, 10.0, 10.0);
    s.update_note_drag(11.0, 11.0);
    assert!(!s.note_drag().unwrap().active);
    s.update_note_drag(10.0, 10.0 + super::super::TAB_DRAG_THRESHOLD_PX + 1.0);
    assert!(s.note_drag().unwrap().active);
    assert!(s.end_note_drag().is_some());
    assert!(s.note_drag().is_none());
}
