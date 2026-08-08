//! **O CENSO DOS PARÂMETROS** — a medição que a varredura B3 do doc 88 exige antes de curar.
//!
//! A ordem do Enio é *"cada nó com o conjunto de params do essencial ao estado-da-arte"*, e a
//! §0 do CLAUDE.md manda MEDIR antes de decidir. A pergunta que este arquivo responde não é
//! *"quantos params tem o catálogo?"* — é **quais nós estão MAGROS**, para a curadoria (que é
//! curadoria, não design) começar por eles em vez de pela ordem alfabética.
//!
//! ⚠️ **Roda aqui e não na shell de propósito:** esta crate é o ponto onde TODO nó é registrado
//! (`register_all_nodes`), e é o build mais barato que enxerga os 118. Uma sonda na shell mediria
//! o mesmo e custaria o app inteiro.
//!
//! Rodar: `cargo test -p ph2d-node-registry-init --test param_census -- --ignored --nocapture`

use ph2d_node_registry::NodeRegistry;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// **SONDA — o retrato de cada nó: quantos params, quantos anotados, quantos com unidade.**
///
/// Um param sem `ParamUiHint` cai num slider genérico `0..1` com o NOME CRU do param como
/// rótulo — é o que a §3 do doc 88 chama de fallback, e ele é honesto mas mudo. A coluna
/// `hint` é, portanto, a que separa *"o nó tem um controle"* de *"o nó tem uma interface"*.
#[test]
#[ignore = "sonda de diagnostico: cargo test -p ph2d-node-registry-init --test param_census -- --ignored --nocapture"]
fn measure_the_param_surface_of_every_node() {
    let reg = registry();

    let mut rows: Vec<_> = reg
        .manifests()
        .map(|m| {
            let hints = reg.param_ui(m.id).unwrap_or(&[]);
            let units = reg.param_units(m.id).unwrap_or(&[]);
            let groups = reg.param_groups(m.id);
            // Um hint pode anotar um TEXT param (curva/gradiente/paleta/fórmula), que não é
            // `ParamSpec` — então a contagem de hints pode passar a de params, e isso não é erro.
            let hinted = m
                .params
                .iter()
                .filter(|p| hints.iter().any(|h| h.param == p.name))
                .count();
            let text_hints = hints
                .iter()
                .filter(|h| !m.params.iter().any(|p| p.name == h.param))
                .count();
            (
                m.name,
                m.params.len(),
                hinted,
                text_hints,
                units.len(),
                groups.len(),
            )
        })
        .collect();
    rows.sort_by_key(|r| r.0);

    println!(
        "\n{:<34} {:>6} {:>6} {:>6} {:>6} {:>7}",
        "no", "params", "hint", "text", "unid", "grupos"
    );
    println!("{}", "-".repeat(70));
    let mut bare = Vec::new();
    let mut thin = Vec::new();
    for (name, params, hinted, text, units, groups) in &rows {
        println!("{name:<34} {params:>6} {hinted:>6} {text:>6} {units:>6} {groups:>7}");
        if *params > 0 && *hinted == 0 {
            bare.push(*name);
        }
        if params + text <= 2 {
            thin.push((*name, *params + *text));
        }
    }

    let total: usize = rows.iter().map(|r| r.1).sum();
    let hinted: usize = rows.iter().map(|r| r.2).sum();
    let with_units: usize = rows.iter().map(|r| r.4).sum();
    println!(
        "\n{} nos - {total} params, {hinted} com hint, {with_units} com unidade",
        rows.len()
    );

    println!(
        "\nSEM HINT NENHUM (o slider generico com o nome cru): {}",
        bare.len()
    );
    for n in &bare {
        println!("  {n}");
    }

    println!("\nMAGROS (<=2 controles no total): {}", thin.len());
    for (n, c) in &thin {
        println!("  {n:<34} {c}");
    }
}
