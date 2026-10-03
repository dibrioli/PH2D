//! ⭐⭐⭐ **A CENA 39 — AS TEXTURAS** (03/10, handoff `AS_TEXTURAS`): cinco formas, cada uma com uma
//! textura do pacote, no tamanho real dela — a bola de PEDRA mostra as três vistas a misturarem-se, a
//! caixa de TIJOLO as faces nítidas, o cilindro de MADEIRA, o toro de CHAPA de metal (o relevo e a
//! rugosidade num metal) e a cápsula de COURO.

use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_ecs::{FieldMaterial, FieldTexture};
use ph2d_field_eval::leaf;

fn em(x: f32, y: f32, z: f32) -> Xform {
    Xform::at(x, y, z)
}

/// # Errors
/// Só se uma das formas violar uma cerca do documento.
pub fn cena_39() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!("[field-smoke] cena 39 — AS TEXTURAS: cinco formas com as texturas do pacote.");
    println!(
        "[field-smoke]            (1) MODEL · painel do topo, Shading · Render. Clique numa forma: \
         a secao Texture mostra qual e' a dela (Stone, Bricks, Wood, Metal Plate, Leather)."
    );
    println!(
        "[field-smoke]            (2) Tile Size muda o tamanho do desenho; Blend amacia a costura \
         na bola; Bumps em 0 deixa a forma lisa, em 2 o relevo fica mais fundo."
    );
    println!(
        "[field-smoke]            (3) Texture: troque por outra, ou From File... para usar uma \
         imagem sua. Gire a camara: a textura fica presa a' forma."
    );
    let quarto = std::f32::consts::FRAC_1_SQRT_2;
    let nodes = vec![
        // 0 — a bola de PEDRA
        leaf(Primitive::Sphere { radius: 0.15 }, em(0.0, 0.15, 0.7)),
        // 1 — a caixa de TIJOLO
        leaf(
            Primitive::Box {
                half: [0.14, 0.14, 0.14],
                round: 0.01,
                chamfer: 0.0,
            },
            em(0.0, 0.14, 0.35),
        ),
        // 2 — o cilindro de MADEIRA, de pé (o eixo dele é Z; um quarto de volta em X põe-no em Y)
        leaf(
            Primitive::Cylinder {
                radius: 0.12,
                half_height: 0.17,
                round: 0.02,
                chamfer: 0.0,
            },
            Xform {
                translation: [0.0, 0.17, 0.0],
                rotation: [quarto, 0.0, 0.0, quarto],
                scale: 1.0,
            },
        ),
        // 3 — o toro de CHAPA, de pé (o eixo dele é Z: o anel olha para a câmara)
        leaf(
            Primitive::Torus {
                major: 0.12,
                minor: 0.045,
            },
            Xform {
                rotation: [0.0, quarto, 0.0, quarto],
                ..em(0.0, 0.165, -0.35)
            },
        ),
        // 4 — a cápsula de COURO, de pé (o eixo dela é Z, como o do cilindro)
        leaf(
            Primitive::Capsule {
                radius: 0.1,
                half_height: 0.1,
            },
            Xform {
                translation: [0.0, 0.2, -0.7],
                rotation: [quarto, 0.0, 0.0, quarto],
                scale: 1.0,
            },
        ),
        Node::new(
            Xform::IDENTITY,
            NodeKind::Combine {
                op: Op::Union(Blend::Sharp),
                children: [0, 1, 2, 3, 4].map(NodeId).to_vec(),
            },
        ),
    ];
    FieldDoc::new(nodes, NodeId(5))
}

/// Os materiais, na ordem das folhas: BRANCOS (a cor tinge a textura; branco mostra-a como é), e a
/// chapa é METAL.
#[must_use]
pub fn materiais_39() -> Vec<FieldMaterial> {
    let branco = FieldMaterial {
        base_color: [1.0; 3],
        ..FieldMaterial::default()
    };
    vec![
        branco,
        branco,
        branco,
        FieldMaterial {
            base_color: [0.95; 3],
            metalness: 1.0,
            ..FieldMaterial::default()
        },
        branco,
    ]
}

/// As texturas, na ordem das folhas: pedra, tijolo, madeira, chapa, couro (o índice do painel).
#[must_use]
pub fn texturas_39() -> Vec<Option<FieldTexture>> {
    [3u8, 1, 2, 4, 6]
        .into_iter()
        .map(|source| {
            Some(FieldTexture {
                source,
                ..FieldTexture::default()
            })
        })
        .collect()
}
