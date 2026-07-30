//! **The preview ribbon draws what the OBJECT does** (FASE 0.2 do plano 12).
//!
//! The second of the three instruments the audit of 2026-07-29 proved broken. One
//! question — *what is this channel's `__seed`?* — had **three answers**, and the audit
//! measured all three (§4 D-J):
//!
//! | quem | seed | o deslocamento de um `Jitter` |
//! |---|---|---|
//! | a FITA | 0.00 | 0.1971 |
//! | o CENSO de cobertura | 0.96 | 0.0736 |
//! | objeto #0 | 0 | 0.1971 |
//! | objeto #1 | 100 | 0.0881 |
//! | objeto #2 | 200 | **0.0089** |
//! | objeto #3 | 300 | 0.0727 |
//!
//! The ribbon always drew object zero's wobble. On the third object the real
//! displacement is **0.9 px** at 100 px/m — *"Jitter não funciona"* was literal, and
//! literal for SOME objects and not others, which is why it read as random.
//!
//! ⚠️ **There was no gate comparing the ribbon to the scene at all.** The one ribbon gate
//! that existed (`the_preview_samples_the_window_once_and_both_views_read_it`) uses
//! `sway` — a pure sine that never reads `__seed`. The fixture did not contain the
//! phenomenon, so it could not have caught this however carefully it was written.
//!
//! ⚠️ These gates go through **`ph2d_timeline::seed_of_target`** for the scene's answer,
//! which is the door the pass itself reads — not a re-derivation of `target * 100`. A
//! second copy of the formula here would be a fourth answer to the same question, in the
//! test that exists to prove there is only one.

use ph2d_expr_recipes::RecipeStack;
use ph2d_panel_timeline::expr_modal_preview as pv;
use ph2d_timeline::PropKind;

/// Evaluate a formula the way the SCENE does, for one target.
///
/// Deliberately a small independent evaluator rather than a call into the pass: the pass
/// needs a document, a clip and a playhead, and what is under test is one binding — *does
/// the ribbon's `__seed` equal the scene's?*. It reads the seed from the product's door.
fn scene_samples(stack: &RecipeStack, base: f32, target: u64) -> Vec<f32> {
    struct B {
        time: f32,
        value: f32,
        seed: f32,
    }
    impl ph2d_expr::Bindings for B {
        fn attr(&self, name: &str) -> f32 {
            match name {
                "value" => self.value,
                "time" => self.time,
                "__seed" => self.seed,
                _ => 0.0,
            }
        }
        fn param(&self, _: &str) -> f32 {
            0.0
        }
    }
    let e = ph2d_expr_parse::parse(&stack.to_formula()).expect("the catalog emits parseable text");
    let seed = ph2d_timeline::seed_of_target(target);
    (0..pv::PREVIEW_SAMPLES)
        .map(|i| {
            let t = (i as f64 / pv::PREVIEW_SAMPLES as f64) * pv::PREVIEW_SECONDS;
            ph2d_expr::eval(
                &e,
                &B {
                    time: t as f32,
                    value: base,
                    seed,
                },
            )
        })
        .collect()
}

/// Recipes that read `__seed` — the only ones that can see this defect. `remap` is the
/// CONTROL: it reads no noise, so it must agree across every target either way.
///
/// ⚠️ **`turbulence` estava aqui e a FASE A o aposentou** (absorvido pelo `shake`), e o que
/// aconteceu é a lição: `RecipeStack::of` faz `filter_map`, então um id que não existe é
/// **silenciosamente descartado** e a pilha nasce VAZIA — a fórmula vira `value`, uma
/// constante. O gate de "objetos diferentes tremem diferente" pegou isso por sorte (uma
/// constante é igual a si mesma), e o gate de IGUALDADE ficou **verde sobre uma fixture
/// vazia**. Daí a asserção de premissa em [`every_probe_names_a_recipe_that_exists`]: um
/// gate que nomeia uma receita por string fica vacuous no dia em que ela sai.
const NOISY: [&str; 2] = ["shake", "jitter"];

/// **Todo id que estas fixtures nomeiam existe.**
///
/// A guarda contra a classe inteira. Sem ela, aposentar uma receita deixa os gates que a
/// nomeiam VERDES e vazios — e um gate vazio é pior que gate nenhum, porque ele é contado.
#[test]
fn every_probe_names_a_recipe_that_exists() {
    for id in NOISY.iter().chain(["remap"].iter()) {
        assert!(
            ph2d_expr_recipes::by_id(id).is_some(),
            "a fixture nomeia {id:?}, que não está no catálogo — `RecipeStack::of` a \
             descartaria em silêncio e estes gates passariam a medir uma pilha VAZIA"
        );
    }
}

/// **For every object, the ribbon's samples are the scene's samples, one for one.**
#[test]
fn the_ribbon_draws_what_the_object_does() {
    let base = pv::preview_value(PropKind::TranslationX);
    for id in NOISY {
        let stack = RecipeStack::of(&[id]);
        for target in [0_u64, 1, 2, 3, 17, 250] {
            let ribbon = pv::sample_window(&stack, base, target);
            let scene = scene_samples(&stack, base, target);
            assert_eq!(
                ribbon.len(),
                scene.len(),
                "{id}: the ribbon and the scene must sample the same window"
            );
            for (i, (r, s)) in ribbon.iter().zip(scene.iter()).enumerate() {
                assert_eq!(
                    r, s,
                    "{id} on target {target}: the ribbon drew {r} at sample {i} where the \
                     object does {s}. The ribbon must not have a seed of its own."
                );
            }
        }
    }
}

/// **A noisy recipe LOOKS different on different objects — and the ribbon follows.**
///
/// ⚠️ This is the half that equality alone cannot state. If `sample_window` ignored the
/// target *and* `scene_samples` did too, the gate above would be green over a ribbon that
/// draws one wobble for the whole scene. Here the ribbon is asked to DISAGREE with itself
/// across targets, which only a real seed can do — and the control (`remap`, which reads
/// no noise) is asked to agree, so "disagrees" cannot be satisfied by drawing garbage.
#[test]
fn a_noisy_recipe_looks_different_on_different_objects_and_flat_ones_do_not() {
    let base = pv::preview_value(PropKind::TranslationX);

    for id in NOISY {
        let stack = RecipeStack::of(&[id]);
        let a = pv::sample_window(&stack, base, 0);
        let b = pv::sample_window(&stack, base, 2);
        let spread = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).abs())
            .fold(0.0_f32, f32::max);
        assert!(
            spread > 1e-6,
            "{id}: objects 0 and 2 must not wobble identically — the ribbon is ignoring \
             the seed (max |Δ| = {spread})"
        );
    }

    // The CONTROL: a recipe that reads no noise is the same on every object, so the
    // assertion above is about the SEED and not about the ribbon being unstable.
    let flat = RecipeStack::of(&["remap"]);
    let a = pv::sample_window(&flat, base, 0);
    let b = pv::sample_window(&flat, base, 2);
    assert_eq!(
        a, b,
        "a recipe with no noise must draw identically on every object"
    );
}

/// **The scene's two evaluators agree with each other**, so "the ribbon matches the scene"
/// is a statement about one number and not about which of two scenes was asked.
///
/// The blend path (`stack_eval`) and the post-composition pass (`expr_pass`) both compute
/// `__seed`; before FASE 0.2 each did it with its own copy of `target * SEED_SPACING`.
/// They agreed — but nothing said they had to, and the ribbon's disagreement was born the
/// same way.
#[test]
fn the_one_seed_door_is_linear_in_the_target_and_spaced() {
    // Read the shape of the law, not its constant: what matters is that distinct targets
    // get distinct, evenly spaced seeds. Pinning `100.0` here would make this gate a
    // second home for a number that already has one.
    let s0 = ph2d_timeline::seed_of_target(0);
    let s1 = ph2d_timeline::seed_of_target(1);
    let s2 = ph2d_timeline::seed_of_target(2);
    assert_eq!(s0, 0.0, "target 0 seeds at the origin");
    assert!(s1 > 0.0, "distinct targets get distinct seeds");
    assert_eq!(
        s2 - s1,
        s1 - s0,
        "the spacing is uniform, so no two bindings can collide by arithmetic"
    );
}
