#![forbid(unsafe_code)]
//! **`sim.step`** — one integration step INSIDE a simulation zone (Motion Nodes O4, doc 48).
//!
//! ## Why this is not `motion.integrate`
//!
//! `motion.integrate` is a stateful node: it holds the simulation itself, pairs each element of
//! a LIVE rest chain with its own prior row by `id`, and adds the accumulated displacement to a
//! position the rest of the graph keeps re-authoring. It is the right node when the sim is a
//! *deviation from an animation*.
//!
//! Inside a zone there is no rest chain. **The stream IS the state** — it came out of the zone,
//! it goes back into the zone, and its positions are the truth. So the step is *stateless*:
//! read `vel`/`accel` off the stream, write `P`/`vel` back. It is Blender's `Set Position` with
//! `velocity × Delta Time`, which is exactly what their simulation zone makes you write by hand.
//!
//! Putting `motion.integrate` inside a zone would give the sim TWO memories — the zone's and the
//! integrator's own `pre` self-loop — and they would disagree the moment a kill node removed a
//! row. One state, one owner: the zone.
//!
//! ## Semi-implicit Euler, and the clock the state carries
//!
//! `vel += accel·dt·w` first, then `P += vel·dt·w` with the NEW velocity (symplectic — stable
//! under oscillatory forces; the same integrator `motion.integrate` uses, for the same reason).
//! `w` is `inv_mass`, so `w = 0` is a hard pin.
//!
//! **`dt` comes from a clock column the state carries** (`sim_t`), not from a frame counter: an
//! element that has never been stepped has no `sim_t`, so its `dt` is 0 and it simply starts.
//! Without that, a zone dropped onto a graph at playhead = 8 s would take one 8-second step and
//! fling everything to infinity on the frame it was created. The clamp (`MAX_DT`) does the same
//! job for a scrub.
//!
//! `accel` is CONSUMED (dropped): it is the transient the `force.*` nodes accumulate, and a step
//! that left it on the stream would re-apply the same force forever, one tick out of date.
//!
//! ## It also keeps the `age`
//!
//! The step owns the clock, so the step owns the ageing: every element's `age` grows by the same
//! `dt` its motion did (doc 50). An element with no `age` yet is newborn — it starts at zero.
//! Nothing else in the library could do this honestly: a node without the sim's own clock would
//! have to guess the frame rate, and the age is what `sim.lifetime` kills by and what
//! `value.attribute` hands to a colour ramp.

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// **The largest step the integrator will take, in seconds — MEDIDO** (bloco Z, doc 91).
///
/// Uma pausa, um scrub ou um quadro perdido chegariam como um `dt` enorme, e o sim explodiria no
/// quadro em que retoma.
///
/// ⚠️ **Era `1/20`, e este era o número MAIS CERTO dos dois do catálogo.** A sonda
/// `measure_the_step_that_a_closed_loop_survives` (`ph2d-node-registry-init`) corre a MESMA
/// malha fechada nos dois integradores — `grid → zona`, com o `pre` de volta por uma
/// `force.attractor` — e mede a excursão de uma grelha nascida dentro de raio `1,0`:
///
/// | `strength` = 40 | dt=1/60 | dt=1/30 | dt=0,05 | dt=0,075 | dt=0,1 |
/// |---|---|---|---|---|---|
/// | `sim.zone`/`sim.step` (grampo 0,05) | 0,83 | 0,89 | 2,49 | 2,48 | 2,48 |
/// | `motion.integrate` (grampo 0,1) | 0,83 | 0,89 | 4,43 | 33,48 | **127,19** |
///
/// As duas linhas são **idênticas até 1/30** — é o mesmo Euler semi-implícito —, e o que as
/// separa dali para a frente é só o grampo. O de `0,05` segurava a cena em `2,49`; o de `0,1`
/// deixava-a chegar a `127`.
///
/// **O número novo é o JOELHO medido**, e o critério está escrito: *um passo legítimo não muda a
/// RESPOSTA, só a resolução*. A barra é o dobro da excursão em regime, e o maior `dt` em que
/// **todo** passo até ele fica dentro dela é `0,0300`. A `0,05` esta malha já mede `2,49` contra
/// uma barra de `1,66`.
///
/// ⚠️ **Em regime isto é byte-idêntico** (o tique fixo é `1/60`); o que muda é só quanto de um
/// scrub a zona absorve.
const MAX_DT: f32 = 0.03;

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("sim.step"),
    name: "sim.step",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Reads the playhead (it is the clock the step is measured against), holds no state of its
    // own — the zone holds it.
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        // Velocity retained per second: 1 = frictionless (the default, and bit-identical to no
        // damping at all), 0.1 = molasses. Applied linearly per step (`1 - (1-damping)·dt`)
        // rather than as `damping^dt`: same thing to first order, and transcendental-free (HR-5).
        //
        // ⚠️ **1,0 é o teto do que o kernel HONRA, e a razão é o sinal de `keep`:** acima dele
        // `keep = 1 - (1-damping)·dt` passa de 1 e o passo **INJETA** energia em vez de tirar.
        // Medido na cena `=31` com `damping = 5` (`keep` = 1,0667): pico 348 u/s a 1 s, 16.744
        // a 2 s, 2,06e14 a 8 s e **`inf`** aos 12 s. **A UI já o impede** — o `max` do hint é
        // 1,0 e um param sem `ParamHardMax` tem a caixa de texto capada nele —, então o valor
        // disfuncional só é alcançável por CÓDIGO (um documento montado por `set_param`, que é
        // o que toda cena de demo é). Não há teto novo a declarar: `ParamHardMax` só ALARGA a
        // faixa digitável (`must be >= the hint's max`), e usá-lo aqui seria um no-op vestindo
        // a palavra "teto".
        ParamSpec {
            name: "damping",
            default: 1.0,
        },
        // ── O LIMITE DE VELOCIDADE (doc 89, folha 13 P1) ────────────────────────
        // APENDADOS, nunca inseridos: a lista de params é lida posicionalmente por quem guarda
        // um índice, e `damping` não pode renumerar.
        //
        // **`0` é DESLIGADO nos dois**, não "congele" — é a leitura que a própria folha propôs
        // (`max_speed = ∞`, ou `0` = desligado) e a do `Force Limit` do Niagara. Um teto de
        // exatamente zero seria um pedido que o `damping` já atende melhor.
        ParamSpec {
            name: "max_speed",
            default: 0.0,
        },
        ParamSpec {
            name: "min_speed",
            default: 0.0,
        },
        // ── O ESTADO ANGULAR (doc 89, folha 13) ─────────────────────────────────
        // APENDADO. `1` = sem arrasto angular, exactamente como o `damping` linear — e a
        // metade angular inteira só corre quando a coluna `spin` EXISTE, então um estado
        // que nunca a autorou é bit-idêntico ao de antes deste param. Ver [`ANGULAR`].
        ParamSpec {
            name: ANGULAR,
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// **O limite de velocidade** ([`MANIFEST`] `max_speed`/`min_speed`, doc 89 folha 13 P1) — a
/// PORTA ÚNICA que o `eval` e o [`GPU_KERNEL`] perguntam, termo por termo.
///
/// `0` desliga cada metade. Fora isso o vetor é reescalado para o comprimento pedido, o que
/// preserva a DIREÇÃO — um limite que mexesse na direção não seria um limite, seria uma força.
///
/// ⚠️ **O TETO VENCE O PISO**, e a ordem das duas linhas é a lei: um piso capaz de empurrar um
/// elemento através do teto tornaria o teto uma sugestão. Com `min > max` o resultado é `max`.
///
/// ⚠️ **Um elemento PARADO não tem direção**, então o piso não o acorda — `v = 0` é o caso
/// degenerado e a resposta honesta é deixá-lo onde está, e não escolher um rumo por ele (o mesmo
/// que o `sim.collide` faz no centro exato de um disco). A consequência que o artista vê: um
/// piso de velocidade **não** levanta o que já assentou; o que ele faz é impedir que o que se
/// move pare.
///
/// ⚠️ **E ele NÃO é um nó novo, de propósito.** Um `sim.speed_limit` a jusante rodaria DEPOIS de
/// o passo já ter avançado a posição, então ele capparia o número que o elemento *reporta* e não
/// a distância que ele *andou* — um tique atrasado, por construção. Só quem tem a velocidade e o
/// `dt` na mão no meio do passo consegue capar as duas; é por isso que isto é um param do
/// integrador (a colocação do `Limit Force` do Niagara: depois das forças, antes do solve). E
/// custa o oitavo escritor de `vel` a menos, num catálogo que tem exatamente sete.
pub fn limit_speed(v: [f32; 2], min_speed: f32, max_speed: f32) -> [f32; 2] {
    if min_speed <= 0.0 && max_speed <= 0.0 {
        return v;
    }
    let sp = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if sp <= f32::EPSILON {
        return v;
    }
    let mut want = sp;
    if min_speed > 0.0 {
        want = want.max(min_speed);
    }
    if max_speed > 0.0 {
        want = want.min(max_speed);
    }
    if want == sp {
        return v;
    }
    let k = want / sp;
    [v[0] * k, v[1] * k]
}

/// The WGSL port of [`step`], element for element (ADR-0135 — the sim-zone
/// family on the GPU). Unlike `motion.integrate` there is no `rest` chain and no
/// id-gather: inside a zone **the stream IS the state**, so this is a single-port
/// kernel reading its own columns and writing them back.
///
/// **The clock is per-element** (`read_sim_t(i)`, not element 0's): a zone that
/// spawns carries newborns with no `sim_t` beside veterans that have one, and the
/// step ages each by *its own* `dt`. When the column is absent entirely,
/// `HAS_sim_t` is false and every element falls back to `params.playhead` ⇒
/// `dt = 0` ⇒ a fresh element STARTS rather than leaping (the `scalar_col` →
/// `None` → `0.0` path on the CPU).
///
/// **The finiteness guard keeps the OLD value**, not a zero: a diverged element
/// resets to where it was rather than NaN-poisoning the zone — so `out_*` seed
/// from `read_*(i)` and are only overwritten when the new state is finite, exactly
/// as the CPU's `if …all(is_finite) { vel[i] = v; p[i] = q; }`.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let st_t_prev = select(params.playhead, read_sim_t(i), HAS_sim_t);\n\
        let st_dt = clamp(params.playhead - st_t_prev, 0.0, STEP_MAX_DT);\n\
        let st_w = read_inv_mass(i);\n\
        var st_v = read_vel(i);\n\
        var st_q = read_P(i);\n\
        let st_a = read_accel(i);\n\
        // Semi-implicit: velocity first, then position with the NEW velocity.\n\
        st_v = st_v + st_a * st_dt * st_w;\n\
        // Linear damping — first-order equivalent of `damping^dt`, transcendental-free.\n\
        let st_keep = 1.0 - (1.0 - params.damping) * st_dt;\n\
        st_v = st_v * st_keep;\n\
        // O LIMITE, entre a velocidade e a posicao: ele capa a distancia que o\n\
        // elemento ANDA neste tique, nao so' o numero que ele reporta depois.\n\
        // Termo por termo com `limit_speed`; o teto vence o piso pela ORDEM.\n\
        if (params.min_speed > 0.0 || params.max_speed > 0.0) {\n\
        \x20   let st_sp = sqrt(dot(st_v, st_v));\n\
        \x20   if (st_sp > STEP_EPS) {\n\
        \x20       var st_want = st_sp;\n\
        \x20       if (params.min_speed > 0.0) { st_want = max(st_want, params.min_speed); }\n\
        \x20       if (params.max_speed > 0.0) { st_want = min(st_want, params.max_speed); }\n\
        \x20       if (st_want != st_sp) { st_v = st_v * (st_want / st_sp); }\n\
        \x20   }\n\
        }\n\
        st_q = st_q + st_v * st_dt * st_w;\n\
        var st_out_v = read_vel(i);\n\
        var st_out_q = read_P(i);\n\
        if (step_finite(st_v) && step_finite(st_q)) {\n\
        \x20   st_out_v = st_v;\n\
        \x20   st_out_q = st_q;\n\
        }\n\
        write_P(i, st_out_q);\n\
        write_vel(i, st_out_v);\n\
        // A METADE ANGULAR -- `spin_step` termo a termo. ⚠️ Guardada pelo `HAS_spin`: sem\n\
        // a coluna nao ha' escrita nenhuma, e o quadro sai bit-identico ao de antes deste\n\
        // param. Escrever aqui incondicionalmente CUNHARIA `spin`/`rot` em todo estado.\n\
        if (HAS_spin) {\n\
        \x20   let st_s0 = read_spin(i);\n\
        \x20   let st_s = st_s0 * (1.0 - (1.0 - params.angular_damping) * st_dt);\n\
        \x20   let st_r = read_rot(i) + st_s * st_dt;\n\
        \x20   if (abs(st_s) <= STEP_F32_MAX && abs(st_r) <= STEP_F32_MAX) {\n\
        \x20       write_spin(i, st_s);\n\
        \x20       write_rot(i, st_r);\n\
        \x20   }\n\
        }\n\
        // The step owns the clock, so the step owns the ageing (doc 50).\n\
        write_age(i, read_age(i) + st_dt);\n\
        write_sim_t(i, params.playhead);\n",
    wgsl_lib: "\
        // ⚠️ O gémeo do `MAX_DT` do Rust, MEDIDO — a tabela está no doc-comment dele.\n\
        // Os dois literais movem-se juntos ou o gate de paridade CPU/GPU acusa.\n\
        const STEP_MAX_DT: f32 = 0.03;\n\
        // `f32::EPSILON`, o mesmo guarda de direcao que o `sim.collide` usa no\n\
        // centro exato de um disco: sem direcao nao ha' o que reescalar.\n\
        const STEP_EPS: f32 = 1.1920929e-7;\n\
        // WGSL has no `isFinite`; `abs(x) <= F32_MAX` is exactly \"not NaN and not\n\
        // infinite\" (every compare against NaN is false, an inf exceeds the max),\n\
        // per-lane rather than through `max()` whose NaN behaviour is undefined.\n\
        const STEP_F32_MAX: f32 = 3.4028235e38;\n\
        fn step_finite(v: vec2<f32>) -> bool {\n\
        \x20   return abs(v.x) <= STEP_F32_MAX && abs(v.y) <= STEP_F32_MAX;\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "vel",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        // The state's own clock, from which `dt` is derived — per element, like
        // the CPU's `t_prev[i]`. Absent → `HAS_sim_t` false → `params.playhead`
        // (`dt = 0`), matching `scalar_col`'s `None`.
        ColumnBinding {
            column: "sim_t",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        // A row with no `age` is newborn: identity 0.
        ColumnBinding {
            column: "age",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        // **O ESTADO ANGULAR** (doc 89 folha 13). Ausente ⇒ `HAS_spin` falso ⇒ a metade
        // angular nem corre, e é isso que mantém um estado sem giro byte a byte.
        ColumnBinding {
            column: "spin",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        // O ângulo que o `spin` move. Ausente = `0°`, que é o que a `identity` diz e o que a
        // CPU assume — as duas metades leem a mesma ausência da mesma maneira.
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        // Transient: eaten here so every tick starts from zero acceleration —
        // without this the same force would re-apply forever, one tick out of date.
        ColumnBinding {
            column: "accel",
            dim: Dim::Vec2,
            access: ColumnAccess::Consume,
            identity: [0.0; 4],
            port: 0,
        },
        // Absent = every element free (`w = 1`); `·1.0` is exact, so a pre-pin
        // graph integrates identically.
        ColumnBinding {
            column: "inv_mass",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
    ],
    params: &["damping", "max_speed", "min_speed", ANGULAR],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

fn vec2_col(s: &Stream, name: &str, n: usize) -> Vec<[f32; 2]> {
    match s.get(name) {
        Some(Column::Vec2(v)) if v.len() == n => v.clone(),
        _ => vec![[0.0, 0.0]; n],
    }
}

fn scalar_col(s: &Stream, name: &str, n: usize, default: f32) -> Option<Vec<f32>> {
    match s.get(name) {
        Some(Column::Scalar(v)) if v.len() == n => Some(v.clone()),
        Some(_) => Some(vec![default; n]),
        None => None,
    }
}

/// One step of the whole zone's state, as a pure function — the node.
/// O nome do param do arrasto angular, e o da coluna que ele amortece.
const ANGULAR: &str = "angular_damping";
/// **A VELOCIDADE ANGULAR**, em GRAUS por segundo — a mesma unidade do `rot` que ela move.
///
/// ⚠️ **Quem a AUTORA já existe:** o `motion.drive` tem o canal `Custom…`, que escreve
/// qualquer coluna pelo nome. `drive(Custom, "spin", Set)` é o *POP Spin* (cada peça gira à
/// sua taxa) e `drive(Custom, "spin", Add)` é o *POP Torque* (um empurrão angular que se
/// acumula). *Não era preciso um canal novo — era preciso alguém INTEGRAR o que ele escreve.*
const SPIN: &str = "spin";
/// O ângulo que a coluna [`SPIN`] move.
const ROT: &str = "rot";

/// **A METADE ANGULAR DO PASSO** — a única porta, portada termo a termo para o kernel.
///
/// `spin *= keep; rot += spin · dt`, com `keep = 1 − (1 − angular_damping)·dt`: a MESMA forma
/// de primeira ordem do amortecimento linear, e pelo mesmo motivo (HR-5, sem transcendentes).
/// A ordem é a semi-implícita do irmão linear — amortece primeiro, integra com o valor NOVO.
///
/// ⚠️ **Em `angular_damping = 1` o factor é exactamente `1,0`**, então uma peça que gira sem
/// arrasto gira com os mesmos bits que giraria sem este param existir.
fn spin_step(spin: f32, rot: f32, dt: f32, angular_damping: f32) -> (f32, f32) {
    let s = spin * (1.0 - (1.0 - angular_damping) * dt);
    (s, rot + s * dt)
}

fn step(
    state: &Stream,
    playhead: f32,
    damping: f32,
    min_speed: f32,
    max_speed: f32,
    angular_damping: f32,
) -> Stream {
    let n = state.count();
    let mut out = Stream::new(n);
    // Everything the sim does not own rides through untouched — `id` above all, so a kill node
    // downstream can tell the survivors apart next tick. `accel` is consumed.
    // ⚠️ **A metade angular só existe se alguém autorou `spin`** — sem a coluna, nada aqui
    // muda e a saída é a de sempre, bit a bit. É isso que torna o param novo inerte por
    // omissão em vez de «neutro por um valor».
    let spin_prev = scalar_col(state, SPIN, n, 0.0);
    let spinning = spin_prev.is_some();
    for (name, col) in state.columns() {
        let owned = matches!(name.as_str(), "accel" | "P" | "vel" | "sim_t" | "age")
            || (spinning && matches!(name.as_str(), SPIN | ROT));
        if !owned {
            out.set(name.clone(), col.clone());
        }
    }
    // The age grows by the same `dt` the motion did — the step owns the clock, so the step owns
    // the ageing. A row with no `age` is newborn: it starts at zero.
    let age_prev = scalar_col(state, "age", n, 0.0).unwrap_or_else(|| vec![0.0; n]);

    let mut p = vec2_col(state, "P", n);
    let mut vel = vec2_col(state, "vel", n);
    let accel = vec2_col(state, "accel", n);
    let w = scalar_col(state, "inv_mass", n, 1.0).unwrap_or_else(|| vec![1.0; n]);
    // No clock on the state = these elements have never been stepped: `dt` is 0, they start.
    let t_prev = scalar_col(state, "sim_t", n, playhead);

    for i in 0..n {
        let dt = t_prev
            .as_ref()
            .map(|t| (playhead - t[i]).clamp(0.0, MAX_DT)) // CLAMP-OK: const bounds, min < max
            .unwrap_or(0.0);
        let (mut v, mut q, a, wi) = (vel[i], p[i], accel[i], w[i]);
        // Semi-implicit: velocity first, then position with the NEW velocity.
        v[0] += a[0] * dt * wi;
        v[1] += a[1] * dt * wi;
        // Linear damping — first-order equivalent of `damping^dt`, and transcendental-free
        // (HR-5). At `damping = 1` it is exactly `·1.0`: an undamped sim is bit-identical.
        let keep = 1.0 - (1.0 - damping) * dt;
        v[0] *= keep;
        v[1] *= keep;
        // O LIMITE, aqui e não depois: ele capa a DISTÂNCIA que o elemento anda neste tique.
        v = limit_speed(v, min_speed, max_speed);
        q[0] += v[0] * dt * wi;
        q[1] += v[1] * dt * wi;
        // A diverged element resets rather than freezing (or NaN-poisoning) the whole zone.
        if v.iter().chain(&q).all(|x| x.is_finite()) {
            vel[i] = v;
            p[i] = q;
        }
    }

    let age: Vec<f32> = (0..n)
        .map(|i| {
            let dt = t_prev
                .as_ref()
                .map(|t| (playhead - t[i]).clamp(0.0, MAX_DT)) // CLAMP-OK: const bounds, min < max
                .unwrap_or(0.0);
            age_prev[i] + dt
        })
        .collect();

    if let Some(spin0) = spin_prev {
        let rot0 = scalar_col(state, ROT, n, 0.0).unwrap_or_else(|| vec![0.0; n]);
        let mut spin = spin0;
        let mut rot = rot0;
        for i in 0..n {
            let dt = t_prev
                .as_ref()
                .map(|t| (playhead - t[i]).clamp(0.0, MAX_DT)) // CLAMP-OK: const bounds
                .unwrap_or(0.0);
            let (s, r) = spin_step(spin[i], rot[i], dt, angular_damping);
            // A mesma rede do irmão linear: uma peça que divergiu repõe-se em vez de
            // envenenar a zona inteira com `NaN`.
            if s.is_finite() && r.is_finite() {
                spin[i] = s;
                rot[i] = r;
            }
        }
        out.set(SPIN, Column::Scalar(spin));
        out.set(ROT, Column::Scalar(rot));
    }

    out.set("P", Column::Vec2(p));
    out.set("vel", Column::Vec2(vel));
    out.set("age", Column::Scalar(age));
    out.set("sim_t", Column::Scalar(vec![playhead; n]));
    out
}

struct SimStep;

impl NodeOp for SimStep {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let playhead = ctx.playhead() as f32;
        let damping = ctx.param("damping");
        // Negativo é lido como desligado pela porta única — um limite de velocidade negativo não
        // é um pedido, e recusá-lo lá vale para os dois caminhos.
        let out = step(
            ctx.input(0),
            playhead,
            damping,
            ctx.param("min_speed"),
            ctx.param("max_speed"),
            ctx.param(ANGULAR),
        );
        ctx.emit(out);
    }
}

pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(SimStep))?;
    // ADR-0155: the particle integrator CONSUMES `accel` and reads `inv_mass` — the
    // sibling of `motion.integrate` for a `sim.spawn` chain.
    reg.register_couplings(
        MANIFEST.id,
        &[
            ph2d_node_registry::Coupling::Consumes("accel"),
            ph2d_node_registry::Coupling::Consumes("inv_mass"),
        ],
    );
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Simulation Step",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

/// Param UI hints.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "damping",
        label: "Damping",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // **O ARRASTO ANGULAR** (doc 89, folha 13). A MESMA faixa e a MESMA escada do irmão
    // linear, e pelo mesmo motivo: `1` é sem arrasto e `0` é o mais forte que a lei de
    // primeira ordem consegue. ⚠️ **Um teto acima de `1` INJETARIA giro** — `keep` passaria de
    // 1 —, que é o defeito que o doc do `damping` mede e nomeia logo acima.
    //
    // ⚠️ **Sem este hint o param existiria e não seria DESENHADO**, e foi assim que ele nasceu:
    // três gates apanharam-no de uma vez (`every_param_spec_carries_a_hint_or_is_folded_into_a_swatch`,
    // `every_scalar_row_comes_from_a_declared_hint`, `every_declared_param_is_drawn_by_some_widget`).
    // *A rede do knob morto desta casa funciona, e ela é uma rede de TRÊS.*
    ParamUiHint {
        param: ANGULAR,
        label: "Angular Damping",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **A faixa de arrasto é MEDIDA** (`probe_natural_speeds`, §0): em 4 s de queda livre sob
    // as gravidades que as cenas do corpus usam (5 / 6 / 8) a velocidade chega a **20 / 24 / 32
    // u/s**, então 40 cobre todas com folga e ainda deixa o slider útil no meio do curso.
    //
    // ⚠️ **E NÃO há `ParamHardMax` aqui, de propósito:** um teto maior é simplesmente MENOS teto —
    // ele degrada para "desligado", que é o default. O que teria um limite disfuncional é o PISO
    // (velocidade alta o bastante para atravessar um colisor num tique), e esse número é do
    // COLISOR (`2·raio / dt`), não deste nó — declará-lo aqui seria inventar um teto de um recurso
    // que este nó não possui.
    ParamUiHint {
        param: "max_speed",
        label: "Speed Limit",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "min_speed",
        label: "Min Speed",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
];

/// ⚠️ **`Length` num par que é metros por SEGUNDO, e a escolha é a menos errada das duas.**
///
/// A tabela de unidades não tem velocidade, e só `ParamUnit::Length` atravessa o
/// `pixels_per_meter`. Declarar `None` deixaria o número CRU: o artista que trabalha em pixels
/// digitaria `500` e o solver guardaria 500 m/s numa cena de 100 px/m — cem vezes o pedido.
/// Declarando `Length` o número converte exatamente certo (o metro do numerador é o que escala) e
/// o que fica uma casa grosseira é o RÓTULO, que mostra a unidade de comprimento do artista onde
/// devia dizer "por segundo".
///
/// **O número certo com o rótulo grosso vence o rótulo certo com o número errado** — e o vão fica
/// NOMEADO: um `ParamUnit::Speed` exigiria um sufixo COMPOSTO (`<unidade>/s`), coisa que nenhuma
/// unidade desta tabela tem hoje, então ele é uma wave da fronteira de display e não um variant.
static PARAM_UNITS: &[ph2d_node_registry::ParamUnitDecl] = &[
    ph2d_node_registry::ParamUnitDecl {
        param: "max_speed",
        unit: ph2d_node_registry::ParamUnit::Length,
    },
    ph2d_node_registry::ParamUnitDecl {
        param: "min_speed",
        unit: ph2d_node_registry::ParamUnit::Length,
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "spin_tests.rs"]
mod spin_tests;
