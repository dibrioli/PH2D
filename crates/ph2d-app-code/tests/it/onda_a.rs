//! Onda A (`docs/CodeEditor/13_plano_bancada.md` §4, itens 3–4) pelo caminho real da tecla e da
//! pintura: Ctrl+/ comenta, Ctrl+Shift+[ / ] dobra, e a lista diz quantos objectos usam cada script.

use crate::{Mesa, centro, ctrl};
use ph2d_editor_core::doc_surface::Key;
use ph2d_host::Modifiers;
use ph2d_script::ScriptId;

const BOB: &str = include_str!("../../../ph2d-code-syntax/tests/it/fixtures/bob.luau");

fn aberto(texto: &str) -> (Mesa, ScriptId) {
    let (mut m, ids) = Mesa::new(&[("a.luau", texto)]);
    m.pede(ids[0]);
    m.pinta(ids[0]);
    (m, ids[0])
}

fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Modifiers::default()
    }
}

fn ctrl_shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..ctrl()
    }
}

fn texto(m: &mut Mesa, id: ScriptId) -> String {
    m.docs().text_of(id).expect("aberto")
}

/// ⭐ Três linhas e uma vazia seleccionadas: Ctrl+/ comenta-as na MENOR indentação (a vazia fica);
/// um segundo Ctrl+/ devolve o texto exacto; e um Ctrl+Z desfaz um toggle inteiro.
#[test]
fn ctrl_barra_comenta_na_menor_indentacao_e_volta_ao_texto_exacto() {
    let original = "  local a = 1\n    local b = 2\n\n  local c = 3\nreturn a\n";
    let (mut m, id) = aberto(original);
    for _ in 0..3 {
        m.tecla(id, Key::Down, shift(), None);
    }
    m.tecla(id, Key::End, shift(), None);
    m.tecla(id, Key::Char('/'), ctrl(), None);
    let comentado = "  -- local a = 1\n  --   local b = 2\n\n  -- local c = 3\nreturn a\n";
    assert_eq!(texto(&mut m, id), comentado);

    m.tecla(id, Key::Char('/'), ctrl(), None);
    assert_eq!(
        texto(&mut m, id),
        original,
        "o segundo Ctrl+/ devolve o texto exacto"
    );

    m.tecla(id, Key::Char('/'), ctrl(), None);
    assert_eq!(texto(&mut m, id), comentado);
    m.tecla(id, Key::Char('z'), ctrl(), None);
    assert_eq!(
        texto(&mut m, id),
        original,
        "um Ctrl+Z desfaz o toggle inteiro"
    );
}

/// Uma linha comentada e outra não: Ctrl+/ comenta TODAS (o idioma do VS Code e do Godot).
#[test]
fn ctrl_barra_com_linhas_mistas_comenta_todas() {
    let (mut m, id) = aberto("-- x\ny\n");
    m.tecla(id, Key::Char('a'), ctrl(), None);
    m.tecla(id, Key::Char('/'), ctrl(), None);
    assert_eq!(texto(&mut m, id), "-- -- x\n-- y\n");
    // Sem selecção: só a linha do cursor.
    let (mut m, id) = aberto("x\ny\n");
    m.tecla(id, Key::Char('/'), ctrl(), None);
    assert_eq!(texto(&mut m, id), "-- x\ny\n");
}

/// Ctrl+Shift+[ dobra o bloco mais fundo que contém o cursor (e o cursor sobe ao cabeçalho);
/// Ctrl+Shift+] desdobra-o.
#[test]
fn ctrl_shift_parenteses_rectos_dobram_e_desdobram_o_bloco_do_cursor() {
    let (mut m, id) = aberto(BOB);
    // A linha 26 (1-based) = 25, dentro de init (23..28).
    m.tecla(id, Key::Char('g'), ctrl(), None);
    m.escreve(id, "26\n");
    m.pinta(id);
    m.tecla(id, Key::Char('['), ctrl_shift(), None);
    assert_eq!(m.docs().closed_folds(id), vec![(23, 28)]);
    assert_eq!(m.docs().cursor_of(id).map(|c| c.0), Some(23));
    m.tecla(id, Key::Char(']'), ctrl_shift(), None);
    assert_eq!(m.docs().closed_folds(id), Vec::<(usize, usize)>::new());

    // A linha 38 (1-based) = 37: dentro de update, do if e do elseif — dobra o mais fundo.
    m.tecla(id, Key::Char('g'), ctrl(), None);
    m.escreve(id, "38\n");
    m.tecla(id, Key::Char('['), ctrl_shift(), None);
    assert_eq!(m.docs().closed_folds(id), vec![(37, 38)]);
    // Sem o Shift, Ctrl+[ não dobra nada.
    m.tecla(id, Key::Char('['), ctrl(), None);
    assert_eq!(m.docs().closed_folds(id), vec![(37, 38)]);
}

/// ⭐ Alt+F (o «Toggle Fold Line» do Godot, a tecla igual em qualquer teclado): dobra o bloco do
/// cursor e, com o cursor no cabeçalho dobrado, desdobra-o.
#[test]
fn alt_f_dobra_e_desdobra_o_bloco_do_cursor() {
    let (mut m, id) = aberto(BOB);
    m.tecla(id, Key::Char('g'), ctrl(), None);
    m.escreve(id, "26\n");
    m.pinta(id);
    let alt = Modifiers {
        alt: true,
        ..Modifiers::default()
    };
    m.tecla(id, Key::Char('f'), alt, None);
    assert_eq!(m.docs().closed_folds(id), vec![(23, 28)]);
    m.tecla(id, Key::Char('f'), alt, None);
    assert_eq!(m.docs().closed_folds(id), Vec::<(usize, usize)>::new());
}

/// ⭐ A lista de scripts pinta «2» ao lado do script que dois objectos correm e «unused» ao lado do
/// que ninguém corre; o rato sobre a contagem dá a frase.
#[test]
fn a_lista_diz_quantos_objectos_usam_cada_script() {
    let (m, ids) = Mesa::new(&[
        ("hero.luau", "local x = 1\n"),
        ("door.luau", "local y = 2\n"),
    ]);
    let mut m = m.com_cena(ids[0], 2);
    m.pede(ids[0]);
    m.pinta(ids[0]);
    let counts = m.docs().list_counts();
    let texts: Vec<(ScriptId, String)> = counts.iter().map(|(i, t, _)| (*i, t.clone())).collect();
    assert_eq!(
        texts,
        vec![(ids[0], "2".to_owned()), (ids[1], "unused".to_owned())]
    );

    let (x, y) = centro(counts[0].2);
    m.rato(ids[0], x, y);
    m.pinta(ids[0]);
    assert_eq!(m.docs().list_count_tip(), Some("Used by 2 objects"));
    let (x, y) = centro(counts[1].2);
    m.rato(ids[0], x, y);
    m.pinta(ids[0]);
    assert_eq!(
        m.docs().list_count_tip(),
        Some("No object uses this script")
    );
}
