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

/// **SONDA: quantos PIXELS de arrasto atravessam o campo inteiro?**
///
/// O censo acima conta *quem registou faixa*. Esta pergunta é outra, e é a do produto: um campo cuja
/// faixa inteira cabe em dois pixels é inutilizável, e o número que diz isso não é *"tem faixa?"* —
/// é `largura_do_intervalo ÷ unidades_por_pixel`.
///
/// ⚠️ **Ela nasceu porque o intervalo tinha QUATRO fontes e a taxa só consultava DUAS.** O clamp do
/// `dispatch::pointer_move` perguntava, por esta ordem: uma taxa registada (⇒ sem limites) · o
/// `number_range` · a projeção afim do **slider ligado** (`[offset, scale+offset]`) · e o `(0,1)` de
/// um chip de canal do picker. A **taxa** parava nas duas primeiras e caía no atalho histórico para
/// as outras duas — então uma caixa que ERA clampada num intervalo conhecido era arrastada a
/// `DRAG_RATE_X × step`, ignorando o intervalo que o clamp ao lado dela já sabia.
///
/// Medido em 2026-08-12, ANTES da porta única: **295** campos com intervalo conhecido, **43** a
/// atravessarem-se inteiros em **menos de 20 px** (o pior em **0,01 px**) e um a 510. Todos os
/// servidos pelo `number_range` cruzavam em **250,00** — o alvo. Hoje a lei mora em
/// [`WidgetStore::number_scrub_law`] e a coluna `fonte` diz de onde o intervalo veio.
///
/// ⚠️ **Ela pergunta ao PRODUTO, e não repete a lei.** Uma sonda com o laço próprio fica cega à
/// porta: continuaria a imprimir os números de ontem depois de a cura landar, e é exactamente a
/// armadilha que o `measure_the_fold_the_product_runs` do Painter existe para nomear.
///
/// Rode: `cargo test -p ph2d-panel-registry-init --test scrub_range_census -- --ignored --nocapture`
#[test]
#[ignore = "sonda: rode com -- --ignored --nocapture"]
fn census_of_how_many_pixels_cross_a_whole_field() {
    use ph2d_editor_core::interaction::drag::DRAG_RANGE_PX_H;

    let _ = ph2d_panel_registry_init::register_all_panels();

    // (painel, fonte-do-intervalo, lo, hi, px_para_atravessar) por campo com intervalo CONHECIDO.
    let mut rows: Vec<(String, &'static str, f64, f64, f64)> = Vec::new();
    let mut unbounded = 0usize;
    with_registry_ref(|reg| {
        for panel in reg.panels() {
            let mut store = WidgetStore::default();
            panel.populate(&mut store);
            let ids: Vec<_> = store.number_fields().map(|(id, _)| id).collect();
            for id in ids {
                // O `step` que o Down escolheria — do BUFFER que o `populate` escreveu, que é o que
                // o artista de facto encontra ao abrir o painel.
                let step = match store.get(id) {
                    Some(ph2d_editor_core::interaction::InteractiveState::NumberInput {
                        buffer,
                        ..
                    }) if buffer.contains('.') => 0.01,
                    _ => 1.0,
                };
                // A LEI DO PRODUTO, pela porta dele.
                let law = store.number_scrub_law(id, step);
                let Some((lo, hi)) = law.bounds else {
                    unbounded += 1;
                    continue;
                };
                // A fonte é só para LER a tabela — a lei já foi respondida acima.
                let source = if store.number_range(id).is_some() {
                    "range"
                } else if store.linked_slider(id).is_some() {
                    "slider"
                } else if store.blender_channel_chip(id).is_some() {
                    "channel"
                } else {
                    "?"
                };
                let px = if law.rate_x > 0.0 {
                    (hi - lo) / law.rate_x
                } else {
                    f64::NAN
                };
                rows.push((panel.manifest.id.to_string(), source, lo, hi, px));
            }
        }
    });

    rows.sort_by(|a, b| a.4.partial_cmp(&b.4).unwrap_or(std::cmp::Ordering::Equal));
    println!("[scrub-px] quantos pixels de arrasto atravessam o campo INTEIRO");
    println!(
        "[scrub-px] {:<26} {:>8} {:>22} {:>12}",
        "painel", "fonte", "intervalo", "px p/ cruzar"
    );
    for (id, source, lo, hi, px) in &rows {
        let iv = format!("[{lo:.4}, {hi:.4}]");
        println!("[scrub-px] {id:<26} {source:>8} {iv:>22} {px:>12.2}");
    }
    let bad = rows.iter().filter(|r| r.4 < 20.0).count();
    println!(
        "[scrub-px] {} campos com intervalo conhecido · {bad} cruzam em MENOS de 20 px · {unbounded} \
         sem intervalo nenhum",
        rows.len()
    );
    println!("[scrub-px] ⚠️ o alvo de desenho é {DRAG_RANGE_PX_H:.0} px (DRAG_RANGE_PX_H)");
    println!(
        "[scrub-px] ⚠️ os {unbounded} sem intervalo NAO sao todos o mesmo caso -- rode a sonda \
         `census_of_who_has_no_interval_and_why`, que separa CALIBRADO de ATALHO"
    );
}

/// **SONDA: dos campos sem intervalo, quantos foram CALIBRADOS e quantos caíram no atalho?**
///
/// ⚠️ **A contagem que a sonda irmã imprime soma duas causas de veredito OPOSTO**, e por isso
/// não pode ser o tamanho de um item. [`WidgetStore::number_scrub_interval`] devolve `None` em
/// dois mundos diferentes:
///
/// - **CALIBRADA** — alguém registou uma taxa (`set_number_drag_rate`), e o doc dela diz que é
///   a receita CERTA para uma caixa com piso e sem tecto: *o alcance dá `step` e piso ao
///   stepper, a taxa dá ao arrasto uma escala calibrada em vez de uma proporção sobre um
///   intervalo que não termina*. A roldana da física, os chips do transporte da timeline e os
///   de transform do Vector já vivem assim. Aqui **não há trabalho**: o número tem dono.
/// - **ATALHO** — ninguém registou nada, e o arrasto cai em `DRAG_RATE_X · step`, com o `step`
///   a sair do BUFFER (`1.0` sem casa decimal, `0.01` com). São **50 ou 0,5 unidades por
///   pixel**, e nenhum dos dois foi medido por alguém.
///
/// Só a segunda linha é pergunta em aberto. E ela reparte outra vez pelo `step`, porque a
/// severidade é dele e não do campo: a **GROSSA** move o valor 500 em dez pixels de arrasto,
/// que é o defeito que a faixa foi criada para curar em 2026-06-25; a **FINA** move 5.
///
/// ⚠️ **O `step` do atalho não é propriedade do campo** — ele é lido do texto que a caixa
/// calha de mostrar, então o MESMO campo troca de coluna no dia em que o valor renderiza sem
/// casa decimal. O censo mede o que o artista encontra **ao abrir o painel**, que é o que o
/// `populate` escreveu.
///
/// Rode: `cargo test -p ph2d-panel-registry-init --test scrub_range_census -- --ignored --nocapture`
#[test]
#[ignore = "sonda: rode com -- --ignored --nocapture"]
fn census_of_who_has_no_interval_and_why() {
    use ph2d_editor_core::interaction::drag::DRAG_RATE_X;

    let _ = ph2d_panel_registry_init::register_all_panels();

    // (painel, calibrados, atalho GROSSO (step 1), atalho FINO (step 0,01))
    let mut rows: Vec<(String, usize, usize, usize)> = Vec::new();
    with_registry_ref(|reg| {
        for panel in reg.panels() {
            let mut store = WidgetStore::default();
            panel.populate(&mut store);
            let ids: Vec<_> = store.number_fields().map(|(id, _)| id).collect();
            let (mut calib, mut coarse, mut fine) = (0usize, 0usize, 0usize);
            for id in ids {
                if store.number_scrub_interval(id).is_some() {
                    continue; // tem intervalo: assunto da sonda irmã
                }
                if store.number_drag_rate(id).is_some() {
                    calib += 1;
                    continue;
                }
                let decimal = matches!(
                    store.get(id),
                    Some(ph2d_editor_core::interaction::InteractiveState::NumberInput {
                        buffer,
                        ..
                    }) if buffer.contains('.')
                );
                if decimal {
                    fine += 1;
                } else {
                    coarse += 1;
                }
            }
            if calib + coarse + fine > 0 {
                rows.push((panel.manifest.id.to_string(), calib, coarse, fine));
            }
        }
    });

    rows.sort_by(|a, b| (b.2 + b.3).cmp(&(a.2 + a.3)).then(a.0.cmp(&b.0)));
    let calib: usize = rows.iter().map(|r| r.1).sum();
    let coarse: usize = rows.iter().map(|r| r.2).sum();
    let fine: usize = rows.iter().map(|r| r.3).sum();

    println!("[scrub-why] campos SEM intervalo, por causa");
    println!(
        "[scrub-why] {:<28} {:>11} {:>10} {:>9}",
        "painel", "calibrado", "atalho 50", "atalho .5"
    );
    for (id, c, g, f) in &rows {
        println!("[scrub-why] {id:<28} {c:>11} {g:>10} {f:>9}");
    }
    println!(
        "[scrub-why] {:<28} {calib:>11} {coarse:>10} {fine:>9}",
        "TOTAL"
    );
    println!(
        "[scrub-why] {} sem intervalo = {calib} CALIBRADOS (taxa registada, nada a fazer) + \
         {} no ATALHO",
        calib + coarse + fine,
        coarse + fine
    );
    println!(
        "[scrub-why] o atalho vale DRAG_RATE_X*step = {:.0} unidades/px (grosso) ou {:.1} (fino)",
        DRAG_RATE_X,
        DRAG_RATE_X * 0.01
    );
    println!(
        "[scrub-why] ⚠️ o `step` sai do BUFFER, nao do campo: o mesmo campo muda de coluna no dia \
         em que o valor renderizar sem casa decimal"
    );

    // A RÉGUA. `50 unidades/px` só é indefensável contra alguma coisa, e a coisa é o que este
    // app JÁ escolheu em toda caixa cuja taxa alguém autorou — as com alcance (a taxa é
    // `largura/250`) e as cinco calibradas. Nenhum destes números é meu.
    let mut authored: Vec<f64> = Vec::new();
    with_registry_ref(|reg| {
        for panel in reg.panels() {
            let mut store = WidgetStore::default();
            panel.populate(&mut store);
            let ids: Vec<_> = store.number_fields().map(|(id, _)| id).collect();
            for id in ids {
                let has_interval = store.number_scrub_interval(id).is_some();
                if !has_interval && store.number_drag_rate(id).is_none() {
                    continue; // o atalho não é autoria: é o que sobra
                }
                let step = match store.get(id) {
                    Some(ph2d_editor_core::interaction::InteractiveState::NumberInput {
                        buffer,
                        ..
                    }) if buffer.contains('.') => 0.01,
                    _ => 1.0,
                };
                authored.push(store.number_scrub_law(id, step).rate_x);
            }
        }
    });
    authored.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if let (Some(lo), Some(hi)) = (authored.first(), authored.last()) {
        let mid = authored[authored.len() / 2];
        println!(
            "[scrub-why] REGUA: {} taxas AUTORADAS neste app -- min {lo:.4} / mediana {mid:.4} / \
             max {hi:.4} unidades por pixel",
            authored.len()
        );
        println!(
            "[scrub-why] o atalho grosso ({:.0}/px) e' {:.0}x a mediana autorada e {:.1}x a MAIOR \
             taxa que alguem escolheu",
            DRAG_RATE_X,
            DRAG_RATE_X / mid,
            DRAG_RATE_X / hi
        );
    }
}
