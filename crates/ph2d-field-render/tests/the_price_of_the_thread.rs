//! ⭐ **SONDA da W135** — quanto custa uma ROSCA, medido no **QUADRO**.
//!
//! ⚠️ **A régua tem de ser o quadro, e não uma varredura de campo** (doc 06 §130.1): a árvore é
//! especializada por ladrilho × fatia, e uma medição por amostra responde a outra pergunta.
//!
//! ⭐⭐ **E aqui o mecanismo do preço é DIFERENTE do do nó**: `starts` **não** acrescenta um ramo à
//! árvore — ele só engorda `b`, o avanço por radiano. O que ele encarece é o **divisor** (`1/k`
//! cresce com `b`), logo o passo da marcha encurta. *Duas causas diferentes, e por isso duas
//! sondas.* É daqui que sai o [`ph2d_field::MAX_THREAD_STARTS`].

use ph2d_field::{FieldDoc, NodeId, Primitive, Xform, thread_depth_ceiling};
use ph2d_field_eval::hybrid::Registry;
use ph2d_field_render::{Orbit, trace};

fn cronometra(nome: &str, p: Primitive) -> f64 {
    let doc =
        FieldDoc::new(vec![ph2d_field_eval::leaf(p, Xform::IDENTITY)], NodeId(0)).expect("a peça");
    let reg = Registry::default();
    let cam = Orbit::default();
    let _ = trace(&doc, &reg, &cam, 320, 180);
    let t0 = std::time::Instant::now();
    let _ = trace(&doc, &reg, &cam, 640, 360);
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    println!("  {nome:<30} {ms:>8.1} ms");
    ms
}

fn rosca(pitch: f32, flank: f32, starts: u32, hands: u32, fraccao: f32) -> Primitive {
    let radius = 0.5;
    Primitive::Thread {
        radius,
        half_height: 0.45,
        pitch,
        depth: thread_depth_ceiling(radius, pitch, flank) * fraccao,
        flank,
        starts,
        hands,
        round: 0.0,
        chamfer: 0.0,
    }
}

#[test]
#[ignore = "sonda: o preço de um quadro com a rosca"]
fn the_price_of_the_thread() {
    println!("\n(load ao lado de cada leitura — CLAUDE.md §5.0)");
    println!(
        "  /proc/loadavg = {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("\n── um quadro a 640×360, uma peça só ──");
    let esfera = cronometra("esfera", Primitive::Sphere { radius: 0.5 });
    let cilindro = cronometra(
        "cilindro",
        Primitive::Cylinder {
            radius: 0.5,
            half_height: 0.45,
            round: 0.0,
            chamfer: 0.0,
        },
    );
    println!("\n── e a rosca, entrada a entrada (passo 0,16 · flanco 30° · uma mão) ──");
    let mut linhas = Vec::new();
    for s in [1_u32, 8, 32, 64, 96, 128, 192, 256, 384] {
        linhas.push((
            s,
            cronometra(
                &format!("rosca (starts = {s})"),
                rosca(0.16, 30.0, s, 1, 0.75),
            ),
        ));
    }
    println!("\n── e a SEGUNDA MÃO, que É um ramo a mais na árvore ──");
    for s in [1_u32, 6, 12] {
        cronometra(
            &format!("serrilhado (starts = {s}, 2 mãos)"),
            rosca(0.16, 45.0, s, 2, 0.70),
        );
    }
    println!("\n── e o PASSO, que é o que faz o filete ser fino ──");
    for pitch in [0.32_f32, 0.16, 0.08, 0.04, 0.02] {
        cronometra(
            &format!("rosca (pitch = {pitch:.2})"),
            rosca(pitch, 30.0, 1, 1, 0.75),
        );
    }
    // ⭐⭐ **O NÚCLEO** — o divisor mora nele (`1/k = √(1 + (b/núcleo)²·cos²α)`), então é aqui que se
    // vê se o [`ph2d_field::THREAD_CORE_FLOOR`] tem recurso ou é palpite. ⚠️ A profundidade é
    // pedida em fracção do RAIO, e não do tecto, para o núcleo ser a variável.
    println!("\n── e o NÚCLEO, que é onde o divisor vive (starts = 8) ──");
    for fundo in [0.60_f32, 0.45, 0.35, 0.25, 0.15, 0.08, 0.04] {
        let radius = 0.5_f32;
        // o passo cresce com a profundidade pedida, para o tecto do período não a cortar
        let depth = radius * (1.0 - fundo);
        let pitch = depth * 2.0 * (30.0_f32).to_radians().tan() * 1.02;
        cronometra(
            &format!("núcleo = {fundo:.2}·R (depth {depth:.3})"),
            Primitive::Thread {
                radius,
                half_height: 0.45,
                pitch,
                depth,
                flank: 30.0,
                starts: 8,
                hands: 1,
                round: 0.0,
                chamfer: 0.0,
            },
        );
    }
    println!("\n  esfera {esfera:.1} ms · cilindro {cilindro:.1} ms");
    println!("  ⇒ a rosca custa, por entrada:");
    for (s, ms) in linhas {
        println!(
            "     starts = {s:>2}   {:.2}× a esfera   {:.2}× o cilindro",
            ms / esfera,
            ms / cilindro
        );
    }
    println!(
        "\n  /proc/loadavg no fim = {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}
