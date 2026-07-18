//! **Fatia 0 do nesting** ([plano](../../../docs/Timeline/04_plano_nesting.md) §2):
//! a pergunta do z, respondida por gate — e o custo, medido.
//!
//! # Por que esta é a fatia bloqueante
//!
//! O aviso mais forte contra o nesting veio do **Spine**, que nunca o implementou em 10 anos:
//! cada skeleton é desenhado inteiro antes do próximo, e não dá para intercalar draw order entre
//! skeletons — *"if that is needed, it is easiest to use a single skeleton"*. Como o nosso z-order
//! é projeção da árvore única ([ADR-0110]), a pergunta é se N instâncias de um container podem
//! coexistir numa pilha só sem que os filhos de uma se embaralhem com os da outra.
//!
//! **A resposta já estava construída, e não é nossa: é o `SortingGroup`** (o *Sorting Group* do
//! Unity, `sorting.rs` §5.2 passo 5) — a sub-árvore inteira ordena como UMA unidade, na posição da
//! raiz do grupo. É exatamente o que "conter" significa, e é por isso que a dor do Spine não é a
//! nossa: o Spine precisava intercalar, e um container **não deve** intercalar.
//!
//! O que o Spine chama de limitação, nós chamamos de semântica — e há escape hatch para o caso
//! raro (`SortingGroup::sort_at_root` num descendente o tira do bloco).
//!
//! # O que "ordena como um bloco" quer dizer, operacionalmente
//!
//! Que os filhos de uma instância ocupam uma **faixa CONTÍGUA** na ordem total. Se o nesting
//! quebrasse o draw order, eles se intercalariam. É isso que os gates medem.
//!
//! O probe de custo é `#[ignore]`: é **medição para decidir**, não regressão a proteger (o gate de
//! perf do nesting nasce na Fatia 2, contra o número que este produzir).
//!
//! ```text
//! cargo test -p ph2d-ecs --release --test nesting_sorts_as_a_block -- --ignored --nocapture
//! ```
//!
//! [ADR-0110]: ../../../docs/architecture/decisions/0110-vector-nodes-are-ecs-entities-one-hierarchy.md

use bevy_ecs::component::Component;
use ph2d_core::Vec2;
use ph2d_ecs::sort_key::{SortScratch, compute_sort_ranks_into};
use ph2d_ecs::{
    ChildOf, PresentWorld, SimWorld, SortInput, SortingGroup, Transform, TransformPropagationState,
    WorklistBuf, YSort,
};

/// Which instance a sprite belongs to, and which sprite it is inside it.
///
/// Instance identity is what the contiguity assertion is ABOUT, so it cannot be
/// `Entity` bits (allocation order) — same reason the determinism gate uses `Label`.
#[derive(Component, Copy, Clone, Debug)]
struct Tag {
    inst: u16,
    idx: u16,
}

fn t(x: f32, y: f32) -> Transform {
    Transform::from_translation(Vec2::new(x, y))
}

/// `n` container instances under one YSort root, each holding `k` sprites, nested `depth` levels
/// deep. `grouped` decides whether the container roots carry [`SortingGroup`].
///
/// # The fixture is ADVERSARIAL on purpose
///
/// The instances' sprites are laid out so their world Y ranges **overlap heavily**: instance `i`
/// sits at `y = i`, and its sprites span `0..2k` locally. Under a plain YSort every sprite of
/// every instance competes on one axis, so *without* grouping they MUST interleave.
///
/// A fixture only proves what it contains ([[reference_topic_fixture_discipline]]): if the
/// instances were spatially separated, "contiguous" would be true for a reason that has nothing to
/// do with `SortingGroup`, and the gate would be green over a feature that never ran.
fn scene(n: u16, k: u16, depth: u8, grouped: bool) -> SimWorld {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let ysort_root = w.spawn((t(0.0, 0.0), YSort::default())).id();

    for inst in 0..n {
        // Each nesting level is a container root; only the outermost needs to be a block for the
        // instances not to interleave, but an inner container is a container too — a descendant
        // `SortingGroup` with `sort_at_root: false` stays INSIDE the enclosing block, which is the
        // containment semantics we want and which `nested_containers_stay_inside_the_outer_block`
        // pins.
        let mut parent = ysort_root;
        for _ in 0..depth {
            let mut e = w.spawn((t(0.0, f32::from(inst)), ChildOf(parent)));
            if grouped {
                e.insert(SortingGroup::default());
            }
            parent = e.id();
        }
        for idx in 0..k {
            w.spawn((
                t(0.0, f32::from(idx) * 2.0),
                ChildOf(parent),
                Tag { inst, idx },
            ));
        }
    }
    sim
}

/// Drive the REAL propagate → sort path and return `(inst, idx)` in render order.
fn render_order(sim: &mut SimWorld) -> Vec<(u16, u16)> {
    let mut present = PresentWorld::new();
    let mut state = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::new();

    let mut inputs: Vec<SortInput> = Vec::new();
    ph2d_ecs::extract!(*sim => present, |sim_w, present_w| {
        propagate(sim_w, &mut state, present_w, &mut worklist, &mut inputs);
    });

    let mut scratch = SortScratch::new();
    compute_sort_ranks_into(&mut scratch, sim.world(), &inputs);
    let mut by_rank: Vec<(u32, (u16, u16))> = inputs
        .iter()
        .map(|s| {
            let tag = sim
                .world()
                .get::<Tag>(s.entity)
                .expect("only tagged sprites enter");
            let rank = scratch.rank(s.entity).expect("every input is ranked");
            (rank, (tag.inst, tag.idx))
        })
        .collect();
    by_rank.sort_unstable();
    by_rank.into_iter().map(|(_, t)| t).collect()
}

/// The propagate half, split out so the probe can reuse it without the ranking.
fn propagate(
    sim_w: &bevy_ecs::world::World,
    state: &mut TransformPropagationState,
    present_w: &mut bevy_ecs::world::World,
    worklist: &mut WorklistBuf,
    inputs: &mut Vec<SortInput>,
) {
    inputs.clear();
    ph2d_ecs::propagate_transforms(sim_w, state, present_w, worklist, |s, _p, e, gt| {
        // Only tagged sprites participate — mirrors the real extract, which emits only
        // sprite-bearing entities (a container root draws nothing).
        if s.get::<Tag>(e).is_some() {
            inputs.push(SortInput {
                entity: e,
                world_pos: gt.translation(),
            });
        }
    });
}

/// The instances, in the order their first sprite appears, with the run length of each.
fn runs(order: &[(u16, u16)]) -> Vec<(u16, usize)> {
    let mut out: Vec<(u16, usize)> = Vec::new();
    for &(inst, _) in order {
        match out.last_mut() {
            Some((cur, len)) if *cur == inst => *len += 1,
            _ => out.push((inst, 1)),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// The z question, as a gate
// ---------------------------------------------------------------------------

/// **THE Fatia 0 answer.** Four container instances whose sprites overlap in Y sort as four
/// contiguous blocks — no interleaving, at any instance count.
#[test]
fn a_container_instance_sorts_as_one_block() {
    let (n, k) = (4, 6);
    let order = render_order(&mut scene(n, k, 1, true));
    assert_eq!(order.len() as u16, n * k, "every sprite is ranked");

    let runs = runs(&order);
    assert_eq!(
        runs.len(),
        n as usize,
        "expected {n} contiguous blocks, got {} runs — the instances INTERLEAVED, \
         which is the Spine problem arriving in our pipeline:\n  {runs:?}",
        runs.len()
    );
    for (inst, len) in &runs {
        assert_eq!(
            *len, k as usize,
            "instance {inst} contributed a run of {len}, not {k}"
        );
    }
}

/// **The positive control, and the reason the gate above means anything.**
///
/// The SAME scene without `SortingGroup` MUST interleave. Without this, "contiguous" could be true
/// because the fixture never had overlapping sprites to begin with, and the gate would be green
/// over a mechanism that never ran ([[feedback_a_green_gate_may_be_green_by_accident]]).
#[test]
fn without_the_group_the_same_scene_interleaves() {
    let (n, k) = (4, 6);
    let order = render_order(&mut scene(n, k, 1, false));
    let runs = runs(&order);
    assert!(
        runs.len() > n as usize,
        "the ungrouped fixture produced {} runs for {n} instances — it did NOT interleave, so the \
         block gate is not testing `SortingGroup` at all. Make the Y ranges overlap.",
        runs.len()
    );
}

/// A container inside a container stays inside the outer block: nesting does not leak.
///
/// `SortingGroup::sort_at_root` defaults to `false`, which is what keeps a descendant group inside
/// the enclosing one. The escape hatch (`true`) is the documented way OUT, and it is deliberately
/// not exercised here — this gate is about the default being containment.
#[test]
fn nested_containers_stay_inside_the_outer_block() {
    let (n, k) = (3, 5);
    for depth in [1_u8, 2, 3] {
        let order = render_order(&mut scene(n, k, depth, true));
        let runs = runs(&order);
        assert_eq!(
            runs.len(),
            n as usize,
            "at depth {depth} the instances split into {} runs instead of {n}",
            runs.len()
        );
    }
}

// ---------------------------------------------------------------------------
// The cost probe (measurement for a decision, not a bar)
// ---------------------------------------------------------------------------

/// Wall-clock for `frames` propagate+sort passes, in microseconds per frame.
fn per_frame_us(n: u16, k: u16, depth: u8, frames: u32) -> (usize, f64) {
    let mut sim = scene(n, k, depth, true);
    let mut present = PresentWorld::new();
    let mut state = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::new();
    let mut inputs: Vec<SortInput> = Vec::new();
    let mut scratch = SortScratch::new();

    // Warm the buffers — the scratch is reused, never refreshly allocated, exactly as the shell's
    // `sim_extract` does it.
    ph2d_ecs::extract!(sim => present, |sw, pw| {
        propagate(sw, &mut state, pw, &mut worklist, &mut inputs);
    });
    compute_sort_ranks_into(&mut scratch, sim.world(), &inputs);
    let sprites = inputs.len();

    let start = std::time::Instant::now();
    for _ in 0..frames {
        ph2d_ecs::extract!(sim => present, |sw, pw| {
            propagate(sw, &mut state, pw, &mut worklist, &mut inputs);
        });
        compute_sort_ranks_into(&mut scratch, sim.world(), &inputs);
    }
    (
        sprites,
        start.elapsed().as_secs_f64() * 1e6 / f64::from(frames),
    )
}

/// **The Fatia 0 cost table.** N instances x depth, on the real propagate → sort path.
///
/// What it is NOT measuring, deliberately: an intermediate raster per container. **We do not have
/// one** — the AE trap (*"Comp 2 receives only the composited frame… and has no history of the
/// layers in the first comp"*) does not apply, because our sort pipeline flattens the whole tree
/// into ONE ordered list of instances. That is the Animate graphic-symbol model, not the AE
/// precomp model, and it is why "compose N containers" is not a new cost class here — it is the
/// same sprite list, longer.
#[test]
#[ignore = "wall-clock probe; run on demand (see module docs)"]
fn the_cost_of_n_instances_by_depth() {
    println!("   inst  depth  sprites   us/frame   us/sprite");
    for depth in [1_u8, 2, 3] {
        for n in [1_u16, 4, 16] {
            let (sprites, us) = per_frame_us(n, 32, depth, 2_000);
            println!(
                "  {n:>5}  {depth:>5}  {sprites:>7}  {us:>9.1}  {:>10.4}",
                us / sprites as f64
            );
        }
    }
}
