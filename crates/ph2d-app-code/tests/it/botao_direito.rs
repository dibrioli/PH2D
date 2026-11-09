//! Item 14 (CodeEditor 13 §4, Onda C): o botão direito no texto abre um menu no ponto do clique — os
//! consertos do Alt+Enter primeiro, depois Go to Definition, Find References, Rename e Toggle Comment.
//! Conduzido por cliques reais nas caixas pintadas.

use crate::acoes::{TEXTO as TYPO, sem_assistente, texto};
use crate::completar::{T0, quadro_em};
use crate::definicao::{clica_em, cursor};
use crate::{Mesa, centro, ctrl};
use ph2d_editor_core::doc_surface::Key;
use ph2d_host::{Modifiers, PointerButton};
use ph2d_script::ScriptId;

const BOB: &str = "local a = 1\n\nfunction update(self, dt)\n  a = a + dt\nend\n";

/// Um clique com o botão direito no centro do byte `at`.
fn direito(m: &mut Mesa, id: ScriptId, at: usize) {
    let (x, y) = m.docs().glyph_center(id, at).expect("à vista");
    m.clica_com(id, x, y, Modifiers::default(), PointerButton::Secondary);
    quadro_em(m, T0);
    m.pinta(id);
}

fn linhas(m: &mut Mesa) -> Vec<String> {
    m.docs()
        .context_painted()
        .map(|p| p.rows.iter().map(|r| r.0.clone()).collect())
        .unwrap_or_default()
}

fn escolhe(m: &mut Mesa, id: ScriptId, titulo: &str) {
    let r = m
        .docs()
        .context_painted()
        .and_then(|p| p.rows.iter().find(|r| r.0 == titulo).map(|r| r.1))
        .unwrap_or_else(|| panic!("«{titulo}» não está no menu"));
    let (x, y) = centro(r);
    m.clica(id, x, y, Modifiers::default());
    m.pinta(id);
}

/// t1: sobre `self.ampiltude` o 1.º item é «Did you mean amplitude?»; escolhê-lo corrige num passo de
/// undo, e o menu fecha.
#[test]
fn botao_direito_no_campo_errado_comeca_pelo_conserto_e_corrige_num_passo() {
    let (mut m, id) = sem_assistente(TYPO);
    let errado = TYPO.find("ampiltude").unwrap();
    direito(&mut m, id, errado + 3);
    let l = linhas(&mut m);
    assert_eq!(
        l.first().map(String::as_str),
        Some("Did you mean amplitude?")
    );
    for c in [
        "Go to Definition",
        "Find References",
        "Rename Symbol",
        "Toggle Comment",
    ] {
        assert!(l.iter().any(|s| s == c), "{c} em {l:?}");
    }
    escolhe(&mut m, id, "Did you mean amplitude?");
    let certo = TYPO.replace("ampiltude", "amplitude");
    assert_eq!(texto(&mut m, id), certo);
    assert!(m.docs().context_painted().is_none(), "fechou");
    m.tecla(id, Key::Char('z'), ctrl(), None);
    assert_eq!(texto(&mut m, id), TYPO, "um Ctrl+Z desfaz o conserto");
}

/// t2: fora da selecção o cursor vai para o clique; dentro, a selecção fica.
#[test]
fn botao_direito_move_o_cursor_salvo_dentro_da_seleccao() {
    let (mut m, id) = sem_assistente(BOB);
    let de = BOB.find("update").unwrap();
    clica_em(&mut m, id, de, Modifiers::default());
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    clica_em(&mut m, id, de + 5, shift);
    m.pinta(id);
    let sel = m.docs().selection_of(id);
    assert!(!sel[0].is_empty(), "seleccionado: {sel:?}");
    direito(&mut m, id, de + 2);
    assert_eq!(m.docs().selection_of(id), sel, "dentro: a selecção fica");
    m.tecla(id, Key::Escape, Modifiers::default(), None);
    let fora = BOB.find("a + dt").unwrap() + 4;
    direito(&mut m, id, fora);
    let c = cursor(&mut m, id);
    assert!((fora..=fora + 1).contains(&c), "o cursor no clique: {c}");
    assert!(m.docs().selection_of(id)[0].is_empty());
}

/// t3: Esc fecha; um clique fora fecha.
#[test]
fn esc_e_clique_fora_fecham_o_menu() {
    let (mut m, id) = sem_assistente(BOB);
    let at = BOB.find("a + dt").unwrap();
    direito(&mut m, id, at);
    assert!(m.docs().context_painted().is_some(), "abriu");
    m.tecla(id, Key::Escape, Modifiers::default(), None);
    m.pinta(id);
    assert!(m.docs().context_painted().is_none(), "o Esc fechou");
    direito(&mut m, id, at);
    assert!(m.docs().context_painted().is_some(), "reabriu");
    // As setas andam e o rato acende; o clique fora (a 1.ª linha) fecha sem escrever nada.
    m.tecla(id, Key::Down, Modifiers::default(), None);
    clica_em(&mut m, id, 1, Modifiers::default());
    m.pinta(id);
    assert!(m.docs().context_painted().is_none(), "o clique fora fechou");
    assert_eq!(texto(&mut m, id), BOB);
}

/// t4: «Toggle Comment» do menu comenta a linha do clique (o mesmo comando do Cmd+/).
#[test]
fn toggle_comment_do_menu_comenta_a_linha() {
    let (mut m, id) = sem_assistente(BOB);
    direito(&mut m, id, BOB.find("a + dt").unwrap());
    escolhe(&mut m, id, "Toggle Comment");
    assert_eq!(
        texto(&mut m, id),
        BOB.replace("  a = a + dt", "  -- a = a + dt")
    );
    m.tecla(id, Key::Char('z'), ctrl(), None);
    assert_eq!(texto(&mut m, id), BOB);
}

/// t5: sobre um número numa função com `self`, «Make it a property» está entre os consertos (antes
/// do Go to Definition).
#[test]
fn no_numero_make_it_a_property_vem_entre_os_primeiros() {
    let src = "function update(self, dt)\n  ph2d.set(self.id, \"x\", 220 * dt)\nend\n";
    let (mut m, id) = sem_assistente(src);
    direito(&mut m, id, src.find("220").unwrap() + 1);
    let l = linhas(&mut m);
    let fix = l
        .iter()
        .position(|s| s == "Make it a property (Inspector field)")
        .unwrap_or_else(|| panic!("oferecida: {l:?}"));
    let nav = l.iter().position(|s| s == "Go to Definition").unwrap();
    assert!(fix < nav, "{l:?}");
}

/// «Help for <nome>» só aparece num nome com página (a porta do Ctrl+clique), e leva a ela.
#[test]
fn help_for_so_num_nome_com_pagina() {
    let (mut m, id) = sem_assistente(TYPO);
    direito(&mut m, id, TYPO.find("ph2d.set").unwrap() + 6);
    let l = linhas(&mut m);
    assert!(l.iter().any(|s| s == "Help for ph2d.set"), "{l:?}");
    escolhe(&mut m, id, "Help for ph2d.set");
    m.tecla(id, Key::Escape, Modifiers::default(), None);
    direito(&mut m, id, TYPO.find("local d").unwrap() + 7);
    let l = linhas(&mut m);
    assert!(!l.iter().any(|s| s.starts_with("Help for")), "{l:?}");
}
