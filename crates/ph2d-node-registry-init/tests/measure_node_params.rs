//! **SONDA** — o param list de CADA nó, tirado do registry (a fonte), não de um doc.
//!
//! ⚠️ **Ela existe porque a conferência do doc 89 mede o produto contra referências, e a
//! coluna «params hoje» de cada célula é uma FOTOGRAFIA.** Uma wave que acrescenta um param
//! fecha a célula dela e deixa as vizinhas a descrever um nó que já não existe — e a folha
//! passa a contar como aberto o que já shipou. Medido em 2026-08-19 na folha 01: a célula
//! dizia *"`motion.emitter`, 10 params"* e o manifesto tinha **20**; três das quatro células
//! do nó pediam coisas que já lá estavam (`size_random`, `dir_mode`, o trio `burst_*`).
//!
//! O consumidor disto é [`conferencia_vs_manifesto.py`](../../../docs/Motion%20Nodes/ferramentas/conferencia_vs_manifesto.py),
//! que cruza esta saída com as folhas e sai VERMELHO quando uma contagem discorda.
//!
//! Correr: `cargo test -p ph2d-node-registry-init --test measure_node_params -- --ignored
//! --nocapture`

use ph2d_node_registry::NodeRegistry;

#[test]
#[ignore = "sonda: imprime, não afirma"]
fn every_node_and_the_params_it_really_has() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    let mut rows: Vec<(String, usize, String)> = reg
        .manifests()
        .map(|m| {
            let names: Vec<&str> = m.params.iter().map(|p| p.name).collect();
            (m.name.to_string(), names.len(), names.join(" "))
        })
        .collect();
    rows.sort();
    println!("# nó\tn\tparams   (derivado do registry)");
    for (name, n, params) in rows {
        println!("{name}\t{n}\t{params}");
    }
}
