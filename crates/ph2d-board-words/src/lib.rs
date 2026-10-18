//! **As palavras de uma nota** (W7) — o que conta para agrupar notas por palavra
//! (`docs/MiroClone/05_facilitacao.md` §2 C7). O «Keywords» do Miro é IA; este é determinístico, e cada
//! passo tem oráculo corrido (`docs/MiroClone/ferramentas/facilitacao_oracle/`):
//!
//! 1. as palavras UAX#29 (o segmentador ICU; o `Intl.Segmenter` do Node é o oráculo);
//! 2. `’` → `'` (com o curvo, `team’s` não dava a raiz de `team`);
//! 3. *case folding* COMPLETO com os acentos ([`ph2d_label_fold::case_fold`]);
//! 4. fora as de uma letra e as palavras vazias do **Snowball** (BSD-3; ⛔ a lista ISO apagava `need`);
//! 5. a raiz **Snowball 3.1.1** (a `libstemmer` instalada é o oráculo, 0 diferenças);
//! 6. a chave é a raiz SEM acentos ([`ph2d_label_fold::fold`]): `Café` = `cafe`.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use icu_segmenter::WordSegmenter;
use icu_segmenter::options::WordBreakInvariantOptions;

/// A língua das notas: a do stemmer e a da lista de palavras vazias.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    English,
    Portuguese,
}

impl Lang {
    /// As línguas, pela ordem do desempate (a 1.ª ganha: a interface é inglesa).
    pub const ALL: [Lang; 2] = [Lang::English, Lang::Portuguese];

    fn algorithm(self) -> snowball::Algorithm {
        match self {
            Lang::English => snowball::Algorithm::English,
            Lang::Portuguese => snowball::Algorithm::Portuguese,
        }
    }

    /// As palavras vazias do Snowball (`snowballstem.org/algorithms/<língua>/stop.txt`, BSD-3), já em
    /// *case folding*.
    fn stop_words(self) -> &'static BTreeSet<String> {
        static EN: OnceLock<BTreeSet<String>> = OnceLock::new();
        static PT: OnceLock<BTreeSet<String>> = OnceLock::new();
        match self {
            Lang::English => EN.get_or_init(|| parse_stop(include_str!("stop_en.txt"))),
            Lang::Portuguese => PT.get_or_init(|| parse_stop(include_str!("stop_pt.txt"))),
        }
    }
}

/// O formato do Snowball: uma palavra no início da linha; `|` começa um comentário.
fn parse_stop(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|l| l.split('|').next()?.split_whitespace().next())
        .map(ph2d_label_fold::case_fold)
        .collect()
}

/// As palavras de `text` (os segmentos UAX#29 que são palavra: nem espaço nem pontuação), como
/// escritas.
#[must_use]
pub fn segment(text: &str) -> Vec<&str> {
    let seg = WordSegmenter::new_for_non_complex_scripts(WordBreakInvariantOptions::default());
    let mut out = Vec::new();
    let mut start = 0;
    for (end, kind) in seg.segment_str(text).iter_with_word_type() {
        if end > start && kind.is_word_like() {
            out.push(&text[start..end]);
        }
        start = end;
    }
    out
}

/// A raiz Snowball 3.1.1 de `word` (já em *case folding*, NFC).
#[must_use]
pub fn stem(lang: Lang, word: &str) -> String {
    snowball::stem(lang.algorithm(), word).into_owned()
}

/// Uma palavra que conta: a chave de comparação (a raiz sem acentos) e a forma escrita (o que um
/// título mostra): em *case folding* com acentos, salvo quando as maiúsculas dizem algo — uma além
/// da 1.ª letra («API», «iPhone») —, e então como foi escrita (W23 A7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keyword {
    pub key: String,
    pub surface: String,
}

/// As palavras de `text` que contam, por ordem e com repetições.
#[must_use]
pub fn keywords(text: &str, lang: Lang) -> Vec<Keyword> {
    let stop = lang.stop_words();
    segment(text)
        .into_iter()
        .filter_map(|w| {
            let written = w.replace('\u{2019}', "'");
            let w = ph2d_label_fold::case_fold(&written);
            let letters = w.chars().filter(|c| c.is_alphabetic()).count();
            if letters < 2 || stop.contains(&w) {
                return None;
            }
            let key = ph2d_label_fold::fold(&stem(lang, &w));
            // A maiúscula de começo de frase não diz nada; uma a meio da palavra, sim.
            let surface = if written.chars().skip(1).any(char::is_uppercase) {
                written
            } else {
                w
            };
            Some(Keyword { key, surface })
        })
        .collect()
}

/// A língua de um conjunto de notas: a que tiver mais palavras vazias nelas (empate = a 1.ª de
/// [`Lang::ALL`]).
#[must_use]
pub fn detect(texts: &[&str]) -> Lang {
    let hits = |lang: Lang| {
        let stop = lang.stop_words();
        texts
            .iter()
            .flat_map(|t| segment(t))
            .filter(|w| stop.contains(&ph2d_label_fold::case_fold(&w.replace('\u{2019}', "'"))))
            .count()
    };
    let mut best = Lang::ALL[0];
    let mut most = hits(best);
    for lang in &Lang::ALL[1..] {
        let n = hits(*lang);
        if n > most {
            (best, most) = (*lang, n);
        }
    }
    best
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
