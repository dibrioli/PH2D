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
//! ⛔ **Ela vive AQUI e não no gate porque tem QUATRO consumidores** — a tabela de strings, as cenas
//! da conferência do Motion, o estilo do slider e (14/10) o censo do que TODAS as cenas do Motion
//! pintam. *Uma lei escrita em dois sítios ainda não é uma lei, só uma PORTA é.*
//!
//! ⚠️ Desde 14/10 ela acusa também a letra acentuada (sem o `é`) e o apóstrofo no lugar do acento
//! (`so'`, `e'`): de 90 para 184 dos 195 textos portugueses que as cenas pintavam, com o resto
//! nomeado no teste `ela_acusa_o_portugues_que_as_cenas_pintavam`.

/// A classe FECHADA do português — nenhuma destas é palavra inglesa.
const FECHADA: &[&str] = &[
    "uma", "umas", "uns", "dos", "das", "nas", "pelo", "pela", "pelos", "pelas", "para", "com",
    "sem", "sobre", "entre", "apos", "aos", "mas", "que", "quando", "onde", "como", "porque",
    "nao", "ja", "ainda", "mais", "menos", "muito", "pouco", "todos", "todas", "este", "esta",
    "esse", "essa", "aquele", "aquela", "isto", "isso", "aquilo", "seu", "sua", "meu", "minha",
    "ele", "ela", "eles", "elas", "lhe", "lhes", "sao", "estao", "foi", "foram", "sera", "tem",
    "havia", "faz", "fazer", "vai", "vao", "pode", "podem", "deve", "devem", "cada", "qualquer",
    "outro", "outra", "agora", "sempre", "ao", "ou", "nem", "numa", "dentro", "fora", "dele",
    "dela", "deles", "delas", "tambem", "aqui", "quem", "quanto", "ninguem",
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
    // 2026-10-14: as legendas e os cartões das cenas do Motion (`portugues_cenas_2026_10_14.txt`).
    // ⛔ FICARAM DE FORA as que o inglês também tem: tremor, forte, vista, mar, magnitude, cantos,
    // giro, viva, alfa, age, leva, cor.
    "campo",
    "onda",
    "ondas",
    "eixo",
    "eixos",
    "corda",
    "osso",
    "ossos",
    "mao",
    "fila",
    "grelha",
    "vida",
    "copias",
    "curva",
    "dobra",
    "espelho",
    "espalha",
    "sombra",
    "faixa",
    "cauda",
    "escada",
    "mola",
    "vento",
    "vortice",
    "emissor",
    "atrator",
    "penacho",
    "tolerancia",
    "velocidade",
    "ordem",
    "grupo",
    "grupos",
    "linhas",
    "palavra",
    "grafico",
    "origem",
    "mundo",
    "meio",
    "rola",
    "salta",
    "mudo",
    "uniforme",
    "unica",
    "dois",
    "tres",
    "fica",
    "numero",
    "tamanho",
    "coluna",
    "cadeia",
];

/// Fins de palavra que só o português tem.
const FINS: &[&str] = &["cao", "coes", "ao", "oes", "mente", "ando", "endo", "indo"];

/// Letras que o português escreve e o inglês do app não: uma palavra que traga uma é portuguesa.
/// ⛔ Sem o `é`: o inglês do app escreve `Bézier` (medido na tabela, 14/10), e tem `café`, `résumé`.
const ACENTOS: &[char] = &['ç', 'ã', 'õ', 'á', 'í', 'ó', 'ú', 'â', 'ê', 'ô', 'à'];

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
    let mut resto = sem_marcador.as_str();
    // Uma aspa simples ABERTA (`'` sem letra antes): o `'` que a fecha não é acento.
    let mut aspa_aberta = false;
    let mut colado = false; // o intervalo começa logo depois de uma palavra
    while let Some(i) = resto.find(char::is_alphabetic) {
        let abre = resto[..i]
            .char_indices()
            .filter(|&(k, c)| c == '\'' && !(colado && k == 0))
            .count();
        aspa_aberta ^= abre % 2 == 1;
        colado = true;
        resto = &resto[i..];
        let fim = resto
            .find(|c: char| !c.is_alphabetic())
            .unwrap_or(resto.len());
        let w = resto[..fim].to_lowercase();
        resto = &resto[fim..];
        // O apóstrofo no lugar do acento (`so'`, `e'`, `da'`, `ate'`): colado ao fim de uma palavra
        // CURTA e sem letra depois. A contracção inglesa (`don't`) tem letra depois; a aspa que
        // fecha (`'Use Selected Path'`) tinha uma aberta; o possessivo (`others'`) é comprido.
        let mut depois = resto.chars();
        let fecha = depois.next() == Some('\'') && !depois.next().is_some_and(char::is_alphabetic);
        let apostrofo = fecha && !aspa_aberta && w.chars().count() <= 4;
        if fecha {
            aspa_aberta = false;
            resto = &resto[1..];
        }
        if w.chars().count() < 2 && !apostrofo {
            continue;
        }
        let e_pt = apostrofo
            || w.contains(ACENTOS)
            || FECHADA.contains(&w.as_str())
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
            // 14/10: o apóstrofo no lugar do acento, a letra acentuada, e a aspa que não o desarma.
            "so' o grupo clareia",
            "TRELI\u{c7}A",
            "the 'Shape' e' red",
            // e a contracção (`don't`) não abre aspa que desarme o acento a seguir (M6, 14/10).
            "don't go, e' red",
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
            // 14/10: as três formas do apóstrofo inglês, e o nome próprio acentuado.
            "pick one with 'Use Selected Path' or the Shape row",
            "Nobody sees the others' votes. I'm done, don't wait",
            "Custom (B\u{e9}zier)",
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

/// ⭐⭐ **O CONTROLO MEDIDO** — o português que as cenas do Motion pintavam em 14/10, lido do produto.
/// A régua de então acusava 90 de 195; esta acusa 184, e as que fogem são NOMEADAS: palavras que o
/// inglês também tem (`age`, `vista`, `tremor`) ou vocabulário que só uma lista decorada veria.
/// Uma régua que melhore encolhe a lista; uma que piore reprova aqui.
#[test]
fn ela_acusa_o_portugues_que_as_cenas_pintavam() {
    const FOGEM: &[&str] = &[
        "1. GUIADA: 3 ramos, tronco de 2",
        "1 · O carimbo",
        "1 STAGGER · Bounce: age",
        "2 DEGRAU · Smooth: age",
        "5. samambaia, a CRESCER",
        "6 TREMOR · 4 oitavas: age",
        "7 TINTA · em Gradient: age",
        "Camera: a vista",
        "FK: os pais decidem",
        "Loop (recomeca)",
        "Size (a magnitude)",
    ];
    let corpus = include_str!("portugues_cenas_2026_10_14.txt");
    let linhas: Vec<&str> = corpus.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(linhas.len(), 195, "o corpus de 14/10 tem 195 textos");
    let fogem: Vec<&str> = linhas.iter().copied().filter(|l| is_english(l)).collect();
    assert_eq!(fogem, FOGEM, "o que escapa à régua mudou");
}
