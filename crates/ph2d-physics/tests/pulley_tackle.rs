//! **A TALHA** (W-Pulley W3) — a roldana montada num corpo que se move, e a
//! vantagem mecânica que vem com ela.
//!
//! O `ratio` da v1 prometia vantagem e descrevia uma corda que não existe (§3 do
//! plano): numa corda única sobre roldanas livres a tensão é uniforme, então os
//! dois corpos sentem a MESMA força. A vantagem verdadeira volta por onde ela vem
//! no mundo — uma roldana montada num corpo que se move é sustentada por DOIS
//! ramos da mesma corda, e o "2" não está escrito em lugar nenhum: ele é a
//! magnitude do Jacobiano daquele eixo.
//!
//! Números medidos em `tests/measure_pulley_tackle.rs`.

use ph2d_physics::PhysicsWorld;
use ph2d_physics::RigidBodyHandle;
use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route::RopeWheel;

/// A talha (*gun tackle*) do §3 do plano: ponta morta no teto, roldana MÓVEL no
/// bloco, roldana FIXA no teto, contrapeso na outra ponta.
///
/// ```text
///   morta (-0.4, 8)          roldana FIXA (0.4, 8)  [cenário]
///          \                        / \
///           \                      /   \
///            \                    /     contrapeso (0.4, 6)
///             \                  /
///              roldana MÓVEL (0, 4)  [montada no BLOCO]
/// ```
///
/// ⚠️ **A cadernal FIXA é o que torna a talha uma talha**, e a primeira fixture
/// desta wave não a tinha: com a ponta morta e a mão as duas acima do bloco,
/// descer o bloco alonga os DOIS ramos e a mão desce junto — os dois lados
/// liberam energia e não existe equilíbrio nenhum para medir.
///
/// `mounted = false` é o **CONTROLE 1:1**: mesma corda, mesmo contrapeso, mesma
/// roldana fixa; o bloco é amarrado DIRETO na ponta da corda, então UM ramo o
/// segura. A única diferença entre os dois braços é quantos ramos seguram.
fn tackle(
    block_mass: f32,
    haul_mass: f32,
    mounted: bool,
) -> (PhysicsWorld, PulleyDesc, RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    const R: f32 = 0.2;
    let area = std::f32::consts::PI * R * R;
    let (dead, _) = w.add_static_cuboid(-0.4, 8.0, 0.1, 0.1);
    let (block, _) = w.add_dynamic_circle(0.0, 4.0, R, block_mass / area);
    let (haul, _) = w.add_dynamic_circle(0.4, 6.0, R, haul_mass / area);
    let movable = RopeWheel {
        centre: [0.0, 4.0],
        // O eixo no CENTRO do bloco: a corda o levanta sem lhe fazer torque.
        body: Some(block),
        local: [0.0, 0.0],
        id: 1,
        ..RopeWheel::default()
    };
    let fixed = RopeWheel {
        centre: [0.4, 8.0],
        id: 2,
        ..RopeWheel::default()
    };
    let wheels = if mounted {
        vec![movable, fixed]
    } else {
        vec![fixed]
    };
    let desc = PulleyDesc {
        id: 1,
        body_a: if mounted { dead } else { block },
        body_b: haul,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: wheels.len() as u32,
        total_length: 0.0,
        motor_rate: 0.0,
        break_force: f32::INFINITY,
    };
    w.set_pulleys(vec![desc], wheels.clone());
    let mut desc = desc;
    desc.total_length = w.pulley_span(&desc).expect("rota sã no repouso");
    w.set_pulleys(vec![desc], wheels);
    (w, desc, block)
}

fn y(w: &PhysicsWorld, h: RigidBodyHandle) -> f32 {
    w.bodies().get(h).map_or(f32::NAN, |b| b.translation().y)
}

/// Quanto o bloco subiu (positivo) em `ticks` tiques.
fn rise(block_mass: f32, haul_mass: f32, mounted: bool, ticks: u32) -> f32 {
    let (mut w, _, block) = tackle(block_mass, haul_mass, mounted);
    let y0 = y(&w, block);
    for _ in 0..ticks {
        w.step();
    }
    y(&w, block) - y0
}

/// **A entrega da wave: metade do contrapeso segura o mesmo bloco.**
///
/// O oráculo é uma tabela 2×2 e não um número: o MESMO contrapeso dá vereditos
/// OPOSTOS conforme a roldana esteja montada no bloco ou não. Um gate de razão
/// entre dois braços doentes seria verde por construção; aqui cada célula é uma
/// afirmação física independente.
///
/// | contrapeso | talha (2 ramos) | 1:1 (um ramo) |
/// |---|---|---|
/// | 1 kg | **equilíbrio** (−0,009 m) | cai (−1,65 m) |
/// | 2 kg | sobe (+0,97 m) | **equilíbrio** (−0,019 m) |
#[test]
fn the_movable_block_needs_half_the_counterweight() {
    const BLOCK: f32 = 2.0;
    // Metade da massa do bloco: a talha SEGURA, o 1:1 não.
    let half_tackle = rise(BLOCK, 1.0, true, 60);
    let half_plain = rise(BLOCK, 1.0, false, 60);
    assert!(
        half_tackle.abs() < 0.05,
        "a talha tinha de equilibrar com metade do peso; subiu {half_tackle:.4} m"
    );
    assert!(
        half_plain < -1.0,
        "o 1:1 com metade do peso tinha de CAIR; moveu {half_plain:.4} m"
    );
    // A massa inteira: agora é o 1:1 que segura, e a talha ERGUE.
    let full_tackle = rise(BLOCK, 2.0, true, 60);
    let full_plain = rise(BLOCK, 2.0, false, 60);
    assert!(
        full_tackle > 0.5,
        "a talha com o peso inteiro tinha de ERGUER; moveu {full_tackle:.4} m"
    );
    assert!(
        full_plain.abs() < 0.05,
        "o 1:1 tinha de equilibrar com o peso inteiro; moveu {full_plain:.4} m"
    );
}

/// **O eixo entra na MASSA EFETIVA — senão a corda ARREMESSA o bloco.**
///
/// Uma roldana montada é mais uma ponta da restrição, então ela entra no `k` do
/// `λ = (Ċ + β·C/dt) / k` **e** no impulso. Esquecer o `k` deixa o sistema
/// *estável e errado* de uma maneira que só um relógio vê: a massa efetiva sai
/// pequena demais, `λ` grande demais, e o passe deixa de ser uma projeção para
/// virar um ganho — que é exatamente o que o cabeçalho do `pulley.rs` diz que
/// esta arquitetura não faz.
///
/// **MEDIDO** (bloco 2 kg, contrapeso 0,25 kg, 2 s):
///
/// | | desce até | excursão para CIMA | assenta? |
/// |---|---|---|---|
/// | produto | −1,0055 m | **0,0000** | sim, em ~0,7 s |
/// | sem o `k` do eixo | −1,03 m | **+3,75 m** | não, oscila |
///
/// O oráculo é FÍSICO e não um limiar escolhido: um bloco pesado com um
/// contrapeso leve **não sobe**, e um sistema com massa efetiva exata **assenta**.
#[test]
fn the_axle_enters_the_effective_mass() {
    let (mut w, _, block) = tackle(2.0, 0.25, true);
    let y0 = y(&w, block);
    let mut peak_up = 0.0_f32;
    let mut settled = y0;
    for tick in 0..120 {
        w.step();
        peak_up = peak_up.max(y(&w, block) - y0);
        if tick == 89 {
            settled = y(&w, block);
        }
    }
    assert!(
        peak_up <= 0.0,
        "a corda ARREMESSOU o bloco {peak_up:.4} m acima de onde ele começou"
    );
    let drift = (y(&w, block) - settled).abs();
    assert!(
        drift < 0.001,
        "o sistema tinha de assentar; andou {drift:.4} m no último meio segundo"
    );
    let fell = y0 - y(&w, block);
    assert!(
        fell > 0.5,
        "a fixture tem de conter o fenômeno: o bloco desceu só {fell:.4} m"
    );
}

/// **A roldana montada anda com o corpo — na ARENA**, que é a lista que o
/// DESENHO lê.
///
/// ⚠️ Refrescar uma cópia deixaria o solver certo e o overlay desenhando a corda
/// passando por onde a roldana **não está** — a segunda opinião que o doc do
/// `physics_overlay_pulley` proíbe em voz alta.
///
/// O gate mede também que o bloco de fato SE MOVEU: um bloco parado tornaria
/// *"a roda o acompanha"* verdadeiro sem que nada acompanhasse nada.
#[test]
fn the_mounted_wheel_rides_the_body() {
    let (mut w, _, block) = tackle(2.0, 0.25, true);
    let y0 = y(&w, block);
    let mut worst = 0.0_f32;
    for _ in 0..60 {
        w.step();
        let d = w.pulley_wheels()[0].centre[1] - y(&w, block);
        worst = worst.max(d.abs());
    }
    // ⚠️ **DESCEU**, com sinal: um bloco pesado sob um contrapeso leve desce, e
    // exigir só `|Δy|` deixaria um bloco ARREMESSADO para cima (o modo de falha da
    // massa efetiva incompleta) satisfazer *"a fixture contém o fenômeno"*.
    let fell = y0 - y(&w, block);
    assert!(
        fell > 0.5,
        "o bloco tinha de DESCER; moveu {fell:.4} m (negativo = a corda o ergueu)"
    );
    // A folga é de UM sub-passo: a arena é refrescada no topo do sub-passo e o
    // rapier integra o corpo depois dele. A 240 Hz e a ~3 m/s isso são ~12 mm.
    assert!(
        worst < 0.05,
        "o eixo tinha de seguir o bloco; ficou {worst:.4} m atrás"
    );
}

/// **Sem corpo, os campos de montagem são INERTES** — e é isso que mantém toda
/// cena anterior a esta wave byte-idêntica.
///
/// Um `local` absurdo numa roldana de cenário não pode mover um único bit: se ele
/// movesse, alguma rota estaria lendo a montagem sem perguntar se ela existe.
#[test]
fn the_mount_fields_are_inert_without_a_body() {
    let run = |local: [f32; 2]| {
        let mut w = PhysicsWorld::new();
        const R: f32 = 0.2;
        let area = std::f32::consts::PI * R * R;
        let (a, _) = w.add_dynamic_circle(-1.0, 2.0, R, 1.0 / area);
        let (b, _) = w.add_dynamic_circle(1.0, 2.0, R, 3.0 / area);
        let wheels = vec![
            RopeWheel {
                centre: [-1.0, 6.0],
                local,
                ..RopeWheel::default()
            },
            RopeWheel {
                centre: [1.0, 6.0],
                local,
                ..RopeWheel::default()
            },
        ];
        let desc = PulleyDesc {
            id: 1,
            body_a: a,
            body_b: b,
            local_a: [0.0, 0.0],
            local_b: [0.0, 0.0],
            wheel_start: 0,
            wheel_count: 2,
            total_length: 10.0,
            motor_rate: 0.0,
            break_force: f32::INFINITY,
        };
        w.set_pulleys(vec![desc], wheels);
        for _ in 0..90 {
            w.step();
        }
        (y(&w, a), y(&w, b), w.pulley_wheels()[0].centre)
    };
    let neutral = run([0.0, 0.0]);
    let absurd = run([99.0, -99.0]);
    assert_eq!(
        neutral, absurd,
        "uma roldana de cenário leu o `local` que ela não tem"
    );
}

/// **A arena é DERIVADA, então o checkpoint não precisa dela** — e este gate é o
/// que torna essa frase conferida em vez de assumida.
///
/// O centro de uma roldana montada é reescrito no topo de todo sub-passo a partir
/// da pose viva do corpo. Logo, restaurar um mundo antigo e dar um passo tem de
/// devolver a roldana ao corpo restaurado, por mais longe que a arena tenha ido
/// enquanto isso. Se algum dia o centro passar a ser acumulado em vez de derivado,
/// é aqui que aparece.
#[test]
fn a_restored_world_puts_the_wheel_back_on_the_body() {
    let (mut w, _, block) = tackle(2.0, 0.25, true);
    for _ in 0..10 {
        w.step();
    }
    let cp = w.checkpoint();
    let early = y(&w, block);
    for _ in 0..50 {
        w.step();
    }
    let late = y(&w, block);
    assert!(
        (early - late) > 0.3,
        "a fixture tem de conter o fenômeno: o bloco quase não andou ({:.4} m)",
        early - late
    );
    w.restore(&cp);
    w.step();
    let d = w.pulley_wheels()[0].centre[1] - y(&w, block);
    assert!(
        d.abs() < 0.05,
        "depois do restore o eixo ficou {d:.4} m longe do corpo"
    );
}
