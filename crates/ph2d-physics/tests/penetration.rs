//! **How far does a landing body sink into the floor?**
//!
//! Reported by Enio, 2026-07-18: *"observa-se alguma interpenetração dos
//! objetos dinâmicos com o chão"*. Measuring first separated two things that
//! look alike and are not:
//!
//! - **at rest, ~1.3 mm** — rapier's `normalized_allowed_linear_error`, 1 mm
//!   by design. At the editor's ~100 px/m that is 0.13 px. Not what anyone
//!   saw, and not worth chasing.
//! - **at impact, 83 mm for 9 frames** — a body landing at 9.4 m/s travels
//!   157 mm per 60 Hz tick, so the tick it first touches it is *already*
//!   deep inside. ~8 px on screen for 0.15 s. That is the report.
//!
//! And the depth is **not a solver failure**: contact damping, the
//! corrective-velocity ceiling, extra solver iterations and CCD were each
//! measured and every one left it at exactly 83.2 mm. It is `velocity × dt`,
//! so the only lever is a smaller `dt` (sub-steps), and the only lever on how
//! long it lasts is the contact spring frequency.

use ph2d_physics::{BodyDefaults, BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

const FLOOR_TOP: f32 = -0.8;
const HALF: f32 = 0.28;

/// Roughly one screen pixel at the editor's default zoom (~100 px/m). The
/// bar is what the ARTIST can see, not a number that flatters the solver.
const VISIBLE_M: f32 = 0.01;

/// Drop one body from `drop_y` onto a floor whose top is at [`FLOOR_TOP`].
/// Returns `(worst penetration, frames it stayed visible, resting depth)`.
fn drop_probe(world: &mut PhysicsWorld, drop_y: f32) -> (f32, u32, f32) {
    world.spawn_body(BodyDesc {
        body_type: RigidBodyType::Fixed,
        x: 0.0,
        y: -1.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Cuboid {
            half_x: 4.0,
            half_y: 0.2,
        },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    });
    let h = world.spawn_body(BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x: 0.0,
        y: drop_y,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Cuboid {
            half_x: HALF,
            half_y: HALF,
        },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    });
    let (mut worst, mut frames) = (0.0f32, 0u32);
    for _ in 0..400 {
        world.step();
        let pen = FLOOR_TOP - (world.body_pose(h).unwrap().translation.y - HALF);
        if pen > worst {
            worst = pen;
        }
        if pen > VISIBLE_M {
            frames += 1;
        }
    }
    let rest = FLOOR_TOP - (world.body_pose(h).unwrap().translation.y - HALF);
    (worst, frames, rest)
}

/// **The gate.** At every drop height the smoke scenes actually use, a body
/// may spend at most ONE frame visibly inside the floor.
///
/// Mutation-tested: dropping `DEFAULT_SUBSTEPS` back to 1 takes the depth to
/// 83 mm, and dropping `DEFAULT_CONTACT_HZ` back to rapier's 30 stretches the
/// recovery to 6+ frames. Either one makes this red — which is the point:
/// the two constants fix two different halves of the same artifact, so each
/// needs to be caught on its own ([[feedback_layered_defenses_need_per_layer_gates]]).
#[test]
fn a_landing_body_is_never_visibly_inside_the_floor_for_more_than_a_frame() {
    for drop_y in [0.0f32, 1.6, 2.7, 4.0] {
        let mut w = PhysicsWorld::new();
        let (worst, frames, rest) = drop_probe(&mut w, drop_y);
        assert!(
            frames <= 1,
            "dropped from y={drop_y}: the body was visibly inside the floor for {frames} frames \
             ({:.1} mm at worst) — the artist sees it sink and pop back out",
            worst * 1000.0
        );
        assert!(
            worst < 0.035,
            "dropped from y={drop_y}: sank {:.1} mm into the floor. Sub-stepping is the only \
             lever on this depth (it is velocity x dt, not a solver failure)",
            worst * 1000.0
        );
        assert!(
            rest < 0.003,
            "dropped from y={drop_y}: resting {:.2} mm inside the floor; rapier's designed slop \
             is 1 mm",
            rest * 1000.0
        );
    }
}

/// The fix must not be bought with jitter — a settled pile that trembles is
/// a worse artifact than one that sinks. Measured at zero before and after.
///
/// # ⛔⛔ Este gate esteve VERDE pela razão errada, e a cura já estava escrita ao lado
///
/// Ele corria 600 tiques (10 s) e só então media — e o [`BodyDefaults::ours`] adormece
/// um corpo depois de `2,0 s` sob o limiar. **As 30 bolas dormiam**, e um corpo a dormir
/// não é integrado: o `Δy` era `0,0` **por construção**, a toda frequência de contacto.
/// *Zero não reprova a não ser que se force o zero a poder falhar.*
///
/// ⭐ A cura é a mesma que a sonda irmã [`measure_settings`] já pagou em
/// `crates/ph2d-physics/tests/measure_settings.rs` (§`stack_of`): **proibir o sono** e
/// medir uma pilha acordada, que é também o pior caso honesto. Aqui ela vem com a
/// **metade justa** — a fixtura AFIRMA que os 30 corpos estão acordados no instante em
/// que a medição começa, senão a proibição podia deixar de morder em silêncio.
///
/// ## Medido (2026-08-30, `--profile ci-test`, 30 bolas, tiques 600→690, pior `|Δy|` por tique)
///
/// | fixtura | corpos acordados | pior tremor | veredito |
/// |---|---|---|---|
/// | ⛔ como estava: sono no default | **0 de 30** | `0,00000 mm` | verde **por construção** |
/// | ⭐ hoje: sono PROIBIDO | **30 de 30** | `0,00000 mm` (`0e0` cru) | verde por MEDIÇÃO |
///
/// ⚠️ **O `x` e a ROTAÇÃO ficam de fora porque foram medidos a zero em TODA a varredura
/// abaixo** — o tremor desta pilha é puramente vertical. *A régua fica no eixo em que o
/// defeito existe, e isso mediu-se antes de se escrever.*
///
/// # ⛔ A BARRA desceu de `1e-4` para `1e-5` m, e o número saiu de duas medições
///
/// Varrida a resposta da pilha ACORDADA a cada botão do solver, um de cada vez (a
/// varredura correu neste ficheiro e foi removida; os números são de 2026-08-30):
///
/// | perturbação | pior tremor | contra a barra antiga (`1e-4`) |
/// |---|---|---|
/// | ⭐ o que shipa (`4` sub-passos, `120 Hz`) | `0,00000 mm` | — |
/// | `ζ` do contacto de `0,5` a `20` | `0,00000 mm` | cego |
/// | tecto de velocidade correctiva `1` a `1000` | `0,00000 mm` | cego |
/// | iterações do solver `1` a `8` | `0,00000 mm` | cego |
/// | sub-passos `1`/`2`/`8` (a `120 Hz`) | `0,00000 mm` | cego |
/// | ⛔ **`3 840 Hz`+ com os `4` sub-passos** | **`0,0606 mm`** | ⛔ **passava VERDE** |
/// | ⛔⛔ `1 920 Hz`+ com **`1`** sub-passo | **`1,106 mm`** | vermelho |
///
/// ⇒ A barra de `1e-4 m` (`0,1 mm/tique`) ficava **acima** do pior defeito que uma
/// constante sozinha consegue produzir (`0,0606 mm`): *a folga era o tamanho do ponto
/// cego*. A nova barra é `1e-5 m` — `6×` abaixo desse defeito e infinitamente acima do
/// produto, que lê **zero exacto**. ⛔ Não é um número escolhido para caber: é a distância
/// entre duas medições, e não há valor do produto a acomodar (o produto é `0`).
///
/// # ⭐ E o par `(frequência, sub-passos)` é UM RELÓGIO, não dois botões
///
/// Nenhum dos dois sozinho treme. `1 920 Hz` com os `4` sub-passos que shipam dá **zero**;
/// `1` sub-passo com os `120 Hz` que shipam dá **zero**; os dois juntos dão `1,1 mm/tique`.
/// A Nyquist do sub-passo é `1/(2·dt_sub)` — `120 Hz` com `4` sub-passos, `30 Hz` com `1` —,
/// e a mola só passa a ser integrada mal quando a frequência atravessa **a do relógio em
/// que ela corre**. *É o mesmo mecanismo que o `solver_params` já escreveu sobre a
/// frequência e o amortecimento: adoptar metade de um par de afinação mistura duas
/// afinações.*
///
/// # A prova de que ele morde (mutação no PRODUTO, 2026-08-30)
///
/// | mutação | pior tremor | este gate |
/// |---|---|---|
/// | — (o que shipa) | `0,00000 mm` | ⭐ verde |
/// | `PhysicsWorld::DEFAULT_CONTACT_HZ` `120,0` → `3840,0` | `0,06056 mm` | ⛔ **VERMELHO** |
/// | + `DEFAULT_SUBSTEPS` `4` → `1` (as duas) | `1,10626 mm` | ⛔ **VERMELHO** |
///
/// ⚠️ **Com a fixtura antiga NENHUMA das duas mordia** — as bolas dormiam e a leitura era
/// `0,0` em qualquer afinação, o que é a definição de um gate que não mede nada.
///
/// ⛔⛔ **E uma nota herdada está DESACTUALIZADA:** o `measure_settings` diz que uma pilha
/// proibida de dormir deriva `0,114 mm` a `1920 Hz`. Re-corrido em 2026-08-30 nesta árvore,
/// aquele arnês imprime `0,0000 mm` **em todas as sete frequências**, a `1920` inclusive.
/// Aquela tabela é **pré-`rapier2d` 0.35**; o solver foi reescrito por baixo dela.
#[test]
fn a_settled_pile_is_completely_still() {
    let mut w = PhysicsWorld::new();
    // ⛔ **Sono PROIBIDO — e não é conveniência de fixtura.** Ver o doc acima: com o sono
    // ligado esta medição lê `0,0` sem simular nada. Os três campos são os mesmos que a
    // sonda `measure_settings::stack_of` usa, sobre os defaults do PRODUTO (`ours`) para
    // que o amortecimento medido continue a ser o dele.
    w.set_body_defaults(BodyDefaults {
        sleep_linear_threshold: 0.0,
        sleep_angular_threshold: 0.0,
        time_until_sleep: f32::MAX,
        ..BodyDefaults::ours()
    });
    w.spawn_body(BodyDesc {
        body_type: RigidBodyType::Fixed,
        x: 0.0,
        y: -1.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Cuboid {
            half_x: 50.0,
            half_y: 0.2,
        },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    });
    for i in 0..30 {
        w.spawn_body(BodyDesc {
            body_type: RigidBodyType::Dynamic,
            x: (i % 6) as f32 * 0.6 - 1.5,
            y: (i / 6) as f32 * 0.6,
            rotation: 0.0,
            density: 1.0,
            shape: ShapeDesc::Ball { radius: 0.25 },
            restitution: 0.0,
            friction: 0.5,
            layer: 0,
            is_sensor: false,
            gravity_scale: 1.0,
            linvel: [0.0, 0.0],
            angvel: 0.0,
            ccd: false,
            lock_rotation: false,
            lock_x: false,
            lock_y: false,
            mass_override: None,
            dominance: 0,
            material: Default::default(),
            damping: None,
            one_way: false,
            effector: None,
            offset: [0.0, 0.0],
        });
    }
    for _ in 0..600 {
        w.step();
    }
    // **A metade JUSTA:** a proibição tem de ter MORDIDO. Sem isto, o dia em que
    // `set_body_defaults` deixasse de chegar aos corpos devolvia este gate ao zero
    // garantido, e nada o diria.
    let (awake, total) = w
        .bodies()
        .iter()
        .filter(|(_, b)| b.is_dynamic())
        .fold((0usize, 0usize), |(a, t), (_, b)| {
            (a + usize::from(!b.is_sleeping()), t + 1)
        });
    assert_eq!(
        awake, total,
        "fixture: os {total} corpos tinham de estar ACORDADOS aos 10 s — um corpo a dormir \
         nao e' integrado e o tremor deste gate seria 0,0 por construcao ({awake} acordados)"
    );

    let mut prev: Vec<f32> = w.body_snapshots().iter().map(|s| s.y).collect();
    let mut worst = 0.0f32;
    for _ in 0..90 {
        w.step();
        let now: Vec<f32> = w.body_snapshots().iter().map(|s| s.y).collect();
        for (a, b) in prev.iter().zip(&now) {
            worst = worst.max((a - b).abs());
        }
        prev = now;
    }
    println!(
        "settled pile: {awake}/{total} awake, worst {:.5} mm/tick (raw {worst:e} m)",
        worst * 1000.0
    );
    // A barra é DERIVADA — ver a tabela no doc: `6x` abaixo do pior defeito que um botão
    // sozinho produz (`0,0606 mm`), e o produto lê zero exacto.
    assert!(
        worst < 1e-5,
        "a settled pile is still moving {:.5} mm/tick — the stiffer contacts bought the \
         penetration back as jitter",
        worst * 1000.0
    );
}

/// Sub-stepping must stay inside HR-4's 1.5 ms physics frame at a body count
/// a 2D game actually ships. A RATIO against the single-step cost, not a
/// wall-clock bar: `ci-test` builds at `opt-level=1` and a stopwatch there
/// measures the profile, not the code.
#[test]
fn sub_stepping_costs_what_it_says_it_costs() {
    let build = |substeps: u32| {
        let mut w = PhysicsWorld::new();
        w.set_substeps(substeps);
        w.spawn_body(BodyDesc {
            body_type: RigidBodyType::Fixed,
            x: 0.0,
            y: -1.0,
            rotation: 0.0,
            density: 1.0,
            shape: ShapeDesc::Cuboid {
                half_x: 50.0,
                half_y: 0.2,
            },
            restitution: 0.0,
            friction: 0.5,
            layer: 0,
            is_sensor: false,
            gravity_scale: 1.0,
            linvel: [0.0, 0.0],
            angvel: 0.0,
            ccd: false,
            lock_rotation: false,
            lock_x: false,
            lock_y: false,
            mass_override: None,
            dominance: 0,
            material: Default::default(),
            damping: None,
            one_way: false,
            effector: None,
            offset: [0.0, 0.0],
        });
        for i in 0..200 {
            w.spawn_body(BodyDesc {
                body_type: RigidBodyType::Dynamic,
                x: (i % 20) as f32 * 0.6 - 5.7,
                y: (i / 20) as f32 * 0.6,
                rotation: 0.0,
                density: 1.0,
                shape: ShapeDesc::Ball { radius: 0.25 },
                restitution: 0.0,
                friction: 0.5,
                layer: 0,
                is_sensor: false,
                gravity_scale: 1.0,
                linvel: [0.0, 0.0],
                angvel: 0.0,
                ccd: false,
                lock_rotation: false,
                lock_x: false,
                lock_y: false,
                mass_override: None,
                dominance: 0,
                material: Default::default(),
                damping: None,
                one_way: false,
                effector: None,
                offset: [0.0, 0.0],
            });
        }
        w
    };
    let time = |substeps: u32| {
        let mut w = build(substeps);
        for _ in 0..120 {
            w.step();
        }
        let t = std::time::Instant::now();
        for _ in 0..100 {
            w.step();
        }
        t.elapsed().as_nanos() as f64 / 100.0
    };
    let one = time(1);
    let four = time(PhysicsWorld::DEFAULT_SUBSTEPS);
    let ratio = four / one;
    println!(
        "200 bodies: 1 substep {:.1} us, 4 substeps {:.1} us ({ratio:.2}x)",
        one / 1000.0,
        four / 1000.0
    );
    assert!(
        ratio < 6.0,
        "4 sub-steps cost {ratio:.2}x a single step — more than the ~4x they are, so something \
         beyond the integration is being repeated per sub-step"
    );
}
