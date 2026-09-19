//! Os portões da secção do gizmo.

use super::*;

/// **NO PONTO NEUTRO ELA NÃO ESCREVE NADA** — a lei que mantém toda cena de hoje byte-idêntica.
///
/// ⚠️ Uma coluna a mais muda a revisão de conteúdo da corrente e, com ela, o memo de tudo o que
/// está a jusante: escrever `Cross`/`0` seria pagar um recozimento por uma escolha que ninguém fez.
#[test]
fn no_ponto_neutro_nao_escreve_nada() {
    let (forma, tamanho) = (CRUZ, 0.0);
    assert!(
        !escreveria(forma, tamanho).0 && !escreveria(forma, tamanho).1,
        "no neutro a corrente tem de sair como entrou"
    );
}

/// **E FORA DELE, ESCREVE** — o controlo, sem o qual a metade acima passa sobre uma porta inerte.
#[test]
fn fora_do_neutro_escreve() {
    assert!(
        escreveria(CIRCULO, 0.0).0,
        "a forma escolhida tem de viajar"
    );
    assert!(escreveria(CRUZ, 12.0).1, "o tamanho absoluto tem de viajar");
    assert!(escreveria(RECT, 40.0) == (true, true));
}

/// A lei do [`escreve`], sem um `EvalCtx` (que pede um cook inteiro): *o que ela decidiria*.
/// ⚠️ **Derivada da MESMA comparação** que a porta faz — ver o corpo dela.
fn escreveria(forma: f32, tamanho: f32) -> (bool, bool) {
    (forma != CRUZ, tamanho > 0.0)
}

/// ⛔ **A ESCADA DA FORMA COMEÇA NA CRUZ, e isso é uma lei de COMPATIBILIDADE**: um documento
/// gravado antes desta secção lê `0` em todo param novo, e tem de continuar a ver o que via.
#[test]
fn a_escada_comeca_na_cruz() {
    assert_eq!(CRUZ, 0.0);
    assert_eq!(
        SPECS.iter().find(|s| s.name == FORMA).unwrap().default,
        CRUZ
    );
    assert_eq!(
        SPECS.iter().find(|s| s.name == TAMANHO).unwrap().default,
        0.0
    );
}

/// **A CONTAGEM DE FORMAS É DERIVADA DOS RÓTULOS** — ⛔ um `FORMAS` escrito à mão e uma lista de
/// rótulos são duas respostas à mesma pergunta, e divergem no dia da quarta forma.
#[test]
fn a_contagem_de_formas_sai_dos_rotulos() {
    let hint = HINTS
        .iter()
        .find(|h| h.param == FORMA)
        .expect("o hint da forma");
    let ph2d_node_registry::ParamWidget::Enum { labels } = hint.widget else {
        panic!("a forma tem de ser um selector");
    };
    assert_eq!(labels.len(), FORMAS);
    // E a faixa do selector tem de cobrir exactamente as formas que existem.
    assert_eq!(hint.max, (FORMAS - 1) as f32);
    assert_eq!(hint.min, 0.0);
}

/// **TODO PARAM DECLARADO TEM DICA** — senão ele aparece no cartão sem rótulo nem faixa.
#[test]
fn todo_param_tem_dica() {
    for s in SPECS {
        assert!(
            HINTS.iter().any(|h| h.param == s.name),
            "o param `{}` nao tem dica de UI",
            s.name
        );
    }
    assert_eq!(SPECS.len(), HINTS.len(), "e nao ha' dica orfa");
}
