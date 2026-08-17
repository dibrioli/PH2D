//! **O PENTE DA DEMÃO É KERNEL OU É A GRADE?** — a §3.1 do handoff, falsificável
//! numa medição.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_layer_comb \
//!   -- --ignored --nocapture --test-threads=1
//! ```
//!
//! A foto do Enio na faixa de dureza alta mostra **listras retangulares
//! regulares**. O handoff propõe que sejam a *parede* da mesa a escadear pela
//! grade de quads da esfera de fábrica, e não o kernel — e nomeia o teste:
//! *o período do pente tem de ser o passo da grade*.
//!
//! ⚠️ **A medição direta é mais barata que uma autocorrelação, e ela separa as
//! duas hipóteses sem escolher um número:** com `hardness = h`, o
//! `apply_hardness_to_distances` manda a `t < h` para **zero**, então a curva
//! satura e o `shape` é **constante** em todo o disco interior. A lei do
//! `layer.cc` leva todo vértice de mesmo `shape` à **mesma altura absoluta** ⇒
//!
//! * o platô ONDULA ⇒ o pente é do **kernel**;
//! * o platô é chato e só a **parede** escadeia ⇒ é **discretização**, e
//!   persegui-la é caçar o alvo errado (o Blender escadeia igual, na topologia
//!   dele).
//!
//! ⚠️ **A régua do platô é a ARESTA da malha, nunca um épsilon escolhido:** um
//! desvio menor que o espaçamento de vértices não tem como ser visto na tela, e
//! um número absoluto envelheceria na primeira mudança de subdivisão.

use ph2d_mesh::{Mesh, shapes};
use ph2d_sculpt3d::{Brush, Dab, RefMode, SculptStroke, Symmetry, Verb};

const CENTRE: [f32; 3] = [0.0, 0.0, 1.0];
const EYE: [f32; 3] = [0.0, 0.0, -1.0];
const R: f32 = 0.45;

fn sphere() -> Mesh {
    shapes::sculpt_sphere(1.0)
}

fn len(q: [f32; 3]) -> f32 {
    (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt()
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    len(d)
}

/// O espaçamento MEDIANO de aresta na região do dab — a régua contra a qual
/// toda ondulação é julgada.
fn edge_near(mesh: &Mesh) -> f32 {
    let (pos, adj) = (mesh.positions(), mesh.adjacency());
    let mut e: Vec<f32> = Vec::new();
    for (i, p) in pos.iter().enumerate() {
        if dist3(*p, CENTRE) > R {
            continue;
        }
        for &j in adj.vert_verts.neighbours(i) {
            e.push(dist3(*p, pos[j as usize]));
        }
    }
    e.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if e.is_empty() { 0.0 } else { e[e.len() / 2] }
}

/// Esfrega `dabs` vezes no MESMO ponto — a demão satura, que é o regime da foto.
fn coat(hardness: f32, dabs: usize) -> (Mesh, Mesh) {
    let rest = sphere();
    let mut mesh = sphere();
    let b = Brush {
        verb: Verb::Layer,
        mode: RefMode::B,
        radius: R,
        strength: Verb::Layer.default_strength(),
        falloff: Verb::Layer.default_falloff(RefMode::B),
        hardness,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for _ in 0..dabs {
        s.dab(&mut mesh, &b, &Dab::at(CENTRE, R, EYE), Symmetry::default());
    }
    (rest, mesh)
}

#[test]
#[ignore]
fn measure_is_the_comb_the_kernel_or_the_grid() {
    println!("\n  == O PLATO ONDULA? (esfera de fabrica, um ponto esfregado) ==");
    println!("  A lei manda todo vertice do plato a' MESMA altura absoluta.");
    println!("  Regua: o espacamento MEDIANO de aresta na regiao do dab.\n");

    let e = edge_near(&sphere());
    println!("  aresta mediana na regiao: {e:.5}\n");

    println!(
        "  {:>5}  {:>6}  {:>9}  {:>9}  {:>9}  {:>9}",
        "h", "plato", "alt.med", "ondulac.", "ond/arest", "parede"
    );
    println!(
        "  {:->5}  {:->6}  {:->9}  {:->9}  {:->9}  {:->9}",
        "", "", "", "", "", ""
    );

    for h in [0.0f32, 0.25, 0.5, 0.75, 0.9] {
        let (rest, mesh) = coat(h, 12);
        // O disco interior: onde o `apply_hardness_to_distances` satura.
        // Com `h = 0` não existe platô — a fatia mede o miolo mesmo assim, e
        // é o CONTROLE (ali a ondulação É o falloff, não um defeito).
        let inner = (h * 0.95).max(0.0) * R;
        let mut hs: Vec<f32> = Vec::new();
        let mut wall = 0.0f32;
        for (i, p) in mesh.positions().iter().enumerate() {
            let p0 = rest.positions()[i];
            let d = dist3(p0, CENTRE);
            let lift = len(*p) - len(p0);
            if d <= inner {
                hs.push(lift);
            }
            if d > inner && d <= R {
                wall = wall.max(lift);
            }
        }
        if hs.is_empty() {
            println!("  {h:>5.2}  {:>6}  (sem plato: hardness zero)", 0);
            continue;
        }
        let mean = hs.iter().sum::<f32>() / hs.len() as f32;
        let (lo, hi) = hs
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
        let ripple = hi - lo;
        println!(
            "  {h:>5.2}  {:>6}  {mean:>9.5}  {ripple:>9.5}  {:>9.4}  {wall:>9.5}",
            hs.len(),
            ripple / e.max(1e-9)
        );
    }

    println!(
        "\n  LEITURA: `ond/arest` << 1 => o plato e' CHATO e o pente e' a PAREDE\n  \
         (discretizacao, o alvo errado). >> 1 => e' o kernel."
    );
}
