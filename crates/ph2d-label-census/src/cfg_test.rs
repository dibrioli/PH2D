//! **É este ficheiro um módulo de TESTE inteiro?** — a pergunta feita ao PAI, que é quem o compila.
//!
//! ⚠️ **Mudou-se para aqui de `ph2d-editor-core/tests/common/cfg_test_modules.rs` (2026-09-13), e o
//! motivo é o que este próprio ficheiro já escrevia:** *«ela vive AQUI porque tem dois donos, e a
//! segunda cópia é a que diverge»*. A régua lexical desta crate faz a mesma pergunta, e os gates por
//! crate dos painéis também — um terceiro dono a copiar isto seria a divergência anunciada. O
//! ficheiro antigo ficou a re-exportar esta função, e o código abaixo é o mesmo, byte a byte.
//!
//! ⚠️ **E a pergunta é feita ao PAI, nunca ao NOME.** Uma lista de nomes (`tests.rs`,
//! `*_tests.rs`, …) é a enumeração que apodrece no dia em que alguém chamar o irmão de outra
//! coisa — e, pior, isentaria um ficheiro de PRODUÇÃO com nome parecido.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

/// Quantos ficheiros esta régua leu do disco desde que o processo começou.
static LEITURAS: AtomicUsize = AtomicUsize::new(0);

/// ⭐⭐⭐ **O instrumento da memória, e ele NÃO é um relógio.**
///
/// ⛔ Um gate de tempo sobre isto seria mais um membro da família de flakes de fan-out do
/// `CLAUDE.md` §5.0 — *«todo gate que compara duas medianas de um RECURSO é candidato»*. Uma
/// CONTAGEM de leituras é determinística: com a memória ela é `O(ficheiros)` e sem ela
/// `O(ficheiros²)`, e a diferença entre as duas não depende de carga nenhuma.
#[must_use]
pub fn leituras_do_disco() -> usize {
    LEITURAS.load(Ordering::Relaxed)
}

/// **Este ficheiro é um módulo de TESTE inteiro?** — perguntado ao PAI, que é quem o gateia.
///
/// ⚠️ **É a mesma lei que o `strip_test_modules` da `hr15` já aplica, uma grafia depois.** Um
/// `#[cfg(test)] mod tests { … }` inline é removido; um `#[cfg(test)] mod tests;` que resolve para
/// `<slug>/tests.rs` era lido como PRODUÇÃO — e o ficheiro é literalmente o mesmo código, movido
/// para o irmão pelo tecto de LOC. Medido em 2026-08-15: a `ph2d-editor-core` tem **onze** desses
/// ficheiros, e **dez passavam por acidente** (nenhum deles usava `.label("`/`.placeholder("`); o
/// décimo-primeiro — o `text_input/tests.rs` — nasceu de um split e trouxe um `.placeholder`
/// consigo, virando o gate vermelho sobre código que nunca correu em produção.
///
/// ⚠️ **E a pergunta é feita ao PAI de propósito, nunca ao NOME do ficheiro.** Uma lista de nomes
/// (`tests.rs`, `*_tests.rs`, …) é a enumeração que apodrece no dia em que alguém chamar o irmão
/// de outra coisa — e, pior, isentaria um ficheiro de produção com nome parecido. Quem sabe se
/// isto é teste é a declaração que o compila.
pub fn is_declared_under_cfg_test(path: &Path) -> bool {
    declared_under_cfg_test(path, 8)
}

/// **A NETA também é teste** — e é ela que obrigava a allowlist a crescer.
///
/// ⚠️ Um `#[cfg(test)] mod tests;` em `skin.rs` gateia `skin/tests.rs`; os módulos que
/// **`skin/tests.rs`** declara (`mod param;`, `mod axis;`, …) **não levam `#[cfg(test)]`**, porque
/// já estão dentro de um. A pergunta *«o meu pai gateia-me?»* respondia `false` para todos eles, e
/// a consequência era uma linha de allowlist escrita à mão **por ficheiro** — cinco só em `skin/`,
/// e a sexta seria escrita hoje. *A enumeração apodrece; a recursão não.*
///
/// ⚠️ **O `depth` não é cautela decorativa:** dois ficheiros podem declarar-se mutuamente (um
/// `#[path]` cruzado é legal em Rust), e sem tecto a varredura de um gate entraria em ciclo. Oito
/// é folgado — a árvore mais funda deste repo tem três degraus.
fn declared_under_cfg_test(path: &Path, depth: u8) -> bool {
    if depth == 0 {
        return false;
    }
    let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) else {
        return false;
    };
    let Some(file_dir) = path.parent() else {
        return false;
    };
    // ⛔⛔ **`x/mod.rs` É o módulo `x`** — o nome dele é o do DIRECTÓRIO, e quem o declara vive um
    // nível ACIMA. Até 2026-09-13 esta função procurava `mod mod;`, que não existe, e respondia
    // «produção» para a árvore inteira: os 12 ficheiros de `interaction/dispatch/tests/` da
    // `ph2d-editor-core` (declarados por `#[cfg(test)] mod tests;` em `dispatch/mod.rs`) e os de
    // `ph2d-tool-painter/src/tool/paint/tests/` eram lidos como código de PRODUTO. Quem o apanhou foi
    // a régua lexical desta crate, a contar «hello world» e «<missing>» como rótulos da interface.
    let (stem, dir) = if file_stem == "mod" {
        let (Some(name), Some(up)) = (
            file_dir.file_name().and_then(|s| s.to_str()),
            file_dir.parent(),
        ) else {
            return false;
        };
        (name, up)
    } else {
        (file_stem, file_dir)
    };
    // ⚠️ **O pai pode estar em TRÊS sítios, e o terceiro foi o que a `hr12` não via.** O
    // `<dir>/mod.rs` e o `<dir>.rs` um nível acima são os dois óbvios; o terceiro é um **IRMÃO
    // PLANO** — `src/paint.rs` a declarar `#[path = "paint_tests.rs"] mod paint_tests;`, com os
    // dois no mesmo directório. Era exactamente esse o caso que obrigava uma linha de allowlist
    // escrita à mão por ficheiro (`paint_wire_tests.rs`), e ele é o padrão mais comum nos painéis:
    // um `_tests.rs` cortado do pai pelo tecto de LOC.
    //
    // A varredura do directório é `O(irmãos)` por ficheiro — e é por isso que as DUAS memórias
    // abaixo existem: sem elas ela é `O(irmãos²)` em LEITURAS de ficheiro, e a régua lexical que
    // chama isto uma vez por ficheiro paga o quadrado da crate inteira.
    let mut parents = vec![dir.join("mod.rs")];
    if let (Some(gp), Some(dir_name)) = (dir.parent(), dir.file_name().and_then(|s| s.to_str())) {
        parents.push(gp.join(format!("{dir_name}.rs")));
    }
    for sib in irmaos(dir).iter() {
        if sib != path {
            parents.push(sib.clone());
        }
    }
    parents.iter().any(|p| {
        let decls = declaracoes(p);
        // O pai gateia-me com um `#[cfg(test)]` próprio…
        decls.iter().any(|d| d.cfg_test && d.casa(stem, path))
            // …ou ele PRÓPRIO é um módulo de teste, e então declarar-me basta.
            || (decls.iter().any(|d| d.casa(stem, path))
                && declared_under_cfg_test(p, depth - 1))
    })
}

/// Uma declaração de módulo lida de um ficheiro-pai — a forma memoizada do que o `decl_matches`
/// re-parsava a cada pergunta.
struct Decl {
    /// O nome que o `mod …;` declara (o último token antes do `;`).
    nome: String,
    /// O ficheiro para onde um `#[path = "…"]` imediatamente acima aponta, já resolvido contra o
    /// directório do AVÔ (que é como o `rustc` o lê a partir de um `<dir>/mod.rs`).
    caminho: Option<PathBuf>,
    /// Ela vem debaixo de um `#[cfg(test)]` (ou de um `cfg(all(test, …))`)?
    cfg_test: bool,
}

impl Decl {
    /// Esta declaração é a DESTE ficheiro — por nome, ou pelo `#[path]` que resolve para ele?
    fn casa(&self, stem: &str, path: &Path) -> bool {
        self.nome == stem || self.caminho.as_deref() == Some(path)
    }
}

/// ⭐⭐⭐ **As declarações de um ficheiro, lidas UMA vez por processo.**
///
/// ⛔⛔ **Sem esta memória a pergunta é `O(irmãos²)` em leituras de disco, e isso estava MEDIDO
/// ao contrário no comentário acima** (*«o custo da suíte inteira ficou abaixo de um segundo»* —
/// verdade quando a população eram dois painéis). A régua lexical pergunta isto **uma vez por
/// ficheiro** e cada pergunta relia TODOS os irmãos: a `ph2d-editor-core` (459 ficheiros, 4,42 MB)
/// lia-se a **1,18 MB/s** contra os **16,05 MB/s** de uma crate de 13 ficheiros — `13,6×` mais
/// devagar por ser maior, que é a assinatura de um quadrático e não de I/O.
///
/// ⚠️ **A memória é correcta pela mesma razão que a da [`crate::language_literals`]:** o fonte não
/// muda enquanto um binário de teste corre. Ela vive no processo e morre com ele.
///
/// ⚠️ **`BTreeMap` e não uma lista** — uma procura linear numa memória que cresce com os ficheiros
/// devolveria o cubo no lugar do quadrado (`CLAUDE.md` §5: `BTreeMap`, nunca `HashMap`).
fn declaracoes(parent: &Path) -> Arc<Vec<Decl>> {
    static MEMO: OnceLock<Mutex<BTreeMap<PathBuf, Arc<Vec<Decl>>>>> = OnceLock::new();
    let memo = MEMO.get_or_init(|| Mutex::new(BTreeMap::new()));
    if let Ok(m) = memo.lock()
        && let Some(v) = m.get(parent)
    {
        return Arc::clone(v);
    }
    LEITURAS.fetch_add(1, Ordering::Relaxed);
    let out = Arc::new(
        fs::read_to_string(parent).map_or_else(|_| Vec::new(), |src| parse_decls(&src, parent)),
    );
    if let Ok(mut m) = memo.lock() {
        m.insert(parent.to_path_buf(), Arc::clone(&out));
    }
    out
}

/// Os `.rs` de um directório, lidos uma vez — o `read_dir` era `O(irmãos)` por ficheiro.
fn irmaos(dir: &Path) -> Arc<Vec<PathBuf>> {
    static MEMO: OnceLock<Mutex<BTreeMap<PathBuf, Arc<Vec<PathBuf>>>>> = OnceLock::new();
    let memo = MEMO.get_or_init(|| Mutex::new(BTreeMap::new()));
    if let Ok(m) = memo.lock()
        && let Some(v) = m.get(dir)
    {
        return Arc::clone(v);
    }
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("rs") {
                out.push(p);
            }
        }
    }
    out.sort();
    let out = Arc::new(out);
    if let Ok(mut m) = memo.lock() {
        m.insert(dir.to_path_buf(), Arc::clone(&out));
    }
    out
}

fn parse_decls(src: &str, parent: &Path) -> Vec<Decl> {
    let mut out = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        // ⛔ **Um comentário no fim da linha do `mod` escondia o nome** (2026-09-13): `mod
        // line_seam_tests; // o seam do card Line` lia-se como o módulo `Line)`, e o ficheiro —
        // declarado sob `#[cfg(test)]` na linha de cima — contava como PRODUÇÃO. Apanhado pela régua
        // lexical sobre a `ph2d-tool-painter`.
        let t = line.split("//").next().unwrap_or_default().trim();
        if !t.starts_with("mod ") && !t.starts_with("pub mod ") && !t.starts_with("pub(") {
            continue;
        }
        // As linhas de atributo IMEDIATAMENTE acima da declaração.
        let mut cfg_test = false;
        let mut declared: Option<PathBuf> = None;
        let mut j = i;
        while j > 0 {
            let a = lines[j - 1].trim();
            // ⛔ **Um COMENTÁRIO entre os atributos escondia o `#[cfg(test)]`** (2026-09-16): a
            //    `ph2d-panel-motion-params` escreve `#[cfg(test)]`, quatro linhas `//` a explicar o
            //    sufixo, e só depois `#[path = "lib_gradient_tests.rs"] mod tests_gradient;` — e este
            //    laço parava no primeiro `//`, lendo o ficheiro de teste como PRODUÇÃO.
            if a.starts_with("//") {
                j -= 1;
                continue;
            }
            if !a.starts_with("#[") {
                break;
            }
            // ⚠️ `cfg(all(test, …))` também é SÓ-teste — um `all` com `test` dentro é falso fora de
            // `cargo test`. Medido 2026-09-12 pelo censo derivado de colisões: o
            // `ph2d-app-motion/src/motion_bridge_library_tests.rs` é declarado com
            // `#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]`
            // e esta função lia-o como PRODUÇÃO. ⛔ `cfg(any(test, …))` NÃO entra: compila no produto
            // com a feature ligada.
            if a.starts_with("#[cfg(test)]") || a.starts_with("#[cfg(all(test") {
                cfg_test = true;
            }
            if let Some(rest) = a.strip_prefix("#[path = \"")
                && let Some(rel) = rest.split('"').next()
            {
                declared = parent.parent().map(|d| d.join(rel));
            }
            j -= 1;
        }
        let nome = t
            .trim_end_matches(';')
            .rsplit(' ')
            .next()
            .unwrap_or_default()
            .to_string();
        out.push(Decl {
            nome,
            caminho: declared,
            cfg_test,
        });
    }
    out
}
