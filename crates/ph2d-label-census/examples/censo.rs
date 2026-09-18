//! **O censo lexical na linha de comando** — a MESMA régua que os gates correm.
//!
//! ```text
//! cargo run -q -p ph2d-label-census --example censo -- crates/ph2d-panel-painter-layers/src
//! cargo run -q -p ph2d-label-census --example censo -- --resumo
//! ```
//!
//! - com uma raiz `src/`: uma linha TSV por literal —
//!   `rel · linha · início · fim (índices de carácter do literal inteiro) · via · texto`, com o
//!   texto escapado (`\t`, `\n`, `\\`) para caber numa linha;
//! - `--resumo`: a contagem por crate de UI (os painéis, as `ph2d-app-*`, a `ph2d-editor-core`, a
//!   `ph2d-param-editors` e a shell), da maior para a menor.
//!
//! ⚠️ Corre da RAIZ do repo (os caminhos são relativos a ela).

use std::path::Path;

use ph2d_label_census::language_literals;

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--resumo") {
        let mut roots: Vec<(String, String)> = Vec::new();
        let mut names: Vec<String> = std::fs::read_dir("crates")
            .map(|rd| {
                rd.flatten()
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        for d in names {
            let ui = (d.starts_with("ph2d-panel-") && d != "ph2d-panel-registry-init")
                || d.starts_with("ph2d-app-")
                || d == "ph2d-editor-core"
                || d == "ph2d-param-editors";
            if ui && Path::new(&format!("crates/{d}/src")).is_dir() {
                roots.push((d.clone(), format!("crates/{d}/src")));
            }
        }
        roots.push(("shells/desktop".into(), "shells/desktop/src".into()));
        let mut rows: Vec<(usize, String)> = roots
            .iter()
            .map(|(n, r)| (language_literals(Path::new(r)).len(), n.clone()))
            .collect();
        rows.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        let total: usize = rows.iter().map(|r| r.0).sum();
        for (n, name) in rows.iter().filter(|r| r.0 > 0) {
            println!("{name:40} {n:5}");
        }
        println!("{:40} {total:5}", "TOTAL");
        return;
    }
    // ⭐⭐⭐ **`--cegos <raiz>`: o COMPLEMENTO da régua**, para a triagem que ela não pode fazer.
    //
    // ⚠️ **Não é uma lista de defeitos — é a população onde um rótulo PODE esconder-se.** Quase
    // tudo aqui é identificador legítimo, e é para não o acusar que as duas cercas existem. Mas
    // as duas já esconderam texto no ecrã com o censo VERDE (as letras do 9-slice em 18/09, o
    // `repeats`/`once` do Inspector em 13/09), e enquanto isso era prosa num doc-comment ninguém
    // conseguia LER a lista.
    if args.first().is_some_and(|a| a == "--cegos") {
        let Some(root) = args.get(1) else {
            eprintln!("uso: censo --cegos <raiz src/>");
            std::process::exit(2);
        };
        let mut files = Vec::new();
        colhe(Path::new(root), &mut files);
        files.sort();
        for f in files {
            let Ok(raw) = std::fs::read_to_string(&f) else {
                continue;
            };
            let rel = f
                .strip_prefix(Path::new(root))
                .unwrap_or(&f)
                .to_string_lossy()
                .replace('\\', "/");
            for (l, motivo) in ph2d_label_census::blind_literals_in(&rel, &raw) {
                println!(
                    "{}\t{}\t{motivo:?}\t{}\t{}",
                    l.rel,
                    l.line,
                    l.via,
                    escape(&l.text)
                );
            }
        }
        return;
    }
    let Some(root) = args.first() else {
        eprintln!("uso: censo <raiz src/> | --resumo | --cegos <raiz src/>");
        std::process::exit(2);
    };
    for l in language_literals(Path::new(root)) {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            l.rel,
            l.line,
            l.chars.start,
            l.chars.end,
            l.via,
            escape(&l.text)
        );
    }
}

/// Os `.rs` debaixo de uma raiz — o `--cegos` varre por ficheiro, e a porta memoizada da crate só
/// devolve os de LÍNGUA.
///
/// ⚠️ Ela **não** salta o que está sob `#[cfg(test)]` ao nível do FICHEIRO (a porta da crate faz
/// isso); o `blind_literals_in` continua a apagar os itens `#[cfg(test)]` de dentro de cada um,
/// como a régua de língua. *Uma lista de triagem que mostre um ficheiro de teste a mais custa uma
/// linha lida; uma que esconda um pintor custa um report do dono.*
fn colhe(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            colhe(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}
