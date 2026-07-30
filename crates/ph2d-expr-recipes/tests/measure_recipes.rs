//! **A régua de uma receita** — o instrumento do §7.1 do plano 12, e o único caminho para
//! um grupo da FASE E: *medir ANTES da UI*.
//!
//! ⚠️ **A medida é por KIND, e isso é a correção da auditoria ao §2.2** — usar a mesma para
//! os três é como *"Shake: mudar os parâmetros não mudava a animação"* passa despercebido:
//!
//! | kind | a pergunta | como se mede |
//! |---|---|---|
//! | **SOURCE** (`combine.is_some()`) | *ela ANIMA?* | excursão no tempo, com base **0 e 1** |
//! | **MODIFICADOR** (`combine.is_none()`) | *ela muda o valor que ENTRA?* | delta contra a identidade sobre uma grade `(tempo, valor)` — **nunca** a amplitude do stack |
//! | **TIME** | *ela retima?* | só mensurável **com uma linha embaixo** (ver `inert_reason`) |
//!
//! ⚠️ **`__seed` aqui é um número FIXO e não-zero.** Esta crate é leaf e não conhece a
//! timeline; o que importa para a régua é que ele NÃO seja zero (a auditoria mediu a fita
//! do card desenhando com `0` enquanto a cena usava `target * 100` — medir com zero é medir
//! o instrumento). O seed real vem de `ph2d_timeline::seed_of_target`, gateado lá.
//!
//! ⚠️⚠️ **A PRIMEIRA coisa que esta régua mediu foi ela mesma, e ela estava errada de duas
//! maneiras — as duas previstas pelo §7.1.**
//!
//! 1. **A EXCURSÃO não responde por um knob que não é amplitude.** Varrendo `Speed` do Shake
//!    a saída ficava em ~0,55 de 5 a 20 (o pico-a-pico de um ruído não cresce com a
//!    frequência) e o `Phase` do Sway variava **0,0059** — os dois seriam declarados quase
//!    mortos por uma régua que só olha o quanto o objeto anda. Um knob de RITMO muda a
//!    taxa; um de FASE muda o alinhamento. A pergunta *"este knob está vivo?"* é sobre o
//!    **SINAL**: `max |saída_a(t) − saída_b(t)|`. Excursão e taxa continuam impressas,
//!    porque elas dizem **o quê** mudou.
//! 2. **A fixture não continha o fenômeno.** O `Roughness` do Shake mediu **0,0000** —
//!    "morto" — e não está: ele é a queda de amplitude ENTRE oitavas, e o default tem
//!    `Detail = 1`, ou seja **uma** oitava. Um knob que só age através de outro parece morto
//!    numa fixture que deixa o outro no default. A régua varre DUAS vezes: com os demais no
//!    default e com os demais no MEIO da faixa, e só chama de morto o que morre nas duas.
//!
//! ⚠️ E o orçamento do default (*"tirou o objeto do quadro?"*) é **SOURCE-only**: um
//! modificador só puxa o valor para DENTRO, então medi-lo contra esse teto é medir a grade
//! do experimento. O `Limit` default marcava 2377% do orçamento — o número é honesto e a
//! pergunta é que não era dele.
//!
//! Rodar: `cargo test -p ph2d-expr-recipes --test measure_recipes -- --nocapture`

use ph2d_expr::{Bindings, eval};
use ph2d_expr_parse::parse;
use ph2d_expr_recipes::{KnobValue, RecipeStack, Row, by_id};

/// O canvas do app, em metros de mundo — a régua de *"tirou o objeto do quadro?"*.
const CANVAS_M: f32 = 40.96;
/// O teto do §7.1 para o DEFAULT: um cinquentavo do canvas.
const DEFAULT_BUDGET_M: f32 = CANVAS_M / 50.0;
/// A janela de tempo que a fita do card desenha.
const WINDOW_S: f32 = 2.0;
const SAMPLES: usize = 240;
/// Os seeds varridos. ⚠️ **UM seed é UMA amostra, e isso mordeu**: a excursão de
/// `wiggle(2, amount)` medida com o seed 100 dá `1,048 × amount`, e varrendo quarenta
/// objetos ela vai de **0,49× a 1,96×** — quatro vezes de espalhamento. Uma faixa derivada
/// de um seed é uma faixa que vale para um objeto.
const SEEDS: [f32; 8] = [0.0, 100.0, 300.0, 700.0, 1100.0, 1900.0, 2600.0, 3300.0];

struct B {
    time: f32,
    value: f32,
    seed: f32,
}
impl Bindings for B {
    fn attr(&self, name: &str) -> f32 {
        match name {
            "time" => self.time,
            "value" => self.value,
            "__seed" => self.seed,
            _ => 0.0,
        }
    }
    fn param(&self, _: &str) -> f32 {
        0.0
    }
}

/// O SINAL que uma linha produz na janela, amostrado, para um seed.
fn signal_seeded(row: &Row, base: f32, seed: f32) -> Vec<f32> {
    let mut stack = RecipeStack::new();
    stack.push(row.clone());
    let e = parse(&stack.to_formula()).expect("o catálogo emite texto que parseia");
    (0..SAMPLES)
        .map(|i| {
            let t = i as f32 / SAMPLES as f32 * WINDOW_S;
            eval(
                &e,
                &B {
                    time: t,
                    value: base,
                    seed,
                },
            )
        })
        .collect()
}

/// O sinal no seed de referência — o que basta para tudo que não lê ruído.
fn signal(row: &Row, base: f32) -> Vec<f32> {
    signal_seeded(row, base, SEEDS[1])
}

/// A TAXA: quanto o sinal se move de amostra para amostra, em média. É ela que responde por
/// um knob de velocidade — a excursão não muda quando o ruído fica mais rápido.
fn rate(row: &Row, base: f32) -> f32 {
    let s = signal(row, base);
    if s.len() < 2 {
        return 0.0;
    }
    let sum: f32 = s.windows(2).map(|w| (w[1] - w[0]).abs()).sum();
    sum / (s.len() - 1) as f32
}

/// A maior diferença PONTO A PONTO entre dois sinais — a pergunta *"este knob está vivo?"*,
/// e a única que serve para amplitude, ritmo E fase de uma vez.
fn signal_diff(a: &Row, b: &Row, base: f32) -> f32 {
    signal(a, base)
        .into_iter()
        .zip(signal(b, base))
        .map(|(x, y)| (x - y).abs())
        .fold(0.0, f32::max)
}

/// A EXCURSÃO de uma fonte: quanto ela move o objeto na janela, no PIOR caso — as duas
/// bases e os oito seeds. ⚠️ O pior seed, não um seed: ver [`SEEDS`].
fn source_excursion(row: &Row) -> f32 {
    let mut worst = 0.0_f32;
    for base in [0.0_f32, 1.0] {
        for seed in SEEDS {
            let s = signal_seeded(row, base, seed);
            let (lo, hi) = s
                .iter()
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), v| {
                    (a.min(*v), b.max(*v))
                });
            worst = worst.max(hi - lo);
        }
    }
    worst
}

/// O DELTA de um modificador: o quanto ele muda o valor que entra, sobre uma grade
/// `(tempo, valor)`. ⚠️ Nunca a amplitude do stack — um modificador sobre um stack
/// constante mede zero por construção.
fn modifier_delta(row: &Row) -> f32 {
    let mut stack = RecipeStack::new();
    stack.push(row.clone());
    let e = parse(&stack.to_formula()).expect("parseia");
    let mut worst = 0.0_f32;
    for i in 0..24 {
        let t = i as f32 / 24.0 * WINDOW_S;
        for j in 0..41 {
            // A grade de VALOR cobre o canvas: um `Limit` de ±1 é invisível numa grade
            // que só passa por [-1, 1].
            let value = -CANVAS_M / 2.0 + j as f32 * CANVAS_M / 40.0;
            let out = eval(
                &e,
                &B {
                    time: t,
                    value,
                    seed: SEEDS[1],
                },
            );
            worst = worst.max((out - value).abs());
        }
    }
    worst
}

/// Uma linha da receita `id` com o knob `ki` em `v`; os DEMAIS no default ou no MEIO da
/// faixa deles (`others_mid`) — ver a lição 2 do doc do módulo.
fn row_with(id: &'static str, ki: usize, v: f32, others_mid: bool) -> Row {
    let rec = by_id(id).expect("a receita existe");
    let mut row = Row::new(id).expect("existe");
    if others_mid {
        for (i, k) in rec.knobs.iter().enumerate() {
            if i != ki && matches!(row.knobs.get(i), Some(KnobValue::Num(_))) {
                row.set(k.key, KnobValue::Num((k.range.0 + k.range.1) * 0.5));
            }
        }
    }
    row.set(rec.knobs[ki].key, KnobValue::Num(v));
    row
}

/// A VIDA de um knob: a maior diferença de sinal que a varredura dele produz — medida com
/// os demais no default E com os demais no meio. Morto = morre nas duas.
fn knob_liveness(id: &'static str, ki: usize) -> (f32, f32) {
    let k = by_id(id).expect("existe").knobs[ki];
    let at = |s: usize| k.range.0 + (k.range.1 - k.range.0) * s as f32 / 4.0;
    let mut best = (0.0_f32, 0.0_f32);
    for base in [0.0_f32, 1.0] {
        for s in 1..5 {
            best.0 = best.0.max(signal_diff(
                &row_with(id, ki, at(0), false),
                &row_with(id, ki, at(s), false),
                base,
            ));
            best.1 = best.1.max(signal_diff(
                &row_with(id, ki, at(0), true),
                &row_with(id, ki, at(s), true),
                base,
            ));
        }
    }
    best
}

/// Como a EXCURSÃO (ou o delta, num modificador) e a TAXA respondem à varredura — o *o quê*
/// mudou, ao lado do *se* mudou.
fn knob_sensitivity(id: &'static str, ki: usize) -> Vec<(f32, f32, f32)> {
    let rec = by_id(id).expect("a receita existe");
    let source = rec.combine.is_some();
    (0..5)
        .map(|s| {
            let k = rec.knobs[ki];
            let v = k.range.0 + (k.range.1 - k.range.0) * s as f32 / 4.0;
            let row = row_with(id, ki, v, false);
            let out = if source {
                source_excursion(&row)
            } else {
                modifier_delta(&row)
            };
            (v, out, rate(&row, 0.0))
        })
        .collect()
}

/// **A régua completa de uma receita**, impressa.
fn measure(id: &'static str) {
    let rec = by_id(id).expect("a receita existe");
    let source = rec.combine.is_some();
    let row = Row::new(id).expect("existe");

    let at_default = if source {
        source_excursion(&row)
    } else {
        modifier_delta(&row)
    };
    println!(
        "\n=== {} ({}, {})",
        rec.label,
        id,
        if source { "SOURCE" } else { "MODIFICADOR" }
    );
    println!(
        "    fórmula default: {}",
        RecipeStack::of(&[id]).to_formula()
    );
    if source {
        println!(
            "    no DEFAULT: {at_default:.4} m  ({:.2}% do orçamento de {DEFAULT_BUDGET_M:.4} m, \
             {:.4} canvas){}",
            at_default / DEFAULT_BUDGET_M * 100.0,
            at_default / CANVAS_M,
            if at_default > DEFAULT_BUDGET_M {
                "  ⚠️ ACIMA DO ORÇAMENTO"
            } else {
                ""
            }
        );
    } else {
        // ⚠️ Um modificador só puxa o valor para DENTRO: o orçamento de *"tirou o objeto do
        // quadro?"* não é pergunta dele. O que se reporta é o alcance da correção.
        println!("    no DEFAULT: corrige até {at_default:.4} m (modificador: nunca EXPULSA)");
    }

    for (ki, k) in rec.knobs.iter().enumerate() {
        if !matches!(row.knobs.get(ki), Some(KnobValue::Num(_))) {
            continue; // link/texto: outra régua (a FASE D)
        }
        let (live_def, live_mid) = knob_liveness(id, ki);
        let seen = knob_sensitivity(id, ki);
        let top = seen.last().map_or(0.0, |(_, o, _)| *o);
        let verdict = match (live_def > 1e-6, live_mid > 1e-6) {
            (false, false) => "   ⚠️ MORTO nas DUAS varreduras",
            (false, true) => "   (só age através de outro knob)",
            _ => "",
        };
        println!(
            "    knob {:<10} faixa [{}, {}] | VIVO: default {live_def:.4} · meio \
             {live_mid:.4}{verdict}",
            k.label, k.range.0, k.range.1,
        );
        if source {
            println!(
                "        no TOPO: {top:.4} m = {:.2} canvas{}",
                top / CANVAS_M,
                if top > CANVAS_M {
                    "  ⚠️ A FAIXA EXPULSA O OBJETO"
                } else {
                    ""
                }
            );
        }
        for (v, o, r) in &seen {
            println!("        {v:>8.3} -> excursão {o:.4}  taxa {r:.5}");
        }
    }
}

/// **O censo do catálogo inteiro**: que knob de que receita EXPULSA o objeto no topo da
/// faixa. A auditoria contou "dez combinações"; este é o número de hoje, por nome.
#[test]
fn census_of_ranges_that_eject_the_object() {
    println!("\n### CENSO — knobs cujo TOPO passa de um canvas ({CANVAS_M} m)");
    let mut n = 0;
    for rec in ph2d_expr_recipes::CATALOG {
        if rec.combine.is_none() {
            continue; // modificador não expulsa
        }
        let row = Row::new(rec.id).expect("existe");
        for (ki, k) in rec.knobs.iter().enumerate() {
            if !matches!(row.knobs.get(ki), Some(KnobValue::Num(_))) {
                continue;
            }
            let top = source_excursion(&row_with(rec.id, ki, k.range.1, false));
            if top > CANVAS_M {
                n += 1;
                println!(
                    "  {:<14} {:<10} topo {} -> {top:.2} m = {:.2} canvas",
                    rec.id,
                    k.label,
                    k.range.1,
                    top / CANVAS_M
                );
            }
        }
    }
    println!("  TOTAL: {n}");
}

#[test]
fn measure_group_one() {
    println!("\n### GRUPO 1 — Shake · Sway · Limit (§7.2 do plano 12)");
    println!(
        "canvas {CANVAS_M} m | janela {WINDOW_S} s | orçamento do default {DEFAULT_BUDGET_M:.4} m"
    );
    for id in ["shake", "sway", "limit"] {
        measure(id);
    }
}
