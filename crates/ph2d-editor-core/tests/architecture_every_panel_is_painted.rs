//! ⭐ **Todo painel do registro é alcançado pelo passeio de z-order.**
//!
//! # Por que este gate existe
//!
//! Registar um painel, ligar a feature e escrever a visibilidade **não chega**: o `paint_hero_screen`
//! percorre uma lista de `NodeId`s, e um painel cujo id não está lá é registado, visível e **nunca
//! pintado**. Nada quebra. Nada avisa. Os gates da própria crate do painel ficam todos verdes,
//! porque o que falta está noutra crate.
//!
//! `screens/hero/paint.rs` carrega **seis** notas a dizer isto — uma por vez que o defeito foi pago
//! (motion, timeline, physics, wet-tuning, tokens, authored, sculpt3d…). A sexta foi um smoke
//! reprovado do painel de modelagem 3D (Enio, 2026-08-19: *"o painel não abre"*), e é ela que
//! transformou a nota repetida neste teste.
//!
//! *Uma regra escrita seis vezes em comentário é uma regra que ninguém está a aplicar.*
//!
//! # ⚠️ SÃO DUAS ROTAS, e a primeira versão deste gate só conhecia uma
//!
//! ```text
//! z_order = hero.store.panel_z_order()        ← rota A: quem foi clicado, arrastado ou LEVANTADO
//!         ∪ PANEL_Z_ORDER_FALLBACK            ← rota B: a lista estática
//! ```
//!
//! A rota A não é só o utilizador: uma **ponte** pode levantar o painel dela com um
//! `bump_panel_z` explícito quando a tool acorda — é o que o `painter_bridge` faz com o painel de
//! camadas. Medir só a rota B declara «nunca pintado» um painel que é pintado em todo quadro.
//!
//! ⚠️ **E o defeito ficou LATENTE**, porque só aparece com as features da workspace ligadas: um
//! `cargo test -p ph2d-editor-core` não compila `ph2d-panel-painter-layers`, o registro não o contém,
//! e o gate passa. O vermelho só nasce no `nextest --workspace` do `ship.sh` — quinze waves depois.
//! *Um gate cuja resposta depende do conjunto de features tem de ser lido no conjunto que o CI usa.*
//!
//! A exceção da rota A é **explícita e provada**: cada entrada nomeia a const, e o segundo teste
//! deste arquivo **procura a chamada na árvore**. Uma ponte que deixe de levantar o painel torna a
//! entrada obsoleta e o gate diz.

use std::path::{Path, PathBuf};

use ph2d_editor_core::panel::with_registry_ref;
use ph2d_editor_core::screens::hero::PANEL_Z_ORDER_FALLBACK;

/// Painéis **fora** do `PANEL_Z_ORDER_FALLBACK` que chegam ao passeio pela rota A — uma ponte
/// levanta-os com `bump_panel_z` quando a tool acorda.
///
/// ⚠️ `(id do manifesto, o que procurar na árvore)`. A segunda metade é o que prova a entrada: ela
/// tem de casar uma chamada real de `bump_panel_z`, senão a exceção está a esconder um painel mudo.
const RAISED_BY_A_BRIDGE: &[(&str, &str)] = &[(
    // `shells/desktop/src/render_loop/painter_bridge.rs` levanta-o ao ativar a tool Painter.
    // ⚠️ Ele **não** está no fallback de propósito: o painel de camadas só faz sentido com a tool
    // acordada, e a ponte é quem sabe disso.
    "painter_layers",
    "bump_panel_z(ph2d_editor::ids::PAINTER_LAYERS_PANEL)",
)];

fn repo_root() -> PathBuf {
    // `crates/ph2d-editor-core` → a raiz.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("a crate vive dois níveis abaixo da raiz")
        .to_path_buf()
}

/// Todo o `.rs` sob os diretórios onde uma **ponte** pode viver.
///
/// ⚠️ **`tests/` fica de FORA, e é o que impede o gate de ser tautológico:** a primeira versão
/// varria a árvore inteira, e a árvore inteira inclui **este arquivo** — que carrega a agulha
/// dentro do [`RAISED_BY_A_BRIDGE`]. Ele encontrava a própria declaração e passava sempre. Uma
/// prova de mutação apanhou-o: tirar a chamada real da ponte deixava-o **verde**.
///
/// A regra que sobra é a certa por si só: *uma ponte vive no `src`, nunca num teste* — um
/// `bump_panel_z` dentro de um `#[test]` prova que o teste levanta o painel, não que o **app** o
/// levanta.
fn tree_sources() -> String {
    let root = repo_root();
    let mut out = String::new();
    let mut stack = vec![root.join("shells"), root.join("crates")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path
                    .file_name()
                    .is_some_and(|n| n == "target" || n == "tests")
                {
                    continue;
                }
                stack.push(path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs")
                && let Ok(text) = std::fs::read_to_string(&path)
            {
                out.push_str(&text);
            }
        }
    }
    out
}

#[test]
fn every_registered_panel_is_reachable_by_the_z_order_walk() {
    // O registro que o app de facto monta. `register_all_panels` é idempotente.
    ph2d_panel_registry_init::register_all_panels();

    let missing: Vec<&'static str> = with_registry_ref(|reg| {
        reg.panels()
            .iter()
            .filter(|p| !PANEL_Z_ORDER_FALLBACK.contains(&p.manifest.panel_node_id))
            .map(|p| p.manifest.id)
            .filter(|id| !RAISED_BY_A_BRIDGE.iter().any(|(name, _)| name == id))
            .collect()
    });

    assert!(
        missing.is_empty(),
        "painéis REGISTADOS que o passeio de z-order nunca alcança (registados, visíveis e \
         NUNCA pintados — nada quebra, nada avisa):\n  {}\n\n\
         fix: acrescente o `NodeId` do painel a `PANEL_Z_ORDER_FALLBACK` em \
         `screens/hero/paint.rs` — ou, se uma PONTE o levanta com `bump_panel_z`, declare-o em \
         `RAISED_BY_A_BRIDGE` neste arquivo (a chamada tem de existir; o gate irmão confere).",
        missing.join("\n  ")
    );
}

/// ⚠️ **A exceção tem de ser VERDADE.** Uma entrada em [`RAISED_BY_A_BRIDGE`] afirma que existe uma
/// ponte a levantar aquele painel — e uma afirmação sem prova é como uma allowlist apodrece: a
/// ponte muda de nome, a chamada sai, e a exceção continua a calar o gate sobre um painel que
/// deixou de ser pintado.
#[test]
fn every_bridge_raise_exception_has_a_real_call_site() {
    let sources = tree_sources();
    let stale: Vec<&str> = RAISED_BY_A_BRIDGE
        .iter()
        .filter(|(_, call)| !sources.contains(*call))
        .map(|(name, _)| *name)
        .collect();
    assert!(
        stale.is_empty(),
        "exceções de `RAISED_BY_A_BRIDGE` sem chamada de `bump_panel_z` na árvore — o painel \
         deixou de ser levantado e a exceção está a esconder isso:\n  {}",
        stale.join("\n  ")
    );
}
