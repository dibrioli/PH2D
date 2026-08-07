//! **O que o composite BOOLEANO cobra** — irmão do [`super::measure_shape_system`] por ASSUNTO.
//!
//! Lá mora *quanto a máquina de shape cobra por conta própria*; aqui, *quanto a Operation cobra*. São
//! duas perguntas com curas diferentes — a primeira é sobre o re-carimbo da figura, a segunda sobre um
//! composite que rasteriza, inunda e traça —, e o corte também tira o arquivo do teto de 700 LOC.
//!
//! A pesquisa que estas tabelas alimentam, com a rota do módulo Vector ao lado, está em
//! `docs/Painter/35_boolean_o_que_o_vector_ensina.md`.

use super::measure_shape_system::{cp, tool};
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::StrokeMethod;

/// **O BOOLEAN é barato ou caro?** — a pergunta do Enio de 2026-08-06.
///
/// A Operation de uma forma (Overlay / Add / Remove) não é um enfeite: escolher **Add** ou **Remove**
/// tira a figura do caminho de dabs e a manda pelo `stroke_boolean_contours`, que **rasteriza a
/// máscara num buffer supersampleado do CANVAS INTEIRO** (`SS = 3` ⇒ 12288² a 4096²), traça os
/// contornos e depois carimba o traçado.
///
/// ⚠️ **O eixo que decide é a TELA, não a figura.** Se a coluna crescer com o canvas enquanto a figura
/// fica do mesmo tamanho, o custo é do buffer e não do desenho — e aí o preço de marcar `Add` numa
/// forma não tem nada a ver com o que o artista pediu.
///
/// ⚠️ **UMA forma marcada já paga.** Com `active_is_bool` a figura ativa entra sozinha no composite
/// (o contorno dela passa a vir do traçado, não dos próprios dabs), então a coluna `1 Add` não é um
/// caso de canto: é o que acontece assim que alguém escolhe a Operation no painel.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_boolean_of_shapes() {
    println!(
        "[shape-sys] o custo de um MOVE com a Operation em ADD — Ellipse 400 px, Digital, raio 48"
    );
    println!(
        "{:>6}  {:>10} {:>10} {:>10} {:>10}  {:>9}",
        "tela", "Overlay", "1 Add", "2 Add", "4 Add", "Add/Ovl"
    );
    for side in [1024u32, 2048, 4096] {
        let mut row = Vec::new();
        for (op, extra) in [(0u8, 0usize), (1, 0), (1, 1), (1, 3)] {
            let mut t = tool(side, PaintMedia::Digital, 48.0);
            #[allow(clippy::cast_precision_loss)]
            let cx = (side / 2) as f32;
            // ⚠️ **A figura é FIXA em 200 px nas três telas.** A primeira versão desta fixture usava
            // `side * 0.1`, e aí a figura crescia junto com o canvas: as duas explicações possíveis
            // — *o custo é do buffer da tela* e *o custo é do desenho* — davam a MESMA coluna, e a
            // tabela não separava nada.
            let r = 200.0f32;
            t.set_stroke_op_mode(op);
            // As formas PARQUEADAS, todas com a mesma Operation.
            for k in 0..extra {
                #[allow(clippy::cast_precision_loss)]
                let dx = -r * 1.6 + (k as f32) * r * 0.8;
                t.paint.brush.stroke_method = StrokeMethod::Ellipse;
                t.on_canvas_pointer(cp([cx + dx, cx], PointerPhase::Down));
                t.on_canvas_pointer(cp([cx + dx + r, cx], PointerPhase::Move));
                t.on_canvas_pointer(cp([cx + dx + r, cx], PointerPhase::Up));
                t.park_active_shape();
            }
            t.paint.brush.stroke_method = StrokeMethod::Ellipse;
            t.on_canvas_pointer(cp([cx, cx], PointerPhase::Down));
            let mut s = Vec::new();
            for k in 0..5 {
                let d = if k % 2 == 0 { r + 2.0 } else { r - 2.0 };
                let e = cp([cx + d, cx], PointerPhase::Move);
                let t0 = std::time::Instant::now();
                t.on_canvas_pointer(e);
                let dt = t0.elapsed().as_secs_f64() * 1e3;
                if k > 0 {
                    s.push(dt);
                }
            }
            t.on_canvas_pointer(cp([cx + r, cx], PointerPhase::Up));
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            row.push(s[s.len() / 2]);
        }
        let ovl = row[0].max(1e-9);
        println!(
            "{side:>6}  {:>10.3} {:>10.3} {:>10.3} {:>10.3}  {:>8.1}x",
            row[0],
            row[1],
            row[2],
            row[3],
            row[1] / ovl
        );
    }
}

/// **O A/B da sub-janela por forma, na MESMA corrida** — a janela cheia contra a caixa de cada forma.
///
/// ⚠️ **Comparar duas CORRIDAS aqui atribuiria a deriva da máquina ao ganho**, e esta worktree divide
/// 32 núcleos com outras linhas: a §5.46 do doc 28 mediu o MESMO passo do produto indo de 14,5 a 30,2
/// ms sem uma linha de código mudar. As duas rotas são cronometradas **alternadas, dentro da corrida,
/// sobre o mesmo estado**, então a carga vira fator comum e a RAZÃO sobrevive.
///
/// ⚠️ A rota "cheia" **não é uma reimplementação**: é o `else` do mesmo laço, o que o composite fazia
/// antes desta sub-janela e o que ele ainda faz para uma forma sem caixa.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_sub_rect_against_the_full_window() {
    use super::stroke_boolean::diag;
    println!("[shape-sys] rasterizar: janela CHEIA x caixa da FORMA — a mesma corrida, 4096");
    println!(
        "{:>8}  {:>11} {:>11}  {:>8}",
        "formas", "cheia(ms)", "caixa(ms)", "razao"
    );
    for extra in [0usize, 1, 3] {
        let side = 4096u32;
        let mut t = tool(side, PaintMedia::Digital, 48.0);
        #[allow(clippy::cast_precision_loss)]
        let cx = (side / 2) as f32;
        let r = 200.0f32;
        t.set_stroke_op_mode(1);
        for k in 0..extra {
            #[allow(clippy::cast_precision_loss)]
            let dx = -r * 1.6 + (k as f32) * r * 0.8;
            t.paint.brush.stroke_method = StrokeMethod::Ellipse;
            t.on_canvas_pointer(cp([cx + dx, cx], PointerPhase::Down));
            t.on_canvas_pointer(cp([cx + dx + r, cx], PointerPhase::Move));
            t.on_canvas_pointer(cp([cx + dx + r, cx], PointerPhase::Up));
            t.park_active_shape();
        }
        t.paint.brush.stroke_method = StrokeMethod::Ellipse;
        t.on_canvas_pointer(cp([cx, cx], PointerPhase::Down));
        let (mut full, mut sub) = (Vec::new(), Vec::new());
        for k in 0..9 {
            let d = if k % 2 == 0 { r + 2.0 } else { r - 2.0 };
            for force in [true, false] {
                diag::set_force_full(force);
                let _ = diag::take();
                t.on_canvas_pointer(cp([cx + d, cx], PointerPhase::Move));
                let g = diag::take();
                let ms = g.raster_us as f64 / 1e3 / f64::from(g.calls.max(1));
                // A 1ª volta aquece o alocador do `region` — descartada, como no irmão acima.
                if k > 0 {
                    if force { &mut full } else { &mut sub }.push(ms);
                }
            }
        }
        diag::set_force_full(false);
        t.on_canvas_pointer(cp([cx + r, cx], PointerPhase::Up));
        let med = |v: &mut Vec<f64>| {
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            v[v.len() / 2]
        };
        let (a, b) = (med(&mut full), med(&mut sub));
        println!(
            "{:>8}  {a:>11.3} {b:>11.3}  {:>7.2}x",
            extra + 1,
            a / b.max(1e-9)
        );
    }
}

/// **DE QUE o composite booleano é feito** — as três fases, medidas no código que SHIPA.
///
/// A tabela acima diz *quanto* custa; esta diz *o quê*, e as três fases têm curas OPOSTAS:
/// **converter** é `O(segmentos)` (a geometria que o artista autorou), **rasterizar** e **traçar**
/// são `O(área da janela supersampleada)` — ou seja, custo que não tem nada a ver com a figura, só
/// com o retângulo que ela ocupa.
///
/// ⚠️ A coluna **`pts`** é a que decide a jusante: cada ponto do contorno traçado vira posição de
/// dab. Um traçado de raster a `SS = 3` devolve um ponto a cada terço de pixel de PERÍMETRO, e é
/// esse número que o `fill_polyline_preview` recebe.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_what_the_boolean_is_made_of() {
    println!("[shape-sys] as tres fases do composite booleano — Ellipse 200 px, Digital, 4096");
    println!(
        "{:>8}  {:>9} {:>9} {:>9}  {:>9}  {:>11} {:>8}",
        "formas", "converte", "rasteriza", "traca", "TOTAL", "celulas", "pts"
    );
    for extra in [0usize, 1, 3] {
        let side = 4096u32;
        let mut t = tool(side, PaintMedia::Digital, 48.0);
        #[allow(clippy::cast_precision_loss)]
        let cx = (side / 2) as f32;
        let r = 200.0f32;
        t.set_stroke_op_mode(1);
        for k in 0..extra {
            #[allow(clippy::cast_precision_loss)]
            let dx = -r * 1.6 + (k as f32) * r * 0.8;
            t.paint.brush.stroke_method = StrokeMethod::Ellipse;
            t.on_canvas_pointer(cp([cx + dx, cx], PointerPhase::Down));
            t.on_canvas_pointer(cp([cx + dx + r, cx], PointerPhase::Move));
            t.on_canvas_pointer(cp([cx + dx + r, cx], PointerPhase::Up));
            t.park_active_shape();
        }
        t.paint.brush.stroke_method = StrokeMethod::Ellipse;
        t.on_canvas_pointer(cp([cx, cx], PointerPhase::Down));
        let _ = super::stroke_boolean::diag::take();
        for k in 0..5 {
            let d = if k % 2 == 0 { r + 2.0 } else { r - 2.0 };
            t.on_canvas_pointer(cp([cx + d, cx], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([cx + r, cx], PointerPhase::Up));
        let g = super::stroke_boolean::diag::take();
        let n = f64::from(g.calls.max(1));
        let (c, ra, tr) = (
            g.convert_us as f64 / 1e3 / n,
            g.raster_us as f64 / 1e3 / n,
            g.trace_us as f64 / 1e3 / n,
        );
        #[allow(clippy::cast_precision_loss)]
        let cells = g.cells as f64 / n;
        #[allow(clippy::cast_precision_loss)]
        let pts = g.pts as f64 / n;
        println!(
            "{:>8}  {c:>9.3} {ra:>9.3} {tr:>9.3}  {:>9.3}  {cells:>11.0} {pts:>8.0}",
            extra + 1,
            c + ra + tr
        );
    }
}

/// **A CENA DO REPORT** — 4 círculos grandes com Operation Add, num 4096, pincel grosso.
///
/// ⚠️ O smoke de 2026-08-07 veio com *"mesmo o preview plano é extremamente custoso; 4 círculos com
/// boolean +, o fps cai para 2 ou menos"* — **500 ms/quadro contra os 4,38 que a tabela do rascunho
/// mediu**. Cem vezes fora ⇒ a fixture daquela tabela (UMA elipse de 400 px, pincel r=96) **não
/// descreve esta cena**, e a atribuição tem de sair de uma que descreva.
///
/// A tabela separa as três coisas que um move faz aqui, porque elas têm curas OPOSTAS:
/// **geometria** (o composite booleano: rasterizar 4 figuras, inundar, traçar) · **carimbo** (os dabs
/// ao longo do contorno traçado) · e o resto do evento. Se o peso for do carimbo, apagar a tinta
/// resolve; se for do composite, apagar a tinta **não resolve nada** — o contorno é traçado do mesmo
/// jeito, porque é ele que diz onde a figura está.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_scene_the_report_describes() {
    use super::stroke_boolean::diag as bdiag;
    println!("[shape-sys] a cena do report: 4 circulos Add, 4096, pincel GROSSO");
    println!(
        "{:>8} {:>7}  {:>9} {:>9} {:>9}  {:>9}  {:>9} {:>9} {:>9}  {:>8}",
        "formas",
        "raio",
        "MOVE",
        "geom",
        "carimbo",
        "SOLTAR",
        "converte",
        "rasteriza",
        "traca",
        "pts"
    );
    for (shapes, r_brush) in [(1usize, 120.0f32), (2, 120.0), (4, 120.0), (4, 40.0)] {
        let side = 4096u32;
        let mut t = tool(side, PaintMedia::Digital, r_brush);
        t.set_stroke_op_mode(1); // Add
        let r = 900.0f32;
        let spots = [
            [1400.0f32, 1400.0],
            [2700.0, 1400.0],
            [1400.0, 2700.0],
            [2700.0, 2700.0],
        ];
        // As figuras PARQUEADAS (todas menos a última), cada uma criada por um gesto real.
        for spot in spots.iter().take(shapes - 1) {
            t.paint.brush.stroke_method = StrokeMethod::Ellipse;
            t.on_canvas_pointer(cp(*spot, PointerPhase::Down));
            t.on_canvas_pointer(cp([spot[0] + r, spot[1]], PointerPhase::Move));
            t.on_canvas_pointer(cp([spot[0] + r, spot[1]], PointerPhase::Up));
            t.park_active_shape();
        }
        // A ATIVA, e o gesto que o artista faz: arrastá-la.
        let last = spots[shapes - 1];
        t.paint.brush.stroke_method = StrokeMethod::Ellipse;
        t.on_canvas_pointer(cp(last, PointerPhase::Down));
        let _ = super::stamp_banded::diag::take();
        let _ = bdiag::take();
        let mut samples = Vec::new();
        for k in 0..5 {
            let d = if k % 2 == 0 { r + 4.0 } else { r - 4.0 };
            let t0 = std::time::Instant::now();
            t.on_canvas_pointer(cp([last[0] + d, last[1]], PointerPhase::Move));
            let total = t0.elapsed().as_secs_f64() * 1e3;
            let ph = super::stamp_banded::diag::take();
            if k > 0 {
                samples.push((total, ph.stamp_us as f64 / 1e3));
            }
        }
        let t_up = std::time::Instant::now();
        t.on_canvas_pointer(cp([last[0] + r, last[1]], PointerPhase::Up));
        let up_ms = t_up.elapsed().as_secs_f64() * 1e3;
        let g = bdiag::take();
        let n = f64::from(g.calls.max(1));
        samples.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let (total, stamp) = samples[samples.len() / 2];
        #[allow(clippy::cast_precision_loss)]
        let pts = g.pts as f64 / n;
        println!(
            "{shapes:>8} {r_brush:>7.0}  {total:>9.3} {:>9.3} {stamp:>9.3}  {up_ms:>9.3}  {:>9.3} {:>9.3} {:>9.3}  {pts:>8.0}",
            (total - stamp).max(0.0),
            g.convert_us as f64 / 1e3 / n,
            g.raster_us as f64 / 1e3 / n,
            g.trace_us as f64 / 1e3 / n,
        );
    }
}
