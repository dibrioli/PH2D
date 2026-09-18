//! ⭐⭐⭐ **O chip da VIGIA mostra um SINAL (`≤` · `≥` · `=`), e é o MESMO que a linha fechada.**
//!
//! # O defeito que isto impede de voltar
//!
//! Até 2026-09-17 a secção falava DUAS línguas sobre o mesmo facto, a uma dobra de distância: a
//! linha FECHADA da lista resumia `≤ 0 → morri` e o chip ABERTO dizia *«drops to or below»*. As
//! duas estavam certas e eram **duas fontes** — o resumo tinha o glifo escrito à mão e o chip
//! puxava três chaves de i18n.
//!
//! ⚠️ **Nenhum gate podia ver isso**, e a razão é a forma: os cinco gates de costura da secção
//! perguntam *«o clique chega?»* e o gate da shell pergunta *«a ORDEM bate com o enum?»* — nenhum
//! pergunta ***o que está ESCRITO no botão***. A cura foi uma PORTA
//! ([`ph2d_panel_inspector::simbolo_da_comparacao`]) com dois leitores, e este gate é o que torna
//! a segunda fonte impossível de reintroduzir em silêncio.
//!
//! # ⛔ E porque são TRÊS sinais e não os cinco que o dono nomeou
//!
//! Ele perguntou por `<= == >= > <`. Num contador **INTEIRO** — e `Compare::holds` recebe `i64` dos
//! dois lados — `valor > n` é **exactamente** `valor >= n+1` e `valor < n` é `valor <= n-1`: o
//! mesmo conjunto de valores, sem aproximação. ⇒ um quarto e um quinto sinal seriam **uma segunda
//! maneira de escrever a MESMA regra**, que é o que a `CLAUDE.md` §5.0 manda medir antes de
//! construir. A decisão fica aqui, ao lado da medição, e não num comentário solto.

use ph2d_panel_inspector::{ids, opcoes_de_comparacao, simbolo_da_comparacao};

/// ⭐⭐ **O que o artista lê no chip É o que a porta devolve** — e a população é a do motor.
///
/// **Mutação que deve sangrar:** devolver um rótulo em palavras dentro de `opcoes_de_comparacao`,
/// ou encurtar `INSP_WATCH_CMP_OPT`.
#[test]
fn o_chip_da_vigia_mostra_o_sinal_da_porta() {
    let opcoes = opcoes_de_comparacao();
    assert_eq!(
        opcoes.len(),
        ids::INSP_WATCH_CMP_OPT.len(),
        "o chip pinta um número de comparações diferente do que endereça"
    );
    // ⛔ Piso de população: uma lista vazia satisfaz todo laço abaixo, em silêncio.
    assert!(opcoes.len() >= 3, "a vigia tem três comparações, no mínimo");
    for (i, opcao) in opcoes.iter().enumerate() {
        let esperado = simbolo_da_comparacao(u8::try_from(i).expect("três cabem num u8"));
        assert_eq!(
            opcao.label, esperado,
            "a opção {i} do chip não diz o que a porta diz — há uma SEGUNDA fonte do rótulo"
        );
    }
}

/// ⭐ **Cada rótulo é UM glifo, e os três são diferentes.**
///
/// A primeira metade é a decisão do dono escrita onde pode ser AFIRMADA (*«em vez de nomes por que
/// não sinais»*); a segunda é o que a torna utilizável — ⛔ *duas opções que se leem igual são uma
/// escolha que o artista não consegue fazer*, e um `match` com dois braços iguais lê-se como
/// correcto num diff.
#[test]
fn cada_sinal_e_um_glifo_so_e_os_tres_sao_distintos() {
    let sinais: Vec<&str> = (0..3).map(simbolo_da_comparacao).collect();
    for (i, s) in sinais.iter().enumerate() {
        assert_eq!(
            s.chars().count(),
            1,
            "a comparação {i} mostra {s:?}, que não é um sinal — o chip desta secção não leva frases"
        );
    }
    for i in 0..sinais.len() {
        for j in (i + 1)..sinais.len() {
            assert_ne!(
                sinais[i], sinais[j],
                "as comparações {i} e {j} mostram o mesmo sinal — são indistinguíveis sob o dedo"
            );
        }
    }
}
