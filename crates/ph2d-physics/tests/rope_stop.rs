//! **O LIMITADOR DE CORDA** (W-RopeStop) — a ponta para antes da roldana, e um
//! nó travado impede a corda de correr.
//!
//! A cena é a mesma da sonda `measure_rope_stop.rs`, e de propósito: os números
//! que os gates afirmam são os que aquela tabela imprime.

use ph2d_physics::PhysicsWorld;
use ph2d_physics::RigidBodyHandle;
use ph2d_physics::world::pulley::{PulleyDesc, stop_at_point, stop_leg, stop_mark};
use ph2d_physics::world::rope_route::{self, RopeWheel, Tangent};

const WHEEL_R: f32 = 0.5;
const WHEEL_Y: f32 = 8.0;

/// Um guincho: tambor no teto, carga pendurada, ponta B morta.
fn winch(stops: [f32; 2]) -> (PhysicsWorld, RigidBodyHandle) {
    const BODY_R: f32 = 0.2;
    let mut w = PhysicsWorld::new();
    let area = std::f32::consts::PI * BODY_R * BODY_R;
    let (dead, _) = w.add_static_cuboid(1.5, WHEEL_Y, 0.1, 0.1);
    let (load, _) = w.add_dynamic_circle(-0.6, 2.0, BODY_R, 1.0 / area);
    let mut wheels = vec![RopeWheel {
        centre: [0.0, WHEEL_Y],
        radius: WHEEL_R,
        id: 1,
        ..RopeWheel::default()
    }];
    let mut scratch = Vec::new();
    rope_route::resolve_sides([-0.6, 2.0], [1.5, WHEEL_Y], &mut wheels, &mut scratch);
    let desc = PulleyDesc {
        id: 1,
        body_a: load,
        body_b: dead,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: 1,
        total_length: 0.0,
        motor_rate: 0.5,
        break_force: f32::INFINITY,
        stops: [0.0, 0.0],
    };
    w.set_pulleys(vec![desc], wheels.clone());
    let mut desc = desc;
    desc.total_length = w.pulley_span(&desc).expect("rota sã");
    desc.stops = stops;
    w.set_pulleys(vec![desc], wheels);
    (w, load)
}

/// A folga de TANGENTE: zero quando a amarração toca o aro.
fn gap(w: &PhysicsWorld, body: RigidBodyHandle) -> f32 {
    let p = w.bodies().get(body).expect("corpo").translation();
    let d = p.x.hypot(p.y - WHEEL_Y);
    (d * d - WHEEL_R * WHEEL_R).max(0.0).sqrt()
}

/// **A carga PARA antes da roldana — e sem limitador ela ENTRA nela.**
///
/// O controle é a mesma cena com `stops` em zero, que é o mundo que já shipava:
/// medido, a folga chega a **0,0000 m** (a amarração no aro) e a rota degenera.
#[test]
fn the_load_stops_before_the_wheel_and_without_a_stop_it_does_not() {
    let (mut free, load) = winch([0.0, 0.0]);
    let mut lowest_free = f32::INFINITY;
    for _ in 0..900 {
        free.step();
        lowest_free = lowest_free.min(gap(&free, load));
    }
    assert!(
        lowest_free < 0.01,
        "o CONTROLE tem de encostar na roldana (folga mínima {lowest_free:.4} m) — \
         sem isso este gate não está medindo o defeito"
    );

    let (mut held, load) = winch([1.5, 0.0]);
    let mut lowest = f32::INFINITY;
    for _ in 0..900 {
        held.step();
        lowest = lowest.min(gap(&held, load));
    }
    assert!(
        lowest > 1.4,
        "o limitador de 1,5 m tem de segurar (folga mínima {lowest:.4} m)"
    );
}

/// **Um nó travado impede a corda de CORRER** — a carga não é arrastada por cima
/// do eixo.
///
/// ⚠️ **O oráculo é a ALTURA, e ele não é arbitrário:** com o limitador em 3,0 m a
/// carga fica presa a `√(3² + 0,5²) = 3,04 m` do centro. Um pêndulo solto de
/// baixo **não alcança a horizontal** — chegar à altura do eixo exigiria erguer
/// 3,04 m de energia que ninguém lhe deu. Então `y < WHEEL_Y` é exatamente
/// *"ninguém está dirigindo esta carga"*.
///
/// Medido com a lei desligada (só o empurrão radial, sem tirar a ponta travada da
/// restrição da corda): `y` final **13,458 m** com a roldana a 8,0 — o guincho a
/// arrastava para fora de quadro.
#[test]
fn a_jammed_end_is_not_dragged_over_the_wheel() {
    let (mut w, load) = winch([3.0, 0.0]);
    let mut highest = f32::NEG_INFINITY;
    for _ in 0..900 {
        w.step();
        highest = highest.max(w.bodies().get(load).expect("corpo").translation().y);
    }
    assert!(
        highest < WHEEL_Y,
        "a carga travada subiu a {highest:.3} m, acima do eixo a {WHEEL_Y} — \
         a corda continuou encurtando pelo ARCO"
    );
}

/// **O limitador só AFASTA.** Uma carga longe da roldana não é puxada para ela —
/// `λ ≥ 0`, a mesma desigualdade da corda pelo lado oposto.
#[test]
fn a_stop_never_pulls_the_load_towards_the_wheel() {
    // Sem motor: a corda não recolhe, então nada além do limitador poderia mover
    // a carga PARA CIMA.
    let (mut w, load) = winch([1.0, 0.0]);
    let mut d = w.pulleys()[0];
    d.motor_rate = 0.0;
    let wheels = w.pulley_wheels().to_vec();
    w.set_pulleys(vec![d], wheels);
    let start = w.bodies().get(load).expect("corpo").translation().y;
    for _ in 0..300 {
        w.step();
    }
    let end = w.bodies().get(load).expect("corpo").translation().y;
    assert!(
        end <= start + 1e-3,
        "a carga estava a 6 m da roldana e subiu de {start:.3} para {end:.3} — \
         o limitador puxou em vez de só afastar"
    );
}

/// **`0` é DESLIGADO ao bit** — a trava no próprio aro é onde a corda já podia
/// chegar, então toda cena anterior é byte-idêntica.
#[test]
fn a_stop_of_zero_is_byte_identical_to_no_stop() {
    let (mut a, load_a) = winch([0.0, 0.0]);
    let (mut b, load_b) = winch([0.0, 0.0]);
    for _ in 0..600 {
        a.step();
        b.step();
    }
    let (pa, pb) = (
        a.bodies().get(load_a).expect("corpo").translation(),
        b.bodies().get(load_b).expect("corpo").translation(),
    );
    assert_eq!(
        (pa.x.to_bits(), pa.y.to_bits()),
        (pb.x.to_bits(), pb.y.to_bits())
    );
}

/// **A marca e o número são INVERSAS** — a lei do seed==sample.
///
/// Se o desenho e o arrasto derivarem a posição por caminhos diferentes, a marca
/// salta debaixo do dedo no instante do clique.
#[test]
fn the_mark_and_the_number_are_inverses() {
    let legs = [
        Tangent {
            from: [0.0, 0.0],
            to: [0.0, 4.0],
            dir: [0.0, 1.0],
            len: 4.0,
            weight: 1.0,
        },
        Tangent {
            from: [1.0, 4.0],
            to: [3.0, 4.0],
            dir: [1.0, 0.0],
            len: 2.0,
            weight: 1.0,
        },
    ];
    let wheels = [RopeWheel {
        centre: [0.0, 4.5],
        radius: 0.5,
        id: 1,
        ..RopeWheel::default()
    }];
    let leg = stop_leg(&legs, &wheels, 0).expect("a ponta A tem roldana");
    assert_eq!(leg.anchor, [0.0, 0.0]);
    assert_eq!(leg.touch, [0.0, 4.0]);
    for s in [0.0f32, 0.5, 1.75, 4.0] {
        let p = stop_mark(&leg, s);
        let back = stop_at_point(&leg, p);
        assert!(
            (back - s).abs() < 1e-4,
            "a marca de {s} voltou como {back} — o desenho e o arrasto discordam"
        );
    }
    // Fora do trecho, clampa: a marca não vai para trás do corpo nem para o outro
    // lado da roldana.
    assert!((stop_at_point(&leg, [0.0, -5.0]) - 4.0).abs() < 1e-4);
    assert!(stop_at_point(&leg, [0.0, 9.0]).abs() < 1e-4);
}

/// **Sem roldana não há limitador** — não é um guarda, é a lei: um limitador é
/// uma trava CONTRA uma roldana.
#[test]
fn a_rope_with_no_wheel_has_no_stop() {
    let legs = [Tangent {
        from: [0.0, 0.0],
        to: [3.0, 0.0],
        dir: [1.0, 0.0],
        len: 3.0,
        weight: 1.0,
    }];
    assert!(stop_leg(&legs, &[], 0).is_none());
    assert!(stop_leg(&legs, &[], 1).is_none());
}

/// **A ponta B mede contra a ÚLTIMA roldana**, não contra a primeira — trocar as
/// duas põe a marca do lado errado da corda.
#[test]
fn the_b_end_measures_against_the_last_wheel() {
    let legs = [
        Tangent {
            from: [0.0, 0.0],
            to: [0.0, 4.0],
            dir: [0.0, 1.0],
            len: 4.0,
            weight: 1.0,
        },
        Tangent {
            from: [1.0, 4.0],
            to: [5.0, 4.0],
            dir: [1.0, 0.0],
            len: 4.0,
            weight: 1.0,
        },
        Tangent {
            from: [6.0, 4.0],
            to: [6.0, 1.0],
            dir: [0.0, -1.0],
            len: 3.0,
            weight: 1.0,
        },
    ];
    let wheels = [
        RopeWheel {
            centre: [0.0, 4.5],
            radius: 0.5,
            id: 1,
            ..RopeWheel::default()
        },
        RopeWheel {
            centre: [5.5, 4.5],
            radius: 0.5,
            id: 2,
            ..RopeWheel::default()
        },
    ];
    let b = stop_leg(&legs, &wheels, 1).expect("a ponta B tem roldana");
    assert_eq!(
        b.anchor,
        [6.0, 1.0],
        "a âncora de B é o FIM do último trecho"
    );
    assert_eq!(b.touch, [6.0, 4.0], "ela encosta na ÚLTIMA roldana");
    assert_eq!(b.centre, wheels[1].centre);
    assert_eq!(b.wheel, 1);
}

/// **NUMA RODA GRANDE o gradiente não é o versor do trecho** — e é ele que decide
/// ONDE a trava segura.
///
/// ⚠️ **Este gate existe porque a mutação sobreviveu aos outros sete.** Trocar
/// `(âncora − centro)/len` pelo versor do trecho passa em toda cena de roldana
/// pequena, porque ali os dois quase coincidem: o ângulo entre eles é
/// `acos(len/d)`, que numa roda de raio 0,5 com 1,5 m de folga vale **18°** e
/// numa de raio 2,0 com 0,5 m de folga vale **76°**.
///
/// Medido nesta cena (roda de raio 2,0, guincho a 0,5 m/s):
///
/// | limitador | folga mínima CERTA | com o gradiente errado |
/// |---|---|---|
/// | 0,50 | **0,4883** | **0,0000** (a trava não segura nada) |
/// | 1,00 | **0,9902** | 0,6883 (segura 31% cedo demais) |
///
/// ⚠️ **E o oráculo que eu escrevi primeiro estava ERRADO e o CONTROLE o
/// derrubou:** eu afirmei que a carga não podia ser atirada de lado além do raio,
/// por intuição. Medido, ela vai a `|x| = 3,976 m` **com** o limitador e a
/// **6,345 m sem nenhum** — a trava REDUZ o balanço, e a asserção dizia o oposto
/// do que o produto faz. O que a mutação de fato quebra é a PRECISÃO, e é ela que
/// este gate afirma.
#[test]
fn the_stop_holds_on_a_big_wheel_where_the_gradient_is_not_the_leg() {
    const BIG_R: f32 = 2.0;
    const BODY_R: f32 = 0.2;
    let mut w = PhysicsWorld::new();
    let area = std::f32::consts::PI * BODY_R * BODY_R;
    let (dead, _) = w.add_static_cuboid(3.0, WHEEL_Y, 0.1, 0.1);
    let (load, _) = w.add_dynamic_circle(0.0, 2.0, BODY_R, 1.0 / area);
    let mut wheels = vec![RopeWheel {
        centre: [0.0, WHEEL_Y],
        radius: BIG_R,
        id: 1,
        ..RopeWheel::default()
    }];
    let mut scratch = Vec::new();
    rope_route::resolve_sides([0.0, 2.0], [3.0, WHEEL_Y], &mut wheels, &mut scratch);
    let desc = PulleyDesc {
        id: 1,
        body_a: load,
        body_b: dead,
        local_a: [0.0, 0.0],
        local_b: [0.0, 0.0],
        wheel_start: 0,
        wheel_count: 1,
        total_length: 0.0,
        motor_rate: 0.5,
        break_force: f32::INFINITY,
        stops: [0.0, 0.0],
    };
    w.set_pulleys(vec![desc], wheels.clone());
    let mut desc = desc;
    desc.total_length = w.pulley_span(&desc).expect("rota sã");
    desc.stops = [0.5, 0.0];
    w.set_pulleys(vec![desc], wheels);

    let mut lowest = f32::INFINITY;
    for _ in 0..900 {
        w.step();
        let p = w.bodies().get(load).expect("corpo").translation();
        let d = p.x.hypot(p.y - WHEEL_Y);
        lowest = lowest.min((d * d - BIG_R * BIG_R).max(0.0).sqrt());
    }
    assert!(
        lowest > 0.45,
        "o limitador de 0,5 m numa roda de raio 2,0 segurou em {lowest:.4} m — \
         o gradiente saiu pelo TRECHO em vez de sair do eixo"
    );
}
