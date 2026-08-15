//! **O Draw Sharp já existe?** — a sonda que decide se a W6 tem um verbo novo
//! ali ou um chip morto.
//!
//! O plano diz *"Draw Sharp — o Draw sobre as posições/normais do pen-down.
//! Vinco duro em vez de domo. ⚠️ Custo quase nulo: o `pre` congelado já
//! existe"*. Mas o nosso `Grip::Stamp` já lê `from_live = accumulate`, ou seja
//! **o Draw com Accumulate DESLIGADO já mede a distância no `pre`** — que é
//! metade da definição da referência (`draw_sharp.cc` usa `orig_data` onde o
//! `draw.cc` usa `position_data.eval`).
//!
//! ⇒ A pergunta que esta sonda responde é a do gate 2 do §8: *o chip novo
//! produziria um resultado diferente do vizinho, acima do piso de paridade?*

use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

fn plane_grid(n: usize, half: f32) -> ph2d_mesh::Mesh {
    let mut pos = Vec::with_capacity((n + 1) * (n + 1));
    for j in 0..=n {
        for i in 0..=n {
            let f = |k: usize| (k as f32 / n as f32) * 2.0 * half - half;
            pos.push([f(i), f(j), 0.0]);
        }
    }
    let at = |i: usize, j: usize| (j * (n + 1) + i) as u32;
    let mut faces = Vec::with_capacity(n * n * 2);
    for j in 0..n {
        for i in 0..n {
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j),
                at(i + 1, j + 1),
            ));
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j + 1),
                at(i, j + 1),
            ));
        }
    }
    ph2d_mesh::Mesh::from_parts(pos, faces).expect("indices validos")
}

/// O perfil transversal da crista: a altura da COLUNA central da grade, por
/// ÍNDICE.
///
/// ⚠️ **Por índice e não por `x ≈ 0`, e a 1ª versão mediu errado por isso:** o
/// Draw empurra pela normal da ÁREA, que se INCLINA à medida que o traço
/// levanta a superfície, então os vértices andam em `x` também — o filtro por
/// coordenada perdia 19 dos 81 e reportava pico **0,000** onde o máximo global
/// era **0,184**.
fn ridge(mesh: &ph2d_mesh::Mesh, n: usize) -> Vec<(f32, f32)> {
    (0..=n)
        .map(|j| {
            let p = mesh.positions()[j * (n + 1) + n / 2];
            (p[1], p[2])
        })
        .collect()
}

fn stroke_along_x(accumulate: bool, dabs: usize) -> ph2d_mesh::Mesh {
    let mut mesh = plane_grid(80, 2.0);
    let brush = Brush {
        verb: Verb::Draw,
        radius: 0.5,
        strength: 0.5,
        accumulate,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    for k in 0..dabs {
        // O espaçamento do produto: `0,15 · r`.
        // O traço é CENTRADO em `x = 0`, que é onde o perfil é medido — senão
        // a secção cai fora da pegada e o pico mede zero (a fixture não conteria
        // o fenômeno).
        let x = (k as f32 - (dabs - 1) as f32 * 0.5) * 0.15 * 0.5;
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::at([x, 0.0, 0.0], 0.5, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }
    mesh
}

#[test]
#[ignore = "sonda de medição: roda sozinha"]
fn measure_whether_the_frozen_draw_is_already_a_sharp_crease() {
    for dabs in [1usize, 9] {
        for accumulate in [true, false] {
            let mesh = stroke_along_x(accumulate, dabs);
            let prof = ridge(&mesh, 80);
            let peak = prof.iter().map(|&(_, z)| z).fold(0.0f32, f32::max);
            // A meia-largura: onde o perfil cruza metade do pico.
            let half_w = prof
                .iter()
                .filter(|&&(_, z)| z >= peak * 0.5)
                .map(|&(y, _)| y.abs())
                .fold(0.0f32, f32::max);
            println!(
                "dabs {dabs}  accumulate {accumulate:<5}  pico {peak:.6}  \
                 meia-largura {half_w:.4}"
            );
        }
    }
}
