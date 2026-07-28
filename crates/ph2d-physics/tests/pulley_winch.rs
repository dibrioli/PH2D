//! **O GUINCHO** (W-Pulley W2) — uma roldana com motor.
//!
//! Irmão de `pulley.rs`, e o corte é por responsabilidade: lá mora o que uma
//! CORDA faz (puxa e não empurra, a massa efetiva das pontas, a folga), aqui o
//! que um TAMBOR DIRIGIDO faz com ela. As tabelas que escolheram os números vivem
//! em `measure_pulley.rs::sweep_the_winch`.

use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route::RopeWheel;
use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyHandle, ShapeDesc};
use rapier2d::dynamics::RigidBodyType;

const R: f32 = 0.2;

fn ball(mass: f32) -> BodyDesc {
    BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x: 0.0,
        y: 2.0,
        rotation: 0.0,
        density: mass / (std::f32::consts::PI * R * R),
        shape: ShapeDesc::Ball { radius: R },
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
    }
}

/// Poste ESTATICO ao alto e a esquerda -> roldana -> carga pendurada **sob** ela.
/// E o que um guincho e: quem recolhe esta preso, quem sobe e a carga.
///
/// ⚠️ VERTICAL de proposito. Com a carga pendurada de LADO a corda encurta e ela
/// **balanca** em vez de subir, e a razao subida/recolhido vira um numero sobre a
/// geometria da fixture (0,21 a 0,71 conforme a velocidade), nao sobre o guincho.
fn winch(radius: f32, omega: f32, mass: f32) -> (PhysicsWorld, PulleyDesc, RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    let (anchor, _) = w.add_static_cuboid(-4.0, 8.0, 0.1, 0.1);
    let load = w.spawn_body(ball(mass));
    let wheel = RopeWheel {
        centre: [0.0, 8.0],
        radius,
        side: 1,
        id: 0,
        break_force: f32::INFINITY,
    };
    let probe = PulleyDesc {
        id: 1,
        body_a: anchor,
        body_b: load,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: 1,
        // Absurdo so para o `pulley_span` responder: ele mede a ROTA, que nao
        // depende de `total_length`.
        total_length: 1.0e9,
        motor_rate: omega * radius,
        break_force: f32::INFINITY,
    };
    w.set_pulleys(vec![probe], vec![wheel]);
    let span = w.pulley_span(&probe).expect("rota valida");
    let d = PulleyDesc {
        total_length: span,
        ..probe
    };
    w.set_pulleys(vec![d], vec![wheel]);
    (w, d, load)
}

fn rise_after(w: &mut PhysicsWorld, load: RigidBodyHandle, ticks: u32) -> f32 {
    let y0 = w.body_pose(load).expect("carga viva").translation.y;
    for _ in 0..ticks {
        w.step();
    }
    w.body_pose(load).expect("carga viva").translation.y - y0
}

/// **O tambor ergue a `ω·r`, e por isso o DIAMETRO e o cambio.**
///
/// O `ratio` que esta wave aposentou prometia vantagem mecanica e nao entregava
/// nenhuma (§3 do modulo). Esta e a vantagem de verdade: o MESMO motor num tambor
/// duas vezes maior recolhe duas vezes mais depressa.
#[test]
fn a_driven_drum_lifts_the_load_at_omega_times_the_radius() {
    for (radius, omega) in [(0.25_f32, 1.0_f32), (0.25, 2.0), (0.5, 1.0), (0.5, 2.0)] {
        let (mut w, d, load) = winch(radius, omega, 1.0);
        let rose = rise_after(&mut w, load, 60);
        let want = omega * radius;
        assert!(
            (w.pulley_reeled(&d) - want).abs() < 1.0e-3,
            "r {radius} w {omega}: recolheu {} em 1 s, devia recolher {want}",
            w.pulley_reeled(&d)
        );
        // 0,90..1,00 e a faixa MEDIDA: a carga parte do repouso, entao o primeiro
        // punhado de tiques fica para tras e a defasagem amortiza ao longo do
        // segundo. Ela nunca ULTRAPASSA o recolhido — a corda nao empurra.
        let ratio = rose / want;
        assert!(
            (0.88..=1.0).contains(&ratio),
            "r {radius} w {omega}: subiu {rose} para {want} de corda recolhida (razao {ratio})"
        );
    }
    // E o cambio: dobrar o RAIO com o mesmo motor dobra a subida.
    let (mut small, _, l1) = winch(0.25, 2.0, 1.0);
    let (mut big, _, l2) = winch(0.5, 2.0, 1.0);
    let (a, b) = (rise_after(&mut small, l1, 60), rise_after(&mut big, l2, 60));
    assert!(
        (b / a - 2.0).abs() < 0.05,
        "o tambor de raio dobrado subiu {b} contra {a} (razao {})",
        b / a
    );
}

/// **A carga nao pesa para o guincho** — e isso e a projecao com massa efetiva
/// EXATA, nao um ganho generoso.
///
/// O guincho e onipotente por construcao, e e por isso que o que o limita e
/// geometria (o teto de [`ph2d_physics::world::pulley::PULLEY_CORRECTION_LAG`]) e
/// nao forca. Um dia isto muda: quando o motor ganhar um teto de tensao, ESTE
/// gate e o que dira que ele passou a existir.
#[test]
fn the_winch_does_not_care_what_the_load_weighs() {
    let mut seen: Option<f32> = None;
    for mass in [0.1_f32, 1.0, 10.0, 100.0, 1000.0] {
        let (mut w, _, load) = winch(0.5, 2.0, mass);
        let rose = rise_after(&mut w, load, 60);
        match seen {
            None => seen = Some(rose),
            Some(first) => assert!(
                (rose - first).abs() < 1.0e-4,
                "{mass} kg subiu {rose}, 0,1 kg subiu {first}"
            ),
        }
    }
}

/// **Pagar corda BAIXA a carga, e quem a baixa e a GRAVIDADE.**
///
/// A metade que um alvo de velocidade nao consegue expressar (o cabecalho do
/// modulo conta por que): com `λ ≥ 0` a corda so puxa, entao alongar `L0` e a
/// unica forma de descer — e a carga nunca e EMPURRADA.
#[test]
fn paying_out_lowers_the_load_and_never_pushes_it() {
    for omega in [-1.0_f32, -2.0] {
        let (mut w, d, load) = winch(0.5, omega, 1.0);
        let fell = -rise_after(&mut w, load, 60);
        assert!(
            w.pulley_reeled(&d) < 0.0,
            "w {omega}: o recolhido devia ser negativo, e {}",
            w.pulley_reeled(&d)
        );
        assert!(fell > 0.4, "w {omega}: desceu so {fell} m em 1 s");
        // Nunca EMPURRADA: a carga nao pode descer mais rapido que a queda livre.
        assert!(
            fell < 4.905,
            "w {omega}: desceu {fell} m, mais que a queda livre"
        );
    }
}

/// **Dois tambores na mesma corda SOMAM as taxas** — e dois em sentidos opostos
/// se anulam, que e o que dois guinchos brigando pela mesma corda fazem.
#[test]
fn two_drums_on_one_rope_sum_their_rates() {
    // O mesmo rig, com a taxa da corda montada de duas maneiras: um tambor a 1,0
    // e "dois" a 0,5 cada (a ponte soma antes de chegar aqui, entao o kernel ve
    // a soma — este gate pina que a SOMA e a lei, e o valor que ela produz).
    let (mut one, d1, l1) = winch(0.5, 2.0, 1.0);
    let a = rise_after(&mut one, l1, 60);

    let (mut two, _, l2) = winch(0.5, 1.0, 1.0);
    // A ponte somaria 0,5 + 0,5 = 1,0 m/s; aqui a soma e escrita a mao.
    let doubled = PulleyDesc {
        motor_rate: 0.5 + 0.5,
        break_force: f32::INFINITY,
        ..d1
    };
    two.set_pulleys(
        vec![PulleyDesc {
            body_a: doubled.body_a,
            body_b: l2,
            ..doubled
        }],
        vec![RopeWheel {
            centre: [0.0, 8.0],
            radius: 0.5,
            side: 1,
            id: 0,
            break_force: f32::INFINITY,
        }],
    );
    let b = rise_after(&mut two, l2, 60);
    assert!(
        (a - b).abs() < 1.0e-3,
        "um tambor a 1,0 m/s subiu {a}; dois a 0,5 subiram {b}"
    );
}

/// **O teto da correcao limita a fuga E deixa o recolhimento normal EXATO.**
///
/// Os dois lados sao um gate so de proposito: um teto que so limitasse seria
/// indistinguivel de um que estrangula o guincho, e a varredura em
/// [`ph2d_physics::world::pulley::PULLEY_CORRECTION_LAG`] mostra que a diferenca
/// entre os dois esta em UM sub-passo de folga.
///
/// ⚠️ O oraculo da fuga e a velocidade MAXIMA ao longo da corrida, nao a pose
/// final: a carga arremessada volta a cruzar a cena, entao um endpoint pode
/// pousar em qualquer lugar (medido: `y` final 8,21 com teto e −20,13 sem).
#[test]
fn the_correction_cap_bounds_the_runaway_without_slowing_the_winch() {
    fn run(lag: Option<f32>) -> (f32, f32) {
        let (mut w, _, load) = winch(0.5, 4.0, 1.0);
        if let Some(lag) = lag {
            w.set_pulley_correction_lag(lag);
        }
        let mut vmax = 0.0_f32;
        let mut y_mid = 0.0_f32;
        for tick in 1..=300 {
            w.step();
            let v = w
                .body_snapshots()
                .into_iter()
                .find(|s| s.handle_index == load.into_raw_parts().0)
                .map(|s| (s.linvel_x * s.linvel_x + s.linvel_y * s.linvel_y).sqrt())
                .unwrap_or(f32::NAN);
            vmax = vmax.max(v);
            if tick == 150 {
                y_mid = w.body_pose(load).expect("carga viva").translation.y;
            }
        }
        (vmax, y_mid)
    }
    let (capped, y_capped) = run(None);
    let (loose, y_loose) = run(Some(1.0e9));
    assert!(
        (y_capped - y_loose).abs() < 1.0e-4,
        "o teto atrasou o guincho: {y_capped} contra {y_loose} aos 2,5 s"
    );
    assert!(
        capped < loose / 20.0,
        "|v| maximo {capped} com teto contra {loose} sem — o teto nao limitou"
    );
}

/// **Uma corda SEM tambor e byte-identica ao mundo pre-motor.**
///
/// A neutralidade nao e higiene: o teto da correcao existe so para quem recolhe,
/// e uma corda parada nao pode ver diferenca nenhuma. Este gate e o que deixa a
/// wave inteira segura para as vinte cenas que ja existiam.
#[test]
fn a_rope_without_a_drum_is_untouched() {
    fn run(lag: Option<f32>) -> [u8; 32] {
        let (mut w, d, _) = winch(0.5, 0.0, 1.0);
        assert_eq!(d.motor_rate, 0.0, "a fixture tem de ser uma corda PARADA");
        if let Some(lag) = lag {
            w.set_pulley_correction_lag(lag);
        }
        for _ in 0..120 {
            w.step();
        }
        w.deterministic_hash()
    }
    assert_eq!(
        run(None),
        run(Some(0.001)),
        "uma corda sem motor sentiu o teto da correcao"
    );
}
