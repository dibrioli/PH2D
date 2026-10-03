//! ⭐⭐⭐ **A CENA 37 — O RENDER POR MALHA** (02/10): seis objetos soltos sobre o chão, cada um da
//! sua cor; um deles COMPOSTO (uma caixa mordida por uma esfera, que tem de ser UM objeto), e uma
//! bolinha a FLUTUAR (a sombra dela é mole; a das peças pousadas é dura onde tocam o chão).

use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_ecs::FieldMaterial;
use ph2d_field_eval::leaf;

fn em(x: f32, y: f32, z: f32) -> Xform {
    Xform::at(x, y, z)
}

/// O documento da cena 37.
///
/// # Errors
/// Só se uma das formas violar uma cerca do documento.
pub fn cena_37() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!("[field-smoke] cena 37 — O RENDER POR MALHA: seis objetos soltos sobre o chao.");
    println!(
        "[field-smoke]            (1) MODEL · painel do topo, Shading · Render. Os seis viram objetos \
         de malha: a imagem fica NITIDA na hora, a girar e a aproximar, sem granulado."
    );
    println!(
        "[field-smoke]            (2) clique num objeto: ele fica selecionado INTEIRO. Arraste a seta \
         do gizmo: ele anda, e a sombra dele anda junto no chao."
    );
    println!(
        "[field-smoke]            (3) a caixa VERDE mordida pela esfera e' UM objeto so' (o corte \
         branco vai junto). A bolinha laranja flutua: a sombra dela e' MOLE; a das pecas pousadas \
         e' DURA onde tocam o chao."
    );
    println!(
        "[field-smoke]            (4) arraste a bola amarela ate' encostar na caixa vermelha e solte: \
         as duas viram UM objeto (clique de novo e as duas ficam selecionadas)."
    );
    let quarto = core::f32::consts::FRAC_1_SQRT_2;
    let caixa = |h: f32, at: Xform| {
        leaf(
            Primitive::Box {
                half: [h, h, h],
                round: 0.02,
                chamfer: 0.0,
            },
            at,
        )
    };
    let nodes = vec![
        // 0 — a caixa VERMELHA
        caixa(0.18, em(-1.1, 0.18, 0.0)),
        // 1 — o cilindro AZUL, de pé (o eixo dele é Z; um quarto de volta em X põe-no em Y)
        leaf(
            Primitive::Cylinder {
                radius: 0.16,
                half_height: 0.22,
                round: 0.02,
                chamfer: 0.0,
            },
            Xform {
                translation: [-0.55, 0.22, 0.0],
                rotation: [quarto, 0.0, 0.0, quarto],
                scale: 1.0,
            },
        ),
        // 2 — a bola AMARELA
        leaf(Primitive::Sphere { radius: 0.2 }, em(0.0, 0.2, 0.0)),
        // 3, 4 — a caixa VERDE e a esfera que a morde (o corte fica BRANCO)
        caixa(0.2, em(0.55, 0.2, 0.0)),
        leaf(Primitive::Sphere { radius: 0.24 }, em(0.75, 0.4, 0.2)),
        // 5 — a mordida
        Node::new(
            Xform::IDENTITY,
            NodeKind::Combine {
                op: Op::Difference(Blend::Sharp),
                children: vec![NodeId(3), NodeId(4)],
            },
        ),
        // 6 — o nó de toro ROXO, de pé
        leaf(
            Primitive::TorusKnot {
                radius: 0.16,
                tube: 0.05,
                cord: ph2d_field::knot_cord_ceiling(0.16, 0.05, 2, 3) * 0.85,
                winds: 2,
                loops: 3,
            },
            em(1.15, 0.27, 0.0),
        ),
        // 7 — a bolinha LARANJA, a flutuar
        leaf(Primitive::Sphere { radius: 0.1 }, em(0.0, 0.75, 0.25)),
        Node::new(
            Xform::IDENTITY,
            NodeKind::Combine {
                op: Op::Union(Blend::Sharp),
                children: [0, 1, 2, 5, 6, 7].map(NodeId).to_vec(),
            },
        ),
    ];
    FieldDoc::new(nodes, NodeId(8))
}

/// As cores, na ordem das FOLHAS (a da Hierarquia): vermelha, azul, amarela, verde, branco (o
/// corte), roxa, laranja.
#[must_use]
pub fn materiais() -> Vec<FieldMaterial> {
    let cor = |c: [f32; 3]| FieldMaterial {
        base_color: c,
        ..FieldMaterial::default()
    };
    vec![
        cor([0.80, 0.10, 0.08]),
        cor([0.08, 0.25, 0.80]),
        cor([0.90, 0.70, 0.08]),
        cor([0.12, 0.60, 0.20]),
        cor([0.92, 0.92, 0.90]),
        cor([0.45, 0.15, 0.70]),
        cor([0.95, 0.45, 0.05]),
    ]
}

/// ⭐⭐⭐ **A CENA 38 — O CÉU** (02/10): quatro bolas e um nó, cada um mostra o céu de um jeito — o
/// CROMO espelha-o, o OURO tinge-o, o plástico AZUL brilhante põe um reflexo nítido por cima da cor,
/// a borracha VERMELHA fosca só recebe a luz dele, e o nó BRANCO mostra de onde ela vem.
///
/// # Errors
/// Só se uma das formas violar uma cerca do documento.
pub fn cena_38() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!("[field-smoke] cena 38 — O CEU: quatro bolas e um no' que mostram o ceu de jeitos diferentes.");
    println!(
        "[field-smoke]            (1) MODEL · painel do topo, Shading · Render. No painel, secao Sky, \
         escolha Sunset: o por do sol aparece ATRAS das pecas e a bola de CROMO espelha-o."
    );
    println!(
        "[field-smoke]            (2) Rotation: o ceu gira e os reflexos andam nas bolas. Strength: \
         mais claro ou mais escuro. Key Light em 0: some a luz de cima e a sombra dela."
    );
    println!(
        "[field-smoke]            (3) Background em Off: o ceu some de tras mas continua a iluminar. \
         Sky em Studio volta a luz de sempre."
    );
    let bola = |r: f32, x: f32| leaf(Primitive::Sphere { radius: r }, em(x, r, 0.0));
    let nodes = vec![
        // 0 — CROMO
        bola(0.22, -1.0),
        // 1 — OURO
        bola(0.22, -0.5),
        // 2 — plástico AZUL brilhante
        bola(0.22, 0.0),
        // 3 — borracha VERMELHA fosca
        bola(0.22, 0.5),
        // 4 — o nó BRANCO, de pé
        leaf(
            Primitive::TorusKnot {
                radius: 0.16,
                tube: 0.05,
                cord: ph2d_field::knot_cord_ceiling(0.16, 0.05, 2, 3) * 0.85,
                winds: 2,
                loops: 3,
            },
            em(1.0, 0.27, 0.0),
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

/// Os materiais da cena 38, na ordem das folhas: cromo, ouro, azul brilhante, vermelho fosco, branco.
#[must_use]
pub fn materiais_38() -> Vec<FieldMaterial> {
    let m = |c: [f32; 3], metal: f32, rugoso: f32| FieldMaterial {
        base_color: c,
        metalness: metal,
        roughness: rugoso,
        ..FieldMaterial::default()
    };
    vec![
        m([0.95, 0.95, 0.95], 1.0, 0.04),
        m([1.0, 0.78, 0.34], 1.0, 0.25),
        m([0.06, 0.2, 0.85], 0.0, 0.08),
        m([0.75, 0.1, 0.07], 0.0, 0.85),
        m([0.9, 0.9, 0.88], 0.0, 0.4),
    ]
}
