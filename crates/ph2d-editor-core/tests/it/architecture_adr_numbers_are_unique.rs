//! **Arch-gate: dois ADRs não podem dividir o mesmo número.**
//!
//! ## Por que esta classe de bug existe, e por que nenhum outro gate a pega
//!
//! Duas linhas paralelas escrevem um ADR na mesma jornada. Cada uma olha o `main` que ela
//! conhece, vê que o último é o `0118`, e escolhe "o próximo livre": **as duas escrevem 0119**.
//!
//! E aí vem o detalhe que torna isso invisível: os **nomes de arquivo são diferentes**
//! (`0119-audio-loop-regions...` e `0119-vector-live-corners...`), então **o git nunca
//! conflita** — as duas entram, em silêncio, e a árvore fica verde.
//!
//! O estrago não é cosmético. `grep ADR-0119` passa a devolver dois assuntos sem relação, e
//! quem chega depois não tem como saber qual é qual. Pior: um `sed` global "consertando" o
//! número **destrói metade das referências**, porque as duas metades são legítimas.
//!
//! Aconteceu de verdade **duas vezes** (áudio × timeline no 0115; áudio × vetor no 0119), e as
//! duas vezes o defeito foi descoberto por leitura, não por gate.
//!
//! ## A regra
//!
//! O valor certo **não existe em nenhum dos dois lados do conflito** — não se ESCOLHE, se
//! CONTA. Este gate conta: se dois arquivos dividem o prefixo `NNNN`, ele fica vermelho e diz
//! quais são. Quem chegou ao `main` primeiro fica com o número; o outro renumera para o próximo
//! livre — **contando também as linhas vivas**, que podem já ter reservado o seguinte.

use std::collections::BTreeMap;
use std::path::Path;

/// O diretório dos ADRs, relativo à raiz do workspace.
const DECISIONS: &str = "../../docs/architecture/decisions";

#[test]
fn no_two_adrs_share_a_number() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(DECISIONS);
    let entries = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("o diretório de ADRs sumiu ({}): {e}", dir.display()));

    let mut by_number: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".md") else {
            continue;
        };
        // O prefixo é `NNNN-`; qualquer outra coisa (README, template) não é um ADR.
        let Some((number, rest)) = stem.split_once('-') else {
            continue;
        };
        // **Uma EMENDA divide o número de propósito** — `0040-amendment-2` é uma emenda AO
        // 0040, e é assim que este repo registra a evolução de uma decisão sem reescrever a
        // história dela. Ela não é um ADR novo disputando o número.
        if rest.starts_with("amendment") {
            continue;
        }
        if number.len() == 4 && number.chars().all(|c| c.is_ascii_digit()) {
            by_number.entry(number.to_owned()).or_default().push(name);
        }
    }
    assert!(
        !by_number.is_empty(),
        "não achei ADR nenhum — o caminho `{DECISIONS}` mudou?"
    );

    // **A dívida herdada FOI PAGA (2026-07-14, linha `line/audio`).** O `0115` estava duplicado
    // (áudio espectral × composição de clips da timeline): duas linhas escolheram "o próximo
    // número livre" contra um `main` que ainda não tinha o ADR da outra. O áudio — que chegou 11
    // minutos depois e tinha menos referências — renumerou para o `0122`, e a allowlist
    // `KNOWN_DEBT` que vivia aqui foi APAGADA junto.
    //
    // Ela era auto-limpante de propósito: um segundo assert exigia que a duplicata AINDA
    // existisse, então o conserto derrubava o gate e cobrava a remoção da exceção. É o desenho
    // que impede o modo de morte mais comum de um gate — a allowlist que sobrevive ao conserto e
    // vira um buraco permanente, verde, no meio da regra.
    //
    // **Não há mais exceção. Duplicata nenhuma passa.**

    let dupes: Vec<(&String, &Vec<String>)> =
        by_number.iter().filter(|(_, v)| v.len() > 1).collect();
    assert!(
        dupes.is_empty(),
        "dois ADRs dividem o mesmo número — e como os NOMES de arquivo diferem, o git nunca \
         conflitou e os dois entraram em silêncio.\n\n{}\n\nQuem chegou ao `main` primeiro fica \
         com o número; o outro renumera para o próximo livre (contando também as linhas VIVAS, \
         que podem já ter reservado o seguinte). NÃO rode um `sed` global no número: as \
         referências dos DOIS assuntos são legítimas, e metade delas seria destruída.",
        dupes
            .iter()
            .map(|(n, files)| format!("  {n}: {}", files.join("  ·  ")))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
