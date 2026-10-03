//! Os gates do **chão que só recebe** — onde ele pousa ([`crate::lowest_point`]). Os da sombra e
//! do céu dele saíram com o Render traçado (03/10): quem o desenha é o `ph2d-mesh-forward`.

use crate::lowest_point;
use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_eval::hybrid::Registry;

fn esfera_em(c: [f32; 3], r: f32) -> FieldDoc {
    FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Sphere { radius: r },
            Xform::at(c[0], c[1], c[2]),
        )],
        NodeId(0),
    )
    .expect("a esfera")
}

// ── a lei do raio ─────────────────────────────────────────────────────────────────────────────

// ── onde o chão está ──────────────────────────────────────────────────────────────────────────

/// ⭐ A esfera: o ponto mais baixo é `c.y − r`, e a busca acha-o à tolerância do olhar fino.
#[test]
fn o_ponto_mais_baixo_de_uma_esfera() {
    let doc = esfera_em([0.2, 0.3, -0.1], 0.25);
    let y = lowest_point(&doc, &Registry::new()).expect("a esfera tem geometria");
    assert!(
        (y - 0.05).abs() <= 1e-4,
        "o ponto mais baixo saiu {y}, e é 0,05"
    );
}

/// A pose que põe um cubo de pé numa quina (a diagonal na vertical).
fn de_pe_na_quina(t: [f32; 3]) -> Xform {
    Xform {
        translation: t,
        rotation: ph2d_field::xform::quat_mul(
            ph2d_field::xform::quat_axis_angle([1.0, 0.0, 0.0], 0.615_479_7),
            ph2d_field::xform::quat_axis_angle([0.0, 0.0, 1.0], std::f32::consts::FRAC_PI_4),
        ),
        scale: 1.0,
    }
}

/// ⭐⭐ **Um cubo de pé numa quina** — a quina é um ponto de medida nula, e o olhar amostra o centro
/// dos pixels: é o caso que pede o TERCEIRO olhar. O esperado sai dos oito cantos, rodados pela
/// mesma pose que o avaliador usa.
#[test]
fn o_ponto_mais_baixo_de_um_cubo_de_pe_na_quina() {
    let pose = de_pe_na_quina([0.1, 0.4, 0.0]);
    let h = 0.2f32;
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Box {
                half: [h; 3],
                round: 0.0,
                chamfer: 0.0,
            },
            pose,
        )],
        NodeId(0),
    )
    .expect("o cubo");
    let mut esperado = f32::INFINITY;
    for sx in [-h, h] {
        for sy in [-h, h] {
            for sz in [-h, h] {
                esperado = esperado.min(pose.apply([sx, sy, sz])[1]);
            }
        }
    }
    let y = lowest_point(&doc, &Registry::new()).expect("o cubo tem geometria");
    assert!(
        (y - esperado).abs() <= 1e-4,
        "a quina está a {esperado} e a busca achou {y}"
    );
}

/// ⭐⭐ **O CONTROLO que diz porque a busca existe: um cilindro INCLINADO, onde a caixa mente.**
///
/// A borda mais baixa de um cilindro de eixo `u` (raio `ρ`, meia-altura `H`) está a
/// `H·|u_y| + ρ·√(1 − u_y²)` abaixo do centro; a caixa rodada soma os dois eixos transversais
/// separadamente (`ρ·(|x_y| + |z_y|)`), e desce abaixo dela. ⚠️ Num cubo a caixa é EXACTA (a quina é
/// um canto dela) — a 1.ª redacção deste controlo usava o cubo e reprovou sobre isso.
#[test]
fn num_cilindro_inclinado_a_caixa_desce_abaixo_da_peca_e_a_busca_nao() {
    let pose = de_pe_na_quina([0.0, 0.3, 0.0]);
    let (rho, hh) = (0.1f32, 0.25f32);
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Cylinder {
                radius: rho,
                half_height: hh,
                round: 0.0,
                chamfer: 0.0,
            },
            pose,
        )],
        NodeId(0),
    )
    .expect("o cilindro");
    // ⚠️ O eixo do `Cylinder` é o `z` (medido: o interior está em `(0, 0, ±h)`).
    let u_y = pose.apply_dir([0.0, 0.0, 1.0])[1];
    let esperado = 0.3 - (hh * u_y.abs() + rho * (1.0 - u_y * u_y).max(0.0).sqrt());
    let reg = Registry::new();
    let y = lowest_point(&doc, &reg).expect("o cilindro tem geometria");
    assert!(
        (y - esperado).abs() <= 1e-4,
        "a borda está a {esperado} e a busca achou {y}"
    );
    let caixa = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
        .expect("a bola")
        .aabb()
        .0[1];
    assert!(
        caixa < esperado - 0.005,
        "a caixa ({caixa}) já não mente sobre a borda ({esperado}) — o controlo deixou de controlar"
    );
}

#[test]
fn o_ponto_mais_baixo_de_duas_pecas_e_o_da_de_baixo() {
    let folha = |c: [f32; 3], r: f32| {
        ph2d_field_eval::leaf(Primitive::Sphere { radius: r }, Xform::at(c[0], c[1], c[2]))
    };
    let doc = FieldDoc::new(
        vec![
            folha([-0.3, 0.5, 0.0], 0.2),
            folha([0.3, 0.1, 0.2], 0.15),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(2),
    )
    .expect("as duas");
    let y = lowest_point(&doc, &Registry::new()).expect("geometria");
    assert!(
        (y + 0.05).abs() < 3e-4,
        "a de baixo toca em −0,05 e saiu {y}"
    );
}
