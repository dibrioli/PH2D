//! **GRUPO 1 da FASE E — Shake · Sway · Limit** (§7.2 do plano 12).
//!
//! As três cobrem os **três kinds** do modelo (fonte-orgânica, fonte-rítmica, modificador),
//! e é por isso que o G1 valida a arquitetura inteira com três receitas.
//!
//! **A frase do animador** (§2.1 — se ela não sai, a receita não entra):
//!
//! * **Shake** — *"treme como uma câmera na mão"*.
//! * **Sway** — *"balança de um lado para o outro, no mesmo ritmo"*.
//! * **Limit** — *"não deixa passar destes dois valores"*.
//!
//! ## O que a medição achou (a régua é `measure_recipes.rs`)
//!
//! ⚠️ **O defeito era a FAIXA, não o default** — e ele tinha nome: `CANVAS_M` (o teto de um
//! VALOR) estava sendo usado como teto de uma **AMPLITUDE**.
//!
//! | | topo antes | excursão | depois |
//! |---|---|---|---|
//! | `Sway.amount` | 40 | **80,00 m = 1,95 canvas** | 20 → 40,0 m |
//! | `Shake.amount` | 40 | **64,18 m = 1,57 canvas** (pior seed) | 20 → 32,1 m |
//!
//! ⚠️ E a primeira medição do Shake **subestimou em 53%**: com um único seed ela dizia
//! 41,91 m, e varrendo quarenta objetos o fator vai de **0,49× a 1,96×** o `amount`. *Um
//! seed é uma amostra.* O teto novo (`AMPLITUDE_M`) sai das duas derivações independentes.
//!
//! ⛔ **Uma hipótese MINHA, medida e REFUTADA — não a refaça:** *"os dois `Speed` estão em
//! unidades diferentes (Sway em rad/s, Shake em Hz), então o número não transfere"*. Medido
//! por cruzamentos de zero: `Speed = 3` dá **0,50 Hz** no Sway e **0,60 Hz** no Shake — 20%,
//! não uma categoria. O modelo NÃO foi reescrito por causa disso, e o report *"a velocidade
//! em shake nunca foi velocidade, parece mais com um seed"* continua sem causa medida.
//!
//! ⚠️ **`Roughness` mede ZERO no default, e não está morto:** ele é a queda de amplitude
//! ENTRE oitavas, e o default tem `Detail = 1` — uma oitava. Com os demais no meio da faixa
//! ele move **19,44 m**. A régua varre as duas vezes por isso.

use ph2d_expr::{Bindings, eval};
use ph2d_expr_parse::parse;
use ph2d_expr_recipes::{CANVAS_M, KnobValue, RecipeStack, Row, by_id};

const WINDOW_S: f32 = 2.0;
const SAMPLES: usize = 240;
/// O pior seed manda — ver o doc do módulo.
const SEEDS: [f32; 6] = [0.0, 100.0, 700.0, 1900.0, 2600.0, 3300.0];

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

fn samples(src: &str, base: f32, seed: f32) -> Vec<f32> {
    let e = parse(src).expect("o catálogo emite texto que parseia");
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

/// A excursão de uma FONTE no pior caso (duas bases × os seeds).
fn excursion(row: &Row) -> f32 {
    let mut stack = RecipeStack::new();
    stack.push(row.clone());
    let src = stack.to_formula();
    let mut worst = 0.0_f32;
    for base in [0.0_f32, 1.0] {
        for seed in SEEDS {
            let s = samples(&src, base, seed);
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

fn row_at(id: &'static str, key: &str, v: f32) -> Row {
    let mut row = Row::new(id).expect("a receita existe");
    row.set(key, KnobValue::Num(v));
    row
}

/// **O neutro é a IDENTIDADE, ao bit.**
///
/// Shake e Sway declaram `Neutrality::Additive` — o ajuste que as torna `value`. Um neutro
/// que erra por um épsilon é um "desligado" que ainda mexe no objeto.
///
/// **Mutação que deve sangrar:** o `emit` somar uma constante.
#[test]
fn the_neutral_setting_is_the_identity_to_the_bit() {
    for (id, key) in [("shake", "amount"), ("sway", "amount")] {
        let mut stack = RecipeStack::new();
        stack.push(row_at(id, key, 0.0));
        let src = stack.to_formula();
        for base in [-3.0_f32, 0.0, 1.0, 7.5] {
            for seed in SEEDS {
                for v in samples(&src, base, seed) {
                    assert_eq!(
                        v, base,
                        "{id} com {key} = 0 tem de devolver `value` AO BIT (base {base}, \
                         seed {seed}) — a fórmula é `{src}`"
                    );
                }
            }
        }
    }
    // O `Limit` NÃO tem neutro, e isso é DADO e não omissão: um clamp sempre pode morder.
    // Declarar um neutro que não existe é o que torna o gate de neutralidade vacuoso.
    assert!(
        matches!(
            by_id("limit").expect("existe").neutral,
            ph2d_expr_recipes::Neutrality::NoNeutral
        ),
        "o Limit declara que não tem neutro"
    );
}

/// **Todo knob das três ACORDA** — e o `Roughness` acorda através do `Detail`.
///
/// ⚠️ A varredura é DUPLA (demais no default / demais no meio) porque um knob que só age
/// através de outro parece morto na primeira. Foi assim que *"mudar os parâmetros não
/// mudava a animação"* nasceu.
///
/// **Mutação que deve sangrar:** o `emit` ignorar um dos knobs (ex.: `c.n(0)` → `"2"`).
#[test]
fn every_knob_of_the_group_wakes_the_output() {
    for id in ["shake", "sway", "limit"] {
        let rec = by_id(id).expect("existe");
        let base_row = Row::new(id).expect("existe");
        for (ki, k) in rec.knobs.iter().enumerate() {
            if !matches!(base_row.knobs.get(ki), Some(KnobValue::Num(_))) {
                continue;
            }
            let mut best = 0.0_f32;
            for others_mid in [false, true] {
                for s in 1..5 {
                    let mk = |v: f32| {
                        let mut row = Row::new(id).expect("existe");
                        if others_mid {
                            for (i, o) in rec.knobs.iter().enumerate() {
                                if i != ki {
                                    row.set(o.key, KnobValue::Num((o.range.0 + o.range.1) * 0.5));
                                }
                            }
                        }
                        row.set(k.key, KnobValue::Num(v));
                        let mut st = RecipeStack::new();
                        st.push(row);
                        st.to_formula()
                    };
                    let lo = mk(k.range.0);
                    let hi = mk(k.range.0 + (k.range.1 - k.range.0) * s as f32 / 4.0);
                    for base in [0.0_f32, 4.0] {
                        let d = samples(&lo, base, SEEDS[1])
                            .into_iter()
                            .zip(samples(&hi, base, SEEDS[1]))
                            .map(|(x, y)| (x - y).abs())
                            .fold(0.0_f32, f32::max);
                        best = best.max(d);
                    }
                }
            }
            assert!(
                best > 1e-4,
                "{id}: o knob `{}` não muda a saída em varredura nenhuma — um knob morto é \
                 o report *\"mudar os parâmetros não mudava a animação\"*",
                k.label
            );
        }
    }
}

/// **O default de uma FONTE deixa o objeto na tela** — com folga.
///
/// **Mutação que deve sangrar:** subir um default para a ordem do canvas.
#[test]
fn the_defaults_of_the_group_stay_on_screen() {
    for (id, measured) in [("shake", 0.482_f32), ("sway", 1.0)] {
        let e = excursion(&Row::new(id).expect("existe"));
        assert!(
            e < CANVAS_M / 10.0,
            "{id} no default move {e:.4} m — um default da ordem do canvas é um objeto que \
             o artista não vê"
        );
        assert!(
            (e - measured).abs() < 0.05,
            "{id}: o default mede {e:.4} m e o número registrado é {measured} — se a \
             receita mudou, o doc do grupo muda junto"
        );
    }
}

/// **Nenhuma FONTE do catálogo expulsa o objeto no topo da faixa.**
///
/// ⚠️ O gate cobre o CATÁLOGO INTEIRO, não o G1, e traz uma allowlist com o número
/// MEDIDO de cada devedor — o grupo que reescrever cada uma apaga a linha dela. Uma
/// allowlist com o número é dívida visível; sem o número é uma isenção que ninguém revisita.
///
/// **Mutação que deve sangrar:** devolver `Shake.amount` ao `CANVAS_M`.
#[test]
fn no_source_range_ejects_the_object() {
    // (id, knob, canvases MEDIDOS hoje, o grupo que a reescreve)
    const OWED: &[(&str, &str, f32, &str)] = &[
        ("orbit-x", "Radius", 1.95, "G6"),
        ("orbit-y", "Radius", 1.70, "G6"),
        ("pendulum", "Amount", 1.50, "G6"),
        ("throw", "Launch Speed", 1.47, "G6"),
        ("throw", "Gravity", 2.28, "G6"),
    ];

    let mut offenders: Vec<(String, String, f32)> = Vec::new();
    for rec in ph2d_expr_recipes::CATALOG {
        if rec.combine.is_none() {
            continue; // um modificador só puxa o valor para DENTRO
        }
        let proto = Row::new(rec.id).expect("existe");
        for (ki, k) in rec.knobs.iter().enumerate() {
            if !matches!(proto.knobs.get(ki), Some(KnobValue::Num(_))) {
                continue;
            }
            let top = excursion(&row_at(rec.id, k.key, k.range.1));
            if top > CANVAS_M {
                offenders.push((rec.id.to_string(), k.label.to_string(), top / CANVAS_M));
            }
        }
    }

    // CONTROLE POSITIVO: a régua tem de continuar ACHANDO os devedores conhecidos — senão
    // ela quebrou e o gate passa por vacuidade.
    for (id, label, canvases, group) in OWED {
        let found = offenders
            .iter()
            .find(|(i, l, _)| i == id && l == label)
            .unwrap_or_else(|| panic!("{id}/{label} devia estar na lista de devedores ({group})"));
        assert!(
            (found.2 - canvases).abs() < 0.15,
            "{id}/{label}: media {:.2} canvas e o número registrado é {canvases} ({group})",
            found.2
        );
    }

    let unexpected: Vec<_> = offenders
        .iter()
        .filter(|(i, l, _)| !OWED.iter().any(|(a, b, _, _)| a == i && b == l))
        .collect();
    assert!(
        unexpected.is_empty(),
        "faixa nova que joga o objeto para fora do quadro: {unexpected:?}. O teto de uma \
         AMPLITUDE é `AMPLITUDE_M` (metade do canvas), não `CANVAS_M` — ver a derivação lá"
    );
    // ⚠️ Uma asserção entre duas consts é sempre-verdadeira e o clippy tem razão em
    // apontá-la — o que separa um teto de VALOR de um teto de AMPLITUDE já está afirmado
    // acima, pela EXCURSÃO medida, que é a única coisa que pode falhar.
}

/// **As três COMPÕEM: uma linha acima e uma abaixo mudam a resposta, e o Limit fecha.**
///
/// O que o §7.1 pede da composição — e o oráculo é o produto: um `Limit` embaixo de duas
/// fontes tem de conter as duas.
#[test]
fn the_group_composes_with_a_row_above_and_a_row_below() {
    let mut st = RecipeStack::new();
    st.push(row_at("shake", "amount", 3.0));
    st.push(row_at("sway", "amount", 4.0));
    st.push({
        let mut r = Row::new("limit").expect("existe");
        r.set("min", KnobValue::Num(-1.0));
        r.set("max", KnobValue::Num(1.0));
        r
    });
    let src = st.to_formula();
    for base in [0.0_f32, 0.5] {
        for seed in SEEDS {
            for v in samples(&src, base, seed) {
                assert!(
                    (-1.0..=1.0).contains(&v),
                    "o Limit no fim da pilha tem de conter TUDO que veio acima: {v} \
                     (fórmula `{src}`)"
                );
            }
        }
    }
    // ...e sem ele a pilha de fato passa dos limites (senão o gate acima é vacuoso).
    let mut open = RecipeStack::new();
    open.push(row_at("shake", "amount", 3.0));
    open.push(row_at("sway", "amount", 4.0));
    let free = samples(&open.to_formula(), 0.0, SEEDS[1]);
    assert!(
        free.iter().any(|v| v.abs() > 1.0),
        "CONTROLE: sem o Limit a pilha sai da faixa — senão o gate acima não prova nada"
    );
}
