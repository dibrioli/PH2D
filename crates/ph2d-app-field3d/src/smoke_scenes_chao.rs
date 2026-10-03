//! ⭐⭐⭐ **A CENA 41 — O CHÃO QUE TAPA** (03/10): a metade de baixo das peças vê o CHÃO, não o céu
//! ([`ph2d_mesh_forward::chao_tapa`]). A bola BRANCA pousada escurece na barriga, mais junto ao
//! contacto; a de CROMO pousada reflete o chão escuro debaixo dela; a bola pequena a `4 cm` do chão
//! escurece por baixo menos que a pousada (o céu passa no vão); a caixa escurece ao pé das faces.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_ecs::FieldMaterial;
use ph2d_field_eval::leaf;

/// O documento da cena 41 — a fila ao longo de Z, como a 40 (a câmara de fábrica atravessa Z).
///
/// # Errors
/// Só se uma das formas violar uma cerca do documento.
pub fn cena_41() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 41 — O CHAO QUE TAPA: a parte de baixo das pecas ve o chao, nao o ceu."
    );
    println!(
        "[field-smoke]            (1) MODEL · painel do topo, Shading · Render. A camara abre de LADO, \
         rente ao chao: arraste um pouco para CIMA (uns 5 a 10 graus) para ver o chao e a parte de baixo."
    );
    println!(
        "[field-smoke]            (2) a bola BRANCA pousada escurece na barriga, mais onde encosta; a de \
         CROMO mostra o chao escuro debaixo dela no reflexo."
    );
    println!(
        "[field-smoke]            (3) a bola pequena flutua 4 cm acima do chao: por baixo escurece MENOS \
         que a pousada (o ceu passa no vao); a caixa escurece ao pe' de cada face."
    );
    let nodes = vec![
        // 0 — a bola BRANCA, pousada
        leaf(Primitive::Sphere { radius: 0.3 }, Xform::at(0.0, 0.3, -0.7)),
        // 1 — a bola de CROMO, pousada
        leaf(
            Primitive::Sphere { radius: 0.22 },
            Xform::at(0.0, 0.22, -0.05),
        ),
        // 2 — a bola pequena, 4 cm acima do chão
        leaf(
            Primitive::Sphere { radius: 0.15 },
            Xform::at(0.0, 0.19, 0.45),
        ),
        // 3 — a caixa, pousada
        leaf(
            Primitive::Box {
                half: [0.15, 0.15, 0.15],
                round: 0.01,
                chamfer: 0.0,
            },
            Xform::at(0.0, 0.15, 0.95),
        ),
        Node::new(
            Xform::IDENTITY,
            NodeKind::Combine {
                op: Op::Union(ph2d_field::Blend::Sharp),
                children: [0, 1, 2, 3].map(NodeId).to_vec(),
            },
        ),
    ];
    FieldDoc::new(nodes, NodeId(4))
}

/// Os materiais, na ordem das folhas: branco fosco, cromo, branco fosco, cinzento médio.
#[must_use]
pub fn materiais_41() -> Vec<FieldMaterial> {
    let m = |c: [f32; 3], metal: f32, rugoso: f32| FieldMaterial {
        base_color: c,
        metalness: metal,
        roughness: rugoso,
        ..FieldMaterial::default()
    };
    vec![
        m([0.85, 0.85, 0.85], 0.0, 0.6),
        m([0.95, 0.95, 0.95], 1.0, 0.05),
        m([0.85, 0.85, 0.85], 0.0, 0.6),
        m([0.45, 0.47, 0.5], 0.0, 0.6),
    ]
}
