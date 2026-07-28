//! **DE QUE É FEITO O CUSTO de um traço** — irmão de [`super::measure_stroke_owners`] (que responde
//! *quem SEGURA os planos*), split pelo cap de LOC e pela linha de corte natural: uma coisa é a
//! POSSE, outra é o PREÇO.
//!
//! As sondas daqui medem sempre **pelas portas do produto** e ablacionam **pela entrada** — nunca
//! instrumentando o laço, porque uma sonda que re-implementa o laço fica cega à porta e segue
//! imprimindo o custo antigo depois de o produto parar de pagá-lo (doc 28 §4.8.2).

use super::measure_stroke_owners::{armed, cp, stroke};
use super::*;

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
            let m =
                super::plane_fork::fork_par(&mut a, &crate::undo::window::WindowCell::default());
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

/// **O que a porta única comprou, medido no PRODUTO** — num gesto BARATO, de propósito.
///
/// Até 2026-07-26 só o depósito de pigmento vinha pela rota paralela; os outros **23 sítios** (fill,
/// smear, blur, clone, seleção, warp, máscara, inpaint, aquarela, e o composite do Wet Paint, que roda a
/// cada TICK) forkavam **serialmente**. E a primeira escrita de todo gesto forka, porque em repouso o
/// canvas tem dois donos — o que a sonda de donos deste arquivo mediu.
///
/// ⚠️ **DUAS fixtures anteriores não conseguiam ver o fork, por motivos diferentes, e as duas ficam
/// escritas:**
///
/// 1. **Um FILL** custa ~130 ms por conta própria a 4096², e a variação entre corridas dele é MAIOR que
///    os ~6 ms do fork: a diferença saía **negativa**. *Um sinal só é mensurável contra um fundo menor
///    que ele.*
/// 2. **Ablar o HISTÓRICO** (`undo.clear()`) não remove o segundo dono de um traço: o `stroke_undo`
///    nasce DENTRO do `paint_begin`, então os dois braços forkam e a diferença é zero. A ablação certa
///    é trocar a ROTA no mesmo gesto.
///
/// Medido assim, no pen-down de um **Blur** (dab limitado pela pegada, ~1 ms de fundo):
///
/// | blur pen-down | 2048² | 4096² |
/// |---|---|---|
/// | `Arc::make_mut` serial | 1,11 | **11,64** |
/// | `fork_par` paralelo | 0,85 | **3,66** |
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn what_the_single_door_bought_a_non_pigment_gesture() {
    use std::time::Instant;

    fn blur_down_ms(side: u32) -> f64 {
        let mut v = Vec::new();
        for _ in 0..7 {
            let mut t = armed(side);
            t.paint.brush.impasto = false;
            for slot in &mut t.paint.brush_by_mode {
                slot.impasto = false;
            }
            stroke(&mut t, 200.0); // um traço commitado antes: o histórico passa a segurar o canvas
            t.set_paint_tool_mode("blur");
            let t0 = Instant::now();
            t.on_canvas_pointer(cp([500.0, 500.0], PointerPhase::Down));
            v.push(t0.elapsed().as_secs_f64() * 1000.0);
        }
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    }

    println!(
        "\n{:<26} {:>10} {:>10}",
        "blur pen-down (ms)", "2048", "4096"
    );
    println!(
        "{:<26} {:>10.2} {:>10.2}\n",
        "pela porta",
        blur_down_ms(2048),
        blur_down_ms(4096)
    );
}

/// **DE QUE SÃO FEITOS OS 12,2 ms QUE O S2 DEIXOU ABERTOS** — e a resposta não estava no lugar previsto.
///
/// O S2 fechou em *"commit 23,7 → 12,2 ms; os 12,2 são extração dos dois lados da janela **+ planos que
/// ninguém declarou**"* (doc 28 §5.19). A segunda metade dessa frase é **falsa** e a rede de auditoria
/// a derruba numa linha: com `PH2D_UNDO_AUDIT=1`, todo commit de traço imprime
/// `nao-declarados=0` e uma janela de 338×156 sobre 1024² — a janela **é** oferecida, e o `split`
/// retorna antes de varrer.
///
/// Esta sonda mede o que sobra, ablacionando pela porta: `WriteWindow::open_write` deixa um acesso
/// aberto, e um acesso aberto faz `hint_for` devolver `None` — **é exatamente o que um sítio esquecido
/// faz**, então o braço "sem janela" é o produto pré-S2, não um harness.
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn what_the_hinted_commit_is_made_of() {
    use std::time::Instant;

    // O pen-up com o fold JÁ PAGO (`commit_stroke_height` chamado antes), então o que se cronometra é o
    // `close_stroke` → `commit_structural_edit`, e nada mais.
    fn commit_ms(side: u32, kill_hint: bool, pin_old_generation: bool) -> f64 {
        let mut t = armed(side);
        let mut v = Vec::new();
        let mut pinned = Vec::new();
        for k in 0..8u8 {
            let y = 200.0 + f32::from(k) * 8.0;
            t.on_canvas_pointer(cp([60.0, y], PointerPhase::Down));
            if pin_old_generation {
                // Os planos de relevo ainda são os da geração ANTERIOR (o fold só os forka abaixo).
                // Segurá-los impede que o commit seja o último dono — se o custo cair, ele era o `free`.
                pinned.push((t.heights.clone(), t.covers.clone(), t.mats.clone()));
            }
            for j in 1..=6u8 {
                t.on_canvas_pointer(cp([60.0 + f32::from(j) * 30.0, y], PointerPhase::Move));
            }
            t.commit_stroke_height(); // a metade do fold, fora do relógio
            if kill_hint {
                // A ablação: um acesso de escrita que nunca declara — o sítio esquecido, encenado.
                let mut w = t.undo_window.get();
                w.open_write();
                t.undo_window.set(w);
            }
            let t0 = Instant::now();
            t.on_canvas_pointer(cp([260.0, y], PointerPhase::Up));
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            if k > 0 {
                v.push(dt);
            }
        }
        v.sort_by(f64::total_cmp);
        std::hint::black_box(&pinned);
        v[v.len() / 2]
    }

    println!(
        "\n{:<34} {:>10} {:>10} {:>10}",
        "commit de undo, impasto (ms)", "1024", "2048", "4096"
    );
    let row = |name: &str, kill: bool, pin: bool| {
        let v: Vec<f64> = [1024u32, 2048, 4096]
            .into_iter()
            .map(|s| commit_ms(s, kill, pin))
            .collect();
        println!("{name:<34} {:>10.2} {:>10.2} {:>10.2}", v[0], v[1], v[2]);
        v
    };
    let hinted = row("com a JANELA declarada", false, false);
    let scanned = row("VARRENDO (um sitio esqueceu)", true, false);
    let pinned = row("com a janela, geracao velha PINADA", false, true);
    println!(
        "{:<34} {:>10.2} {:>10.2} {:>10.2}",
        "o que a janela poupa",
        scanned[0] - hinted[0],
        scanned[1] - hinted[1],
        scanned[2] - hinted[2]
    );
    println!(
        "{:<34} {:>10.2} {:>10.2} {:>10.2}",
        "o que o `free` custa",
        hinted[0] - pinned[0],
        hinted[1] - pinned[1],
        hinted[2] - pinned[2]
    );

    // ⚠️ **Segunda testemunha, e ela não passa pelo produto.** A ablação por pinagem mede a diferença
    // entre dois braços do MESMO caminho; esta linha mede a grandeza sozinha — soltar os três planos de
    // relevo de uma tela. Se as duas concordarem em ORDEM, a atribuição é boa; se não, a pinagem estava
    // medindo outra coisa (pressão de alocador, reuso de páginas) e a conclusão cai.
    let free_only = |side: usize| -> f64 {
        let n = side * side;
        let mut lo = f64::INFINITY;
        for _ in 0..5 {
            let h: Vec<f32> = vec![0.5; n];
            let c: Vec<u8> = vec![7; n];
            let m: Vec<[u8; 7]> = vec![[3; 7]; n];
            std::hint::black_box((h.len(), c.len(), m.len()));
            let t0 = Instant::now();
            drop(h);
            drop(c);
            drop(m);
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            if dt < lo {
                lo = dt;
            }
        }
        lo
    };
    println!(
        "{:<34} {:>10.2} {:>10.2} {:>10.2}\n",
        "  (2a testemunha: `drop` dos 3 planos)",
        free_only(1024),
        free_only(2048),
        free_only(4096)
    );
}

/// **AS DUAS METADES DO PEN-UP, pelas portas do produto** — a ablação diz o TOTAL do histórico
/// (27,9 ms a 4096²) e o `record_structural` isolado diz 11,7: faltavam ~16 ms, e esta sonda os nomeia.
///
/// ⚠️ **A ablação tira duas coisas de uma vez** (o commit estrutural **e** o segundo dono que o snapshot
/// representa), então a diferença dela NÃO é o commit. Aqui as duas metades são cronometradas
/// separadamente **com o snapshot VIVO**, chamando as funções do produto na ordem do produto:
/// `commit_stroke_height` e depois o `Up` (cujo `close_stroke` re-chama o fold — no-op, ele já drenou o
/// `stroke_paint` — e roda o `commit_structural_edit`).
///
/// Isto não re-implementa laço nenhum: são as duas chamadas que o `close_stroke` faz, na ordem dele.
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn what_the_two_halves_of_the_pen_up_cost() {
    use std::time::Instant;

    fn halves(side: u32) -> (f64, f64) {
        let mut t = armed(side);
        let (mut fold, mut rest) = (Vec::new(), Vec::new());
        for k in 0..8u8 {
            let y = 200.0 + f32::from(k) * 8.0;
            t.on_canvas_pointer(cp([60.0, y], PointerPhase::Down));
            for j in 1..=6u8 {
                t.on_canvas_pointer(cp([60.0 + f32::from(j) * 30.0, y], PointerPhase::Move));
            }
            // Metade 1: o fold do relevo, com o `stroke_undo` do pen-down ainda VIVO (o segundo dono).
            let t0 = Instant::now();
            t.commit_stroke_height();
            let a = t0.elapsed().as_secs_f64() * 1000.0;
            // Metade 2: o resto do pen-up — e o fold que o `close_stroke` re-chama já não tem trabalho.
            let t1 = Instant::now();
            t.on_canvas_pointer(cp([260.0, y], PointerPhase::Up));
            let b = t1.elapsed().as_secs_f64() * 1000.0;
            if k > 0 {
                fold.push(a);
                rest.push(b);
            }
        }
        fold.sort_by(f64::total_cmp);
        rest.sort_by(f64::total_cmp);
        (fold[fold.len() / 2], rest[rest.len() / 2])
    }

    println!(
        "\n{:<28} {:>10} {:>10} {:>10}",
        "pen-up impasto (ms)", "1024", "2048", "4096"
    );
    let (f1, r1) = halves(1024);
    let (f2, r2) = halves(2048);
    let (f4, r4) = halves(4096);
    println!("commit_stroke_height (fold)  {f1:>10.2} {f2:>10.2} {f4:>10.2}");
    println!("o resto (commit de undo)     {r1:>10.2} {r2:>10.2} {r4:>10.2}");
    println!(
        "TOTAL                        {:>10.2} {:>10.2} {:>10.2}\n",
        f1 + r1,
        f2 + r2,
        f4 + r4
    );
}

/// **DE QUE É FEITO o `record_structural`** — os scans somam 4,0 ms a 4096² e ele custa 11,7: a sonda
/// abre a diferença em vez de a atribuir por subtração.
///
/// Cronometra, sobre os MESMOS dois endpoints: (a) o `PlaneDeltas::split` inteiro — a porta que varre e
/// extrai; (b) o `record_structural` completo, que o embrulha. O que sobrar entre (a) e (b) é a
/// contabilidade do controller (cursor, cap, pilhas).
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn what_the_record_structural_is_made_of() {
    use std::time::Instant;

    println!(
        "\n{:<30} {:>10} {:>10} {:>10}",
        "record_structural, impasto (ms)", "1024", "2048", "4096"
    );
    let (mut split, mut whole) = (Vec::new(), Vec::new());
    for side in [1024u32, 2048, 4096] {
        let mut t = armed(side);
        stroke(&mut t, 200.0);
        let (mut lo_s, mut lo_w) = (f64::INFINITY, f64::INFINITY);
        for k in 0..5u8 {
            let before = t.snapshot_model();
            let y = 300.0 + f32::from(k) * 8.0;
            t.on_canvas_pointer(cp([60.0, y], PointerPhase::Down));
            for j in 1..=6u8 {
                t.on_canvas_pointer(cp([60.0 + f32::from(j) * 30.0, y], PointerPhase::Move));
            }
            t.paint.stroke_undo = None;
            t.on_canvas_pointer(cp([260.0, y], PointerPhase::Up));
            let after = t.snapshot_model();

            // (a) só o motor de delta, sobre CÓPIAS dos endpoints (o split esvazia o que recebe).
            let (mut b, mut a) = (before.clone(), after.clone());
            let t0 = Instant::now();
            let d = crate::undo_planes::PlaneDeltas::split(&mut b, &mut a, None);
            let s = t0.elapsed().as_secs_f64() * 1000.0;
            std::hint::black_box(d.heap_bytes());

            // (b) o commit inteiro, pela porta real.
            let t1 = Instant::now();
            t.undo.record_structural(before, after);
            let w = t1.elapsed().as_secs_f64() * 1000.0;
            if s < lo_s {
                lo_s = s;
            }
            if w < lo_w {
                lo_w = w;
            }
        }
        split.push(lo_s);
        whole.push(lo_w);
    }
    println!(
        "{:<30} {:>10.2} {:>10.2} {:>10.2}",
        "PlaneDeltas::split", split[0], split[1], split[2]
    );
    println!(
        "{:<30} {:>10.2} {:>10.2} {:>10.2}",
        "record_structural (total)", whole[0], whole[1], whole[2]
    );
    println!();
}

/// **O QUE UM CTRL+Z CUSTA** — o outro lado do delta, e o preço que a U1 nomeou (0,43 ms a 2048² ·
/// **13,37 a 4096²**).
///
/// A materialização de um `Patch` começa **clonando o plano do cursor** (é ele que serve tudo fora da
/// janela), então um undo carrega uma cópia de documento por plano que a entrada tocou. A cópia é a mesma
/// que a porta de fork faz, e agora vai pelo mesmo primitivo paralelo (`crate::plane_copy`).
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn what_an_undo_costs() {
    use std::time::Instant;

    fn undo_ms(side: u32) -> f64 {
        let mut t = armed(side);
        let mut v = Vec::new();
        for k in 0..9u8 {
            stroke(&mut t, 200.0 + f32::from(k) * 8.0);
        }
        for k in 0..8u8 {
            let t0 = Instant::now();
            let ok = t.undo_last();
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            assert!(ok, "havia o que desfazer");
            if k > 0 {
                v.push(dt);
            }
        }
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    }

    println!(
        "\n{:<24} {:>10} {:>10} {:>10}",
        "undo (mediana, ms)", "1024", "2048", "4096"
    );
    println!(
        "{:<24} {:>10.2} {:>10.2} {:>10.2}\n",
        "impasto",
        undo_ms(1024),
        undo_ms(2048),
        undo_ms(4096)
    );
}
