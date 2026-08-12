//! **SONDA (`--ignored`): quantos campos numéricos sabem a própria faixa de scrub?**
//!
//! ## Por que esta sonda existe, e o que ela corrige numa afirmação minha
//!
//! O plano da UI viva (§4) prometia construir o *scrub numérico* como se ele não existisse. **Ele
//! existe, completo, desde a M14.A** — `NUMBER_INPUT_DRAG_THRESHOLD_PX = 4.0` (o mesmo 4 que o
//! plano propunha «a medir no smoke»), o `crossed_threshold` que resolve a ambiguidade
//! *caret contra scrub* no Down, o bloqueio de eixo, e o `DRAG_SHIFT_MUL` na tecla que o plano
//! propunha para precisão. Escrever a wave sem primeiro fazer `git grep` teria sido reconstruir
//! por cima de código shipado.
//!
//! O que **não** existe em toda a parte é a metade que o plano acertou pelo motivo certo: a
//! sensibilidade tem de sair da **FAIXA do campo**. Quem regista `set_number_range` arrasta a
//! `[min,max]` inteira em `DRAG_RANGE_PX_H = 250 px`; quem não regista cai no atalho histórico
//! **`DRAG_RATE_X = 50` unidades de passo por pixel** — dez pixels de arrasto movem o valor 500, que
//! é literalmente o defeito que a faixa foi criada para curar em 2026-06-25 (*«um campo ±1 deixa de
//! disparar para lá de 100 em meia dúzia de pixels»*).
//!
//! ⚠️ **A pergunta é de COBERTURA, e por isso é medida e não estimada.** Um `git grep` conta
//! *chamadas escritas*, não *campos vivos*: a maioria nasce dentro de laços e de helpers
//! partilhados, então o numerador e o denominador do grep são os dois falsos. Esta sonda popula
//! cada painel REGISTADO e pergunta ao store — a mesma disciplina do `param_census` dos nós, e
//! pela mesma razão (esta é a crate mais barata que enxerga todos os painéis).
//!
//! ## ⚠️ O que a coluna `na constante` **não** prova, e como quase me enganou
//!
//! Ela é um **limite superior**, nunca um veredito. O `populate` é o piso da população, e um painel
//! pode registar a faixa no **paint** — que é o que o `motion_params` faz (a linha depende do nó
//! seleccionado, então a faixa só é conhecida ao desenhar). O censo mede-o em `0 / 32` e o crate
//! **tem** `set_number_range`: a leitura ingénua desses 32 seria trabalho inventado.
//!
//! A leitura honesta corre em dois passos: (1) esta sonda dá a lista de suspeitos; (2) para cada
//! painel dela, um `git grep set_number_range -- <crate>` separa *nunca regista* de *regista ao
//! pintar*. Medido em 2026-08-12, os **certos** — zero registos no crate inteiro — eram
//! `grid_snap` (31) · `color_equalization` (15) · `bgremoval` (6) · `padding` (4) ·
//! `equalize_sizes` (3) · `upscale` (1). Os outros quatro painéis da lista registam algures e não
//! podem ser julgados por este instrumento.
//!
//! Rode: `cargo test -p ph2d-panel-registry-init --test scrub_range_census -- --ignored --nocapture`

use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::with_registry_ref;

#[test]
#[ignore = "sonda: rode com -- --ignored --nocapture"]
fn census_of_number_fields_that_know_their_own_range() {
    let _ = ph2d_panel_registry_init::register_all_panels();

    let mut rows: Vec<(String, usize, usize)> = Vec::new();
    with_registry_ref(|reg| {
        for panel in reg.panels() {
            // ⚠️ Um store NOVO por painel: partilhado, o primeiro a registar um id ficaria com ele
            // e o censo creditaria os campos ao painel errado.
            let mut store = WidgetStore::default();
            panel.populate(&mut store);
            let (mut with_range, mut without) = (0usize, 0usize);
            for (_, range) in store.number_fields() {
                if range.is_some() {
                    with_range += 1;
                } else {
                    without += 1;
                }
            }
            if with_range + without > 0 {
                rows.push((panel.manifest.id.to_string(), with_range, without));
            }
        }
    });

    rows.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
    let total_with: usize = rows.iter().map(|r| r.1).sum();
    let total_without: usize = rows.iter().map(|r| r.2).sum();

    println!("[scrub-census] campos numéricos por painel, no `populate`");
    println!(
        "[scrub-census] {:<28} {:>10} {:>12}",
        "painel", "com faixa", "na constante"
    );
    for (id, with_range, without) in &rows {
        println!("[scrub-census] {id:<28} {with_range:>10} {without:>12}");
    }
    println!(
        "[scrub-census] {:<28} {total_with:>10} {total_without:>12}",
        "TOTAL"
    );
    let total = total_with + total_without;
    if total > 0 {
        let pct = total_with as f64 / total as f64 * 100.0;
        println!("[scrub-census] cobertura: {pct:.1}% ({total_with} de {total})");
    }
    println!(
        "[scrub-census] ⚠️ `na constante` = arrasta a DRAG_RATE_X (50 unidades de passo por pixel), \
         não à faixa do campo"
    );
    println!(
        "[scrub-census] ⚠️ o `populate` é o PISO: campos registados só no `paint` (ou por seleção \
         viva) não aparecem aqui"
    );
}
