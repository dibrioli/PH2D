//! Gates de `source.text`.
//!
//! ⚠️ Este crate é uma FOLHA: ele não alcança a fonte nem a biblioteca de vetor, e
//! por isso não há aqui um gate de *"a letra saiu bonita"*. O que ele PODE afirmar
//! é o contrato que a divergência do shell atacaria: a chave, as portas únicas, e
//! o que o nó faz quando ninguém publicou nada. O layout é gateado do lado do
//! shell, onde a fonte existe.

use super::*;
use ph2d_node_registry::NodeRegistry;

fn defaults(name: &str) -> f32 {
    MANIFEST
        .params
        .iter()
        .find(|p| p.name == name)
        .map(|p| p.default)
        .unwrap_or_else(|| panic!("param {name} existe no manifesto"))
}

/// **Todo param da chave está no manifesto, e todo param do manifesto está na
/// chave.** É a metade executável da lei que o doc do `param::ALL` enuncia: um
/// param que o manifesto declara e a chave não vê fica **inerte depois da primeira
/// vez** — o artista mexe, o cache devolve a geometria velha, e nada acusa.
#[test]
fn the_key_and_the_manifest_carry_the_same_params() {
    let from_manifest: Vec<&str> = MANIFEST.params.iter().map(|p| p.name).collect();
    let mut a = from_manifest.clone();
    let mut b: Vec<&str> = param::ALL.to_vec();
    a.sort_unstable();
    b.sort_unstable();
    assert_eq!(a, b, "a chave e o manifesto listam os mesmos params");
}

/// A chave MUDA quando qualquer entrada muda — os seis params, a fonte e o texto.
/// Uma entrada que não mexe a chave é um controle morto com cara de vivo.
#[test]
fn every_input_moves_the_key() {
    let base = text_key(defaults, "", "Text");
    for name in param::ALL {
        let bumped = text_key(
            |n| {
                if n == *name {
                    defaults(n) + 1.0
                } else {
                    defaults(n)
                }
            },
            "",
            "Text",
        );
        assert_ne!(bumped, base, "o param `{name}` tem de mexer a chave");
    }
    assert_ne!(text_key(defaults, "Inter", "Text"), base, "a fonte");
    assert_ne!(text_key(defaults, "", "Texu"), base, "o texto");
}

/// ⚠️ **A ambiguidade que o comprimento da fonte existe para matar.** Sem o
/// prefixo, `font="a:b" text="c"` e `font="a" text="b:c"` mintariam a mesma chave
/// — dois blocos diferentes a partilhar uma publicação, e o sintoma seria um texto
/// a desenhar o outro, sem erro nenhum.
#[test]
fn a_colon_in_the_font_name_cannot_forge_another_blocks_key() {
    let a = text_key(defaults, "a:b", "c");
    let b = text_key(defaults, "a", "b:c");
    assert_ne!(a, b, "o comprimento da fonte desambigua a concatenacao");
}

/// **O texto de fábrica é SEMEADO, nunca inferido.** O nó o declara ao registry
/// (`register_text_defaults`) e o editor o escreve no grafo ao soltar; o `eval`
/// resolve ausente como VAZIO. ⚠️ Sem isto o canvas desenharia `"Text"` sobre um
/// campo de painel vazio — duas respostas à mesma pergunta, lado a lado.
#[test]
fn the_factory_text_is_seeded_not_inferred() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).expect("registra");
    assert_eq!(
        reg.text_defaults(MANIFEST.id),
        &[(TEXT_KEY, DEFAULT_TEXT)],
        "o no declara o texto de fabrica"
    );
    // ...e o `eval` NÃO o repete: ausente é vazio.
    assert_eq!(text_of(None), "");
    assert_eq!(text_of(Some("")), "");
    assert_eq!(text_of(Some("oi")), "oi");
    // A fonte não tem texto de fábrica: vazio É a embutida.
    assert!(
        !reg.text_defaults(MANIFEST.id)
            .iter()
            .any(|(k, _)| *k == FONT_KEY),
        "semear uma familia seria escolher a tipografia pelo artista"
    );
    assert_eq!(font_of(None), "");
    assert_eq!(font_of(Some("Inter")), "Inter");
}

/// Os índices que um grafo salvo guarda. ⚠️ **APPEND ONLY** — mover um renomeia a
/// escolha de todo documento já autorado, em silêncio.
#[test]
fn the_stored_indices_do_not_move() {
    assert_eq!(Align::from_index(0.0), Align::Left);
    assert_eq!(Align::from_index(1.0), Align::Center);
    assert_eq!(Align::from_index(2.0), Align::Right);
    assert_eq!(Pivot::from_index(0.0), Pivot::Pen);
    assert_eq!(Pivot::from_index(1.0), Pivot::Center);
    // E o default do manifesto é o Center, que é o que faz a rotação por
    // caractere ler certo sem ninguém tocar num controle.
    assert_eq!(Pivot::from_index(defaults(param::PIVOT)), Pivot::Center);
}

/// O nó registra e declara-se **fonte de vetor VIVO**. ⚠️ Sem esta linha o
/// documento não recusa o cook da GPU e as letras saem como **quadrados brancos**
/// do atlas — o modo de falha que o ADR-0154 nomeia, e que nenhum gate de unidade
/// deste crate veria sem o perguntar aqui.
#[test]
fn the_node_declares_itself_a_live_vector_source() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).expect("registra");
    assert!(
        reg.is_live_vector_source(MANIFEST.id),
        "sem isto o cook GPU desenha quadrados brancos"
    );
}

/// Cada param tem uma row, e cada row nomeia um param que existe — incluindo os
/// dois TEXT params, que não estão no manifesto e ainda assim têm de ser
/// desenhados (a 4ª lei do doc 88: *todo param é desenhado*).
#[test]
fn every_param_is_painted_including_the_two_strings() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).expect("registra");
    let hints = reg.param_ui(MANIFEST.id).expect("tem hints");
    for p in MANIFEST.params {
        assert!(
            hints.iter().any(|h| h.param == p.name),
            "o param `{}` tem de ser desenhado",
            p.name
        );
    }
    for k in [TEXT_KEY, FONT_KEY] {
        assert!(
            hints
                .iter()
                .any(|h| h.param == k && matches!(h.widget, ParamWidget::Text)),
            "`{k}` e' um campo de TEXTO"
        );
    }
}
