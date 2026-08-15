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

/// ⚠️ **ONDE O CAMPO SALTA** — o report do Enio (*"modo L, o falloff parece ter
/// borda dura"*, com estrias em escada num arco).
///
/// Uma borda dura é uma **descontinuidade**: dois vértices vizinhos que recebem
/// deslocamentos muito diferentes. A malha desenha isso como escada porque só há
/// vértices em posições discretas. Esta sonda percorre as ARESTAS e diz qual é o
/// maior salto e **a que distância da âncora ele está** — que é o que separa as
/// hipóteses (a indicadora corta em `REACH·raio`; o anel do cursor é `raio`).
#[test]
#[ignore = "sonda"]
fn where_the_field_jumps() {
    let rest = sphere();
    let mut tris: Vec<[u32; 3]> = Vec::new();
    rest.triangle_indices(&mut tris);
    for mode in [RefMode::S, RefMode::L] {
        let out = stroke(Verb::Move, mode, 0.0, [0.35, 0.0, 0.0], 12);
        let (a, b) = (rest.positions(), out.positions());
        let disp = |i: usize| {
            let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        };
        let dist = |i: usize| {
            let r = [a[i][0] - TIP[0], a[i][1] - TIP[1], a[i][2] - TIP[2]];
            (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt() / R
        };
        // O maior salto POR ARESTA, normalizado pelo comprimento da aresta.
        let mut worst = (0.0f32, 0.0f32, 0.0f32);
        // E o histograma do salto por banda de distância, para ver se ele tem
        // um LUGAR ou está espalhado.
        let mut bands = vec![(0.0f32, 0usize); 20];
        for t in &tris {
            for (u, v) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                let (u, v) = (u as usize, v as usize);
                let len = ((a[u][0] - a[v][0]).powi(2)
                    + (a[u][1] - a[v][1]).powi(2)
                    + (a[u][2] - a[v][2]).powi(2))
                .sqrt();
                if len < 1e-6 {
                    continue;
                }
                let jump = (disp(u) - disp(v)).abs() / len;
                let at = 0.5 * (dist(u) + dist(v));
                if jump > worst.0 {
                    worst = (jump, at, len);
                }
                let bi = ((at / 4.0) * 20.0) as usize;
                if bi < 20 {
                    bands[bi].0 = bands[bi].0.max(jump);
                    bands[bi].1 += 1;
                }
            }
        }
        println!(
            "\n  {} — maior salto {:.4} por unidade de aresta, a {:.2} raios da âncora",
            mode.label(),
            worst.0,
            worst.1
        );
        println!("  banda (raios) | maior salto | arestas");
        for (i, (j, n)) in bands.iter().enumerate() {
            if *n == 0 {
                continue;
            }
            let lo = i as f32 * 4.0 / 20.0;
            if *j > 0.02 {
                println!("  {:.2}-{:.2}   | {:>10.4} | {:>6}", lo, lo + 0.2, j, n);
            }
        }
    }
}

/// ⚠️ **O TAMANHO do degrau na fronteira da pegada**, em unidades de mundo — o
/// que a sonda do gradiente localizou, agora medido como o artista o vê.
#[test]
#[ignore = "sonda"]
fn how_big_is_the_cliff_at_the_footprint_edge() {
    let rest = sphere();
    let out = stroke(Verb::Move, RefMode::L, 0.0, [0.35, 0.0, 0.0], 12);
    let (a, b) = (rest.positions(), out.positions());
    println!("\n  r/raio | deslocamento médio | n");
    let band = |lo: f32, hi: f32| -> (f32, usize) {
        let (mut s, mut n) = (0.0f32, 0usize);
        for i in 0..a.len() {
            let r = [a[i][0] - TIP[0], a[i][1] - TIP[1], a[i][2] - TIP[2]];
            let rm = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt() / R;
            if rm < lo || rm >= hi {
                continue;
            }
            let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
            s += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            n += 1;
        }
        (if n == 0 { 0.0 } else { s / n as f32 }, n)
    };
    for k in 0..16 {
        let lo = k as f32 * 0.2;
        let (m, n) = band(lo, lo + 0.2);
        println!("  {:.2}-{:.2} | {:>18.5} | {}", lo, lo + 0.2, m, n);
    }

    // ⚠️ O CONTROLE: o s-mode chega a zero na fronteira DELE (1,0 raio) do mesmo
    // jeito, ou desce liso? Sem esta metade, "o campo cliffa" nao e um achado —
    // e uma observacao sobre um numero que talvez todo modo tenha.
    let out_s = stroke(Verb::Move, RefMode::S, 0.0, [0.35, 0.0, 0.0], 12);
    let bs = out_s.positions();
    println!("\n  CONTROLE s-mode (pegada = 1,0 raio)");
    println!("  r/raio | deslocamento medio | n");
    for k in 0..10 {
        let lo = 0.6 + k as f32 * 0.1;
        let (hi, mut sum, mut n) = (lo + 0.1, 0.0f32, 0usize);
        for i in 0..a.len() {
            let r = [a[i][0] - TIP[0], a[i][1] - TIP[1], a[i][2] - TIP[2]];
            let rm = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt() / R;
            if rm < lo || rm >= hi {
                continue;
            }
            let d = [bs[i][0] - a[i][0], bs[i][1] - a[i][1], bs[i][2] - a[i][2]];
            sum += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            n += 1;
        }
        let m = if n == 0 { 0.0 } else { sum / n as f32 };
        println!("  {:.2}-{:.2} | {:>18.5} | {}", lo, hi, m, n);
    }
    // A aresta média da malha, para saber se o degrau é visível.
    let mut tris: Vec<[u32; 3]> = Vec::new();
    rest.triangle_indices(&mut tris);
    let mut el = 0.0f32;
    let mut ne = 0usize;
    for t in &tris {
        for (u, v) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            let (u, v) = (u as usize, v as usize);
            el += ((a[u][0] - a[v][0]).powi(2)
                + (a[u][1] - a[v][1]).powi(2)
                + (a[u][2] - a[v][2]).powi(2))
            .sqrt();
            ne += 1;
        }
    }
    println!(
        "\n  aresta média {:.4} · raio da esfera 1,0 · raio do pincel {R}",
        el / ne as f32
    );
}

/// ⚠️ **O RESIDUO NA FRONTEIRA, por familia e por alcance** — o numero que
/// decide se o corte da indicadora e de graca ou e um penhasco.
#[test]
#[ignore = "sonda"]
fn what_the_field_still_carries_where_we_cut_it() {
    use ph2d_sculpt3d::kelvinlet::{Scales, rigid_profile};
    // A familia Quad: multiplos 1..4, pesos do sistema
    //   Sw = 0 · Sw m^2 = 0 · Sw m^4 = 0  =>  mata 1/r, 1/r^3 E 1/r^5.
    const QUAD: [(f32, f32); 4] = [(7.0, 1.0), (-14.0, 2.0), (9.0, 3.0), (-2.0, 4.0)];
    // ⚠️ Montada pela API PUBLICA: `radial(r,e) = rigid_profile(r,e,Mono)·2,5/e³`,
    // entao a Quad e uma combinacao de perfis Mono — sem abrir nada do kernel.
    fn quad_profile(r: f32, eps: f32) -> f32 {
        let (mut num, mut norm) = (0.0f32, 0.0f32);
        for &(w, m) in &QUAD {
            let e = eps * m;
            let inv = 2.5 / (e * e * e);
            num += w * rigid_profile([r, 0.0, 0.0], e, Scales::Mono) * inv;
            norm += w * inv;
        }
        num / norm
    }
    println!("\n  perfil normalizado (1,0 = bico), eps = 1");
    println!("  r/eps |     Mono |       Bi |      Tri |     Quad");
    for k in 1..=8 {
        let r = k as f32;
        let m = rigid_profile([r, 0.0, 0.0], 1.0, Scales::Mono);
        let b = rigid_profile([r, 0.0, 0.0], 1.0, Scales::Bi);
        let t = rigid_profile([r, 0.0, 0.0], 1.0, Scales::Tri);
        let q = quad_profile(r, 1.0);
        println!("  {r:>5.0} | {m:>8.5} | {b:>8.5} | {t:>8.5} | {q:>8.5}");
    }
    println!("\n  ⚠️ o corte de hoje e r/eps = 3 (KELVINLET_REACH)");
}

/// ⚠️ **UM dab contra DOZE** — separa o que o CAMPO faz do que o TRACO faz.
#[test]
#[ignore = "sonda"]
fn one_dab_against_twelve() {
    use ph2d_sculpt3d::kelvinlet::{Scales, grab, rigid_profile};
    let rest = sphere();
    let a = rest.positions().to_vec();
    let prof = |out: &ph2d_mesh::Mesh, tag: &str| {
        let b = out.positions();
        println!("\n  {tag}");
        println!("  r/raio | desloc medio | razao/bico");
        let mut tip = 0.0f32;
        for k in 0..16 {
            let (lo, hi) = (k as f32 * 0.2, k as f32 * 0.2 + 0.2);
            let (mut s, mut n) = (0.0f32, 0usize);
            for i in 0..a.len() {
                let r = [a[i][0] - TIP[0], a[i][1] - TIP[1], a[i][2] - TIP[2]];
                let rm = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt() / R;
                if rm < lo || rm >= hi {
                    continue;
                }
                let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
                s += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                n += 1;
            }
            let m = if n == 0 { 0.0 } else { s / n as f32 };
            if k == 0 {
                tip = m;
            }
            println!(
                "  {lo:.2}-{hi:.2} | {m:>12.5} | {:>10.5}",
                if tip > 0.0 { m / tip } else { 0.0 }
            );
        }
    };
    prof(
        &stroke(Verb::Move, RefMode::L, 0.0, [0.35, 0.0, 0.0], 1),
        "UM evento",
    );
    prof(
        &stroke(Verb::Move, RefMode::L, 0.0, [0.35, 0.0, 0.0], 12),
        "DOZE eventos",
    );

    // O que o KERNEL diz, nas duas leituras que existem.
    println!("\n  r/eps | rigid_profile | |grab|/|grab(0)|");
    let g0 = grab([1e-6, 0.0, 0.0], 1.0, [1.0, 0.0, 0.0], Scales::Tri);
    let n0 = (g0[0] * g0[0] + g0[1] * g0[1] + g0[2] * g0[2]).sqrt();
    for k in 1..=6 {
        let r = k as f32;
        let p = rigid_profile([r, 0.0, 0.0], 1.0, Scales::Tri);
        let g = grab([r, 0.0, 0.0], 1.0, [1.0, 0.0, 0.0], Scales::Tri);
        let gn = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
        println!("  {r:>5.0} | {p:>13.5} | {:>17.5}", gn / n0);
    }
}

/// ⚠️ **O DEGRAU no corte, medido num anel FINO** — a media de banda mistura o
/// interior; o que decide e o ultimo anel antes do corte contra o bico.
#[test]
#[ignore = "sonda"]
fn the_step_at_the_cut_in_a_thin_shell() {
    let rest = sphere();
    let a = rest.positions().to_vec();
    let shell = |b: &[[f32; 3]], lo: f32, hi: f32| -> (f32, usize) {
        let (mut s, mut n) = (0.0f32, 0usize);
        for i in 0..a.len() {
            let r = [a[i][0] - TIP[0], a[i][1] - TIP[1], a[i][2] - TIP[2]];
            let rm = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt() / R;
            if rm < lo || rm >= hi {
                continue;
            }
            let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
            s += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            n += 1;
        }
        (if n == 0 { 0.0 } else { s / n as f32 }, n)
    };
    for (mode, cut, tag) in [(RefMode::S, 1.0f32, "S"), (RefMode::L, 3.0, "L")] {
        let out = stroke(Verb::Move, mode, 0.0, [0.35, 0.0, 0.0], 12);
        let b = out.positions();
        let (tip, _) = shell(b, 0.0, 0.25);
        let (inner, ni) = shell(b, cut - 0.12, cut);
        let (outer, no) = shell(b, cut, cut + 0.12);
        println!(
            "  {tag}: bico {tip:.5} | ultimo anel DENTRO {inner:.5} ({ni} vts) \
             | 1o anel FORA {outer:.5} ({no} vts) | degrau = {:.2}% do bico",
            100.0 * (inner - outer) / tip
        );
    }
}

/// **O PERFIL DO VINCO** — a sonda que decide se o `Crease` pode ter `l-mode`.
///
/// ⚠️ **O Crease é uma COMPOSIÇÃO** (a matriz §3 pede *Draw + Kelvinlets
/// pinch*), e é isso que o separa dos cinco verbos da W5-B: metade dele é um
/// APERTO lateral e a outra metade é um AFUNDAMENTO pela normal. A lei que os
/// cinco usam — *"com campo, a curva é o SUPORTE do campo"* — foi escrita para
/// quem tem o deslocamento INTEIRO vindo do kernel; num verbo composto ela
/// alcança também a metade que **não** é do campo.
///
/// Por isso a sonda mede as duas metades SEPARADAS, ao longo de um raio:
/// **quanto o vértice afundou** (a projeção na normal do polo) e **quanto ele
/// andou de lado** (a componente no plano). Um vinco é FUNDO e ESTREITO; um
/// vinco que afunda igual até três raios não é um vinco, é uma cratera.
///
/// Rodar: `cargo test -p ph2d-sculpt3d --release --test measure_field_verbs
/// -- --ignored --nocapture measure_the_crease_trench`
#[test]
#[ignore = "sonda de medição"]
fn measure_the_crease_trench() {
    // Anéis de distância ao bico, em fração do RAIO do pincel — até 3×, que é
    // o que um campo pediria de pegada.
    const BANDS: [f32; 7] = [0.25, 0.5, 0.75, 1.0, 1.5, 2.0, 2.9];

    for mode in [RefMode::S, RefMode::B, RefMode::L] {
        let before = sphere();
        let after = stroke(Verb::Crease, mode, 1.0, [0.0, 0.0, 0.0], 6);

        println!("\n=== Crease {mode:?} ===");
        println!("  banda(r)      n    afundou     lateral");
        for w in BANDS.windows(2) {
            let (lo, hi) = (w[0] * R, w[1] * R);
            let mut n = 0usize;
            let (mut dig, mut lat) = (0.0f64, 0.0f64);
            for (i, p0) in before.positions().iter().enumerate() {
                let d = [p0[0] - TIP[0], p0[1] - TIP[1], p0[2] - TIP[2]];
                let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                if dist < lo || dist >= hi {
                    continue;
                }
                let p1 = after.positions()[i];
                let m = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
                // A normal do polo é +z: afundar é andar em −z.
                let along = -m[2];
                let side = (m[0] * m[0] + m[1] * m[1]).sqrt();
                n += 1;
                dig += f64::from(along);
                lat += f64::from(side);
            }
            if n == 0 {
                continue;
            }
            let inv = 1.0 / n as f64;
            println!(
                "  {:.2}-{:.2} {:>6}  {:>9.5}  {:>9.5}",
                w[0],
                w[1],
                n,
                dig * inv,
                lat * inv
            );
        }
    }
}

/// **AS TRÊS ESCALAS MUDAM O BARRO?** — a pergunta que decide se o
/// `Elastic Deform` tem conteúdo, e a lei §3 do plano aplicada a um knob.
///
/// O `Elastic Deform` do Blender oferece **Grab · Grab Biscale · Grab Triscale ·
/// Scale · Twist**. Os quatro tipos de deformação já existem aqui como VERBOS
/// com `l-mode` (Move · Twist · Local Scale · Pinch); o que a nossa tabela não
/// tem é a escolha entre `Mono`/`Bi`/`Tri`, hoje presa no `Scales::default()`.
///
/// Rodar: `cargo test -p ph2d-sculpt3d --release --test measure_field_verbs
/// -- --ignored --nocapture measure_whether_the_scales_change_the_clay`
#[test]
#[ignore = "sonda de medição"]
fn measure_whether_the_scales_change_the_clay() {
    use ph2d_sculpt3d::kelvinlet::{Scales, grab};

    // O deslocamento do campo de agarre ao longo de um raio, por família.
    // ⚠️ Medido no KERNEL e não no barro, e o motivo é honesto: a família não
    // é autorável hoje, então não há porta de produto para a sonda dirigir.
    // O que ela responde é a pergunta ANTERIOR — *há conteúdo a expor?*
    const EPS: f32 = 0.5;
    let pull = [1.0f32, 0.0, 0.0];
    println!("\n=== o agarre por família de escala (ε = {EPS}) ===");
    println!("   r/ε      Mono        Bi       Tri     Bi/Mono   Tri/Mono");
    for k in [0.0f32, 0.25, 0.5, 1.0, 1.5, 2.0, 2.5, 2.9] {
        let r = [k * EPS, 0.0, 0.0];
        let m = grab(r, EPS, pull, Scales::Mono)[0];
        let b = grab(r, EPS, pull, Scales::Bi)[0];
        let t = grab(r, EPS, pull, Scales::Tri)[0];
        println!(
            "  {k:>4.2}  {m:>8.5}  {b:>8.5}  {t:>8.5}   {:>7.4}   {:>7.4}",
            b / m,
            t / m
        );
    }

    // ⚠️ **A 1ª versão desta segunda tabela perguntava a coisa ERRADA** — ela
    // computava *"que alcance esta família precisa para deixar o resíduo que a
    // Tri deixa em 3"* (Mono 28,8 · Bi 4,2), e eu ia fazer o `KELVINLET_REACH`
    // virar função da família por causa dela. O doc-comment do
    // [`rim_landing`] já tinha REFUTADO esse movimento, com medição:
    // *"alargar o alcance NÃO é a cura — 4 deixa 1,19 %, 5 deixa 0,48 %, 6
    // deixa 0,215 %, nunca zero; a janela dá exatamente zero, por construção,
    // a QUALQUER alcance"*.
    //
    // ⇒ A pergunta certa é a que o artista vê: **qual é o perfil DEPOIS da
    // aterrissagem**, por família, no alcance que já shipa. A janela é `C¹`
    // nas duas pontas, então o composto desce a zero liso em toda família — o
    // que muda entre elas é a LARGURA da influência, que é a feature.
    println!("\n=== o perfil que o barro recebe (campo × aterrissagem, reach = 3) ===");
    println!("   r/ε      Mono        Bi       Tri");
    let landed = |k: f32, sc: Scales| {
        let f = grab([k * EPS, 0.0, 0.0], EPS, pull, sc)[0];
        f * ph2d_sculpt3d::kelvinlet::rim_landing(k / 3.0)
    };
    for k in [0.0f32, 0.5, 1.0, 1.5, 2.0, 2.25, 2.5, 2.75, 3.0] {
        println!(
            "  {k:>4.2}  {:>8.5}  {:>8.5}  {:>8.5}",
            landed(k, Scales::Mono),
            landed(k, Scales::Bi),
            landed(k, Scales::Tri)
        );
    }

    // A meia-largura: onde cada família cai a metade do bico DEPOIS da janela.
    // É o número que diz *quão largo* é o agarre, que é o que o knob vende.
    println!("\n  família   meia-largura (r/ε)   fração do bico na borda");
    for (name, sc) in [
        ("Mono", Scales::Mono),
        ("Bi", Scales::Bi),
        ("Tri", Scales::Tri),
    ] {
        let peak = landed(0.0, sc);
        let mut half = 0.0f32;
        while half < 3.0 && landed(half, sc) > 0.5 * peak {
            half += 0.01;
        }
        println!(
            "  {name:>5}   {half:>17.2}   {:>22.5}",
            landed(2.25, sc) / peak
        );
    }
}
