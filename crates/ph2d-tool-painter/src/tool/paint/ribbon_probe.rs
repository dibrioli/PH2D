//! **O QUE A FITA CUSTA POR EVENTO** — a medição que decide se os tetos do
//! `ph2d_painter_brush::line_kind::RIBBON_*` são de RECURSO ou de LOOK (plano 38 W6).
//!
//! ⚠️ **Pela porta do PRODUTO** (`on_canvas_pointer` + `on_tick`), nunca por um laço próprio: a
//! lição de que uma sonda com laço próprio fica CEGA à porta já custou duas waves a este módulo
//! (doc 28 §5.11, §5.46), e a de que *duas fixtures "grandes" diferentes dão números incomparáveis*
//! custou outra (§5.40).
//!
//! ⚠️ **E a fita é o PRIMEIRO tipo cujo custo NÃO se concentra no evento de movimento:** ela
//! percorre caminho no TIQUE, e a cauda dela cai toda no **pen-up**. Uma sonda que medisse só o
//! `Move` diria que a fita é de graça — e diria a verdade sobre o evento errado.
//!
//! Rodar: `cargo test -p ph2d-tool-painter --release measure_the_ribbon -- --ignored --nocapture --test-threads=1`

use super::measure_shape_system::{cp, tool};
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, Tool};
use ph2d_painter_brush::StrokeMethod;
use ph2d_painter_brush::line_kind::LineKind;
use std::time::Instant;

/// Um traço reto de `secs` a `speed` px/s, com o tique do produto entre os eventos.
/// Devolve `(pior Move ms, pior tique ms, pen-up ms)`.
fn straight(
    kind: LineKind,
    weight: f32,
    friction: f32,
    radius: f32,
    speed: f32,
) -> (f64, f64, f64) {
    let side = 2048u32;
    let dt = 1.0 / 60.0;
    let mut t = tool(side, PaintMedia::Digital, radius);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.line_kind = kind;
    t.paint.brush.ribbon_weight = weight;
    t.paint.brush.ribbon_friction = friction;
    let y = f32::from(u16::try_from(side / 2).unwrap_or(u16::MAX));
    let mut x = 200.0f32;
    t.on_canvas_pointer(cp([x, y], PointerPhase::Down));
    let (mut worst_move, mut worst_tick) = (0.0f64, 0.0f64);
    for _ in 0..120 {
        x += speed * dt;
        let a = Instant::now();
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        worst_move = worst_move.max(a.elapsed().as_secs_f64() * 1e3);
        let b = Instant::now();
        t.on_tick(dt * 1e3);
        worst_tick = worst_tick.max(b.elapsed().as_secs_f64() * 1e3);
    }
    let c = Instant::now();
    t.on_canvas_pointer(cp([x, y], PointerPhase::Up));
    (worst_move, worst_tick, c.elapsed().as_secs_f64() * 1e3)
}

/// **O ORÇAMENTO DA FITA** — o pior Move, o pior tique e o PEN-UP, contra o kill de 8 ms.
///
/// ⚠️ **A primeira linha é o CONTROLE** (`None`, a fita desarmada): sem ela a tabela não diz se um
/// número é da fita ou do pincel.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_ribbon_budget_per_event() {
    println!("[ribbon] traco reto de 2 s a 2400 px/s, 2048^2, pela porta do produto");
    println!(
        "{:>9} {:>7} {:>8} {:>6}  {:>10} {:>10} {:>10} {:>10}",
        "tipo", "weight", "friction", "raio", "pior move", "pior tick", "pen-up", "veredito"
    );
    // A corrida de aquecimento paga o *first-touch* dos planos: a lição do doc 28 §5.13.
    let _ = straight(LineKind::None, 0.0, 0.0, 24.0, 2400.0);
    for radius in [24.0f32, 100.0, 200.0] {
        for (kind, w, f) in [
            (LineKind::None, 0.0, 0.0),
            (LineKind::Ribbon, 0.25, 0.30),
            (LineKind::Ribbon, 0.45, 0.30),
            (LineKind::Ribbon, 1.0, 0.30),
            (LineKind::Ribbon, 1.0, 0.05),
            (LineKind::Ribbon, 1.0, 1.0),
        ] {
            let (mv, tk, up) = straight(kind, w, f, radius, 2400.0);
            let worst = mv.max(tk).max(up);
            println!(
                "{:>9} {w:>7.2} {f:>8.2} {radius:>6.0}  {mv:>10.3} {tk:>10.3} {up:>10.3} {:>10}",
                format!("{kind:?}"),
                if worst > 8.0 { "ESTOURA" } else { "sob o kill" }
            );
        }
    }
    println!("[ribbon] leitura: o teto e o maior peso cujo PIOR evento cabe nos 8 ms.");
}

/// **RENDERIZAR E OLHAR** — a fita desenhada por um gesto ONDULADO, pela porta do produto, salva em
/// PGM para inspeção visual.
///
/// ⚠️ **Cinco hipóteses caíram por medição antes desta sonda existir** (o `extend` emitindo no dedo;
/// o replay do taper; um salto entre dabs; o `settle` do quadro parado; a teia do Sketchy a vazar) —
/// todas testadas contra o MOTOR, e o motor está limpo por toda via que eu o dirija. Quando a
/// dedução falha cinco vezes, o que falta é **olhar o que o produto pinta**, que é o precedente do
/// `push_look_probe`.
///
/// `PH2D_RIBBON_LOOK_DIR=<dir> cargo test -p ph2d-tool-painter --release probe_ribbon_look -- --ignored --nocapture`
#[test]
#[ignore = "render-and-look, not a gate"]
fn probe_ribbon_look() {
    let Some(dir) = std::env::var_os("PH2D_RIBBON_LOOK_DIR") else {
        println!("[look] defina PH2D_RIBBON_LOOK_DIR=<dir>");
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&dir).expect("dir");
    let side = 1024u32;
    let dt = 1.0 / 60.0;
    for (nome, kind, w, fr, gr, ru) in [
        (
            "controle_none",
            LineKind::None,
            0.0f32,
            0.30f32,
            0.0f32,
            0.0f32,
        ),
        // O CONTROLE da faixa: a fita SEM travessas degenera na linha atrasada sozinha.
        ("banda_r000", LineKind::Ribbon, 0.08, 0.30, 0.0, 0.0),
        // A faixa, na largura que o atraso dá (~40 px a 0,08 — a tabela do `RIBBON_MAX_SUBSTEPS`).
        ("banda_r050", LineKind::Ribbon, 0.08, 0.30, 0.0, 0.5),
        ("banda_r100", LineKind::Ribbon, 0.08, 0.30, 0.0, 1.0),
        // Faixa LARGA — o atraso é a largura, então mais peso abre mais a faixa.
        ("banda_w020", LineKind::Ribbon, 0.20, 0.30, 0.0, 0.5),
        ("banda_w045", LineKind::Ribbon, 0.45, 0.30, 0.0, 0.5),
        // Os extremos que o roteiro manda mexer: o chicote (atrito no PISO) e o peso.
        ("ribbon_w100_fr000", LineKind::Ribbon, 1.0, 0.0, 0.0, 0.5),
        ("ribbon_w100_grav", LineKind::Ribbon, 1.0, 0.30, 0.5, 0.5),
    ] {
        let mut t = tool(side, PaintMedia::Digital, 6.0);
        t.paint.brush.stroke_method = StrokeMethod::Space;
        t.paint.brush.line_kind = kind;
        t.paint.brush.ribbon_weight = w;
        t.paint.brush.ribbon_friction = fr;
        t.paint.brush.ribbon_gravity = gr;
        t.paint.brush.ribbon_rungs = ru;
        // O gesto da foto: uma onda que desacelera nos picos (a mao vira devagar).
        let pt = |u: f32| {
            let x = 80.0 + u * 860.0;
            let y = 512.0 + (u * 18.0).sin() * 200.0;
            [x, y]
        };
        // ⚠️ **O predicado do DEPÓSITO entra no relatório**, e não é enfeite: a faixa nasceu com o
        // motor a costurar 343 travessas por traço e o tool MUDO (o `sews_threads` do spec enumerava
        // os tipos), e a imagem saía idêntica ao controle. Sem esta linha o próximo a olhar uma
        // imagem vazia procura o defeito no motor, onde ele não está.
        let deposita = t.threads_own_the_gesture();
        t.on_canvas_pointer(cp(pt(0.0), PointerPhase::Down));
        let mut u = 0.0f32;
        let mut frame = 0;
        while u < 1.0 {
            // varios eventos por quadro, com a velocidade a variar
            // ⚠️ **UM evento por quadro, com passo GROSSO** — um mouse de 125 Hz e uma mao rapida
            // andam 30-60 px por evento. A 1ª versao desta sonda andava 1,4 px, vinte vezes mais
            // fina que o produto, e nao continha o fenomeno.
            u = (u + 0.012 * (0.15 + (u * 18.0).cos().abs())).min(1.0);
            t.on_canvas_pointer(cp(pt(u), PointerPhase::Move));
            t.on_tick(dt * 1e3);
            frame += 1;
        }
        t.on_canvas_pointer(cp(pt(1.0), PointerPhase::Up));
        for _ in 0..60 {
            t.on_tick(dt * 1e3);
        }
        // PGM: a tinta e o alpha do canvas.
        let px = &t.canvas_rgba;
        let mut buf = format!("P5\n{side} {side}\n255\n").into_bytes();
        for i in 0..(side as usize * side as usize) {
            buf.push(px[i * 4]);
        }
        let p = dir.join(format!("{nome}.pgm"));
        std::fs::write(&p, buf).expect("write");
        // ⚠️ **A pergunta é ALTERNAÇÃO, não largura** — "uma faixa com travessas" e "uma linha
        // grossa" têm a mesma extensão vertical; o que as separa é o vão ENTRE as travessas. Uma
        // coluna que corta uma travessa fica cheia, a vizinha fica só com os dois trilhos ⇒ um
        // traço sólido tem `preenchimento` alto em TODA coluna, e a faixa tem o mínimo baixo.
        let (mut span_max, mut fill_min, mut fill_max) = (0usize, 1.0f32, 0.0f32);
        for x in (300..700).step_by(1) {
            let col: Vec<bool> = (0..side as usize)
                .map(|y| px[(y * side as usize + x) * 4] < 200)
                .collect();
            let (Some(a), Some(b)) = (col.iter().position(|&v| v), col.iter().rposition(|&v| v))
            else {
                continue;
            };
            let span = b - a + 1;
            if span < 4 {
                continue; // uma coluna que só pega a ponta não descreve a faixa
            }
            #[allow(clippy::cast_precision_loss)]
            let fill = col[a..=b].iter().filter(|&&v| v).count() as f32 / span as f32;
            span_max = span_max.max(span);
            fill_min = fill_min.min(fill);
            fill_max = fill_max.max(fill);
        }
        println!(
            "[look] {nome:18}: {frame:3} quadros · deposita fios? {deposita:5} · \
             faixa {span_max:3} px · preenchimento {fill_min:.2}..{fill_max:.2} -> {}",
            p.display()
        );
    }
}

/// **A ABLAÇÃO DA ESPÍCULA** — a MESMA imagem com e sem cada fase, e a DIFERENÇA é a tinta daquela
/// fase. É o oráculo honesto: um "trecho reto" medido sobre dabs não distingue a inflexão de uma
/// onda de uma espícula, e foi por isso que a sonda de fases apontou para o gesto inteiro.
///
/// `PH2D_RIBBON_LOOK_DIR=<dir> cargo test -p ph2d-tool-painter --release probe_ribbon_spike -- --ignored --nocapture`
#[test]
#[ignore = "render-and-look, not a gate"]
fn probe_ribbon_spike() {
    let Some(dir) = std::env::var_os("PH2D_RIBBON_LOOK_DIR") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&dir).expect("dir");
    let side = 1024u32;
    let dt = 1.0 / 60.0;
    // (nome, pausa no meio do gesto?, cauda no pen-up?)
    for (nome, pausa, cauda) in [
        ("spike_completo", true, true),
        ("spike_sem_pausa", false, true),
        ("spike_sem_cauda", true, false),
        ("spike_nu", false, false),
    ] {
        let mut t = tool(side, PaintMedia::Digital, 6.0);
        t.paint.brush.stroke_method = StrokeMethod::Space;
        t.paint.brush.line_kind = LineKind::Ribbon;
        let pt = |u: f32| [80.0 + u * 880.0, 512.0 + (u * 16.0).sin() * 200.0];
        // ⚠️ **VINTE E CINCO eventos, não cento e dez** — uma mão rápida a 60 Hz anda ~40 px por
        // evento, e a 1ª versão desta sonda andava 8. Com o gesto lento o atraso é pequeno, a faixa
        // quase não abre e a fixture **não continha o fenômeno**: a imagem saía limpa e teria
        // inocentado o produto.
        const EVENTOS: usize = 25;
        t.on_canvas_pointer(cp(pt(0.0), PointerPhase::Down));
        for i in 1..=EVENTOS {
            #[allow(clippy::cast_precision_loss)]
            let u = i as f32 / EVENTOS as f32;
            t.on_canvas_pointer(cp(pt(u), PointerPhase::Move));
            t.on_tick(dt * 1e3);
            // A mão PARA no meio, com o botão preso — o gesto que a foto do Enio sugere.
            if pausa && i == EVENTOS / 2 {
                for _ in 0..25 {
                    t.on_canvas_pointer(cp(pt(u), PointerPhase::Move));
                    t.on_tick(dt * 1e3);
                }
            }
        }
        t.on_canvas_pointer(cp(pt(1.0), PointerPhase::Up));
        if cauda {
            for _ in 0..60 {
                t.on_tick(dt * 1e3);
            }
        }
        let px = &t.canvas_rgba;
        let mut buf = format!("P5\n{side} {side}\n255\n").into_bytes();
        for i in 0..(side as usize * side as usize) {
            buf.push(px[i * 4]);
        }
        let p = dir.join(format!("{nome}.pgm"));
        std::fs::write(&p, buf).expect("write");
        println!(
            "[spike] {nome}: pausa={pausa} cauda={cauda} -> {}",
            p.display()
        );
    }
}
