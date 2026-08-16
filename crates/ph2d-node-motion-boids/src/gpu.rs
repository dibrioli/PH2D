//! The GPU kernel for `motion.boids` (ADR-0140 Phase 3b) — the O(N²) all-pairs
//! flock, answered on the device by the spatial grid.
//!
//! The sequencer builds a neighbourhood grid over the `pre` state's `P` (cell =
//! `radius`, so a 3×3 cell sweep is exactly the within-radius set the CPU scans)
//! before this pass; the body then reproduces `crate::step`/`crate::seed` bit-for
//! -bit at the seed (integer `hash3`) and within ε at the step (the neighbour SET
//! is identical, only the float sum ORDER differs — ADR-0140 D4).
//!
//! - **Seed vs step** branches on `HAS_state_P`: tick 0 (the `pre` is Empty, so
//!   the state carries no `P`) seeds the hashed cloud; from tick 1 it steps. The
//!   two are separate compiled pipelines (presence-keyed), like the integrator.
//! - **`count` is NOT a kernel param.** The dispatch length already IS the count
//!   (via [`count_law`]), and a param called `count` would collide with the
//!   uniform's built-in `count` (`codegen::wgsl_field`). The body reads
//!   `params.count`, the dispatch size, which the count law makes equal.
//! - The **home** is read broadcast (`0` when unconnected — the origin, exactly
//!   the CPU's `value_head` of an empty stream).

use ph2d_nodegraph::gpu::{
    ColumnAccess, ColumnBinding, CountLawCtx, GpuKernel, GridSpec, SourceWindow,
};
use ph2d_nodegraph::node::{RECOMMENDED_MAX_ELEMENTS, param_as_count};
use ph2d_nodegraph::port::Dim;

/// The state `P`/`vel` (read off `pre`, written to the output) plus the two value
/// targets (broadcast). `sim_t` rides `pre` too: read for `dt`, written forward.
static BINDINGS: &[ColumnBinding] = &[
    ColumnBinding {
        column: "P",
        dim: Dim::Vec2,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 2,
    },
    ColumnBinding {
        column: "vel",
        dim: Dim::Vec2,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 2,
    },
    ColumnBinding {
        column: "sim_t",
        dim: Dim::Scalar,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 2,
    },
    // The external urge a `force.*` in the state chain accumulated. Absent ⇒ the
    // identity zeros, so a flock no force reaches produces the module it always
    // produced.
    //
    // ⚠️ **`Consume` states the transient discipline — and today it is NOT
    // observable, which is measured, not assumed.** It is the right word (the
    // flock's output must never carry `accel` forward, or next tick's forces
    // would add onto a stale value while the CPU's fresh stream starts at zero),
    // and it is what `motion.integrate` declares for the same column. But the
    // ride-through it suppresses runs off the BASE port, and this node's base
    // (port 0, a `VALUE`) does not match its output (`INST_VEC2`), so
    // `encode::rides_base` is false and the output starts empty: `Consume`
    // removes a column that was never there.
    //
    // Mutating it to `Read` therefore passes every gate — **run, not guessed**.
    // No gate is written for it, because a gate that cannot fail for the reason
    // it names is worse than none; the day this node's port 0 becomes an
    // instance stream, the word here is already correct and the parity gate
    // above starts earning it.
    ColumnBinding {
        column: "accel",
        dim: Dim::Vec2,
        access: ColumnAccess::Consume,
        identity: [0.0; 4],
        port: 2,
    },
    // ⚠️ **A massa INVERSA, e a identidade dela NÃO é zero.** Ausente lê como `1`
    // (livre) — pôr `[0.0; 4]` aqui congelaria TODO bando que nenhum pino alcança,
    // que é o modo de falha mais caro que uma binding pode ter.
    //
    // `Consume` e não `Read` pela MESMA disciplina do `accel` acima: o pino
    // MULTIPLICA no que já está no stream, então um bando que reemitisse
    // `inv_mass` faria um pino parcial decair a cada tique.
    ColumnBinding {
        column: "inv_mass",
        dim: Dim::Scalar,
        access: ColumnAccess::Consume,
        identity: [1.0, 0.0, 0.0, 0.0],
        port: 2,
    },
    ColumnBinding {
        column: "v",
        dim: Dim::Scalar,
        access: ColumnAccess::ReadBroadcast,
        identity: [0.0; 4],
        port: 0,
    },
    ColumnBinding {
        column: "v",
        dim: Dim::Scalar,
        access: ColumnAccess::ReadBroadcast,
        identity: [0.0; 4],
        port: 1,
    },
];

/// The count is the `count` param (a source mints its own agents), NOT port 0's
/// length — port 0 is a value home of length 1 or 0. The SAME clamp the CPU
/// `eval` uses, so the two dispatch the same number of agents.
fn count_law(ctx: &CountLawCtx<'_>) -> SourceWindow {
    let n = param_as_count((ctx.param)("count"), RECOMMENDED_MAX_ELEMENTS).max(1);
    SourceWindow::of_count(n)
}

/// Constants + `hash3` (the integer splitmix of `crate::hash`, so the seed is
/// bit-exact CPU↔GPU) + a finite guard (the `is_finite` of `crate::step`).
const WGSL_LIB: &str = r#"
const BOIDS_SEED_SPREAD: f32 = 3.0;
const BOIDS_SEED_REF: f32 = 64.0;
const BOIDS_MIN_SPEED_FRAC: f32 = 0.2;
const BOIDS_EPS: f32 = 1e-6;
const BOIDS_MAX_DT: f32 = 0.1;

fn boids_hash3(a: u32, b: u32, lane: u32) -> f32 {
    var h = a * 0x9e3779b9u + b * 0x85ebca6bu + lane * 0xc2b2ae35u;
    h = h ^ (h >> 16u);
    h = h * 0x7feb352du;
    h = h ^ (h >> 15u);
    h = h * 0x846ca68bu;
    h = h ^ (h >> 16u);
    return f32(h >> 8u) / 16777216.0;
}

// Graus -> cosseno do meio-angulo. O verbatim do `crate::cos_half_fov`, e o
// `-1.0` EXATO em >= 360 e' o que faz o ramo do cone ficar desligado nas duas
// rotas: um `cos(180deg)` do vendedor que saisse -0.99999994 ligaria o teste no
// device e nao na CPU, e as duas passariam a percorrer vizinhancas diferentes.
fn boids_cos_half_fov(deg: f32) -> f32 {
    if (deg >= 360.0) { return -1.0; }
    return cos(radians(max(deg, 0.0)) * 0.5);
}

fn boids_finite2(v: vec2<f32>) -> bool {
    return v.x == v.x && v.y == v.y && abs(v.x) < 3.4028235e38 && abs(v.y) < 3.4028235e38;
}
"#;

/// The per-element body. `read_state_*`/`write_*`/`grid_*` are generated by
/// `ph2d_gpu_cook::codegen` (multi-input node ⇒ readers are port-qualified).
const WGSL: &str = r#"
    let home = vec2<f32>(read_target_x_v(i), read_target_y_v(i));
    if (!HAS_state_P) {
        // SEED (tick 0 / empty pre): a hashed cloud of positions + velocities
        // around home, matching `crate::seed` bit-for-bit (integer hash). The
        // half-extent is fixed, or grown with √N so a huge flock stays a bounded
        // -density murmuration (`Params::seed_half_extent`); the one `sqrt` is
        // the only op the two seeds diverge on, and only at `spread` on → ε.
        let seed = u32(round(max(params.seed, 0.0)));
        let half = select(BOIDS_SEED_SPREAD,
                          BOIDS_SEED_SPREAD * sqrt(f32(params.count) / BOIDS_SEED_REF),
                          params.spread > 0.5);
        let px = home.x + (boids_hash3(seed, i, 0u) - 0.5) * 2.0 * half;
        let py = home.y + (boids_hash3(seed, i, 1u) - 0.5) * 2.0 * half;
        let vx = (boids_hash3(seed, i, 2u) - 0.5) * params.max_speed;
        let vy = (boids_hash3(seed, i, 3u) - 0.5) * params.max_speed;
        write_P(i, vec2<f32>(px, py));
        write_vel(i, vec2<f32>(vx, vy));
        write_sim_t(i, params.playhead);
        return;
    }
    // STEP — the three Reynolds urges over the within-radius neighbours, found
    // through the grid (`crate::step`).
    let pi = read_state_P(i);
    let vi = read_state_vel(i);
    let dt = clamp(params.playhead - read_state_sim_t(i), 0.0, BOIDS_MAX_DT);
    let r2 = params.radius * params.radius;
    let sep_r2 = params.separation_radius * params.separation_radius;
    let cos_half = boids_cos_half_fov(params.fov);
    // MASSA INFINITA: o agente nao se move -- e continua a ser VISTO, porque o
    // laco de vizinhos le `read_state_P(j)`/`read_state_vel(j)`, o estado de
    // ENTRADA. O espelho exacto do `crate::step`, incluindo a velocidade a ZERO.
    let w_i = read_state_inv_mass(i);
    if (w_i <= 0.0) {
        write_P(i, pi);
        write_vel(i, vec2<f32>(0.0, 0.0));
        write_sim_t(i, params.playhead);
        return;
    }
    var sep = vec2<f32>(0.0, 0.0);
    var align = vec2<f32>(0.0, 0.0);
    var centroid = vec2<f32>(0.0, 0.0);
    var neighbours = 0u;
    let ci = grid_cell_of(pi);
    for (var dy = -1; dy <= 1; dy = dy + 1) {
        for (var dx = -1; dx <= 1; dx = dx + 1) {
            let c = ci + vec2<i32>(dx, dy);
            // The collide's per-cell cull (its `2d0297c0` mechanism), ported: a
            // cell whose NEAREST point is already past the perception radius is
            // skipped before touching a single boid in it. Exact, never
            // approximate -- a neighbour needs `dsq <= r2` and every point of
            // the cell is at least this far; the skip is STRICT `>` because the
            // neighbour test is INCLUSIVE (`dsq > r2` skips), the mirror of the
            // collide's `>=` whose contact is strict.
            //
            // MEASURED at ~6%, not the collide-context "~20%" estimate (ABA'
            // with cooldowns, A/A' repeatable to 0,1% at scale: 1M 15,5->14,6 ·
            // 2M 43,8->41,0 · 8M 293->274 ms/tick; non-monotonic +1% at 4M).
            // Only the 4 CORNER cells of a 3x3 sweep are ever skippable, ~21,5%
            // of the time each (1 - pi/4) -- that bounds the win; the collide
            // gains more because its reach is 2-3 (25-49 cells). ⚠️ A first
            // measurement WITHOUT cooldowns read this as -9% and nearly refuted
            // it: three heavy sweeps back-to-back drift the GPU clock by 20%
            // A-to-A, more than the effect under test.
            let cl = vec2<f32>(f32(c.x), f32(c.y)) * params.grid_cell;
            let cg = max(max(cl - pi, pi - (cl + vec2<f32>(params.grid_cell, params.grid_cell))), vec2<f32>(0.0, 0.0));
            if (dot(cg, cg) > r2) { continue; }
            let b = grid_bucket_of(c);
            let hi = grid_starts[b + 1u];
            for (var s = grid_starts[b]; s < hi; s = s + 1u) {
                let j = grid_sorted[s];
                if (j == i) { continue; }
                let pj = read_state_P(j);
                let cj = grid_cell_of(pj);
                // Exact-cell dedup: count j only while visiting its own cell.
                if (cj.x != c.x || cj.y != c.y) { continue; }
                let d = pi - pj;
                let dsq = dot(d, d);
                if (dsq < BOIDS_EPS) { continue; }
                // O ESPACO PESSOAL -- fisico, nao perceptual: corre ANTES do gate
                // de percepcao e do cone, o `crate::step` verbatim. O ramo mantem
                // `separation_radius = 0` byte-identico.
                if (sep_r2 > 0.0 && dsq < sep_r2) {
                    sep = sep + d * (1.0 / dsq);
                }
                if (dsq > r2) { continue; }
                // O CONE DE VISAO -- o `crate::step` verbatim, com o mesmo ramo:
                // em 360 nada aqui roda, entao o disco de hoje nao paga a raiz.
                if (cos_half > -1.0) {
                    let v_sq = dot(vi, vi);
                    // Um agente PARADO enxerga tudo (sem heading nao ha cone).
                    if (v_sq > BOIDS_EPS) {
                        let ahead = dot(-d, vi);
                        if (ahead < cos_half * sqrt(dsq * v_sq)) { continue; }
                    }
                }
                if (sep_r2 <= 0.0) {
                    sep = sep + d * (1.0 / dsq);
                }
                align = align + read_state_vel(j);
                centroid = centroid + pj;
                neighbours = neighbours + 1u;
            }
        }
    }
    var accel = vec2<f32>(0.0, 0.0);
    if (neighbours > 0u) {
        let inv_n = 1.0 / f32(neighbours);
        accel = accel + (align * inv_n - vi) * params.alignment;
        accel = accel + (centroid * inv_n - pi) * params.cohesion;
    }
    // FORA do `neighbours > 0` -- o `crate::step` verbatim, e byte-identico com o
    // espaco pessoal desligado (somar `0 * separation` e' somar zero).
    accel = accel + sep * params.separation;
    accel = accel + (home - pi) * params.seek;
    // Reynolds' steering budget — the CPU's `norm()` verbatim: length, the EPS
    // early-out, then unit·budget in that order (divide, then multiply), so the
    // two routes agree to the bit and not merely to an epsilon.
    if (params.max_force > 0.0) {
        let steer_len = sqrt(dot(accel, accel));
        if (steer_len >= BOIDS_EPS && steer_len > params.max_force) {
            let unit = accel / steer_len;
            accel = unit * params.max_force;
        }
    }
    accel = accel + read_state_accel(i);
    // A massa inversa escala TUDO o que chega -- o impulso proprio e o do mundo --,
    // o espelho do `crate::step`. Peso `1` e' o bando de hoje AO BIT.
    accel = accel * w_i;
    var nv = vi + accel * dt;
    let speed = sqrt(dot(nv, nv));
    let min_speed = params.max_speed * clamp(params.speed_floor, 0.0, 1.0);
    if (speed > params.max_speed) {
        nv = nv / speed * params.max_speed;
    } else if (speed > BOIDS_EPS && speed < min_speed) {
        nv = nv / speed * min_speed;
    }
    var np = pi + nv * dt;
    if (!(boids_finite2(np) && boids_finite2(nv))) {
        np = home;
        nv = vec2<f32>(0.0, 0.0);
    }
    write_P(i, np);
    write_vel(i, nv);
    write_sim_t(i, params.playhead);
"#;

/// The registered kernel.
pub const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: WGSL,
    wgsl_lib: WGSL_LIB,
    bindings: BINDINGS,
    params: &[
        "seed",
        "radius",
        "separation",
        "separation_radius",
        "alignment",
        "cohesion",
        "seek",
        "max_speed",
        "max_force",
        // ⚠️ **Esta lista NÃO é derivada do manifesto**, e é por isso que ela é o
        // sítio que se esquece: um param novo compila, coza na CPU, e o device
        // recusa o shader com `invalid field accessor` — falha alta, ao menos, e
        // não silêncio.
        "fov",
        "speed_floor",
        "spread",
    ],
    count_law: Some(count_law),
    variant_by_param: None,
    // ⚠️ **A RECUSA existe porque a GRADE é a PERCEPÇÃO, e um espaço pessoal maior
    // que ela seria visto pela CPU e não pelo device.**
    //
    // O [`GRID`] declara `cell_param: "radius"`, e o kernel varre as 3×3 células em
    // volta — exactamente o conjunto dentro do `radius`, e **nada além dele**. Com
    // `separation_radius > radius` a CPU (all-pairs) acha vizinhos que a varredura
    // do device nem visita, e as duas rotas divergiriam **em silêncio**, sobre uma
    // cena que reivindica `fully_gpu`. É o mesmo modo de falha que o `inv_mass` do
    // pino quase produziu, com os dez gates de paridade verdes porque nenhum deles
    // punha o caso na fixture.
    //
    // ⚠️ **O preço está NOMEADO e é o caso útil:** medido, o que fecha a
    // sobreposição de 40 agentes de tamanho 1,0 é `separation_radius ≈ 4` contra um
    // `radius` de 2 — ou seja, exactamente a configuração que recua para a CPU.
    // Alargar a célula da grade ao `max(radius, separation_radius)` é a cura, e é
    // wave própria: o `GridSpec.cell_param` é UM param, e dar-lhe um segundo é
    // side-metadata foundational que atravessa os seis declarantes de grade.
    applicable: Some(|p| p("separation_radius") <= p("radius")),
};

/// The neighbourhood grid: over the state `P` (port 2, the `pre` feedback), cell
/// = the perception `radius`.
pub const GRID: GridSpec = GridSpec {
    column: "P",
    port: 2,
    cell_param: "radius",
    // One dispatch per cook: boids is a simulation STEP, so the TICK already is
    // the iteration — sweeping twice here would run two frames of flocking in one.
    sweeps_param: None,
};
