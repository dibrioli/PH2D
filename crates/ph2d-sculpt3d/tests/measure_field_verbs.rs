//! **O QUE OS CINCO VERBOS DE CAMPO FAZEM AO BARRO** — a sonda da W5-B.
//!
//! ⚠️ **Ela dirige a porta do PRODUTO** (`SculptStroke::dab`), nunca os kernels:
//! a §7.11 acabou de pagar duas vezes por medir peça isolada — a tabela que
//! escolheu o `KELVINLET_REACH` mediu num eixo só, e a recusa que ela escreveu
//! era uma ESTIMATIVA. O que decide um chip é o que o artista vê, e o que o
//! artista vê sai daqui.
//!
//! Rodar: `cargo test -p ph2d-sculpt3d --release --test measure_field_verbs -- --ignored --nocapture`

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Brush, Dab, RefMode, SculptStroke, Symmetry, Verb};

fn sphere() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(32, 48, 1.0)
}

/// O centro do dab, no polo `+z` da esfera.
const TIP: [f32; 3] = [0.0, 0.0, 1.0];
const EYE: [f32; 3] = [0.0, 0.0, -1.0];
const R: f32 = 0.5;

/// Um traço de `steps` eventos com o gesto a crescer linearmente — o caminho do
/// produto, não um dab solto.
fn stroke(verb: Verb, mode: RefMode, amount: f32, pull: [f32; 3], steps: usize) -> Mesh {
    let mut mesh = sphere();
    let b = Brush {
        verb,
        mode,
        radius: R,
        strength: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 1..=steps {
        let t = k as f32 / steps as f32;
        let d = if verb == Verb::SnakeHook {
            // O gancho quer a ÂNCORA a andar — é o que o separa do Move.
            Dab::pulling(
                [TIP[0] + pull[0] * t, TIP[1], TIP[2]],
                R,
                EYE,
                [
                    pull[0] / steps as f32,
                    pull[1] / steps as f32,
                    pull[2] / steps as f32,
                ],
            )
        } else if verb == Verb::Move {
            Dab::pulling(TIP, R, EYE, [pull[0] * t, pull[1] * t, pull[2] * t])
        } else {
            let mut d = Dab::pulling(TIP, R, EYE, [0.0; 3]);
            d.amount = amount * t;
            d
        };
        s.dab(&mut mesh, &b, &d, Symmetry::default());
    }
    mesh
}

/// Quantos vértices se moveram e quanto barro andou no total.
fn moved(rest: &Mesh, out: &Mesh) -> (usize, f32) {
    let (a, b) = (rest.positions(), out.positions());
    let mut n = 0;
    let mut sum = 0.0;
    for i in 0..a.len() {
        let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
        let m = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if m > 1e-6 {
            n += 1;
            sum += m;
        }
    }
    (n, sum)
}

/// ⚠️ **A pergunta que decide o desenho do Twist e do LocalScale:** um giro tem
/// de PRESERVAR a distância ao eixo. Devolve o maior crescimento relativo do
/// raio (medido do eixo que passa pela âncora na direção do olho) entre os
/// vértices que de facto se moveram — `1,0` é um giro honesto.
fn worst_radius_growth(rest: &Mesh, out: &Mesh) -> f32 {
    let (a, b) = (rest.positions(), out.positions());
    let mut worst: f32 = 1.0;
    for i in 0..a.len() {
        let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
        if (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() <= 1e-6 {
            continue;
        }
        // O eixo é `-eye` e passa por TIP: a distância a ele é a norma da
        // componente perpendicular, que aqui é simplesmente o raio em `xy`.
        let ra = ((a[i][0] - TIP[0]).powi(2) + (a[i][1] - TIP[1]).powi(2)).sqrt();
        let rb = ((b[i][0] - TIP[0]).powi(2) + (b[i][1] - TIP[1]).powi(2)).sqrt();
        if ra > 1e-4 {
            worst = worst.max(rb / ra);
        }
    }
    worst
}

#[test]
#[ignore = "sonda"]
fn what_the_five_field_verbs_do_to_the_clay() {
    let rest = sphere();
    println!("\n  verbo        modo | vértices |     soma | maior crescimento do raio");
    println!("  ------------------|----------|----------|--------------------------");
    for (verb, amount, pull) in [
        (Verb::SnakeHook, 0.0, [0.35, 0.0, 0.0]),
        (Verb::Twist, 1.2, [0.0; 3]),
        (Verb::LocalScale, 0.5, [0.0; 3]),
        (Verb::Pinch, 0.0, [0.0; 3]),
        (Verb::Magnify, 0.0, [0.0; 3]),
    ] {
        for mode in [RefMode::S, RefMode::L] {
            let out = stroke(verb, mode, amount, pull, 12);
            let (n, sum) = moved(&rest, &out);
            println!(
                "  {:<12} {:>4} | {:>8} | {:>8.3} | {:>8.4}",
                verb.label(),
                mode.label(),
                n,
                sum,
                worst_radius_growth(&rest, &out)
            );
        }
    }
}

/// ⚠️ **O NÚMERO QUE JUSTIFICA O PERFIL ESCALAR.** Entregar a torção como
/// DESLOCAMENTO (`base + perfil · (ω × r)`) é a rota que o `kelvinlet::twist`
/// oferece; entregá-la como ÂNGULO é a que o produto usa. As duas concordam no
/// limite de ângulo pequeno e divergem onde o artista trabalha — e a divergência
/// tem um nome: o barro INCHA.
#[test]
#[ignore = "sonda"]
fn the_displacement_route_inflates_the_clay_and_the_angle_route_does_not() {
    use ph2d_sculpt3d::kelvinlet::{Scales, rigid_profile, twist};
    println!("\n  θ (rad) | deslocamento | ângulo  | inflação prevista √(1+θ²)");
    println!("  --------|--------------|---------|---------------------------");
    // Um ponto a meio raio do bico, perpendicular ao eixo.
    let r = [R * 0.5, 0.0, 0.0];
    let axis = [0.0, 0.0, 1.0];
    let r0 = (r[0] * r[0] + r[1] * r[1]).sqrt();
    for theta in [0.1_f32, 0.25, 0.5, 1.0, 2.0] {
        let omega = [axis[0] * theta, axis[1] * theta, axis[2] * theta];
        let u = twist(r, R, omega, Scales::default());
        let disp = [r[0] + u[0], r[1] + u[1]];
        let by_disp = (disp[0] * disp[0] + disp[1] * disp[1]).sqrt() / r0;
        // A rota do produto: gira o MESMO perfil como ângulo.
        let p = rigid_profile(r, R, Scales::default());
        let a = theta * p;
        let rot = [
            r[0] * a.cos() - r[1] * a.sin(),
            r[0] * a.sin() + r[1] * a.cos(),
        ];
        let by_angle = (rot[0] * rot[0] + rot[1] * rot[1]).sqrt() / r0;
        println!(
            "  {:>7.2} | {:>12.4} | {:>7.4} | {:>9.4}",
            theta,
            by_disp,
            by_angle,
            (1.0 + (theta * p).powi(2)).sqrt()
        );
    }
}

/// ⚠️ **O que separa o `l-mode` do Pinch do `s-mode`: o VOLUME.** O puxão
/// lateral leva barro para dentro e não o devolve; a `F` de traço zero espirra
/// pela normal o que aperta no plano. Mede as duas componentes.
#[test]
#[ignore = "sonda"]
fn the_pinch_gives_back_along_the_normal_what_it_takes_from_the_plane() {
    let rest = sphere();
    println!("\n  modo | lateral (para dentro) | normal (para fora)");
    println!("  -----|-----------------------|--------------------");
    for mode in [RefMode::S, RefMode::L] {
        let out = stroke(Verb::Pinch, mode, 0.0, [0.0; 3], 12);
        let (a, b) = (rest.positions(), out.positions());
        let (mut lat, mut nrm) = (0.0f32, 0.0f32);
        for i in 0..a.len() {
            let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
            if (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() <= 1e-6 {
                continue;
            }
            // A normal do dab é `+z` no polo; o plano é `xy`.
            let radial = [a[i][0] - TIP[0], a[i][1] - TIP[1]];
            let rn = (radial[0] * radial[0] + radial[1] * radial[1]).sqrt();
            if rn > 1e-4 {
                lat += (d[0] * radial[0] + d[1] * radial[1]) / rn;
            }
            nrm += d[2];
        }
        println!("  {:>4} | {:>21.4} | {:>18.4}", mode.label(), lat, nrm);
    }
}

/// ⚠️ **O que a MUTAÇÃO pode fingir e o que ela não pode.** Tirar o braço de
/// campo de um verbo faz o alvo cair no modo que já shipava — mas a PEGADA
/// continua a do campo e a curva continua a INDICADORA, ou seja todo vértice de
/// uma pegada 3× mais larga leva o gesto INTEIRO. Somar sobre a pegada mede a
/// pegada; o que separa os dois é o **PERFIL**.
#[test]
#[ignore = "sonda"]
fn what_the_missing_field_arm_can_fake() {
    let rest = sphere();
    for (verb, pull) in [
        (Verb::SnakeHook, [0.35f32, 0.0, 0.0]),
        (Verb::Pinch, [0.0; 3]),
    ] {
        println!("\n  {} — deslocamento por banda de distância", verb.label());
        println!("  modo | 0,1-0,4 R | 0,9-1,1 R | 1,5-2,0 R | aro÷bico");
        for mode in [RefMode::S, RefMode::L] {
            let out = stroke(verb, mode, 0.0, pull, 12);
            let (a, b) = (rest.positions(), out.positions());
            let band = |lo: f32, hi: f32| -> f32 {
                let (mut sum, mut n) = (0.0f32, 0usize);
                for i in 0..a.len() {
                    let r = [a[i][0] - TIP[0], a[i][1] - TIP[1], a[i][2] - TIP[2]];
                    let rm = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
                    if rm < lo * R || rm >= hi * R {
                        continue;
                    }
                    let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
                    sum += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                    n += 1;
                }
                if n == 0 { 0.0 } else { sum / n as f32 }
            };
            let (tip, mid, rim) = (band(0.1, 0.4), band(0.9, 1.1), band(1.5, 2.0));
            println!(
                "  {:>4} | {:>9.4} | {:>9.4} | {:>9.4} | {:>8.4}",
                mode.label(),
                tip,
                mid,
                rim,
                if tip > 0.0 { rim / tip } else { 0.0 }
            );
        }
    }
}
