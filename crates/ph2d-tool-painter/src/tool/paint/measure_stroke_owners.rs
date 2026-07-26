//! **QUEM segura os planos, e de que é feito o PEN-UP** — as duas perguntas que reescreveram o plano da
//! porta única de escrita.
//!
//! A decomposição anterior (doc 28 §5.13) disse que o pen-up custa 38,9 ms a 4096² **porque o
//! `commit_stroke_height` forka os três planos de relevo**, e fechou a conta somando `Vec::clone` dos
//! três: 0,40 + 9,94 + 18,13 = 28,47 contra 32,8 medidos. As sondas deste arquivo derrubaram os dois
//! lados dessa frase.
//!
//! ## 1. O dono extra é PERMANENTE, e é o histórico
//!
//! Em repouso, com dois traços commitados e nenhum gesto aberto, **os quatro planos canvas-shaped têm
//! DOIS donos**; limpar o histórico leva a contagem a **um**. Ou seja o `cursor` que a U1 instalou como
//! base de todo delta é um segundo dono permanente — e a hipótese fácil (*"o dono extra é o snapshot de
//! pen-down"*) descrevia só metade. Consequência de projeto: um journal que substituísse apenas o
//! `paint.stroke_undo` deixaria a contagem em 2 e **não mudaria um milissegundo**.
//!
//! ## 2. O fork custa um TERÇO do que a aritmética dizia
//!
//! `Vec::clone` é um memcpy **serial**; o produto usa [`super::plane_fork::fork_par`], que é paralelo.
//! Medido pela porta do produto a 4096²: **0,29 + 3,16 + 5,79 = 9,25 ms**, não 28,47. A soma que
//! "fechava" fechava por **coincidência** — e uma atribuição que casa com o total por acidente é pior
//! que nenhuma, porque encerra a investigação no lugar errado.
//!
//! ## 3. O pen-up é o COMMIT DE UNDO, e o `commit_stroke_height` é footprint-bound
//!
//! Ablacionando pela entrada (`paint.stroke_undo = None` faz o `close_stroke` pular o commit
//! estrutural), a 4096² com impasto: **40,20 ms completo contra 3,49 sem o commit** — e o que sobra é
//! **plano na tela** (3,35 / 3,43 / 3,49 a 1024²/2048²/4096²), que é a forma correta para trabalho
//! limitado pela pegada. **91% do pen-up era o histórico**, cujo `PlaneDeltas::split` varre
//! `diff_window` sobre os quatro planos: ~256 MB de comparação por traço.

use super::*;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::Falloff;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um canvas com pincel de impasto — o modo cujo pen-up paga os três planos de relevo.
fn armed(size: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: 40.0,
        hardness: 0.5,
        falloff: Falloff::Sphere,
        strength: 1.0,
        color: [0.2, 0.3, 0.8],
        impasto: true,
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t
}

fn stroke(t: &mut PainterTool, y: f32) {
    t.on_canvas_pointer(cp([60.0, y], PointerPhase::Down));
    for k in 1..=6u8 {
        let x = 60.0 + f32::from(k) * 30.0;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([260.0, y], PointerPhase::Up));
}

fn plane_owners<T>(
    m: &std::collections::BTreeMap<crate::layers::LayerId, std::sync::Arc<Vec<T>>>,
    active: Option<crate::layers::LayerId>,
) -> usize {
    active
        .and_then(|a| m.get(&a).map(std::sync::Arc::strong_count))
        .unwrap_or(0)
}

fn owners(t: &PainterTool) -> (usize, usize, usize, usize) {
    let a = t.layers.active();
    (
        std::sync::Arc::strong_count(&t.canvas_rgba),
        plane_owners(&t.heights, a),
        plane_owners(&t.covers, a),
        plane_owners(&t.mats, a),
    )
}

/// **A contagem de donos em REGIME, e de quem é cada um.**
///
/// Roda dois traços (o primeiro instala o histórico) e conta os donos de cada plano canvas-shaped no
/// repouso entre eles — depois derruba os suspeitos, um a um, para atribuir.
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn who_holds_the_planes_when_a_stroke_begins() {
    let mut t = armed(1024);
    stroke(&mut t, 200.0);
    stroke(&mut t, 300.0);

    let (c, h, cv, m) = owners(&t);
    eprintln!("\n[donos] REGIME (2 tracos commitados, nenhum gesto aberto)");
    eprintln!("  canvas_rgba {c} · heights {h} · covers {cv} · mats {m}");
    eprintln!("  (1 = so o tool ⇒ a proxima escrita e IN PLACE; ≥2 ⇒ ela COPIA o plano inteiro)");

    // Suspeito 1: o snapshot de pen-down. Em repouso ele nem existe.
    eprintln!("  stroke_undo vivo? {}", t.paint.stroke_undo.is_some());

    // Suspeito 2: o histórico (o `cursor` da U1 + as entradas).
    t.undo.clear();
    let (c2, h2, cv2, m2) = owners(&t);
    eprintln!("[donos] depois de LIMPAR o historico");
    eprintln!("  canvas_rgba {c2} · heights {h2} · covers {cv2} · mats {m2}");

    // E dentro do gesto: quantos donos o primeiro dab encontra?
    let mut t2 = armed(1024);
    stroke(&mut t2, 200.0);
    t2.on_canvas_pointer(cp([60.0, 300.0], PointerPhase::Down));
    let (c3, h3, cv3, m3) = owners(&t2);
    eprintln!("[donos] DENTRO do gesto (logo apos o pen-down)");
    eprintln!("  canvas_rgba {c3} · heights {h3} · covers {cv3} · mats {m3}\n");
}

/// **De que é feito o PEN-UP, por ABLAÇÃO** — a sonda que nomeou a causa (ver o cabeçalho, §3).
///
/// Pergunta ao produto ablacionando pela ENTRADA, nunca instrumentando o laço: o mesmo pen-up com e sem
/// o **commit de undo** (`paint.stroke_undo = None` faz `close_stroke` pular `commit_structural_edit`).
/// O que sobra é o `commit_stroke_height`.
///
/// ⚠️ A ablação tira DUAS coisas de uma vez, e é honesto dizer quais: o commit estrutural em si, **e** o
/// segundo dono que aquele snapshot representa — sem ele os forks do `commit_stroke_height` acontecem
/// in-place. É por isso que a diferença (36,7 ms a 4096²) é maior que o `record_structural` isolado
/// (10,96): a conta é `commit` + `forks` (9,25) + o `free()` dos buffers que o fork deixou órfãos.
///
/// | 4096², impasto | antes | depois |
/// |---|---|---|
/// | pen-up completo | 40,20 | **32,34** |
/// | pen-up sem o commit | 3,49 | 3,93 |
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn what_the_pen_up_is_made_of_by_ablation() {
    use std::time::Instant;

    // Um traço FIXO em px (o mesmo desenho nas duas telas), 8 traços no MESMO tool, o 1º descartado,
    // mediana dos sete.
    //
    // ⚠️ **O tool é reusado de propósito.** Um tool novo por repetição faz de TODO traço o primeiro da
    // camada, e o primeiro paga a alocação preguiçosa dos três planos de relevo (192 MB a 4096²):
    // mediria a estreia repetidamente em vez do regime que o artista vive. Descartar o 1º é o que
    // separa as duas coisas.
    fn pen_up_ms(side: u32, impasto: bool, with_undo_commit: bool) -> f64 {
        let mut t = armed(side);
        if !impasto {
            t.paint.brush.impasto = false;
            for slot in &mut t.paint.brush_by_mode {
                slot.impasto = false;
            }
        }
        let mut v = Vec::new();
        for k in 0..8u8 {
            let y = 200.0 + f32::from(k) * 8.0; // caminhos distintos, mesma forma
            t.on_canvas_pointer(cp([60.0, y], PointerPhase::Down));
            for j in 1..=6u8 {
                t.on_canvas_pointer(cp([60.0 + f32::from(j) * 30.0, y], PointerPhase::Move));
            }
            if !with_undo_commit {
                t.paint.stroke_undo = None; // a ablação: o commit estrutural não roda
            }
            let t0 = Instant::now();
            t.on_canvas_pointer(cp([260.0, y], PointerPhase::Up));
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            if k > 0 {
                v.push(dt);
            }
        }
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    }

    println!(
        "\n{:<22} {:>10} {:>10} {:>10}",
        "pen-up (mediana de 7)", "1024", "2048", "4096"
    );
    for (name, impasto) in [("digital", false), ("impasto", true)] {
        for (tag, undo) in [("completo", true), ("sem undo", false)] {
            let a = pen_up_ms(1024, impasto, undo);
            let b = pen_up_ms(2048, impasto, undo);
            let c = pen_up_ms(4096, impasto, undo);
            println!("{name} {tag:<13} {a:>10.2} {b:>10.2} {c:>10.2}");
        }
    }
    println!(
        "\n  (completo - sem undo) = o que o COMMIT DE UNDO custa; o resto e o commit_stroke_height\n"
    );
}

/// **O commit de undo, isolado pela porta dele** — a confirmação do que a ablação nomeou.
///
/// A ablação diz que o pen-up com impasto a 4096² custa 40,2 ms e que **36,7 deles somem quando o
/// commit estrutural não roda**. Esta sonda chama o commit direto (`record_structural`, a porta real)
/// sobre dois snapshots que diferem por UM traço, e mostra como o custo cresce com a tela.
///
/// O que ele faz por dentro é `PlaneDeltas::split`, que roda **`diff_window` sobre todo plano que os
/// `Arc`s não deram como idêntico** — com impasto são quatro (canvas + heights + covers + mats), e a
/// 4096² isso é ~256 MB de comparação por traço.
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn what_the_undo_commit_costs() {
    use std::time::Instant;

    println!(
        "\n{:<28} {:>10} {:>10} {:>10}",
        "record_structural (ms)", "1024", "2048", "4096"
    );
    for (name, impasto) in [("digital", false), ("impasto", true)] {
        let mut row = Vec::new();
        let mut snaps = Vec::new();
        for side in [1024u32, 2048, 4096] {
            let mut t = armed(side);
            if !impasto {
                t.paint.brush.impasto = false;
                for slot in &mut t.paint.brush_by_mode {
                    slot.impasto = false;
                }
            }
            stroke(&mut t, 200.0); // aquece: aloca os planos e instala o histórico
            let mut lo = f64::INFINITY;
            let mut snap_lo = 0.0f64;
            for k in 0..5u8 {
                let before = t.snapshot_model();
                let y = 300.0 + f32::from(k) * 8.0;
                t.on_canvas_pointer(cp([60.0, y], PointerPhase::Down));
                for j in 1..=6u8 {
                    t.on_canvas_pointer(cp([60.0 + f32::from(j) * 30.0, y], PointerPhase::Move));
                }
                t.paint.stroke_undo = None; // o close_stroke não commita; commitamos nós, cronometrado
                t.on_canvas_pointer(cp([260.0, y], PointerPhase::Up));
                let t1 = Instant::now();
                let after = t.snapshot_model();
                let snap = t1.elapsed().as_secs_f64() * 1000.0;
                let t0 = Instant::now();
                t.undo.record_structural(before, after);
                let dt = t0.elapsed().as_secs_f64() * 1000.0;
                if dt < lo {
                    lo = dt;
                    snap_lo = snap;
                }
            }
            row.push(lo);
            snaps.push(snap_lo);
        }
        println!(
            "{name:<28} {:>10.2} {:>10.2} {:>10.2}",
            row[0], row[1], row[2]
        );
        println!(
            "{:<28} {:>10.2} {:>10.2} {:>10.2}",
            "  (snapshot_model)", snaps[0], snaps[1], snaps[2]
        );
    }
    println!();
}

/// **O que um fork de plano custa PELA PORTA DO PRODUTO** — e é uma conferência da minha própria
/// aritmética, não uma medição nova.
///
/// A decomposição do pen-up (doc 28 §5.13) fechou os 32,8 ms que o impasto acrescenta somando
/// **`Vec::clone`** dos três planos: 0,40 + 9,94 + 18,13 = 28,47. ⚠️ Mas `Vec::clone` é um memcpy
/// **SERIAL**, e o produto não usa memcpy: usa [`super::plane_fork::fork_par`], que é **paralelo** e
/// que o gate do próprio módulo mede em **3,3× mais rápido** num plano de f32 a 4096².
///
/// Se o fork paralelo custa um terço do memcpy, então a soma que "fechou" fechou por coincidência e
/// **outros ~20 ms do pen-up estão em outro lugar**. Uma atribuição que casa com o total por acidente é
/// pior que nenhuma: ela encerra a investigação no lugar errado.
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn what_a_plane_fork_costs_through_the_products_own_door() {
    use std::sync::Arc;
    use std::time::Instant;

    fn best<T: Copy + Send + Sync>(src: &Arc<Vec<T>>, reps: u32) -> f64 {
        let mut lo = f64::INFINITY;
        for _ in 0..reps {
            let mut a = Arc::clone(src);
            let _keep = Arc::clone(src); // o segundo dono: é ele que obriga a cópia
            let t0 = Instant::now();
            let m = super::plane_fork::fork_par(&mut a);
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            std::hint::black_box(&m[0]);
            if dt < lo {
                lo = dt;
            }
        }
        lo
    }

    println!(
        "\n{:<10} {:>8} {:>12} {:>12} {:>8}",
        "plano", "MB", "fork_par ms", "memcpy ms", "razao"
    );
    for side in [2048usize, 4096] {
        let n = side * side;
        println!("-- tela {side}x{side} --");
        let mut acc = 0.0f64;

        let covers: Arc<Vec<u8>> = Arc::new(vec![7u8; n]);
        let heights: Arc<Vec<f32>> = Arc::new(vec![0.5f32; n]);
        let mats: Arc<Vec<ph2d_painter_brush::material::MaterialBytes>> =
            Arc::new(vec![[3u8; 7]; n]);

        let mut row = |name: &str, bytes: usize, par: f64, ser: f64| {
            acc += par;
            #[allow(clippy::cast_precision_loss)]
            let mb = bytes as f64 / (1024.0 * 1024.0);
            println!(
                "{name:<10} {mb:>8.0} {par:>12.3} {ser:>12.3} {:>8.2}x",
                ser / par
            );
        };
        let ser = |f: &mut dyn FnMut()| {
            let mut lo = f64::INFINITY;
            for _ in 0..5 {
                let t0 = Instant::now();
                f();
                let dt = t0.elapsed().as_secs_f64() * 1000.0;
                if dt < lo {
                    lo = dt;
                }
            }
            lo
        };
        let mut sink_u8 = Vec::new();
        let s_c = ser(&mut || sink_u8 = (*covers).clone());
        std::hint::black_box(sink_u8.len());
        let mut sink_f32 = Vec::new();
        let s_h = ser(&mut || sink_f32 = (*heights).clone());
        std::hint::black_box(sink_f32.len());
        let mut sink_m = Vec::new();
        let s_m = ser(&mut || sink_m = (*mats).clone());
        std::hint::black_box(sink_m.len());

        row("covers", n, best(&covers, 5), s_c);
        row("heights", n * 4, best(&heights, 5), s_h);
        row("mats", n * 7, best(&mats, 5), s_m);
        println!("{:<10} {:>8} {acc:>12.3}", "SOMA", "");
    }
    println!();
}
