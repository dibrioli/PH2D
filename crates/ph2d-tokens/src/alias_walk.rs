//! **A LEI DO CICLO, escrita UMA vez** — o kernel que as duas camadas de override perguntam.
//!
//! # Por que ele é genérico
//!
//! Um alias é `a → b`, e a pergunta *"fazer `a` seguir `b` fecha um laço?"* **não sabe nada sobre o
//! que `a` VALE**: ela é sobre o grafo, não sobre cor nem sobre pixels. Escrevê-la uma vez por
//! família seria duas respostas à mesma pergunta — e a segunda é a que esquece o auto-alias no dia
//! em que alguém ajusta a primeira.
//!
//! # ⚠️ O FAN-OUT chegou com a math (W4c.3), e ele mudou a forma da lei
//!
//! Enquanto o grafo era só de aliases, um nó tinha **um** sucessor e a caminhada era uma corrente.
//! Uma **expressão** (`{spacing.md} + {radius.lg}`) tem **N**, e a pergunta deixa de ser *"a
//! corrente volta?"* para ser *"algum caminho volta?"* — uma busca em profundidade, não um passeio.
//!
//! ⚠️ **E o conjunto-visitado SUBSUMIU a casa dos pombos.** A versão anterior capava a caminhada em
//! `ALL.len()` saltos, argumentando que mais do que isso significa ter visitado o mesmo token duas
//! vezes. Estava certo, e era uma *inferência* sobre a contagem; um `visited` **observa** a
//! repetição em vez de a deduzir, o que é ao mesmo tempo mais forte (não depende de quem chama
//! passar o `max_hops` certo) e mais simples (não há segundo braço a explicar). O teto saiu do
//! parâmetro e passou a ser o tamanho do grafo, que é o que ele sempre quis dizer.
//!
//! # ⚠️ A caminhada de LEITURA **não** está aqui, e a ausência é a decisão
//!
//! Ela termina num VALOR (uma cor, um número) ou num token de fábrica, e esses terminais **são** o
//! tipo: generificá-los pediria um parâmetro de valor e um trait para poupar quatro linhas por
//! família. O que se partilha é a **lei**; o que difere é o que a cadeia **entrega**.

/// *Ligar `token` a `targets` fecharia um laço?* — devolve **onde** ele fecha.
///
/// `successors` responde *"deste token, para quais outros o valor dele aponta?"* — vazio termina
/// aquele ramo (um literal, ou um slot que ninguém autorou). Um alias devolve zero ou um; uma
/// expressão devolve tantos quantos os tokens que ela lê.
///
/// ⚠️ **Os alvos são testados ANTES do primeiro salto**, e é isso que apanha o auto-alias: um token
/// que segue a si mesmo — ou uma fórmula que se lê a si mesma, `{spacing.md} * 2` em `spacing.md` —
/// é um laço de comprimento um, e uma busca que só comparasse *depois* de avançar passaria por cima
/// dele.
///
/// ⚠️ **O token devolvido é onde a busca REENCONTROU o de partida**, e é accionável: é o nome que a
/// mensagem ao artista precisa de dizer.
pub(crate) fn closes_a_loop<T: Copy + PartialEq>(
    token: T,
    targets: &[T],
    successors: impl Fn(T) -> Vec<T>,
) -> Option<T> {
    let mut stack: Vec<T> = targets.to_vec();
    let mut visited: Vec<T> = Vec::new();
    while let Some(cur) = stack.pop() {
        if cur == token {
            return Some(cur);
        }
        // ⚠️ O `visited` é o que torna o grafo FINITO para esta busca: sem ele, uma tabela que já
        // chegou cíclica de um arquivo faria a pilha crescer para sempre. Ele não é uma optimização.
        if visited.contains(&cur) {
            continue;
        }
        visited.push(cur);
        stack.extend(successors(cur));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::closes_a_loop;

    /// A tabela do teste: um grafo de inteiros, que é exactamente o que o kernel vê.
    fn walk(edges: &[(u8, u8)]) -> impl Fn(u8) -> Vec<u8> + '_ {
        move |t| {
            edges
                .iter()
                .filter(|(a, _)| *a == t)
                .map(|(_, b)| *b)
                .collect()
        }
    }

    #[test]
    fn a_token_that_follows_itself_is_a_loop_of_length_one() {
        // ⚠️ O caso que uma busca "salta primeiro, compara depois" deixa passar.
        assert_eq!(closes_a_loop(1, &[1], walk(&[])), Some(1));
    }

    #[test]
    fn a_fresh_target_closes_nothing() {
        assert_eq!(closes_a_loop(1, &[2], walk(&[])), None);
    }

    #[test]
    fn the_loop_is_reported_where_it_closes() {
        // 1 → 2 pedido; 2 → 3 → 1 já existe ⇒ fecha em 1.
        assert_eq!(closes_a_loop(1, &[2], walk(&[(2, 3), (3, 1)])), Some(1));
    }

    #[test]
    fn a_long_honest_chain_is_not_a_loop() {
        assert_eq!(
            closes_a_loop(9, &[1], walk(&[(1, 2), (2, 3), (3, 4)])),
            None
        );
    }

    /// ⚠️ **O caso que a corrente não sabia fazer:** um dos ramos volta, o outro não.
    #[test]
    fn a_loop_down_one_branch_of_a_fan_out_is_still_a_loop() {
        // 9 lê 1 e 5. O ramo do 1 morre; o do 5 volta ao 9.
        assert_eq!(
            closes_a_loop(9, &[1, 5], walk(&[(1, 2), (5, 6), (6, 9)])),
            Some(9)
        );
    }

    /// ⚠️ E o irmão: fan-out **sem** laço termina, em vez de ficar preso num losango.
    #[test]
    fn a_diamond_without_a_loop_terminates() {
        // 1 e 2 convergem em 3 — o `visited` impede que o 3 seja expandido duas vezes.
        assert_eq!(closes_a_loop(9, &[1, 2], walk(&[(1, 3), (2, 3)])), None);
    }

    /// Uma tabela que já chegou CÍCLICA de um arquivo não pendura a busca.
    ///
    /// ⚠️ Este era o braço da "casa dos pombos", e a afirmação mudou de forma: antes ele só podia
    /// dizer *"há laço algures"* (o token devolvido era onde a contagem estourou); agora a busca
    /// **termina** e responde a pergunta que lhe foi feita — este laço não passa pelo 9, então não
    /// fecha nada PARA o 9, e o `visited` é o que garante que ela pára.
    #[test]
    fn a_pre_existing_loop_that_does_not_reach_the_token_terminates() {
        assert_eq!(
            closes_a_loop(9, &[1], walk(&[(1, 2), (2, 3), (3, 1)])),
            None
        );
    }
}
