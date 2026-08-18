//! **O QUE O SHARPEN FAZ, E ONDE ELE PARA DE FAZER** — a sonda que decide o
//! `SHARPEN_MAX` antes de o número ser escrito.
//!
//! Três perguntas, e nenhuma delas se responde por raciocínio:
//!
//! 1. **ele AFIA?** — a régua é o *degrau* da superfície: quanto a malha salta
//!    de um vértice para o vizinho. Afiar é aumentar essa transição, e é por
//!    isso que a régua não é a excursão (que qualquer deslocamento move) nem o
//!    pico de curvatura (que a lei ALISA de propósito, é metade do mecanismo);
//! 2. **onde ele explode?** — a força cresce até a malha deixar de ser a malha;
//! 3. **o fatiamento é honesto?** — a mesma força total, entregue em número de
//!    fatias diferente, tem de convergir em vez de saltar.
//!
//! Ela **imprime e não afirma**. Rodar:
//! `cargo test -p ph2d-sculpt3d --test measure_sharpen_law --release -- --ignored --nocapture`

use ph2d_mesh::{Mesh, shapes::uv_sphere_noisy};
use ph2d_sculpt3d::{Brush, FilterKind, SculptStroke, Verb};

/// **A FIXTURE — uma esfera com uma CRISTA, e a primeira que escrevi não
/// continha o fenómeno.**
///
/// ⚠️ **Ruído branco NÃO é detalhe a afiar, e a medição foi categórica.** Com
/// `uv_sphere_noisy(.., 0.04)` a curvatura é comparável em TODO vértice, então
/// o pré-passe normaliza e `f` fica alto em toda parte; com `f → 1` o gather é
/// multiplicado por `(1 − f) → 0` e **só o termo médio sobrevive** ⇒ a lei
/// degenera num alisador (medido: o degrau caía a `0,667×` e a curvatura de
/// pico junto). *A lei precisa de contraste entre uma feição e o fundo, e
/// ruído é o campo em que esse contraste não existe.*
///
/// Aqui é uma crista gaussiana em torno do equador sobre uma esfera LISA: `f`
/// alto na crista, baixo no resto — que é a configuração em que os dois termos
/// puxam em direções opostas e a lei faz o que o nome dela diz.
fn wrinkled() -> Mesh {
    let mut m = uv_sphere_noisy(48, 64, 1.0, 0.0);
    for p in m.positions_mut() {
        let r = norm(*p);
        if r <= f32::EPSILON {
            continue;
        }
        let t = p[1] / r;
        let bump = (-(t * t) / (2.0 * 0.15 * 0.15)).exp() * 0.12;
        let s = (r + bump) / r;
        *p = [p[0] * s, p[1] * s, p[2] * s];
    }
    m
}

/// **A LARGURA DA CRISTA** — a latitude (em `sin`) onde o perfil radial cai a
/// meio caminho entre o topo e o fundo.
///
/// ⚠️ **É a régua que separa *afiar* de *alisar*, e o degrau sozinho não a
/// substitui:** afiar uma crista é ESTREITÁ-LA (a transição fica curta), e uma
/// crista que só cresce em altura teria degrau maior sem ser mais afiada.
fn ridge_half_width(mesh: &Mesh) -> f32 {
    let pos = mesh.positions();
    let (mut top, mut floor) = (0.0f32, f32::MAX);
    for p in pos {
        let r = norm(*p);
        if r <= f32::EPSILON {
            continue;
        }
        let t = (p[1] / r).abs();
        if t < 0.02 {
            top = top.max(r);
        }
        if t > 0.7 {
            floor = floor.min(r);
        }
    }
    let half = (top + floor) * 0.5;
    // A menor |latitude| cujo raio já caiu abaixo do meio caminho.
    let mut w = 1.0f32;
    for p in pos {
        let r = norm(*p);
        if r <= f32::EPSILON {
            continue;
        }
        let t = (p[1] / r).abs();
        if r < half {
            w = w.min(t);
        }
    }
    w
}

fn brush() -> Brush {
    Brush {
        verb: Verb::Smooth,
        ..Brush::default()
    }
}

/// Roda UM filtro sobre a fixture e devolve a malha.
fn filtered(kind: FilterKind, amount: f32) -> Mesh {
    let mut m = wrinkled();
    let mut s = SculptStroke::default();
    s.filter_begin(&m);
    s.filter(&mut m, &brush(), kind, amount);
    m
}

fn norm(d: [f32; 3]) -> f32 {
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// **O DEGRAU** — o maior salto de posição entre dois vértices vizinhos,
/// medido ao longo da normal local aproximada pela direção radial.
///
/// ⚠️ **É a régua de *afiado*, e as duas óbvias não servem:** a excursão sobe
/// com qualquer deslocamento (um Inflate a moveria) e o pico de curvatura DESCE
/// de propósito (a lei achata os picos — é metade do mecanismo dela). O que uma
/// aresta é, é uma transição curta e alta, e é isso que esta mede.
fn max_step(mesh: &Mesh) -> f32 {
    let pos = mesh.positions();
    let adj = mesh.adjacency();
    let mut worst = 0.0f32;
    for v in 0..pos.len() {
        let p = pos[v];
        let r = norm(p);
        for &nb in adj.vert_verts.neighbours(v) {
            let q = pos[nb as usize];
            worst = worst.max((norm(q) - r).abs());
        }
    }
    worst
}

/// O maior `|média_do_anel − p|` — a curvatura de pico.
fn max_curvature(mesh: &Mesh) -> f32 {
    let pos = mesh.positions();
    let adj = mesh.adjacency();
    let mut worst = 0.0f32;
    for v in 0..pos.len() {
        let p = pos[v];
        let mut sum = [0.0f64; 3];
        let mut n = 0u32;
        for &nb in adj.vert_verts.neighbours(v) {
            let q = pos[nb as usize];
            for k in 0..3 {
                sum[k] += f64::from(q[k]);
            }
            n += 1;
        }
        if n == 0 {
            continue;
        }
        let inv = 1.0 / f64::from(n);
        let d = [
            (sum[0] * inv) as f32 - p[0],
            (sum[1] * inv) as f32 - p[1],
            (sum[2] * inv) as f32 - p[2],
        ];
        worst = worst.max(norm(d));
    }
    worst
}

/// O maior deslocamento contra a pose de entrada.
fn excursion(before: &[[f32; 3]], mesh: &Mesh) -> f32 {
    before
        .iter()
        .zip(mesh.positions())
        .map(|(p, q)| norm([q[0] - p[0], q[1] - p[1], q[2] - p[2]]))
        .fold(0.0, f32::max)
}

#[test]
#[ignore = "sonda: imprime, não afirma"]
fn measure_what_the_sharpen_does_and_where_it_breaks() {
    let base = wrinkled();
    let pre = base.positions().to_vec();

    println!("\n=== O QUE ELE FAZ — a fixture em repouso ===");
    println!(
        "  {} vértices | degrau {:.6} | largura da crista {:.4} | curvatura de pico {:.6}",
        pre.len(),
        max_step(&base),
        ridge_half_width(&base),
        max_curvature(&base)
    );

    println!("\n=== A VARREDURA — a força total cresce até quebrar ===");
    println!("\n  força | fatias | excursão | degrau    | Δdegrau | largura | curv. pico | finito");
    let step0 = max_step(&base);
    for f in [
        0.1f32, 0.25, 0.5, 0.75, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.75, 2.0, 3.0, 4.0,
    ] {
        let m = filtered(FilterKind::Sharpen, f);
        let finite = m
            .positions()
            .iter()
            .all(|p| p.iter().all(|c| c.is_finite()));
        let st = max_step(&m);
        println!(
            "  {f:>5.2} | {:>6} | {:>8.6} | {st:>9.6} | {:>7.3}× | {:>7.4} | {:>10.6} | {finite}",
            (f / 0.5).ceil().max(1.0) as u32,
            excursion(&pre, &m),
            st / step0,
            ridge_half_width(&m),
            max_curvature(&m)
        );
    }

    println!("\n=== O CONTROLE — força zero não pode mover um vértice ===");
    let m0 = filtered(FilterKind::Sharpen, 0.0);
    println!("  excursão {:.6e}", excursion(&pre, &m0));

    println!("\n=== A MALHA LISA — sem detalhe não há contraste a fazer ===");
    let mut smooth_mesh = uv_sphere_noisy(24, 36, 1.0, 0.0);
    let smooth_pre = smooth_mesh.positions().to_vec();
    let mut s = SculptStroke::default();
    s.filter_begin(&smooth_mesh);
    s.filter(&mut smooth_mesh, &brush(), FilterKind::Sharpen, 1.0);
    println!(
        "  excursão {:.6e}  (esperado ~0: o pré-passe normaliza pelo maior, e num campo uniforme todo f colapsa)",
        excursion(&smooth_pre, &smooth_mesh)
    );
}

#[test]
#[ignore = "sonda: imprime, não afirma"]
fn measure_whether_the_slicing_converges() {
    println!("\n=== O FATIAMENTO — a MESMA força total, em número de fatias crescente ===");
    println!(
        "  (o produto fatia por `MAX_STEP = 0,5`; aqui a fatia é forçada para\n   \
         medir se a lei converge em vez de saltar)"
    );

    let base = wrinkled();
    let pre = base.positions().to_vec();

    for total in [0.5f32, 1.0, 2.0] {
        println!("\n  força total {total:.2}");
        println!("    fatias | degrau    | excursão | vs a fatia anterior");
        let mut prev: Option<Mesh> = None;
        for n in [1u32, 2, 4, 8, 16] {
            let per = total / n as f32;
            // ⚠️ Uma fatia acima do teto de estabilidade não é comparável — a
            // referência recusa-a, e o produto nunca a produz.
            if per > 0.5 {
                println!("    {n:>6} | (fatia {per:.3} > 0,5 — acima do teto de estabilidade)");
                continue;
            }
            let mut m = wrinkled();
            let mut s = SculptStroke::default();
            s.filter_begin(&m);
            // O produto escolhe `n` sozinho; aqui repetimos a chamada para
            // forçar o número de fatias, o que é EXACTAMENTE o que a referência
            // faz com um evento de rato por iteração.
            for _ in 0..n {
                s.filter(&mut m, &brush(), FilterKind::Sharpen, per);
                s.filter_begin(&m);
            }
            let d = prev.as_ref().map_or(f32::NAN, |p| {
                p.positions()
                    .iter()
                    .zip(m.positions())
                    .map(|(a, b)| norm([b[0] - a[0], b[1] - a[1], b[2] - a[2]]))
                    .fold(0.0, f32::max)
            });
            println!(
                "    {n:>6} | {:>9.6} | {:>8.6} | {d:>10.6}",
                max_step(&m),
                excursion(&pre, &m)
            );
            prev = Some(m);
        }
    }
}
