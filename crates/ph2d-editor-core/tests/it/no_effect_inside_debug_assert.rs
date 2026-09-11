//! **Nenhuma ESCRITA mora dentro de um `debug_assert`.**
//!
//! `debug_assert!` e irmãos **apagam o argumento inteiro** num build de release. Uma chamada que
//! só AFIRMA some sem consequência; uma que **ESCREVE** leva o produto com ela — e em silêncio,
//! porque a suíte de debug continua verde.
//!
//! ⚠️ **Isto não é hipótese: custou um botão.** O `Reset This Mode` dos tokens fazia
//! `debug_assert_eq!(set_color_overrides(keep), 0)`, então num build de release ele **não fazia
//! nada** — nem erro, nem aviso, nem um pixel diferente. Só a suíte em RELEASE o via, e ela não
//! corre por default.
//!
//! A cura é mecânica e não custa nada: **o valor primeiro, a asserção depois** — a asserção só
//! precisa do NÚMERO.
//!
//! ```ignore
//! let dropped = set_color_overrides(keep);   // a escrita acontece SEMPRE
//! debug_assert_eq!(dropped, 0);              // a afirmação é que some em release
//! ```

use std::path::{Path, PathBuf};

/// Os verbos que denunciam uma ESCRITA. Uma lista pequena e específica de propósito: ela é o
/// vocabulário de mutação deste repo, e um verbo novo entra aqui quando aparecer — o gate é uma
/// rede fina sobre a forma conhecida, não um analisador.
const WRITERS: &[&str] = &[
    "set_", "push", "insert", "remove", "clear", "take", "drain", "write", "replace", "swap",
];

fn workspace_root() -> PathBuf {
    // `crates/ph2d-editor-core` -> a raiz.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("a raiz da workspace")
        .to_path_buf()
}

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            rs_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// `true` se a linha é um `debug_assert*` cujo argumento CHAMA um dos [`WRITERS`].
///
/// ⚠️ Ela pede o parêntese logo depois do verbo: `debug_assert!(dropped == 0)` lê um valor já
/// computado e passa; `debug_assert_eq!(set_x(v), 0)` chama e não passa. É a diferença exata
/// entre a forma segura e a que apaga produto.
fn writes_inside_a_debug_assert(line: &str) -> bool {
    let t = line.trim_start();
    if !t.starts_with("debug_assert") {
        return false;
    }
    let Some(args) = t.split_once('(').map(|(_, r)| r) else {
        return false;
    };
    WRITERS.iter().any(|w| {
        args.match_indices(w).any(|(i, _)| {
            // O verbo tem de ser o começo de um identificador (senão `offset` casa `set_`)…
            let before_ok = i == 0
                || !args.as_bytes()[i - 1].is_ascii_alphanumeric()
                    && args.as_bytes()[i - 1] != b'_';
            // …e tem de ser CHAMADO: o próximo `(` vem antes de qualquer separador.
            let rest = &args[i + w.len()..];
            let call = rest
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .is_some_and(|j| rest.as_bytes()[j] == b'(');
            before_ok && call
        })
    })
}

#[test]
fn no_write_hides_inside_a_debug_assert() {
    let root = workspace_root();
    let mut files = Vec::new();
    for dir in ["crates", "shells"] {
        rs_files(&root.join(dir), &mut files);
    }
    assert!(
        files.len() > 500,
        "a varredura achou {} arquivos — o scanner está quebrado, e um gate que não lê nada \
         passa sempre",
        files.len()
    );

    let mut bad = Vec::new();
    for f in &files {
        // Os arquivos de teste ficam de fora: lá um `debug_assert` que escreve é uma fixture, e
        // o produto não o carrega.
        if f.to_string_lossy().contains("_tests.rs") || f.to_string_lossy().contains("/tests/") {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(f) else {
            continue;
        };
        for (n, line) in src.lines().enumerate() {
            if writes_inside_a_debug_assert(line) {
                bad.push(format!(
                    "{}:{}  {}",
                    f.strip_prefix(&root).unwrap_or(f).display(),
                    n + 1,
                    line.trim()
                ));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "escrita DENTRO de um `debug_assert` — ela some no build de release, levando o produto \
         junto e em silêncio:\n  {}\n\nconserte hoisting o valor: `let x = escreve(); \
         debug_assert_eq!(x, ...);`",
        bad.join("\n  ")
    );
}

/// **O CONTROLE POSITIVO** — o reconhecedor de facto reconhece.
///
/// ⚠️ Sem ele, um scanner que devolvesse `false` para tudo deixaria o gate acima VERDE para
/// sempre [[feedback_a_negative_search_needs_a_positive_control]]. As linhas seguras estão aqui
/// pelo mesmo motivo: um reconhecedor que diz sim a tudo também "passa".
#[test]
fn the_recogniser_recognises() {
    for bad in [
        "        debug_assert_eq!(set_color_overrides(keep), 0);",
        "debug_assert!(map.insert(k, v).is_none());",
        "  debug_assert_eq!(buf.take(), None);",
        "debug_assert!(list.push_back(x).is_ok());",
    ] {
        assert!(writes_inside_a_debug_assert(bad), "nao reconheceu: {bad}");
    }
    for ok in [
        // O padrão CURADO: o valor já foi computado fora.
        "        debug_assert_eq!(dropped, 0);",
        // Uma LEITURA, ainda que o nome contenha um verbo por dentro.
        "debug_assert!(offset_of(x) > 0);",
        // Um campo, não uma chamada.
        "debug_assert_eq!(self.inserted, 3);",
        // Não é asserção nenhuma.
        "let n = set_color_overrides(keep);",
    ] {
        assert!(!writes_inside_a_debug_assert(ok), "falso positivo: {ok}");
    }
}
