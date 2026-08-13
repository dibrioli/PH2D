//! **A ÁRVORE** — como um esqueleto deixa de ser uma corrente única.
//!
//! Todo pacote de rig tem uma árvore de ossos (Spine: *"bones are hierarchical"*;
//! Rive: cadeias de ossos), e este source só emitia `parent[i] = i − 1`. A árvore
//! sempre foi **representável** — a coluna `parent` é livre e o [`crate::fk`] já a
//! honra, resolvendo por PAI e não por `i − 1` —, e nada a **escrevia**.
//!
//! ## O formato, e por que é texto
//!
//! `filho=pai`, separado por espaço ou vírgula: **`"5=2 9=2"`** pendura as juntas
//! 5 e 9 na junta 2. Quem não é listado mantém a corrente. Um `pai = -1` faz da
//! junta uma **raiz** — dois esqueletos num stream só, de graça.
//!
//! É um **text param** (o canal do doc 32) porque um `ParamSpec` é `f32` e o
//! manifesto é congelado; a árvore vive no `Graph`, ao lado do manifesto e nunca
//! dentro dele — o mesmo canal da fórmula do `motion.expression` e do nome de
//! forma do `motion.path`.
//!
//! ## ⚠️ Um override inválido é IGNORADO, e a alternativa é pior
//!
//! O [`crate::fk`] trata um pai que não aponta para trás como **RAIZ** — então
//! honrar `"2=5"` (para a frente), `"3=99"` (fora da faixa) ou `"x=y"` (lixo)
//! **DESTACARIA o membro inteiro** do rig. Um erro de digitação que não faz nada é
//! recuperável (o galho simplesmente não aparece); um que desmembra o personagem
//! não é. A corrente é o que sobra, sempre.

/// Colapsa o espaço em branco: um espaço ao lado de um `=` **desaparece** (é o
/// mesmo token), e qualquer outra corrida de espaços vira UM separador.
///
/// ⚠️ **Sem isto `"3 = 1"` é inerte**, e foi o gate dos separadores que pegou: se
/// a divisão acontece por espaço ANTES de acontecer por `=`, `3`, `=` e `1` viram
/// três tokens e nenhum é um override. *"Os separadores são os que a mão usa"* só
/// é verdade se o espaço em torno do `=` for um deles.
fn normalise(spec: &str) -> String {
    let mut out = String::with_capacity(spec.len());
    let (mut pending, mut after_eq) = (false, false);
    for c in spec.chars() {
        if c.is_whitespace() {
            pending = !out.is_empty();
            continue;
        }
        // O espaço ANTES de um `=` some; o de DEPOIS também (o `after_eq`).
        if c != '=' && pending && !after_eq {
            out.push(' ');
        }
        after_eq = c == '=';
        pending = false;
        out.push(c);
    }
    out
}

/// Aplica os overrides de `spec` sobre a corrente `parent[i] = i − 1`.
///
/// `n` é a contagem de juntas. Devolve a coluna `parent` inteira, já em `f32` —
/// a forma que o `fk` lê.
pub(crate) fn parents(n: usize, spec: &str) -> Vec<f32> {
    #[expect(
        clippy::cast_precision_loss,
        reason = "um índice de junta (dezenas), exato em f32"
    )]
    let mut parent: Vec<f32> = (0..n).map(|i| i as f32 - 1.0).collect();
    for tok in normalise(spec).split([' ', ',', ';']) {
        if tok.is_empty() {
            continue;
        }
        let Some((c, p)) = tok.split_once('=') else {
            continue; // sem `=` não é um override; a corrente fica
        };
        let (Ok(child), Ok(par)) = (c.trim().parse::<i64>(), p.trim().parse::<i64>()) else {
            continue;
        };
        // A junta tem de existir, e o pai tem de APONTAR PARA TRÁS (ou ser `-1`,
        // a raiz). Ver a nota do módulo: honrar o resto destacaria o membro.
        if child <= 0 || child >= n as i64 {
            continue; // a junta 0 é a raiz do rig e não tem pai a trocar
        }
        if par < -1 || par >= child {
            continue;
        }
        #[expect(
            clippy::cast_precision_loss,
            reason = "um índice de junta (dezenas), exato em f32"
        )]
        let v = par as f32;
        parent[child as usize] = v;
    }
    parent
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Um `branches` VAZIO é a corrente de sempre** — o neutro, e é ele que
    /// mantém todo rig já autorado intocado.
    #[test]
    fn an_empty_spec_is_the_single_chain_it_always_was() {
        for spec in ["", "   ", ",,", "\n\t"] {
            assert_eq!(
                parents(5, spec),
                vec![-1.0, 0.0, 1.0, 2.0, 3.0],
                "`{spec:?}`"
            );
        }
    }

    /// **Dois galhos na mesma junta** — a forma canônica (um tronco com dois
    /// braços), e o que a corrente única não sabe dizer.
    #[test]
    fn two_limbs_can_hang_off_the_same_joint() {
        assert_eq!(
            parents(6, "3=1 5=1"),
            vec![-1.0, 0.0, 1.0, 1.0, 3.0, 1.0],
            "3 e 5 penduram em 1; 2 e 4 mantêm a corrente"
        );
    }

    /// **Os separadores são os que a mão usa** — espaço, vírgula, quebra de linha.
    #[test]
    fn the_separators_are_the_ones_a_hand_types() {
        let want = parents(6, "3=1 5=1");
        for spec in ["3=1,5=1", "3=1, 5=1", "3=1\n5=1", " 3 = 1 ; 5 = 1 "] {
            assert_eq!(parents(6, spec), want, "`{spec}`");
        }
    }

    /// **`pai = -1` faz da junta uma RAIZ** — dois esqueletos num stream só.
    #[test]
    fn a_parent_of_minus_one_starts_a_second_root() {
        assert_eq!(parents(4, "2=-1"), vec![-1.0, 0.0, -1.0, 2.0]);
    }

    /// **Um override que o `fk` não pode honrar é IGNORADO, não obedecido.**
    ///
    /// ⚠️ É a metade que carrega o peso: o `fk` trata um pai que não aponta para
    /// trás como RAIZ, então obedecer a qualquer um destes **destacaria o membro
    /// inteiro**. Um erro de digitação que não faz nada é recuperável; um que
    /// desmembra o personagem não é.
    #[test]
    fn an_override_the_resolver_cannot_honour_is_ignored_not_obeyed() {
        let chain = parents(6, "");
        for bad in [
            "2=5",  // para a frente
            "3=3",  // ele próprio
            "3=99", // pai fora da faixa
            "99=1", // junta fora da faixa
            "0=2",  // a raiz do rig não tem pai a trocar
            "-2=1", // junta negativa
            "3=-9", // pai antes da raiz
            "x=y",  // lixo
            "3",    // sem `=`
            "=",    // vazio dos dois lados
        ] {
            assert_eq!(parents(6, bad), chain, "`{bad}` tem de ser inerte");
        }
    }

    /// **O último override de uma junta VENCE** — escrever a mesma junta duas
    /// vezes é ambiguidade do autor, e a regra tem de ser dizível numa frase.
    #[test]
    fn the_last_override_of_a_joint_wins() {
        assert_eq!(parents(6, "4=1 4=2"), parents(6, "4=2"));
    }
}
