//! ⭐⭐ **O CORPO DO GATE POR CRATE** — *«nenhuma palavra deste painel é escrita no fonte»*.
//!
//! Até 2026-09-16 cada painel migrado copiava ~110 linhas do mesmo gate (Hierarquia, Inspector,
//! Painter, Vector) — e a crate existe exactamente para que a régua não tenha N cópias
//! ([`crate`]: *«a segunda cópia é a que diverge»*). A partir dos painéis migrados nesse dia o gate
//! de cada crate é a LISTA dela (prefixo, tabela, excepções) mais três chamadas daqui.
//!
//! ⚠️ **As funções devolvem a lista de acusações, e o gate afirma** — a mensagem de falha fica no
//! gate da crate, com o nome dela, e esta crate continua sem saber o que é um teste.

use std::path::{Path, PathBuf};

use crate::{keys, language_literals};

/// Uma excepção nomeada: `(ficheiro relativo a src/, texto exacto, porquê)`.
pub type Excecao = (&'static str, &'static str, &'static str);

/// O `src/` e a raiz do repo de uma crate em `crates/<nome>/`, a partir do `CARGO_MANIFEST_DIR`.
#[must_use]
pub fn raizes(manifest_dir: &str) -> (PathBuf, PathBuf) {
    let crate_dir = Path::new(manifest_dir);
    let repo = crate_dir
        .parent()
        .and_then(Path::parent)
        .expect("crates/<crate>/ tem dois pais")
        .to_path_buf();
    (crate_dir.join("src"), repo)
}

/// ⭐⭐⭐ **Os textos com cara de língua escritos no fonte**, fora das excepções — um por linha,
/// `rel:linha · "texto"`.
#[must_use]
pub fn intrusos(src_root: &Path, excecoes: &[Excecao]) -> Vec<String> {
    language_literals(src_root)
        .into_iter()
        .filter(|l| !excecoes.iter().any(|(f, t, _)| l.rel == *f && l.text == *t))
        .map(|l| format!("{}:{} · {:?}", l.rel, l.line, l.text))
        .collect()
}

/// ⭐ **A METADE JUSTA**: as excepções sem mecanismo escrito (`porquê` curto demais) ou que já não
/// abrigam literal nenhum. ⚠️ É também o controlo de vacuidade: uma régua partida devolve zero
/// literais — e aí TODA excepção aparece aqui.
#[must_use]
pub fn excecoes_mortas(src_root: &Path, excecoes: &[Excecao]) -> Vec<String> {
    let hits = language_literals(src_root);
    let mut out = Vec::new();
    for (file, text, why) in excecoes {
        if why.len() <= 40 {
            out.push(format!(
                "`{file}` · {text:?}: não diz o mecanismo — uma lista sem mecanismo é uma licença"
            ));
        }
        if !hits.iter().any(|l| l.rel == *file && l.text == *text) {
            out.push(format!(
                "`{file}` · {text:?}: já não abriga literal nenhum (ou a régua ficou cega) — apague a \
                 linha"
            ));
        }
    }
    out
}

/// O censo das chaves de um prefixo.
#[derive(Debug)]
pub struct Chaves {
    /// Quantas a tabela declara.
    pub declaradas: usize,
    /// Quantas o repo usa (fora da tabela).
    pub usadas: usize,
    /// Usadas e NÃO declaradas — o `tr` pintaria o identificador cru.
    pub sem_traducao: Vec<String>,
    /// Declaradas e não usadas por ninguém.
    pub orfas: Vec<String>,
}

/// ⭐⭐ **As chaves `prefixo*` dos dois lados** — usadas em todo o repo, declaradas nas `tabelas`.
#[must_use]
pub fn chaves(repo: &Path, prefixo: &str, tabelas: &[&str]) -> Chaves {
    let usadas = keys::keys_used(repo, prefixo, tabelas);
    let declaradas = keys::keys_declared(repo, tabelas, prefixo);
    Chaves {
        declaradas: declaradas.len(),
        usadas: usadas.len(),
        sem_traducao: usadas
            .iter()
            .filter(|(k, _)| !declaradas.contains(*k))
            .map(|(k, f)| format!("{k}  (usada em {f})"))
            .collect(),
        orfas: declaradas
            .iter()
            .filter(|k| !usadas.contains_key(*k))
            .cloned()
            .collect(),
    }
}
