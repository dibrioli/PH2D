//! **Os ganhos MEDIDOS de um motor** — os três números que fazem um motor
//! cumprir o que a row diz, e as varreduras que os escolheram.
//!
//! Irmão de `joints.rs`, separado dele quando os dois juntos passaram do cap de
//! 700 LOC. O corte não é de tamanho: são **120 linhas de tabela de medição** ao
//! lado de uma função que constrói joints, e elas respondem uma pergunta
//! diferente — *por que estes números e não outros*. Cada tabela é reproduzível
//! contra o caminho de PRODUTO (`spawn_joint_tuned`), que é o motivo de aquela
//! porta existir.

/// How hard a motor corrects its velocity error — rapier's motor `damping`,
/// with `stiffness` at zero, which is what makes it a *velocity* motor.
///
/// **Not a knob, and MEASURED.** It is not exposed because it is not a
/// question the artist has: they say how fast and how strong, and this is the
/// number that makes those two mean what they say. Too low and the motor
/// cannot hold its speed under load — it is a suggestion, not a motor.
///
/// A 0.2 kg arm hanging straight down, told to turn at 4 rad/s for 5 s. A
/// motor that keeps its word travels 20 rad; one that cannot lift the arm past
/// horizontal stalls near zero:
///
/// | tracking | `max_force` 0.1 | 1.0 | 100 |
/// |---|---|---|---|
/// | 10  | 0.49 | 1.39 | 1.39 |  ← cannot lift its own arm
/// | 50  | 0.49 | 18.57 | 18.90 |
/// | **100** | **0.49** | **19.21** | **19.62** |  ← the knee
/// | 300 | 0.49 | 19.37 | 19.90 |
/// | 1000 | 0.49 | 19.39 | 19.97 |
///
/// The first column is the point: at every tracking value a motor capped at
/// 0.1 N·m still stalls. Raising this does not make motors unstoppable — it
/// makes a motor that *is* strong enough actually reach its speed.
///
/// ⚠️ **RE-MEASURED at 1000 when the LINEAR motor arrived (W-J6), and the old
/// 100 was visibly wrong there.** A velocity motor is a damping term, so working
/// against gravity it settles a fixed `g / tracking` SHORT of what it was told —
/// a shortfall that is 2.6% of a hinge's 4 rad/s and **20% of a rail's 0.5 m/s**,
/// because the two defaults are small numbers in different units. Told 0.5 m/s up
/// a vertical rail:
///
/// | tracking | achieved | shortfall |
/// |---|---|---|
/// | 100 | 0.4019 m/s | 0.0981 |  ← a fifth of the speed, silently
/// | 300 | 0.4674 m/s | 0.0326 |
/// | **1000** | **0.4903 m/s** | **0.0097** |
/// | 3000 | 0.4967 m/s | 0.0033 |
/// | 10000 | 0.4990 m/s | 0.0010 |
///
/// 1000 is where the shortfall stops being something you can see (2%), and the
/// angular table above already measured it as *better* on that side too
/// (19.39/19.97 travelled against 19.21/19.62). Both columns of the stall test
/// still read 0.49 at 1000, so the `max_force` ceiling keeps meaning what it
/// says. Reproduce with `cargo test -p ph2d-physics linear_motor_tracking_sweep
/// -- --ignored --nocapture`.
///
/// ⚠️ This MOVES the pose of every existing scene with a hinge motor (they now
/// reach their stated speed), and therefore the `physics_ecs_c9` hash.
pub(super) const MOTOR_TRACKING: f32 = 1000.0;

/// How hard a **servo** pulls towards its target — rapier's motor `stiffness`,
/// the number that makes [`MotorMode::Position`] a *place* rather than a
/// suggestion.
///
/// **Not a knob, and MEASURED**, for the same reason [`MOTOR_TRACKING`] is not
/// one: the artist says *where* and *how strong*, and this is the number that
/// makes those two mean what they say. The rig is the one the `MOTOR_TRACKING`
/// table already uses — a 0.2 kg, 1 m arm hanging straight down from a pin — told
/// to hold **+45°**, which gravity spends the whole run pulling it away from, at
/// the DEFAULT `max_force` of 10 and at the damping chosen below:
///
/// | stiffness | settles at | droop | time to ±1° | overshoot |
/// |---|---|---|---|---|
/// | 100 | −59.21° | 104.21° | never | 0.00° |  ← cannot lift its own arm
/// | 300 | 5.78° | 39.22° | never | 0.00° |
/// | 1000 | 41.02° | 3.98° | never | 0.00° |  ← visibly sags
/// | 3000 | 44.13° | 0.87° | 1.65 s | 0.00° |
/// | **10000** | **44.74°** | **0.26°** | **0.42 s** | **0.00°** |
/// | 30000 | 44.91° | 0.09° | 0.60 s | 59.74° |  ← starts slapping again
///
/// The droop is `gravity_torque / stiffness`, so it never has a true knee — it
/// just gets smaller. What picks 10000 is the pair of ends: below it the sag is
/// something you can see (a degree at 3000, four at 1000), and above it the
/// approach starts to overshoot again and takes *longer* to arrive. A quarter of
/// a degree is held; the arm arrives in under half a second and does not pass.
///
/// ⚠️ **The same number serves a Slider and a Rope**, whose error is in metres
/// rather than radians. A settling time goes as `1/√stiffness` either way, and
/// the steady-state sag becomes `g / stiffness` ≈ **1 mm** — the units of the
/// free degree of freedom cancel, which is why there is one constant and not
/// one per kind.
///
/// Reproduce with `cargo test -p ph2d-physics servo_gain_sweep -- --ignored
/// --nocapture` (the sweep drives `spawn_joint_with_gains`, the product path).
/// `pub(super)` only so the sweeps in `world::tests` can hold one gain at its
/// shipped value while varying another.
pub(super) const SERVO_STIFFNESS: f32 = 10_000.0;

/// How hard a servo resists its own approach — rapier's motor `damping` in
/// [`MotorMode::Position`]. Also MEASURED, on the same arm and at the stiffness
/// above, and the quantity it buys is **overshoot**, not accuracy:
///
/// | damping | settles at | time to ±1° | overshoot |
/// |---|---|---|---|
/// | 50 | 44.77° | 1.08 s | 106.14° |  ← flies past 150° and swings back
/// | 200 | 44.77° | 0.18 s | 67.58° |
/// | 400 | 44.76° | 0.40 s | 27.28° |
/// | 500 | 44.75° | 0.38 s | 11.07° |
/// | **700** | **44.74°** | **0.42 s** | **0.00°** |  ← lands without passing
/// | 1000 | 44.72° | 0.58 s | 0.00° |
///
/// 700 is where the overshoot reaches zero; past it the servo only takes longer
/// to arrive, which is not a thing anyone asked for. ⚠️ **The `2√k` of textbook
/// critical damping is wrong here by 3.5×** (it would be 200, which overshoots
/// by 67°) — rapier's motor is acceleration-based and solved with the contacts,
/// so the analytic number is a starting point for a sweep, not an answer.
pub(super) const SERVO_DAMPING: f32 = 700.0;
