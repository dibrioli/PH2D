//! **O QUE PARTE SOB CARGA** (W-Pulley W2) — a corda e os eixos.
//!
//! Terceiro irmão de `pulley.rs`, e o corte segue por responsabilidade: lá mora o
//! que uma CORDA faz, em `pulley_winch.rs` o que um TAMBOR faz com ela, e aqui o
//! que acontece quando algo **cede**. As tabelas que escolheram os números vivem
//! em `measure_pulley.rs::sweep_the_break`.

use ph2d_physics::PhysicsWorld;
use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route::RopeWheel;

const WHEEL_ID: u64 = 77;
const ROPE_ID: u64 = 1;

/// Poste ESTÁTICO -> roldana -> carga pendurada. `anchor` diz onde o poste fica,
/// e é ele que decide o ÂNGULO do enlace — que é o que separa a tensão da
/// resultante no eixo.
fn rig(
    anchor_at: [f32; 2],
    mass: f32,
    rope_break: f32,
    wheel_break: f32,
) -> (PhysicsWorld, PulleyDesc, ph2d_physics::RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    const R: f32 = 0.2;
    let area = std::f32::consts::PI * R * R;
    let (anchor, _) = w.add_static_cuboid(anchor_at[0], anchor_at[1], 0.1, 0.1);
    let (load, _) = w.add_dynamic_circle(0.6, 2.0, R, mass / area);
    let wheel = RopeWheel {
        centre: [0.0, 8.0],
        radius: 0.5,
        side: 1,
        id: WHEEL_ID,
        break_force: wheel_break,
        ..RopeWheel::default()
    };
    let probe = PulleyDesc {
        id: ROPE_ID,
        body_a: anchor,
        body_b: load,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: 1,
        total_length: 1.0e9,
        motor_rate: 0.0,
        break_force: rope_break,
    };
    w.set_pulleys(vec![probe], vec![wheel]);
    let span = w.pulley_span(&probe).expect("rota válida");
    let d = PulleyDesc {
        total_length: span,
        ..probe
    };
    w.set_pulleys(vec![d], vec![wheel]);
    (w, d, load)
}

fn y_of(w: &PhysicsWorld, h: ph2d_physics::RigidBodyHandle) -> f32 {
    w.body_pose(h).expect("corpo vivo").translation.y
}

/// **Uma corda pendurada lê o PESO que ela segura**, e o número é o dela.
///
/// Este é o mesmo padrão-ouro que o `joint_break` do W-J7 estabeleceu (um peso
/// pendurado num pino lê `m·g` exatamente), agora num vínculo que o rapier não
/// tem: a tensão vem do `λ` do nosso próprio passe, e `λ/dt` tinha de ser
/// newtons de verdade — senão um limiar de ruptura seria um número sem unidade.
#[test]
fn a_rope_reads_the_weight_it_is_holding() {
    for mass in [0.5_f32, 1.0, 2.0, 5.0, 10.0] {
        let (mut w, d, _) = rig([-4.0, 8.0], mass, f32::INFINITY, f32::INFINITY);
        for _ in 0..120 {
            w.step();
        }
        let t = w.pulley_tension(d.id);
        let want = mass * 9.81;
        assert!(
            (t / want - 1.0).abs() < 0.01,
            "{mass} kg: a corda lê {t:.4} N e o peso é {want:.4} N"
        );
    }
}

/// **O EIXO carrega a RESULTANTE, e ela não é a tensão** — a metade da wave que
/// justifica um limiar POR RODA ao lado do da corda.
///
/// A corda puxa o eixo ao longo das duas direções em que ela SAI da roda, então
/// a carga é `T·|u_saída − u_entrada|`: um enlace de 180° carrega **2T**, um de
/// 90° carrega `√2·T`, e um que quase não desvia a corda carrega quase nada. Os
/// três casos num gate só, porque é o CONTRASTE que prova que o número não é
/// simplesmente a tensão com outro nome.
#[test]
fn an_axle_carries_the_resultant_not_the_tension() {
    for (anchor, want, tol, what) in [
        ([-0.6_f32, 2.0_f32], 2.0_f32, 0.05_f32, "enlace de ~180°"),
        (
            [-4.0, 8.0],
            std::f32::consts::SQRT_2,
            0.06,
            "enlace de ~90°",
        ),
        ([-1.0, 8.0], 1.12, 0.06, "desvio pequeno"),
    ] {
        let (mut w, d, _) = rig(anchor, 1.0, f32::INFINITY, f32::INFINITY);
        for _ in 0..120 {
            w.step();
        }
        let (t, axle) = (w.pulley_tension(d.id), w.pulley_axle_load(WHEEL_ID));
        assert!(t > 1.0, "{what}: a corda não estava carregando nada");
        let ratio = axle / t;
        assert!(
            (ratio - want).abs() < tol,
            "{what}: eixo/tensão = {ratio:.4}, esperado {want:.4}"
        );
    }
}

/// **Uma corda que parte para de segurar, e a carga CAI.**
///
/// E ela não volta: a ruptura é estado da CORRIDA, não do documento — desfazê-la
/// é um Reset, nunca uma edição que o artista tenha de desfazer.
#[test]
fn a_rope_that_parts_stops_holding_and_the_load_falls() {
    // 5 N contra uma carga de 1 kg (9,81 N): não tem como aguentar.
    let (mut w, d, load) = rig([-4.0, 8.0], 1.0, 5.0, f32::INFINITY);
    let y0 = y_of(&w, load);
    let mut broke_at = None;
    for tick in 1..=120 {
        w.step();
        if broke_at.is_none() && !w.pulley_breaks().is_empty() {
            let b = w.pulley_breaks()[0];
            assert!(!b.is_wheel, "quem partiu foi a corda, não o eixo");
            assert_eq!(b.id, ROPE_ID);
            assert!(b.load > 5.0, "a carga do rompimento foi {:.4} N", b.load);
            broke_at = Some(tick);
        }
    }
    assert!(broke_at.is_some(), "a corda tinha de partir");
    assert!(!w.pulley_is_intact(d.id), "ela devia estar partida");
    // Cair, e cair MUITO: sem a corda a carga está em queda livre.
    assert!(
        y0 - y_of(&w, load) > 1.0,
        "a carga desceu só {:.4} m depois de a corda partir",
        y0 - y_of(&w, load)
    );
    // E a corda não segura mais nada: a tensão do tique seguinte é zero.
    w.step();
    assert_eq!(w.pulley_tension(d.id), 0.0);
}

/// **Um eixo que cede tira a roldana da ROTA — e o caminho ENCURTA.**
///
/// É o que torna a ruptura segura por construção: a rota sem aquela roldana é
/// mais curta, então `C < 0` (folga) e o passe não aplica impulso nenhum. Nada
/// é arremessado — a carga simplesmente cai.
///
/// ⚠️ O oráculo tem DUAS metades e a segunda é a que importa: além de a carga
/// cair, a velocidade dela tem de ficar na ordem da queda livre. Um gate que só
/// pedisse *"caiu"* passaria por cima de uma carga arremessada a 4000 m/s.
#[test]
fn a_broken_axle_leaves_the_route_and_nothing_is_thrown() {
    // O eixo de um enlace de ~180° carrega ~2T ≈ 19,6 N; 12 N não aguenta.
    let (mut w, d, load) = rig([-0.6, 2.0], 1.0, f32::INFINITY, 12.0);
    let y0 = y_of(&w, load);
    let mut broke = false;
    let mut vmax = 0.0_f32;
    for _ in 0..120 {
        w.step();
        if !broke && !w.pulley_breaks().is_empty() {
            let b = w.pulley_breaks()[0];
            assert!(b.is_wheel, "quem cedeu foi o EIXO");
            assert_eq!(b.id, WHEEL_ID);
            assert_eq!(b.point, [0.0, 8.0], "o evento aponta o centro da roda");
            broke = true;
        }
        let v = w
            .body_snapshots()
            .into_iter()
            .find(|s| s.handle_index == load.into_raw_parts().0)
            .map(|s| (s.linvel_x * s.linvel_x + s.linvel_y * s.linvel_y).sqrt())
            .unwrap_or(f32::NAN);
        vmax = vmax.max(v);
    }
    assert!(broke, "o eixo tinha de ceder");
    assert!(!w.pulley_wheel_is_intact(WHEEL_ID));
    // A corda em si NÃO partiu — ela só não passa mais por ali.
    assert!(w.pulley_is_intact(d.id), "a corda não devia ter partido");
    assert!(
        y0 - y_of(&w, load) > 1.0,
        "a carga desceu só {:.4} m",
        y0 - y_of(&w, load)
    );
    // Queda livre por 2 s chega a ~19,6 m/s. Um arremesso passaria disso em
    // ordens de grandeza — este é o teto que separa "caiu" de "foi lançada".
    assert!(
        vmax < 40.0,
        "a carga foi ARREMESSADA a {vmax:.1} m/s em vez de cair"
    );
}

/// **Limiares infinitos são o mundo pré-ruptura, AO BIT.**
///
/// A neutralidade é o que deixa esta wave segura para as cenas que já existiam:
/// nada compara verdadeiro contra `∞`, então nada rompe e nada muda.
#[test]
fn infinite_thresholds_change_nothing() {
    let mut hashes = Vec::new();
    for (rope, wheel) in [(f32::INFINITY, f32::INFINITY), (1.0e30, 1.0e30)] {
        let (mut w, _, _) = rig([-4.0, 8.0], 1.0, rope, wheel);
        for _ in 0..120 {
            w.step();
        }
        assert!(w.pulley_breaks().is_empty());
        hashes.push(w.deterministic_hash());
    }
    assert_eq!(
        hashes[0], hashes[1],
        "um limiar alto e um limiar infinito têm de dar a MESMA corrida"
    );
}
