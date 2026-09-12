//! ⭐⭐ **NENHUMA dependência sobe uma camada** — auditoria de arquitectura 2026-09-12, A1.
//!
//! # O defeito, medido
//!
//! O [`architecture_cycle_prevention`](super::architecture_cycle_prevention) nasceu na Wave 8 para
//! duas perguntas — a `editor-core` não depende de painel nem de ferramenta concreta; painel não
//! depende de painel. A W2 inventou DUAS espécies depois dele, as famílias `ph2d-app-*` e as folhas
//! partilhadas, e **nenhum gate dizia em que direcção elas podiam depender**. A auditoria achou,
//! sobre o grafo de `cargo metadata`, **três folhas a subir** (`ph2d-pan-diag → ph2d-app-flip`,
//! `ph2d-vec-art-live → ph2d-panel-vector`, `ph2d-audio-desktop → ph2d-panel-audio-editor`) e **três
//! famílias a chamar famílias** (`app-motion → app-vec`, `app-motion → app-flip`,
//! `app-components → app-physics`) — onde o ADR-0075 diz que os sistemas *«nunca chamam uns aos
//! outros directamente»*. Todas foram curadas por ASSUNTO (tipos para as folhas do domínio, a câmera
//! para o renderizador, as sementes injectadas pela composição, a família do áudio nomeada), e a
//! catraca abaixo nasce **VAZIA**.
//!
//! # As espécies — pelo nome do pacote e pelo directório (a convenção da casa É a classificação)
//!
//! - **composição**: os membros de `shells/`;
//! - **registo**: `*-registry-init` (compõem por codegen);
//! - **família**: `ph2d-app-*`, menos o substrato `ph2d-app-host`;
//! - **painel**: `ph2d-panel-*`;
//! - **ferramenta concreta**: `ph2d-tool-*`, menos os contratos (`ph2d-tool-registry`,
//!   `ph2d-tool-runtime`);
//! - tudo o resto — motores, contratos, substrato, e os membros de `tools/` e `tests/` — é **folha**.
//!
//! # As leis
//!
//! Sobre `[dependencies]` e `[build-dependencies]`, onde o produto é montado:
//! 1. de uma **família** só dependem a composição e os registos;
//! 2. de um **painel** só dependem famílias, a composição e os registos;
//! 3. de uma **ferramenta concreta** só dependem painéis, famílias, a composição e os registos.
//!
//! E sobre `[dev-dependencies]`: 4. de uma **família** continuam a depender só a composição e os
//! registos — um atalho de teste é a mesma aresta com outro nome.
//!
//! ⚠️ **O que as leis PERMITEM, de propósito:** folha → contrato (`ph2d-tool-registry`) e folha →
//! substrato (`ph2d-app-host`) apontam para baixo; painel → ferramenta é o ADR-0040; família → painel
//! e família → ferramenta é a família a compor a UI do seu módulo.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// ⛔ **A catraca — nasce VAZIA e só encolhe.** `(de, para, porquê)`. Uma entrada nova precisa de
/// nomear o bloqueador e de uma wave com dono; o gate irmão reprova a entrada que deixou de ser aresta
/// ou deixou de subir.
const ARESTAS_TOLERADAS: &[(&str, &str, &str)] = &[];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Especie {
    Composicao,
    Registo,
    Familia,
    Painel,
    Ferramenta,
    Folha,
}

fn especie(nome: &str, dir_rel: &str) -> Especie {
    if dir_rel.starts_with("shells/") {
        return Especie::Composicao;
    }
    if dir_rel.starts_with("tools/") || dir_rel.starts_with("tests/") {
        return Especie::Folha;
    }
    if nome.ends_with("-registry-init") {
        return Especie::Registo;
    }
    if let Some(r) = nome.strip_prefix("ph2d-app-") {
        return if r == "host" {
            Especie::Folha
        } else {
            Especie::Familia
        };
    }
    if nome.starts_with("ph2d-panel-") {
        return Especie::Painel;
    }
    if let Some(r) = nome.strip_prefix("ph2d-tool-") {
        return if matches!(r, "registry" | "runtime") {
            Especie::Folha
        } else {
            Especie::Ferramenta
        };
    }
    Especie::Folha
}

fn permitido(de: Especie, para: Especie, dev: bool) -> bool {
    use Especie::{Composicao, Familia, Ferramenta, Painel, Registo};
    match para {
        Familia => matches!(de, Composicao | Registo),
        Painel if !dev => matches!(de, Familia | Composicao | Registo),
        Ferramenta if !dev => matches!(de, Painel | Familia | Composicao | Registo),
        _ => true,
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("a raiz da workspace")
        .to_path_buf()
}

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
    corpo
        .split(',')
        .filter_map(|s| {
            let s = s.trim().trim_matches(|c| c == '"' || c == '\'');
            (!s.is_empty()).then(|| s.to_owned())
        })
        .collect()
}

fn package_name(toml: &str) -> Option<String> {
    table(toml, "[package]").lines().find_map(|l| {
        let (k, v) = l.split('#').next()?.split_once('=')?;
        (k.trim() == "name").then(|| v.trim().trim_matches('"').to_owned())
    })
}

/// `(nome, directório relativo, manifesto)` de cada membro.
fn members(root: &Path) -> Vec<(String, String, String)> {
    let toml = fs::read_to_string(root.join("Cargo.toml")).expect("o Cargo.toml da raiz");
    let ws = table(&toml, "[workspace]");
    let padroes = toml_string_array(&ws, "members");
    assert!(
        padroes.iter().any(|p| p == "crates/*"),
        "controlo do parser: `[workspace] members` não mostrou `crates/*` — leu {padroes:?}"
    );
    let mut dirs = BTreeSet::new();
    for p in &padroes {
        if let Some(dir) = p.strip_suffix("/*") {
            let rd = fs::read_dir(root.join(dir)).unwrap_or_else(|e| panic!("`{dir}`: {e}"));
            for e in rd.flatten() {
                if e.path().join("Cargo.toml").is_file() {
                    dirs.insert(format!("{dir}/{}", e.file_name().to_string_lossy()));
                }
            }
        } else if root.join(p).join("Cargo.toml").is_file() {
            dirs.insert(p.clone());
        }
    }
    dirs.into_iter()
        .map(|d| {
            let m = fs::read_to_string(root.join(&d).join("Cargo.toml")).expect("manifesto");
            let n = package_name(&m).unwrap_or_else(|| panic!("{d}: sem `[package] name`"));
            (n, d, m)
        })
        .collect()
}

/// As chaves de dependência de um manifesto: `(nome, é dev?)`.
fn deps(manifesto: &str) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    let mut tabela: Option<bool> = None;
    for l in manifesto.lines() {
        let t = l.trim();
        if t.starts_with('[') {
            tabela = t
                .ends_with("dependencies]")
                .then(|| t.contains("dev-dependencies"));
            continue;
        }
        let Some(dev) = tabela else {
            continue;
        };
        if t.starts_with('#') {
            continue;
        }
        let Some((k, _)) = t.split_once('=') else {
            continue;
        };
        let k = k.trim();
        let k = k.split('.').next().unwrap_or(k).trim();
        if !k.is_empty() && !k.contains(' ') && !k.contains('"') {
            out.push((k.to_owned(), dev));
        }
    }
    out
}

struct Grafo {
    especies: BTreeMap<String, Especie>,
    arestas: BTreeSet<(String, String, bool)>,
}

fn grafo() -> Grafo {
    let root = root();
    let ms = members(&root);
    assert!(
        ms.len() >= 300,
        "li só {} membros — o leitor partiu-se e o gate ficaria verde por vácuo",
        ms.len()
    );
    let especies: BTreeMap<String, Especie> = ms
        .iter()
        .map(|(n, d, _)| (n.clone(), especie(n, d)))
        .collect();
    let mut arestas = BTreeSet::new();
    for (n, _, m) in &ms {
        for (d, dev) in deps(m) {
            if especies.contains_key(&d) && &d != n {
                arestas.insert((n.clone(), d, dev));
            }
        }
    }
    Grafo { especies, arestas }
}

fn sobe(g: &Grafo, de: &str, para: &str, dev: bool) -> bool {
    !permitido(g.especies[de], g.especies[para], dev)
}

#[test]
fn no_dependency_climbs_a_layer() {
    let g = grafo();
    let conta = |e: Especie| g.especies.values().filter(|x| **x == e).count();
    assert!(
        conta(Especie::Familia) >= 9,
        "li {} famílias",
        conta(Especie::Familia)
    );
    assert!(
        conta(Especie::Painel) >= 20,
        "li {} painéis",
        conta(Especie::Painel)
    );
    assert!(
        conta(Especie::Ferramenta) >= 10,
        "li {} ferramentas",
        conta(Especie::Ferramenta)
    );
    let toleradas: BTreeSet<(&str, &str)> =
        ARESTAS_TOLERADAS.iter().map(|(a, b, _)| (*a, *b)).collect();
    let sobem: Vec<String> = g
        .arestas
        .iter()
        .filter(|(de, para, dev)| {
            sobe(&g, de, para, *dev) && !toleradas.contains(&(de.as_str(), para.as_str()))
        })
        .map(|(de, para, dev)| {
            format!(
                "{de} ({:?}) → {para} ({:?}){}",
                g.especies[de.as_str()],
                g.especies[para.as_str()],
                if *dev { "  [dev]" } else { "" }
            )
        })
        .collect();
    assert!(
        sobem.is_empty(),
        "estas dependências SOBEM uma camada:\n  {}\n\n\
         cura (a da auditoria A1): leve o que se usa do outro lado para a folha do ASSUNTO — um tipo \
         para a crate do domínio, uma lei para a crate do motor, uma tabela para ser injectada pela \
         composição. ⛔ Nunca um re-export: uma fachada é a mesma aresta com outro nome.",
        sobem.join("\n  ")
    );
}

#[test]
fn the_tolerated_edges_are_still_edges_and_still_climb() {
    let g = grafo();
    for (de, para, porque) in ARESTAS_TOLERADAS {
        assert!(
            !porque.trim().is_empty(),
            "`{de} → {para}` tolerada sem motivo"
        );
        let existe = |dev| {
            g.arestas
                .contains(&((*de).to_owned(), (*para).to_owned(), dev))
        };
        assert!(
            (existe(false) && sobe(&g, de, para, false))
                || (existe(true) && sobe(&g, de, para, true)),
            "`{de} → {para}` está na catraca e já não é uma aresta que sobe — apague a linha"
        );
    }
    // ⚠️ Sentinela: com a catraca vazia, a metade acima é verde por construção — então prove que o
    // leitor VÊ as três espécies de aresta que as leis PERMITEM.
    for (de, para) in [
        ("ph2d-host-desktop", "ph2d-app-motion"),
        ("ph2d-panel-vector", "ph2d-tool-vector"),
        ("ph2d-app-registry-init", "ph2d-app-vec"),
    ] {
        assert!(
            g.arestas.contains(&(de.to_owned(), para.to_owned(), false)),
            "sentinela: o leitor não viu `{de} → {para}` — ele partiu-se e o gate mede nada"
        );
        assert!(
            !sobe(&g, de, para, false),
            "sentinela: `{de} → {para}` devia ser permitida"
        );
    }
}

/// ⚠️ **E as leis sabem dizer «não».**
#[test]
fn the_rules_can_say_no() {
    use Especie::{Composicao, Familia, Ferramenta, Folha, Painel, Registo};
    assert!(!permitido(Folha, Familia, false), "folha → família");
    assert!(!permitido(Familia, Familia, false), "família → família");
    assert!(
        !permitido(Familia, Familia, true),
        "família → família, em dev"
    );
    assert!(!permitido(Folha, Painel, false), "folha → painel");
    assert!(!permitido(Painel, Painel, false), "painel → painel");
    assert!(!permitido(Folha, Ferramenta, false), "folha → ferramenta");
    assert!(
        permitido(Painel, Ferramenta, false),
        "painel → ferramenta é o ADR-0040"
    );
    assert!(
        permitido(Composicao, Familia, true),
        "a composição liga famílias"
    );
    assert!(
        permitido(Registo, Familia, false),
        "o registo liga famílias"
    );
    assert_eq!(especie("ph2d-app-host", "crates/ph2d-app-host"), Folha);
    assert_eq!(
        especie("ph2d-app-registry-init", "crates/ph2d-app-registry-init"),
        Registo
    );
    assert_eq!(especie("ph2d-panel-sync", "tools/ph2d-panel-sync"), Folha);
    assert_eq!(
        especie("ph2d-tool-registry", "crates/ph2d-tool-registry"),
        Folha
    );
    assert_eq!(
        especie("ph2d-tool-vector", "crates/ph2d-tool-vector"),
        Ferramenta
    );
    assert_eq!(especie("ph2d-host-desktop", "shells/desktop"), Composicao);
    assert_eq!(
        deps(
            "[dependencies]\nph2d-a = { path = \"../a\" }\nph2d-b.workspace = true\n# ph2d-c = 1\n\n[dev-dependencies]\nph2d-d = \"1\"\n[features]\nx = []\n"
        ),
        vec![
            ("ph2d-a".to_owned(), false),
            ("ph2d-b".to_owned(), false),
            ("ph2d-d".to_owned(), true),
        ]
    );
}
