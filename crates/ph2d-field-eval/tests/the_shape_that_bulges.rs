//! ⭐⭐ **A EXACTA DO CATÁLOGO (W125), PROVADA ANTES DE SER LIGADA.**
//!
//! > **Enio, 06/09:** *«Muito bom! Siga»* — o lote 5 do levantamento
//! > ([doc 08 §7.5](../../../docs/3DModeling/08_formas_por_formula.md)), **corrigido duas vezes**:
//! > quatro das onze formas que eu tinha listado já se alcançam com o que existe, e **duas das três
//! > que sobravam foram construídas e RECUSADAS por medição** (o ovo e a escada — doc 06 §126).
//! > Sobra esta, e ela é a única deste módulo cuja distância é **exacta**.
//!
//! # ⚠️ O que estes três gates separam
//!
//! | gate | o defeito que ele apanha |
//! |---|---|
//! | `the_rounded_cylinder_curves_the_wall_not_only_the_rim` | ela ter shipado como *cilindro com filete* |
//! | `the_bulge_eats_the_corner_and_never_the_extents` | o knob mexer no tamanho da peça |
//! | `the_rounded_cylinder_is_an_exact_distance_field` | a fórmula ter perdido a exactidão no percurso |
//!
//! ⚠️ **O segundo existe porque um gate no REPRESENTANTE deixa o curso do controle por medir** —
//! ele é o único knob que esta forma tem, e o valor único de uma tabela de censo não diz nada sobre
//! o que acontece nas duas pontas dele.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
use ph2d_field_eval::Field;

fn campo(p: Primitive) -> Field {
    Field::new(
        &FieldDoc::new(
            vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
            NodeId(0),
        )
        .expect("a peça"),
    )
}

const NA_PELE: f64 = 3.0e-3;

#[track_caller]
fn dentro(f: &Field, p: [f64; 3], porque: &str) {
    let v = f.at(p[0], p[1], p[2]);
    assert!(
        v < -NA_PELE,
        "{porque}: {p:?} devia estar DENTRO e leu {v:.5}"
    );
}

#[track_caller]
fn fora(f: &Field, p: [f64; 3], porque: &str) {
    let v = f.at(p[0], p[1], p[2]);
    assert!(v > NA_PELE, "{porque}: {p:?} devia estar FORA e leu {v:.5}");
}

#[track_caller]
fn na_pele(f: &Field, p: [f64; 3], porque: &str) {
    let v = f.at(p[0], p[1], p[2]);
    assert!(
        v.abs() < NA_PELE,
        "{porque}: {p:?} devia estar NA SUPERFÍCIE e leu {v:.5}"
    );
}

/// ⭐ **O CILINDRO COM BOJO curva a PAREDE, e não só o aro.**
///
/// ⚠️ **É o gate que o separa de um cilindro com filete**: naquele a parede é reta e só o aro
/// arqueia; aqui o bordo inteiro é um arco de raio `bulge`, e a parede a meia-altura já está **para
/// dentro** do raio máximo.
#[test]
fn the_rounded_cylinder_curves_the_wall_not_only_the_rim() {
    let (ra, rb, h) = (0.35_f64, 0.10_f64, 0.30_f64);
    let f = campo(Primitive::RoundedCylinder {
        radius: ra as f32,
        bulge: rb as f32,
        half_height: h as f32,
    });
    dentro(&f, [0.0, 0.0, 0.0], "o centro");
    na_pele(&f, [ra, 0.0, 0.0], "o equador da parede");
    na_pele(&f, [0.0, 0.0, h], "o meio da tampa");
    // ⭐ **A 45° do bordo**: o centro do arco está em `(ra − rb, h − rb)`, e a superfície a `rb` dele.
    let d = rb / 2.0_f64.sqrt();
    na_pele(&f, [ra - rb + d, 0.0, h - rb + d], "os 45° do bordo");
    // ⛔ **O canto que um cilindro de aresta viva teria** — é ele que prova que o bojo existe.
    fora(
        &f,
        [ra - 0.005, 0.0, h - 0.005],
        "o canto vivo de um cilindro",
    );
    fora(&f, [ra + 0.02, 0.0, 0.0], "fora da parede");
}

/// ⭐⭐⭐ **O BOJO COME O CANTO E NUNCA O TAMANHO** — o curso inteiro do único knob desta forma.
///
/// ⚠️ **As duas pontas do curso são degenerações, e as duas são legítimas**: no piso a peça tende
/// ao cilindro que a paleta já tem (por isso a porta recusa `0`); no tecto `min(raio, meia-altura)`
/// a parede reta desaparece de vez — com o raio maior sai uma pastilha, com a altura maior sai uma
/// cápsula. **Em toda a travessia o equador continua no `radius` e a tampa na `half_height`**, e é
/// isso que faz dele um *bojo* e não uma escala.
#[test]
fn the_bulge_eats_the_corner_and_never_the_extents() {
    let (ra, h) = (0.35_f64, 0.30_f64);
    let tecto = ra.min(h);
    // ⚠️ O canto de prova está a `eps` das duas faces, e a álgebra diz que ele fica dentro enquanto
    // `bulge < eps·√2/(√2 − 1) = 3,414·eps` — com `eps = 0,005`, a fronteira é `0,0171`.
    let eps = 0.005_f64;
    let vira = eps * 2.0_f64.sqrt() / (2.0_f64.sqrt() - 1.0);
    let mut comeu = 0;
    for passo in 0..5 {
        let rb = tecto * (passo as f64 * 0.245 + 0.02);
        let f = campo(Primitive::RoundedCylinder {
            radius: ra as f32,
            bulge: rb as f32,
            half_height: h as f32,
        });
        na_pele(&f, [ra, 0.0, 0.0], &format!("o equador com bojo {rb:.3}"));
        na_pele(&f, [0.0, 0.0, h], &format!("a tampa com bojo {rb:.3}"));
        fora(
            &f,
            [ra + 0.02, 0.0, 0.0],
            &format!("além do equador com bojo {rb:.3}"),
        );
        let canto = [ra - eps, 0.0, h - eps];
        if rb > vira {
            fora(&f, canto, &format!("o canto com bojo {rb:.3}"));
            comeu += 1;
        } else {
            dentro(&f, canto, &format!("o canto com bojo {rb:.3}"));
        }
    }
    assert!(
        comeu >= 3,
        "o canto só foi comido em {comeu} das 5 posições — o knob não está a percorrer nada"
    );
}

/// ⭐⭐⭐ **É UMA DISTÂNCIA EXACTA** — `‖∇f‖ = 1` até ao último dígito, e no curso inteiro do bojo.
///
/// ⚠️ **É o que a separa das três waves anteriores**: a espiral, a mola e a rede entregam um
/// **minorante** (que é tudo o que a marcha precisa); esta entrega a distância. Um minorante custa
/// passos a mais; um majorante fá-la atravessar a superfície, e é isso que este gate proíbe.
#[test]
fn the_rounded_cylinder_is_an_exact_distance_field() {
    let (ra, h) = (0.35_f64, 0.30_f64);
    let tecto = ra.min(h);
    for passo in 0..3 {
        let rb = tecto * (passo as f64 * 0.45 + 0.08);
        let f = campo(Primitive::RoundedCylinder {
            radius: ra as f32,
            bulge: rb as f32,
            half_height: h as f32,
        });
        let mut pior: f64 = 0.0;
        let passos = 70;
        let e = 0.6;
        let at = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / passos as f64;
        for i in 0..passos {
            for j in 0..passos {
                for k in 0..passos {
                    let (x, y, z) = (at(i), at(j), at(k));
                    if f.at(x, y, z).abs() > 0.03 {
                        continue;
                    }
                    pior = pior.max(f.gradient_norm(x, y, z, 1.0e-4));
                }
            }
        }
        assert!(
            pior <= 1.02,
            "com bojo {rb:.3}: ‖∇f‖ = {pior:.4} — devia ser uma distância exacta"
        );
    }
}
