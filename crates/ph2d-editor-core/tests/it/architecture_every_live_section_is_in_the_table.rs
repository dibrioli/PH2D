//! ⛔⛔ **Toda secção viva do Inspector TEM de estar na tabela** — o censo que faltava.
//!
//! # Porque ele existe, com a data
//!
//! A [`ph2d_editor_core::ids::LIVE_SECTIONS`] nasceu em 2026-08-21 para curar uma podridão medida:
//! três cabeçalhos pintavam o chevron e **não dobravam**, e sete pontos de cor estavam mortos,
//! porque cada secção era enumerada em **quatro** sítios que ninguém ligava entre si.
//!
//! ⚠️⚠️ **E a recaída aconteceu à mesma:** em 2026-09-14 as secções FACTORY e LIFECYCLE (TOP-20 #11
//! e #12) shiparam **fora** da tabela. O preço estava escrito no doc dela — quem falta ali não é
//! `mark_collapsible_section`ado e não é `is_section_header_id` —, e o sintoma é mudo: o cabeçalho
//! pinta o chevron, o artista carrega, e nada dobra. Quem o viu foi a wave seguinte (TOP-20 #13),
//! ao ir escrever a mesma linha.
//!
//! ⇒ *uma tabela que cura a podridão continua a ter um passo que se pode ESQUECER, e a única
//! coisa que fecha isso é um censo.* Este.
//!
//! # ⚠️ A forma do censo, e as duas armadilhas que ele evita
//!
//! - **Piso de população**: uma varredura partida devolve zero declarações e `faltam.is_empty()`
//!   sobre uma lista vazia é trivialmente verdadeiro — o defeito que o HOWTO §2.7 chama de *falha
//!   MUDA*. O piso trava isso.
//! - **Lê os DOIS lados do disco**: as declarações e a tabela. Comparar a tabela consigo mesma
//!   (por exemplo, exigir `LIVE_SECTIONS.len() == N`) seria um espelho, e um espelho não acusa.

use std::collections::BTreeSet;
use std::path::PathBuf;

fn raiz_dos_ids() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/ids")
}

/// Todos os `INSP_LIVE_*_SECTION` declarados.
fn declaradas() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let dir = raiz_dos_ids();
    let entradas = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("a pasta dos ids tem de existir em {}: {e}", dir.display()));
    for e in entradas.flatten() {
        let p = e.path();
        if p.extension().is_none_or(|x| x != "rs") {
            continue;
        }
        let texto = std::fs::read_to_string(&p).expect("um ficheiro de ids lê-se");
        for linha in texto.lines() {
            let Some(resto) = linha.trim().strip_prefix("pub const INSP_LIVE_") else {
                continue;
            };
            let Some(nome) = resto.split(':').next() else {
                continue;
            };
            let nome = nome.trim();
            if nome.ends_with("_SECTION") {
                out.insert(format!("INSP_LIVE_{nome}"));
            }
        }
    }
    out
}

/// Todos os que a tabela nomeia.
fn na_tabela() -> BTreeSet<String> {
    // ⚠️ `include_str!` e não um `read_to_string`: se alguém mover este ficheiro, o gate **falha a
    // compilar** em vez de passar a ler um caminho que já não existe (HOWTO §2.1).
    let texto = include_str!("../../src/ids/live_sections.rs");
    let mut out = BTreeSet::new();
    let Some(corpo) = texto.split("pub const LIVE_SECTIONS").nth(1) else {
        panic!("a tabela LIVE_SECTIONS tem de existir neste ficheiro");
    };
    let corpo = corpo.split("\n];").next().unwrap_or("");
    for linha in corpo.lines() {
        let t = linha.trim();
        let Some(resto) = t.strip_prefix('(') else {
            continue;
        };
        if let Some(nome) = resto.split(',').next() {
            let nome = nome.trim();
            if nome.starts_with("INSP_LIVE_") && nome.ends_with("_SECTION") {
                out.insert(nome.to_string());
            }
        }
    }
    out
}

#[test]
fn every_live_section_is_in_the_table() {
    let decl = declaradas();
    let tab = na_tabela();
    // ⛔ **O PISO**, nas duas grandezas — ver o cabeçalho.
    assert!(
        decl.len() >= 20,
        "a varredura das declarações leu {} — ela está partida, e uma lista vazia passaria em \
         silêncio",
        decl.len()
    );
    assert!(
        tab.len() >= 20,
        "a leitura da tabela leu {} entradas — ela está partida",
        tab.len()
    );

    let faltam: Vec<&String> = decl.difference(&tab).collect();
    assert!(
        faltam.is_empty(),
        "estas secções vivas do Inspector NÃO estão em `LIVE_SECTIONS`: {faltam:?}\n\n\
         ⚠️ O preço está escrito no doc da tabela: quem falta ali não é `mark_collapsible_section`ado \
         e não é `is_section_header_id` — o cabeçalho pinta o chevron e a dobra NÃO PODE acontecer. \
         O sintoma é mudo, e foi assim que as duas da fábrica passaram uma wave inteira."
    );

    // ⭐ **E o outro sentido**: uma entrada na tabela sem `const` declarado é um nome que não
    // compila, logo não pode acontecer — mas uma entrada DUPLICADA pode, e ela faria o
    // `LIVE_SECTION_IDS` ter o mesmo id duas vezes.
    let sobram: Vec<&String> = tab.difference(&decl).collect();
    assert!(
        sobram.is_empty(),
        "a tabela nomeia secções que ninguém declara: {sobram:?}"
    );
}
