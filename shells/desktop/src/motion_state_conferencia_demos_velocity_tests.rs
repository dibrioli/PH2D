//! Gates da cena `=47` — **PARA ONDE ISTO VAI**.
//!
//! Eles medem a geometria que a cena COZINHA, não a intenção com que foi escrita:
//! se a mensagem diz *"a de baixo incha onde acelera"*, o cozido tem de o mostrar.
//!
//! ⚠️ **Toda medição AVANÇA O TICK.** Sem `advance_tick` a aresta `pre` nunca
//! carrega estado: o `motion.velocity` nunca tem um ontem, toda velocidade é zero e
//! as seis fileiras saem IDÊNTICAS — a fixture ficaria verde sobre uma cena morta,
//! e pior, sobre exatamente o modo de falha que a wave introduz. É a mesma lição
//! que as cenas `=38` e `=46` pagaram.
//!
//! ⚠️ **E a régua é a SILHUETA (o `size`) ou o `rot`, nunca o Y cru** — cada fileira
//! carrega o próprio `offset_y`, e comparar Y entre bandas mede o `BAND_GAP`. É a
//! quarta vez que este repo paga essa lição.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// 60 ticks = um segundo — o relógio que o app corre.
const DT: f64 = 1.0 / 60.0;

/// Cozinha a cena por `ticks` ticks e devolve, por banda, a coluna pedida em cada
/// tick (uma linha por tick, um valor por peça).
fn trace(ticks: usize, col: &str, lane: usize) -> Vec<Vec<f32>> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut doc = MotionDoc::default();
    let sinks = build_velocity_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), BANDS, "uma fileira por banda");

    let mut cook = Cook::new();
    let mut out = Vec::with_capacity(ticks);
    for t in 0..ticks {
        #[expect(clippy::cast_precision_loss, reason = "poucos ticks")]
        let now = t as f64 * DT;
        let s = cook.cook(&doc.graph, &reg, sinks[lane], now).expect("cook")[0]
            .as_stream()
            .clone();
        // ⚠️ Uma coluna AUSENTE é zeros no comprimento cheio, nunca um pânico: é a
        // identidade que a lowering assume, e é literalmente o estado do controle
        // da banda 3 — um stream que ninguém rodou não carrega `rot`, e exigi-la
        // faria o gate reprovar o controle por ele estar CERTO.
        let row = match s.get(col) {
            Some(Column::Vec2(v)) => v.iter().map(|p| p[0]).collect(),
            Some(Column::Scalar(v)) => v.clone(),
            _ => vec![0.0; s.count()],
        };
        out.push(row);
        cook.advance_tick(&doc.graph, &reg, now).expect("tick");
    }
    out
}

/// O maior e o menor valor de uma linha.
fn span(row: &[f32]) -> (f32, f32) {
    row.iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), v| (lo.min(*v), hi.max(*v)))
}

/// **A VELOCIDADE ENGORDA A PEÇA, E O CONTROLE PROVA QUE ELA E' A CAUSA.**
///
/// ⚠️ *"A fileira tem tamanhos diferentes"* não seria oráculo nenhum — qualquer
/// campo por-índice a produziria. A pergunta é se a de cima, **com o mesmo
/// percurso**, é UNIFORME: é ela que mostra que o canal `Speed` devolvia zeros e
/// que o nó novo é o que os preenche.
#[test]
fn the_speed_swells_the_piece_and_the_control_stays_uniform() {
    let last = 90;
    let control = trace(last, "size", 0).pop().expect("ticks");
    let driven = trace(last, "size", 1).pop().expect("ticks");

    let (clo, chi) = span(&control);
    assert!(
        (chi - clo).abs() < 1e-4,
        "CONTROLE: sem o `motion.velocity` a fileira tem UM tamanho so'; medido [{clo}, {chi}]"
    );
    let (lo, hi) = span(&driven);
    assert!(
        hi - lo > clo * 0.5,
        "a fileira dirigida tem de variar de forma visivel: [{lo}, {hi}] contra um repouso de {clo}"
    );
    assert!(
        lo >= clo - 1e-4,
        "e' um `Add` sobre o tamanho de repouso: nada pode ficar MENOR que {clo}; veio {lo}"
    );
}

/// **A PEÇA INCHA QUANDO ACELERA — ao longo do TEMPO, não só ao longo da fileira.**
///
/// ⚠️ Este é o gate que uma foto não consegue fazer. A peça 0 percorre um círculo,
/// então a velocidade dela é constante em MAGNITUDE — e é por isso que o oráculo
/// não é *"o tamanho dela oscila"* e sim que ela **CRESCE do repouso** e depois
/// fica: o primeiro tick não tem ontem (tamanho de repouso) e os seguintes têm.
#[test]
fn the_first_tick_has_no_yesterday_and_the_next_ones_have() {
    let tr = trace(30, "size", 1);
    let first = tr[0][0];
    let later = tr[20][0];
    let rest = trace(30, "size", 0)[0][0];
    assert!(
        (first - rest).abs() < 1e-4,
        "no 1o tick nao ha ontem, entao a peca tem o tamanho de repouso {rest}; veio {first}"
    );
    assert!(
        later > first * 1.2,
        "e do 2o tick em diante ela carrega a velocidade: {first} -> {later}"
    );
}

/// **O TRAÇO APONTA PARA ONDE VAI** — o *align to velocity*.
///
/// ⚠️ O oráculo não é *"tem `rot`"*: zero é um ângulo válido e o controle o tem.
/// A pergunta é se os ângulos **VARREM o círculo** — numa órbita a tangente passa
/// por todas as direções, e é isso que separa *alinhado* de *rodado por um número*.
#[test]
fn the_trace_points_where_it_is_going_and_the_control_does_not() {
    let last = 60;
    let control = trace(last, "rot", 2).pop().expect("ticks");
    let aimed = trace(last, "rot", 3).pop().expect("ticks");

    let (clo, chi) = span(&control);
    assert!(
        (chi - clo).abs() < 1e-4,
        "CONTROLE: sem alinhamento todos os tracos ficam no mesmo angulo; medido [{clo}, {chi}]"
    );
    let (lo, hi) = span(&aimed);
    assert!(
        hi - lo > 180.0,
        "a tangente de uma orbita varre o circulo; a faixa medida e' [{lo}, {hi}]"
    );
}

/// **O `smooth` ACALMA O TREMOR.**
///
/// ⚠️ A régua é a variação de um tick para o SEGUINTE na MESMA peça — é isso que
/// *"pisca"* significa. Uma faixa (máximo menos mínimo) não serve: as duas fileiras
/// visitam os mesmos extremos, e a diferença está em quão depressa elas os cruzam.
#[test]
fn the_smooth_calms_the_jitter_tick_to_tick() {
    let raw = trace(120, "size", 4);
    let smoothed = trace(120, "size", 5);
    // A agitação: a média, sobre a fileira e sobre o tempo, do salto entre ticks.
    let churn = |tr: &[Vec<f32>]| {
        let mut sum = 0.0f32;
        let mut n = 0usize;
        for (a, b) in tr.iter().zip(tr.iter().skip(1)) {
            for (x, y) in a.iter().zip(b) {
                sum += (y - x).abs();
                n += 1;
            }
        }
        #[expect(clippy::cast_precision_loss, reason = "n e' pequeno")]
        let d = n as f32;
        sum / d
    };
    let (a, b) = (churn(&raw), churn(&smoothed));
    assert!(
        b < a * 0.5,
        "o one-pole tem de cortar a agitacao para menos de metade: crua {a:.5}, suavizada {b:.5}"
    );
    assert!(
        a > 1e-4,
        "CONTROLE: a fileira crua tem de TREMER de facto -- se ela ja' e' lisa, a fixture nao \
         contem o fenomeno e o par nao diz nada. Medido {a:.5}"
    );
}

/// **A CENA MONTA AS SEIS BANDAS, E A LISTA DO LOG DESCREVE-AS.**
///
/// ⚠️ A contagem é DERIVADA (`BAND_LABELS.len()`), nunca um literal: um número
/// escrito à mão só sabe dizer *"mudou"*, e a pergunta é se a lista que o artista
/// lê corresponde ao que a cena de facto empilha.
#[test]
fn the_scene_builds_every_band_the_log_names() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut doc = MotionDoc::default();
    let sinks = build_velocity_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), BAND_LABELS.len());
    assert_eq!(sinks.len(), BANDS);
}

/// **SONDA — os números que a mensagem do smoke cita.**
///
/// Ela não afirma nada: imprime o que as seis bandas de facto cozinham, para a
/// mensagem do roteador ser escrita a partir de uma MEDIÇÃO e não de uma
/// expectativa. Rodar:
/// `cargo test -p ph2d-host-desktop --release conferencia_demos_velocity -- --ignored --nocapture`
#[test]
#[ignore = "sonda: mede, nao afirma"]
fn measure_what_the_six_bands_draw() {
    println!("\n== a cena =47, medida ==");
    for (b, label) in BAND_LABELS.iter().enumerate() {
        let col = if b == 2 || b == 3 { "rot" } else { "size" };
        let tr = trace(120, col, b);
        let last = tr.last().expect("ticks");
        let (lo, hi) = span(last);
        // A agitação tick-a-tick da MESMA peça — a régua do par 5-6.
        let mut sum = 0.0f32;
        let mut n = 0usize;
        for (a, c) in tr.iter().zip(tr.iter().skip(1)) {
            for (x, y) in a.iter().zip(c) {
                sum += (y - x).abs();
                n += 1;
            }
        }
        #[expect(clippy::cast_precision_loss, reason = "n e' pequeno")]
        let churn = sum / n as f32;
        println!("  {label}\n      {col}: [{lo:.4} .. {hi:.4}]  agitacao/tick {churn:.5}");
    }
}
