//! **O que o PEN-DOWN custa** — as sondas do gesto, irmãs do [`super::measure_impasto_cost`]
//! (que mede o DAB) pelo teto de LOC e porque são outra pergunta.
//!
//! Aquele arquivo responde *"quanto custa carimbar"*; este responde *"quanto custa COMEÇAR"* — e a
//! diferença entre os dois é o que separou, nesta jornada, o custo específico do pen-down (o fork do
//! canvas + o setup dos planos) do custo de um dab comum, que todo move paga (doc 28 §4.5).

use super::measure_impasto_cost::{cp, ms};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};

/// **O que os 5,1 ms do RELEVO no pen-down SÃO** (doc 28 §4.4).
///
/// A pergunta que decide a próxima wave: aquele custo é **setup canvas-proporcional** (os cinco planos
/// do traço, que são do tamanho da TELA) ou é o **trabalho do primeiro dab** (que é do tamanho da
/// PEGADA)? Os dois são invisíveis um ao outro num número só.
///
/// A separação é o raio: varrendo o pincel de 10 a 100 sobre a MESMA tela, um custo canvas-proporcional
/// fica **plano** e um custo de pegada cresce com **r²** — 100× entre as pontas. É a mesma régua dos
/// gates de razão do módulo, e ela é imune à deriva da máquina porque compara duas medições da mesma
/// corrida.
#[test]
#[ignore]
fn is_the_relief_pen_down_the_planes_or_the_dab() {
    use ph2d_painter_brush::height::DrawTo;
    println!("[relief-pd] pen-down do RELEVO por raio — plano = os PLANOS, r² = o DAB");
    for size in [2048u32, 4096] {
        let mut row = Vec::new();
        for radius in [10.0f32, 25.0, 50.0, 100.0] {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
            t.toggle_brush_impasto();
            t.paint.brush.impasto_draw_to = DrawTo::Depth;
            for slot in &mut t.paint.brush_by_mode {
                slot.impasto_draw_to = DrawTo::Depth;
            }
            t.set_brush_size_px(radius * 2.0);
            let c = size as f32 * 0.5;
            // Aquece: o 1º traço da camada é o único que aloca; medimos os SEGUINTES, que é o regime.
            t.on_canvas_pointer(cp([c - 300.0, c], PointerPhase::Down));
            t.on_canvas_pointer(cp([c - 260.0, c], PointerPhase::Up));
            let mut best = f64::MAX;
            for k in 1..=4u8 {
                let y = c + 40.0 * f64::from(k) as f32;
                let v = ms(&mut || {
                    t.on_canvas_pointer(cp([c - 300.0, y], PointerPhase::Down));
                });
                t.on_canvas_pointer(cp([c - 260.0, y], PointerPhase::Up));
                best = best.min(v);
            }
            row.push((radius, best));
        }
        let txt = row
            .iter()
            .map(|(r, v)| format!("r{r:.0} {v:.2}"))
            .collect::<Vec<_>>()
            .join(" | ");
        let ratio = row[3].1 / row[0].1;
        println!("[relief-pd] {size}^2  {txt}  =>  r100/r10 = {ratio:.2}x (pegada previa 100x)");
    }
}

/// **O DELAY DO PRIMEIRO TRAÇO** (Enio, 2026-07-26: *"bem melhor e bastante aceitável com exceção do
/// delay do primeiro traço"*).
///
/// A decomposição do traço mede o custo POR MOVE, e por ela o 1º traço é o mais BARATO (110 contra 143 ms
/// sobre tinta) — ou seja o que o artista sente não está lá. O que ele sente é a **latência do
/// pen-down**: o intervalo entre clicar e o primeiro dab aparecer.
///
/// Esta sonda separa as três coisas que um pen-down pode pagar, e mede cada uma no tamanho em que ela
/// dói: o **1º** pen-down de uma camada virgem (que aloca o que for lazy), o **2º** (tudo já alocado) e o
/// **move** que vem depois. A diferença entre o 1º e o 2º é, por definição, o custo de *estrear*.
#[test]
#[ignore]
fn the_first_stroke_latency() {
    use ph2d_painter_brush::height::DrawTo;
    println!("[pendown] ms — 1o pen-down (camada virgem) | 2o pen-down | move seguinte");
    for size in [2048u32, 4096] {
        for (name, impasto, draw_to) in [
            ("impasto OFF (controle)", false, DrawTo::ColorAndDepth),
            ("impasto ON, so PIGMENTO", true, DrawTo::Color),
            ("impasto ON, so RELEVO", true, DrawTo::Depth),
            ("impasto ON (default)", true, DrawTo::ColorAndDepth),
        ] {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
            if impasto {
                t.toggle_brush_impasto();
                t.paint.brush.impasto_draw_to = draw_to;
                for slot in &mut t.paint.brush_by_mode {
                    slot.impasto_draw_to = draw_to;
                }
            }
            t.set_brush_size_px(200.0);
            let c = size as f32 * 0.5;
            // 1º pen-down: a camada nunca foi tocada.
            let first = ms(&mut || {
                t.on_canvas_pointer(cp([c - 300.0, c], PointerPhase::Down));
            });
            // ⚠️ O move TEM de andar mais que o espaçamento, senão nenhum dab nasce e a coluna mede
            // ZERO — que foi o que a 1ª versão desta sonda reportou, e um zero desses lê como
            // "o move é grátis" quando na verdade é "o move não aconteceu".
            let step = t.paint.brush.dab_spacing_px().max(4.0) * 2.0;
            let mut mid = f64::MAX;
            for k in 1..=4u8 {
                let x = c - 300.0 + step * f32::from(k);
                mid = mid.min(ms(&mut || {
                    t.on_canvas_pointer(cp([x, c], PointerPhase::Move));
                }));
            }
            let move_ms = mid;
            t.on_canvas_pointer(cp([c + 100.0, c], PointerPhase::Up));
            // Os pen-downs SEGUINTES: tudo o que era lazy já existe. Se o custo NÃO cair, ele é
            // por-gesto (uma cópia canvas-sized por traço) e não estreia de nada.
            let mut rest = Vec::new();
            for k in 1..=4u8 {
                let y = c + 40.0 * f32::from(k);
                rest.push(ms(&mut || {
                    t.on_canvas_pointer(cp([c - 300.0, y], PointerPhase::Down));
                }));
                t.on_canvas_pointer(cp([c - 260.0, y], PointerPhase::Up));
            }
            let tail = rest
                .iter()
                .map(|v| format!("{v:.2}"))
                .collect::<Vec<_>>()
                .join(" ");
            println!(
                "[pendown] {size}^2 {name:<24} 1o {first:>7.2} | seguintes {tail} | move {move_ms:.2}"
            );
        }
    }
}

/// **O DAB DE RELEVO custa 8× o de pigmento** (3,30 contra 0,39 ms a raio 100, 4096²) — e isto é o
/// custo de TODO move, não só do pen-down. Esta sonda pergunta de onde ele vem.
///
/// ⚠️ **A 1ª versão desta sonda era RUÍDO e eu acreditei nela.** Ela construía um `PainterTool` por
/// configuração — canvas novo de 64 MB, planos novos, páginas novas — e tirava a mediana de 8 moves.
/// O que a desmascarou foi uma mutação de medição que **não podia** tocar a linha de controle (*"só o
/// depósito"* tem o AA desligado, então o caminho mutado nem é chamado ali) e mesmo assim a viu saltar
/// de **4,26 para 5,99 ms**: ±40% de deriva entre corridas, com um efeito medido de 38% em cima.
///
/// A cura é **PAREAR**: um tool, uma tela, um traço, e as configurações **alternadas entre os moves**.
/// Cada amostra encontra as mesmas páginas, o mesmo estado de alocador e a mesma vizinhança do canvas,
/// então a diferença entre dois grupos é a configuração e nada mais. É o mesmo raciocínio dos gates de
/// RAZÃO do módulo: comparar duas medições da MESMA corrida é imune à deriva que separa duas corridas.
#[test]
#[ignore]
fn where_the_relief_dab_spends_its_time() {
    use ph2d_painter_brush::height::DrawTo;
    const SIZE: u32 = 4096;
    const ROUNDS: usize = 24;
    // (rótulo, smooth edges, push) — o SETTLE saiu: ele roda no commit (pen-up), não por dab, então
    // uma sonda de MOVE não pode vê-lo e mediu ruído nas duas versões.
    let cfgs = [
        ("tudo ligado (o default)", true, 1.0f32),
        ("sem o AA do filme", false, 1.0),
        ("sem o PUSH", true, 0.0),
        ("sem nenhum dos dois", false, 0.0),
    ];
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    t.toggle_brush_impasto();
    t.paint.brush.impasto_draw_to = DrawTo::Depth;
    for slot in &mut t.paint.brush_by_mode {
        slot.impasto_draw_to = DrawTo::Depth;
    }
    t.set_brush_size_px(200.0);
    let c = SIZE as f32 * 0.5;
    let step = t.paint.brush.dab_spacing_px().max(4.0) * 2.0;
    // ⚠️ SOMA por grupo, não mediana por-move: nem todo move produz um dab (o espaçamento decide), e a
    // mediana de uma amostra em que a maioria é `0,00` mede **quantos moves ficaram vazios**, não o
    // custo de um dab — foi o que a 2ª versão desta sonda reportou. Cada configuração percorre a MESMA
    // distância, então a soma do grupo carrega o mesmo número de dabs e é a comparação honesta.
    let mut totals = vec![0.0f64; cfgs.len()];
    let mut dabs = vec![0usize; cfgs.len()];
    // Um traço só, longo, com as configurações ALTERNADAS move a move.
    t.on_canvas_pointer(cp([c - 1500.0, c], PointerPhase::Down));
    let mut x = c - 1500.0;
    for round in 0..ROUNDS {
        for (k, (_, aa, push)) in cfgs.iter().enumerate() {
            t.paint.brush.impasto_smooth_edges = *aa;
            t.paint.brush.impasto_push = *push;
            x += step;
            let v = ms(&mut || {
                t.on_canvas_pointer(cp([x, c], PointerPhase::Move));
            });
            // A 1ª rodada aquece (a 1ª escrita em cada página do traço); as outras contam.
            if round > 0 {
                totals[k] += v;
                if v > 0.05 {
                    dabs[k] += 1;
                }
            }
        }
    }
    t.on_canvas_pointer(cp([x, c], PointerPhase::Up));
    println!("[relief-dab] raio 100, {SIZE}² — ms por MOVE, PAREADO (alternado no mesmo traço)");
    let mut per_dab = Vec::new();
    for (k, (name, _, _)) in cfgs.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "contagem pequena")]
        let d = dabs[k].max(1) as f64;
        per_dab.push(totals[k] / d);
        println!(
            "[relief-dab] {name:<28} total {:.1} ms em {} dabs => {:.2} ms/dab",
            totals[k],
            dabs[k],
            totals[k] / d
        );
    }
    // ⚠️ CONTROLE: os grupos têm de ter carregado o MESMO número de dabs. Se não tiverem, eles não
    // percorreram o mesmo trabalho e a diferença entre eles não é a configuração.
    assert!(
        dabs.iter().max().unwrap() - dabs.iter().min().unwrap() <= 1,
        "controle: os grupos carregaram numeros de dabs diferentes ({dabs:?}) — a comparacao nao vale"
    );
    println!(
        "[relief-dab] => o AA custa {:.2} ms ({:.0}% do dab) | o PUSH custa {:.2} ms",
        per_dab[0] - per_dab[1],
        (per_dab[0] - per_dab[1]) / per_dab[0] * 100.0,
        per_dab[0] - per_dab[2]
    );
}
