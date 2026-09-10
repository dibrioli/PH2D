//! ⭐⭐⭐ **NENHUM RÓTULO DESTA CRATE É ESCRITO DENTRO DO PINTOR** — o HR-15, com instrumento.
//!
//! > `CLAUDE.md` §0.3: *«UI canônica: zero hex, zero `f32` literal de UI, **zero string
//! > hardcoded** — tudo via tokens / i18n (HR-15).»*
//!
//! # ⛔⛔ Por que este gate é POR CRATE, e não do repo
//!
//! O censo de 2026-09-10 (`scripts/censo-texto-pintado.py`) conta **438** literais pintados em
//! **19** crates, e **11 delas são de outras linhas**. Uma catraca global com essa dívida dentro
//! poria *a próxima linha que pinte um rótulo vermelha por causa de um gate desta* — o contrário
//! do que o `CLAUDE.md` §0.2 pede de um toque foundational (*projecte-o para isolamento*).
//!
//! ⇒ este gate fala **só da `ph2d-editor-core`**, que é a crate desta linha, e é o **MOLDE**: cada
//! linha dona de uma crate de painel copia-o com a lista dela.
//!
//! # ⚠️ A régua é um PONTO FIXO, porque o censo anterior conhecia UMA porta de 127
//!
//! Um literal chega ao ecrã se for passado no argumento de texto de um pintor — **ou** a uma função
//! que repassa esse parâmetro a um pintor, e isso é **recursivo**:
//!
//! ```text
//! paint_text(ts, scene, texto, …)                                      ← a semente
//! fn paint_left_label(.., texto: &str, ..) { paint_text(.., texto, ..) }        ← 1.º grau
//! fn paint_slider_chip_row(.., rotulo: &str, ..) { paint_left_label(.., rotulo, ..) }  ← 2.º grau
//! ```
//!
//! ⛔⛔ **A primeira medição desta grandeza saiu `4×` errada** por parar na semente: ela publicou
//! `108` e o número é `438`. *Um censo textual tem de saber TODAS as formas do que lê* — e aqui a
//! forma não é um nome, é um caminho.
//!
//! # ⚠️ O que ele NÃO vê, declarado
//!
//! Literais que chegam por variável, `const`, tabela de `&str` ou `format!`. ⇒ **zero acusados
//! aqui não prova zero literais**; prova que ninguém os escreveu na chamada. *Uma régua que não
//! declara o alcance dela é uma régua que alguém vai ler como maior do que é.*

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// ⭐ As excepções, **com o mecanismo** — nunca uma lista aberta.
///
/// ⚠️ Uma lista de dívida tolerada que não diz *porquê* é uma licença (`CLAUDE.md` §5.0). Cada
/// entrada é `(caminho relativo a `src/`, porquê)`, e o teste irmão exige que ela ainda descreva
/// alguma coisa.
const NOT_LANGUAGE: &[(&str, &str)] = &[
    (
        "widget/showcase",
        "A GALERIA DE WIDGETS e' uma BANCADA, nao uma superficie de produto -- o subtitulo dela \
         di-lo: «reference for peripheral agents». Os rotulos sao os NOMES DOS NOSSOS WIDGETS \
         (`Rect2Editor`, `BitmaskGrid32`, `VariantEditor (recursive, depth <=4)`) e o conteudo de \
         amostra que os demonstra. Traduzir o nome de um tipo nosso nao e' i18n, e' ruido -- e a \
         bancada tem o mesmo estatuto do `widget-lab`, que ate' a aparencia forca para o \
         redesenho, «porque e' onde ele se estuda».",
    ),
    (
        "widget/command_palette/header.rs",
        "SIMBOLOS DESENHADOS COMO LETRAS: o «X» e' o fecho e o «x» e' o visto. O proprio ficheiro \
         ja' o dizia -- «o mesmo idioma do X de fechar, que tambem e' uma letra». Uma lingua nova \
         nao os traduz; um icone e' que os substituiria, e isso e' outra obra.",
    ),
];

fn src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// O fonte sem comentários **e sem o interior das strings mexido** — a prosa não pinta nada, e um
/// `//` dentro de uma string não abre comentário.
fn strip_comments(src: &str) -> String {
    let b: Vec<char> = src.chars().collect();
    let mut out = String::with_capacity(src.len());
    let (mut i, n) = (0usize, b.len());
    let (mut in_str, mut esc) = (false, false);
    while i < n {
        let c = b[i];
        if in_str {
            out.push(c);
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(c);
            i += 1;
            continue;
        }
        if c == '/' && i + 1 < n && b[i + 1] == '/' {
            while i < n && b[i] != '\n' {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        if c == '/' && i + 1 < n && b[i + 1] == '*' {
            let mut d = 0usize;
            while i < n {
                if b[i] == '/' && i + 1 < n && b[i + 1] == '*' {
                    d += 1;
                    out.push_str("  ");
                    i += 2;
                } else if b[i] == '*' && i + 1 < n && b[i + 1] == '/' {
                    d -= 1;
                    out.push_str("  ");
                    i += 2;
                    if d == 0 {
                        break;
                    }
                } else {
                    out.push(if b[i] == '\n' { '\n' } else { ' ' });
                    i += 1;
                }
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Os argumentos de topo de uma chamada cujo `(` está em `open`.
fn args_of(s: &[char], open: usize) -> Vec<String> {
    let (mut depth, mut i) = (0usize, open);
    let (mut in_str, mut esc) = (false, false);
    let mut start = open + 1;
    let mut parts = Vec::new();
    while i < s.len() {
        let c = s[i];
        if in_str {
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        match c {
            '"' => in_str = true,
            '(' | '[' | '{' => {
                depth += 1;
                if depth == 1 {
                    start = i + 1;
                }
            }
            ')' | ']' | '}' => {
                depth -= 1;
                if depth == 0 {
                    parts.push(s[start..i].iter().collect());
                    return parts;
                }
            }
            ',' if depth == 1 => {
                parts.push(s[start..i].iter().collect());
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    parts
}

/// O identificador imediatamente antes de um `(` — o nome do que está a ser chamado.
fn callee(s: &[char], open: usize) -> String {
    let mut j = open;
    while j > 0 && s[j - 1].is_whitespace() {
        j -= 1;
    }
    let end = j;
    while j > 0 && (s[j - 1].is_alphanumeric() || s[j - 1] == '_') {
        j -= 1;
    }
    s[j..end].iter().collect()
}

struct File {
    rel: String,
    chars: Vec<char>,
}

fn files() -> Vec<File> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<File>) {
        let Ok(rd) = fs::read_dir(dir) else { return };
        let mut entries: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                if p.file_name().is_some_and(|n| n == "tests") {
                    continue;
                }
                walk(&p, root, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
                if name == "tests.rs" || name.ends_with("_tests.rs") {
                    continue;
                }
                if let Ok(s) = fs::read_to_string(&p) {
                    out.push(File {
                        rel: p
                            .strip_prefix(root)
                            .unwrap_or(&p)
                            .to_string_lossy()
                            .replace('\\', "/"),
                        chars: strip_comments(&s).chars().collect(),
                    });
                }
            }
        }
    }
    let root = src_root();
    let mut v = Vec::new();
    walk(&root, &root, &mut v);
    v
}

/// `nome da fn -> índices dos parâmetros `&str``, para toda função da crate.
fn signatures(files: &[File]) -> BTreeMap<String, BTreeSet<(usize, String)>> {
    let mut out: BTreeMap<String, BTreeSet<(usize, String)>> = BTreeMap::new();
    for f in files {
        let mut i = 0usize;
        while let Some(k) = find_from(&f.chars, i, "fn ") {
            i = k + 3;
            // o nome
            let mut j = i;
            while j < f.chars.len() && f.chars[j].is_whitespace() {
                j += 1;
            }
            let s = j;
            while j < f.chars.len() && (f.chars[j].is_alphanumeric() || f.chars[j] == '_') {
                j += 1;
            }
            if j == s {
                continue;
            }
            let name: String = f.chars[s..j].iter().collect();
            // salta genéricos e chega ao `(`
            while j < f.chars.len() && f.chars[j] != '(' && f.chars[j] != '{' && f.chars[j] != ';' {
                j += 1;
            }
            if j >= f.chars.len() || f.chars[j] != '(' {
                continue;
            }
            for (idx, a) in args_of(&f.chars, j).iter().enumerate() {
                let a = a.trim();
                if let Some((lhs, rhs)) = a.split_once(':') {
                    let rhs = rhs.trim();
                    let lhs = lhs.trim().trim_start_matches("mut ").trim();
                    if rhs.starts_with('&')
                        && rhs.trim_start_matches('&').trim_start().starts_with("str")
                        && lhs.chars().all(|c| c.is_alphanumeric() || c == '_')
                        && !lhs.is_empty()
                    {
                        out.entry(name.clone())
                            .or_default()
                            .insert((idx, lhs.to_string()));
                    }
                }
            }
        }
    }
    out
}

fn find_from(s: &[char], from: usize, pat: &str) -> Option<usize> {
    let p: Vec<char> = pat.chars().collect();
    (from..s.len().saturating_sub(p.len() - 1)).find(|&i| s[i..i + p.len()] == p[..])
}

/// A função que contém a posição `at`, se alguma.
fn host_of(sig_starts: &[(usize, String)], at: usize) -> Option<&String> {
    sig_starts
        .iter()
        .take_while(|(s, _)| *s < at)
        .last()
        .map(|(_, n)| n)
}

/// ⭐⭐⭐ **O PONTO FIXO:** semente = `paint_*`/`draw_*` com um `&str`; depois cresce por quem
/// repassa o próprio parâmetro a uma porta já conhecida.
fn doors(files: &[File]) -> BTreeMap<String, BTreeSet<usize>> {
    let sigs = signatures(files);
    let mut doors: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
    for (n, slots) in &sigs {
        if n.starts_with("paint_") || n.starts_with("draw_") {
            doors.insert(n.clone(), slots.iter().map(|(i, _)| *i).collect());
        }
    }
    for _ in 0..12 {
        let mut grew = false;
        for f in files {
            let starts = fn_starts(f);
            let mut i = 0usize;
            while i < f.chars.len() {
                if f.chars[i] != '(' {
                    i += 1;
                    continue;
                }
                let name = callee(&f.chars, i);
                let Some(slots) = doors.get(&name).cloned() else {
                    i += 1;
                    continue;
                };
                let parts = args_of(&f.chars, i);
                if let Some(host) = host_of(&starts, i)
                    && let Some(hslots) = sigs.get(host)
                {
                    for idx in &slots {
                        let Some(a) = parts.get(*idx) else { continue };
                        let a = a.trim().trim_start_matches('&').trim();
                        for (hidx, hname) in hslots {
                            if a == hname && doors.entry(host.clone()).or_default().insert(*hidx) {
                                grew = true;
                            }
                        }
                    }
                }
                i += 1;
            }
        }
        if !grew {
            break;
        }
    }
    doors
}

fn fn_starts(f: &File) -> Vec<(usize, String)> {
    let mut v = Vec::new();
    let mut i = 0usize;
    while let Some(k) = find_from(&f.chars, i, "fn ") {
        i = k + 3;
        let mut j = i;
        while j < f.chars.len() && f.chars[j].is_whitespace() {
            j += 1;
        }
        let s = j;
        while j < f.chars.len() && (f.chars[j].is_alphanumeric() || f.chars[j] == '_') {
            j += 1;
        }
        if j > s {
            v.push((k, f.chars[s..j].iter().collect()));
        }
    }
    v
}

/// Um argumento que é **só** um literal de string, e que tem pelo menos uma letra.
fn literal_of(arg: &str) -> Option<String> {
    let t = arg.trim();
    let inner = t.strip_prefix('"')?.strip_suffix('"')?;
    if inner.contains('"') || !inner.chars().any(char::is_alphabetic) {
        return None;
    }
    Some(inner.to_string())
}

/// `(caminho, porta, literal)` de tudo o que esta crate escreve dentro do pintor.
fn painted_literals() -> Vec<(String, String, String)> {
    let files = files();
    let doors = doors(&files);
    let mut out = Vec::new();
    for f in &files {
        let mut i = 0usize;
        while i < f.chars.len() {
            if f.chars[i] != '(' {
                i += 1;
                continue;
            }
            let name = callee(&f.chars, i);
            if let Some(slots) = doors.get(&name) {
                let parts = args_of(&f.chars, i);
                for idx in slots {
                    if let Some(a) = parts.get(*idx)
                        && let Some(lit) = literal_of(a)
                    {
                        out.push((f.rel.clone(), name.clone(), lit));
                        break;
                    }
                }
            }
            i += 1;
        }
    }
    out
}

/// ⭐⭐⭐ **A crate desta linha não escreve rótulos dentro do pintor.**
#[test]
fn every_label_this_crate_paints_comes_from_the_string_table() {
    let hits = painted_literals();

    // ⛔ **CONTROLO DE VACUIDADE, com a metade justa:** um ponto fixo partido devolve zero portas e
    //    zero acusados, e lê-se como aprovado. As excepções existem e TÊM de ser encontradas.
    assert!(
        hits.len() >= 20,
        "o ponto fixo achou só {} literais — ele está partido, e um censo partido lê-se como \
         aprovado (as excepções declaradas sozinhas são mais do que isto)",
        hits.len()
    );

    let intrusos: Vec<String> = hits
        .iter()
        .filter(|(rel, _, _)| !NOT_LANGUAGE.iter().any(|(e, _)| rel.starts_with(e)))
        .map(|(rel, door, lit)| format!("{rel} · {door}(\"{lit}\")"))
        .collect();
    assert!(
        intrusos.is_empty(),
        "estes rótulos são escritos dentro do pintor e nunca chegam à tabela de strings \
         (HR-15):\n  {}\n\nA cura é uma chave em `crates/ph2d-i18n/src/chrome.rs` e um \
         `ph2d_i18n::tr(\"chrome.…\")` na chamada. ⚠️ Se o texto NÃO é língua (o nome de um tipo \
         nosso, um símbolo desenhado como letra), a cura é uma linha em `NOT_LANGUAGE` **com o \
         mecanismo** — nunca sem ele.",
        intrusos.join("\n  ")
    );
}

/// ⭐ **A METADE JUSTA: cada excepção ainda descreve alguma coisa?**
///
/// ⛔⛔ Uma catraca sem censo de obsolescência não desce: ela vira licença (`CLAUDE.md` §5.0).
#[test]
fn every_named_exception_still_shelters_a_real_literal() {
    let hits = painted_literals();
    for (path, motivo) in NOT_LANGUAGE {
        assert!(
            motivo.len() > 40,
            "a excepção `{path}` não diz o mecanismo — uma lista sem mecanismo é uma licença"
        );
        let n = hits
            .iter()
            .filter(|(rel, _, _)| rel.starts_with(path))
            .count();
        assert!(
            n > 0,
            "a excepção `{path}` já não abriga literal nenhum — apague a linha, senão ela fica \
             aberta para o próximo ficheiro que caia debaixo desse caminho"
        );
    }
}
