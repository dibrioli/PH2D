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

/// **O QUE CUSTA UM FRAME COM A ÁGUA VIVA** — o repro do smoke do Enio (2026-07-28): *"IMG 4096, uma
/// pincelada grande e molhada, FPS cai para 4"*.
///
/// ⚠️ **As sondas irmãs medem o MOVE, e o move não é o que ele viu.** Um move só acontece enquanto a
/// mão anda; o que derruba o FPS *depois* da pincelada é o **tick** — a sim continua correndo, e ela
/// roda uma vez por frame quer o artista mexa o mouse ou não. Medir o move e concluir sobre o FPS é
/// medir a coisa errada com precisão.
///
/// 4 FPS são **250 ms/frame**. Esta tabela diz quanto disso é o tick, por tela e por raio.
#[test]
#[ignore = "medicao — rode com --release --ignored --nocapture"]
fn measure_what_a_frame_of_live_water_costs() {
    const FRAME_MS: f32 = 16.6;
    println!(
        "\n{:<8} {:>7} {:>12} {:>12} {:>12}",
        "tela", "raio", "traco ms", "tick p50 ms", "tick max ms"
    );
    for side in [1024u32, 2048, 4096] {
        for radius in [40.0f32, 100.0, 200.0] {
            let mut t = wetted(side, radius);
            let mid = (side / 2) as f32;
            let x0 = radius + 20.0;
            let x1 = ((side as f32) - radius - 20.0).min(x0 + 400.0);

            let t0 = Instant::now();
            t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
            let mut x = x0 + 40.0;
            while x <= x1 {
                t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
                let _ = t.take_preview_arc();
                x += 40.0;
            }
            t.on_canvas_pointer(cp([x1, mid], PointerPhase::Up));
            let stroke_ms = t0.elapsed().as_secs_f64() * 1e3;

            // A água viva: o que o app paga POR FRAME depois de soltar.
            let mut ticks = Vec::new();
            let mut areas = Vec::new();
            for _ in 0..12 {
                t.marks.clear();
                let t1 = Instant::now();
                ph2d_editor_core::tool::Tool::on_tick(&mut t, FRAME_MS);
                ticks.push(t1.elapsed().as_secs_f64() * 1e3);
                // ⚠️ A ÁREA que o tick declara suja é o que decide o UPLOAD, e o upload é do app, não
                // do tool: um tick barato que suja a tela inteira custa um documento por frame na
                // ponte. Medir só o relógio do tool responderia a pergunta errada.
                let a: u64 = t
                    .marks
                    .iter()
                    .map(|r| u64::from(r.w) * u64::from(r.h))
                    .sum();
                areas.push(a as f64);
                let _ = t.take_preview_arc();
            }
            ticks.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            areas.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            let full = f64::from(side) * f64::from(side);
            println!(
                "{side:<8} {radius:>7.0} {stroke_ms:>12.2} {:>12.2} {:>12.2} {:>11.1}%",
                ticks[ticks.len() / 2],
                ticks[ticks.len() - 1],
                100.0 * areas[areas.len() / 2] / full
            );
        }
    }
    println!();
}

/// **O CUSTO DO TICK EM FUNÇÃO DO `dt`** — a pergunta que o log do produto abriu (2026-07-28).
///
/// O shell chama `on_tick(frame_ms_now)`: o `dt` **é o wall clock do frame anterior**. Se o custo do
/// tick crescer com o `dt`, um frame lento pede um tick mais caro, que deixa o frame seguinte mais
/// lento — **realimentação positiva**, e é assim que 60 FPS viram 4 sem que nenhuma medida a `dt`
/// FIXO mostre problema. A sonda irmã mede a `dt = 16,6` e vê 3,5 ms; esta varre o `dt`.
///
/// ⚠️ O log do Enio traz `frame p50=35,1` com `dispatch p50=1,9` e `periodo real 57,1 ms` — o Painter
/// é 5% do frame, então o que esta tabela procura é se a água **realimenta** o resto.
#[test]
#[ignore = "medicao — rode com --release --ignored --nocapture"]
fn measure_whether_the_tick_feeds_back_on_a_slow_frame() {
    println!(
        "\n{:<8} {:>8} {:>12} {:>12}",
        "tela", "dt ms", "tick p50 ms", "tick max ms"
    );
    for side in [2048u32, 4096] {
        for dt in [16.6f32, 33.3, 57.1, 120.0, 250.0] {
            let mut t = wetted(side, 100.0);
            let mid = (side / 2) as f32;
            let x0 = 120.0;
            let x1 = ((side as f32) - 120.0).min(x0 + 400.0);
            t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
            let mut x = x0 + 40.0;
            while x <= x1 {
                t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
                let _ = t.take_preview_arc();
                x += 40.0;
            }
            t.on_canvas_pointer(cp([x1, mid], PointerPhase::Up));

            let mut ticks = Vec::new();
            for _ in 0..12 {
                let t1 = Instant::now();
                ph2d_editor_core::tool::Tool::on_tick(&mut t, dt);
                ticks.push(t1.elapsed().as_secs_f64() * 1e3);
                let _ = t.take_preview_arc();
            }
            ticks.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            println!(
                "{side:<8} {dt:>8.1} {:>12.2} {:>12.2}",
                ticks[ticks.len() / 2],
                ticks[ticks.len() - 1]
            );
        }
    }
    println!();
}

/// **O TICK DA ÁGUA NÃO REALIMENTA UM FRAME LENTO** — a propriedade, não o número.
///
/// O shell chama `on_tick(frame_ms_now)`, então o `dt` **é o wall clock do frame anterior**: se o custo
/// do tick crescer com o `dt`, um frame lento compra um tick caro que compra um frame mais lento. Foi
/// isso que levou 4096² a **4 FPS** no smoke de 2026-07-28, e nenhuma medida a `dt` FIXO podia vê-lo.
///
/// ⚠️ **O oráculo é uma RAZÃO** (`dt` de um frame travado ÷ `dt` de 60 Hz), e não um wall-clock: um
/// teto absoluto mede o perfil e a carga da máquina, enquanto a razão pergunta a coisa certa — *o custo
/// deste tick depende de quão lento foi o frame anterior?* Um laço fechado é ilimitado por natureza,
/// então qualquer teto o pega; a barra fica larga de propósito para não flakar sob suíte carregada.
///
/// ⚠️ **Mutação que sangra:** `WET_MAX_STEPS` de volta a 5 — a razão medida vai a **24×**.
#[test]
fn the_wet_tick_does_not_feed_back_on_a_slow_frame() {
    /// O `dt` de um frame de 60 Hz.
    const FAST_MS: f32 = 16.6;
    /// O `dt` de um frame travado a 4 FPS — o que o smoke reportou.
    const STALLED_MS: f32 = 250.0;
    /// Quantas vezes o tick pode ficar mais caro quando o frame anterior travou.
    /// Quantas vezes o tick pode ficar mais caro quando o frame anterior travou.
    ///
    /// ⚠️ **Medido dos DOIS lados, não escolhido:** com o cap do produto (2) a razão é **2,9×**; com o
    /// cap antigo (5) ela é **7,8×**. O teto fica no meio, com folga para os dois — um bar colado num
    /// dos lados vira flake sob suíte carregada, e a 1ª versão deste gate mediu 6,1 contra teto 6,0.
    ///
    /// ⚠️ E a razão **não é 1,0**, de propósito: o catch-up de um passo é intencional (sem ele a água
    /// nunca alcança o relógio num frame acima de 25 ms). A realimentação foi **contida**, não
    /// eliminada — e o gate afirma exatamente isso.
    const MAX_RATIO: f64 = 5.0;

    let cost = |dt: f32| -> f64 {
        // ⚠️ **4096², e a tela é PARTE do fixture.** A 1024² a razão mede 5,4× com o cap velho e o
        // gate fica VERDE sobre o defeito reportado — a realimentação só domina quando um passo de sim
        // custa o bastante para empurrar o frame seguinte, e é a 4096² que o Enio a viu. Uma fixture
        // menor aqui não é "um teste mais barato": é um teste de outra coisa.
        let mut t = wetted(4096, 100.0);
        t.on_canvas_pointer(cp([140.0, 2048.0], PointerPhase::Down));
        let mut x = 180.0f32;
        while x <= 560.0 {
            t.on_canvas_pointer(cp([x, 2048.0], PointerPhase::Move));
            let _ = t.take_preview_arc();
            x += 40.0;
        }
        t.on_canvas_pointer(cp([560.0, 2048.0], PointerPhase::Up));
        let mut ms = Vec::new();
        for _ in 0..9 {
            let t0 = Instant::now();
            ph2d_editor_core::tool::Tool::on_tick(&mut t, dt);
            ms.push(t0.elapsed().as_secs_f64() * 1e3);
            let _ = t.take_preview_arc();
        }
        ms.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        ms[ms.len() / 2]
    };

    let fast = cost(FAST_MS);
    let stalled = cost(STALLED_MS);
    // Controle: sem trabalho medível a razão não significa nada.
    assert!(
        fast > 0.02,
        "controle: o tick da agua nao custou nada ({fast:.4} ms) — a fixture nao tem agua viva"
    );
    let ratio = stalled / fast;
    assert!(
        ratio <= MAX_RATIO,
        "o tick REALIMENTA: {stalled:.2} ms depois de um frame de {STALLED_MS} ms contra \
         {fast:.2} ms a 60 Hz = {ratio:.1}x (teto {MAX_RATIO:.0}x)"
    );
}
