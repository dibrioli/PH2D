//! **O que o `Style: Solid` cobra por evento de ponteiro** — e o que a Symmetry e o Tiling fazem
//! com esse número (auditoria do Enio, 2026-08-15: *"atenção especial para performance em Symmetry
//! Circular + Tiling"*).
//!
//! ⚠️ **A FORMA responde antes do RELÓGIO, e é de propósito.** Um preenchimento de gesto vivo é um
//! **re-carimbo**: a cada ponto novo o polígono INTEIRO muda, então o produto refaz a mancha do
//! zero. As três grandezas que decidem o custo são contáveis e **não dependem da carga da máquina**:
//!
//! 1. **quantos pontos** o conjunto de laços tem (o caminho × as cópias de simetria × as tiles);
//! 2. **que área** o retângulo do preenchimento cobre (o `fill_coverage` é `O(área)` e a transação
//!    salva e restaura essa área a cada evento);
//! 3. **como as duas crescem ao longo do traço** — porque um custo por-evento que cresce com o
//!    caminho é um traço `O(n²)`, e isso não aparece num evento isolado.
//!
//! Rodar: `cargo test -p ph2d-tool-painter --release measure_what_a_solid_move -- --ignored
//! --nocapture --test-threads=1`

use super::measure_shape_system::{cp, tool};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::Tool;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

/// O que UM evento de ponteiro deixou para trás, em grandezas que não têm relógio dentro.
#[derive(Clone, Copy, Default)]
pub(super) struct EventShape {
    /// Pontos do caminho de tinta acumulado (a raia primária).
    pub(super) path: usize,
    /// Laços que o preenchimento deste evento percorre (caminho × simetria × tiling).
    pub(super) loops: usize,
    /// Soma dos pontos de todos os laços — o que o `fill_coverage` percorre em ARESTAS.
    pub(super) points: usize,
    /// Área do retângulo que a transação salva, preenche e restaura.
    pub(super) rect_px: usize,
    /// Milissegundos do evento inteiro (`on_canvas_pointer`).
    pub(super) ms: f64,
}

/// Mede a forma do preenchimento SEM o carimbar — a mesma pergunta que o produto faz ao entrar em
/// [`PainterTool::stamp_solid_preview`], feita de fora para poder ser contada.
fn shape_now(t: &PainterTool) -> (usize, usize, usize, usize) {
    let loops = t.solid_fill_loops();
    let points: usize = loops.iter().map(Vec::len).sum();
    let rect = t
        .solid_fill_rect(&loops)
        .map_or(0, |r| (r.w as usize) * (r.h as usize));
    (t.paint.solid_path.len(), loops.len(), points, rect)
}

/// Um laço circular de `steps` eventos, em Solid, com a configuração já armada. Devolve a forma
/// depois de CADA evento.
///
/// ⚠️ **O laço é DESCENTRADO do eixo da simetria, e a fixture nasceu sem isso.** O centro radial
/// default é o centro do canvas; um círculo desenhado À VOLTA dele é invariante sob rotação, então
/// as doze cópias caem umas sobre as outras e a rosácea **não abre** — a tabela media 12 laços e
/// um retângulo do tamanho de um. Fora do eixo, as cópias orbitam e a caixa cresce, que é o que o
/// artista de facto desenha.
pub(super) fn solid_arc(
    t: &mut PainterTool,
    orbit: [f32; 2],
    radius: f32,
    steps: usize,
) -> Vec<EventShape> {
    let mut out = Vec::with_capacity(steps);
    // Um círculo, porque é o gesto onde a rosácea da simetria circular abre depressa.
    let at = |k: usize| {
        #[allow(clippy::cast_precision_loss)]
        let a = k as f32 * 0.11;
        // HR-5 não se aplica a uma sonda, mas manter o mesmo caminho para todas as configurações
        // importa: as tabelas têm de ser comparáveis entre si.
        [orbit[0] + radius * a.cos(), orbit[1] + radius * a.sin()]
    };
    t.on_canvas_pointer(cp(at(0), PointerPhase::Down));
    for k in 1..=steps {
        let t0 = std::time::Instant::now();
        t.on_canvas_pointer(cp(at(k), PointerPhase::Move));
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        let (path, loops, points, rect_px) = shape_now(t);
        out.push(EventShape {
            path,
            loops,
            points,
            rect_px,
            ms,
        });
    }
    t.on_canvas_pointer(cp(at(steps), PointerPhase::Up));
    out
}

/// As configurações da tabela: o nome e o que armar no tool.
pub(super) fn arm(t: &mut PainterTool, sym: &str, tiling: bool) {
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    match sym {
        "mirror" => t.toggle_symmetry_enabled(),
        "circ12" => {
            t.toggle_symmetry_enabled();
            t.toggle_symmetry_circular();
            t.set_symmetry_segments(12);
        }
        _ => {}
    }
    if tiling {
        t.toggle_brush_tiling(0);
        t.toggle_brush_tiling(1);
    }
}

/// **A TABELA** — o que um move de Solid cobra em cada combinação, e como isso cresce.
///
/// ⚠️ **O laço orbita PERTO DA BORDA**, e é o que torna a coluna do Tiling não-vazia: um laço
/// interior não cruza costura nenhuma, então `tiled_loops` devolve a entrada verbatim e a tabela
/// mediria *"Tiling é de graça"* sobre uma fixture que não o exercita.
#[test]
#[ignore = "medição, não gate — rode com --test-threads=1 na máquina calma"]
fn measure_what_a_solid_move_costs_under_symmetry_and_tiling() {
    let side = 1024u32;
    let steps = 96usize;
    println!(
        "\n=== O QUE UM MOVE DE SOLID COBRA (canvas {side}x{side}, {steps} eventos, r=6) ===\n\
         \x20   pontos = arestas que o fill percorre  ·  rect = area que a transacao salva+preenche+restaura\n"
    );
    println!(
        "{:<22} {:>7} {:>7} {:>9} {:>11} {:>13} {:>9} {:>9}",
        "config", "path", "loops", "pontos", "rect px", "SOMA rect px", "ms p50", "ms max"
    );
    for tiling in [false, true] {
        for sym in ["off", "mirror", "circ12"] {
            let mut t = tool(side, PaintMedia::Digital, 6.0);
            arm(&mut t, sym, tiling);
            // Orbita fora do eixo radial (a rosácea abre) E com a caixa a PASSAR da borda direita
            // (`960 + 140 > 1024`), que é a condição que o `tiled_loops` testa.
            let ev = solid_arc(&mut t, [960.0, 512.0], 140.0, steps);
            let last = *ev.last().unwrap();
            let sum_rect: usize = ev.iter().map(|e| e.rect_px).sum();
            let mut ms: Vec<f64> = ev.iter().map(|e| e.ms).collect();
            ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let p50 = ms[ms.len() / 2];
            let max = *ms.last().unwrap();
            let name = format!("sym={sym} tiling={}", if tiling { "on" } else { "off" });
            println!(
                "{name:<22} {:>7} {:>7} {:>9} {:>11} {:>13} {p50:>9.3} {max:>9.3}",
                last.path, last.loops, last.points, last.rect_px, sum_rect
            );
        }
    }

    // A curva do custo AO LONGO do traço: é ela que separa "caro" de "quadrático".
    println!("\n--- como cresce ao longo do traço (sym=circ12, tiling=on) ---");
    println!(
        "{:>6} {:>7} {:>9} {:>11} {:>9}",
        "evento", "path", "pontos", "rect px", "ms"
    );
    let mut t = tool(side, PaintMedia::Digital, 6.0);
    arm(&mut t, "circ12", true);
    let ev = solid_arc(&mut t, [960.0, 512.0], 140.0, steps);
    for (i, e) in ev.iter().enumerate() {
        if i % 12 == 0 || i + 1 == ev.len() {
            println!(
                "{:>6} {:>7} {:>9} {:>11} {:>9.3}",
                i + 1,
                e.path,
                e.points,
                e.rect_px,
                e.ms
            );
        }
    }
}

/// **DE QUE É FEITO um move de Solid** — as peças medidas pela porta do produto, não por um laço
/// próprio.
///
/// ⚠️ **A decomposição corta o EVENTO em três, e a terceira é o resto por subtração** — o depósito
/// dos dabs, a corda e o que mais o `on_canvas_pointer` faz. Um laço próprio que re-implementasse o
/// preenchimento mediria a minha aritmética, não a do produto.
#[test]
#[ignore = "medição, não gate — rode com --test-threads=1 na máquina calma"]
fn measure_what_a_solid_move_is_made_of() {
    let side = 1024u32;
    let steps = 96usize;
    println!(
        "\n=== DE QUE É FEITO UM MOVE DE SOLID (canvas {side}x{side}, no evento {steps}) ===\n"
    );
    println!(
        "{:<20} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>10}",
        "config", "pontos", "loops", "cov", "write", "save", "restore", "TRANSACAO"
    );
    for sym in ["off", "mirror", "circ12"] {
        let mut t = tool(side, PaintMedia::Digital, 6.0);
        arm(&mut t, sym, true);
        let ev = solid_arc(&mut t, [960.0, 512.0], 140.0, steps);
        let evt_ms = ev.last().unwrap().ms;

        // As peças, no estado em que o traço as deixou.
        let n = 12;
        let t0 = std::time::Instant::now();
        let mut loops = Vec::new();
        for _ in 0..n {
            loops = t.solid_fill_loops();
        }
        let loops_ms = t0.elapsed().as_secs_f64() * 1e3 / f64::from(n);
        let rect = t.solid_fill_rect(&loops).expect("a mancha tem de existir");
        #[allow(clippy::cast_precision_loss)]
        let origin = [rect.x as f32, rect.y as f32];
        let t1 = std::time::Instant::now();
        for _ in 0..n {
            let cov = ph2d_painter_brush::solid::fill_coverage(
                &loops,
                rect.w as usize,
                rect.h as usize,
                origin,
            );
            std::hint::black_box(&cov);
        }
        let fill_ms = t1.elapsed().as_secs_f64() * 1e3 / f64::from(n);
        let t2 = std::time::Instant::now();
        for _ in 0..n {
            let px = t.save_region(&rect);
            std::hint::black_box(&px);
        }
        let save_ms = t2.elapsed().as_secs_f64() * 1e3 / f64::from(n);
        let px = t.save_region(&rect);
        let t4 = std::time::Instant::now();
        for _ in 0..n {
            t.restore_region(&rect, &px);
        }
        let restore_ms = t4.elapsed().as_secs_f64() * 1e3 / f64::from(n);
        // `stamp_solid` = a cobertura MAIS a escrita do `over` sobre o retângulo; a escrita sai por
        // subtração, que é honesto porque as duas correm na mesma porta e sobre a mesma área.
        let t5 = std::time::Instant::now();
        for _ in 0..n {
            t.stamp_solid(&loops, rect);
        }
        let solid_ms = t5.elapsed().as_secs_f64() * 1e3 / f64::from(n);
        let write_ms = (solid_ms - fill_ms).max(0.0);
        // A TRANSAÇÃO inteira: descascar → medir → salvar → preencher → carimbar a corda. Repetida,
        // ela refaz exactamente o mesmo trabalho (o restore devolve o estado que ela salvou).
        let t3 = std::time::Instant::now();
        for _ in 0..n {
            t.stamp_solid_preview();
        }
        let txn_ms = t3.elapsed().as_secs_f64() * 1e3 / f64::from(n);

        let points: usize = loops.iter().map(Vec::len).sum();
        println!(
            "{:<20} {points:>8} {loops_ms:>8.3} {fill_ms:>8.3} {write_ms:>8.3} {save_ms:>8.3} {restore_ms:>8.3} {txn_ms:>10.3}",
            format!("sym={sym} til=on")
        );
        let _ = evt_ms;

        // **ARESTAS ou ÁREA?** — o mesmo retângulo, o mesmo conjunto de laços com um ponto em cada
        // oito. Se o custo mal se mexer, decimar o caminho não compra nada e o termo é a ÁREA.
        let thin: Vec<Vec<[f32; 2]>> = loops
            .iter()
            .map(|lp| lp.iter().step_by(8).copied().collect())
            .collect();
        let thin_pts: usize = thin.iter().map(Vec::len).sum();
        let t6 = std::time::Instant::now();
        for _ in 0..n {
            let cov = ph2d_painter_brush::solid::fill_coverage(
                &thin,
                rect.w as usize,
                rect.h as usize,
                origin,
            );
            std::hint::black_box(&cov);
        }
        let thin_ms = t6.elapsed().as_secs_f64() * 1e3 / f64::from(n);
        // …e o PISO de área: o mesmo retângulo com um triângulo minúsculo. O que sobra é a alocação
        // do acumulador, o zero dele e a soma corrida — o custo que NÃO depende do caminho.
        let tiny = vec![vec![
            origin,
            [origin[0] + 3.0, origin[1]],
            [origin[0], origin[1] + 3.0],
        ]];
        let t7 = std::time::Instant::now();
        for _ in 0..n {
            let cov = ph2d_painter_brush::solid::fill_coverage(
                &tiny,
                rect.w as usize,
                rect.h as usize,
                origin,
            );
            std::hint::black_box(&cov);
        }
        let floor_ms = t7.elapsed().as_secs_f64() * 1e3 / f64::from(n);
        println!(
            "{:<20} {thin_pts:>8} {:>8} {thin_ms:>8.3}   <- o MESMO fill com 1 ponto em cada 8",
            "  (decimado 8x)", ""
        );
        println!(
            "{:<20} {:>8} {:>8} {floor_ms:>8.3}   <- PISO de AREA (um triangulo de 3 px)",
            "  (so' a area)", 3, ""
        );
    }
}

/// **O MESMO, com a TELA maior** — porque a área do retângulo é o termo que a tela multiplica, e um
/// custo limitado pela PEGADA não deveria responder a ela.
#[test]
#[ignore = "medição, não gate — rode com --test-threads=1 na máquina calma"]
fn measure_whether_a_solid_move_is_bound_by_the_canvas() {
    let steps = 24usize;
    println!("\n=== O MOVE DE SOLID RESPONDE À TELA? (sym=circ12 + tiling, {steps} eventos) ===\n");
    println!(
        "{:>9} {:>9} {:>11} {:>13} {:>9} {:>9}",
        "canvas", "pontos", "rect px", "SOMA rect px", "ms p50", "ms max"
    );
    for side in [512u32, 1024, 2048] {
        #[allow(clippy::cast_precision_loss)]
        let f = side as f32;
        let mut t = tool(side, PaintMedia::Digital, 6.0);
        arm(&mut t, "circ12", true);
        let ev = solid_arc(&mut t, [f * 0.86, f * 0.5], f * 0.14, steps);
        let last = *ev.last().unwrap();
        let sum_rect: usize = ev.iter().map(|e| e.rect_px).sum();
        let mut ms: Vec<f64> = ev.iter().map(|e| e.ms).collect();
        ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!(
            "{:>9} {:>9} {:>11} {:>13} {:>9.3} {:>9.3}",
            format!("{side}x{side}"),
            last.points,
            last.rect_px,
            sum_rect,
            ms[ms.len() / 2],
            ms.last().unwrap()
        );
    }
}
