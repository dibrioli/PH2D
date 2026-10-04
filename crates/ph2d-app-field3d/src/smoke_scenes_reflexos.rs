//! ⭐⭐⭐ **A CENA 42 — OS REFLEXOS** (04/10): a bola de CROMO no meio mostra as VIZINHAS coloridas, e
//! as sombras delas no chão, no reflexo (as capturas de reflexo do `ph2d_mesh_forward`); a bola de
//! ALUMÍNIO ESCOVADO mostra-as borradas.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_ecs::FieldMaterial;
use ph2d_field_eval::leaf;

fn caixa(h: f32, x: f32, z: f32) -> Node {
    leaf(
        Primitive::Box {
            half: [h, h, h],
            round: 0.01,
            chamfer: 0.0,
        },
        Xform::at(x, h, z),
    )
}

fn bola(r: f32, x: f32, z: f32) -> Node {
    leaf(Primitive::Sphere { radius: r }, Xform::at(x, r, z))
}

/// O documento da cena 42 — a fila ao longo de Z (a câmara de fábrica atravessa Z), uma vizinha do
/// lado da câmara e outra atrás do cromo.
///
/// # Errors
/// Só se uma das formas violar uma cerca do documento.
pub fn cena_42() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 42 — OS REFLEXOS: a bola de CROMO mostra as vizinhas e as sombras delas."
    );
    println!(
        "[field-smoke]            (1) MODEL · painel do topo, Shading · Render. A bola de CROMO e' a \
         do meio."
    );
    println!(
        "[field-smoke]            (2) no CROMO aparecem a bola VERMELHA, a bola AMARELA e a caixa AZUL \
         (e a VERDE, girando a camara), cada uma do lado onde esta'."
    );
    println!(
        "[field-smoke]            (3) a bola de ALUMINIO ESCOVADO (atras, ao lado da caixa verde) mostra \
         as vizinhas BORRADAS. Gire a camara: os reflexos andam com a vista; mova uma bola: o reflexo \
         segue-a."
    );
    let nodes = vec![
        // 0 — o CROMO, no meio
        bola(0.3, 0.0, 0.0),
        // 1 — a bola VERMELHA, à esquerda na fila
        bola(0.2, 0.0, -0.7),
        // 2 — a caixa VERDE, à direita na fila
        caixa(0.18, 0.0, 0.72),
        // 3 — a bola AMARELA, do lado da câmara
        bola(0.17, 0.55, -0.25),
        // 4 — a caixa AZUL, atrás do cromo
        caixa(0.15, -0.6, 0.2),
        // 5 — o ALUMÍNIO ESCOVADO, a última da fila
        bola(0.2, 0.0, 1.35),
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

/// Os materiais, na ordem das folhas: cromo, vermelho, verde, amarelo e azul foscos, alumínio
/// escovado.
#[must_use]
pub fn materiais_42() -> Vec<FieldMaterial> {
    let m = |c: [f32; 3], metal: f32, rugoso: f32| FieldMaterial {
        base_color: c,
        metalness: metal,
        roughness: rugoso,
        ..FieldMaterial::default()
    };
    vec![
        m([0.95, 0.95, 0.95], 1.0, 0.05),
        m([0.8, 0.1, 0.08], 0.0, 0.6),
        m([0.1, 0.65, 0.15], 0.0, 0.6),
        m([0.9, 0.75, 0.1], 0.0, 0.6),
        m([0.1, 0.25, 0.85], 0.0, 0.6),
        m([0.9, 0.9, 0.92], 1.0, 0.35),
    ]
}
