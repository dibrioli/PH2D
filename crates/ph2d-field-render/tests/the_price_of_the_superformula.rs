//! ⛔⛔⛔ **SONDA da auditoria de 06/09** — *«a superfórmula tem performance menor que as outras
//! formas? Isso é esperado?»* (Enio).
//!
//! ⚠️ **A régua tem de ser o QUADRO, e não uma varredura de campo.** A W128 mediu `3,8×` uma esfera
//! **por amostra** e concluiu que o preço estava bem — mas a árvore é **especializada por ladrilho ×
//! fatia**, e o que viaja com ela é outra coisa.

use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
use ph2d_field_eval::hybrid::Registry;
use ph2d_field_render::{Orbit, SPECIALISED, trace};
use std::sync::atomic::Ordering;

fn doc_de(p: Primitive) -> FieldDoc {
    FieldDoc::new(vec![ph2d_field_eval::leaf(p, Xform::IDENTITY)], NodeId(0)).expect("a peça")
}

fn cronometra(nome: &str, p: Primitive) -> f64 {
    let doc = doc_de(p);
    let reg = Registry::default();
    let cam = Orbit::default();
    // Aquece.
    let _ = trace(&doc, &reg, &cam, 320, 180);
    let antes = SPECIALISED.load(Ordering::Relaxed);
    let scans = ph2d_field_eval::ops_gielis::SCANS.load(Ordering::Relaxed);
    let t0 = std::time::Instant::now();
    let _ = trace(&doc, &reg, &cam, 640, 360);
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    let regioes = SPECIALISED.load(Ordering::Relaxed) - antes;
    let s = ph2d_field_eval::ops_gielis::SCANS.load(Ordering::Relaxed) - scans;
    println!("  {nome:<24} {ms:>8.1} ms   ({regioes} regiões, {s} varreduras do divisor)");
    ms
}

#[test]
#[ignore = "sonda: o preço de um quadro com a superfórmula"]
fn the_price_of_the_superformula() {
    println!("\n── um quadro a 640×360, uma peça só ──");
    let esfera = cronometra("esfera", Primitive::Sphere { radius: 0.35 });
    let caixa = cronometra(
        "caixa",
        Primitive::Box {
            half: [0.3; 3],
            round: 0.05,
            chamfer: 0.0,
        },
    );
    let sq = cronometra(
        "superquadrática",
        Primitive::Superquadric {
            half: [0.35, 0.30, 0.35],
            exponent_top: 4.0,
            exponent_side: 4.0,
        },
    );
    let sf = cronometra(
        "SUPERFÓRMULA",
        Primitive::Superformula {
            half: [0.35, 0.19, 0.35],
            top_symmetry: 5.0,
            top_n1: 0.6,
            top_n2: 1.7,
            top_n3: 1.7,
            side_symmetry: 4.0,
            side_n1: 2.0,
            side_n2: 2.0,
            side_n3: 2.0,
        },
    );
    println!(
        "\n  ⇒ a superfórmula custa {:.1}× a esfera, {:.1}× a caixa e {:.1}× a superquadrática",
        sf / esfera,
        sf / caixa,
        sf / sq
    );
}

/// ⛔⛔⛔ **A MINA: com um DESENHO na cena, a árvore é reconstruída por LADRILHO × FATIA.**
///
/// O caminho por ladrilho liga quando o documento tem um perfil (`RegionCompiler::is_worth_it`), e
/// aí o `compile_in_region_with` **percorre todos os nós** e reconstrói cada folha — a superfórmula
/// incluída — uma vez **por região**.
#[test]
#[ignore = "sonda: o preço da superfórmula ao lado de um desenho"]
fn the_price_of_the_superformula_next_to_a_drawing() {
    use ph2d_field::{FillRule, Profile};
    let contorno: Vec<[f32; 2]> = (0..64)
        .map(|i| {
            let a = std::f64::consts::TAU * f64::from(i) / 64.0;
            [(0.45 * a.cos()) as f32, (0.45 * a.sin()) as f32]
        })
        .collect();
    let perfil = Profile::new(vec![contorno], FillRule::NonZero, 1e-4).expect("perfil");
    let extrusao = Primitive::Extrude {
        profile: perfil,
        half_height: 0.25,
        round: 0.0,
        chamfer: 0.0,
    };
    let so_desenho = FieldDoc::new(
        vec![ph2d_field_eval::leaf(extrusao.clone(), Xform::IDENTITY)],
        NodeId(0),
    )
    .expect("doc");
    let com_esfera = FieldDoc::new(
        vec![
            ph2d_field_eval::leaf(extrusao.clone(), Xform::IDENTITY),
            ph2d_field_eval::leaf(
                Primitive::Sphere { radius: 0.2 },
                Xform {
                    translation: [0.7, 0.0, 0.0],
                    ..Xform::IDENTITY
                },
            ),
            ph2d_field::Node::new(
                Xform::IDENTITY,
                ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
            ),
        ],
        NodeId(2),
    )
    .expect("doc");
    let com_sf = FieldDoc::new(
        vec![
            ph2d_field_eval::leaf(extrusao, Xform::IDENTITY),
            ph2d_field_eval::leaf(
                Primitive::Superformula {
                    half: [0.2, 0.11, 0.2],
                    top_symmetry: 5.0,
                    top_n1: 0.6,
                    top_n2: 1.7,
                    top_n3: 1.7,
                    side_symmetry: 4.0,
                    side_n1: 2.0,
                    side_n2: 2.0,
                    side_n3: 2.0,
                },
                Xform {
                    translation: [0.7, 0.0, 0.0],
                    ..Xform::IDENTITY
                },
            ),
            ph2d_field::Node::new(
                Xform::IDENTITY,
                ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
            ),
        ],
        NodeId(2),
    )
    .expect("doc");

    println!("\n── um quadro a 640×360, com um DESENHO na cena (o caminho por ladrilho) ──");
    let reg = Registry::default();
    let cam = Orbit::default();
    for (nome, doc) in [
        ("só o desenho", &so_desenho),
        ("desenho + esfera", &com_esfera),
        ("desenho + SUPERFÓRMULA", &com_sf),
    ] {
        let _ = trace(doc, &reg, &cam, 320, 180);
        let antes = SPECIALISED.load(Ordering::Relaxed);
        let scans = ph2d_field_eval::ops_gielis::SCANS.load(Ordering::Relaxed);
        let t0 = std::time::Instant::now();
        let _ = trace(doc, &reg, &cam, 640, 360);
        println!(
            "  {nome:<24} {:>9.1} ms   ({} regiões, {} VARREDURAS do divisor)",
            t0.elapsed().as_secs_f64() * 1000.0,
            SPECIALISED.load(Ordering::Relaxed) - antes,
            ph2d_field_eval::ops_gielis::SCANS.load(Ordering::Relaxed) - scans
        );
    }
}

/// ⭐⭐⭐ **O GATE: a conta da forma corre uma vez por FORMA, e não por ladrilho.**
///
/// ⛔⛔ **Um defeito só de CUSTO é invisível a todo gate de imagem** — a W128 shipou com `3 852`
/// varreduras de uma dimensão por quadro (`642×` o necessário) e a imagem estava perfeita. O que o
/// apanha é um **contador**, e ele tem de correr no caminho que o produto toma: com um desenho na
/// cena, que é o que liga a especialização por ladrilho.
///
/// ⚠️ **Contador e não relógio:** esta workstation corre vários agentes, e nenhuma leitura de tempo
/// vale acima de `load ~5`. *Uma contagem é imune à carga.*
#[test]
fn the_shape_constants_are_computed_once_per_shape_not_once_per_tile() {
    use ph2d_field::{FillRule, Profile};
    let contorno: Vec<[f32; 2]> = (0..64)
        .map(|i| {
            let a = std::f64::consts::TAU * f64::from(i) / 64.0;
            [(0.45 * a.cos()) as f32, (0.45 * a.sin()) as f32]
        })
        .collect();
    let perfil = Profile::new(vec![contorno], FillRule::NonZero, 1e-4).expect("perfil");
    let doc = FieldDoc::new(
        vec![
            ph2d_field_eval::leaf(
                Primitive::Extrude {
                    profile: perfil,
                    half_height: 0.25,
                    round: 0.0,
                    chamfer: 0.0,
                },
                Xform::IDENTITY,
            ),
            ph2d_field_eval::leaf(
                Primitive::Superformula {
                    half: [0.2, 0.11, 0.2],
                    top_symmetry: 5.0,
                    top_n1: 0.6,
                    top_n2: 1.7,
                    top_n3: 1.7,
                    side_symmetry: 4.0,
                    side_n1: 2.0,
                    side_n2: 2.0,
                    side_n3: 2.0,
                },
                Xform {
                    translation: [0.7, 0.0, 0.0],
                    ..Xform::IDENTITY
                },
            ),
            ph2d_field::Node::new(
                Xform::IDENTITY,
                ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
            ),
        ],
        NodeId(2),
    )
    .expect("doc");
    let reg = Registry::default();
    let cam = Orbit::default();

    // ⚠️ **O quadro FRIO é o que o artista paga ao ARRASTAR um knob** — ali os parâmetros mudam a
    // cada quadro e o memo falha de propósito. O que ele NÃO pode pagar é uma conta por região.
    let antes = ph2d_field_eval::ops_gielis::SCANS.load(Ordering::Relaxed);
    let _ = trace(&doc, &reg, &cam, 640, 360);
    let regioes = SPECIALISED.load(Ordering::Relaxed);
    let frio = ph2d_field_eval::ops_gielis::SCANS.load(Ordering::Relaxed) - antes;

    // E o quadro MORNO: a mesma forma outra vez.
    let antes = ph2d_field_eval::ops_gielis::SCANS.load(Ordering::Relaxed);
    let _ = trace(&doc, &reg, &cam, 640, 360);
    let morno = ph2d_field_eval::ops_gielis::SCANS.load(Ordering::Relaxed) - antes;

    println!("  regiões especializadas: {regioes} · varreduras: frio {frio}, morno {morno}");
    assert!(
        regioes > 50,
        "a cena tinha de LIGAR a especialização por ladrilho (só {regioes} regiões) — sem ela este \
         gate não mede o caminho do produto"
    );
    // ⚠️ **A barra é por THREAD**: o memo é `thread_local`, e a marcha corre em rayon. `4` é a conta
    // de uma forma (dois `r_max` e dois máximos), e o tecto dá margem para o número de threads.
    assert!(
        frio <= 4 * 64,
        "quadro FRIO pagou {frio} varreduras com {regioes} regiões — a conta da forma está a correr \
         por LADRILHO em vez de por FORMA"
    );
    assert_eq!(
        morno, 0,
        "quadro MORNO tinha de pagar ZERO varreduras e pagou {morno}"
    );
}
