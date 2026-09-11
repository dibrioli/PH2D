//! **A página de versões que todo agente novo lê é DERIVADA do `Cargo.lock` — ou fica vermelha.**
//!
//! ## Por que este gate existe
//!
//! O `docs/IntegracaoMultiAgente/STACK_VERSOES.md` responde, numa página, a pergunta que toda
//! linha nova faz antes da primeira linha de código: *«que `wgpu`? que `vello`? que Rust?»*.
//! Uma página assim é útil exactamente na medida em que é **verdadeira** — e uma tabela de
//! versões escrita à mão é a coisa que este repo já viu envelhecer mais depressa. Na auditoria
//! de 2026-08-30 foram achadas **~70 afirmações de versão obsoletas** espalhadas pelos docs, e a
//! lei do §5.0 do `CLAUDE.md` já foi paga cinco vezes: *«a fonte de cada número é o código, não
//! a seção»*.
//!
//! ⇒ A página fica, porque um agente novo não deve ter de correr uma sonda para saber em que
//! stack está. Mas ela passa a ser **gateada**: quem subir uma dependência ou é obrigado a
//! editá-la, ou vê este portão vermelho. *A tabela não pode envelhecer em silêncio.*
//!
//! ## As duas metades, e por que a segunda não é decorativa
//!
//! A primeira metade compara cada par `(crate, versão)` da tabela com o `Cargo.lock`. A segunda
//! afirma que a tabela de facto **tem** linhas — porque um parse que não casa nada devolve zero
//! divergências, e *um balde que ninguém enche lê-se como perfeito* (lição medida desta casa: uma
//! régua de valência lia mediana `0,0` como «perfeito» quando o balde nunca era preenchido).
//!
//! ## O terceiro teste: as quatro cópias de `glam`
//!
//! Elas são o **MECANISMO** e não resíduo — a física corre `scalar-math` (HR-5, determinismo) e as
//! crates de desenho ficam com SIMD; unificar desligaria o SIMD de oito delas. É uma recusa
//! MEDIDA (`docs/Atualizar Stack/04_registro.md` §15). Um gate que só olhasse a versão primária
//! ficaria verde no dia em que alguém «arrumasse» isso. Este fica vermelho, e obriga a decisão a
//! ser tomada de propósito.

use std::collections::BTreeMap;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

const DOC: &str = "docs/IntegracaoMultiAgente/STACK_VERSOES.md";

/// Nomes da 1.ª coluna que **não** são crates — cada um com a sua própria fonte de verdade.
const NOT_A_CRATE: [&str; 2] = ["Rust (toolchain)", "edition"];

/// Lê o valor de uma chave `chave = "valor"` do TOML, sem dep de parser.
fn toml_string(text: &str, key: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .find_map(|l| {
            let rest = l.strip_prefix(key)?.trim_start();
            let rest = rest.strip_prefix('=')?.trim();
            rest.strip_prefix('"')?.split('"').next().map(str::to_owned)
        })
}

/// `Cargo.lock` → `nome -> [versões]`. Um nome pode ter várias (é o caso do `glam`).
fn lockfile_versions(lock: &str) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut name: Option<String> = None;
    for line in lock.lines().map(str::trim) {
        if line == "[[package]]" {
            name = None;
        } else if let Some(v) = line.strip_prefix("name = ") {
            name = v.trim_matches('"').to_owned().into();
        } else if let Some(v) = line.strip_prefix("version = ")
            && let Some(n) = name.take()
        {
            out.entry(n)
                .or_default()
                .push(v.trim_matches('"').to_owned());
        }
    }
    out
}

/// Tira a ênfase markdown e o espaço de uma célula.
fn plain(cell: &str) -> String {
    cell.replace("**", "").replace('*', "").trim().to_owned()
}

/// As linhas da tabela, já em pares `(nome, versão)`.
///
/// A célula de versão pode trazer um parêntese em itálico com contexto (as cópias do `glam`);
/// ele é cortado de propósito — quem o afirma é o teste dedicado, mais abaixo.
fn doc_pairs(md: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for line in md.lines().map(str::trim) {
        if !line.starts_with('|') || line.starts_with("|---") {
            continue;
        }
        let cells: Vec<&str> = line.trim_matches('|').split('|').collect();
        if cells.len() < 2 {
            continue;
        }
        let names_cell = plain(cells[0]);
        // O parêntese de contexto sai ANTES do split: ele carrega barras.
        let vers_cell = plain(cells[1].split("*(").next().unwrap_or(""));
        if names_cell.is_empty() || vers_cell.is_empty() {
            continue;
        }
        // Uma versão começa por dígito. Isto salta o cabeçalho da tabela sozinho.
        if !vers_cell.starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }
        let names: Vec<String> = names_cell.split('/').map(|s| s.trim().to_owned()).collect();
        let vers: Vec<String> = vers_cell.split('/').map(|s| s.trim().to_owned()).collect();
        assert!(
            vers.len() == names.len() || vers.len() == 1,
            "linha ambígua em {DOC}: {} nome(s) para {} versão(ões) — {line}\n\
             Escreva uma versão por nome, ou uma só quando todas partilham a mesma.",
            names.len(),
            vers.len()
        );
        for (i, n) in names.iter().enumerate() {
            pairs.push((n.clone(), vers.get(i).unwrap_or(&vers[0]).clone()));
        }
    }
    pairs
}

#[test]
fn the_stack_versions_page_matches_the_lockfile() {
    let r = root();
    let md = std::fs::read_to_string(r.join(DOC)).unwrap_or_else(|e| panic!("{DOC}: {e}"));
    let lock = std::fs::read_to_string(r.join("Cargo.lock")).expect("Cargo.lock");
    let manifest = std::fs::read_to_string(r.join("Cargo.toml")).expect("Cargo.toml da raiz");
    let toolchain =
        std::fs::read_to_string(r.join("rust-toolchain.toml")).expect("rust-toolchain.toml");

    let locked = lockfile_versions(&lock);
    let mut wrong: Vec<String> = Vec::new();

    for (name, want) in doc_pairs(&md) {
        // As duas linhas que não são crates têm fonte própria.
        if NOT_A_CRATE.contains(&name.as_str()) {
            let (got, from) = match name.as_str() {
                "Rust (toolchain)" => (
                    toml_string(&toolchain, "channel").unwrap_or_default(),
                    "rust-toolchain.toml `channel`",
                ),
                _ => (
                    toml_string(&manifest, "edition").unwrap_or_default(),
                    "Cargo.toml `edition`",
                ),
            };
            if got != want {
                wrong.push(format!("{name}: a página diz {want}, o {from} diz {got}"));
            }
            continue;
        }

        match locked.get(&name) {
            None => wrong.push(format!(
                "{name}: a página lista {want} e o Cargo.lock não tem essa crate nenhuma"
            )),
            Some(vs) if !vs.contains(&want) => wrong.push(format!(
                "{name}: a página diz {want}, o Cargo.lock tem {}",
                vs.join(" / ")
            )),
            Some(_) => {}
        }
    }

    // A MSRV é o pin, e a página afirma o pin — logo ela afirma a MSRV. Se um dia divergirem,
    // é o `architecture_msrv_is_the_pinned_toolchain` que explica; aqui só não deixamos passar.
    let msrv = toml_string(&manifest, "rust-version").unwrap_or_default();
    let pin = toml_string(&toolchain, "channel").unwrap_or_default();
    if msrv != pin {
        wrong.push(format!(
            "a MSRV ({msrv}) e o pin ({pin}) divergiram — a página afirma que são o mesmo número"
        ));
    }

    assert!(
        wrong.is_empty(),
        "{DOC} envelheceu contra o Cargo.lock:\n  {}\n\n\
         Subir uma dependência OBRIGA a editar essa página — é ela que um agente novo lê antes\n\
         da primeira linha de código, e uma tabela de versões errada custa mais que a ausência\n\
         dela. Corra `bash scripts/stack-audit.sh --tetos` e escreva o que ele deu.",
        wrong.join("\n  ")
    );
}

/// **Controle: a tabela tem linhas, e as que mais importam estão lá.**
///
/// Sem isto, um `|` a menos numa linha de cabeçalho, ou uma renomeação do ficheiro seguida de um
/// `unwrap_or_default()`, faria o parse casar **zero** pares — e zero divergências lê-se como
/// aprovado. As cinco crates abaixo são as que decidem uma linha de trabalho inteira.
#[test]
fn the_page_actually_carries_the_versions_it_promises() {
    let r = root();
    let md = std::fs::read_to_string(r.join(DOC)).unwrap_or_else(|e| panic!("{DOC}: {e}"));
    let pairs = doc_pairs(&md);

    assert!(
        pairs.len() >= 14,
        "{DOC} devia listar pelo menos 14 pares (nome, versão) e o parse achou {} — \
         a tabela mudou de forma e este gate deixou de a ler.",
        pairs.len()
    );

    for must in [
        "Rust (toolchain)",
        "wgpu",
        "vello",
        "parley",
        "rapier2d",
        "bevy_ecs",
    ] {
        assert!(
            pairs.iter().any(|(n, _)| n == must),
            "{DOC} não lista `{must}` — é uma das entradas que decidem uma linha inteira."
        );
    }
}

/// **As cópias de `glam` são o mecanismo, não resíduo — e unificá-las é uma DECISÃO.**
///
/// A física corre `scalar-math` (HR-5, determinismo) e as crates de desenho ficam com SIMD. A
/// unificação foi medida e recusada (`docs/Atualizar Stack/04_registro.md` §15). Este gate não
/// impede a decisão; impede que ela seja tomada **sem querer**, num `cargo update` distraído.
#[test]
fn the_glam_copies_are_still_the_mechanism() {
    let r = root();
    let lock = std::fs::read_to_string(r.join("Cargo.lock")).expect("Cargo.lock");
    let locked = lockfile_versions(&lock);
    let glam = locked.get("glam").expect("`glam` sumiu do Cargo.lock");

    assert!(
        glam.len() > 1,
        "o `glam` colapsou numa cópia só ({}). Isso desliga o `scalar-math` da física ou o SIMD \
         das crates de desenho — uma das duas, e as duas são regressões medidas.\n\
         Se a unificação foi de propósito, é AQUI que se escreve o número novo, com a medição \
         ao lado (§15 do registo) e a página STACK_VERSOES.md actualizada.",
        glam.join(" / ")
    );
}
