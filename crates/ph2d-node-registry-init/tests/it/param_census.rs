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

/// **SONDA — O BALANCEAMENTO: onde o DEFAULT senta na régua, e quanto da régua o artista alcança.**
///
/// O report do Enio (doc 88 §10) foi *"sliders mal balanceados; a menor mudança faz um extremo
/// efeito"*, e depois **"quase tudo nesse módulo está mal ajustado"**. A §10 curou a classe cuja
/// causa é a LEI (knob consumido como taxa por-passo ⇒ resposta exponencial). Esta sonda ataca a
/// outra metade, que é visível **sem cozinhar nada**: a RÉGUA.
///
/// Três defeitos, cada um com o número que o nomeia:
/// - **FORA** — o default não cabe em `[min, max]`. O slider nasce mostrando outra coisa; é o
///   único aqui que é defeito duro, sem julgamento de gosto.
/// - **ALCANCE** — o default é positivo e `max / default` é grande: a vizinhança onde o artista
///   de fato trabalha ocupa uma fração ínfima do curso, então **nudge é impossível** e todo
///   arrasto é um salto. É a forma estrutural do que o Enio viu.
/// - **PASSO** — quantos degraus de `step` cabem no curso. Poucos ⇒ o slider é um seletor
///   grosseiro; muitos ⇒ o teclado leva uma era para atravessar.
///
/// ⚠️ **Default no ZERO não é flagado**, e é decisão: `amount = 0` é o NEUTRO de um efeito
/// (a lei `every_kind_is_born_neutral`), não um slider mal posto — a régua ali está certa e
/// quem decide se a resposta é linear é uma medição pela porta do produto, não esta tabela.
#[test]
#[ignore = "sonda de diagnostico: cargo test -p ph2d-node-registry-init --test param_census -- --ignored --nocapture"]
fn measure_where_each_default_sits_on_its_slider() {
    let reg = registry();

    let mut outside: Vec<(String, f32, f32, f32)> = Vec::new();
    let mut reach: Vec<(String, f32, f32, f32)> = Vec::new();
    let mut steps: Vec<(String, f32, f32)> = Vec::new();
    let mut counted = 0usize;

    for m in reg.manifests() {
        let hints = reg.param_ui(m.id).unwrap_or(&[]);
        for p in m.params {
            let Some(h) = hints.iter().find(|h| h.param == p.name) else {
                continue;
            };
            // Widgets sem régua contínua (chip, toggle, enum, seed) não têm curso a balancear.
            if !matches!(
                h.widget,
                ph2d_node_registry::ParamWidget::Slider
                    | ph2d_node_registry::ParamWidget::IntSlider
                    | ph2d_node_registry::ParamWidget::Angle
            ) {
                continue;
            }
            counted += 1;
            let key = format!("{}.{}", m.name, p.name);
            let span = h.max - h.min;

            if p.default < h.min || p.default > h.max {
                outside.push((key.clone(), p.default, h.min, h.max));
            }
            // ⚠️ O teto HARD tem de vir do REGISTRY, nunca de um grep: um nó pode declarar os
            // hints num módulo irmão (`params_ui.rs`), e a 1ª versão desta medição varria só
            // `src/lib.rs` — ela reportou o `motion.emitter` SEM soft/hard quando ele é o nó
            // que mais os usa, incluindo um hard MIN. Uma varredura que enumera um nome de
            // arquivo mente sobre as crates que se partem.
            let hard = reg.param_hard_max(m.id, p.name);
            if p.default > 0.0 && h.max / p.default >= 20.0 {
                reach.push((
                    format!("{key}{}", if hard.is_some() { " [hard]" } else { "" }),
                    p.default,
                    h.max,
                    h.max / p.default,
                ));
            }
            if h.step > 0.0 && span > 0.0 {
                let n = span / h.step;
                if !(4.0..=4000.0).contains(&n) {
                    steps.push((key, n, h.step));
                }
            }
        }
    }

    reach.sort_by(|a, b| b.3.total_cmp(&a.3));
    steps.sort_by(|a, b| a.1.total_cmp(&b.1));

    println!("\n{counted} params continuos (slider/int/angle) com hint\n");

    println!(
        "FORA DA REGUA (o default nao cabe em [min,max]): {}",
        outside.len()
    );
    for (k, d, lo, hi) in &outside {
        println!("  {k:<40} default {d}  range [{lo}, {hi}]");
    }

    println!(
        "\nALCANCE (max/default >= 20 -- nudge impossivel): {}",
        reach.len()
    );
    for (k, d, hi, r) in &reach {
        println!("  {k:<40} default {d:<10} max {hi:<10} = {r:>8.0}x");
    }

    println!("\nPASSO (degraus fora de 4..4000): {}", steps.len());
    for (k, n, s) in &steps {
        println!("  {k:<40} {n:>10.1} degraus de {s}");
    }
}
