//! **The gates for the catalog's SCALE** (plano 10 §8, the smoke of 2026-07-29).
//!
//! Split from `catalog.rs` by subject when that file crossed the 700-LOC cap, and
//! the line of the cut is the meaning: that file asks *does the catalog emit text
//! the one parser accepts and compose correctly?*, this one asks *are its numbers
//! the right SIZE for a canvas?* — a question the first smoke of the modal put on
//! the table and nothing else in the suite was asking.

use ph2d_expr::{Bindings, eval};
use ph2d_expr_parse::parse;
use ph2d_expr_recipes::{CATALOG, Family, KnobKind, KnobValue, RecipeStack, RowKind};

mod shared;
use shared::{Knobs, row_with, set_knobs};

// ------------------------------------------------------------- the scale

/// The window a default is JUDGED over.
///
/// ⚠️ Not an arbitrary number and not the modal's — it is *how long the artist
/// looks before deciding the default is wrong*, and it is the same two seconds the
/// preview loops over for the same reason. Judge over ten and `Free Fall` fails for
/// being gravity; judge over a tenth and nothing fails at all.
const JUDGE_SECONDS: f32 = 2.0;

/// Samples across that window.
const JUDGE_SAMPLES: usize = 240;

/// The whole curve a formula draws across the judging window.
///
/// `value` and any link are non-zero and different from each other: at `value = 0`
/// every MULTIPLICATIVE recipe (`Flicker`, `Multiply / Add`) is flat zero whatever
/// its knobs say, and a gate reading that would call them broken.
fn trajectory(src: &str, value: f32) -> Option<Vec<f32>> {
    struct Clock(f32, f32);
    impl Bindings for Clock {
        fn attr(&self, name: &str) -> f32 {
            match name {
                "time" => self.0,
                "value" => self.1,
                // Any `Name.prop` a link knob points at — DIFFERENT per name, so a
                // recipe that reads two links (`Blend Two`, `Distance`) is not
                // handed the same number twice and quietly collapsed.
                other => 0.3 + other.len() as f32 * 0.11,
            }
        }
        fn param(&self, _: &str) -> f32 {
            0.0
        }
    }
    let e = parse(src).ok()?;
    Some(
        (0..JUDGE_SAMPLES)
            .map(|i| {
                let t = i as f32 / JUDGE_SAMPLES as f32 * JUDGE_SECONDS;
                let v = eval(&e, &Clock(t, value));
                if v.is_finite() { v } else { 0.0 }
            })
            .collect(),
    )
}

/// How far a formula takes a property that is resting at zero.
fn peak_excursion(src: &str) -> Option<f32> {
    Some(
        trajectory(src, 0.0)?
            .into_iter()
            .fold(0.0_f32, |p, v| p.max(v.abs())),
    )
}

/// **No recipe flings the object off a 4K canvas at its own defaults.**
///
/// ⚠️ Born RED, and this is the gate for the first smoke of the modal: *"Shake —
/// changing the parameters did not change the animation"*. The formula responded
/// (there is a gate for that below); the SCREEN did not, because `amount: 30` meant
/// **30 metres = 3000 px** at the project's 100 px/m and the object had left the
/// frame — so every knob value looked identical, which is to say looked like
/// nothing. Measured on the pre-change catalog: **14 of 47 value recipes** did this,
/// the worst (`Free Fall`) by **777 480 px**.
///
/// The bar is HALF a canvas because the object starts at the centre: a default that
/// reaches the edge is already at the limit of useful, and one that passes it is
/// invisible. Mutation: put `amount` back to `30.0` in `life.rs` ⇒ RED, naming Shake
/// and its number.
#[test]
fn no_recipe_flings_the_object_off_a_4k_canvas() {
    let half_canvas = ph2d_expr_recipes::CANVAS_M * 0.5;
    let mut off = Vec::new();
    for r in CATALOG {
        if r.kind != RowKind::Value {
            continue;
        }
        let mut stack = RecipeStack::new();
        stack.push(row_with(r.id, Knobs::Default));
        let src = stack.to_formula();
        let Some(peak) = peak_excursion(&src) else {
            continue;
        };
        if peak > half_canvas {
            off.push(format!(
                "{} reaches {peak:.2} m ({:.0} px)",
                r.id,
                peak * 100.0
            ));
        }
    }
    assert!(
        off.is_empty(),
        "these defaults put the object off a 4K canvas ({half_canvas} m from centre):\n  {}",
        off.join("\n  ")
    );
}

/// **Turning a knob turns what the formula PRODUCES** — the half of the smoke
/// report that was never broken, pinned so nobody "fixes" it.
///
/// ⚠️ The oracle is the produced CURVE, not the emitted text and not its peak: a
/// recipe that interpolated its knob into a comment would change the string and
/// animate identically, and a peak is blind to frequency — `Sway` at 2 Hz and at
/// 30 Hz reach the same height, so the first version of this gate called eleven
/// working knobs dead.
///
/// ⚠️ **Scoped to the three MOVING families, and the narrowing is honest rather
/// than convenient.** Logic, Link, Field and Shape SELECT between values, and no
/// single fixture can make both sides of a branch live at once: with the condition
/// false the `Then` knob is genuinely unreachable, and a gate that called that
/// "inert" would need a per-recipe table of conditions to keep in sync with the
/// emitters — the drift-prone shape this catalog exists to avoid. Wave, Life and
/// PhysicsLite are exactly where the smoke report came from, and there every number
/// knob is continuous in the output, so the sweep can be total.
#[test]
fn turning_a_knob_turns_what_the_formula_produces() {
    let mut inert = Vec::new();
    for r in CATALOG {
        if r.kind != RowKind::Value
            || !matches!(r.family, Family::Wave | Family::Life | Family::PhysicsLite)
        {
            continue;
        }
        for (i, k) in r.knobs.iter().enumerate() {
            if k.kind != KnobKind::Number {
                continue;
            }
            let sample = |v: f32| {
                // Links FILLED: a knob that only matters once the pick-whip has
                // been used (`Switch`'s Above, `Gate`'s thresholds) is live code,
                // and an empty link would report it dead.
                let mut row = row_with(r.id, Knobs::Perturbed);
                set_knobs(&mut row, Knobs::Default);
                for (j, kk) in r.knobs.iter().enumerate() {
                    if kk.kind == KnobKind::Link {
                        row.knobs[j] = KnobValue::Link("Ball.x".into());
                    }
                }
                row.knobs[i] = KnobValue::Num(v);
                let mut stack = RecipeStack::new();
                stack.push(row);
                trajectory(&stack.to_formula(), 0.7)
            };
            let (lo, hi) = k.range;
            // Two thirds apart inside the range, so neither probe is the default
            // (a knob compared against itself never moves).
            let (a, b) = (lo + (hi - lo) / 6.0, lo + (hi - lo) * 5.0 / 6.0);
            if let (Some(x), Some(y)) = (sample(a), sample(b))
                && x.iter().zip(&y).all(|(p, q)| (p - q).abs() < f32::EPSILON)
            {
                inert.push(format!("{}.{}", r.id, k.key));
            }
        }
    }
    assert!(
        inert.is_empty(),
        "these knobs do not reach the arithmetic: {inert:?}"
    );
}

/// **Every knob steps in a number a person would type.**
///
/// The number box's arrows and its drag both read the registered step; without one
/// the dispatch falls back to a buffer heuristic (`1.0` for a value with no dot),
/// and one click on an Amount whose default is `0.3` moves three canvases. Mutation:
/// return the raw `span / 200` without snapping ⇒ steps like `0.3987` appear, RED.
#[test]
fn every_knob_steps_in_a_number_a_person_would_type() {
    const NICE: [f32; 9] = [0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0];
    for r in CATALOG {
        for k in r.knobs {
            if !matches!(k.kind, KnobKind::Number | KnobKind::Literal) {
                continue;
            }
            let s = k.step_value();
            assert!(
                NICE.contains(&s),
                "{}.{} steps by {s}, which is not a number anyone types",
                r.id,
                k.key
            );
            let span = (k.range.1 - k.range.0).abs();
            assert!(
                s <= span || span == 0.0,
                "{}.{} steps past its own range in one click",
                r.id,
                k.key
            );
        }
    }
    // The one override, named: octaves size an unrolled noise tree, so they are
    // integers and a derived 0.01 would be a control that does nothing four times
    // out of five.
    let detail = ph2d_expr_recipes::by_id("turbulence")
        .unwrap()
        .knobs
        .iter()
        .find(|k| k.key == "detail")
        .unwrap();
    assert!(
        (detail.step_value() - 1.0).abs() < f32::EPSILON,
        "wiggle octaves step by whole numbers"
    );
}

// ------------------------------------------------- the smoke of 2026-07-29 (2)

/// How many times a curve crosses its own mean, per second — the RATE it wobbles at.
fn crossings_per_second(v: &[f32]) -> f32 {
    let mean = v.iter().sum::<f32>() / v.len() as f32;
    let n = v
        .windows(2)
        .filter(|w| (w[0] - mean) * (w[1] - mean) < 0.0)
        .count();
    n as f32 / JUDGE_SECONDS
}

/// **A Speed knob makes it FASTER — it does not reroll it.**
///
/// ⚠️ Red-first against the smoke: *"a velocidade em shake nunca foi velocidade,
/// parece mais com um seed"*. Measured on the old lowering, across a 32x sweep of
/// the knob: **494 to 509 crossings per second** — flat, because the frozen
/// `noise(x)` primitive HASHES the bit pattern of `x`, so adjacent inputs are
/// uncorrelated and there is no frequency to change. Multiplying by Speed only
/// picked a different set of random numbers, which is the definition of a seed.
///
/// The oracle is the crossing RATE and not the values, because rerolling changes the
/// values too — that is precisely what made the defect invisible to every gate this
/// catalog already had.
#[test]
fn a_speed_knob_makes_the_wobble_faster_not_different() {
    for id in ["shake", "drift", "sway"] {
        let rate = |speed: f32| {
            let mut row = row_with(id, Knobs::Default);
            row.knobs[0] = KnobValue::Num(speed);
            let mut st = RecipeStack::new();
            st.push(row);
            crossings_per_second(&trajectory(&st.to_formula(), 0.0).unwrap())
        };
        let (slow, fast) = (rate(1.0), rate(8.0));
        assert!(
            fast > slow * 3.0,
            "{id}: 8x the Speed must wobble far faster, got {slow:.2} -> {fast:.2} crossings/s"
        );
    }
}

/// **`Jitter` still rides the raw HASH**, and that is not an oversight.
///
/// It asks for ONE fixed random offset per Seed — a value that never moves while the
/// clock does. Give it the smooth noise and neighbouring seeds stop being
/// independent, so "change Seed to reroll" would start returning near-misses.
///
/// ⚠️ The oracle is a FRACTIONAL seed, and it has to be: at an INTEGER `x` the two
/// noises are equal by construction (`mix(a, b, 0) == a`), so the first version of
/// this gate compared 7 against 8 and was measuring nothing — it failed only because
/// two unrelated draws happened to land 0.03 apart. A half-integer is where they
/// part company: the hash gives an unrelated number, the smooth noise gives exactly
/// the midpoint of its neighbours.
#[test]
fn jitter_rerolls_on_a_fractional_seed_because_it_wants_the_hash() {
    let at = |seed: f32| {
        let mut row = row_with("jitter", Knobs::Default);
        row.knobs[0] = KnobValue::Num(seed);
        row.knobs[1] = KnobValue::Num(1.0);
        let mut st = RecipeStack::new();
        st.push(row);
        trajectory(&st.to_formula(), 0.0).unwrap()[0]
    };
    let (lo, mid, hi) = (at(7.0), at(7.5), at(8.0));
    let smooth_would_be = (lo + hi) * 0.5;
    assert!(
        (mid - smooth_would_be).abs() > 0.05,
        "a Jitter seed must HASH, not interpolate: seed 7.5 gave {mid:.4}, and the \
         midpoint of its neighbours ({lo:.4}, {hi:.4}) is {smooth_would_be:.4}"
    );
    // …and it does NOT move with the clock: a Jitter is an offset, not a wobble.
    let mut st = RecipeStack::new();
    st.push(row_with("jitter", Knobs::Default));
    let v = trajectory(&st.to_formula(), 0.0).unwrap();
    assert!(
        v.windows(2).all(|w| (w[0] - w[1]).abs() < f32::EPSILON),
        "a Jitter holds still while the clock runs"
    );
}

/// **O CENSO — receita por receita, nos defaults: parseia? e o valor VARIA?**
///
/// Uma SONDA (`#[ignore]`), não um gate: ela imprime a tabela que decide onde
/// trabalhar, e um gate que falhasse aqui estaria afirmando um veredito de produto
/// (*"esta receita devia mexer"*) que só o smoke pode dar.
///
/// ⚠️ Mede com `value = 0` **E** `value = 0.7` de propósito: com repouso zero toda
/// receita MULTIPLICATIVA sai plana em zero por aritmética, e isso é indistinguível
/// de quebrada. A coluna que separa as duas é a única leitura honesta da tabela.
///
/// Rodar: `cargo test -p ph2d-expr-recipes --test catalog_scale -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime o censo, não afirma veredito"]
fn census_of_every_recipe_at_its_defaults() {
    let amp = |v: &[f32]| {
        let (lo, hi) = v
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &x| (l.min(x), h.max(x)));
        hi - lo
    };
    println!(
        "\n{:<22} {:<9} {:>9} {:>9} {:>7}  veredito",
        "id", "família", "amp@0", "amp@0.7", "cruz/s"
    );
    let (mut dead, mut zero_only, mut alive) = (vec![], vec![], vec![]);
    for r in CATALOG {
        let mut st = RecipeStack::new();
        st.push(row_with(r.id, Knobs::Default));
        let src = st.to_formula();
        let (a0, a7, cps, verdict) = match (trajectory(&src, 0.0), trajectory(&src, 0.7)) {
            (Some(t0), Some(t7)) => {
                let (a0, a7) = (amp(&t0), amp(&t7));
                let cps = crossings_per_second(&t7);
                let v = if a7 <= f32::EPSILON {
                    "CONSTANTE"
                } else if a0 <= f32::EPSILON {
                    "ZERO@value=0"
                } else {
                    "viva"
                };
                (a0, a7, cps, v)
            }
            _ => (f32::NAN, f32::NAN, f32::NAN, "PARSE-FAIL"),
        };
        println!(
            "{:<22} {:<9} {a0:>9.4} {a7:>9.4} {cps:>7.2}  {verdict}",
            r.id,
            format!("{:?}", r.family)
        );
        match verdict {
            "PARSE-FAIL" => dead.push((r.id, src)),
            "CONSTANTE" => dead.push((r.id, src)),
            "ZERO@value=0" => zero_only.push(r.id),
            _ => alive.push(r.id),
        }
    }
    println!(
        "\n== NÃO MEXEM (parse-fail ou constantes): {} ==",
        dead.len()
    );
    for (id, src) in &dead {
        println!("  {id:<22} {src}");
    }
    println!(
        "\n== SÓ ZERADAS COM value=0 (multiplicativas): {} ==",
        zero_only.len()
    );
    println!("  {}", zero_only.join(" "));
    println!("\n== VIVAS: {} ==\n  {}", alive.len(), alive.join(" "));
}

/// **Segunda leitura do censo: um MODIFICADOR precisa de uma linha acima dele.**
///
/// ⚠️ O censo mede cada receita SOZINHA e chama de constante tudo que é função de
/// `value` — o que é a verdade sobre a medição, não sobre a receita: `Limit`,
/// `Speed`, `If Greater` MODIFICAM o que vem antes, e alimentados com um `value`
/// constante devolvem constante *corretamente*. Esta sonda põe um gerador (`Sway`)
/// em cima e re-mede: o que ACORDA é modificador sadio; o que continua parado é
/// defeito de verdade.
///
/// Rodar: `cargo test -p ph2d-expr-recipes --test catalog_scale -- --ignored --nocapture`
#[test]
#[ignore = "sonda: a segunda leitura do censo"]
fn census_second_reading_modifiers_over_a_generator() {
    let amp = |v: &[f32]| {
        let (lo, hi) = v
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &x| (l.min(x), h.max(x)));
        hi - lo
    };
    let (mut woke, mut still_dead, mut inert) = (vec![], vec![], vec![]);
    println!(
        "\n{:<22} {:<9} {:>10} {:>10}",
        "id", "família", "sozinho", "sob Sway"
    );
    for r in CATALOG {
        let mut solo = RecipeStack::new();
        solo.push(row_with(r.id, Knobs::Default));
        let mut over = RecipeStack::new();
        over.push(row_with("sway", Knobs::Default));
        over.push(row_with(r.id, Knobs::Default));
        let a = |st: &RecipeStack| trajectory(&st.to_formula(), 0.7).map_or(f32::NAN, |t| amp(&t));
        let (s, o) = (a(&solo), a(&over));
        if s <= f32::EPSILON {
            // ⚠️ Uma linha que ESPERA UM ALVO é inerte no fold, então "acordou sob o
            // gerador" mediria a variação do GERADOR e não a dela — sucesso VAZIO, do
            // mesmo formato que esta sessão já pegou duas vezes. Ela é contada à parte,
            // e o que se afirma dela é só o que foi medido: não congela mais a
            // propriedade (gate `a_row_waiting_for_a_target_leaves_the_property_alone`).
            let waiting = row_with(r.id, Knobs::Default).waiting_for();
            println!(
                "{:<22} {:<9} {s:>10.4} {o:>10.4}  {}",
                r.id,
                format!("{:?}", r.family),
                waiting.unwrap_or("")
            );
            if let Some(k) = waiting {
                inert.push((r.id, k));
            } else if o > f32::EPSILON {
                woke.push(r.id);
            } else {
                still_dead.push((r.id, over.to_formula()));
            }
        }
    }
    println!(
        "\n== ACORDARAM sob um gerador (modificadores sadios): {} ==\n  {}",
        woke.len(),
        woke.join(" ")
    );
    println!(
        "\n== INERTES até escolherem alvo (não congelam mais a propriedade): {} ==",
        inert.len()
    );
    for (id, k) in &inert {
        println!("  {id:<22} espera: {k}");
    }
    println!(
        "\n== CONTINUAM PARADAS (defeito de verdade): {} ==",
        still_dead.len()
    );
    for (id, src) in &still_dead {
        println!("  {id:<22} {src}");
    }
}

/// **Uma linha que espera um alvo é INERTE — ela não substitui a propriedade.**
///
/// ⚠️ Red-first contra o smoke (*"alguns não funcionam em nada"*). Um knob de link
/// nasce VAZIO e o `EmitCtx::link` o desce como o literal `0` — de propósito, por uma
/// razão que continua certa (a fórmula tem de parsear em toda tecla, senão o card
/// apaga ao adicionar a linha). O que nunca foi precificado é a consequência: `0`
/// parseia **e substitui a propriedade por uma constante**. Medido nos defaults, antes
/// desta correção:
///
/// ```text
/// follow            0*1 + 0     -> o objeto TELEPORTA para a origem e congela
/// offset-copy       0 + 0.2     -> e estaciona em 0,2
/// fade-by-distance  mix(1,0,..) -> preso em 1
/// ```
///
/// Quatorze receitas, UMA causa — e é a mesma de *"não produz a curva do grafo"*,
/// porque uma constante desenha uma reta. Um defeito, dois sintomas.
///
/// O oráculo é a IDENTIDADE, não a ausência do `0`: a fórmula de uma linha inacabada
/// tem de ser exatamente o que uma planilha VAZIA produz. Procurar `"0"` no texto
/// passaria com qualquer receita que só mudasse a aritmética do teleporte.
#[test]
fn a_row_waiting_for_a_target_leaves_the_property_alone() {
    let empty = RecipeStack::new().to_formula();
    for r in CATALOG {
        let row = row_with(r.id, Knobs::Default);
        let Some(missing) = row.waiting_for() else {
            continue;
        };
        let mut st = RecipeStack::new();
        st.push(row);
        assert_eq!(
            st.to_formula(),
            empty,
            "{}: espera o knob {missing:?} e mesmo assim reescreve a propriedade",
            r.id
        );
        // …e continua parseável, que é a razão pela qual o `0` existia.
        assert!(parse(&st.to_formula()).is_ok(), "{}: não parseia", r.id);
    }
}

/// **E o que ela espera é dito por UMA função** — a mesma que decide o fold.
#[test]
fn the_missing_knob_is_named_by_the_door_that_skips_the_row() {
    let follow = row_with("follow", Knobs::Default);
    assert!(
        follow.waiting_for().is_some(),
        "um link vazio é uma linha inacabada"
    );
    assert!(
        !follow.is_live(),
        "…e uma linha inacabada não entra no fold"
    );
    let mut filled = follow.clone();
    assert!(filled.set("target", KnobValue::Link("Ball.x".into())));
    assert_eq!(
        filled.waiting_for(),
        None,
        "com o alvo escolhido, a linha está pronta"
    );
    assert!(filled.is_live() && filled.waiting_for().is_none());
}

#[test]
#[ignore = "sonda: a medicao por tras do default de tolerancia do If Near"]
fn measure_the_tolerance_band_of_if_near() {
    // Sob um gerador tipico (Sway, amplitude 1 m), que fracao do tempo o sinal passa
    // dentro de +-tol de uma marca no meio da faixa?
    let wave = "value + sin((time + 0)*3)*0.5";
    for tol in [0.01_f32, 0.02, 0.05, 0.1, 0.2, 0.4] {
        let src = format!("select(abs(({wave}) - 0.5) < {tol}, 1, 0)");
        let t = trajectory(&src, 0.7).unwrap();
        let fired = t.iter().filter(|v| **v > 0.5).count();
        println!(
            "tol {tol:>5.2} m ({:>5.1} px)  dispara em {:>5.1}% do tempo  ({fired}/{} amostras)",
            tol * 100.0,
            100.0 * fired as f32 / t.len() as f32,
            t.len()
        );
    }
}
