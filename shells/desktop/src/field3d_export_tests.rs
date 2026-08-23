//! **Os gates da SAÍDA para arquivo** — e a sonda que decidiu qual número o artista lê.
//!
//! ⚠️ O `field3d_export` em si abre um diálogo nativo (`rfd`) e não é alcançável de um teste. O que
//! se prende aqui é a metade **pura**: o que se conta sobre a malha que de facto saiu.

use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};

fn leaf(p: Primitive, x: f32) -> Node {
    Node {
        xform: Xform::at(x, 0.0, 0.0),
        kind: NodeKind::Leaf(p),
        mods: Vec::new(),
    }
}

fn one(p: Primitive) -> FieldDoc {
    FieldDoc::new(vec![leaf(p, 0.0)], NodeId(0)).expect("uma forma")
}

/// Uma peça **fora do centro**, para o bordo e a peça não coincidirem por acidente.
fn two_apart() -> FieldDoc {
    FieldDoc::new(
        vec![
            leaf(Primitive::Sphere { radius: 0.15 }, -0.6),
            leaf(Primitive::Sphere { radius: 0.15 }, 0.6),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
                mods: Vec::new(),
            },
        ],
        NodeId(2),
    )
    .expect("duas esferas afastadas")
}

fn mesh_of(doc: &FieldDoc, depth: u8) -> ph2d_mesh::Mesh {
    let reg = crate::field3d_smoke::sampled_registry();
    ph2d_field_eval::extract::extract(doc, &reg, depth).expect("extrai")
}

/// ⭐ **A SONDA QUE ESCOLHEU O NÚMERO** — o bordo da grade contra o tamanho da malha.
///
/// Há dois números disponíveis e eles **não são o mesmo**:
///
/// | candidato | o que é | por que NÃO serve |
/// |---|---|---|
/// | a caixa do **bordo** (`bounds::bounding_ball().aabb()`) | o cubo que envolve a **esfera** que contém a peça — e a grade ainda lhe soma `PAD_FRACTION` (5 %) por cima | é **andaime**: conservador por construção, e cúbico — um objeto fino reporta o lado maior nos três eixos |
/// | a caixa da **malha** (`Mesh::bounds()`) | o que de facto foi escrito no arquivo | ⭐ é a resposta à pergunta *"que tamanho isto tem no Blender?"* |
///
/// Rode com `--ignored --nocapture`.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_the_grid_box_against_the_real_piece() {
    let reg = crate::field3d_smoke::sampled_registry();
    let cases: [(&str, FieldDoc); 3] = [
        ("esfera r=0,40", one(Primitive::Sphere { radius: 0.4 })),
        (
            "caixa FINA 0,80 x 0,80 x 0,04",
            one(Primitive::Box {
                half: [0.4, 0.4, 0.02],
                round: 0.0,
            }),
        ),
        ("duas esferas afastadas", two_apart()),
    ];
    println!(
        "\n{:<28}  {:>22}  {:>22}  razão",
        "peça", "bordo (x,y,z)", "malha (x,y,z)"
    );
    for (name, doc) in cases {
        let ball = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("tem geometria");
        let (lo, hi) = ball.aabb();
        let grid = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
        let b = mesh_of(&doc, 7).bounds();
        let real = [
            b.max[0] - b.min[0],
            b.max[1] - b.min[1],
            b.max[2] - b.min[2],
        ];
        let ratio: Vec<String> = (0..3)
            .map(|k| format!("{:.2}x", grid[k] / real[k].max(1e-6)))
            .collect();
        println!(
            "{name:<28}  {:>6.3}{:>8.3}{:>8.3}  {:>6.3}{:>8.3}{:>8.3}  {}",
            grid[0],
            grid[1],
            grid[2],
            real[0],
            real[1],
            real[2],
            ratio.join(" ")
        );
    }
}

/// ⭐ **O tamanho que a exportação diz é o da MALHA que saiu** — não o do andaime.
///
/// ⚠️ **A sonda irmã mede a diferença, e ela não é pequena:** a caixa da grade é o cubo que envolve
/// a **esfera** de bordo mais 5 % de folga, então ela é **cúbica** e conservadora por construção.
/// Numa peça fina os dois números divergem por mais de uma ordem de grandeza no eixo curto — dizer o
/// do andaime seria responder *"que tamanho tem a caixa em que eu desenhei"* a quem perguntou
/// *"que tamanho tem a peça"*.
#[test]
fn the_reported_size_is_the_mesh_that_shipped_not_the_grid_that_built_it() {
    let doc = one(Primitive::Box {
        half: [0.4, 0.4, 0.02],
        round: 0.0,
    });
    let mesh = mesh_of(&doc, 7);
    let said = super::piece_size(&mesh);
    let b = mesh.bounds();
    for (k, s) in said.iter().enumerate() {
        let real = b.max[k] - b.min[k];
        assert!(
            (s - real).abs() < 1e-6,
            "o eixo {k} disse {s} e a malha mede {real}"
        );
    }
    // ⭐ O eixo CURTO é o que separa as duas respostas: a caixa da grade é cúbica.
    let reg = crate::field3d_smoke::sampled_registry();
    let ball = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("tem geometria");
    let (lo, hi) = ball.aabb();
    assert!(
        (hi[2] - lo[2]) > said[2] * 5.0,
        "esta fixture existe para os dois números DIVERGIREM no eixo curto — \
         grade {} contra malha {}; se convergiram, ela deixou de provar o que prova",
        hi[2] - lo[2],
        said[2]
    );
}

/// ⚠️ **Uma malha VAZIA não inventa um tamanho.** O `Aabb::EMPTY` é invertido de propósito, e
/// subtrair as pontas dele daria números negativos — que num toast leem como um defeito da peça.
#[test]
fn an_empty_mesh_reports_zero_instead_of_a_negative_size() {
    let empty = ph2d_mesh::Mesh::default();
    assert_eq!(
        super::piece_size(&empty),
        [0.0; 3],
        "uma malha sem vértices tem de reportar zero, não a caixa invertida"
    );
}
