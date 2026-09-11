//! **O QUE OS TRÊS MODOS DO GRAB FAZEM COM O BARRO** — a sonda que respondeu ao
//! report *"os modos B e L do grab estão bizarros"* antes de qualquer hipótese.
//!
//! `cargo test -p ph2d-sculpt3d --release --test measure_grab_modes -- --ignored
//! --nocapture --test-threads=1`
//!
//! ⚠️ **Ela dirige a porta do PRODUTO** (`begin` + N × `dab`), com o número de
//! eventos que um arrasto real entrega — não um dab só. O [`Grip::Hold`]
//! recomputa o peso a cada evento, então um perfil medido num dab não é o que o
//! artista vê, e foi exatamente aí que os dois defeitos se escondiam.

use ph2d_sculpt3d::{Brush, Dab, RefMode, SculptStroke, Symmetry, Verb};

const R: f32 = 0.5;
const TIP: [f32; 3] = [0.0, 0.0, 1.0];
const EYE: [f32; 3] = [0.0, 0.0, -1.0];

fn sphere() -> ph2d_mesh::Mesh {
    ph2d_mesh::shapes::uv_sphere(32, 48, 1.0)
}

fn grab(mode: RefMode) -> Brush {
    Brush {
        verb: Verb::Move,
        mode,
        radius: R,
        strength: 1.0,
        ..Brush::default()
    }
}

/// Um arrasto REAL: `steps` eventos, o puxão total crescendo até `pull`.
fn drag(mode: RefMode, pull: [f32; 3], steps: usize) -> ph2d_mesh::Mesh {
    let mut mesh = sphere();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    for k in 1..=steps {
        let t = k as f32 / steps as f32;
        let p = [pull[0] * t, pull[1] * t, pull[2] * t];
        stroke.dab(
            &mut mesh,
            &grab(mode),
            &Dab::pulling(TIP, R, EYE, p),
            Symmetry::default(),
        );
    }
    mesh
}

fn nearest(m: &ph2d_mesh::Mesh, p: [f32; 3]) -> usize {
    (0..m.vert_count())
        .min_by(|&a, &b| {
            let d = |k: usize| {
                let q = m.positions()[k];
                (q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2)
            };
            d(a).total_cmp(&d(b))
        })
        .expect("a esfera tem vértices")
}

fn disp(a: &ph2d_mesh::Mesh, b: &ph2d_mesh::Mesh, i: usize) -> f32 {
    let (u, v) = (a.positions()[i], b.positions()[i]);
    ((u[0] - v[0]).powi(2) + (u[1] - v[1]).powi(2) + (u[2] - v[2]).powi(2)).sqrt()
}

/// Um ponto na esfera a `arc` de distância angular do topo, na direção `dir`.
fn on_sphere(dir: [f32; 3], arc: f32) -> [f32; 3] {
    let (s, c) = arc.sin_cos();
    [dir[0] * s, dir[1] * s, c]
}

/// **O PERFIL que o artista vê**, nos três modos, depois de um arrasto real.
#[test]
#[ignore = "sonda"]
fn what_the_three_modes_do_to_the_clay() {
    let pull = [0.35_f32, 0.0, 0.0];
    let rest = sphere();
    println!("\n== deslocamento apos um arrasto de |pull| = 0,35, raio 0,5, 20 dabs ==");
    println!("(fracao do puxao; o bico deveria seguir o dedo = 1,000)");
    println!("{:>10} | {:>8} | {:>8} | {:>8}", "r/raio", "S", "B", "L");
    let modes = [RefMode::S, RefMode::B, RefMode::L];
    let meshes: Vec<_> = modes.iter().map(|&m| drag(m, pull, 20)).collect();
    let mag = (pull[0] * pull[0] + pull[1] * pull[1] + pull[2] * pull[2]).sqrt();

    for (label, dir) in [
        ("a FRENTE (+x)", [1.0_f32, 0.0, 0.0]),
        ("ao LADO (+y)", [0.0, 1.0, 0.0]),
        ("ATRAS (-x)", [-1.0, 0.0, 0.0]),
    ] {
        println!("-- {label} --");
        for k in [0.0_f32, 0.25, 0.5, 0.75, 1.0, 1.25] {
            let p = on_sphere(dir, k * R);
            let i = nearest(&rest, p);
            let row: Vec<String> = meshes
                .iter()
                .map(|m| format!("{:>8.4}", disp(&rest, m, i) / mag))
                .collect();
            println!("{k:>10.2} | {}", row.join(" | "));
        }
    }
}

/// **O MAIOR SALTO ENTRE VIZINHOS** — o que a malha consegue desenhar como
/// degrau, contra o comprimento de uma aresta.
#[test]
#[ignore = "sonda"]
fn the_rim_step_in_each_mode() {
    let pull = [0.35_f32, 0.0, 0.0];
    let rest = sphere();
    println!("\n== maior salto de deslocamento entre vizinhos (unidades de mundo) ==");
    let mut tris = Vec::new();
    rest.triangle_indices(&mut tris);
    let mut edge = 0.0_f32;
    for t in &tris {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            let (p, q) = (rest.positions()[a as usize], rest.positions()[b as usize]);
            let d = ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
            edge = edge.max(d);
        }
    }
    for mode in [RefMode::S, RefMode::B, RefMode::L] {
        let m = drag(mode, pull, 20);
        let mut step = 0.0_f32;
        for t in &tris {
            for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                let (i, j) = (a as usize, b as usize);
                step = step.max((disp(&rest, &m, i) - disp(&rest, &m, j)).abs());
            }
        }
        println!("{mode:?}: salto {step:.4}   (aresta maxima {edge:.4})");
    }
}

/// **A TRAJETÓRIA do bico ao longo do arrasto** — é ela que diz se um modo
/// realimenta. O bico tem de seguir o dedo, evento a evento.
#[test]
#[ignore = "sonda"]
fn does_the_tip_follow_the_finger_all_the_way() {
    println!("\n== deslocamento do BICO por evento (fracao do puxao daquele instante) ==");
    println!("{:>6} | {:>8} | {:>8} | {:>8}", "evento", "S", "B", "L");
    let total = [0.6_f32, 0.0, 0.0];
    let steps = 12;
    let rest = sphere();
    let tip = nearest(&rest, TIP);
    let mut rows: Vec<Vec<f32>> = vec![Vec::new(); 3];
    for (c, mode) in [RefMode::S, RefMode::B, RefMode::L].into_iter().enumerate() {
        let mut mesh = sphere();
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        for k in 1..=steps {
            let t = k as f32 / steps as f32;
            let p = [total[0] * t, total[1] * t, total[2] * t];
            stroke.dab(
                &mut mesh,
                &grab(mode),
                &Dab::pulling(TIP, R, EYE, p),
                Symmetry::default(),
            );
            let want = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            rows[c].push(disp(&rest, &mesh, tip) / want);
        }
    }
    for (k, s) in rows[0].iter().enumerate() {
        println!(
            "{:>6} | {:>8.4} | {:>8.4} | {:>8.4}",
            k + 1,
            s,
            rows[1][k],
            rows[2][k]
        );
    }
}

/// **QUANTOS VÉRTICES cada modo move**, e quanto de barro no total.
#[test]
#[ignore = "sonda"]
fn how_much_clay_each_mode_takes() {
    let pull = [0.35_f32, 0.0, 0.0];
    let rest = sphere();
    println!("\n== quantos vertices se movem, e o total percorrido ==");
    for mode in [RefMode::S, RefMode::B, RefMode::L] {
        let m = drag(mode, pull, 20);
        let mut n = 0;
        let mut sum = 0.0_f32;
        let mut peak = 0.0_f32;
        for i in 0..rest.vert_count() {
            let d = disp(&rest, &m, i);
            if d > 1e-5 {
                n += 1;
            }
            sum += d;
            peak = peak.max(d);
        }
        println!("{mode:?}: {n:>5} vertices  soma {sum:>8.3}  pico {peak:.4}");
    }
}

/// **O SINAL do campo elástico ao longo do raio** — a pergunta que a magnitude
/// esconde: um campo que troca de sinal empurra o barro para TRÁS num anel.
#[test]
#[ignore = "sonda"]
fn does_the_elastic_field_change_sign() {
    use ph2d_sculpt3d::kelvinlet::{Scales, grab as kgrab};
    println!("\n== componente do campo NA DIRECAO do puxao (bico = 1,000) ==");
    println!(
        "{:>6} | {:>19} | {:>19}",
        "r/eps", "AO LADO  Mono/Bi/Tri", "A FRENTE Mono/Bi/Tri"
    );
    let f = [1.0_f32, 0.0, 0.0];
    for k in [0.0_f32, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 4.0, 6.0] {
        let at = |p: [f32; 3], s| kgrab(p, 1.0, f, s)[0];
        let side = [
            at([0.0, k, 0.0], Scales::Mono),
            at([0.0, k, 0.0], Scales::Bi),
            at([0.0, k, 0.0], Scales::Tri),
        ];
        let ahead = [
            at([k, 0.0, 0.0], Scales::Mono),
            at([k, 0.0, 0.0], Scales::Bi),
            at([k, 0.0, 0.0], Scales::Tri),
        ];
        println!(
            "{k:>6.1} | {:>6.3}{:>7.3}{:>7.3} | {:>6.3}{:>7.3}{:>7.3}",
            side[0], side[1], side[2], ahead[0], ahead[1], ahead[2]
        );
    }
    println!("\n(a pegada do produto corta em r/eps = 3,0; o anel do cursor esta' em 1,0)");
}

/// **O QUE CUSTA CRESCER A CONSULTA** — a alternativa que o doc do
/// `KELVINLET_REACH` recusou por ESTIMATIVA (*"~9× os vértices"*), medida.
#[test]
#[ignore = "sonda"]
fn what_a_three_times_wider_query_costs() {
    const K1_MS: f64 = 8.0;
    let mut mesh = ph2d_mesh::shapes::uv_sphere(32, 48, 1.0);
    for _ in 0..3 {
        mesh = ph2d_mesh::subdivide(&mesh);
    }
    println!(
        "\n== custo de um dab de Grab, malha de {} vertices ==",
        mesh.vert_count()
    );
    println!(
        "{:>8} | {:>10} | {:>10} | {:>10}",
        "raio", "vertices", "ms/dab", "% do K1"
    );
    for radius in [0.1_f32, 0.3, 0.5, 0.9, 1.5] {
        let mut m = mesh.clone();
        let mut stroke = SculptStroke::default();
        stroke.begin(&m);
        let b = Brush {
            verb: Verb::Move,
            mode: RefMode::S,
            radius,
            strength: 1.0,
            ..Brush::default()
        };
        stroke.dab(
            &mut m,
            &b,
            &Dab::pulling(TIP, radius, EYE, [0.01, 0.0, 0.0]),
            Symmetry::default(),
        );
        let n = {
            let mut q = ph2d_mesh::QueryScratch::default();
            let mut out = Vec::new();
            m.verts_in_sphere(TIP, radius, &mut q, &mut out);
            out.len()
        };
        let reps = 40;
        let t0 = std::time::Instant::now();
        for k in 1..=reps {
            let p = 0.2 * k as f32 / reps as f32;
            stroke.dab(
                &mut m,
                &b,
                &Dab::pulling(TIP, radius, EYE, [p, 0.0, 0.0]),
                Symmetry::default(),
            );
        }
        let ms = t0.elapsed().as_secs_f64() * 1e3 / f64::from(reps);
        println!(
            "{radius:>8.1} | {n:>10} | {ms:>10.3} | {:>9.1}%",
            ms / K1_MS * 100.0
        );
    }
    println!("\n(o raio 1,5 numa esfera de raio 1 e' a malha INTEIRA na pegada)");
}

/// **O MESMO PUXÃO, ENTREGUE EM UM E EM DOZE EVENTOS** — a lei do traço, medida
/// no modo cuja lei de kernel pesa pela normal.
#[test]
#[ignore = "sonda"]
fn the_same_pull_delivered_in_one_and_in_twelve_events() {
    let rest = sphere();
    println!("\n== divergencia entre 1 dab e 12 dabs, mesmo puxao total ==");
    for pull in [[0.2_f32, 0.0, 0.0], [0.6, 0.0, 0.0], [0.9, 0.0, 0.0]] {
        for mode in [RefMode::S, RefMode::B, RefMode::L] {
            let a = drag(mode, pull, 1);
            let b = drag(mode, pull, 12);
            let mut worst = 0.0_f32;
            for i in 0..rest.vert_count() {
                let (p, q) = (a.positions()[i], b.positions()[i]);
                let d =
                    ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
                worst = worst.max(d);
            }
            println!(
                "  |pull| {:.1}  {mode:?}: pior divergencia {worst:.6}",
                pull[0]
            );
        }
    }
}
