//! **O que a mancha do Solid DESENHA** — o irmão do [`super::measure_solid_cost`], que mede o que
//! ela CUSTA. Cortado por responsabilidade quando o pai bateu o teto de LOC.
//!
//! ⚠️ **A linha do corte é a pergunta:** aqui perguntam-se coisas cuja resposta é um conjunto de
//! texels (a teia sobreviveu? a transação escreveu fora do retângulo? cada tipo de linha muda o
//! desenho? decimar o caminho é de graça?); lá, coisas cuja resposta é um relógio ou uma contagem.

use super::measure_shape_system::{cp, tool};
use super::measure_solid_cost::{arm, solid_arc};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::Tool;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

/// **A TEIA SOBREVIVE AO PREENCHIMENTO?** — a pergunta que a ORDEM do ciclo de traço levanta.
///
/// Num evento de ponteiro o produto faz, nesta ordem: `stamp_dabs` (que abre a transação do Solid,
/// **salva** o retângulo e escreve a mancha) e só DEPOIS `park_stroke` → `stamp_threads`. Ou seja, os
/// fios caem **fora** do instantâneo que a transação guardou — e o `peel_drag_preview` do evento
/// seguinte restaura exactamente esse retângulo.
///
/// ⚠️ **A fixture põe a `Strength` do pincel em ZERO, e é ela que torna a pergunta respondível.** Um
/// fio que cai DENTRO da região cheia é invisível por construção (mesma cor sobre mesma cor), então
/// um oráculo que conte texels sobre a mancha **não distingue apagado de invisível** — a primeira
/// versão deste probe mediu `0 de 117` e não podia dizer qual dos dois. A tinta do fio sai por um
/// canal PRÓPRIO (`thread_ink` lê `thread_width_px`/`thread_opacity`, nunca a `strength`), então com
/// a força a zero a mancha e os dabs não escrevem um byte, a transação continua a salvar e a
/// restaurar o retângulo, e **o que estiver na tela é a teia e só ela**.
#[test]
#[ignore = "medição, não gate — auditoria de 2026-08-15"]
fn measure_whether_the_web_survives_the_fill() {
    use ph2d_painter_brush::StrokeMethod;
    use ph2d_painter_brush::line_kind::LineKind;

    let side = 256u32;
    let ink = |t: &PainterTool| -> Vec<bool> {
        t.canvas_rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| p[0] < 250)
            .collect()
    };
    let run = |kind: LineKind, solid: bool| -> Vec<bool> {
        let mut t = tool(side, PaintMedia::Digital, 3.0);
        t.paint.brush.line_kind = kind;
        t.paint.brush.strength = 0.0; // a mancha e os dabs ficam mudos; a teia não
        t.paint.brush.sketchy_reach = 3.0;
        t.paint.brush.sketchy_density = 1.0;
        t.paint.brush.thread_width_px = 1.0;
        t.paint.brush.thread_opacity = 0.5;
        t.paint.brush.stroke_method = StrokeMethod::Space;
        if solid {
            t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
        }
        // Um zigue-zague: as pernas ficam longe o bastante para a teia atravessar o VÃO, que é onde
        // um fio é visível por cima do branco.
        let c = 128.0f32;
        t.on_canvas_pointer(cp([c - 40.0, c], PointerPhase::Down));
        for leg in 0..6 {
            #[allow(clippy::cast_precision_loss)]
            let x = c - 40.0 + (leg as f32) * 16.0;
            let up = if leg % 2 == 0 { 1.0 } else { -1.0 };
            for k in 1..=8 {
                #[allow(clippy::cast_precision_loss)]
                let y = c + up * (k as f32) * 4.0;
                t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp([x + 16.0, c], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([c + 56.0, c], PointerPhase::Up));
        ink(&t)
    };

    let a = run(LineKind::Sketchy, true); // só a teia, com a transação do Solid a correr
    let b = run(LineKind::None, true); // o CONTROLE: sem fios e com força zero, a tela fica limpa
    let c = run(LineKind::Sketchy, false); // só a teia, sem transação nenhuma

    let count = |x: &[bool]| -> usize { x.iter().filter(|v| **v).count() };
    let with_solid = count(&a);
    let without_solid = count(&c);
    let control = count(&b);
    println!(
        "\n=== A TEIA SOBREVIVE AO PREENCHIMENTO? (zigue-zague, Sketchy, Strength 0, canvas {side}) ===\n\
         \x20  texels entintados (só a teia pinta):\n\
         \x20    Sketchy + Solid : {with_solid}\n\
         \x20    Sketchy sem Solid: {without_solid}\n\
         \x20    CONTROLE (sem fios): {control}   <- tem de ser 0, senão a fixture mede outra coisa\n"
    );
    println!(
        "  veredito: a teia sob Solid vale {:.1}% da teia sem Solid",
        if without_solid == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                100.0 * with_solid as f64 / without_solid as f64
            }
        }
    );
}

/// **A MANCHA É FATO DO CAMINHO, OU DE QUANTOS EVENTOS O ENTREGARAM?** — a lei que este módulo já
/// pagou quatro vezes no relevo, feita à transação do Solid.
///
/// ⚠️ **A suspeita tem endereço:** a transação salva o retângulo dos LAÇOS mais o raio da corda, e o
/// Tiling replica um dab quando `centro ± raio` passa a costura — que é uma régua **maior** que a da
/// caixa do laço. Um caminho colado à borda direita tem a caixa DENTRO da tela e dabs de corda cuja
/// pegada passa dela: a cópia envolvida cai na borda ESQUERDA, fora do retângulo salvo, e o restore
/// do evento seguinte não a alcança. Se for assim, cada evento deixa um fantasma e o desenho passa a
/// depender da taxa de eventos.
#[test]
#[ignore = "medição, não gate — auditoria de 2026-08-15"]
fn measure_whether_the_fill_depends_on_the_event_rate() {
    let side = 256u32;
    // Um caminho colado à borda direita: a caixa não cruza a costura, a pegada dos dabs sim.
    let path = |k: usize, n: usize| -> [f32; 2] {
        #[allow(clippy::cast_precision_loss)]
        let f = k as f32 / n as f32;
        [
            248.0 - 8.0 * (f * std::f32::consts::TAU).sin(),
            40.0 + 170.0 * f,
        ]
    };
    let run = |events: usize, tiling: bool| -> Vec<u8> {
        let mut t = tool(side, PaintMedia::Digital, 5.0);
        t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
        if tiling {
            t.toggle_brush_tiling(0);
        }
        t.on_canvas_pointer(cp(path(0, events), PointerPhase::Down));
        for k in 1..events {
            t.on_canvas_pointer(cp(path(k, events), PointerPhase::Move));
        }
        t.on_canvas_pointer(cp(path(events, events), PointerPhase::Up));
        t.canvas_rgba.to_vec()
    };
    println!(
        "\n=== A MANCHA DEPENDE DA TAXA DE EVENTOS? (canvas {side}, caminho colado à borda) ===\n\
         \x20  ⚠️ contaminado: o próprio caminho difere entre 6 e 60 eventos (a coluna tiling=off é o piso)\n"
    );
    for tiling in [false, true] {
        let few = run(6, tiling);
        let many = run(60, tiling);
        let diff = few
            .as_chunks::<4>()
            .0
            .iter()
            .zip(many.as_chunks::<4>().0.iter())
            .filter(|(a, b)| a[0].abs_diff(b[0]) > 8)
            .count();
        // …e onde eles diferem: a faixa envolvida é a coluna 0..12 da borda esquerda.
        let wrap: usize = few
            .as_chunks::<4>()
            .0
            .iter()
            .zip(many.as_chunks::<4>().0.iter())
            .enumerate()
            .filter(|(i, (a, b))| (i % side as usize) < 12 && a[0].abs_diff(b[0]) > 8)
            .count();
        println!(
            "  tiling={:<3}  texels que diferem entre 6 e 60 eventos: {diff:>6}   (na faixa envolvida: {wrap})",
            if tiling { "on" } else { "off" }
        );
    }

    // ── O ORÁCULO EXATO ──────────────────────────────────────────────────────────────────────────
    // Descascar o preview no fim do gesto tem de devolver **exactamente** a tinta cumulativa: os
    // dabs, e nada mais. O que sobrar é fantasma — tinta que a transação escreveu FORA do retângulo
    // que ela salvou, e que nenhum restore volta a alcançar.
    let bare = |solid: bool, tiling: bool| -> Vec<u8> {
        let events = 40usize;
        let mut t = tool(side, PaintMedia::Digital, 5.0);
        if solid {
            t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
        }
        if tiling {
            t.toggle_brush_tiling(0);
        }
        t.on_canvas_pointer(cp(path(0, events), PointerPhase::Down));
        for k in 1..events {
            t.on_canvas_pointer(cp(path(k, events), PointerPhase::Move));
        }
        t.peel_drag_preview(); // fora a mancha e a corda deste evento
        t.canvas_rgba.to_vec()
    };
    println!(
        "\n  --- oráculo EXATO: descascada, a tela do Solid tem de ser a do gesto sem Solid ---"
    );
    for tiling in [false, true] {
        let a = bare(true, tiling);
        let b = bare(false, tiling);
        let ghosts = a
            .as_chunks::<4>()
            .0
            .iter()
            .zip(b.as_chunks::<4>().0.iter())
            .filter(|(x, y)| x[0].abs_diff(y[0]) > 8)
            .count();
        let wrap = a
            .as_chunks::<4>()
            .0
            .iter()
            .zip(b.as_chunks::<4>().0.iter())
            .enumerate()
            .filter(|(i, (x, y))| (i % side as usize) < 12 && x[0].abs_diff(y[0]) > 8)
            .count();
        println!(
            "  tiling={:<3}  fantasmas: {ghosts:>6}   (na faixa envolvida: {wrap})",
            if tiling { "on" } else { "off" }
        );
    }
}

/// **O CENSO DOS SEIS TIPOS SOB SOLID** — cada um faz alguma coisa DISTINTA, ou algum deles é
/// inerte? (auditoria do Enio, 2026-08-15: *"os demais traços não ficaram bons … o efeito do traço
/// não acontece"*).
///
/// ⚠️ **O oráculo é a diferença contra o MESMO gesto em `None`, não um valor absoluto.** Metade dos
/// tipos move a TINTA (Speed arremessa, Ribbon atrasa) e metade decora o traço (Sketchy, Wire) —
/// então *"quantos texels ele pinta"* não os compara. O que se pergunta é: **este tipo muda o
/// desenho?** Um tipo inerte sob Solid dá zero.
///
/// # O que ele mediu (2026-08-15, gesto em C, canvas 256, r=4, Rough armado a 0,4)
///
/// ```text
/// tipo        vs None SEM  vs None COM    tinta SEM    tinta COM
/// Speed                 0            0         3263        11768
/// Sketchy            1075          799         3767        12250
/// Wire               1341            0         3989        11768
/// Ribbon             3194        12227           44           44
/// Rough              3119         1778         4309        12202
/// ```
///
/// **Nenhum tipo é apagado pelo Solid** — a coluna `tinta COM` é ≥ a do `None` em todos. O que a
/// tabela mostra são três coisas de naturezas diferentes, e nenhuma é um defeito da mancha:
///
/// - **Wire dá 0 sob Solid e 1341 sem ele.** Os laços do arame cortam a CONCAVIDADE do C, que é
///   exactamente a região que o preenchimento enche — tinta da mesma cor dentro de uma região cheia
///   dessa cor é **invisível por construção**. O Sketchy dá 799 porque a teia dele alcança fora.
/// - **Speed dá 0 nas duas colunas**, e a fixture é a razão: o arremesso é `v · T`, e um arco de
///   passos curtos com o estabilizador ligado quase não tem `v`. A régua do tipo é o
///   [`super::line_speed_probe`], não este censo.
/// - **Ribbon pinta 44 texels com E sem Solid** — ou seja, **o Solid não é a variável**. A fita é
///   uma MOLA, e passos de ~1,65 px nunca a aceleram: ela fica dentro de um espaçamento e o motor
///   emite um dab. Medido, 160 eventos sobre o MESMO arco dão os mesmos 44, e o probe próprio dela
///   (`probe_ribbon_look`, reta rápida a 2048²) mede faixas de 42 a 356 px. É pergunta da FITA, com
///   dono e sonda próprios; fica NOMEADA e não foi perseguida aqui.
///
/// ⚠️ **E o `Rough` só entra na tabela porque a sonda o ARMA:** `rough_amount` e `rough_bowing`
/// nascem em **0,0**, então escolher `Rough` no dropdown não muda um pixel até o artista mexer no
/// slider. O `spec_default.rs` argumenta o contrário quatro linhas acima, para o Ribbon — *"um tipo
/// escolhido tem de FAZER alguma coisa"* — e o Spray teve de armar um default pela mesma razão.
/// **Decisão de produto do Enio**, não contrabandeada aqui.
#[test]
#[ignore = "medição, não gate — auditoria de 2026-08-15"]
fn measure_that_every_line_kind_does_something_under_solid() {
    use ph2d_painter_brush::StrokeMethod;
    use ph2d_painter_brush::line_kind::LineKind;

    let side = 256u32;
    let run = |kind: LineKind, solid: bool| -> Vec<u8> {
        let mut t = tool(side, PaintMedia::Digital, 4.0);
        t.paint.brush.line_kind = kind;
        t.paint.brush.stroke_method = StrokeMethod::Space;
        t.paint.brush.sketchy_reach = 3.0;
        t.paint.brush.sketchy_density = 1.0;
        t.paint.brush.thread_width_px = 1.0;
        t.paint.brush.thread_opacity = 0.5;
        // ⚠️ **O Rough é ARMADO à mão, e isso é o achado**: `rough_amount`/`rough_bowing` nascem em
        // ZERO, então escolher `Rough` no dropdown não muda um pixel até o artista mexer no slider.
        // O censo mede a CAPACIDADE do tipo; o default fica NOMEADO à parte.
        t.paint.brush.rough_amount = 0.4;
        t.paint.brush.rough_bowing = 0.4;
        if solid {
            t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
        }
        // Um "C": ele cerca área (a mancha existe) e volta para perto de si (há vizinhos a costurar).
        let c = 128.0f32;
        t.on_canvas_pointer(cp([c + 50.0, c - 50.0], PointerPhase::Down));
        for k in 1..=40 {
            #[allow(clippy::cast_precision_loss)]
            let a = 0.9 + (k as f32) * 0.11;
            t.on_canvas_pointer(cp(
                [c + 60.0 * a.cos(), c + 60.0 * a.sin()],
                PointerPhase::Move,
            ));
        }
        t.on_canvas_pointer(cp([c + 50.0, c + 50.0], PointerPhase::Up));
        t.canvas_rgba.to_vec()
    };
    let diff = |a: &[u8], b: &[u8]| -> usize {
        a.as_chunks::<4>()
            .0
            .iter()
            .zip(b.as_chunks::<4>().0.iter())
            .filter(|(x, y)| x[0].abs_diff(y[0]) > 8)
            .count()
    };
    println!(
        "\n=== CADA TIPO DE LINHA MUDA O DESENHO SOB SOLID? (gesto em C, canvas {side}) ===\n"
    );
    println!(
        "{:<10} {:>12} {:>12} {:>12} {:>12}",
        "tipo", "vs None SEM", "vs None COM", "tinta SEM", "tinta COM"
    );
    let base_off = run(LineKind::None, false);
    let base_on = run(LineKind::None, true);
    for kind in [
        LineKind::Speed,
        LineKind::Sketchy,
        LineKind::Wire,
        LineKind::Ribbon,
        LineKind::Rough,
    ] {
        let off = run(kind, false);
        let on = run(kind, true);
        let ink_off = off.as_chunks::<4>().0.iter().filter(|p| p[0] < 250).count();
        let ink_on = on.as_chunks::<4>().0.iter().filter(|p| p[0] < 250).count();
        println!(
            "{:<10} {:>12} {:>12} {:>12} {:>12}",
            format!("{kind:?}"),
            diff(&off, &base_off),
            diff(&on, &base_on),
            ink_off,
            ink_on
        );
    }
    let i0 = base_off
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] < 250)
        .count();
    let i1 = base_on
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] < 250)
        .count();
    println!("{:<10} {:>12} {:>12} {i0:>12} {i1:>12}", "None", 0, 0);
}

/// **DECIMAR O CAMINHO DE TINTA É DE GRAÇA, OU É MUDANÇA DE LOOK?** — a pergunta que o item 2 dos
/// abertos deixou, respondida com número em vez de opinião.
///
/// O caminho grava **todo centro de dab**, e dabs distam ~1 px. Numa curva de raio `R` amostrada a
/// passo `s`, a flecha de `k` pontos é `(k·s)²/(8R)` — ou seja, num traço quase reto muitos pontos
/// consecutivos descrevem a MESMA fronteira. Decimá-los corta arestas do `fill_coverage`, que é o
/// que sobrou serial depois do ADR-0158.
///
/// ⚠️ **O oráculo é a COBERTURA, não a contagem:** mover a fronteira `t` px muda a área coberta do
/// texel de borda em até `t`, ou seja `255·t` níveis. A tolerância só é *de graça* se o pior delta
/// couber no arredondamento que o `u8` já faz (**1 nível**); acima disso é decisão de LOOK e o smoke
/// é quem julga.
#[test]
#[ignore = "medição, não gate — auditoria de 2026-08-15"]
fn measure_whether_decimating_the_ink_path_is_free() {
    /// Douglas-Peucker, iterativo — só para medir; se pagar, vai para o produto.
    fn dp(pts: &[[f32; 2]], tol: f32) -> Vec<[f32; 2]> {
        if pts.len() < 3 {
            return pts.to_vec();
        }
        let mut keep = vec![false; pts.len()];
        keep[0] = true;
        keep[pts.len() - 1] = true;
        let mut stack = vec![(0usize, pts.len() - 1)];
        while let Some((i, j)) = stack.pop() {
            if j <= i + 1 {
                continue;
            }
            let (a, b) = (pts[i], pts[j]);
            let d = [b[0] - a[0], b[1] - a[1]];
            let len = (d[0] * d[0] + d[1] * d[1]).sqrt().max(1e-6);
            let (mut worst, mut at) = (0.0f32, i);
            for (k, p) in pts.iter().enumerate().take(j).skip(i + 1) {
                let e = ((p[0] - a[0]) * d[1] - (p[1] - a[1]) * d[0]).abs() / len;
                if e > worst {
                    worst = e;
                    at = k;
                }
            }
            if worst > tol {
                keep[at] = true;
                stack.push((i, at));
                stack.push((at, j));
            }
        }
        pts.iter()
            .zip(keep.iter())
            .filter(|(_, k)| **k)
            .map(|(p, _)| *p)
            .collect()
    }

    let side = 1024u32;
    let mut t = tool(side, PaintMedia::Digital, 6.0);
    arm(&mut t, "circ12", true);
    let _ = solid_arc(&mut t, [960.0, 512.0], 140.0, 96);
    // O caminho que o gesto deixou, e a mancha que ele produz hoje.
    let path = t.paint.solid_path.clone();
    let full = t.solid_fill_loops();
    let rect = t.solid_fill_rect(&full).expect("a mancha tem de existir");
    #[allow(clippy::cast_precision_loss)]
    let origin = [rect.x as f32, rect.y as f32];
    let cov = |loops: &[Vec<[f32; 2]>]| {
        ph2d_painter_brush::solid::fill_coverage(loops, rect.w as usize, rect.h as usize, origin)
    };
    let base = cov(&full);
    println!(
        "\n=== DECIMAR O CAMINHO DE TINTA É DE GRAÇA? (caminho de {} pontos, rosácea de {} laços) ===\n\
         \x20  o piso do arredondamento do `u8` é 1 nível — acima disso é decisão de LOOK\n",
        path.len(),
        full.len()
    );
    println!(
        "{:>10} {:>10} {:>9} {:>12} {:>12} {:>10}",
        "tol px", "pontos", "corte", "pior delta", "texels != ", "fill ms"
    );
    for tol in [0.0f32, 0.01, 0.02, 0.05, 0.1, 0.25, 0.5] {
        let thin = dp(&path, tol);
        t.paint.solid_path = thin.clone();
        let loops = t.solid_fill_loops();
        let out = cov(&loops);
        let worst = base
            .iter()
            .zip(out.iter())
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap_or(0);
        let ne = base.iter().zip(out.iter()).filter(|(a, b)| a != b).count();
        let n = 6;
        let t0 = std::time::Instant::now();
        for _ in 0..n {
            std::hint::black_box(cov(&loops));
        }
        let ms = t0.elapsed().as_secs_f64() * 1e3 / f64::from(n);
        #[allow(clippy::cast_precision_loss)]
        let cut = 1.0 - thin.len() as f64 / path.len() as f64;
        println!(
            "{tol:>10.2} {:>10} {:>8.0}% {worst:>12} {ne:>12} {ms:>10.3}",
            thin.len(),
            cut * 100.0
        );
    }
}
