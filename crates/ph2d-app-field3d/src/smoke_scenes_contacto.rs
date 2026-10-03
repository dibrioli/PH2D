//! ⭐⭐⭐ **A CENA 40 — O CONTACTO** (03/10): peças que quase se tocam e peças fundidas, pousadas no
//! chão. A bola AMARELA a `1 cm` por cima da caixa AZUL e a VERMELHA a `1 cm` da face dela escurecem
//! a caixa onde a olham (e são escurecidas por ela); o toro VERDE deitado escurece por dentro do
//! anel; o L LARANJA (duas caixas fundidas) escurece no vinco; o chão escurece à volta de todos.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_ecs::FieldMaterial;
use ph2d_field_eval::leaf;

// ⚠️ A fila vai ao longo de Z: na câmara de fábrica é Z que atravessa o ecrã (ao longo de X as peças
// escondiam-se umas atrás das outras — a 1.ª foto).
fn caixa(h: [f32; 3], y: f32, z: f32) -> Node {
    leaf(
        Primitive::Box {
            half: h,
            round: 0.01,
            chamfer: 0.0,
        },
        Xform::at(0.0, y, z),
    )
}

/// O documento da cena 40.
///
/// # Errors
/// Só se uma das formas violar uma cerca do documento.
pub fn cena_40() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!("[field-smoke] cena 40 — O CONTACTO: pecas que quase se tocam, e pecas fundidas.");
    println!(
        "[field-smoke]            (1) MODEL · painel do topo, Shading · Render. A bola AMARELA esta' \
         1 cm acima da caixa AZUL e a VERMELHA 1 cm ao lado dela: onde se olham, as duas escurecem."
    );
    println!(
        "[field-smoke]            (2) o toro VERDE escurece por dentro do anel; o L LARANJA escurece \
         no canto de dentro; o chao escurece a volta de cada peca, mais onde ela encosta."
    );
    println!(
        "[field-smoke]            (3) clique na bola VERMELHA e arraste a seta do gizmo para longe: a \
         mancha escura na caixa vai-se com ela, na hora. Traga-a de volta: a mancha volta."
    );
    let quarto = core::f32::consts::FRAC_1_SQRT_2;
    let nodes = vec![
        // 0 — a caixa AZUL
        caixa([0.2, 0.2, 0.2], 0.2, -0.6),
        // 1 — a bola AMARELA, 1 cm acima dela
        leaf(
            Primitive::Sphere { radius: 0.18 },
            Xform::at(0.0, 0.59, -0.6),
        ),
        // 2 — a bola VERMELHA, 1 cm ao lado da face +z
        leaf(
            Primitive::Sphere { radius: 0.22 },
            Xform::at(0.0, 0.22, -0.17),
        ),
        // 3 — o toro VERDE, deitado (o eixo dele é Z; um quarto de volta em X põe-no em Y)
        leaf(
            Primitive::Torus {
                major: 0.22,
                minor: 0.07,
            },
            Xform {
                translation: [0.0, 0.07, 0.38],
                rotation: [quarto, 0.0, 0.0, quarto],
                scale: 1.0,
            },
        ),
        // 4, 5 — o L LARANJA: uma laje e uma coluna que entra 2 cm nela (fundidas)
        caixa([0.15, 0.07, 0.15], 0.07, 0.85),
        caixa([0.15, 0.18, 0.05], 0.30, 0.75),
        Node::new(
            Xform::IDENTITY,
            NodeKind::Combine {
                op: Op::Union(ph2d_field::Blend::Sharp),
                children: [0, 1, 2, 3, 4, 5].map(NodeId).to_vec(),
            },
        ),
    ];
    FieldDoc::new(nodes, NodeId(6))
}

/// As cores, na ordem das folhas: azul, amarela, vermelha, verde, laranja, laranja — tons MÉDIOS.
#[must_use]
pub fn materiais_40() -> Vec<FieldMaterial> {
    let cor = |c: [f32; 3]| FieldMaterial {
        base_color: c,
        ..FieldMaterial::default()
    };
    vec![
        cor([0.30, 0.42, 0.62]),
        cor([0.85, 0.68, 0.18]),
        cor([0.75, 0.18, 0.13]),
        cor([0.20, 0.58, 0.30]),
        cor([0.85, 0.45, 0.15]),
        cor([0.85, 0.45, 0.15]),
    ]
}
