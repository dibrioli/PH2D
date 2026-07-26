//! **De que é feito um MOVE de Wet Paint** — a frente que o censo dos quatro meios abriu sem querer.
//!
//! O censo (`measure_the_four_media`) mediu o move dos quatro meios em duas telas, e três deles são
//! **planos** — 1,17→1,21 (Digital), 3,07→3,12 (Watercolor), 2,00→1,93 (Impasto). O Wet Paint é o
//! único que **sobe com a TELA**: 2,32 → 14,26 de 2048² para 4096², ou seja **6× para 4× a área**.
//!
//! ⚠️ **Isso é uma afirmação sobre a FORMA do trabalho, não sobre velocidade.** Um move é limitado
//! pela PEGADA: o pincel cobre o mesmo número de texels seja qual for o tamanho do documento, e é por
//! isso que os outros três não se mexem. Um custo que quadruplica com a área está varrendo um PLANO —
//! a mesma família do fold do impasto que esta jornada acabou de curar (201,5 → 14,55 ms).
//!
//! ## O que este arquivo mede, e por que assim
//!
//! Nada é re-implementado: cada linha dirige `on_canvas_pointer`, a porta de verdade — a lição que o
//! impasto pagou (*uma sonda que re-implementa o laço fica CEGA à porta*). E antes de cronometrar
//! qualquer coisa, a sonda pergunta a **FORMA**: quantos texels o move de fato marcou como sujos.
//!
//! Um número estrutural decide o caso sem depender do relógio: se a região marcada é do tamanho da
//! **PEGADA**, o plano está escondido em outro lugar (o despacho do dab, algum reconcile); se ela é do
//! tamanho da **TELA**, o composite está pintando a folha inteira a cada movimento do mouse, e a causa
//! é essa. Cronômetro sozinho diz *quanto*; a área diz *o quê*.

use super::*;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::Falloff;
use std::time::Instant;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um canvas armado em Wet Paint com o pincel grande e macio do censo — o mesmo pincel, para os
/// números destas tabelas serem comparáveis com os de lá.
fn wetted(size: u32, radius: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: radius,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_media(PaintMedia::WetPaint);
    t
}

/// Um traço reto: devolve `(move ms mediano, texels marcados medianos)` por move.
///
/// ⚠️ O passo é CONSTANTE (a mão do artista anda a mesma distância seja qual for o documento) — um
/// passo proporcional à tela reportaria *"o dobro dos dabs"* como *"o dobro do custo por move"*, que
/// é exatamente como a primeira versão da varredura de raio deste repo enganou a si mesma.
fn drag(size: u32, radius: f32) -> (f64, f64) {
    let mut t = wetted(size, radius);
    let mid = f64::from(size / 2) as f32;
    let x0 = radius + 20.0;
    const STEP_PX: f32 = 40.0;
    let x1 = x0 + STEP_PX * 20.0;
    assert!(x1 < (size as f32) - radius, "o traço tem de caber na tela");

    t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
    let _ = t.take_preview_arc();

    let mut moves = Vec::new();
    let mut areas = Vec::new();
    let mut x = x0 + STEP_PX;
    while x <= x1 {
        t.marks.clear();
        let t0 = Instant::now();
        t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
        moves.push(t0.elapsed().as_secs_f64() * 1e3);
        // A ÁREA que este move declarou suja — a forma do trabalho, perguntada ao produto.
        let a: u64 = t
            .marks
            .iter()
            .map(|r| u64::from(r.w) * u64::from(r.h))
            .sum();
        areas.push(a as f64);
        let _ = t.take_preview_arc();
        x += STEP_PX;
    }
    t.on_canvas_pointer(cp([x1, mid], PointerPhase::Up));

    moves.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    areas.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    (moves[moves.len() / 2], areas[areas.len() / 2])
}

/// **A forma do move**: custo e área marcada em três telas, com o pincel FIXO.
///
/// A coluna que decide é `área / pegada`: 1× significa que o move marcou exatamente o que o pincel
/// cobriu (limitado pela pegada, correto); crescer com a tela significa plano.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_what_a_wet_move_marks() {
    const RADIUS: f32 = 100.0;
    // A pegada de um move: o disco do dab varrido pelo passo — a ordem de grandeza que a área marcada
    // teria se o trabalho fosse limitado pela pegada.
    let footprint = f64::from(2.0 * RADIUS + 40.0) * f64::from(2.0 * RADIUS);

    println!(
        "\n{:<8} {:>10} {:>14} {:>12} {:>12}",
        "canvas", "move ms", "texels sujos", "vs pegada", "vs tela"
    );
    for size in [1024u32, 2048, 4096] {
        let (ms, area) = drag(size, RADIUS);
        let canvas = f64::from(size) * f64::from(size);
        println!(
            "{size:<8} {ms:>10.3} {area:>14.0} {:>11.2}x {:>11.2}%",
            area / footprint,
            100.0 * area / canvas,
        );
    }
    println!("\n(pegada de referência: {footprint:.0} texels)\n");
}

/// **Quantos donos tem o canvas quando o composite vai escrever nele** — e o que custa a cópia que
/// um segundo dono obriga.
///
/// `wetpaint_composite` termina em `Arc::make_mut(&mut self.canvas_rgba)`, que entrega o slice **se o
/// tool for dono único** e **CLONA A TELA INTEIRA** se não for. A pergunta é de uma linha e decide a
/// frente inteira: com dois donos, todo move paga uma cópia do documento.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_who_else_holds_the_canvas_during_a_wet_move() {
    const RADIUS: f32 = 100.0;
    println!(
        "\n{:<10} {:>8} {:>10} {:>16}",
        "meio", "canvas", "donos", "cópia de tela ms"
    );
    for size in [1024u32, 2048, 4096] {
        // Um traço de aquarela: o meio VIZINHO, que é plano na tela — o controle.
        let mut w = PainterTool::default();
        w.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        w.set_paint_media(PaintMedia::Watercolor);
        let mid = f64::from(size / 2) as f32;
        w.on_canvas_pointer(cp([RADIUS + 20.0, mid], PointerPhase::Down));
        let _ = w.take_preview_arc();
        w.on_canvas_pointer(cp([RADIUS + 60.0, mid], PointerPhase::Move));
        let wc_owners = Arc::strong_count(&w.canvas_rgba);

        // E o Wet Paint, na MESMA situação.
        let mut t = wetted(size, RADIUS);
        t.on_canvas_pointer(cp([RADIUS + 20.0, mid], PointerPhase::Down));
        let _ = t.take_preview_arc();
        t.on_canvas_pointer(cp([RADIUS + 60.0, mid], PointerPhase::Move));
        let _ = t.take_preview_arc();
        let owners = Arc::strong_count(&t.canvas_rgba);

        // O preço de UMA cópia de tela, medido pela mesma operação que o composite faz.
        let mut samples = Vec::new();
        for _ in 0..5 {
            let a = Arc::clone(&t.canvas_rgba);
            let mut b = Arc::clone(&t.canvas_rgba);
            let t0 = Instant::now();
            let slice = Arc::make_mut(&mut b);
            samples.push(t0.elapsed().as_secs_f64() * 1e3);
            std::hint::black_box(slice.as_ptr());
            drop(a);
        }
        samples.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        println!("{:<10} {size:>8} {wc_owners:>10} {:>16}", "Watercolor", "—");
        println!(
            "{:<10} {size:>8} {owners:>10} {:>16.3}",
            "Wet Paint", samples[0]
        );
    }
    println!();
}

/// **As duas curas candidatas, medidas em vez de escolhidas.**
///
/// O token do guard existe para responder *"alguém trocou o canvas debaixo de mim?"* — uma pergunta de
/// IDENTIDADE, que não precisa de POSSE. Duas formas de tirar a posse:
///
/// * **soltar** o handle antes da escrita e re-armá-lo depois (o composite já re-arma no fim);
/// * guardá-lo como **`Weak`** — que não conta como dono forte e, ainda por cima, **PRENDE a
///   alocação**, então o endereço não pode ser reciclado (o ABA que o ADR-0124 pagou no editor de
///   áudio, onde seis caches identificavam um buffer pelo ENDEREÇO).
///
/// A dúvida é o que o `Arc::make_mut` faz com um `Weak` vivo: a documentação diz que ele não CLONA o
/// valor, mas ele move para uma alocação nova — e mover um `Vec` é mover 24 bytes de cabeçalho, não o
/// buffer. Isso é afirmação sobre a `std`, então **é medido**, não citado.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_the_two_cures_for_the_identity_token() {
    println!(
        "\n{:<8} {:>14} {:>14} {:>14}",
        "canvas", "dono único ms", "com Weak ms", "com Arc ms"
    );
    for size in [1024u32, 2048, 4096] {
        let n = (size as usize) * (size as usize) * 4;
        let mut sole = Vec::new();
        let mut weak_held = Vec::new();
        let mut arc_held = Vec::new();
        for _ in 0..5 {
            // Dono único: o caminho que o produto DEVERIA estar tomando.
            let mut a = Arc::new(vec![7u8; n]);
            let t0 = Instant::now();
            let s = Arc::make_mut(&mut a);
            sole.push(t0.elapsed().as_secs_f64() * 1e3);
            std::hint::black_box(s.as_ptr());

            // Com um `Weak` vivo.
            let mut b = Arc::new(vec![7u8; n]);
            let w = Arc::downgrade(&b);
            let t1 = Instant::now();
            let s = Arc::make_mut(&mut b);
            weak_held.push(t1.elapsed().as_secs_f64() * 1e3);
            std::hint::black_box(s.as_ptr());
            drop(w);

            // Com um segundo `Arc` vivo: o produto de hoje.
            let mut c = Arc::new(vec![7u8; n]);
            let keep = Arc::clone(&c);
            let t2 = Instant::now();
            let s = Arc::make_mut(&mut c);
            arc_held.push(t2.elapsed().as_secs_f64() * 1e3);
            std::hint::black_box(s.as_ptr());
            drop(keep);
        }
        // O MÍNIMO: uma máquina carregada só sabe deixar mais lento.
        let lo = |v: &mut Vec<f64>| {
            v.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            v[0]
        };
        println!(
            "{size:<8} {:>14.4} {:>14.4} {:>14.4}",
            lo(&mut sole),
            lo(&mut weak_held),
            lo(&mut arc_held)
        );
    }
    println!();
}
