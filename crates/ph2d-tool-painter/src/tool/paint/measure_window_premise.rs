//! **A janela do fold é honesta?** — a sonda que pergunta se `Some(rect)` de fato CONTÉM tudo que
//! mudou. Irmã de `measure_gpu_frontier.rs` e separada dele de propósito: aquele mede *quanto custa
//! o fold*, este mede *se a reivindicação é verdadeira*, e são duas perguntas.
//!
//! Split por teto de LOC (o irmão bateu 746 > 700) na linha de corte do assunto, não do tamanho.

// ─────────────────────────────────────────────────────────────────────────────
// A PREMISSA DA JANELA: `Some` promete confinamento — mas confina TUDO?
// ─────────────────────────────────────────────────────────────────────────────

/// **Mede se o retângulo sujo CONTÉM todo texel de relevo que mudou.**
///
/// A pista GPU inteira (upload por-camada e fold do impasto) lê `Some(rect)` como *"a mudança deste
/// frame cabe aqui"*. Se algum caminho move um texel FORA dele, o produto serve estado velho ali — e
/// como a pista CPU lê o MESMO `dirty_rect` para o `preview_upload_bbox` dela, um furo destes não é
/// da GPU: é das duas.
///
/// ## Medido (2026-07-25, 96²)
///
/// | cenário | reivindicação | texels FORA | pior Δcover | pior Δh |
/// |---|---|---|---|---|
/// | traço comum (CONTROLE) | `11,29 34x23` | **0** | 0 | 0,000 |
/// | drag dot ENCOLHENDO | `41,14 15x36` | **4** | 85/255 | **0,000** |
///
/// ⚠️ **A premissa vale exatamente onde o produto passa a vida** (um traço comum não escapa um
/// texel) e vaza **4 texels** no re-stamp que ENCOLHE — em COBERTURA apenas, com a altura intacta.
/// Não é da pista GPU: as duas pistas leem o MESMO `dirty_rect` (`preview_upload_bbox` é o gêmeo
/// dele), então o mesmo resíduo existe no caminho CPU e **precede** a wave do fold regional. É
/// contabilidade do re-stamp — quem restaura a pegada MAIOR anterior não a une ao retângulo — e o
/// dono daquele caminho é quem deve fechá-la.
///
/// Não afirma nada; IMPRIME. `cargo test -p ph2d-tool-painter --release the_window_premise -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_window_premise_is_measured_not_assumed() {
    use crate::tool::PainterTool;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
    use ph2d_painter_brush::{BrushSpec, Falloff, StrokeMethod};

    const N: u32 = 96;
    fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
        CanvasPointer {
            pos,
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        }
    }
    fn brush(method: StrokeMethod, r: f32) -> BrushSpec {
        BrushSpec {
            radius_px: r,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.1, 0.2, 0.3],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.5,
            impasto_smoothing: 0.0,
            impasto_body: 1.0,
            stroke_method: method,
            ..Default::default()
        }
    }
    fn arm(t: &mut PainterTool, b: BrushSpec) {
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
    }
    // (h, cover, mats) por texel — o que a luz de fato integra.
    fn planes_opt(t: &PainterTool) -> Option<Vec<(f32, u8, u8, u8)>> {
        let p = t.impasto_gpu_planes()?;
        Some(
            (0..(N * N) as usize)
                .map(|i| (p.relief[i], p.cover[i], p.mat0[i * 4], p.mat1[i * 4]))
                .collect(),
        )
    }
    // Quantos texels mudaram FORA do retângulo declarado, e o pior delta de cobertura lá fora.
    fn escape(
        before: &[(f32, u8, u8, u8)],
        after: &[(f32, u8, u8, u8)],
        claim: Option<(u32, u32, u32, u32)>,
    ) -> (usize, u8, f32) {
        let (mut n, mut worst_cov, mut worst_h) = (0usize, 0u8, 0.0f32);
        for y in 0..N {
            for x in 0..N {
                let i = (y * N + x) as usize;
                if before[i] == after[i] {
                    continue;
                }
                let inside = claim.is_none_or(|(rx, ry, rw, rh)| {
                    x >= rx && x < rx + rw && y >= ry && y < ry + rh
                });
                if !inside {
                    n += 1;
                    worst_cov = worst_cov.max(after[i].1.abs_diff(before[i].1));
                    worst_h = worst_h.max((after[i].0 - before[i].0).abs());
                }
            }
        }
        (n, worst_cov, worst_h)
    }

    println!("[window-premise] cenario                     claim              fora  dcov   dh");
    let probe = |name: &str, build: &dyn Fn() -> PainterTool, act: &dyn Fn(&mut PainterTool)| {
        let mut t = build();
        let _ = t.take_preview_dirty(); // zera a reivindicação anterior
        let Some(before) = planes_opt(&t) else {
            // Uma sessao de sculpt CONGELA os planos no pen-down, entao a sonda nao tem "antes" para
            // comparar: o cenario nao monta desta forma, e dizer isso e mais honesto que imprimir zero.
            println!("[window-premise] {name:<28} (cenario nao montou nesta sonda)");
            return;
        };
        act(&mut t);
        let dirty = t.take_preview_dirty();
        let claim = t.preview_gpu_region();
        let after = planes_opt(&t).expect("planos no DEPOIS");
        let (n, dc, dh) = escape(&before, &after, claim);
        let c = claim.map_or("None (inteiro)".to_string(), |(x, y, w, h)| {
            format!("{x},{y} {w}x{h}")
        });
        println!("[window-premise] {name:<28} {c:<18} {n:>5} {dc:>5} {dh:>6.3}  (dirty={dirty})");
    };

    // CONTROLE: um traço comum. Aqui a premissa TEM de valer.
    probe(
        "traco comum (controle)",
        &|| {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
            arm(&mut t, brush(StrokeMethod::FreeHand, 8.0));
            t.on_canvas_pointer(cp([20.0, 40.0], PointerPhase::Down));
            t.on_canvas_pointer(cp([30.0, 40.0], PointerPhase::Move));
            t
        },
        &|t| {
            t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Move));
        },
    );

    // DRAG DOT que ENCOLHE: o re-stamp desenha um disco MENOR, e o anel do disco maior
    // anterior tem de ser restaurado — a restauração está dentro do retângulo?
    probe(
        "drag dot encolhendo",
        &|| {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
            arm(&mut t, brush(StrokeMethod::DragDot, 6.0));
            t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Down));
            t.on_canvas_pointer(cp([48.0, 20.0], PointerPhase::Move)); // disco GRANDE
            t
        },
        &|t| {
            t.on_canvas_pointer(cp([48.0, 42.0], PointerPhase::Move)); // encolhe
        },
    );

    // SCULPT INFLATE: a bola move MATÉRIA, e o suporte dela é a janela dilatada.
    probe(
        "sculpt inflate",
        &|| {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
            arm(&mut t, brush(StrokeMethod::FreeHand, 10.0));
            t.on_canvas_pointer(cp([30.0, 48.0], PointerPhase::Down));
            t.on_canvas_pointer(cp([66.0, 48.0], PointerPhase::Move));
            t.on_canvas_pointer(cp([66.0, 48.0], PointerPhase::Up));
            t.set_paint_tool_mode("sculpt");
            t.set_sculpt_mode(7); // Inflate
            t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Down));
            t
        },
        &|t| {
            t.on_canvas_pointer(cp([50.0, 48.0], PointerPhase::Move));
        },
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// O SNAPSHOT DO PINCEL NÃO PODE CUSTAR O TAMANHO DA TELA
// ─────────────────────────────────────────────────────────────────────────────

/// **`brush_settings()` é O(1) na tela — e um dia não foi.**
///
/// Ele publica um `Copy` de floats para o painel, e o frame o chama **DUAS** vezes (o publish do painel
/// e o anel do cursor). Um dos campos, `sculpt_can_filter_stroke`, é um **booleano** que era respondido
/// construindo o payload inteiro: `Arc::new(live_paint.clone())` — 16,7 M de `f32` = **67 MB memcpy**, a
/// 4096², **por chamada**. Medido no produto pelo split por-chamada do `PH2D_PAINT_PERF`:
///
/// | slot | p50 | | |
/// |---|---|---|---|
/// | `PANEL/brushsnap` | **3,9 ms** | `CHROME/ring` | **3,7 ms** |
///
/// …que somam exatamente o `dispatch p50 = 7,5 ms` que o Enio via **com a mão parada**.
///
/// **RAZÃO, nunca wall-clock:** o `ci-test` compila em `opt-level=1` e uma barra de milissegundos mede o
/// PERFIL, não o código. A tela de 4096² tem **16× a área** da de 1024², então um custo canvas-shaped
/// aparece como razão ~16; um snapshot honesto fica plano.
///
/// **Mutação que deve sangrar:** `has_live_stroke_envelope()` → `live_stroke_envelope().is_some()` em
/// `can_filter_last_stroke`.
#[test]
fn the_brush_snapshot_costs_the_same_on_a_canvas_sixteen_times_bigger() {
    use crate::tool::PainterTool;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
    use ph2d_painter_brush::{BrushSpec, Falloff};

    fn snapshot_ms(side: u32) -> f64 {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (side * side * 4) as usize], side, side);
        let b = BrushSpec {
            radius_px: 10.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.1, 0.2, 0.3],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.5,
            impasto_smoothing: 0.0,
            impasto_body: 1.0,
            ..Default::default()
        };
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        // A premissa: um traço DE VERDADE, senão o envelope está vazio e o caminho caro nunca corre —
        // a fixture tem de CONTER o fenômeno.
        let cp = |p: [f32; 2], phase| CanvasPointer {
            pos: p,
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        let m = f64::from(side) / 96.0;
        t.on_canvas_pointer(cp([30.0 * m as f32, 48.0 * m as f32], PointerPhase::Down));
        t.on_canvas_pointer(cp([66.0 * m as f32, 48.0 * m as f32], PointerPhase::Move));
        t.on_canvas_pointer(cp([66.0 * m as f32, 48.0 * m as f32], PointerPhase::Up));

        const N: u32 = 20;
        let t0 = std::time::Instant::now();
        for _ in 0..N {
            std::hint::black_box(t.brush_settings());
        }
        t0.elapsed().as_secs_f64() * 1e3 / f64::from(N)
    }

    let small = snapshot_ms(1024);
    let big = snapshot_ms(4096);
    // Piso para não dividir por um relógio de resolução grossa num snapshot que é, corretamente, ~0.
    let ratio = big / small.max(1e-4);
    println!("[brush-snapshot] 1024 {small:.4} ms · 4096 {big:.4} ms · razao {ratio:.2}x");
    assert!(
        ratio < 4.0,
        "o snapshot do pincel cresceu com a TELA (razao {ratio:.2}x para 16x a area: {small:.4} -> \
         {big:.4} ms). Alguem voltou a responder uma PERGUNTA construindo o PAYLOAD — o frame chama \
         isto duas vezes, entao o preco sai em dobro e com a mao parada."
    );
}
