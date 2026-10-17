//! I6 (Onda 2, `docs/CodeEditor/03_onda2.md`): ir para a definição — Ctrl+clique e F12. O servidor é o
//! FALSO de `assistente.rs`, a responder como o `luau-lsp` 1.70.1 respondeu na sonda do I6
//! (`ferramentas/lspprobe_i6_i11.py`): num local, a declaração; na API e em `self.<propriedade>`, `[]`.
//!
//! ⛔ Toda asserção de «NÃO salta / NÃO pergunta» dá tempo REAL a uma resposta chegar ([`nunca`]).

use crate::Mesa;
use crate::assistente::{Partilhado, Plano, com_falso, converge, letra, tecla};
use crate::completar::{T0, quadro_em};
use crate::dicas::{ate, balao, nunca};
use crate::ctrl;
use ph2d_editor_core::doc_surface::Key;
use ph2d_host::Modifiers;
use ph2d_script::ScriptId;
use std::ops::Range;
use std::time::Duration;

const TEXTO: &str = "ph2d.property(\"amplitude\", 1.0)\n\
local brilho = 1\n\
function update(self, dt)\n  \
local d = brilho * self.amplitude\n  \
ph2d.set(self.id, \"x\", d) -- brilho\n\
end\n";

fn e_palavra(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Como o `luau-lsp`: um nome com `local <nome>` no script ⇒ a declaração; o resto (a API,
/// `self.<campo>`) ⇒ `[]`.
fn define(texto: &str, b: usize) -> Vec<Range<usize>> {
    let ini = texto[..b].rfind(|c| !e_palavra(c)).map_or(0, |i| i + 1);
    let fim = texto[b..]
        .find(|c| !e_palavra(c))
        .map_or(texto.len(), |i| b + i);
    let nome = &texto[ini..fim];
    if nome.is_empty() || texto[..ini].ends_with("self.") || texto[..ini].ends_with("ph2d.") {
        return Vec::new();
    }
    texto
        .find(&format!("local {nome} "))
        .map(|i| i + 6..i + 6 + nome.len())
        .into_iter()
        .collect()
}

fn mesa(atraso: Duration) -> (Mesa, ScriptId, Partilhado) {
    let (mut m, ids, visto) = com_falso(
        &[("a.luau", TEXTO)],
        Plano {
            atraso,
            pairar: Some(balao),
            definir: Some(define),
            ..Plano::default()
        },
    );
    let id = ids[0];
    m.pede(id);
    converge(&mut m, &visto, "abrir");
    quadro_em(&mut m, T0);
    m.pinta(id);
    (m, id, visto)
}

fn clica_em(m: &mut Mesa, id: ScriptId, byte: usize, mods: Modifiers) {
    let (x, y) = m
        .docs()
        .glyph_center(id, byte)
        .unwrap_or_else(|| panic!("o byte {byte} não está à vista"));
    m.clica(id, x, y, mods);
}

fn cursor(m: &mut Mesa, id: ScriptId) -> usize {
    m.docs().cursor_of(id).expect("aberto").1
}

fn definicoes(v: &Partilhado) -> usize {
    v.lock().unwrap().definicoes
}

/// ⭐ Ctrl+clique num uso de um local salta para a declaração dele. Controlo: o mesmo clique sem Ctrl
/// põe o cursor ali e não pergunta nada.
#[test]
fn ctrl_clique_num_local_vai_para_a_declaracao() {
    let (mut m, id, visto) = mesa(Duration::ZERO);
    let uso = TEXTO.find("= brilho").unwrap() + 2;
    let decl = TEXTO.find("local brilho").unwrap() + 6;
    clica_em(&mut m, id, uso + 1, Modifiers::default());
    nunca(&mut m, id, T0, "um clique sem Ctrl perguntou ou saltou", |m| {
        // O centro de uma letra põe o cursor de um lado ou do outro dela.
        definicoes(&visto) == 0 && (uso + 1..=uso + 2).contains(&cursor(m, id))
    });
    // Noutro byte (o mesmo seria um duplo clique: a mesa clica sempre no mesmo instante).
    clica_em(&mut m, id, uso + 3, ctrl());
    ate(&mut m, id, T0, "o Ctrl+clique não saltou", |m| cursor(m, id) == decl);
    assert_eq!(definicoes(&visto), 1);
    assert_eq!(m.docs().hover_of(id), None, "um sítio no script não abre o balão");
}

/// ⭐ F12 numa função da API (o servidor dá `[]`): abre a documentação no balão, já e sem mexer no
/// cursor; o rato a passar não a leva; uma tecla fecha-a.
#[test]
fn f12_na_api_abre_a_documentacao_e_o_cursor_fica() {
    let (mut m, id, visto) = mesa(Duration::ZERO);
    let set = TEXTO.find("ph2d.set").unwrap() + 6;
    clica_em(&mut m, id, set, Modifiers::default());
    tecla(&mut m, id, Key::F(12), Modifiers::default(), None, "");
    ate(&mut m, id, T0, "o F12 não abriu a documentação", |m| {
        m.docs().hover_of(id).is_some()
    });
    assert!(m.docs().hover_of(id).unwrap().contains("local set"), "o balão é o de `set`");
    assert_eq!(cursor(&mut m, id), set, "o cursor não sai do sítio");
    assert_eq!(definicoes(&visto), 1);
    let (pintado, _) = m.docs().hints_painted(id);
    assert!(pintado.is_some(), "o balão pintou-se");
    // O rato noutro nome: o balão aberto pelo comando fica.
    let outro = TEXTO.find("update").unwrap();
    crate::dicas::aponta(&mut m, id, outro + 2);
    nunca(&mut m, id, T0, "o rato levou o balão do F12", |m| {
        m.docs().hover_of(id).is_some_and(|h| h.contains("local set"))
    });
    tecla(&mut m, id, Key::Escape, Modifiers::default(), None, "");
    assert_eq!(m.docs().hover_of(id), None, "uma tecla fecha-o");
}

/// ⭐ `self.amplitude` (o servidor dá `[]`): a declaração é a NOSSA, o `ph2d.property("amplitude", …)`.
#[test]
fn ctrl_clique_numa_propriedade_vai_para_a_ph2d_property() {
    let (mut m, id, _visto) = mesa(Duration::ZERO);
    let uso = TEXTO.find("self.amplitude").unwrap() + 5;
    let decl = TEXTO.find("amplitude").unwrap();
    clica_em(&mut m, id, uso + 3, ctrl());
    ate(&mut m, id, T0, "não foi à ph2d.property", |m| cursor(m, id) == decl);
    assert_eq!(m.docs().hover_of(id), None);
}

/// Uma tecla antes da resposta larga o pedido: a resposta (para um texto que já não há) não salta.
#[test]
fn uma_tecla_antes_da_resposta_larga_o_salto() {
    let (mut m, id, visto) = mesa(Duration::from_millis(40));
    let uso = TEXTO.find("= brilho").unwrap() + 2;
    clica_em(&mut m, id, uso + 2, ctrl());
    letra(&mut m, id, 'z');
    let depois = uso + 3;
    for _ in 0..40 {
        quadro_em(&mut m, T0);
        assert_eq!(cursor(&mut m, id), depois, "a resposta velha saltou");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(definicoes(&visto) <= 1);
}

/// Num comentário (ou numa string) o Ctrl+clique é um clique: não pergunta.
#[test]
fn num_comentario_o_ctrl_clique_nao_pergunta() {
    let (mut m, id, visto) = mesa(Duration::ZERO);
    let com = TEXTO.find("-- brilho").unwrap() + 5;
    clica_em(&mut m, id, com, ctrl());
    nunca(&mut m, id, T0, "perguntou dentro de um comentário", |m| {
        definicoes(&visto) == 0
            && (com..=com + 1).contains(&cursor(m, id))
            && m.docs().hover_of(id).is_none()
    });
}

/// ⭐ A definição e as cores são géneros DIFERENTES na sessão: um Ctrl+clique com a pergunta das cores
/// ainda em voo (servidor lento) salta E as cores chegam — um nunca cancela o outro.
#[test]
fn a_definicao_e_as_cores_em_voo_nao_se_cancelam() {
    let (mut m, ids, visto) = com_falso(
        &[("a.luau", TEXTO)],
        Plano {
            atraso: Duration::from_millis(40),
            pairar: Some(balao),
            definir: Some(define),
            colorir: Some(crate::cores::colore),
            ..Plano::default()
        },
    );
    let id = ids[0];
    m.pede(id);
    converge(&mut m, &visto, "abrir");
    // O quadro que pergunta as cores; o clique logo a seguir, com elas em voo.
    quadro_em(&mut m, T0);
    m.pinta(id);
    let uso = TEXTO.find("= brilho").unwrap() + 2;
    let decl = TEXTO.find("local brilho").unwrap() + 6;
    clica_em(&mut m, id, uso + 3, ctrl());
    ate(&mut m, id, T0, "o Ctrl+clique não saltou", |m| cursor(m, id) == decl);
    ate(&mut m, id, T0, "as cores em voo perderam-se", |m| {
        m.docs().semantic_count(id) > 0
    });
}
