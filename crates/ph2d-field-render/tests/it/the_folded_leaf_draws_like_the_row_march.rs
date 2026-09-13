//! ⛔⛔ **UMA FOLHA DE PERFIL DEBAIXO DE UMA OPERAÇÃO QUE REMAPEIA DESENHA-SE COMO A MARCHA HONESTA**
//! (auditoria da W148, 2026-09-13).
//!
//! A marcha por ladrilho especializa cada folha de perfil contra a região do ladrilho, e o mapa que
//! leva a região ao plano da folha compõe **poses**. Um espelho, uma matriz ou uma torção numa
//! **operação** (um gesto do produto desde a W79) dobra os filhos — e a região da cópia dobrada chega
//! à folha num sítio onde a especialização não guardou as arestas certas. Medido no documento
//! (`ph2d-field-eval::tests::the_specialisation_gives_up_under_a_remapping_ancestor`): `0,496` de
//! desacordo dentro da região.
//!
//! ⭐ Este gate mede o que o artista veria: a imagem da marcha por ladrilho contra a da marcha por
//! LINHA, que não especializa nada. ⚠️ **O controlo é o gémeo sem o modificador**: as duas marchas já
//! discordam num pixel ou dois de silhueta por amostragem, e a pergunta é se a dobra acrescenta a isso.

use ph2d_field::{
    Blend, FieldDoc, FillRule, Node, NodeId, NodeKind, Op, Primitive, Profile, Unary, Xform,
};
use ph2d_field_eval::hybrid::Registry;
use ph2d_field_render::{Gbuffer, Orbit, trace_by_rows_for_test, trace_cached_for_test};

/// Uma elipse fina e densa, fora do eixo, e uma esfera pequena — debaixo de uma operação com `mods`.
fn folded(mods: Vec<Unary>) -> FieldDoc {
    let ellipse: Vec<[f32; 2]> = (0..168)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / 168.0;
            [0.3 * a.cos(), 0.06 * a.sin()]
        })
        .collect();
    let nodes = vec![
        Node {
            xform: Xform::at(0.35, 0.1, 0.0),
            kind: NodeKind::Leaf(Primitive::Extrude {
                profile: Profile::new(vec![ellipse], FillRule::NonZero, 1e-4).expect("perfil"),
                half_height: 0.15,
                round: 0.0,
                chamfer: 0.0,
            }),
            mods: Vec::new(),
            verb: None,
        },
        Node {
            xform: Xform::at(0.0, 0.5, 0.0),
            kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.08 }),
            mods: Vec::new(),
            verb: None,
        },
        Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Combine {
                op: Op::Union(Blend::Sharp),
                children: vec![NodeId(0), NodeId(1)],
            },
            mods,
            verb: None,
        },
    ];
    FieldDoc::new(nodes, NodeId(2)).expect("a operação com modificador")
}

fn mismatched(a: &Gbuffer, b: &Gbuffer) -> usize {
    a.hit.iter().zip(&b.hit).filter(|(x, y)| x != y).count()
}

#[test]
fn a_leaf_under_a_folding_operation_draws_like_the_row_march() {
    let reg = Registry::new();
    let (w, h) = (240u32, 144u32);
    let cam = Orbit {
        half_extent: 1.1,
        ..Orbit::default()
    };
    let twin = folded(Vec::new());
    let ctrl = mismatched(
        &trace_cached_for_test(&twin, &reg, &cam, w, h, false, None),
        &trace_by_rows_for_test(&twin, &reg, &cam, w, h),
    );
    let twin_hits = trace_by_rows_for_test(&twin, &reg, &cam, w, h)
        .hit
        .iter()
        .filter(|x| **x)
        .count();
    let mut report = Vec::new();
    for (name, m) in [
        ("espelho", Unary::Mirror { offset: 0.0 }),
        (
            "matriz",
            Unary::Array {
                count: 3,
                spacing: 0.9,
                joint: ph2d_field::Joint::SHARP,
                axis: ph2d_field::mods::ARRAY_AXIS,
            },
        ),
    ] {
        let doc = folded(vec![m]);
        let row = trace_by_rows_for_test(&doc, &reg, &cam, w, h);
        let row_hits = row.hit.iter().filter(|x| **x).count();
        let tiled = trace_cached_for_test(&doc, &reg, &cam, w, h, false, None);
        let pix = mismatched(&tiled, &row);
        println!(
            "{name}: marcha por linha desenha {row_hits} px (o gémeo {twin_hits}) · por ladrilho \
             discorda em {pix} px (o controlo em {ctrl})"
        );
        report.push((name, row_hits, pix));
    }
    for (name, row_hits, pix) in report {
        // ⛔ O controlo positivo da fixture: a dobra tem de pôr mais peça na imagem.
        assert!(
            row_hits * 10 > twin_hits * 13,
            "{name}: a dobra quase não acrescentou peça ({row_hits} px contra {twin_hits}) — a cópia \
             está fora do quadro, e o gate não mede nada"
        );
        assert!(
            pix <= ctrl + 3,
            "{name}: a marcha por ladrilho discorda da por linha em {pix} pixels (o gémeo sem o \
             modificador discorda em {ctrl}) — a folha foi especializada no espaço SEM a dobra do pai"
        );
    }
}
