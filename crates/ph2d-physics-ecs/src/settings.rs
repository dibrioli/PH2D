//! [`PhysicsSettings`] — the world's authored settings (ADR-0131 D8).
//!
//! The components are the truth-at-rest for each BODY; this is the truth-at-rest
//! for the WORLD. Like them it is plain serde data, and like them the live
//! `PhysicsWorld` is *derived* from it — [`PhysicsBridge::set_settings`] is the
//! only thing that ever pushes it into rapier.
//!
//! ## Every range here was MEASURED (CLAUDE.md §0.0)
//!
//! A limit that only says "for safety" is a guess waiting for a smoke. Each
//! ceiling below names the resource it is a limit *of* and carries the table the
//! measurement produced (`tests/measure_settings.rs` in `ph2d-physics`, run with
//! `--release`; the harness is `#[ignore]`d so it never costs CI time).
//!
//! [`PhysicsBridge::set_settings`]: crate::PhysicsBridge::set_settings

use ph2d_physics::{BodyDefaults, LayerMatrix, MAX_LAYERS, PhysicsWorld};
use serde::{Deserialize, Serialize};

/// Gravity, per axis (m/s²).
///
/// Not a resource limit — the artist may legitimately want zero (top-down),
/// negative-Y (normal), or positive-Y (a world that falls up). The bound is on
/// what the fixed tick can still *depict*: W2a measured penetration as `v × dt`,
/// and at 100 m/s² a body is moving fast enough within half a second to be
/// inside the floor the frame it arrives no matter how the solver is tuned.
/// Ten Earth gravities is past every stylised game and short of the regime
/// where the tick itself is the artifact.
pub const GRAVITY_LIMIT: f32 = 100.0;

/// Integration sub-steps per tick.
///
/// **Resource: CPU, against HR-4's 1.5 ms.** Measured at 500 awake bodies —
/// perfectly linear, ~0.127 ms per sub-step:
///
/// | substeps | ms/tick | of HR-4 |
/// |---|---|---|
/// | 1 | 0.134 | 8.9% |
/// | 4 *(default)* | 0.511 | 34.1% |
/// | 8 | 1.017 | 67.8% |
/// | **12** | **1.529** | **101.9%** |
/// | 16 | 2.024 | 135.0% |
///
/// Twelve is where the stress scene consumes the entire frame budget, so it is
/// the last honest offer.
pub const MAX_SUBSTEPS: u32 = 12;

/// Solver iterations per step.
///
/// **Resource: CPU, same budget.** Measured at 500 awake bodies:
///
/// | iterations | ms/tick | of HR-4 |
/// |---|---|---|
/// | 4 *(default)* | 0.498 | 33.2% |
/// | 8 | 0.766 | 51.0% |
/// | **16** | **1.286** | **85.7%** |
/// | 24 | 1.808 | 120.5% |
///
/// Sixteen fits; twenty-four does not.
pub const MAX_SOLVER_ITERATIONS: u32 = 16;

/// Contact spring frequency (Hz) — how fast leftover penetration is pushed out.
///
/// **Resource: solver stability, and the first hypothesis was WRONG.** The
/// spring is integrated at `substep_dt`, so a Nyquist ceiling of
/// `1/(2·substep_dt)` = 120 Hz looked obvious. Measured, rapier's soft-constraint
/// formulation is stable far past it: a settled stack that is forbidden to sleep
/// drifts **exactly 0.0000 mm** at 30, 60, 120, 240, 480 and 960 Hz, and only at
/// **1920 Hz** does anything move (0.114 mm = 0.011 px at 100 px/m).
///
/// So the cap is not Nyquist. 480 sits one full octave inside the last
/// measured-clean value, and four times past where the knob's *effect* already
/// saturates: W2a measured recovery at 30 Hz = 9 frames and at 120 Hz = 1 frame,
/// and one frame is the floor — no frequency can beat it.
///
/// ⚠️ The zero-drift readings only became meaningful once the harness stopped
/// letting the stack **sleep**: a sleeping body is not integrated, so the first
/// run reported 0.0000 mm at every frequency including 1920, and that zero was
/// guaranteed rather than observed.
pub const MIN_CONTACT_HZ: f32 = 30.0;
/// See [`MIN_CONTACT_HZ`].
pub const MAX_CONTACT_HZ: f32 = 480.0;

/// Linear / angular drag.
///
/// **Bounds MEANING, not a resource.** rapier scales velocity by
/// `1/(1 + damping·dt)`, so terminal speed under gravity is ≈ `g/damping` and
/// the curve has no cliff — measured speed retained after one second of falling:
///
/// | damping | speed after 1 s | of free fall |
/// |---|---|---|
/// | 0 *(default)* | 9.81 m/s | 100% |
/// | 1 | 6.19 m/s | 63% |
/// | 5 | 1.95 m/s | 20% |
/// | **10** | **0.98 m/s** | **10%** |
/// | 20 | 0.49 m/s | 5% |
///
/// At 10 a body *drifts* rather than falls; past it every doubling only halves
/// an already-slow drift, so the knob would be choosing between shades of
/// "stopped".
pub const MAX_DAMPING: f32 = 10.0;

/// Air-drag coefficient — the world's "how thick is the air", lumping `½·ρ·Cd`.
///
/// **Resource: the sleep threshold.** Terminal fall speed is `√(mg/(k·L))`, so a
/// big enough `k` pushes a SMALL body's terminal speed under the sleep threshold
/// and it stops in mid-air — which reads as a bug, not as thick air. Measured
/// terminal speed after 10 s (free fall would be ~98 m/s):
///
/// | k | 0.28 m | 0.50 m | 1.00 m | 2.00 m | |
/// |---|---|---|---|---|---|
/// | 0.1 | 5.24 | 7.00 | 9.90 | 14.01 | noticeable |
/// | 1 | 1.66 | 2.21 | 3.13 | 4.43 | underwater |
/// | **10** | **0.52** | **0.70** | 0.99 | 1.40 | syrup — the last honest value |
/// | 20 | **0.00** | 0.50 | 0.70 | 0.99 | the small body FELL ASLEEP mid-air |
/// | 50 | **0.00** | **0.00** | 0.44 | 0.63 | two of them did |
pub const MAX_AIR_DRAG: f32 = 10.0;

/// Speed below which a body may fall asleep (m/s and rad/s).
///
/// Semantic, not resource: o tecto é *«mais rápido do que qualquer coisa a que se chame
/// repouso»*.
///
/// ⚠️ **A frase que aqui estava — «é comparado com a VELOCIDADE do corpo» — deixou de ser
/// verdade na `rapier2d` 0.35.** Ela compara agora uma **deriva de pose por passo** do ponto mais
/// afastado do corpo (translação + rotação dobrada lá dentro), e essa deriva inclui as **correcções
/// de posição do solver**, que nunca aparecem nas velocidades. O tecto de `10` continua folgado, o
/// que salva o número — mas a justificação que o derivava já não vale.
/// 10 m/s is a body crossing a 1000 px screen in one second at the project's
/// default 100 px/m — past that you are not describing rest, you are switching
/// the simulation off.
pub const MAX_SLEEP_THRESHOLD: f32 = 10.0;

/// **A sanidade do interruptor de adormecer** — a UMA porta pela qual o
/// [`PhysicsSettings::sleep_angular_threshold`] entra.
///
/// Duas leis, e a segunda é a que faltava:
///
/// 1. o lado **ligado** continua a ser clampado por [`MAX_SLEEP_THRESHOLD`] — um ficheiro
///    hand-editado não entrega ao solver um número que ninguém mediu;
/// 2. o lado **desligado** colapsa em [`ph2d_physics::SLEEP_SPIN_DISABLED`], em vez de ser
///    esmagado para `0.0`. ⛔ *O `clamp(0.0, MAX)` antigo tornava metade dos estados do campo
///    inalcançável* — a rapier lê o sinal, e a porta de entrada não deixava um sinal negativo
///    passar, logo *«nunca dorme»* não era exprimível de lado nenhum do produto.
///
/// ⚠️ `NaN` cai no ramo ligado (`>= 0.0` é falso para `NaN`… e por isso ele cai no ramo
/// DESLIGADO): explicitado aqui porque um `NaN` que chegasse à rapier como limiar faria toda
/// comparação de sono ser falsa — que é precisamente *«nunca dorme»*. O colapso é fiel.
#[must_use]
fn sanitize_sleep_spin(v: f32) -> f32 {
    if v >= 0.0 {
        v.min(MAX_SLEEP_THRESHOLD)
    } else {
        ph2d_physics::SLEEP_SPIN_DISABLED
    }
}

/// How long a body must stay under both thresholds before sleeping (seconds).
///
/// Semantic: dez segundos significam *«efectivamente nunca»* sem uma infinidade que o artista não
/// consiga voltar a escrever.
///
/// ⚠️ **A âncora que aqui estava caducou:** o default da rapier era `2 s` na 0.28 e é **`0,5 s`**
/// desde a 0.35, então dez segundos são hoje **20×**, não cinco. ⛔ Quem re-derivar este tecto da
/// frase antiga obtém `2,5`. O nosso `time_until_sleep` continua em `2,0 s` **por medição** — ver
/// `BodyDefaults::ours`.
pub const MAX_TIME_UNTIL_SLEEP: f32 = 10.0;

/// The world's authored physics settings.
///
/// **Append-only** (postcard is positional — ADR-0131's schema note): new knobs
/// go at the END, and `Default` must keep meaning "exactly what the engine did
/// before this field existed".
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsSettings {
    /// Gravity, m/s². Y-up, so Earth is `(0, -9.81)`.
    pub gravity_x: f32,
    /// See [`Self::gravity_x`].
    pub gravity_y: f32,
    /// Integration sub-steps per tick — the only lever on how deep a fast body
    /// is already overlapping the frame it first touches (W2a).
    pub substeps: u32,
    /// Solver iterations per step.
    pub solver_iterations: u32,
    /// Contact spring frequency, Hz — how fast leftover overlap is pushed out.
    pub contact_hz: f32,
    /// World-level drag applied to every body.
    pub linear_damping: f32,
    /// See [`Self::linear_damping`].
    pub angular_damping: f32,
    /// Speed below which a body may sleep.
    pub sleep_linear_threshold: f32,
    /// ⭐⭐ **O interruptor de adormecer, guardado como o `f32` que a rapier quer.**
    ///
    /// ⚠️ **Não é «a rotação abaixo da qual um corpo dorme» — essa frase caducou na `rapier2d`
    /// 0.35.** Para todo corpo com collider (isto é, toda entidade da cena) a rapier lê deste
    /// número **apenas o sinal**: `>= 0` = o corpo pode dormir · `< 0` = nunca dorme. O mecanismo,
    /// com o trecho da 0.35.3 ao lado, está em [`ph2d_physics::SLEEP_SPIN_DISABLED`].
    ///
    /// ⛔ **O campo fica `f32` e fica aqui** ainda que o produto o trate como um bit: ele viaja no
    /// `.ph2dproj` e o postcard é **posicional** — trocá-lo por um `bool` mudaria o significado de
    /// todos os bytes seguintes em cada ficheiro já gravado. Ler o sinal é gratuito e não parte
    /// nada: todo projecto gravado até 2026-08-30 tem aqui um valor `>= 0` (o `clamped` antigo
    /// forçava-o), que é exactamente *«pode dormir»*, que é o que aqueles projectos faziam.
    ///
    /// A porta é [`Self::sleep_enabled`] / [`Self::with_sleep_enabled`]; ⛔ não compare este campo
    /// com zero à mão, ou a lei fica escrita em dois sítios e deixa de ser lei.
    pub sleep_angular_threshold: f32,
    /// Time under both thresholds before a body sleeps, seconds.
    pub time_until_sleep: f32,
    /// Which layers collide with which — row `i`, bit `j`. **Appended**, and
    /// stored as raw rows rather than as [`LayerMatrix`] so the file format does
    /// not depend on a type's private shape.
    ///
    /// ⚠️ Read back through [`LayerMatrix::from_rows`], which **symmetrizes**: a
    /// hand-edited or future-build file must not be able to install a matrix the
    /// type says cannot exist (rapier ANDs both directions, so a half-set pair
    /// means "no collision" — a rule nobody wrote).
    pub layer_matrix: [u8; MAX_LAYERS],
    /// Air-drag coefficient. **Appended** — postcard is positional, so new
    /// fields go at the END and `Default` must keep meaning "what the engine
    /// did before this existed" (here: `0.0`, a vacuum).
    ///
    /// ⚠️ Not a tuning of [`Self::linear_damping`] — a different MODEL. Damping
    /// is a uniform velocity decay (mass cannot enter it); this is the drag
    /// equation, so it scales with cross-section and is resisted by mass. See
    /// `ph2d_physics::world::drag`.
    pub air_drag: f32,
}

impl Default for PhysicsSettings {
    /// **Exactly what the engine already did.** Every value is read from the
    /// engine's own constants rather than transcribed, so this cannot drift
    /// away from them in silence — the failure mode being avoided is a project
    /// that never opened the panel simulating differently than before the panel
    /// existed.
    fn default() -> Self {
        // ⚠️ `ours()`, não `rapier()`. Estes valores são PERSISTIDOS neste struct (ele é `serde`
        // e viaja no `.ph2dproj`), então lê-los de uma fonte que se move faria uma cena nova e um
        // projecto gravado terem tatos diferentes. Ver o doc de `BodyDefaults::ours`.
        let b = BodyDefaults::ours();
        Self {
            gravity_x: 0.0,
            gravity_y: PhysicsWorld::DEFAULT_GRAVITY_Y,
            substeps: PhysicsWorld::DEFAULT_SUBSTEPS,
            // rapier's own, and the wrapper does not override it (W2a measured
            // that raising contact DAMPING goes the wrong way, and left it be).
            solver_iterations: DEFAULT_SOLVER_ITERATIONS,
            contact_hz: PhysicsWorld::DEFAULT_CONTACT_HZ,
            linear_damping: b.linear_damping,
            angular_damping: b.angular_damping,
            sleep_linear_threshold: b.sleep_linear_threshold,
            sleep_angular_threshold: b.sleep_angular_threshold,
            time_until_sleep: b.time_until_sleep,
            layer_matrix: LayerMatrix::all().rows(),
            air_drag: 0.0,
        }
    }
}

/// rapier's own solver iteration count — `4` na 0.28 e na 0.35.
///
/// ⚠️ **A 2.ª metade da frase antiga era falsa e contradizia este próprio ficheiro:** ela dizia
/// que *«o `PhysicsWorld` nunca chama `set_solver_iterations`»*, e o `apply_to` mais abaixo
/// chama-o. Isto é o valor de PARTIDA, não o que corre sempre.
pub const DEFAULT_SOLVER_ITERATIONS: u32 = 4;

impl PhysicsSettings {
    /// Clamp every field into its documented range.
    ///
    /// Called on the way IN (from the panel, and from a loaded project file),
    /// because a range that only lives in the slider is not a range: a project
    /// saved by a future build, or a hand-edited one, would otherwise hand the
    /// solver a number nothing measured.
    pub fn clamped(self) -> Self {
        Self {
            gravity_x: self.gravity_x.clamp(-GRAVITY_LIMIT, GRAVITY_LIMIT),
            gravity_y: self.gravity_y.clamp(-GRAVITY_LIMIT, GRAVITY_LIMIT),
            substeps: self.substeps.clamp(1, MAX_SUBSTEPS),
            solver_iterations: self.solver_iterations.clamp(1, MAX_SOLVER_ITERATIONS),
            contact_hz: self.contact_hz.clamp(MIN_CONTACT_HZ, MAX_CONTACT_HZ),
            linear_damping: self.linear_damping.clamp(0.0, MAX_DAMPING),
            angular_damping: self.angular_damping.clamp(0.0, MAX_DAMPING),
            sleep_linear_threshold: self.sleep_linear_threshold.clamp(0.0, MAX_SLEEP_THRESHOLD),
            // ⚠️ **Sanidade que PRESERVA O SINAL**, e não um `clamp(0.0, ..)`. O clamp antigo
            // tornava o lado «nunca dorme» inalcançável — o campo tinha dois estados e a porta de
            // entrada só deixava passar um deles. Ver o doc do campo.
            sleep_angular_threshold: sanitize_sleep_spin(self.sleep_angular_threshold),
            time_until_sleep: self.time_until_sleep.clamp(0.0, MAX_TIME_UNTIL_SLEEP),
            air_drag: self.air_drag.clamp(0.0, MAX_AIR_DRAG),
            // Symmetrized, not merely copied — see the field docs.
            layer_matrix: LayerMatrix::from_rows(self.layer_matrix).rows(),
        }
    }

    /// ⭐ **Podem os corpos deste mundo adormecer?** — a única pergunta que a `rapier2d` 0.35 faz
    /// ao [`Self::sleep_angular_threshold`], e portanto a única que o painel pode oferecer.
    ///
    /// ⚠️ **Um `>= 0.0`, não um `> 0.0`:** é o teste que a rapier faz, literalmente. Um limiar de
    /// `0,0` significa *«pode dormir»* lá dentro (a barra angular é o `π/2` fixo dela), e um
    /// `> 0.0` aqui leria ao contrário um projecto gravado com o slider no mínimo.
    #[must_use]
    pub fn sleep_enabled(self) -> bool {
        self.sleep_angular_threshold >= 0.0
    }

    /// O sentido inverso de [`Self::sleep_enabled`] — o que o interruptor do painel escreve.
    ///
    /// ⚠️ **Ligar repõe o default do produto, não a magnitude anterior.** Guardar a magnitude para
    /// a devolver seria lembrar um número que **nenhum consumidor lê** (ver o doc do campo): a
    /// única coisa que sobreviveria o ciclo desligar→ligar é um valor invisível. Desligar escreve
    /// [`ph2d_physics::SLEEP_SPIN_DISABLED`], que é o que a própria rapier escreve em
    /// `RigidBodyActivation::cannot_sleep()`.
    #[must_use]
    pub fn with_sleep_enabled(self, on: bool) -> Self {
        Self {
            sleep_angular_threshold: if on {
                // O default do PRODUTO (`BodyDefaults::ours`), não o da rapier: é o valor que
                // todo `.ph2dproj` gravado já carrega, então ligar devolve o mundo ao que era.
                BodyDefaults::ours().sleep_angular_threshold
            } else {
                ph2d_physics::SLEEP_SPIN_DISABLED
            },
            ..self
        }
    }

    /// The body-level half of these settings.
    pub(crate) fn body_defaults(self) -> BodyDefaults {
        BodyDefaults {
            linear_damping: self.linear_damping,
            angular_damping: self.angular_damping,
            sleep_linear_threshold: self.sleep_linear_threshold,
            sleep_angular_threshold: self.sleep_angular_threshold,
            time_until_sleep: self.time_until_sleep,
        }
    }

    /// Push every value into a world.
    ///
    /// **The single door**, and the reason it exists: a fresh `PhysicsWorld` is
    /// built in three places (construction, `rebuild`, `rebuild_from_rest`), and
    /// each one starts from the engine defaults. Before this, only gravity was
    /// carried across a rewind — so a scrub backwards would have silently reset
    /// every other knob the artist had touched.
    pub(crate) fn apply_to(self, world: &mut PhysicsWorld) {
        world.set_gravity(self.gravity_x, self.gravity_y);
        world.set_substeps(self.substeps);
        world.set_solver_iterations(self.solver_iterations as usize);
        world.set_contact_frequency(self.contact_hz);
        world.set_body_defaults(self.body_defaults());
        world.set_air_drag(self.air_drag);
        world.set_layer_matrix(LayerMatrix::from_rows(self.layer_matrix));
    }
}
