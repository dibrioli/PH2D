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
    let Some(root) = args.first() else {
        eprintln!("uso: censo <raiz src/> | --resumo");
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
