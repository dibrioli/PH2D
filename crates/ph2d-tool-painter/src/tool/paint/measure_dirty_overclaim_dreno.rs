//! T2 da sonda do over-claim — filho de `measure_dirty_overclaim.rs` (`#[path]`) pelo tecto de LOC.

use super::*;

// ─────────────────────────────────────────────────────────────────────────────
// T2 — O QUE O 1º CONSUMIDOR GANHOU (o composite parcial da pista CPU)
// ─────────────────────────────────────────────────────────────────────────────

/// **Quanto custa uma drenagem, agora que ela percorre TILES.**
///
/// A sonda do over-claim mede a REIVINDICAÇÃO; esta mede o RELÓGIO do consumidor que passou a honrá-la.
/// Ela não afirma nada — o antes/depois se obtém rodando-a com a mutação *"um passe sobre o bbox"*
/// aplicada ao `take_preview_arc`, que é como o número desta wave foi levantado.
///
/// ## Medido (2026-07-25, 1024², pincel r=12, impasto ligado)
///
/// | cena | bbox (antes) | tiles (depois) |
/// |---|---|---|
/// | traço reto curto | *ver output* | *ver output* |
/// | diagonal, mão rápida | | |
/// | traço em L | | |
///
/// Rodar: `cargo test -p ph2d-tool-painter --profile smoke the_partial_drain_cost -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_partial_drain_cost_is_measured_not_assumed() {
    let _ = crate::raio_da_sonda::perfil_da_sonda();
    /// Mede a MEDIANA de `take_preview_arc` sobre um gesto repetido — mediana, não média, porque um
    /// outlier de agendador não descreve o produto.
    ///
    /// ⚠️ **O gesto é UM TRAÇO ABERTO, drenado entre lotes de movimento** — que é o que o produto faz
    /// 60 vezes por segundo. A 1ª versão desta sonda fechava o traço (`PointerPhase::Up`) a cada
    /// rodada e media o frame do **COMMIT**, onde o envelope inteiro é re-sujado: ali a reivindicação
    /// por tiles é igual ou MAIOR que o bbox (medido, 414.208 contra 419.904 texels) e a frente parece
    /// inútil. O frame do commit acontece uma vez por traço; os outros sessenta são estes.
    fn drain_ms(name: &str, act: &dyn Fn(&mut PainterTool, u8)) {
        let mut t = fresh(StrokeMethod::Space, 12.0);
        // A pilha tem de ser NÃO-trivial para a drenagem COMPOR (senão ela devolve o `canvas_rgba` cru
        // e esta sonda mediria um `Arc::clone`). Um traço de impasto basta.
        t.on_canvas_pointer(cp([500.0, 500.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([510.0, 500.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([510.0, 500.0], PointerPhase::Up));
        let _ = t.take_preview_arc();

        let mut samples = Vec::new();
        for round in 0..12u8 {
            act(&mut t, round);
            let t0 = std::time::Instant::now();
            let got = t.take_preview_arc();
            samples.push(t0.elapsed().as_secs_f64() * 1e3);
            assert!(got.is_some(), "a drenagem nao produziu nada");
        }
        samples.sort_by(f64::total_cmp);
        println!(
            "[drain] {name:<26} p50 {:>7.3} ms  braco {:?}",
            samples[samples.len() / 2],
            t.last_drain_branch
        );
    }

    println!("[drain] cena                        mediana");
    // Cada rodada é UM FRAME de um traço que continua aberto: o lote de movimento, e depois a drenagem.
    drain_ms("reto, mao calma", &|t, k| {
        if k == 0 {
            t.on_canvas_pointer(cp([60.0, 512.0], PointerPhase::Down));
        }
        let x0 = 60.0 + f32::from(k) * 60.0;
        for i in 1u8..=16 {
            t.on_canvas_pointer(cp([x0 + f32::from(i) * 3.75, 512.0], PointerPhase::Move));
        }
    });
    drain_ms("diagonal, mao rapida", &|t, k| {
        if k == 0 {
            t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Down));
        }
        let o = 40.0 + f32::from(k) * 80.0;
        for i in 1u8..=16 {
            let d = o + f32::from(i) * 5.0;
            t.on_canvas_pointer(cp([d, d], PointerPhase::Move));
        }
    });
    // O L num único frame: as duas pernas no MESMO lote, que é o gesto que o bbox mais mente.
    drain_ms("cotovelo num frame", &|t, k| {
        if k == 0 {
            t.on_canvas_pointer(cp([100.0, 100.0], PointerPhase::Down));
        }
        let o = 100.0 + f32::from(k) * 20.0;
        for i in 1u8..=8 {
            t.on_canvas_pointer(cp([o + f32::from(i) * 50.0, o], PointerPhase::Move));
        }
        for i in 1u8..=8 {
            t.on_canvas_pointer(cp([o + 400.0, o + f32::from(i) * 50.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([o + 20.0, o + 400.0], PointerPhase::Move));
    });
}

/// **ONDE vão os 14 ms de uma drenagem parcial.**
///
/// A medição do T2 mostrou que cortar a área do composite em **5×** move o relógio da drenagem em
/// **5%** — o que só pode significar que o composite não é o custo dominante. Esta sonda parte o
/// braço parcial em estágios e mede cada um sobre a MESMA região, para o próximo passo da frente
/// atacar o que de fato custa em vez do que a intuição diz.
///
/// Rodar: `cargo test -p ph2d-tool-painter --profile smoke the_partial_drain_stages -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_partial_drain_stages_are_measured_not_assumed() {
    let _ = crate::raio_da_sonda::perfil_da_sonda();
    use crate::compositor::{Region, composite_region};

    let mut t = fresh(StrokeMethod::Space, 12.0);
    // Pilha não-trivial + relevo de verdade (senão a luz não roda e a sonda mede outra coisa).
    t.on_canvas_pointer(cp([500.0, 500.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([510.0, 500.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([510.0, 500.0], PointerPhase::Up));
    let _ = t.take_preview_arc();
    // A diagonal de mão rápida: a cena cujo bbox mente 17×.
    t.on_canvas_pointer(cp([100.0, 100.0], PointerPhase::Down));
    for i in 1u8..=16 {
        let d = 100.0 + f32::from(i) * 37.5;
        t.on_canvas_pointer(cp([d, d], PointerPhase::Move));
    }
    let bbox = t.dirty_rect.expect("o gesto reivindicou alguma coisa");
    println!(
        "[stages] bbox {}x{} = {} texels",
        bbox.w,
        bbox.h,
        u64::from(bbox.w) * u64::from(bbox.h)
    );

    let ms = |f: &mut dyn FnMut()| {
        const N: u32 = 8;
        let t0 = std::time::Instant::now();
        for _ in 0..N {
            f();
        }
        t0.elapsed().as_secs_f64() * 1e3 / f64::from(N)
    };

    let active = t.layers.active().unwrap_or(crate::layers::LayerId(0));
    let mut buf = Vec::new();
    println!(
        "[stages] composite_region(bbox)     {:>7.3} ms",
        ms(&mut || {
            let src = crate::tool::internal::ToolPixelSource {
                active_id: active,
                active_rgba: &t.canvas_rgba,
                images: &t.images,
            };
            buf = composite_region(&t.layers, &src, N, N, bbox);
        })
    );
    println!(
        "[stages] apply_impasto_light(bbox)  {:>7.3} ms",
        ms(&mut || t.apply_impasto_light(&mut buf, bbox))
    );
    println!(
        "[stages] impasto_fields() sozinho   {:>7.3} ms",
        ms(&mut || {
            std::hint::black_box(t.impasto_fields().is_some());
        })
    );
    let full = Region {
        x: 0,
        y: 0,
        w: N,
        h: N,
    };
    println!(
        "[stages] composite_region(TELA)     {:>7.3} ms",
        ms(&mut || {
            let src = crate::tool::internal::ToolPixelSource {
                active_id: active,
                active_rgba: &t.canvas_rgba,
                images: &t.images,
            };
            std::hint::black_box(composite_region(&t.layers, &src, N, N, full).len());
        })
    );
}
