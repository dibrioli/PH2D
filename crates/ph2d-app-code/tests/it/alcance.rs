//! Acabamento (item 4 da Onda 2, `docs/CodeEditor/03_onda2.md`): com o Ctrl carregado, o nome sob o
//! rato que o Ctrl+clique leva a algum sítio sublinha e o ponteiro vira a mão. O servidor é o FALSO de
//! `assistente.rs`, a responder como o `luau-lsp` 1.70.1 respondeu na sonda
//! `ferramentas/lspprobe_acabamento.py` (`medicoes_onda2/acabamento_bruto.json`): um tipo no balão de
//! quase tudo — até de `if`, de `end` (a assinatura da função) e de um número.
//!
//! ⭐ A lei: **a mão nunca mente** — ela aparece onde, e só onde, o Ctrl+clique faz alguma coisa.

use crate::Mesa;
use crate::assistente::{Partilhado, Plano, com_falso, converge};
use crate::completar::{T0, quadro_em};
use crate::ctrl;
use crate::definicao::{clica_em, cursor};
use crate::dicas::{aponta, ate, nunca};
use ph2d_editor_core::doc_surface::{DocumentSurface, PointerLook};
use ph2d_panel_code::Ink;
use ph2d_script::ScriptId;
use serde_json::{Value, json};
use std::ops::Range;
use std::time::Duration;

const TEXTO: &str = "ph2d.property(\"amplitude\", 1.0)\n\
local brilho = 1\n\
function update(self, dt)\n  \
local d = brilho * self.amplitude\n  \
if d > 0 and self.ampiltude then\n    \
pirnt(d)\n  \
end\n  \
ph2d.set(self.id, \"x\", d) -- brilho\n\
end\n";

fn e_palavra(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn palavra(texto: &str, b: usize) -> (usize, &str) {
    let ini = texto[..b].rfind(|c| !e_palavra(c)).map_or(0, |i| i + 1);
    let fim = texto[b..]
        .find(|c| !e_palavra(c))
        .map_or(texto.len(), |i| b + i);
    (ini, &texto[ini..fim])
}

/// Como o `luau-lsp`: um nome com `local <nome>` no script ⇒ a declaração; o resto ⇒ `[]`.
fn define(texto: &str, b: usize) -> Vec<Range<usize>> {
    let (ini, nome) = palavra(texto, b);
    if nome.is_empty() || texto[..ini].ends_with("self.") || texto[..ini].ends_with("ph2d.") {
        return Vec::new();
    }
    texto
        .find(&format!("local {nome} "))
        .map(|i| i + 6..i + 6 + nome.len())
        .into_iter()
        .collect()
}

/// Como o `luau-lsp` 1.70.1 na sonda: a API documentada; um tipo solto no campo do `self`; o erro
/// num global que não existe; a assinatura da função numa palavra reservada; `number` num número.
fn pairar(texto: &str, b: usize) -> Value {
    let (ini, nome) = palavra(texto, b);
    let antes = &texto[..ini];
    let luau = |s: &str| json!({ "contents": { "kind": "markdown", "value": format!("```luau\n{s}\n```") } });
    match nome {
        "" => Value::Null,
        "local" => Value::Null,
        "if" | "then" | "end" | "function" => luau("function update<T>(self: { ampiltude: T }, dt: number): ()"),
        "and" => luau("T | false"),
        "pirnt" => luau("type pirnt = *error-type*"),
        "ph2d" => luau("type ph2d = {\n    set: (entity: number) -> ()\n}"),
        "ampiltude" => luau("T"),
        _ if nome.starts_with(|c: char| c.is_ascii_digit()) => luau("number"),
        _ if antes.ends_with("ph2d.") => json!({ "contents": { "kind": "markdown", "value":
            format!("```luau\nfunction ph2d.{nome}(entity: number): ()\n```\n----------\nThe {nome} doc.") } }),
        _ if antes.ends_with("self.") => luau("number"),
        _ => luau(&format!("local {nome}: number")),
    }
}

fn mesa() -> (Mesa, ScriptId, Partilhado) {
    let (mut m, ids, visto) = com_falso(
        &[("a.luau", TEXTO)],
        Plano {
            atraso: Duration::ZERO,
            pairar: Some(pairar),
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

fn ctrl_carregado(m: &mut Mesa, sim: bool) {
    m.hero.store.set_cmd_held(sim);
}

fn olhar(m: &mut Mesa, id: ScriptId, byte: usize) -> Option<PointerLook> {
    let (x, y) = m.docs().glyph_center(id, byte).expect("à vista");
    m.docs().pointer_look(id.0, x, y)
}

fn sublinhado(m: &mut Mesa, id: ScriptId) -> Option<Range<usize>> {
    m.docs().link_of(id).0
}

fn definicoes(v: &Partilhado) -> usize {
    v.lock().unwrap().definicoes
}

/// O byte da 1.ª ocorrência de `agulha`, deslocado `off`.
fn em(agulha: &str, off: usize) -> usize {
    TEXTO.find(agulha).unwrap_or_else(|| panic!("{agulha:?}")) + off
}

/// Cada espécie de palavra: `(o que é, byte, a mão aparece)`.
fn casos() -> Vec<(&'static str, usize, bool)> {
    vec![
        ("local usado (sítio no script)", em("= brilho", 3), true),
        ("self.<propriedade> declarada", em("self.amplitude", 7), true),
        ("função da API documentada", em("ph2d.set", 6), true),
        ("tabela da API", em("ph2d.set", 1), true),
        ("campo do motor: só o tipo", em("self.id", 6), false),
        ("campo que não existe: só o tipo", em("self.ampiltude", 7), false),
        ("global que não existe: *error-type*", em("pirnt", 1), false),
        ("palavra reservada if", em("if d", 0), false),
        ("palavra reservada then", em("then", 1), false),
        ("palavra reservada and", em("and self", 1), false),
        ("número", em("> 0", 2), false),
        ("nome num comentário", em("-- brilho", 4), false),
        ("texto entre aspas", em("\"x\"", 1), false),
    ]
}

/// ⭐⭐ A mão nunca mente: em cada espécie de palavra, com o Ctrl carregado, a mão e o sublinhado
/// aparecem se, e só se, o Ctrl+clique ali salta o cursor ou abre a documentação. Palavras reservadas,
/// números, comentários e aspas nem se perguntam ao assistente.
#[test]
fn a_mao_aparece_onde_o_ctrl_clique_faz_alguma_coisa() {
    for (o_que, b, mao) in casos() {
        let (mut m, id, visto) = mesa();
        ctrl_carregado(&mut m, true);
        aponta(&mut m, id, b);
        if mao {
            ate(&mut m, id, T0, &format!("{o_que}: a mão não apareceu"), |m| {
                olhar(m, id, b) == Some(PointerLook::Hand)
                    && sublinhado(m, id).is_some_and(|w| w.contains(&b))
                    && m.docs().link_of(id).1
            });
        } else {
            nunca(&mut m, id, T0, &format!("{o_que}: a mão apareceu"), |m| {
                olhar(m, id, b) == Some(PointerLook::Text) && sublinhado(m, id).is_none()
            });
        }
        let pedidos = definicoes(&visto);
        if o_que.starts_with("palavra reservada")
            || o_que == "número"
            || o_que.contains("comentário")
            || o_que.contains("aspas")
        {
            assert_eq!(pedidos, 0, "{o_que}: perguntou ao assistente");
        }
        // O Ctrl+clique no mesmo sítio faz alguma coisa se, e só se, havia a mão.
        ctrl_carregado(&mut m, false);
        clica_em(&mut m, id, b, ctrl());
        // O centro de uma letra põe o cursor de um lado ou do outro dela: isso não é um salto.
        let fez =
            |m: &mut Mesa| !(b..=b + 1).contains(&cursor(m, id)) || m.docs().hover_of(id).is_some();
        if mao {
            ate(&mut m, id, T0, &format!("{o_que}: a mão prometeu e o clique não fez nada"), fez);
        } else {
            nunca(&mut m, id, T0, &format!("{o_que}: o clique fez algo sem a mão"), |m| !fez(m));
        }
    }
}

/// Sem o Ctrl: o I sobre o texto, nada sublinhado e nenhuma pergunta. Carregar no Ctrl com o rato
/// PARADO sublinha (não é preciso mexer o rato); largá-lo volta ao I. Voltar a carregar sobre o mesmo
/// nome não pergunta outra vez.
#[test]
fn o_ctrl_com_o_rato_parado_sublinha_e_largar_desfaz() {
    let (mut m, id, visto) = mesa();
    let b = em("= brilho", 3);
    aponta(&mut m, id, b);
    nunca(&mut m, id, T0, "sem o Ctrl houve mão ou pergunta", |m| {
        olhar(m, id, b) == Some(PointerLook::Text) && sublinhado(m, id).is_none()
    });
    assert_eq!(definicoes(&visto), 0);
    let antes = m.docs().ink_painted(id, b);
    assert_ne!(antes, Ink::Link);
    ctrl_carregado(&mut m, true);
    ate(&mut m, id, T0, "o Ctrl com o rato parado não sublinhou", |m| {
        olhar(m, id, b) == Some(PointerLook::Hand)
    });
    // O nome inteiro na cor do link (o VS Code pinta o texto da cor do traço), os vizinhos não.
    let w = sublinhado(&mut m, id).expect("sublinhado");
    assert!(w.clone().all(|k| m.docs().ink_painted(id, k) == Ink::Link));
    assert_ne!(m.docs().ink_painted(id, w.end), Ink::Link);
    ctrl_carregado(&mut m, false);
    quadro_em(&mut m, T0);
    m.pinta(id);
    assert_eq!(olhar(&mut m, id, b), Some(PointerLook::Text));
    assert_eq!(sublinhado(&mut m, id), None);
    assert!(!m.docs().link_of(id).1, "largar o Ctrl deixou o sublinhado pintado");
    assert_eq!(m.docs().ink_painted(id, b), antes, "largar o Ctrl não devolveu a cor");
    ctrl_carregado(&mut m, true);
    ate(&mut m, id, T0, "voltar ao Ctrl não sublinhou", |m| {
        sublinhado(m, id).is_some()
    });
    assert_eq!(definicoes(&visto), 1, "o mesmo nome perguntou-se outra vez");
}

/// Uma edição muda a revisão: o veredito velho não vale (o nome pode ter deixado de existir) até a
/// resposta nova chegar.
#[test]
fn uma_edicao_pergunta_outra_vez() {
    let (mut m, id, visto) = mesa();
    let b = em("= brilho", 3);
    ctrl_carregado(&mut m, true);
    aponta(&mut m, id, b);
    ate(&mut m, id, T0, "não sublinhou", |m| sublinhado(m, id).is_some());
    m.docs().go_to(id.0, TEXTO.lines().count() - 1, 3);
    m.escreve(id, " ");
    ate(&mut m, id, T0, "a edição não perguntou outra vez", |m| {
        definicoes(&visto) >= 2 && sublinhado(m, id).is_some()
    });
}

/// O ponteiro fora da aba não é dela; sobre a calha é a seta.
#[test]
fn fora_do_texto_nao_ha_mao() {
    let (mut m, id, _) = mesa();
    ctrl_carregado(&mut m, true);
    assert_eq!(m.docs().pointer_look(id.0, -10.0, -10.0), None);
    let (_, y) = m.docs().glyph_center(id, em("= brilho", 3)).unwrap();
    assert_eq!(
        m.docs().pointer_look(id.0, crate::AREA.x + 2.0, y),
        Some(PointerLook::Arrow),
        "a calha"
    );
}
