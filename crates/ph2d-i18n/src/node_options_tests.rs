//! A RÉGUA da [`super::node_options`] — a tabela dela está ORDENADA e cada chave é encontrável.
//!
//! ⚠️ Ela saiu de dentro do `node_options.rs` na integração de 2026-09-20, por tecto de LOC
//! (`701` contra `700`) e **por responsabilidade**: aquele ficheiro é a TABELA, e a ordem dela
//! ser load-bearing não faz da régua parte da tabela. ⛔ O corte NÃO toca na ordem nem numa
//! entrada — o que se moveu é quem mede.

/// ⛔⛔ **A TABELA TEM DE ESTAR ORDENADA** — o [`super::tr`] procura por `binary_search`, e
/// uma entrada fora de ordem faz a busca devolver `None` para uma chave que **ESTÁ** na
/// tabela: o selector pinta o identificador cru e `leak_key` vaza por quadro.
///
/// ⛔⛔⛔ **Esta régua já existia para as OUTRAS DUAS tabelas por tuplo**
/// (`node_params::testes_do_corte::as_duas_metades_estao_ordenadas`, cujo doc descreve este
/// defeito por extenso) e **esta ficou de fora** — e foi exactamente esta que se partiu, em
/// 2026-09-17, com quinze entradas escritas no meio dos `node.motion.*`.
///
/// ⚠️ *Uma lei conhecida, escrita e gateada em dois de três sítios é indistinguível de uma
/// lei cumprida* — e quem a acusou foi um gate de OUTRA crate, sobre **dois** selectores do
/// `motion.wiggle`: uma amostra do estrago, nunca o estrago.
#[test]
fn as_entradas_estao_ordenadas() {
    let t = super::ENTRADAS;
    // ⛔ Piso de população: uma tabela vazia está trivialmente ordenada.
    assert!(
        t.len() >= 100,
        "só {} entradas — a tabela encolheu?",
        t.len()
    );
    for w in t.windows(2) {
        assert!(
            w[0].0 < w[1].0,
            "FORA DE ORDEM: {:?} vem antes de {:?} — tudo o que vier depois desta linha \
             deixa de resolver, sem erro de compilação",
            w[0].0,
            w[1].0
        );
    }
    // ⭐ O CONTROLO POSITIVO da ordenação: a busca de facto acha cada uma. Sem ele, uma
    //   régua que lesse a lista errada concordaria consigo própria.
    for (k, v) in t {
        assert_eq!(super::tr(k), Some(*v), "`{k}` não é encontrável");
    }
}
