//! Os gates do [`SequencePlayer`] (TOP-20 #19).

use super::SequencePlayer;

fn p(nome: &str) -> SequencePlayer {
    SequencePlayer {
        container: nome.to_owned(),
    }
}

/// ⭐ **O nome resolve para o ÍNDICE, e é o nome que viaja.** Um índice guardado no objecto tocaria
/// a cutscene do vizinho no dia em que alguém apagasse o container de cima.
#[test]
fn o_nome_resolve_para_o_indice_do_container() {
    let nomes = ["Intro", "Porta", "Fim"];
    assert_eq!(p("Porta").resolve(nomes), Some(1));
    assert_eq!(p("Intro").resolve(nomes), Some(0));
    assert_eq!(p("Fim").resolve(nomes), Some(2));
}

/// ⛔ **Um nome em branco NÃO é uma sequência** — a mesma regra do `SignalOnHit` e do marcador da
/// timeline, para que «sem cutscene» e «cutscene por escolher» não leiam igual.
#[test]
fn um_nome_em_branco_nao_e_uma_sequencia() {
    assert_eq!(p("").name(), None);
    assert_eq!(p("   ").name(), None);
    assert_eq!(p("   ").resolve(["Intro"]), None);
}

/// ⚠️ **Os DOIS lados são aparados** — um container gravado com um espaço à direita e um componente
/// sem ele são a mesma cutscene para quem os escreveu.
#[test]
fn os_dois_lados_sao_aparados() {
    assert_eq!(p(" Porta ").resolve(["Intro", "Porta  "]), Some(1));
}

/// ⛔ **Um nome que não existe não toca nada** — e não o PRIMEIRO, que é o modo de falha caro: a
/// cutscene errada a correr lê-se como um defeito do motor.
#[test]
fn um_nome_que_nao_existe_nao_toca_nada() {
    assert_eq!(p("Ausente").resolve(["Intro", "Porta"]), None);
    assert_eq!(p("Intro").resolve([]), None);
}

/// ⚠️ **Dois containers com o mesmo nome: ganha o PRIMEIRO, e isso é uma decisão declarada.**
/// Devolver `None` faria o artista perder a cutscene por uma duplicação que ele não vê.
#[test]
fn com_nomes_duplicados_ganha_o_primeiro_e_isso_e_declarado() {
    assert_eq!(p("Porta").resolve(["Porta", "Porta"]), Some(0));
}
