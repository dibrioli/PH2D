//! ⛔⛔⛔ **NENHUM `.rs` DA WORKSPACE FICA FORA DO BUILD** — o gate de um defeito que é MUDO por natureza.
//!
//! # O defeito
//!
//! Um ficheiro `.rs` que nenhum `mod` declara **não é compilado**. Ele fica no disco, no git e no
//! diff — e fora do compilador. Nada o acusa: `cargo check` verde, `clippy` verde, a suíte verde com
//! testes a menos, e ninguém conta os que faltam.
//!
//! # O que ele custou, medido (W2, 2026-09-12)
//!
//! | ficheiro | o que se perdeu | desde |
//! |---|---|---|
//! | `ph2d-app-vec/src/bool_reach_tests.rs` | 3 gates | a Fase D — apanhado pelo `ONLY-A` da prova de perda |
//! | `ph2d-vec-scene/src/arrows_tests.rs` | **os 8 testes das setas** | `ea2817b73` («remove a seta circular») |
//! | `ph2d-vec-art-live/src/brush_cost_probe.rs` | uma sonda de **583 linhas** | `fe7d9b93a` (a Fase C) |
//!
//! Os dois últimos só apareceram quando a integração da Fase D varreu a workspace inteira lendo os
//! `.d` que o compilador escreve. ⇒ este gate é esse censo, feito permanente.
//!
//! # ⚠️⚠️ As DUAS metades da mesma auditoria
//!
//! *Quantos pais tem este ficheiro?* **Zero** é o órfão (deixa de correr); **dois** é o DUPLICADO —
//! o mesmo ficheiro declarado por dois `mod` na **mesma** crate compila como dois módulos, os testes
//! dele correm a dobrar e a contagem sobe sem ninguém ter escrito um teste. Os dois são mudos, e na
//! W2/L2 Fase B (11/09) nasceram **13 órfãos e 3 duplicados** ao mesmo tempo — *uma contagem total
//! sozinha não os vê, porque se cancelam* (`project-memory/feedback_the_orphan_and_the_double_
//! declaration_are_one_audit.md`, que prescrevia este instrumento um dia antes de ele existir).
//!
//! ⚠️ **O duplicado mede-se POR RAIZ, nunca por membro:** a `lib` e cada ficheiro de `tests/` são
//! crates distintas, e o mesmo ficheiro alcançado por duas delas é partilha legítima, não duplicado.
//!
//! # Como resolve (a regra do `rustc`)
//!
//! A partir das RAÍZES de cada membro — `src/lib.rs`, `src/main.rs`, `build.rs`, `src/bin/`,
//! `tests/`, `examples/`, `benches/`, e todo `path =` / `build =` do manifesto — segue:
//! - `mod x;` → `x.rs` ou `x/mod.rs`, na pasta do ficheiro se ele é raiz, `lib.rs`, `main.rs`,
//!   `mod.rs` **ou foi carregado por `#[path]`**; senão numa subpasta com o nome dele
//!   (`foo.rs` → `foo/x.rs`);
//! - `#[path = "p"] mod x;` → `p`, relativo à pasta do ficheiro (mais os `mod a { }` em linha);
//! - `include!("p.rs")`.
//!
//! ⚠️⚠️ **A regra do `#[path]` é a que a primeira versão errou**: um ficheiro carregado por `#[path]`
//! resolve os filhos na PRÓPRIA pasta, como um `mod.rs`. Tratado como ficheiro normal, o
//! `measure_preview_drain.rs` do Painter lia-se órfão — e quem o apanhou foi o ORÁCULO.
//!
//! # ⚠️ Porque é textual, e com que oráculo foi validado
//!
//! Os `.d` do compilador são a verdade sobre o que foi CONSTRUÍDO, mas são cegos ao gémeo desligado
//! (`shells/desktop/src/sculpt3d_absent.rs` é declarado e nunca construído com a feature ligada) e
//! exigem uma build. Este gate lê DECLARAÇÕES, e foi validado contra os `.d` em 12/09: `0` órfãos
//! nas 7 545 fontes de 361 membros, os controlos abaixo todos alcançados, e **provado por mutação**
//! (apagar a declaração da sonda faz este censo acusar exactamente esse ficheiro, e mais nenhum).
//!
//! ⛔ **Não há lista de tolerância, de propósito.** Um órfão ou se declara ou se apaga; uma entrada
//! de «órfão aceite» seria um ficheiro que ninguém compila com licença para continuar assim.
//!
//! Sem deps (só `std`), como os gates de arquitectura irmãos.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/ph2d-editor-core; dois pais = a raiz da workspace.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

/// O corpo de uma tabela TOML (`[workspace]`), até ao cabeçalho seguinte.
fn table(toml: &str, header: &str) -> String {
    let mut dentro = false;
    let mut out = String::new();
    for l in toml.lines() {
        let t = l.trim();
        if t.starts_with('[') {
            dentro = t == header;
            continue;
        }
        if dentro {
            out.push_str(l);
            out.push('\n');
        }
    }
    out
}

/// As strings de um array TOML `chave = [ ... ]`, que pode ocupar várias linhas.
fn toml_string_array(corpo_da_tabela: &str, chave: &str) -> Vec<String> {
    let linhas: Vec<&str> = corpo_da_tabela.lines().collect();
    let Some(i) = linhas.iter().position(|l| {
        l.trim_start()
            .strip_prefix(chave)
            .is_some_and(|r| r.trim_start().starts_with('='))
    }) else {
        return Vec::new();
    };
    let mut buf = String::new();
    for l in &linhas[i..] {
        let sem_comentario = l.split('#').next().unwrap_or("");
        buf.push_str(sem_comentario);
        buf.push('\n');
        if sem_comentario.contains(']') {
            break;
        }
    }
    let corpo = buf.split_once('[').map_or("", |(_, r)| r);
    let corpo = corpo.split(']').next().unwrap_or("");
    let cs: Vec<char> = corpo.chars().collect();
    let mut out = Vec::new();
    let mut k = 0;
    while k < cs.len() {
        let q = cs[k];
        if q == '"' || q == '\'' {
            let ini = k + 1;
            let mut fim = ini;
            while fim < cs.len() && cs[fim] != q {
                fim += 1;
            }
            out.push(cs[ini..fim].iter().collect());
            k = fim + 1;
        } else {
            k += 1;
        }
    }
    out
}

/// Os membros da workspace, expandidos do `[workspace] members` (só `<dir>/*` e caminhos literais),
/// sem os do `exclude`.
fn members(root: &Path) -> Vec<PathBuf> {
    let toml = fs::read_to_string(root.join("Cargo.toml")).expect("o Cargo.toml da raiz");
    let ws = table(&toml, "[workspace]");
    let padroes = toml_string_array(&ws, "members");
    assert!(
        padroes.iter().any(|p| p == "crates/*"),
        "controlo do parser: `[workspace] members` não mostrou `crates/*` — leu {padroes:?}"
    );
    let excluidos: BTreeSet<PathBuf> = toml_string_array(&ws, "exclude")
        .iter()
        .filter_map(|e| fs::canonicalize(root.join(e)).ok())
        .collect();
    let mut out = BTreeSet::new();
    for p in &padroes {
        if let Some(dir) = p.strip_suffix("/*") {
            assert!(
                !dir.contains('*'),
                "padrão de membros não suportado: `{p}` — este gate só expande `<dir>/*`"
            );
            let rd = fs::read_dir(root.join(dir)).unwrap_or_else(|e| panic!("`{dir}`: {e}"));
            for e in rd.flatten() {
                let d = e.path();
                if d.join("Cargo.toml").is_file()
                    && let Ok(c) = fs::canonicalize(&d)
                {
                    out.insert(c);
                }
            }
        } else {
            assert!(
                !p.contains('*'),
                "padrão de membros não suportado: `{p}` — este gate só expande `<dir>/*`"
            );
            if root.join(p).join("Cargo.toml").is_file()
                && let Ok(c) = fs::canonicalize(root.join(p))
            {
                out.insert(c);
            }
        }
    }
    out.into_iter().filter(|m| !excluidos.contains(m)).collect()
}

/// As raízes de compilação de um membro: as convencionais e as que o manifesto declara.
fn roots(m: &Path) -> Vec<PathBuf> {
    let mut r: Vec<PathBuf> = ["src/lib.rs", "src/main.rs", "build.rs"]
        .iter()
        .map(|rel| m.join(rel))
        .collect();
    for sub in ["src/bin", "tests", "examples", "benches"] {
        let Ok(rd) = fs::read_dir(m.join(sub)) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                r.push(p.join("main.rs"));
            } else if p.extension().is_some_and(|x| x == "rs") {
                r.push(p);
            }
        }
    }
    if let Ok(t) = fs::read_to_string(m.join("Cargo.toml")) {
        let mut seccao = String::new();
        for l in t.lines() {
            let s = l.trim();
            if s.starts_with('[') {
                seccao = s.to_string();
                continue;
            }
            let chave = match seccao.as_str() {
                "[lib]" | "[[bin]]" | "[[test]]" | "[[example]]" | "[[bench]]" => "path",
                "[package]" => "build",
                _ => continue,
            };
            if let Some(resto) = s.strip_prefix(chave)
                && let Some(v) = resto.trim_start().strip_prefix('=')
                && let Some(v) = v.trim().strip_prefix('"')
                && let Some(v) = v.split('"').next()
            {
                r.push(m.join(v));
            }
        }
    }
    r.into_iter()
        .filter(|p| p.is_file())
        .filter_map(|p| fs::canonicalize(p).ok())
        .collect()
}

/// `mod x;` ou `mod x {` (com `pub`, `pub(...)` e atributos na mesma linha) → `(x, ';' | '{')`.
fn mod_decl(line: &str) -> Option<(String, char)> {
    let mut t = line.trim_start();
    while let Some(resto) = t.strip_prefix("#[") {
        let mut prof = 1usize;
        let mut fim = None;
        for (i, c) in resto.char_indices() {
            match c {
                '[' => prof += 1,
                ']' => {
                    prof -= 1;
                    if prof == 0 {
                        fim = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        t = resto[fim? + 1..].trim_start();
    }
    if let Some(resto) = t.strip_prefix("pub") {
        let r = match resto.trim_start().strip_prefix('(') {
            Some(dentro) => &dentro[dentro.find(')')? + 1..],
            None => resto,
        };
        // o modificador separa-se de `mod` por espaço (`pub mod`, `pub(crate) mod`)
        if !r.starts_with(char::is_whitespace) {
            return None;
        }
        t = r.trim_start();
    }
    let resto = t.strip_prefix("mod")?;
    if !resto.starts_with(char::is_whitespace) {
        return None;
    }
    let resto = resto.trim_start();
    let resto = resto.strip_prefix("r#").unwrap_or(resto);
    let ident: String = resto
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if ident.is_empty() || ident.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    match resto[ident.len()..].trim_start().chars().next() {
        Some(c @ (';' | '{')) => Some((ident, c)),
        _ => None,
    }
}

/// O caminho de um `#[path = "..."]` nesta linha.
fn path_attr(line: &str) -> Option<String> {
    let mut hay = line;
    while let Some(i) = hay.find("#[") {
        let resto = &hay[i + 2..];
        if let Some(r) = resto.trim_start().strip_prefix("path")
            && let Some(r) = r.trim_start().strip_prefix('=')
            && let Some(r) = r.trim_start().strip_prefix('"')
            && let Some(fim) = r.find('"')
            && fim > 0
            && r[fim + 1..].trim_start().starts_with(']')
        {
            return Some(r[..fim].to_string());
        }
        hay = resto;
    }
    None
}

/// Os `include!("x.rs")` desta linha.
fn includes(line: &str) -> Vec<String> {
    const MACRO: &str = "include!";
    let mut out = Vec::new();
    let mut hay = line;
    while let Some(i) = hay.find(MACRO) {
        let resto = &hay[i + MACRO.len()..];
        if let Some(r) = resto.trim_start().strip_prefix('(')
            && let Some(r) = r.trim_start().strip_prefix('"')
            && let Some(fim) = r.find('"')
            && r[..fim].ends_with(".rs")
            && r[fim + 1..].trim_start().starts_with(')')
        {
            out.push(r[..fim].to_string());
        }
        hay = resto;
    }
    out
}

/// `{` menos `}` na linha, sem contar os que estão dentro de strings e de literais de char.
fn brace_delta(line: &str) -> i64 {
    let cs: Vec<char> = line.chars().collect();
    let mut d = 0i64;
    let mut i = 0;
    while i < cs.len() {
        match cs[i] {
            '"' => {
                let mut j = i + 1;
                while j < cs.len() && cs[j] != '"' {
                    j += if cs[j] == '\\' { 2 } else { 1 };
                }
                if j < cs.len() {
                    i = j;
                }
            }
            '\'' => {
                if cs.get(i + 1) == Some(&'\\') && cs.get(i + 3) == Some(&'\'') {
                    i += 3;
                } else if cs.get(i + 1).is_some_and(|c| *c != '\'' && *c != '\\')
                    && cs.get(i + 2) == Some(&'\'')
                {
                    i += 2;
                }
            }
            '{' => d += 1,
            '}' => d -= 1,
            _ => {}
        }
        i += 1;
    }
    d
}

/// Segue as declarações a partir de `f`. `mod_rs` = o ficheiro resolve os filhos na própria pasta
/// (raiz, ou carregado por `#[path]`).
fn walk(
    f: &Path,
    mod_rs: bool,
    alcancados: &mut BTreeSet<PathBuf>,
    pais: &mut BTreeMap<PathBuf, Vec<String>>,
) {
    let Ok(f) = fs::canonicalize(f) else {
        return;
    };
    if !alcancados.insert(f.clone()) {
        return;
    }
    let Ok(src) = fs::read_to_string(&f) else {
        return;
    };
    let dir = f.parent().expect("um ficheiro tem pasta").to_path_buf();
    let nome = f.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let base = if mod_rs || matches!(nome, "lib.rs" | "main.rs" | "mod.rs") {
        dir.clone()
    } else {
        dir.join(f.file_stem().and_then(|s| s.to_str()).unwrap_or(""))
    };
    let mut pendente: Option<String> = None;
    let mut em_linha: Vec<(String, i64)> = Vec::new();
    let mut prof = 0i64;
    let mut em_bloco = false;
    for bruta in src.lines() {
        let mut line: &str = bruta;
        if em_bloco {
            match line.find("*/") {
                Some(i) => {
                    em_bloco = false;
                    line = &line[i + 2..];
                }
                None => continue,
            }
        }
        let s = line.trim();
        if s.starts_with("//") {
            continue;
        }
        if s.contains("/*") && !s.contains("*/") {
            em_bloco = true;
        }
        let path_aqui = path_attr(line);
        if let Some(p) = &path_aqui {
            pendente = Some(p.clone());
        }
        for inc in includes(line) {
            if let Ok(c) = fs::canonicalize(dir.join(&inc)) {
                alcancados.insert(c);
            }
        }
        let decl = mod_decl(line);
        if let Some((ident, ';')) = &decl {
            let nomes: Vec<&str> = em_linha.iter().map(|(n, _)| n.as_str()).collect();
            let candidatos: Vec<PathBuf> = match &pendente {
                Some(p) => {
                    let mut d = dir.clone();
                    d.extend(&nomes);
                    vec![d.join(p)]
                }
                None => {
                    let mut d = base.clone();
                    d.extend(&nomes);
                    vec![d.join(format!("{ident}.rs")), d.join(ident).join("mod.rs")]
                }
            };
            let via_path = pendente.is_some();
            if let Some(c) = candidatos.into_iter().find(|c| c.is_file()) {
                // regista o PAI antes de descer — a 2.ª declaração devolve cedo no `walk`
                if let Ok(cc) = fs::canonicalize(&c) {
                    pais.entry(cc)
                        .or_default()
                        .push(format!("{} (`mod {ident}`)", f.display()));
                }
                walk(&c, via_path, alcancados, pais);
            }
            pendente = None;
        }
        if let Some((ident, '{')) = &decl {
            em_linha.push((ident.clone(), prof));
            pendente = None;
        }
        prof += brace_delta(line);
        while em_linha.last().is_some_and(|(_, p0)| prof <= *p0) {
            em_linha.pop();
        }
        if path_aqui.is_none() && !s.is_empty() && !s.starts_with("#[") && decl.is_none() {
            pendente = None;
        }
    }
}

struct Census {
    membros: usize,
    fontes: usize,
    alcancados: BTreeSet<PathBuf>,
    orfaos: Vec<String>,
    duplicados: Vec<String>,
}

fn census() -> &'static Census {
    static C: OnceLock<Census> = OnceLock::new();
    C.get_or_init(|| {
        let root = fs::canonicalize(workspace_root()).expect("a raiz da workspace");
        let membros = members(&root);
        let mut alcancados = BTreeSet::new();
        let mut fontes = 0usize;
        let mut orfaos = Vec::new();
        let mut duplicados = Vec::new();
        let raiz_txt = format!("{}/", root.display());
        for m in &membros {
            let mut deste = BTreeSet::new();
            for r in roots(m) {
                // um percurso POR RAIZ: cada raiz é uma crate, e o duplicado só existe dentro de uma
                let mut desta_raiz = BTreeSet::new();
                let mut pais: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();
                walk(&r, true, &mut desta_raiz, &mut pais);
                for (fich, quem) in pais {
                    if quem.len() >= 2 {
                        let rel = |t: &str| t.replacen(&raiz_txt, "", 1);
                        duplicados.push(format!(
                            "{} — declarado {}× na crate de {}: {}",
                            rel(&fich.display().to_string()),
                            quem.len(),
                            rel(&r.display().to_string()),
                            quem.iter().map(|q| rel(q)).collect::<Vec<_>>().join(" · ")
                        ));
                    }
                }
                deste.extend(desta_raiz);
            }
            let mut pilha = vec![m.clone()];
            while let Some(d) = pilha.pop() {
                let Ok(rd) = fs::read_dir(&d) else {
                    continue;
                };
                for e in rd.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        // o build, e qualquer outro pacote aninhado, não são deste membro
                        if p.file_name().is_some_and(|n| n == "target")
                            || p.join("Cargo.toml").is_file()
                        {
                            continue;
                        }
                        pilha.push(p);
                    } else if p.extension().is_some_and(|x| x == "rs") {
                        fontes += 1;
                        if let Ok(c) = fs::canonicalize(&p)
                            && !deste.contains(&c)
                        {
                            orfaos.push(c.strip_prefix(&root).unwrap_or(&c).display().to_string());
                        }
                    }
                }
            }
            alcancados.extend(deste);
        }
        orfaos.sort();
        duplicados.sort();
        Census {
            membros: membros.len(),
            fontes,
            alcancados,
            orfaos,
            duplicados,
        }
    })
}

#[test]
fn no_source_file_in_the_workspace_is_left_out_of_the_build() {
    let c = census();
    // ⛔ Pisos de população (HOWTO §2.7): uma varredura que perdesse o sujeito leria zero órfãos e
    //    passaria por vacuidade. Medido em 12/09: 361 membros, 7 545 fontes.
    assert!(
        c.membros >= 300,
        "controlo: só {} membros da workspace — o parser do `[workspace]` perdeu o sujeito",
        c.membros
    );
    assert!(
        c.fontes >= 5_000,
        "controlo: só {} ficheiros `.rs` varridos — a varredura perdeu o sujeito",
        c.fontes
    );
    assert!(
        c.orfaos.is_empty(),
        "estes ficheiros estão no disco e NENHUM `mod` / `#[path]` / `include!` / `path =` de \
         manifesto os alcança — logo NÃO SÃO COMPILADOS:\n  {}\n\nSe são código, declare-os; se são \
         testes, declare-os com `#[cfg(test)] #[path = \"...\"] mod ...;` no módulo cuja lei \
         exercitam. ⛔ Apagar não é a cura por omissão: um órfão pode ser trabalho que se perdeu \
         numa mudança de casa — foi o caso dos três de 2026-09-12 (HOWTO §2.14).",
        c.orfaos.join("\n  ")
    );
}

/// **O instrumento alcança os casos DIFÍCEIS** — cada um é uma regra de resolução que, errada, faria
/// o censo acusar código vivo (e ser apagado por descrédito) ou aceitar um órfão.
#[test]
fn the_census_reaches_every_hard_case() {
    let c = census();
    let root = fs::canonicalize(workspace_root()).expect("a raiz da workspace");
    let casos: &[(&str, &str)] = &[
        (
            "órfão curado em 12/09 (a sonda da Fase C)",
            "crates/ph2d-vec-art-live/src/brush_cost_probe.rs",
        ),
        (
            "órfão curado em 12/09 (as setas, desde ea2817b73)",
            "crates/ph2d-vec-scene/src/arrows_tests.rs",
        ),
        (
            "gémeo DESLIGADO — declarado, nunca construído por omissão",
            "shells/desktop/src/sculpt3d_absent.rs",
        ),
        (
            "`foo.rs` + `foo/` — os filhos vivem numa subpasta",
            "crates/ph2d-app-audio/src/editor/batch.rs",
        ),
        (
            "submódulo de `tests/it/main.rs` — e é este próprio gate",
            "crates/ph2d-editor-core/tests/it/architecture_no_orphan_source_file.rs",
        ),
        (
            "`[[bin]] path =` do manifesto",
            "crates/ph2d-physics-ecs/src/bin/physics_ecs_c9/main.rs",
        ),
        (
            "`include!(\"panel.rs\")`",
            "crates/ph2d-panel-authored/src/generated/panel.rs",
        ),
        (
            "filho de um ficheiro carregado por `#[path]` — a regra que a 1.ª versão errou",
            "crates/ph2d-tool-painter/src/tool/paint/measure_preview_drain.rs",
        ),
    ];
    let mut falhas = Vec::new();
    for (porque, rel) in casos {
        match fs::canonicalize(root.join(rel)) {
            Ok(p) if c.alcancados.contains(&p) => {}
            Ok(_) => falhas.push(format!("NÃO ALCANÇADO ({porque}): {rel}")),
            Err(_) => falhas.push(format!(
                "O CONTROLO MUDOU DE SÍTIO ({porque}): {rel} — actualize o endereço, não apague o controlo"
            )),
        }
    }
    assert!(
        falhas.is_empty(),
        "o censo de órfãos falhou um caso difícil:\n  {}",
        falhas.join("\n  ")
    );
}

/// **Os parsers vêem todas as formas de declaração** — e recusam as que não o são.
#[test]
fn the_parsers_see_every_declaration_form() {
    for (linha, esperado) in [
        ("mod x;", Some(("x", ';'))),
        ("pub mod x;", Some(("x", ';'))),
        ("pub(crate) mod x;", Some(("x", ';'))),
        ("pub(in crate::a) mod x;", Some(("x", ';'))),
        ("    #[cfg(test)] mod tests;", Some(("tests", ';'))),
        ("mod r#type;", Some(("type", ';'))),
        ("mod tests {", Some(("tests", '{'))),
        ("module x;", None),
        ("pub fn mod_x() {}", None),
        ("public mod x;", None),
        ("mod 9x;", None),
    ] {
        let visto = mod_decl(linha);
        assert_eq!(
            visto.as_ref().map(|(n, c)| (n.as_str(), *c)),
            esperado,
            "`mod_decl` leu mal: {linha:?}"
        );
    }
    assert_eq!(
        path_attr(r#"#[path = "keys_scene.rs"]"#).as_deref(),
        Some("keys_scene.rs")
    );
    assert_eq!(path_attr(r#"#[doc = "x.rs"]"#), None);
    assert_eq!(
        includes(r#"include!("panel.rs");"#),
        vec!["panel.rs".to_string()]
    );
    assert!(includes(r#"include_str!("panel.rs")"#).is_empty());
    assert_eq!(brace_delta(r#"let s = "{"; if a {"#), 1);
    assert_eq!(brace_delta("match c { '{' => 1, _ => 0 }"), 0);
}

/// ⛔ **A outra metade: nenhum ficheiro tem DOIS pais na mesma crate.**
#[test]
fn no_source_file_is_declared_twice_in_the_same_crate() {
    let c = census();
    assert!(
        c.fontes >= 5_000,
        "controlo: só {} ficheiros `.rs` varridos — a varredura perdeu o sujeito",
        c.fontes
    );
    assert!(
        c.duplicados.is_empty(),
        "estes ficheiros são declarados por DOIS `mod` na MESMA crate — compilam como dois módulos, \
         os testes deles correm a dobrar e a contagem sobe sem ninguém ter escrito um teste:\n  {}\n\n\
         Apague a declaração a mais (no `nextest-list-diff` ele aparece ao mesmo tempo em `MOVED` e \
         em `ONLY-B`). ⚠️ Se as duas estão sob `cfg` mutuamente exclusivos não há duplicado ao \
         compilar — mas este gate lê declarações e não features: junte-as numa só.",
        c.duplicados.join("\n  ")
    );
}
