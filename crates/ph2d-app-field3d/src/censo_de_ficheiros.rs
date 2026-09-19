//! ⭐⭐⭐ **O QUE É CÓDIGO DE TESTE DERIVA-SE — nunca se adivinha pelo NOME do ficheiro.**
//!
//! # ⛔⛔⛔ Porque esta porta existe: o mesmo defeito, três vezes no mesmo directório
//!
//! Três censos desta crate varrem `src/` à procura de uma chamada e perguntam *«este ficheiro é
//! produto?»* respondendo **`!nome.ends_with("_tests.rs")`**. ⚠️ E o comentário de um deles já
//! escreve a fronteira certa, à letra: *«a fronteira certa não é "este ficheiro": é "código que
//! corre no app"»* — e a linha seguinte implementa a errada.
//!
//! O modo de falha é **barulhento e enganador**: em 2026-09-19 uma SONDA nova
//! ([`crate::preview::borda_sondas`], compilada só sob `#[cfg(test)]` e sem o sufixo) foi lida como
//! PRODUTO, e o censo do engrossamento reprovou com a mensagem *«o `Resolution` do artista deixa de
//! ter efeito observável»* sobre um produto **correcto**. ⇒ *um censo que classifica por nome acusa
//! o autor do ficheiro seguinte que não seguir a convenção.*
//!
//! ⚠️ **É a mesma lei que a `line/sculpt3d` pagou em 15/09** (`CLAUDE.md` §5), e a cura é a mesma:
//! **o que um ficheiro de teste declara por `#[path]` é código de teste**, transitivamente.
//!
//! # ⚠️ E a direcção MUDA é a perigosa
//!
//! Excluir a mais deixa um censo **verde a medir nada**. É por isso que esta porta tem controlo
//! próprio ([`o_censo_separa_a_sonda_do_produto`]) e que os três leitores comparam contra uma lista
//! **nomeada** de chamadores — um `assert_eq!` contra `["smoke_draw.rs"]` reprova tanto por um
//! chamador a mais como por um a menos.

use std::collections::BTreeSet;
use std::path::Path;

/// ⭐⭐⭐ **Os ficheiros de `src/` que só existem sob `#[cfg(test)]`**, derivados das declarações.
///
/// Ele colhe todo `#[path = "X.rs"]` cuja linha anterior (saltando doc-comments) seja
/// `#[cfg(test)]`, e depois **fecha transitivamente**: o que um ficheiro já classificado como teste
/// declara também é teste, com ou sem `cfg` próprio.
#[must_use]
pub(crate) fn ficheiros_de_teste(dir: &Path) -> BTreeSet<String> {
    let mut texto: Vec<(String, String)> = Vec::new();
    for entry in std::fs::read_dir(dir)
        .expect("o directório `src` existe")
        .flatten()
    {
        let p = entry.path();
        if p.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let nome = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        if let Ok(s) = std::fs::read_to_string(&p) {
            texto.push((nome, s));
        }
    }
    // A semente: o que o produto declara SOB `#[cfg(test)]`, mais a convenção de nome — ela não é a
    // régua, mas um ficheiro que a siga também é teste e ignorá-lo seria excluir a menos.
    let mut teste: BTreeSet<String> = texto
        .iter()
        .map(|(n, _)| n.clone())
        .filter(|n| n.ends_with("_tests.rs"))
        .collect();
    for (_, s) in &texto {
        teste.extend(declarados_sob_cfg_test(s));
    }
    // O fecho transitivo: um ficheiro de teste pode declarar irmãos.
    loop {
        let antes = teste.len();
        let novos: BTreeSet<String> = texto
            .iter()
            .filter(|(n, _)| teste.contains(n))
            .flat_map(|(_, s)| declarados(s))
            .collect();
        teste.extend(novos);
        if teste.len() == antes {
            return teste;
        }
    }
}

/// Os `#[path = "X.rs"]` cuja declaração anterior é um `#[cfg(test)]`.
fn declarados_sob_cfg_test(src: &str) -> Vec<String> {
    let linhas: Vec<&str> = src.lines().collect();
    let mut out = Vec::new();
    for (i, l) in linhas.iter().enumerate() {
        let Some(nome) = caminho(l) else { continue };
        // Salta os doc-comments entre o `cfg` e o `path`.
        let anterior = linhas[..i]
            .iter()
            .rev()
            .find(|p| !p.trim_start().starts_with("///") && !p.trim().is_empty());
        if anterior.is_some_and(|p| p.trim() == "#[cfg(test)]") {
            out.push(nome);
        }
    }
    out
}

/// Todos os `#[path = "X.rs"]` de um ficheiro, com `cfg` ou sem ele.
fn declarados(src: &str) -> Vec<String> {
    src.lines().filter_map(caminho).collect()
}

/// `#[path = "X.rs"]` → `X.rs`.
fn caminho(linha: &str) -> Option<String> {
    let t = linha.trim();
    let resto = t.strip_prefix("#[path = \"")?;
    let nome = resto.strip_suffix("\"]")?;
    nome.ends_with(".rs").then(|| nome.to_string())
}

/// ⭐ **O CONTROLO da porta** — sem ele, uma derivação que devolvesse o directório inteiro deixaria
/// os três censos verdes a medir zero ficheiros, e uma que devolvesse o conjunto vazio deixá-los-ia
/// a acusar toda sonda nova.
#[test]
fn o_censo_separa_a_sonda_do_produto() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let teste = ficheiros_de_teste(&dir);
    for n in [
        "borda_sondas.rs",
        "borda_tests.rs",
        "device_probes.rs",
        "preview_device_tests.rs",
    ] {
        assert!(
            teste.contains(n),
            "`{n}` é compilado só sob `#[cfg(test)]` e o censo leu-o como PRODUTO — um censo por \
             texto passaria a acusar quem lá escrever uma chamada legítima"
        );
    }
    for n in ["smoke_draw.rs", "gpu_frame.rs", "preview.rs", "smoke.rs"] {
        assert!(
            !teste.contains(n),
            "`{n}` é produto e o censo excluiu-o — a direcção MUDA: os três censos que lêem esta \
             porta ficariam verdes a medir menos do que julgam"
        );
    }
}
