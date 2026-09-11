//! ⛔⛔⛔ **UM PAINEL PODE ESTAR NO REGISTO E FORA DO BINÁRIO — e nada avisa.**
//!
//! Report do Enio, 2026-09-01: *«Widget Lab não abriu»*. O painel estava escrito, registado,
//! testado (10 gates verdes), no `PANEL_Z_ORDER_FALLBACK`, com a linha no menu *Window* e o id no
//! `WidgetStore`. **E não existia no binário.**
//!
//! # O mecanismo
//!
//! O shell põe `default-features = false` no `ph2d-panel-registry-init` e **re-declara** cada
//! painel como feature própria:
//!
//! ```toml
//! ph2d-panel-registry-init = { path = "...", default-features = false }
//! # ...
//! panel-widget-gallery = ["ph2d-panel-registry-init/panel-widget-gallery"]
//! ```
//!
//! ⇒ um painel no `default` **daquela** crate e sem linha **aqui** é compilado para fora. E o que
//! fica na tela é o pior resultado possível: a linha do menu continua **pintada e clicável**,
//! porque quem a pinta é a `ph2d-editor-core`, que não sabe nada de features do shell. O artista
//! clica e não acontece nada.
//!
//! # ⚠️ Por que NENHUM gate existente o apanha
//!
//! | gate | o que ele vê | por que falha aqui |
//! |---|---|---|
//! | `every_window_menu_row_reaches_a_consumer` | corre em `ph2d-panel-registry-init` | ali o painel **existe** (features `default` da própria crate) |
//! | `every_registered_panel_is_reachable_by_the_z_order_walk` | idem | idem |
//! | `build_typed_registry_matches_enabled_features` | conta o que as features ligam | é **consistente** com um painel desligado |
//!
//! ⭐ É a lei que o `CLAUDE.md` §2 já escreve para os testes — *«`-p <crate>` sozinho usa as
//! features POBRES: corra junto com o shell»* — aplicada às **features**, e não aos testes.
//! *Uma crate testada sozinha é testada num mundo que o produto não habita.*
//!
//! # A régua
//!
//! Os dois manifestos são a fonte; não há lista escrita à mão. Todo `panel-*` que a registry liga
//! por omissão tem de (a) ter feature homónima no shell e (b) estar no `default` do shell.

use std::path::PathBuf;

fn manifest(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("nao li {}: {e}", p.display()))
}

/// Os nomes dentro do bloco `default = [ ... ]` de um manifesto.
///
/// ⚠️ **Os comentários descascam-se ANTES de dividir por vírgula, e não depois.** A 1.ª redacção
/// dividia primeiro e descartava os pedaços que começavam por `#` — o que deitava fora **seis**
/// painéis, porque no manifesto da registry o comentário vem na linha ACIMA da entrada e cai no
/// mesmo pedaço. Só o controlo positivo do corpus (`>= 20`) o apanhou.
/// *Um parser de manifesto que perde entradas em silêncio é a mesma doença que este gate cura.*
fn default_features(toml: &str) -> Vec<String> {
    let Some(i) = toml.find("\ndefault = [") else {
        return Vec::new();
    };
    let rest = &toml[i + 11..];
    let Some(j) = rest.find(']') else {
        return Vec::new();
    };
    let body: String = rest[..j]
        .lines()
        .map(|l| l.split('#').next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    body.split(',')
        .filter_map(|s| {
            let s = s.trim().trim_matches('"').trim();
            (!s.is_empty()).then(|| s.to_string())
        })
        .collect()
}

/// Os nomes de feature declarados (o lado esquerdo de `nome = [...]`).
fn declared_features(toml: &str) -> Vec<String> {
    toml.lines()
        .filter_map(|l| {
            let l = l.trim();
            if l.starts_with('#') {
                return None;
            }
            let (name, _) = l.split_once('=')?;
            let name = name.trim();
            (!name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'))
            .then(|| name.to_string())
        })
        .collect()
}

/// ⭐⭐⭐ **Todo painel que a registry liga por omissão chega ao binário do shell.**
#[test]
fn every_panel_the_registry_ships_reaches_the_binary() {
    let shell = manifest("Cargo.toml");
    let registry = manifest("../../crates/ph2d-panel-registry-init/Cargo.toml");

    let wanted: Vec<String> = default_features(&registry)
        .into_iter()
        .filter(|f| f.starts_with("panel-"))
        .collect();
    // ⚠️ Controle positivo do CORPUS: um parser partido devolveria zero e o gate passava sobre
    // nada — que é exactamente a forma de falha que ele existe para curar.
    assert!(
        wanted.len() >= 20,
        "li so' {} painéis no `default` da registry — o parser do manifesto partiu-se e este \
         gate ficaria verde por vácuo",
        wanted.len()
    );

    let shell_declared = declared_features(&shell);
    let shell_default = default_features(&shell);

    let mut missing = Vec::new();
    let mut declared_but_off = Vec::new();
    for f in &wanted {
        if !shell_declared.contains(f) {
            missing.push(f.clone());
        } else if !shell_default.contains(f) {
            declared_but_off.push(f.clone());
        }
    }

    assert!(
        missing.is_empty(),
        "estes painéis existem no `default` do `ph2d-panel-registry-init` e o shell NAO declara \
         feature para eles — sao compilados para FORA do binario, e a linha do menu *Window* fica \
         pintada, clicavel e MORTA:\n  {}\n\n\
         cura: em `shells/desktop/Cargo.toml`, acrescente\n    \
         {} = [\"ph2d-panel-registry-init/{}\"]\n  \
         e ponha o nome no `default` do shell.",
        missing.join("\n  "),
        missing[0],
        missing[0],
    );

    assert!(
        declared_but_off.is_empty(),
        "estes painéis te^m feature no shell mas ficaram FORA do `default` dele — o build normal \
         nao os leva:\n  {}\n\n\
         \u{26a0} Se a omissao for deliberada (uma distribuicao `lite`), a feature nao devia estar \
         no `default` da registry tambem.",
        declared_but_off.join("\n  ")
    );
}

/// ⚠️ **E a varredura sabe dizer «não».** Sem isto, um `declared_features` que devolvesse tudo (ou
/// um `default_features` que devolvesse a lista errada) faria o gate acima passar sempre.
#[test]
fn the_manifest_reader_can_still_say_no() {
    let shell = manifest("Cargo.toml");
    let declared = declared_features(&shell);
    assert!(
        declared.contains(&"panel-widget-gallery".to_string()),
        "o leitor nao ve^ uma feature que existe — ele esta' a ler o ficheiro errado"
    );
    assert!(
        !declared.contains(&"panel-uma-feature-que-nao-existe".to_string()),
        "o leitor ve^ uma feature inventada — ele nao esta' a medir o que diz medir"
    );
    assert!(
        default_features(&shell).contains(&"panel-inspector".to_string()),
        "o `default` do shell nao inclui o Inspector — ou o parser do bloco `default` partiu"
    );
}
