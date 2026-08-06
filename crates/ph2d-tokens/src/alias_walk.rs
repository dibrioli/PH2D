//! **A LEI DO CICLO, escrita UMA vez** — o kernel que as duas camadas de override perguntam.
//!
//! # Por que ele é genérico
//!
//! Um alias é `a → b`, e a pergunta *"fazer `a` seguir `b` fecha um laço?"* **não sabe nada sobre o
//! que `a` VALE**: ela é sobre o grafo, não sobre cor nem sobre pixels. Escrevê-la uma vez por
//! família seria duas respostas à mesma pergunta — e a segunda é a que esquece o auto-alias no dia
//! em que alguém ajusta a primeira.
//!
//! # ⚠️ A caminhada de LEITURA **não** está aqui, e a ausência é a decisão
//!
//! Ela termina num VALOR (uma cor, um número) ou num token de fábrica, e esses terminais **são** o
//! tipo: generificá-los pediria um parâmetro de valor e um trait para poupar quatro linhas por
//! família. O que se partilha é a **lei**; o que difere é o que a cadeia **entrega**.

/// *Fazer `token` seguir `target` fecharia um laço?* — devolve **onde** ele fecha.
///
/// `alias_of` responde *"este slot é um alias, e para quem?"* — `None` termina a cadeia (um valor
/// literal, ou um slot que ninguém autorou).
///
/// ⚠️ **O teste é feito ANTES do primeiro salto**, e é isso que apanha o auto-alias: um token que
/// segue a si mesmo é um laço de comprimento um, e uma caminhada que só comparasse *depois* de
/// avançar passaria por cima dele.
///
/// ⚠️ **`max_hops` é a casa dos pombos, não um teto escolhido**: só existem `N` slots por modo,
/// então uma caminhada com mais de `N` saltos **visitou o mesmo token duas vezes** — e isso É um
/// ciclo. Quem chama passa o `ALL.len()` da sua família.
///
/// ⚠️ **O token devolvido significa coisas diferentes nos dois braços**, e isto vale para quem
/// escreve a mensagem ao artista: no encontro (`cur == token`) ele é **onde o laço fecha**, e é
/// accionável; no estouro da casa dos pombos ele é apenas **onde a caminhada parou** — há um laço
/// algures naquela cadeia, mas nada garante que seja neste token. O segundo braço só é alcançável
/// por uma tabela corrompida fora da porta, e é por isso que o produto não constrói frase sobre ele.
pub(crate) fn closes_a_loop<T: Copy + PartialEq>(
    token: T,
    target: T,
    max_hops: usize,
    alias_of: impl Fn(T) -> Option<T>,
) -> Option<T> {
    let mut cur = target;
    for _ in 0..=max_hops {
        if cur == token {
            return Some(cur);
        }
        match alias_of(cur) {
            Some(next) => cur = next,
            None => return None,
        }
    }
    Some(cur)
}

#[cfg(test)]
mod tests {
    use super::closes_a_loop;

    /// A tabela do teste: um grafo de inteiros, que é exactamente o que o kernel vê.
    fn walk(edges: &[(u8, u8)]) -> impl Fn(u8) -> Option<u8> + '_ {
        move |t| edges.iter().find(|(a, _)| *a == t).map(|(_, b)| *b)
    }

    #[test]
    fn a_token_that_follows_itself_is_a_loop_of_length_one() {
        // ⚠️ O caso que uma caminhada "salta primeiro, compara depois" deixa passar.
        assert_eq!(closes_a_loop(1, 1, 8, walk(&[])), Some(1));
    }

    #[test]
    fn a_fresh_target_closes_nothing() {
        assert_eq!(closes_a_loop(1, 2, 8, walk(&[])), None);
    }

    #[test]
    fn the_loop_is_reported_where_it_closes() {
        // 1 → 2 pedido; 2 → 3 → 1 já existe ⇒ fecha em 1.
        assert_eq!(closes_a_loop(1, 2, 8, walk(&[(2, 3), (3, 1)])), Some(1));
    }

    #[test]
    fn a_long_honest_chain_is_not_a_loop() {
        assert_eq!(
            closes_a_loop(9, 1, 8, walk(&[(1, 2), (2, 3), (3, 4)])),
            None
        );
    }

    #[test]
    fn a_chain_longer_than_the_pigeonhole_is_a_loop_even_without_meeting_the_token() {
        // 1 → 2 → 3 → 1 é um laço que NÃO passa pelo token de partida (9). A casa dos pombos é a
        // segunda camada: ela existe para uma tabela que chegou de um ARQUIVO já corrompida.
        //
        // ⚠️ O que se afirma é **que há laço**, e não QUAL token volta: neste braço o valor é onde
        // a caminhada parou, e depende de quantos saltos couberam. A 1ª versão deste gate cravava
        // `Some(1)` e falhou sobre produto correcto — a expectativa é que estava errada.
        assert!(closes_a_loop(9, 1, 3, walk(&[(1, 2), (2, 3), (3, 1)])).is_some());
    }
}
