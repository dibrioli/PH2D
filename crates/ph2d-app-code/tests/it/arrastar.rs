//! Item 8 (`docs/CodeEditor/07_arrastar_para_o_codigo.md`): um arrasto entre painéis sobre a aba de script
//! — o cursor fino onde cairia, o balão das teclas, e a soltura num passo de undo, pelas portas do
//! `DocumentSurface` que a shell chama (`drop_hover` · `drop`).

use crate::Mesa;
use ph2d_editor_core::doc_surface::{DocumentSurface, DropAt, Dropping};
use ph2d_host::Modifiers;
use ph2d_script::ScriptId;

const TEXTO: &str = "function update(self)\n  ph2d.set(, \"x\", 1)\nend\n";

fn mesa() -> (Mesa, ScriptId) {
    let (mut m, ids) = Mesa::new(&[("a.luau", TEXTO)]);
    let id = ids[0];
    m.pede(id);
    m.pinta(id);
    (m, id)
}

/// O ponto do ecrã logo à direita do início do byte `b` (o clique cai nele).
fn ponto(m: &mut Mesa, id: ScriptId, b: usize) -> (f32, f32) {
    let c = m.docs().caret_rect_at(id, b).expect("à vista");
    (c.x + 1.0, c.y + c.h * 0.5)
}

fn em<'a>(p: (f32, f32), names: &'a [String], ctrl: bool) -> DropAt<'a> {
    DropAt {
        x: p.0,
        y: p.1,
        what: Dropping::Objects(names),
        mods: Modifiers {
            ctrl,
            ..Modifiers::default()
        },
    }
}

#[test]
fn por_cima_do_texto_mostra_onde_cai_e_largar_escreve_num_passo_de_undo() {
    let (mut m, id) = mesa();
    let b = TEXTO.find(", \"x\"").expect("agulha");
    let p = ponto(&mut m, id, b);
    let porta = ["Porta".to_owned()];
    assert!(
        m.docs().drop_hover(id.0, Some(em(p, &porta, false))),
        "sobre o texto aceita"
    );
    let (at, balao) = m.docs().drop_mark_of(id).expect("a marca");
    assert_eq!(at, b, "o cursor fino é o byte do clique");
    assert_eq!(balao.len(), 2, "o que larga e o que o Ctrl faz: {balao:?}");
    m.pinta(id);
    assert!(m.docs().drop(id.0, em(p, &porta, false)));
    assert!(m.docs().drop_mark_of(id).is_none(), "largar apaga a marca");
    let esperado = TEXTO.replacen("set(,", "set(ph2d.object(\"Porta\"),", 1);
    assert_eq!(m.docs().text_of(id).as_deref(), Some(esperado.as_str()));
    let fim = b + "ph2d.object(\"Porta\")".len();
    assert_eq!(
        m.docs().selection_of(id),
        [fim..fim],
        "o cursor no fim do que se escreveu"
    );
    assert!(m.docs().undo(id.0, false));
    assert_eq!(
        m.docs().text_of(id).as_deref(),
        Some(TEXTO),
        "um Ctrl+Z desfaz a soltura inteira"
    );
}

#[test]
fn com_ctrl_a_marca_e_a_escrita_vao_para_o_inicio_da_linha() {
    let (mut m, id) = mesa();
    let b = TEXTO.find(", \"x\"").expect("agulha");
    let p = ponto(&mut m, id, b);
    let nomes = ["Porta".to_owned(), "Caixa Grande".to_owned()];
    assert!(m.docs().drop_hover(id.0, Some(em(p, &nomes, true))));
    let linha = TEXTO.find("ph2d.set").expect("agulha");
    assert_eq!(
        m.docs().drop_mark_of(id).map(|d| d.0),
        Some(linha),
        "o Ctrl escreve no início da linha"
    );
    assert!(m.docs().drop(id.0, em(p, &nomes, true)));
    assert_eq!(
        m.docs().text_of(id).as_deref(),
        Some(
            "function update(self)\n  local porta = ph2d.object(\"Porta\")\n  local caixa_grande = ph2d.object(\"Caixa Grande\")\n  ph2d.set(, \"x\", 1)\nend\n"
        )
    );
}

#[test]
fn fora_do_texto_nao_aceita_e_sair_apaga_a_marca() {
    let (mut m, id) = mesa();
    let b = TEXTO.find(", \"x\"").expect("agulha");
    let p = ponto(&mut m, id, b);
    let porta = ["Porta".to_owned()];
    assert!(m.docs().drop_hover(id.0, Some(em(p, &porta, false))));
    assert!(
        m.docs().drop_hover(id.0, None) == false && m.docs().drop_mark_of(id).is_none(),
        "saiu"
    );
    let calha = (1.0, p.1);
    assert!(
        !m.docs().drop_hover(id.0, Some(em(calha, &porta, false))),
        "na calha não"
    );
    assert!(!m.docs().drop(id.0, em(calha, &porta, false)));
    assert_eq!(
        m.docs().text_of(id).as_deref(),
        Some(TEXTO),
        "nada se escreveu"
    );
}
