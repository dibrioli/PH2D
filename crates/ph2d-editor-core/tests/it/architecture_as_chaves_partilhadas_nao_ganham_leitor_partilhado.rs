//! **As três chaves de param partilhadas entre tipos de nó (`clamp`, `step`, `value`) não ganham
//! leitor PARTILHADO** — a catraca do doc `docs/Motion Nodes/110_ciclo_6_valor_e_pulso.md` §14.1 (3).
//!
//! As chaves FICAM (são formato de ficheiro: o documento gravado lê o nome de volta). O que as
//! tornaria um defeito é o dia em que algum código as ler POR NOME sem estar preso a um tipo de nó —
//! o precedente é o `SUBSTEPS_PARAM`, que o pump lê em QUALQUER nó (`cook_substep.rs`), e uma corda
//! que chamava ao seu param `substeps` virou declarante sem querer.
//!
//! **População:** todo `.rs` sob `crates/` e `shells/`, MENOS:
//! - `crates/ph2d-node-*/` — as crates donas dos params (incluindo as seis donas destas chaves);
//! - `crates/ph2d-i18n/` — os rótulos são `node.<tipo>.param.<p>`, nunca o nome nu;
//! - código de teste e de cena: pastas `tests/`, ficheiros `*_tests.rs`, `*_probe.rs` ou cujo nome
//!   contenha `demo`, `smoke` ou `census`, ficheiros que o PAI declara sob `#[cfg(test)]`
//!   (`ph2d_label_census::cfg_test`) e, dentro do ficheiro, os itens `#[cfg(test)]` e os
//!   comentários (`ph2d_label_census::so_codigo_de_produto`).
//!
//! **Leitura por nome** (sobre o código de produto): `param("K")`, `param_default("K")`,
//! `text_param("K")`, `fan_param("K")`; `resolve_param(…"K"…)`; `.get("K")` num ficheiro que abre
//! um mapa de overrides (`node_param_overrides(`, `node_text_param_overrides(`, `node_params(`,
//! `node_text_params(`); e `const`/`static` `<NOME>: &str = "K"`. Importar a constante de uma crate
//! DONA (`ph2d_node_value_table::VALUE_KEY`) não conta: é leitura presa ao tipo por construção.

use std::fs;
use std::path::{Path, PathBuf};

use ph2d_label_census::cfg_test::is_declared_under_cfg_test;
use ph2d_label_census::so_codigo_de_produto;

const CHAVES: [&str; 3] = ["clamp", "step", "value"];
const LEITORES: [&str; 4] = ["param", "param_default", "text_param", "fan_param"];
const MAPAS: [&str; 4] = [
    "node_param_overrides(",
    "node_text_param_overrides(",
    "node_params(",
    "node_text_params(",
];
/// Medido em 2026-10-07: a população tem `3 840` ficheiros. Piso ≈ 80 %: um filtro que passe a
/// excluir quase tudo reprova aqui, e não aprova por varrer nada.
const PISO_DE_FICHEIROS: usize = 3_070;

fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rel(raiz: &Path, p: &Path) -> String {
    p.strip_prefix(raiz)
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

fn rs_sob(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let nome = e.file_name();
        let nome = nome.to_string_lossy();
        if p.is_dir() {
            if nome != "target" && !nome.starts_with('.') {
                rs_sob(&p, out);
            }
        } else if nome.ends_with(".rs") {
            out.push(p);
        }
    }
}

fn fora_da_populacao(rel: &str, p: &Path) -> bool {
    let nome = rel.rsplit('/').next().unwrap_or(rel);
    rel.starts_with("crates/ph2d-node-")
        || rel.starts_with("crates/ph2d-i18n/")
        || rel.split('/').any(|c| c == "tests")
        || nome.ends_with("_tests.rs")
        || nome.ends_with("_probe.rs")
        || ["demo", "smoke", "census"].iter().any(|m| nome.contains(m))
        || is_declared_under_cfg_test(p)
}

fn populacao(raiz: &Path) -> Vec<PathBuf> {
    let mut todos = Vec::new();
    for topo in ["crates", "shells"] {
        rs_sob(&raiz.join(topo), &mut todos);
    }
    todos.retain(|p| !fora_da_populacao(&rel(raiz, p), p));
    todos.sort();
    todos
}

fn ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn abre_token(code: &str, i: usize) -> bool {
    !code[..i].chars().next_back().is_some_and(ident)
}

fn literal(chave: &str) -> String {
    format!("\"{chave}\"")
}

/// As leituras por nome de `chaves` em `code` (já sem comentários nem testes): `(linha, chave,
/// forma)`, a linha contada a partir de 1.
fn leituras(code: &str, chaves: &[&str]) -> Vec<(usize, String, &'static str)> {
    let linha = |i: usize| code[..i].matches('\n').count() + 1;
    let mut out = Vec::new();
    for m in LEITORES {
        let agulha = format!("{m}(");
        for (i, _) in code.match_indices(&agulha) {
            let resto = code[i + agulha.len()..].trim_start();
            for k in chaves {
                if abre_token(code, i) && resto.starts_with(&literal(k)) {
                    out.push((linha(i), (*k).to_owned(), "param(\"K\")"));
                }
            }
        }
    }
    for (i, _) in code.match_indices("resolve_param(") {
        if !abre_token(code, i) {
            continue;
        }
        let corpo = &code[i + "resolve_param(".len()..];
        let mut fundo = 1usize;
        let fim = corpo
            .char_indices()
            .find(|&(_, c)| {
                match c {
                    '(' => fundo += 1,
                    ')' => fundo -= 1,
                    _ => {}
                }
                fundo == 0
            })
            .map_or(corpo.len(), |(j, _)| j);
        for k in chaves {
            if corpo[..fim].contains(&literal(k)) {
                out.push((linha(i), (*k).to_owned(), "resolve_param(…\"K\"…)"));
            }
        }
    }
    if MAPAS.iter().any(|m| code.contains(m)) {
        for (i, _) in code.match_indices(".get(") {
            let resto = code[i + ".get(".len()..].trim_start();
            for k in chaves {
                if resto.starts_with(&literal(k)) {
                    out.push((linha(i), (*k).to_owned(), "overrides.get(\"K\")"));
                }
            }
        }
    }
    let mut inicio = 0usize;
    for l in code.split_inclusive('\n') {
        for decl in ["const ", "static "] {
            for (c, _) in l.match_indices(decl) {
                if !abre_token(l, c) {
                    continue;
                }
                let Some((tipo, valor)) = l[c + decl.len()..].split_once('=') else {
                    continue;
                };
                for k in chaves {
                    // `tipo` com `:` — o `'static ` de `&'static str` não abre declaração.
                    if tipo.contains(':')
                        && tipo.contains("str")
                        && valor.trim_start().starts_with(&literal(k))
                    {
                        out.push((linha(inicio + c), (*k).to_owned(), "const = \"K\""));
                    }
                }
            }
        }
        inicio += l.len();
    }
    out
}

/// `(ficheiro:linha, chave, forma)` de toda leitura por nome de `chaves` na população.
fn varre(raiz: &Path, fontes: &[PathBuf], chaves: &[&str]) -> Vec<(String, String, &'static str)> {
    let mut out = Vec::new();
    for p in fontes {
        let code = so_codigo_de_produto(&fs::read_to_string(p).unwrap_or_default());
        for (l, k, forma) in leituras(&code, chaves) {
            out.push((format!("{}:{l}", rel(raiz, p)), k, forma));
        }
    }
    out
}

#[test]
fn nenhuma_das_tres_chaves_e_lida_por_nome_fora_do_dono() {
    let raiz = raiz();
    let fontes = populacao(&raiz);
    let achados = varre(&raiz, &fontes, &CHAVES);
    println!(
        "população: {} ficheiros · leituras por nome de {CHAVES:?}: {}",
        fontes.len(),
        achados.len()
    );
    for (onde, k, forma) in &achados {
        println!("  {onde}  `{k}`  ({forma})");
    }
    assert!(
        fontes.len() >= PISO_DE_FICHEIROS,
        "o controlo: a catraca tem de VER a árvore — a população tem {} ficheiros, o piso é \
         {PISO_DE_FICHEIROS}",
        fontes.len()
    );
    // Os vizinhos medidos que NÃO são leituras de param vivem na população: a recusa deles é
    // feita sobre a árvore, não sobre uma cópia. (O `("step", "sim.step")` do
    // `motion_bridge_sim_figures.rs` não está aqui: o pai declara-o `#[cfg(test)]`.)
    for vizinho in [
        "crates/ph2d-app-motion/src/motion_bridge_edit.rs",
        "crates/ph2d-timeline/src/frame_solve.rs",
        "crates/ph2d-timeline/src/expr_pass.rs",
        "crates/ph2d-nodegraph/src/cook_substep.rs",
    ] {
        assert!(
            fontes.iter().any(|p| rel(&raiz, p) == vizinho),
            "o controlo: `{vizinho}` saiu da população — a recusa dos vizinhos deixou de ser \
             medida sobre a árvore"
        );
    }
    let falhas: Vec<String> = achados
        .iter()
        .map(|(onde, k, forma)| {
            format!(
                "a chave `{k}` ganhou um consumidor PARTILHADO em {onde} ({forma}) — ela virou um \
                 `substeps`: a cura é renomear COM migração (doc 110 §14.1 (3))"
            )
        })
        .collect();
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

#[test]
fn a_mesma_regua_ve_o_precedente_substeps_na_arvore() {
    // O controlo positivo sobre a árvore: a régua, apontada à chave que JÁ é partilhada, acha a
    // porta que a declara — senão um zero acima não quer dizer nada.
    let raiz = raiz();
    let fontes = populacao(&raiz);
    let achados = varre(&raiz, &fontes, &["substeps"]);
    assert!(
        achados
            .iter()
            .any(|(onde, _, _)| onde.starts_with("crates/ph2d-nodegraph/src/cook_substep.rs:")),
        "a régua deixou de ver o `SUBSTEPS_PARAM` em `cook_substep.rs` — achou só {achados:?}"
    );
}

#[test]
fn o_detector_ve_as_formas_e_recusa_os_vizinhos() {
    let lidas = |src: &str| -> Vec<String> {
        leituras(&so_codigo_de_produto(src), &CHAVES)
            .into_iter()
            .map(|(_, k, _)| k)
            .collect()
    };
    assert_eq!(lidas("let x = ctx.param(\"clamp\");"), ["clamp"]);
    assert_eq!(lidas("m.param_default(\"step\")"), ["step"]);
    assert_eq!(lidas("ctx.text_param(\"value\")"), ["value"]);
    assert_eq!(lidas("ctx.fan_param(\"step\", k)"), ["step"]);
    assert_eq!(
        lidas("let v = resolve_param(\n    graph,\n    node,\n    m,\n    \"clamp\",\n    d,\n);"),
        ["clamp"]
    );
    assert_eq!(
        lidas("let o = graph.node_param_overrides(n);\nlet s = o.and_then(|m| m.get(\"step\"));"),
        ["step"]
    );
    assert_eq!(lidas("pub const RESERVADA: &str = \"value\";"), ["value"]);
    assert_eq!(lidas("static R: &'static str = \"clamp\";"), ["clamp"]);
    // Os vizinhos medidos em 2026-10-07 — nomes de figura, rótulo de leitura, variável de expressão,
    // JSON — e a escrita, o comentário e o teste.
    for vizinho in [
        "const FIGS: &[(&str, &str)] = &[\n    (\"step\", \"sim.step\"),\n];",
        "Reading::Value(v) => (\"value\".to_string(), v),",
        "match name {\n    \"value\" => self.value,\n    _ => 0.0,\n}",
        "let x = o.get(\"value\")?.as_f64()?;",
        "graph.set_param(n, \"clamp\", 1.0);",
        "g.set_text_param(t, ph2d_node_value_table::VALUE_KEY, col);",
        "// ctx.param(\"clamp\") num comentário não lê nada",
        "#[cfg(test)]\nmod t {\n    fn f(c: &C) {\n        c.param(\"step\");\n    }\n}",
        "let y = ctx.my_param(\"clamp\");",
    ] {
        assert!(lidas(vizinho).is_empty(), "falso positivo em: {vizinho}");
    }
}
