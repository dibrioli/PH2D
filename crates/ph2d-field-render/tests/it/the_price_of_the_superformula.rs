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
/// vale acima de `load ~5`. ⛔⛔ *Mas um contador atrás de um memo POR THREAD é imune ao RELÓGIO, não ao
/// ESCALONADOR* — ver as duas metades do gate, e porque a lei exacta corre numa pool de uma thread.
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

    // ⭐⭐ **A LEI, EXACTA — numa pool de UMA thread.** O memo é `thread_local` e a marcha corre em rayon:
    // com a pool fixa numa thread o quadro inteiro corre numa thread só, e a conta deixa de depender de
    // quem o escalonador acorda.
    // ⚠️ **O quadro FRIO é o que o artista paga ao ARRASTAR um knob** — ali os parâmetros mudam a
    // cada quadro e o memo falha de propósito. O que ele NÃO pode pagar é uma conta por região.
    let uma = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .expect("pool de uma thread");
    let (regioes, frio, morno) = uma.install(|| {
        let antes = ph2d_field_eval::ops_gielis::SCANS.load(Ordering::Relaxed);
        let _ = trace(&doc, &reg, &cam, 640, 360);
        let regioes = SPECIALISED.load(Ordering::Relaxed);
        let frio = ph2d_field_eval::ops_gielis::SCANS.load(Ordering::Relaxed) - antes;
        // E o quadro MORNO: a mesma forma outra vez.
        let antes = ph2d_field_eval::ops_gielis::SCANS.load(Ordering::Relaxed);
        let _ = trace(&doc, &reg, &cam, 640, 360);
        let morno = ph2d_field_eval::ops_gielis::SCANS.load(Ordering::Relaxed) - antes;
        (regioes, frio, morno)
    });

    println!(
        "  UMA thread: regiões especializadas {regioes} · varreduras: frio {frio}, morno {morno}"
    );
    assert!(
        regioes > 50,
        "a cena tinha de LIGAR a especialização por ladrilho (só {regioes} regiões) — sem ela este \
         gate não mede o caminho do produto"
    );
    // `4` é a conta de UMA forma (dois `r_max` e dois máximos), e aqui é EXACTA: a thread é uma.
    assert_eq!(
        frio, 4,
        "quadro FRIO pagou {frio} varreduras com {regioes} regiões numa thread só — a conta da forma \
         está a correr por LADRILHO em vez de por FORMA"
    );
    assert_eq!(
        morno, 0,
        "quadro MORNO pagou {morno} varreduras numa thread só — o memo não sobreviveu ao quadro"
    );

    // ⭐ **E na pool do PRODUTO (a global):** a lei é por THREAD — nenhuma thread paga a forma duas
    // vezes, logo os dois quadros juntos custam no máximo `4 × (threads da pool + a que chama)`.
    // ⛔⛔ **Aqui NÃO se afirma `morno == 0`** (medido na integração de 13/09): uma thread que o quadro
    // frio não chegou a usar paga a PRIMEIRA conta dela no morno, e quais são usadas decide-o o
    // escalonador. Sozinho, este gate leu `frio 132 = 33 × 4, morno 0` dez vezes em dez; com 24 cópias
    // concorrentes leu `morno 4` e `morno 8` em 3 a 4 de cada 24; e o `ship.sh` reprovou-o uma vez
    // com `frio 128, morno 4` sem uma linha de Rust mudada. *Uma contagem é imune ao RELÓGIO, não ao
    // ESCALONADOR* — é a família de flakes de fan-out do CLAUDE.md §5.0, com um contador no lugar do
    // relógio.
    let tecto = 4 * u64::try_from(rayon::current_num_threads() + 1)
        .expect("o número de threads cabe num u64");
    let antes = ph2d_field_eval::ops_gielis::SCANS.load(Ordering::Relaxed);
    let _ = trace(&doc, &reg, &cam, 640, 360);
    let _ = trace(&doc, &reg, &cam, 640, 360);
    let dois = ph2d_field_eval::ops_gielis::SCANS.load(Ordering::Relaxed) - antes;
    println!("  pool do produto: dois quadros, {dois} varreduras (tecto {tecto})");
    assert!(
        dois <= tecto,
        "dois quadros na pool do produto pagaram {dois} varreduras contra o tecto de {tecto} (4 por \
         thread) — alguma thread pagou a forma mais de uma vez"
    );
}

/// ⛔⛔ **A RESOLUÇÃO em que o dono corre** — o report de 06/09: *«não houve melhora
/// significativa»*, sobre uma cura que a régua mediu em `−42 %`.
///
/// ⚠️ **A hipótese nº 1 é que a régua mediu a cena errada:** as leituras da auditoria são
/// `640×360` num arnês; ele corre o **pill MODEL** numa janela cheia. *Uma sonda que mede outra
/// resolução mede outro programa.*
#[test]
#[ignore = "sonda: o quadro na resolução do dono"]
fn the_price_at_the_resolution_the_owner_runs() {
    let reg = Registry::default();
    let cam = Orbit::default();
    let formas: [(&str, Primitive); 3] = [
        ("esfera", Primitive::Sphere { radius: 0.35 }),
        (
            "superquadrática",
            Primitive::Superquadric {
                half: [0.35, 0.30, 0.35],
                exponent_top: 4.0,
                exponent_side: 4.0,
            },
        ),
        (
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
        ),
    ];
    println!("\n── o QUADRO por resolução (orçamento de 60 fps = 16,7 ms) ──");
    println!(
        "  {:<18} {:>10} {:>10} {:>10} {:>10}",
        "", "640×360", "1280×720", "1920×1080", "2560×1440"
    );
    for (nome, p) in formas {
        let doc = doc_de(p);
        let mut linha = format!("  {nome:<18}");
        for (w, h) in [(640_u32, 360_u32), (1280, 720), (1920, 1080), (2560, 1440)] {
            let _ = trace(&doc, &reg, &cam, w / 2, h / 2);
            let mut melhor = f64::INFINITY;
            for _ in 0..3 {
                let t0 = std::time::Instant::now();
                let _ = trace(&doc, &reg, &cam, w, h);
                melhor = melhor.min(t0.elapsed().as_secs_f64() * 1000.0);
            }
            linha.push_str(&format!(" {melhor:>9.1}"));
        }
        println!("{linha}");
    }
    println!(
        "\n  ⚠️ o melhor de 3 por célula, e o `load` ao lado importa: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}
