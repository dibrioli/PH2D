//! **A polia, medida** (W-Pulley) — as tabelas que os números do produto citam.
//!
//! Roda pelo caminho do PRODUTO (`PhysicsWorld::step`, com o passe onde ele de
//! fato mora), nunca por uma segunda cópia do laço num arquivo de teste: uma
//! sonda que re-implementa o que mede fica cega à porta, e esta linha já pagou
//! isso mais de uma vez.
//!
//! `cargo test -p ph2d-physics --test measure_pulley -- --ignored --nocapture`

use ph2d_physics::PhysicsWorld;
use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route::RopeWheel;

/// Uma balança de Atwood: duas roldanas no alto, um corpo pendurado sob cada
/// uma, ligados por uma corda que passa pelas duas.
///
/// Massas IGUAIS, de propósito: é a única configuração que tem equilíbrio
/// estático, e portanto a única em que "quanto a corda estica em regime" é uma
/// pergunta com resposta. Com massas diferentes o sistema acelera para sempre
/// (é o que uma máquina de Atwood faz) e o esticamento nunca assenta.
/// As duas roldanas de raio ZERO — o modelo de PONTO, que a rota reproduz
/// exatamente. As tabelas desta sonda foram medidas com ele, e é ele que as
/// mantém comparáveis.
fn point_wheels() -> Vec<RopeWheel> {
    vec![
        RopeWheel {
            centre: [-1.0, 4.0],
            radius: 0.0,
            side: 1,
        },
        RopeWheel {
            centre: [1.0, 4.0],
            radius: 0.0,
            side: 1,
        },
    ]
}

fn atwood(mass_a: f32, mass_b: f32) -> (PhysicsWorld, PulleyDesc, ph2d_physics::RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    const R: f32 = 0.2;
    let area = std::f32::consts::PI * R * R;
    let (a, _) = w.add_dynamic_circle(-1.0, 2.0, R, mass_a / area);
    let (b, _) = w.add_dynamic_circle(1.0, 2.0, R, mass_b / area);
    let desc = PulleyDesc {
        body_a: a,
        body_b: b,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: 2,
        // l1 = l2 = 2 na pose de repouso.
        id: 1,
        total_length: 4.0,
        motor_rate: 0.0,
    };
    w.set_pulleys(vec![desc], point_wheels());
    (w, desc, a)
}

/// `l1 + ratio·l2 − L0` — o quanto a corda está esticada AGORA.
fn stretch(w: &PhysicsWorld, d: &PulleyDesc) -> f32 {
    w.pulley_span(d).unwrap_or(f32::NAN) - d.total_length
}

#[test]
#[ignore = "measurement, not a gate"]
fn sweep_the_pulley_bias() {
    println!("\n=== O ESTICAMENTO EM REGIME, por beta (Atwood 1 kg x 1 kg, 2 s) ===");
    println!(
        "{:>6} | {:>12} | {:>12} | {:>10}",
        "beta", "regime (m)", "pico (m)", "tremor"
    );
    for beta in [0.05_f32, 0.1, 0.2, 0.4, 0.8, 1.0, 1.5, 2.0] {
        let (mut w, d, _) = atwood(1.0, 1.0);
        w.set_pulley_bias(beta);
        let mut peak = 0.0_f32;
        // Depois de assentar, a amplitude pico-a-pico do esticamento ao longo de
        // meio segundo: um valor que oscila diz "a correcao passa do alvo",
        // e é isso que separa preciso de instável.
        let mut lo = f32::INFINITY;
        let mut hi = f32::NEG_INFINITY;
        for tick in 0..120 {
            w.step();
            let s = stretch(&w, &d);
            peak = peak.max(s);
            if tick >= 90 {
                lo = lo.min(s);
                hi = hi.max(s);
            }
        }
        println!(
            "{beta:>6.2} | {:>12.4} | {peak:>12.4} | {:>10.5}",
            stretch(&w, &d),
            hi - lo
        );
    }
}

#[test]
#[ignore = "measurement, not a gate"]
fn sweep_the_load_the_rope_holds() {
    println!("\n=== O ESTICAMENTO por CARGA (beta de producao, 2 s) ===");
    println!(
        "{:>10} | {:>12} | {:>12}",
        "massa (kg)", "regime (m)", "pico (m)"
    );
    for m in [0.1_f32, 1.0, 10.0, 100.0] {
        let (mut w, d, _) = atwood(m, m);
        let mut peak = 0.0_f32;
        for _ in 0..120 {
            w.step();
            peak = peak.max(stretch(&w, &d));
        }
        println!("{m:>10.1} | {:>12.4} | {peak:>12.4}", stretch(&w, &d));
    }
}

#[test]
#[ignore = "measurement, not a gate"]
fn the_atwood_machine_accelerates() {
    println!("\n=== A MAQUINA DE ATWOOD (o que a polia existe para fazer) ===");
    println!(
        "{:>12} | {:>10} | {:>10} | {:>12}",
        "m_a : m_b", "y_a (m)", "y_b (m)", "soma (m)"
    );
    for (ma, mb) in [(1.0_f32, 1.0_f32), (2.0, 1.0), (4.0, 1.0)] {
        let (mut w, d, a) = atwood(ma, mb);
        for _ in 0..60 {
            w.step();
        }
        let pa = w.body_pose(d.body_a).unwrap().translation;
        let pb = w.body_pose(d.body_b).unwrap().translation;
        let _ = a;
        println!(
            "{ma:>5.0} : {mb:<4.0} | {:>10.4} | {:>10.4} | {:>12.4}",
            pa.y,
            pb.y,
            // O que a corda promete: o que um lado desce, o outro sobe.
            (2.0 - pa.y) - (pb.y - 2.0)
        );
    }
}

#[test]
#[ignore = "measurement, not a gate"]
fn sweep_the_bias_against_a_contact() {
    // A varredura livre (`sweep_the_pulley_bias`) melhora monotonicamente com
    // beta e nunca treme — mas ela nao contem o fenomeno que de fato limita uma
    // correcao EXPLICITA: a corda disputando com o solver de CONTATO. Aqui a
    // carga descansa no chao com a corda esticada demais, entao o passe da polia
    // puxa para cima a cada sub-passo e o contato empurra de volta.
    println!("\n=== A CORDA CONTRA O CHAO (a carga apoiada, 2 s) ===");
    println!(
        "{:>6} | {:>12} | {:>14} | {:>12}",
        "beta", "y final (m)", "tremor y (m)", "|v| final"
    );
    for beta in [0.05_f32, 0.1, 0.2, 0.4, 0.8, 1.0, 1.5, 2.0] {
        let mut w = PhysicsWorld::new();
        const R: f32 = 0.2;
        let area = std::f32::consts::PI * R * R;
        w.add_static_cuboid(0.0, 0.0, 6.0, 0.2);
        // A carga pousa no chao (y = 0,4); o contrapeso pendura do outro lado e
        // puxa a corda o tempo todo, entao ela fica ESTICADA sobre um corpo que
        // o chao nao deixa subir.
        let (a, _) = w.add_dynamic_circle(-1.0, 0.4, R, 4.0 / area);
        let (b, _) = w.add_dynamic_circle(1.0, 3.0, R, 1.0 / area);
        let d = PulleyDesc {
            body_a: a,
            body_b: b,
            local_a: [0.0, 0.0],
            local_b: [0.0, 0.0],
            wheel_start: 0,
            wheel_count: 2,
            // ⚠️ EXATAMENTE esticada na pose inicial (l1 = 3,6 · l2 = 1,0): o
            // contrapeso de 1 kg puxa a corda o tempo todo tentando erguer uma
            // carga de 4 kg que o chao segura. Uma corda que ja nasce curta
            // demais ARRANCA a carga do chao num transiente violento, que e
            // outro fenomeno — e foi o que a primeira versao mediu.
            id: 1,
            total_length: 4.6,
            motor_rate: 0.0,
        };
        w.set_pulleys(vec![d], point_wheels());
        // ⚠️ SEM esta linha as oito corridas sao a MESMA corrida, e a tabela
        // sai com oito linhas identicas dizendo nada. Aconteceu.
        w.set_pulley_bias(beta);
        let mut lo = f32::INFINITY;
        let mut hi = f32::NEG_INFINITY;
        for tick in 0..120 {
            w.step();
            if tick >= 90 {
                let y = w.body_pose(a).unwrap().translation.y;
                lo = lo.min(y);
                hi = hi.max(y);
            }
        }
        let pose = w.body_pose(a).unwrap().translation;
        let v = w
            .body_snapshots()
            .iter()
            .find(|s| s.linvel_y.abs() >= 0.0)
            .map(|_| 0.0);
        let _ = v;
        let speed = w
            .body_snapshots()
            .iter()
            .map(|s| (s.linvel_x * s.linvel_x + s.linvel_y * s.linvel_y).sqrt())
            .fold(0.0_f32, f32::max);
        println!(
            "{beta:>6.2} | {:>12.4} | {:>14.5} | {speed:>12.4}",
            pose.y,
            hi - lo
        );
    }
}

#[test]
#[ignore = "measurement, not a gate"]
fn what_a_frozen_axis_looks_like_to_the_rope() {
    use rapier2d::dynamics::RigidBodyType;
    println!("\n=== ANCORA CONGELADA vs PAREDE (carga de 3 kg pendurada) ===");
    for (label, kind, lock) in [
        ("dynamic lock_y", RigidBodyType::Dynamic, true),
        ("fixed        ", RigidBodyType::Fixed, false),
    ] {
        let mut w = PhysicsWorld::new();
        const R: f32 = 0.2;
        let area = std::f32::consts::PI * R * R;
        let mut d0 = ph2d_physics::BodyDesc {
            body_type: kind,
            x: -1.0,
            y: 2.0,
            rotation: 0.0,
            density: 1.0 / area,
            shape: ph2d_physics::ShapeDesc::Ball { radius: R },
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
            lock_y: lock,
            mass_override: None,
            dominance: 0,
            material: Default::default(),
            damping: None,
            one_way: false,
            effector: None,
            offset: [0.0, 0.0],
        };
        d0.lock_y = lock;
        let a = w.spawn_body(d0);
        let (b, _) = w.add_dynamic_circle(1.0, 2.0, R, 3.0 / area);
        let d = PulleyDesc {
            body_a: a,
            body_b: b,
            local_a: [0.0, 0.0],
            local_b: [0.0, 0.0],
            wheel_start: 0,
            wheel_count: 2,
            id: 1,
            total_length: 4.0,
            motor_rate: 0.0,
        };
        w.set_pulleys(vec![d], point_wheels());
        for _ in 0..90 {
            w.step();
        }
        let pa = w.body_pose(a).unwrap().translation;
        println!(
            "{label} : B y = {:.5} | C = {:.6} | A = ({:.5}, {:.5}) | span = {:.5}",
            w.body_pose(b).unwrap().translation.y,
            stretch(&w, &d),
            pa.x,
            pa.y,
            w.pulley_span(&d).unwrap()
        );
        println!("                 k = {:?}", w.pulley_branch_k(&d));
    }
}

#[test]
#[ignore = "measurement, not a gate"]
fn what_the_rate_term_of_a_non_dynamic_body_buys() {
    use rapier2d::dynamics::RigidBodyType;
    // O `rate` de um corpo nao-dinamico entra em Cdot mas o `k` dele nao entra
    // na massa efetiva. A mutacao que zera esse rate SOBREVIVEU aos gates, entao
    // a pergunta e: o termo compra alguma coisa OBSERVAVEL?
    println!("\n=== O ATRASO DA CARGA ATRAS DO GUINCHO, por velocidade ===");
    println!(
        "{:>12} | {:>12} | {:>12}",
        "v (m/s)", "C final (m)", "atraso (m)"
    );
    for speed in [0.5_f32, 1.2, 3.0, 6.0] {
        let mut w = PhysicsWorld::new();
        const R: f32 = 0.2;
        let area = std::f32::consts::PI * R * R;
        let drum = ph2d_physics::BodyDesc {
            body_type: RigidBodyType::KinematicPositionBased,
            x: -1.0,
            y: 2.0,
            rotation: 0.0,
            density: 1.0 / area,
            shape: ph2d_physics::ShapeDesc::Ball { radius: R },
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
        };
        let a = w.spawn_body(drum);
        let (b, _) = w.add_dynamic_circle(1.0, 2.0, R, 1.0 / area);
        let d = PulleyDesc {
            body_a: a,
            body_b: b,
            local_a: [0.0, 0.0],
            local_b: [0.0, 0.0],
            wheel_start: 0,
            wheel_count: 2,
            id: 1,
            total_length: 4.0,
            motor_rate: 0.0,
        };
        w.set_pulleys(vec![d], point_wheels());
        let dt = 1.0 / 60.0;
        for tick in 0..60 {
            w.set_next_kinematic_pose(a, -1.0, 2.0 - speed * dt * (tick + 1) as f32, 0.0);
            w.step();
        }
        let drum_drop = speed * dt * 60.0;
        let load_rise = w.body_pose(b).unwrap().translation.y - 2.0;
        println!(
            "{speed:>12.2} | {:>12.5} | {:>12.5}",
            stretch(&w, &d),
            drum_drop - load_rise
        );
    }
}

/// **O GUINCHO** (W2) — um tambor que recolhe, medido pelo caminho do produto.
///
/// ⚠️ A montagem e VERTICAL de proposito, e a primeira versao nao era: com a
/// carga pendurada de LADO a corda encurta e ela **balanca** em vez de subir, e a
/// razao subida/recolhido saiu 0,21 a 0,71 conforme a velocidade — um numero
/// sobre a geometria da fixture, nao sobre o guincho. Com a carga sob a roldana o
/// ramo e vertical e a subida E o recolhimento.
///
/// ⚠️ E a ancora fica LONGE: um poste a `r` do centro esta EM CIMA do circulo, e
/// a tangente comum nao existe ali (o guarda `|D| > |R|` da rota) — foi assim que
/// a linha de raio 1,0 entrou em panico na primeira corrida.
#[test]
#[ignore = "measurement, not a gate"]
fn sweep_the_winch() {
    /// Corda: poste ESTATICO ao alto e a esquerda -> roldana -> carga pendurada
    /// **sob** ela. E o que um guincho e: quem recolhe esta preso, quem sobe e a
    /// carga.
    fn rig(
        radius: f32,
        omega: f32,
        mass: f32,
    ) -> (PhysicsWorld, PulleyDesc, ph2d_physics::RigidBodyHandle) {
        let mut w = PhysicsWorld::new();
        const R: f32 = 0.2;
        let area = std::f32::consts::PI * R * R;
        let (anchor, _) = w.add_static_cuboid(-4.0, 8.0, 0.1, 0.1);
        let (load, _) = w.add_dynamic_circle(0.0, 2.0, R, mass / area);
        let wheel = RopeWheel {
            centre: [0.0, 8.0],
            radius,
            side: 1,
        };
        let probe = PulleyDesc {
            id: 1,
            body_a: anchor,
            body_b: load,
            local_a: [0.0, 0.0],
            local_b: [0.0, 0.0],
            wheel_start: 0,
            wheel_count: 1,
            // Um comprimento absurdo so para o `pulley_span` responder: ele mede a
            // ROTA, que nao depende de `total_length`.
            total_length: 1.0e9,
            motor_rate: omega * radius,
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

    println!("\n=== O GUINCHO ERGUE A `w*r` (1 s, carga de 1 kg) ===");
    println!(
        "{:>8} | {:>9} | {:>12} | {:>12} | {:>10} | {:>8} | {:>12}",
        "r (m)", "w (rad/s)", "w*r (m/s)", "recolhido", "subiu (m)", "razao", "aperto C (m)"
    );
    for (radius, omega) in [
        (0.25_f32, 1.0_f32),
        (0.25, 2.0),
        (0.5, 1.0),
        (0.5, 2.0),
        (1.0, 1.0),
        (1.0, 2.0),
    ] {
        let (mut w, d, load) = rig(radius, omega, 1.0);
        let y0 = w.body_pose(load).unwrap().translation.y;
        for _ in 0..60 {
            w.step();
        }
        let rose = w.body_pose(load).unwrap().translation.y - y0;
        let reeled = w.pulley_reeled(&d);
        let c = w.pulley_span(&d).unwrap_or(f32::NAN) - (d.total_length - reeled);
        println!(
            "{radius:>8.2} | {omega:>9.2} | {:>12.4} | {reeled:>12.4} | {rose:>10.4} |              {:>8.4} | {c:>12.6}",
            omega * radius,
            rose / (omega * radius)
        );
    }

    println!("\n=== A CARGA PESA? (r 0,5 - w 2 rad/s - 1 s) ===");
    println!(
        "{:>10} | {:>12} | {:>10} | {:>12}",
        "massa (kg)", "recolhido", "subiu (m)", "aperto C (m)"
    );
    for mass in [0.1_f32, 1.0, 10.0, 100.0, 1000.0] {
        let (mut w, d, load) = rig(0.5, 2.0, mass);
        let y0 = w.body_pose(load).unwrap().translation.y;
        for _ in 0..60 {
            w.step();
        }
        let reeled = w.pulley_reeled(&d);
        println!(
            "{mass:>10.1} | {reeled:>12.4} | {:>10.4} | {:>12.6}",
            w.body_pose(load).unwrap().translation.y - y0,
            w.pulley_span(&d).unwrap_or(f32::NAN) - (d.total_length - reeled)
        );
    }

    println!("\n=== PAGANDO CORDA (w NEGATIVO, r 0,5 - 1 s) ===");
    println!(
        "{:>9} | {:>12} | {:>10}",
        "w (rad/s)", "recolhido", "desceu (m)"
    );
    for omega in [-1.0_f32, -2.0, -4.0] {
        let (mut w, d, load) = rig(0.5, omega, 1.0);
        let y0 = w.body_pose(load).unwrap().translation.y;
        for _ in 0..60 {
            w.step();
        }
        println!(
            "{omega:>9.2} | {:>12.4} | {:>10.4}",
            w.pulley_reeled(&d),
            y0 - w.body_pose(load).unwrap().translation.y
        );
    }

    println!("\n=== O ESTOL NAO ATRAPALHA O RECOLHIMENTO NORMAL (1 s) ===");
    println!(
        "{:>8} | {:>9} | {:>12} | {:>12} | {:>10}",
        "r (m)", "w (rad/s)", "recolhido", "w*r em 1 s", "razao"
    );
    for (radius, omega) in [(0.25_f32, 1.0_f32), (0.5, 2.0), (1.0, 2.0)] {
        let (mut a, da, _) = rig(radius, omega, 1.0);
        for _ in 0..60 {
            a.step();
        }
        let ra = a.pulley_reeled(&da);
        println!(
            "{radius:>8.2} | {omega:>9.2} | {ra:>12.4} | {:>12.4} | {:>10.4}",
            omega * radius,
            ra / (omega * radius)
        );
    }

    println!("\n=== A RODA COMO OBSTACULO (r 0,5 - w 4 rad/s - 5 s) ===");
    println!(
        "{:>6} | {:>12} | {:>10} | {:>12} | {:>12}",
        "t (s)", "recolhido", "y", "|v|", "ate a roda"
    );
    {
        let mut w = PhysicsWorld::new();
        const R: f32 = 0.2;
        let area = std::f32::consts::PI * R * R;
        let (anchor, _) = w.add_static_cuboid(-4.0, 8.0, 0.1, 0.1);
        let (load, _) = w.add_dynamic_circle(0.0, 2.0, R, 1.0 / area);
        // A RODA, com corpo: um disco estatico do tamanho dela, no centro dela.
        w.add_static_cuboid(0.0, 8.0, 0.5, 0.5);
        let wheel = RopeWheel {
            centre: [0.0, 8.0],
            radius: 0.5,
            side: 1,
        };
        let probe = PulleyDesc {
            id: 1,
            body_a: anchor,
            body_b: load,
            local_a: [0.0, 0.0],
            local_b: [0.0, 0.0],
            wheel_start: 0,
            wheel_count: 1,
            total_length: 1.0e9,
            motor_rate: 4.0 * 0.5,
        };
        w.set_pulleys(vec![probe], vec![wheel]);
        let span = w.pulley_span(&probe).expect("rota valida");
        let d = PulleyDesc {
            total_length: span,
            ..probe
        };
        w.set_pulleys(vec![d], vec![wheel]);
        for tick in 1..=300 {
            w.step();
            if tick % 30 == 0 {
                let v = w
                    .body_snapshots()
                    .into_iter()
                    .find(|s| s.handle_index == load.into_raw_parts().0)
                    .map(|s| (s.linvel_x * s.linvel_x + s.linvel_y * s.linvel_y).sqrt())
                    .unwrap_or(f32::NAN);
                let y = w.body_pose(load).unwrap().translation.y;
                println!(
                    "{:>6.2} | {:>12.4} | {y:>10.4} | {v:>12.4} | {:>12.4}",
                    tick as f32 / 60.0,
                    w.pulley_reeled(&d),
                    8.0 - y
                );
            }
        }
    }

    println!("\n=== VARREDURA DO TETO (em SUB-PASSOS de recolhimento) ===");
    println!(
        "{:>10} | {:>12} | {:>10} | {:>12} | {:>12}",
        "C_LAG", "recolhido", "y final", "|v| MAX", "y aos 2,5 s"
    );
    for cap in [4.0_f32, 5.0, 6.0, 7.0, 8.0, 1.0e9] {
        let (mut w, d, load) = rig(0.5, 4.0, 1.0);
        w.set_pulley_correction_lag(cap);
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
                y_mid = w.body_pose(load).unwrap().translation.y;
            }
        }
        println!(
            "{cap:>11.1} | {:>12.4} | {:>10.4} | {vmax:>12.4} | {y_mid:>12.4}",
            w.pulley_reeled(&d),
            w.body_pose(load).unwrap().translation.y
        );
    }

    println!("\n=== ATE O FIM DA CORDA (r 0,5 - w 4 rad/s - 5 s) ===");
    println!(
        "{:>6} | {:>12} | {:>10} | {:>10} | {:>12} | {:>12}",
        "t (s)", "recolhido", "L0 (m)", "y", "|v|", "ate a roda"
    );
    let (mut w, d, load) = rig(0.5, 4.0, 1.0);
    for tick in 1..=300 {
        w.step();
        if tick % 30 == 0 {
            let reeled = w.pulley_reeled(&d);
            let v = w
                .body_snapshots()
                .into_iter()
                .find(|s| s.handle_index == load.into_raw_parts().0)
                .map(|s| (s.linvel_x * s.linvel_x + s.linvel_y * s.linvel_y).sqrt())
                .unwrap_or(f32::NAN);
            println!(
                "{:>6.2} | {reeled:>12.4} | {:>10.4} | {:>10.4} | {v:>12.4} | {:>12.4}",
                tick as f32 / 60.0,
                d.total_length - reeled,
                w.body_pose(load).unwrap().translation.y,
                8.0 - w.body_pose(load).unwrap().translation.y
            );
        }
    }
}
