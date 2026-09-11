//! ⛔⛔ **A SEXTA porta de registo de um painel — a que mata em silêncio.**
//!
//! Report do Enio, 2026-08-30: *«clicar no Pill Assets não abre nenhuma janela»*. O painel
//! existia, estava na `PANEL_REGISTRY`, tinha 15 gates de costura verdes, entrava no z-order walk,
//! e o `ph2d-panel-sync` tinha regenerado os dois blocos de codegen. **Nunca foi construído.**
//!
//! # O mecanismo
//!
//! O `shells/desktop` depende do `ph2d-panel-registry-init` com **`default-features = false`** e
//! enumera cada painel numa feature própria. ⇒ a feature `default` **da crate do registro não vale
//! nada aqui**: quem decide o que o binário tem é a lista `default` deste `Cargo.toml`.
//!
//! ⚠️ **E o gate que devia apanhá-lo mede outra coisa.** O
//! `build_typed_registry_matches_enabled_features` corre dentro do `ph2d-panel-registry-init`, com
//! os defaults **daquela** crate — que incluíam o painel novo. Ele ficou verde sobre um binário
//! que não o tinha: *um gate que mede os defaults de uma crate não diz nada sobre o que o binário
//! ship-a.*
//!
//! # O que este gate faz
//!
//! Lê os dois `Cargo.toml` como TEXTO (nenhuma feature precisa de estar ligada para ele correr) e
//! afirma, para cada `panel-X` que o registro oferece, que **este shell o alcança**: ou ele está na
//! lista `default` daqui, ou está em [`DELIBERATELY_OUT`] com o motivo escrito.
//!
//! ⚠️ **A metade JUSTA:** ele também reprova uma entrada de [`DELIBERATELY_OUT`] que já não
//! descreve nada — senão a lista vira licença (a lei das catraltas, `CLAUDE.md` §5.0).

use std::collections::BTreeSet;
use std::path::Path;

/// Painéis que este shell **NÃO** carrega, cada um com o motivo.
///
/// ⛔ Vazio hoje, e isso é o estado correcto: todo painel da árvore é alcançável do desktop. Uma
/// entrada aqui é dívida declarada, não um esconderijo.
const DELIBERATELY_OUT: &[(&str, &str)] = &[];

fn read(rel: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .join(rel);
    std::fs::read_to_string(&root).unwrap_or_else(|e| panic!("nao consegui ler {root:?}: {e}"))
}

/// As features `panel-*` que a crate do registro oferece — lidas do bloco de codegen do
/// `ph2d-panel-sync`, que é a fonte.
fn panels_the_registry_offers() -> BTreeSet<String> {
    let toml = read("crates/ph2d-panel-registry-init/Cargo.toml");
    let body = toml
        .split("# <ph2d-panel-sync:features:begin>")
        .nth(1)
        .and_then(|s| s.split("# <ph2d-panel-sync:features:end>").next())
        .expect("os marcadores do ph2d-panel-sync sumiram do registry-init");
    body.lines()
        .filter_map(|l| l.split_once('='))
        .map(|(name, _)| name.trim().to_string())
        .filter(|n| n.starts_with("panel-"))
        .collect()
}

/// A lista `default` deste shell.
fn shell_default_features() -> BTreeSet<String> {
    let toml = read("shells/desktop/Cargo.toml");
    let body = toml
        .split("\ndefault = [")
        .nth(1)
        .and_then(|s| s.split(']').next())
        .expect("a lista `default` do shell sumiu");
    body.lines()
        .filter_map(|l| l.split('"').nth(1))
        .map(str::to_string)
        .collect()
}

#[test]
fn every_registry_panel_is_reachable_from_this_shell() {
    let offered = panels_the_registry_offers();
    assert!(
        offered.len() > 10,
        "o parse achou {} painéis — o bloco do ph2d-panel-sync mudou de forma",
        offered.len()
    );
    let enabled = shell_default_features();
    assert!(
        enabled.contains("panel-inspector"),
        "o parse da lista `default` do shell nao achou nem o Inspector — ele mudou de forma"
    );
    let out: BTreeSet<&str> = DELIBERATELY_OUT.iter().map(|(n, _)| *n).collect();

    let missing: Vec<&String> = offered
        .iter()
        .filter(|p| !enabled.contains(*p) && !out.contains(p.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "estes paineis existem no registo e o binario do desktop NAO os carrega \
         (o pill abre e nao acontece nada): {missing:?}\n\
         Acrescente a feature a' lista `default` de shells/desktop/Cargo.toml, ou declare-a em \
         DELIBERATELY_OUT com o motivo."
    );

    // A metade JUSTA: uma entrada que ja' nao descreve nada.
    let stale: Vec<&str> = DELIBERATELY_OUT
        .iter()
        .map(|(n, _)| *n)
        .filter(|n| !offered.contains(*n) || enabled.contains(*n))
        .collect();
    assert!(
        stale.is_empty(),
        "estas entradas de DELIBERATELY_OUT ja' nao descrevem nada (o painel sumiu, ou ele passou \
         a estar no `default`): {stale:?} — apague-as, senao a lista vira licenca."
    );
}

/// ⭐⭐ **A prova em RUNTIME, com as features DESTE binário.**
///
/// ⚠️ O gate acima lê texto; este constrói o registo real. Ele corre no `shells/desktop`, logo a
/// resolução de features é **a do binário** — que é exactamente o que o
/// `build_typed_registry_matches_enabled_features` (dentro do `ph2d-panel-registry-init`, com os
/// defaults DAQUELA crate) não consegue medir.
#[test]
fn the_registry_this_binary_builds_contains_the_asset_browser() {
    let reg = ph2d_panel_registry_init::build_typed_registry();
    assert!(
        reg.find_by_panel_node_id(ph2d_editor::ids::ASSET_PANEL)
            .is_some(),
        "o navegador de assets NAO esta' no registo deste binario — o pill abre e nao acontece nada"
    );
    assert!(
        reg.find_by_panel_node_id(ph2d_editor::ids::INSP_PANEL)
            .is_some(),
        "nem o Inspector esta' — a leitura do registo mudou de forma, re-leia este gate"
    );
}
