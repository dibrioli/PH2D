//! **O que um MOVE de SHAPE EDITOR custa** — o report do Enio de 2026-08-03: *"a pintura com strokes
//! vivos (Line, Freehand, Ellipse, Polygon, Curve) está extremamente lenta em todos os modos"*.
//!
//! A diferença estrutural contra o traço à mão livre está escrita no `stamp_preview.rs`: um método de
//! **re-stamp** desfaz a pegada anterior e **re-carimba a FIGURA INTEIRA** a cada evento, então o custo
//! de um move não é o de *um dab a mais* — é o de **todos os dabs da figura**, sempre. Um traço à mão
//! livre acumula (`Space`): cada move carimba só o pedaço novo.
//!
//! Esta sonda não afirma nada; ela IMPRIME. Rodar:
//! `cargo test -p ph2d-tool-painter --release the_shape_move -- --ignored --nocapture --test-threads=1`

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::StrokeMethod;

use super::media::PaintMedia;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn tool(side: u32, media: PaintMedia, radius: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (side * side * 4) as usize], side, side);
    t.set_paint_media(media);
    t.set_brush_size_px(radius * 2.0);
    t
}

fn ms(f: &mut dyn FnMut()) -> f64 {
    let t0 = std::time::Instant::now();
    f();
    t0.elapsed().as_secs_f64() * 1e3
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

/// **A tabela do report: quanto custa UM move de cada método, em cada meio.**
///
/// O oráculo é a comparação com o `Space` (mão livre) na MESMA tela e com o MESMO pincel — se o
/// re-stamp custa 20× o incremental, o número diz de onde vem a lentidão.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_shape_move_costs_this_much_in_every_medium() {
    println!("[shape] custo MEDIANO de um MOVE (ms) — pincel r=24, figura ~500 px");
    println!(
        "{:<12} {:>6}  {:>9} {:>9} {:>9} {:>9} {:>9}",
        "meio", "tela", "Space", "Line", "Ellipse", "Polygon", "Curve"
    );
    for media in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        for side in [2048u32, 4096] {
            let c = f64::from(side) / 2.0;
            let mut row = Vec::new();
            for method in [
                StrokeMethod::Space,
                StrokeMethod::Line,
                StrokeMethod::Ellipse,
                StrokeMethod::Polygon,
                StrokeMethod::Arc,
            ] {
                let mut t = tool(side, media, 24.0);
                t.paint.brush.stroke_method = method;
                #[allow(clippy::cast_possible_truncation)]
                let cx = c as f32;
                // ⚠️ O editor de **Line** é uma POLILINHA: o 1º Down cria UM ponto e o agarra, e um
                // ponto não desenha nada. Sem o 2º ponto a coluna mede zero e leria como *"Line é de
                // graça"* — a fixture não conteria o fenômeno. Os outros métodos abrem no 1º Down.
                if method == StrokeMethod::Line {
                    t.on_canvas_pointer(cp([cx - 60.0, cx], PointerPhase::Down));
                    t.on_canvas_pointer(cp([cx - 60.0, cx], PointerPhase::Up));
                }
                t.on_canvas_pointer(cp([cx, cx], PointerPhase::Down));
                // Oito moves que CRESCEM a figura até ~500 px de raio. O 1º é descartado (aloca).
                let mut samples = Vec::new();
                for k in 1..=8 {
                    #[allow(clippy::cast_precision_loss)]
                    let d = 60.0 + (k as f32) * 55.0;
                    let e = cp([cx + d, cx], PointerPhase::Move);
                    let dt = ms(&mut || {
                        t.on_canvas_pointer(e);
                    });
                    if k > 1 {
                        samples.push(dt);
                    }
                }
                t.on_canvas_pointer(cp([cx + 500.0, cx], PointerPhase::Up));
                row.push(median(&mut samples));
            }
            println!(
                "{:<12} {:>6}  {:>9.3} {:>9.3} {:>9.3} {:>9.3} {:>9.3}",
                format!("{media:?}"),
                side,
                row[0],
                row[1],
                row[2],
                row[3],
                row[4]
            );
        }
    }
}

/// **Como o custo cresce com o TAMANHO da figura** — a assinatura do re-stamp.
///
/// Um move é limitado pela PEGADA num método incremental (o pincel cobre os mesmos texels seja qual
/// for o comprimento já desenhado); num re-stamp ele é limitado pela FIGURA. Se a coluna cresce com o
/// raio, o custo é da figura inteira, e é isso que o report descreve.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_restamp_cost_grows_with_the_whole_figure() {
    println!("[shape] o custo de UM move contra o TAMANHO da figura (ms) — Digital, 2048, r=24");
    println!("{:>8}  {:>9} {:>9}", "raio px", "Ellipse", "Space");
    let side = 2048u32;
    for r in [50.0f32, 100.0, 200.0, 400.0, 800.0] {
        let mut out = Vec::new();
        for method in [StrokeMethod::Ellipse, StrokeMethod::Space] {
            let mut t = tool(side, PaintMedia::Digital, 24.0);
            t.paint.brush.stroke_method = method;
            let cx = 1024.0f32;
            t.on_canvas_pointer(cp([cx, cx], PointerPhase::Down));
            let mut samples = Vec::new();
            for k in 0..8 {
                // Oscila em torno do raio alvo: a figura fica do MESMO tamanho, e o que se mede é o
                // custo de re-carimbá-la, não o de crescê-la.
                #[allow(clippy::cast_precision_loss)]
                let d = r + if k % 2 == 0 { 2.0 } else { -2.0 };
                let e = cp([cx + d, cx], PointerPhase::Move);
                let dt = ms(&mut || {
                    t.on_canvas_pointer(e);
                });
                if k > 0 {
                    samples.push(dt);
                }
            }
            t.on_canvas_pointer(cp([cx + r, cx], PointerPhase::Up));
            out.push(median(&mut samples));
        }
        println!("{:>8.0}  {:>9.3} {:>9.3}", r, out[0], out[1]);
    }
}

/// **O MESMO comprimento de caminho, à mão livre contra re-stamp** — a comparação que decide se o
/// re-stamp é *ineficiente* ou apenas *repetido*.
///
/// Um traço à mão livre de `L` pixels carimba a mesma quantidade de tinta que uma figura de perímetro
/// `L`. Se os dois custam o mesmo por pixel de caminho, o re-stamp **não tem defeito** — ele só refaz
/// o trabalho inteiro a cada evento, e a cura é *não refazer*. Se o re-stamp custa muito mais por
/// pixel, há sobrecarga POR CHAMADA, e a cura é outra.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_same_path_costs_this_much_by_each_road() {
    println!("[shape] custo por PIXEL DE CAMINHO — Digital, 2048, pincel r=24");
    let side = 2048u32;
    let cx = 1024.0f32;

    // (a) MÃO LIVRE: um traço reto de 2000 px entregue em 50 eventos de 40 px.
    let mut t = tool(side, PaintMedia::Digital, 24.0);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.on_canvas_pointer(cp([24.0, cx], PointerPhase::Down));
    let free_ms = ms(&mut || {
        for k in 1..=50 {
            #[allow(clippy::cast_precision_loss)]
            let x = 24.0 + (k as f32) * 40.0;
            t.on_canvas_pointer(cp([x, cx], PointerPhase::Move));
        }
    });
    t.on_canvas_pointer(cp([2024.0, cx], PointerPhase::Up));

    // (b) RE-STAMP: UM move de uma elipse cujo perímetro é ~2000 px (raio 318).
    let mut t = tool(side, PaintMedia::Digital, 24.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    t.on_canvas_pointer(cp([cx, cx], PointerPhase::Down));
    let mut samples = Vec::new();
    for k in 0..8 {
        let d = 318.0 + if k % 2 == 0 { 2.0 } else { -2.0 };
        let e = cp([cx + d, cx], PointerPhase::Move);
        let dt = ms(&mut || {
            t.on_canvas_pointer(e);
        });
        if k > 0 {
            samples.push(dt);
        }
    }
    t.on_canvas_pointer(cp([cx + 318.0, cx], PointerPhase::Up));
    let shape_ms = median(&mut samples);

    let free_per_px = free_ms * 1e3 / 2000.0;
    let shape_per_px = shape_ms * 1e3 / 2000.0;
    println!(
        "  mão livre : traço de 2000 px em 50 eventos = {free_ms:8.3} ms  => {free_per_px:6.2} us/px"
    );
    println!(
        "  re-stamp  : UM move, perímetro 2000 px     = {shape_ms:8.3} ms  => {shape_per_px:6.2} us/px"
    );
    println!(
        "  razão por pixel de caminho: {:.2}x",
        shape_per_px / free_per_px.max(1e-9)
    );
}

/// **O Per-Layer Color hoje** — o handoff de 2026-07-04 fechou a frente de CPU em 7,9 ms/move com o
/// kernel em bandas, e o report do Enio de 2026-08-03 diz que ele *"também precisa de otimizações"*.
///
/// Duas rotas, e elas custam ordens diferentes: **cor por camada ESCOLHIDA** (assa um stamp premul por
/// camada e blita) contra **Texture Color** (o kernel dinâmico, que lê a imagem por dab). O número que
/// importa é o de HOJE, não o da nota.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_per_layer_colour_costs_this_much_today() {
    println!("[layers] custo MEDIANO de um MOVE (ms) — 2048, pincel r=48, camadas 64x64");
    println!(
        "{:<22} {:>7} {:>10} {:>10}",
        "rota", "camadas", "mão livre", "Ellipse"
    );
    for (name, pick) in [("cor escolhida", true), ("Texture Color", false)] {
        for n in [2usize, 8, 16] {
            let mut out = Vec::new();
            for method in [StrokeMethod::Space, StrokeMethod::Ellipse] {
                let mut t = tool(2048, PaintMedia::Digital, 48.0);
                t.paint.brush.stroke_method = method;
                let layers: Vec<(Vec<u8>, u32, u32)> =
                    (0..n).map(|_| (vec![200u8; 64 * 64 * 4], 64, 64)).collect();
                t.set_brush_shape_layers(layers);
                t.toggle_brush_shape_per_layer_color();
                if pick {
                    for i in 0..n {
                        #[allow(clippy::cast_precision_loss)]
                        let c = (i as f32) / (n as f32);
                        t.set_brush_shape_layer_color(i, [c, 1.0 - c, 0.5]);
                    }
                }
                let cx = 1024.0f32;
                t.on_canvas_pointer(cp([cx, cx], PointerPhase::Down));
                let mut samples = Vec::new();
                for k in 0..6 {
                    let d = if method == StrokeMethod::Ellipse {
                        200.0 + if k % 2 == 0 { 2.0 } else { -2.0 }
                    } else {
                        #[allow(clippy::cast_precision_loss)]
                        {
                            60.0 + (k as f32) * 40.0
                        }
                    };
                    let e = cp([cx + d, cx], PointerPhase::Move);
                    let dt = ms(&mut || {
                        t.on_canvas_pointer(e);
                    });
                    if k > 0 {
                        samples.push(dt);
                    }
                }
                t.on_canvas_pointer(cp([cx + 200.0, cx], PointerPhase::Up));
                out.push(median(&mut samples));
            }
            println!("{name:<22} {n:>7} {:>10.3} {:>10.3}", out[0], out[1]);
        }
    }
}

/// **De que um move de re-stamp é FEITO** — ablação pelas portas do produto.
///
/// O `stamp_drag_preview` faz cinco coisas por move: restaura a pegada anterior · zera o relevo do
/// traço · restaura o sculpt · SALVA a pegada nova · carimba a figura inteira. Esta sonda mede o
/// conjunto e depois o mesmo gesto com o relevo desligado, que é a única metade ablacionável pela
/// porta do artista.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn what_a_shape_move_is_made_of() {
    println!("[shape] ablação por ENTRADA — Ellipse r=400, pincel r=24");
    println!(
        "{:>6}  {:>10} {:>10} {:>10}",
        "tela", "Digital", "Impasto", "razão"
    );
    for side in [1024u32, 2048, 4096] {
        let mut out = Vec::new();
        for media in [PaintMedia::Digital, PaintMedia::Impasto] {
            let mut t = tool(side, media, 24.0);
            t.paint.brush.stroke_method = StrokeMethod::Ellipse;
            #[allow(clippy::cast_precision_loss)]
            let cx = (side / 2) as f32;
            t.on_canvas_pointer(cp([cx, cx], PointerPhase::Down));
            let mut samples = Vec::new();
            for k in 0..8 {
                let d = 400.0 + if k % 2 == 0 { 2.0 } else { -2.0 };
                let e = cp([cx + d, cx], PointerPhase::Move);
                let dt = ms(&mut || {
                    t.on_canvas_pointer(e);
                });
                if k > 0 {
                    samples.push(dt);
                }
            }
            t.on_canvas_pointer(cp([cx + 400.0, cx], PointerPhase::Up));
            out.push(median(&mut samples));
        }
        println!(
            "{:>6}  {:>10.3} {:>10.3} {:>9.2}x",
            side,
            out[0],
            out[1],
            out[1] / out[0].max(1e-9)
        );
    }
}
