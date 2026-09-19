//! ⭐⭐⭐ **A RÉGUA DA LÍNGUA — o que este texto tem de PORTUGUÊS.**
//!
//! Ela nasceu em 2026-09-19 por **ordem do dono** (*«tudo em inglês»*), e o que a tornou necessária
//! é o que nenhuma das trinta catracas do HR-15 pergunta: elas perguntam *«esta palavra veio da
//! TABELA?»*, e nenhuma pergunta *«e a tabela está em que LÍNGUA?»*. Medido nesse dia: **oito**
//! frases em português na tabela inglesa — todas escritas à mão **fora** dos marcadores do script
//! de migração — e **23** palavras portuguesas pintadas no canvas das cenas da conferência.
//!
//! # ⚠️ O que ela É, declarado
//!
//! Um **PISO**, nunca um detector de português. Ela acusa por tokens da classe FECHADA (artigos,
//! preposições, pronomes — nenhum deles palavra inglesa) e por **MORFOLOGIA** (`-ção/-cao`,
//! `-ões/-oes`, `-ão/-ao`, `-mente`, `-ando/-endo/-indo`). Uma frase curta sem nenhum dos dois
//! passa-lhe ao lado — `"colorize a recalcular"` é o exemplo medido. *Uma régua heurística que se
//! declara é utilizável; uma que se julga completa é uma licença.*
//!
//! ⛔ **Ela vive AQUI e não no gate porque tem DOIS consumidores** — a tabela de strings e as cenas
//! da conferência do Motion. *Uma lei escrita em dois sítios ainda não é uma lei, só uma PORTA é.*

/// A classe FECHADA do português — nenhuma destas é palavra inglesa.
const FECHADA: &[&str] = &[
    "uma", "umas", "uns", "dos", "das", "nas", "pelo", "pela", "pelos", "pelas", "para", "com",
    "sem", "sobre", "entre", "apos", "aos", "mas", "que", "quando", "onde", "como", "porque",
    "nao", "ja", "ainda", "mais", "menos", "muito", "pouco", "todos", "todas", "este", "esta",
    "esse", "essa", "aquele", "aquela", "isto", "isso", "aquilo", "seu", "sua", "meu", "minha",
    "ele", "ela", "eles", "elas", "lhe", "lhes", "sao", "estao", "foi", "foram", "sera", "tem",
    "havia", "faz", "fazer", "vai", "vao", "pode", "podem", "deve", "devem", "cada", "qualquer",
    "outro", "outra",
];

/// Palavras ABERTAS que este repo já viu pintadas, e que o inglês não tem.
///
/// ⚠️ Ela é curta de propósito: cada entrada saiu de um defeito MEDIDO, e uma lista que crescesse
/// por precaução passaria a acusar inglês.
const MEDIDAS: &[&str] = &[
    "peca",
    "pecas",
    "malha",
    "nivel",
    "borda",
    "cena",
    "barro",
    "pilha",
    "precisa",
    "trabalha",
    "empurra",
    "escondida",
    "montada",
    "reverte",
    "alcancar",
    "encostar",
    "alvo",
    "mira",
    "rastro",
    "corte",
    "banda",
    "rampa",
    "forma",
    "solta",
    "desvia",
    "aparado",
    "picotado",
    "antes",
    "depois",
];

/// Fins de palavra que só o português tem.
const FINS: &[&str] = &["cao", "coes", "ao", "oes", "mente", "ando", "endo", "indo"];

/// ⭐⭐ **As palavras portuguesas que este texto traz** — vazio quando ele passa por inglês.
///
/// ⚠️ **Os marcadores de `format!` saem primeiro**: `{alvo}` é um nome de código, e o artista lê o
/// VALOR dele. Foi essa exacta confusão que fez a 1.ª sonda desta jornada acusar
/// `"never fires · {verbo} · {alvo}"`, que é uma frase inglesa.
#[must_use]
pub fn portuguese_tokens(text: &str) -> Vec<String> {
    let mut sem_marcador = String::with_capacity(text.len());
    let mut dentro = false;
    for c in text.chars() {
        match c {
            '{' => dentro = true,
            '}' => dentro = false,
            _ if !dentro => sem_marcador.push(c),
            _ => {}
        }
    }
    let mut achados: Vec<String> = Vec::new();
    for w in sem_marcador.split(|c: char| !c.is_alphabetic()) {
        let w = w.to_lowercase();
        if w.len() < 2 {
            continue;
        }
        let e_pt = FECHADA.contains(&w.as_str())
            || MEDIDAS.contains(&w.as_str())
            || FINS.iter().any(|f| w.len() > f.len() + 1 && w.ends_with(f));
        if e_pt && !achados.contains(&w) {
            achados.push(w);
        }
    }
    achados
}

/// Atalho: este texto passa por inglês?
#[must_use]
pub fn is_english(text: &str) -> bool {
    portuguese_tokens(text).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **O CONTROLO POSITIVO** — as frases que esta régua nasceu para apanhar.
    #[test]
    fn ela_ve_o_portugues_que_a_fez_nascer() {
        for f in [
            "ha' uma pilha de multiresolucao montada -- J reverte-a",
            "nao ha' peca nenhuma para cortar",
            "ALVO",
            "DEPOIS",
            "PICOTADO",
        ] {
            assert!(!is_english(f), "a régua não viu português em {f:?}");
        }
    }

    /// ⛔ **E o NEGATIVO** — sem ele, uma régua que acusasse tudo passaria o teste de cima.
    #[test]
    fn ela_nao_acusa_ingles() {
        for f in [
            "a multiresolution stack is mounted -- J reverts it",
            "Snap to grid",
            "TARGET",
            "RELEASE",
            "never fires \u{b7} {verbo} \u{b7} {alvo}",
            "Canonical widget showcase \u{b7} reference for peripheral agents",
        ] {
            assert!(
                is_english(f),
                "a régua acusou inglês em {f:?}: {:?}",
                portuguese_tokens(f)
            );
        }
    }
}
