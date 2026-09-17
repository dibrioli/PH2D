//! ⭐⭐⭐ **O NOME DE UM NÓ É UMA CHAVE, E ELA DERIVA DO TIPO DELE.**
//!
//! ⛔⛔ **Porque este gate mora AQUI e não em 135 sítios:** os nomes vivem um por crate
//! (`ph2d-node-*`), e um censo por crate seria 135 ficheiros a dizer a mesma coisa — cada um a
//! poder envelhecer sozinho. ⭐ Esta crate é a **única porta** por onde todos passam: o
//! `register_all_nodes` regista os 137, e o registo sabe, ao mesmo tempo, o **tipo** (do
//! `NodeManifest`) e o **nome** (do `NodeUiManifest`). *Um gate que precisa do par tem de viver
//! onde o par existe.*
//!
//! ⚠️ **O nome NÃO é derivável do tipo** — `motion.four_point_warp` mostra-se *Corner Pin* e
//! `source.lsystem` mostra-se *L-System*. A **chave** é, e é isso que este ficheiro afirma.

use ph2d_node_registry::NodeRegistry;

#[test]
fn every_node_name_is_a_key_derived_from_its_type() {
    let mut reg = NodeRegistry::default();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("o registo tem de montar");
    let tipos: Vec<(&'static str, ph2d_nodegraph::node::NodeTypeId)> =
        reg.manifests().map(|m| (m.name, m.id)).collect();
    // ⛔ Piso de população: um registo vazio passaria trivialmente.
    assert!(
        tipos.len() >= 130,
        "o registo tem {} tipos — encolheu?",
        tipos.len()
    );
    let mut erradas = Vec::new();
    let mut com_nome = 0usize;
    for (nome_do_tipo, id) in &tipos {
        let Some(ui) = reg.ui_manifest(*id) else {
            continue;
        };
        com_nome += 1;
        let esperada = format!("node.{nome_do_tipo}.name");
        if ui.display_key != esperada {
            erradas.push(format!(
                "{nome_do_tipo} declara {:?} e a derivação do tipo dá {esperada:?}",
                ui.display_key
            ));
        }
    }
    assert!(
        com_nome >= 125,
        "só {com_nome} tipos declaram nome — a população encolheu?"
    );
    assert!(
        erradas.is_empty(),
        "estes nomes não são a chave derivada do tipo:\n  {}\n\nA lei é `node.<tipo>.name` — ela \
         não se escolhe, e o texto vive em `crates/ph2d-i18n/src/node_catalog.rs`.",
        erradas.join("\n  ")
    );
}

/// ⭐⭐ **E a chave RESOLVE-SE** — a metade que o gate irmão não pode fazer.
///
/// ⛔ Uma chave bem derivada e ausente da tabela pinta o **identificador cru no cartão** (`tr`
/// devolve a própria chave), e o cartão é o que o artista lê primeiro. ⚠️ E o `tr` de uma chave
/// desconhecida faz `leak_key` (`Box::leak`) — num grafo repintado por quadro, é um vazamento por
/// quadro e por cartão.
#[test]
fn every_node_name_key_resolves_to_a_word() {
    let mut reg = NodeRegistry::default();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("o registo tem de montar");
    let mut cruas = Vec::new();
    let mut n = 0usize;
    for m in reg.manifests().collect::<Vec<_>>() {
        let Some(ui) = reg.ui_manifest(m.id) else {
            continue;
        };
        n += 1;
        let palavra = ph2d_i18n::tr(ui.display_key);
        if palavra == ui.display_key {
            cruas.push(format!("{}: {:?}", m.name, ui.display_key));
        }
    }
    assert!(n >= 125, "só {n} tipos com nome — a população encolheu?");
    assert!(
        cruas.is_empty(),
        "estas chaves não têm palavra em `node_catalog.rs` e o cartão pinta o identificador:\n  {}",
        cruas.join("\n  ")
    );
}
