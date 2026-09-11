//! **Uma variante que ninguém NOMEIA em lado nenhum não pode ser produzida nem lida.**
//!
//! ## A espécie que os dois instrumentos novos não vêem
//!
//! A caça de 2026-08-30 construiu duas sondas — «o controlo pintado tem leitor» e «o dreno cobre
//! toda a variante». As duas trabalham sobre **ids de widget** (`ids::*`) e sobre **filas de
//! `drain_*`**. O `WidgetEvent` não é nenhuma das duas coisas: é um enum que o `editor-core`
//! publica e que **cada painel** consome no seu próprio `match`.
//!
//! ⇒ Um `variant` acrescentado ao enum «para quando alguém precisar» fica lá para sempre, e a
//! forma da sua morte é a mesma dos knobs: *o painel oferece, e do outro lado não há ninguém.*
//! Foi assim que a `SelectionChanged(NodeId)` viveu — declarada *«Tabs / Dropdown / TreeView —
//! selected index changed»*, com **uma** ocorrência em todo o repositório: a própria declaração.
//!
//! ## A régua, e por que ela não pode dar falso positivo
//!
//! Para **construir** uma variante é preciso escrever o nome dela. Para a **casar** num `match`,
//! idem. ⇒ *zero ocorrências fora do sítio onde ela é declarada* prova, por construção, que ela
//! não tem produtor **nem** leitor. Não é heurística: é a única forma de a mencionar.
//!
//! ⚠️ **O nome tem de ser procurado nas DUAS formas** — qualificado (`WidgetEvent::Foo`) e nu
//! (`Foo(..)`, depois de um `use …WidgetEvent::*`). Procurar só a qualificada acusaria de morta
//! uma variante viva num ficheiro que importa o enum inteiro; e procurar o nome nu em toda a parte
//! daria vivas as que se chamam como uma palavra comum (`Focus`, `Blur`, `Cancel`, `Submit`).
//! Por isso o nome nu **só conta** em ficheiros que de facto fazem esse `use`.
//!
//! ## As duas metades
//!
//! A primeira acusa a variante sem uma única menção. A segunda é o **controlo**: se o parse
//! deixar de achar as variantes, ou de ler a árvore, ele devolve zero acusações — e *um balde que
//! ninguém enche lê-se como perfeito*. Ela exige que a esmagadora maioria das variantes seja
//! encontrada VIVA, que é o estado normal deste enum.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// Onde o enum vive — o único sítio em que uma variante pode aparecer sem estar a ser usada.
const DECL: &str = "crates/ph2d-editor-core/src/interaction/event.rs";

/// As raízes varridas. `crates/`, `tools/` e `shells/` são todo o código que compilamos.
const ROOTS: [&str; 3] = ["crates", "tools", "shells"];

/// As variantes de `WidgetEvent`, lidas do FONTE — ⛔ nunca uma lista escrita à mão, que estaria
/// errada no dia em que alguém acrescentasse a próxima (e é precisamente a próxima que interessa).
fn variants(decl_src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in decl_src.lines() {
        if line.contains("pub enum WidgetEvent") {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if line.starts_with('}') {
            break;
        }
        let t = line.trim();
        // Uma variante é um identificador em maiúscula no início da linha, com a indentação do
        // corpo do enum. Doc-comments, atributos e campos de variante de struct não passam.
        if !line.starts_with("    ") || line.starts_with("     ") {
            continue;
        }
        let name: String = t.chars().take_while(char::is_ascii_alphanumeric).collect();
        if name.is_empty() || !name.starts_with(|c: char| c.is_ascii_uppercase()) {
            continue;
        }
        // O que vem depois do nome só pode ser `(`, `{` ou `,` — senão é outra coisa.
        let rest = t[name.len()..].trim_start();
        if rest.starts_with('(') || rest.starts_with('{') || rest.starts_with(',') {
            out.push(name);
        }
    }
    out
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            // `target/` e `vendor/` não são código nosso.
            let skip = p
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s == "target" || s == "vendor" || s.starts_with('.'));
            if !skip {
                collect_rs(&p, out);
            }
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Quantas vezes `name` é mencionado como variante de `WidgetEvent`, fora da declaração.
fn mentions(files: &[(PathBuf, String)], decl: &Path, name: &str) -> usize {
    let qualified = format!("WidgetEvent::{name}");
    let mut n = 0;
    for (path, src) in files {
        if path == decl {
            continue;
        }
        n += src.matches(qualified.as_str()).count();
        // Nome NU: só vale em ficheiros que importam o enum inteiro.
        if src.contains("WidgetEvent::*") {
            for line in src.lines() {
                let t = line.trim_start();
                if t.starts_with("//") || t.contains("WidgetEvent::*") {
                    continue;
                }
                // Uma menção nua é `Nome(` ou `Nome {` ou `Nome =>`.
                for pat in [
                    format!("{name}("),
                    format!("{name} {{"),
                    format!("{name} =>"),
                ] {
                    if t.contains(pat.as_str()) {
                        n += 1;
                    }
                }
            }
        }
    }
    n
}

fn scan() -> (Vec<String>, Vec<String>, Vec<String>) {
    let r = root();
    let decl = r.join(DECL);
    let decl_src = std::fs::read_to_string(&decl).unwrap_or_else(|e| panic!("{DECL}: {e}"));
    let vars = variants(&decl_src);

    let mut paths = Vec::new();
    for d in ROOTS {
        collect_rs(&r.join(d), &mut paths);
    }
    let files: Vec<(PathBuf, String)> = paths
        .into_iter()
        .filter_map(|p| std::fs::read_to_string(&p).ok().map(|s| (p, s)))
        .collect();

    let (mut dead, mut alive) = (Vec::new(), Vec::new());
    for v in &vars {
        if mentions(&files, &decl, v) == 0 {
            dead.push(v.clone());
        } else {
            alive.push(v.clone());
        }
    }
    (vars, alive, dead)
}

#[test]
fn no_widget_event_variant_is_unmentioned_outside_its_declaration() {
    let (_, _, dead) = scan();
    assert!(
        dead.is_empty(),
        "variante(s) de `WidgetEvent` que NINGUÉM nomeia em todo o repositório: {}\n\
         \n\
         Para construir uma variante é preciso escrever o nome dela; para a casar num `match`,\n\
         idem. Zero menções prova que ela não tem produtor NEM leitor — um painel que a emitisse\n\
         não existiria, e um que a esperasse também não.\n\
         \n\
         A cura é apagá-la. Uma variante guardada «para quando alguém precisar» faz toda sonda\n\
         futura mentir sobre a superfície deste enum, e o custo de a reintroduzir no dia em que\n\
         houver um consumidor é uma linha.\n\
         \n\
         Se ela existe para um consumidor que está a ser construído AGORA, escreva o consumidor\n\
         na mesma wave — é a lei da DIRETIVA §1 (o consumidor faz parte deste work item).",
        dead.join(", ")
    );
}

/// **Controle: o censo encontrou o enum, as variantes e a árvore.**
///
/// Sem ele, um `pub enum WidgetEvent` renomeado, ou uma indentação diferente, faria `variants()`
/// devolver a lista vazia — e uma lista vazia não tem mortos, o que se lê como aprovado.
#[test]
fn the_census_actually_read_the_enum_and_the_tree() {
    let (vars, alive, _) = scan();
    assert!(
        vars.len() >= 10,
        "o censo achou {} variante(s) de `WidgetEvent` — a declaração mudou de forma e este gate \
         deixou de a ler",
        vars.len()
    );
    assert!(
        alive.len() >= vars.len() - 2,
        "só {} de {} variantes foram achadas VIVAS. Ou a árvore não foi lida, ou a régua de \
         menção partiu-se: o estado normal deste enum é quase tudo vivo, e um censo que acusa \
         metade está a medir-se a si próprio.\nvivas: {:?}",
        alive.len(),
        vars.len(),
        alive
    );
}
