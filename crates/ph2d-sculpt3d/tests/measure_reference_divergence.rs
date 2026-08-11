//! **O ATLAS DA DIVERGÊNCIA** — quanto o nosso motor difere do porte da
//! referência, verbo a verbo, num único dab.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_reference_divergence \
//!   -- --ignored --nocapture
//! ```
//!
//! # Por que uma sonda e não um gate
//!
//! Ela não afirma nada: ela **mede**. Os números que ela imprime é que dizem
//! quais verbos estão longe e quanto, e é isso que decide a ordem do trabalho —
//! a alternativa é eu escolher a ordem pela minha leitura do código, que é
//! exatamente como se conserta o verbo errado primeiro.
//!
//! # ⚠️ Ela alimenta os DOIS lados com a MESMA malha, de propósito
//!
//! A referência recebe as posições, as normais e a máscara da **nossa** malha,
//! e a **mesma** pegada. Não é conveniência: é o que isola *a lei do kernel* —
//! a pergunta — de *como as normais são computadas* e *quem está sob o pincel*,
//! que são outras duas perguntas com gates próprios. Uma sonda que deixasse as
//! três variarem juntas reportaria um número que não aponta para nada.
//!
//! # ⚠️ E ela converte a POLARIDADE da máscara
//!
//! O nosso `DEFAULT_MASK = 0` é *totalmente esculpível*; o `mAr[ind + 2]` da
//! referência é o oposto. Quem esquece isto mede o pincel de um lado contra a
//! máscara do outro e conclui que a lei diverge.

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Brush, Dab, Falloff, SculptStroke, Symmetry, Verb, ref_kernels as rk};

/// A malha das duas medições — a nossa, com as nossas normais.
fn sphere() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(64, 128, 1.0)
}

/// Onde o pincel pousa: um ponto SOBRE a superfície, no equador, longe dos
/// polos (onde a topologia da esfera UV degenera e o número falaria sobre a
/// malha em vez de sobre o verbo).
fn dab_at(radius: f32) -> Dab {
    let c = [0.7f32.cos(), 0.0, 0.7f32.sin()];
    let len = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
    let eye = [-c[0] / len, -c[1] / len, -c[2] / len];
    Dab::at(c, radius, eye)
}

/// A pegada: os vértices dentro da esfera do dab.
fn footprint(mesh: &Mesh, d: &Dab) -> Vec<u32> {
    let r2 = d.radius * d.radius;
    mesh.positions()
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            let v = [p[0] - d.center[0], p[1] - d.center[1], p[2] - d.center[2]];
            v[0] * v[0] + v[1] * v[1] + v[2] * v[2] < r2
        })
        .map(|(i, _)| i as u32)
        .collect()
}

/// A máscara **na polaridade da referência** (`1` = livre) — a nossa é o
/// contrário, e este é o único lugar em que a conversão acontece.
fn free_of(mesh: &Mesh) -> Vec<f32> {
    match mesh.masks() {
        Some(m) => m.iter().map(|&x| ph2d_sculpt3d::mask_ops::free_weight(x)).collect(),
        None => vec![1.0; mesh.vert_count()],
    }
}

/// As posições achatadas em `xyz`, que é a forma que o porte fala.
fn flat(mesh: &Mesh) -> Vec<f32> {
    mesh.positions().iter().flat_map(|p| *p).collect()
}

/// O maior deslocamento por componente de um campo contra a linha de base.
fn reach_of(a: &[f32], base: &[f32]) -> f64 {
    a.iter()
        .zip(base)
        .map(|(x, b)| (f64::from(*x) - f64::from(*b)).abs())
        .fold(0.0, f64::max)
}

struct Row {
    verb: &'static str,
    ours: f64,
    theirs: f64,
    /// O maior |nosso − deles| por componente.
    gap: f64,
}

#[test]
#[ignore = "sonda: imprime a tabela, não afirma nada"]
fn what_separates_our_kernels_from_the_reference() {
    const R: f32 = 0.45;
    let mut rows: Vec<Row> = Vec::new();

    // Cada linha é `(verbo, a intensidade DE FÁBRICA da tool correspondente no
    // original, o kernel da referência)`. A intensidade não é escolhida por
    // mim: é o `_intensity` do construtor de cada tool.
    let cases: &[(&'static str, Verb, f64, bool)] = &[
        ("Draw/brush", Verb::Draw, 0.5, false),
        ("Clay", Verb::Clay, 0.5, false),
        ("Flatten", Verb::Flatten, 0.75, true),
        ("Inflate", Verb::Inflate, 0.3, false),
        ("Crease", Verb::Crease, 0.75, true),
        ("Pinch", Verb::Pinch, 0.75, false),
    ];

    for &(name, verb, intensity, negative) in cases {
        let mut mesh = sphere();
        let base = flat(&mesh);
        let d = dab_at(R);
        let fp = footprint(&mesh, &d);
        let free = free_of(&mesh);
        let normals: Vec<f32> = mesh.normals().iter().flat_map(|n| *n).collect();

        // --- O NOSSO, pela porta do produto.
        let brush = Brush {
            verb,
            strength: intensity as f32,
            falloff: Falloff::Smooth,
            invert: negative,
            ..Brush::default()
        };
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        stroke.dab(&mut mesh, &brush, &d, Symmetry::default());
        let ours = flat(&mesh);

        // --- O DELES, sobre a MESMA malha e a MESMA pegada.
        let mut theirs = base.clone();
        let center = [
            f64::from(d.center[0]),
            f64::from(d.center[1]),
            f64::from(d.center[2]),
        ];
        let eye = [f64::from(d.eye[0]), f64::from(d.eye[1]), f64::from(d.eye[2])];
        let r2 = f64::from(d.radius) * f64::from(d.radius);
        let mut front = Vec::new();
        rk::front_vertices(&normals, &fp, eye, &mut front);
        let a_normal = rk::area_normal(&normals, &free, &front).expect("normal de área");
        match verb {
            Verb::Draw => rk::brush(
                &mut theirs, &free, &fp, None, a_normal, center, r2, intensity, negative,
            ),
            Verb::Clay => {
                let mut ctr = rk::area_center(&base, &free, &front).expect("centro");
                let off = rk::clay_plane_offset(r2.sqrt());
                for k in 0..3 {
                    ctr[k] += a_normal[k] * off;
                }
                rk::flatten(
                    &mut theirs, &free, &fp, None, a_normal, ctr, center, r2, intensity, negative,
                );
            }
            Verb::Flatten => {
                let ctr = rk::area_center(&base, &free, &front).expect("centro");
                rk::flatten(
                    &mut theirs, &free, &fp, None, a_normal, ctr, center, r2, intensity, negative,
                );
            }
            Verb::Inflate => rk::inflate(
                &mut theirs,
                &normals,
                &free,
                &fp,
                Some(&base),
                center,
                r2,
                intensity,
                negative,
            ),
            Verb::Crease => rk::crease(
                &mut theirs,
                &free,
                &fp,
                Some(&base),
                a_normal,
                center,
                r2,
                intensity,
                negative,
            ),
            Verb::Pinch => rk::pinch(&mut theirs, &free, &fp, center, r2, intensity, negative),
            _ => unreachable!("a tabela só tem verbos de carimbo"),
        }

        rows.push(Row {
            verb: name,
            ours: reach_of(&ours, &base),
            theirs: reach_of(&theirs, &base),
            gap: reach_of(&ours, &theirs),
        });
    }

    println!("\n== UM DAB, a MESMA malha e a MESMA pegada ({} vértices) ==", {
        let m = sphere();
        footprint(&m, &dab_at(R)).len()
    });
    println!(
        "{:<12} {:>12} {:>12} {:>10} {:>12}",
        "verbo", "nosso", "referência", "razão", "|diferença|"
    );
    for r in &rows {
        let ratio = if r.theirs > 0.0 {
            format!("{:.2}x", r.ours / r.theirs)
        } else {
            "-".into()
        };
        println!(
            "{:<12} {:>12.6} {:>12.6} {:>10} {:>12.6}",
            r.verb, r.ours, r.theirs, ratio, r.gap
        );
    }

    // A curva, isolada — o fator que multiplica TODOS os verbos.
    println!("\n== A CURVA, ponto a ponto ==");
    println!(
        "{:>6} {:>12} {:>12} {:>10}",
        "t", "referência", "Smooth", "razão"
    );
    for i in 0..=8 {
        let t = f64::from(i) / 8.0;
        let r = rk::falloff(t);
        let s = f64::from(Falloff::Smooth.weight(t as f32));
        let ratio = if s > 0.0 {
            format!("{:.3}x", r / s)
        } else {
            "-".into()
        };
        println!("{t:>6.3} {r:>12.6} {s:>12.6} {ratio:>10}");
    }

    // O ACÚMULO — o checkbox que o Enio reportou como quebrado. Um traço reto
    // de N dabs sobre o mesmo lugar, com e sem ele.
    println!("\n== O ACUMULA, medido: N dabs no MESMO lugar ==");
    println!(
        "{:>5} {:>16} {:>16} {:>16}",
        "dabs", "nosso OFF", "nosso ON", "referência"
    );
    for n in [1usize, 2, 4, 8, 16] {
        let d = dab_at(R);
        let mk = |accumulate: bool| {
            let mut mesh = sphere();
            let base = flat(&mesh);
            let brush = Brush {
                verb: Verb::Draw,
                strength: 0.5,
                accumulate,
                ..Brush::default()
            };
            let mut s = SculptStroke::default();
            s.begin(&mesh);
            for _ in 0..n {
                s.dab(&mut mesh, &brush, &d, Symmetry::default());
            }
            reach_of(&flat(&mesh), &base)
        };
        // A referência: N aplicações do kernel sobre o estado VIVO, que é o que
        // ela faz — a normal de área é recomputada a cada dab, como no
        // `Brush.stroke`.
        let mut mesh = sphere();
        let base = flat(&mesh);
        let free = free_of(&mesh);
        let mut theirs = base.clone();
        let center = [
            f64::from(d.center[0]),
            f64::from(d.center[1]),
            f64::from(d.center[2]),
        ];
        let eye = [f64::from(d.eye[0]), f64::from(d.eye[1]), f64::from(d.eye[2])];
        let r2 = f64::from(d.radius) * f64::from(d.radius);
        let fp = footprint(&mesh, &d);
        for _ in 0..n {
            let normals: Vec<f32> = mesh.normals().iter().flat_map(|n| *n).collect();
            let mut front = Vec::new();
            rk::front_vertices(&normals, &fp, eye, &mut front);
            let an = rk::area_normal(&normals, &free, &front).expect("normal");
            rk::brush(&mut theirs, &free, &fp, None, an, center, r2, 0.5, false);
            for (i, p) in mesh.positions_mut().iter_mut().enumerate() {
                *p = [theirs[i * 3], theirs[i * 3 + 1], theirs[i * 3 + 2]];
            }
            mesh.rebuild();
        }
        println!(
            "{n:>5} {:>16.6} {:>16.6} {:>16.6}",
            mk(false),
            mk(true),
            reach_of(&theirs, &base)
        );
    }
    println!();
}
