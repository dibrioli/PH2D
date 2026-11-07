//! **A rodada única do fecho da tinta viva** (dono 06/11): o engasgo de trocar de modo, o custo dos consertos, a sombra com a
//! compressão sozinha, a compressão com várias camadas e o redo. Variantes intercaladas no mesmo processo; o resumo é o mínimo.
//! `cargo test --profile smoke -p ph2d-tool-painter regua_do_fecho -- --ignored --nocapture --test-threads=1`

use super::agua_sobre_oleo_tests::{ponteiro, preset};
use super::*;
use crate::tool::paint::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use std::time::{Duration, Instant};

const TAMANHOS: [(usize, usize); 2] = [(1920, 1080), (4096, 2560)];

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .split(' ')
        .take(3)
        .collect::<Vec<_>>()
        .join(" ")
}

fn resumo(v: &mut [f64]) -> String {
    v.sort_by(f64::total_cmp);
    format!(
        "mín {:7.1} · mediana {:7.1}",
        v.first().copied().unwrap_or(0.0),
        v.get(v.len() / 2).copied().unwrap_or(0.0)
    )
}

/// Um quadro da ponte do app: o tique e quem consome a imagem.
fn quadro(t: &mut PainterTool) {
    super::borracha_defeitos_tests::quadros(t, super::borracha_defeitos_tests::Caminho::App, 1);
}

fn traco_inteiro(t: &mut PainterTool, w: usize, y: f32) {
    t.on_canvas_pointer(ponteiro(20.0, y, PointerPhase::Down));
    for k in 1..=30 {
        let x = 20.0 + k as f32 * (w as f32 - 40.0) / 30.0;
        t.on_canvas_pointer(ponteiro(x, y, PointerPhase::Move));
        quadro(t);
    }
    t.on_canvas_pointer(ponteiro(w as f32 - 20.0, y, PointerPhase::Up));
}

/// Óleo em `passadas` traços de largura inteira, no caminho do app.
fn pintada(w: usize, h: usize, passadas: usize) -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; w * h * 4], w as u32, h as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_compor_por_quadro(true);
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o meio");
    t.set_brush_size_px(60.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    for p in 0..passadas {
        traco_inteiro(&mut t, w, (p as f32 + 0.5) * h as f32 / passadas as f32);
        t.on_tick(1000.0 / 60.0);
    }
    t
}

/// Sai do Wet, à vez. `Dorme` é o de agora, `DormeInteira` recompõe a tela inteira como antes da cura. `Acaba` é a base
/// de antes: a sessão acaba, e a seguinte só tem o traço novo. `AcabaCheia` é a mesma porta com a tela toda pintada:
/// quanto o main paga ao sair da 1.ª vez.
#[derive(Clone, Copy, Debug)]
enum Saida {
    Dorme,
    DormeInteira,
    Acaba,
    AcabaCheia,
}

/// ⭐ O engasgo do dono: trocar de modo logo depois de soltar × com a tinta assentada, contra a base. `pend` = quantas
/// amostras saíram com imagem no ar ou dívida.
#[test]
#[ignore = "régua: corre por nome com --ignored"]
fn regua_do_fecho_engasgo_de_trocar_de_modo() {
    use super::adormecer_so_o_que_falta_tests::SAI_INTEIRA;
    const NOMES: [Saida; 4] = [
        Saida::Dorme,
        Saida::DormeInteira,
        Saida::Acaba,
        Saida::AcabaCheia,
    ];
    let nova = |w: usize, h: usize, nome: &str| {
        let mut t = pintada(w, h, 10);
        t.preset_aplicar(&preset(nome)).expect("o meio");
        t.set_brush_size_px(60.0);
        t
    };
    for (w, h) in TAMANHOS {
        for preset_nome in [
            "Oil/Dabber/Dabber.ph2d-brush.png",
            "Watercolor/Fluid/Wet Round.ph2d-brush.png",
        ] {
            let mut vs = [
                nova(w, h, preset_nome),
                nova(w, h, preset_nome),
                nova(w, h, preset_nome),
            ];
            let mut res: [[Vec<f64>; 2]; 4] = Default::default();
            let mut pend = [[0usize; 2]; 4];
            for r in 0..7usize {
                for v in 0..4 {
                    let i = (v + r) % 4;
                    // A tela cheia custa uma pintura inteira por amostra: 3 rodadas chegam.
                    if i == 3 && r >= 3 {
                        continue;
                    }
                    for (a, espera) in [(0usize, 0usize), (1, 90)] {
                        let mut cheia = (i == 3).then(|| nova(w, h, preset_nome));
                        let t = match cheia.as_mut() {
                            Some(t) => t,
                            None => &mut vs[i],
                        };
                        traco_inteiro(t, w, 40.0 + 90.0 * (r + a) as f32);
                        for _ in 0..espera {
                            quadro(t);
                        }
                        pend[i][a] += usize::from(
                            t.paint
                                .wetpaint
                                .session
                                .as_ref()
                                .is_some_and(|s| s.otica.pendente()),
                        );
                        SAI_INTEIRA.set(i != 0);
                        let t0 = Instant::now();
                        if i >= 2 {
                            t.wetpaint_end_session();
                        }
                        t.set_paint_media(PaintMedia::Digital);
                        res[i][a].push(t0.elapsed().as_secs_f64() * 1e3);
                        SAI_INTEIRA.set(false);
                        t.set_paint_media(PaintMedia::WetPaint);
                        t.wet_acorda();
                        quadro(t);
                    }
                }
            }
            for (i, s) in NOMES.iter().enumerate() {
                let [ref mut logo, ref mut assente] = res[i];
                println!(
                    "[engasgo] {w}×{h} {preset_nome:.14} {s:?}: logo depois de soltar {} ms (pend {}/{}) | assentada {} ms (pend {}/{})",
                    resumo(logo),
                    pend[i][0],
                    logo.len(),
                    resumo(assente),
                    pend[i][1],
                    assente.len()
                );
            }
        }
    }
    println!("[engasgo] carga {}", carga());
}

/// O custo dos consertos no enterro, a 4K: parte a parte e com a orla desligada.
#[test]
#[ignore = "régua: corre por nome com --ignored"]
fn regua_do_fecho_custo_dos_consertos() {
    for (w, h) in TAMANHOS {
        let mut t = pintada(w, h, 10);
        for _ in 0..30 {
            quadro(&mut t);
        }
        let mut sess = t.paint.wetpaint.session.take().expect("a sessão");
        sess.bring_home();
        t.undo.write_state.vigia_a_tela(None);
        let mut com = Vec::new();
        let mut sem = Vec::new();
        let (mut planos, mut inteira, mut congela) = (Vec::new(), Vec::new(), Vec::new());
        let (gw, gh) = sess.grid;
        for r in 0..7usize {
            for v in 0..2 {
                let i = (v + r) % 2;
                let velha = Arc::clone(&t.canvas_rgba);
                {
                    let tela = Arc::make_mut(&mut t.canvas_rgba);
                    for y in 100 + 50 * r..140 + 50 * r {
                        for x in 100..1100 {
                            tela[(y * w + x) * 4] = 200 + (r + v) as u8;
                        }
                    }
                }
                super::borracha_fronteira_tests::com_degrau::set_de_fora(i == 1);
                let t0 = Instant::now();
                t.wet_enterra_o_que_mudou(&mut sess, &velha);
                let ms = t0.elapsed().as_secs_f64() * 1e3;
                if i == 0 {
                    com.push(ms);
                } else {
                    sem.push(ms);
                }
            }
            super::borracha_fronteira_tests::com_degrau::set_de_fora(false);
            let t0 = Instant::now();
            t.wet_planos_em_dia(&mut sess, (100, 100, 1100, 500.min(gh)));
            planos.push(t0.elapsed().as_secs_f64() * 1e3);
            let t0 = Instant::now();
            t.wet_planos_em_dia(&mut sess, (1, 1, gw, gh));
            inteira.push(t0.elapsed().as_secs_f64() * 1e3);
            let t0 = Instant::now();
            t.wet_congela(&mut sess, None);
            congela.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        println!(
            "[consertos] {w}×{h}: enterrar um traço {} ms | sem a orla {} ms | planos, caixa do traço {} ms | planos, grade inteira {} ms | congelar {} ms",
            resumo(&mut com),
            resumo(&mut sem),
            resumo(&mut planos),
            resumo(&mut inteira),
            resumo(&mut congela)
        );
    }
    println!("[consertos] carga {}", carga());
}

/// `VmRSS` e `VmHWM` do processo, em MB.
fn memoria() -> String {
    let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let campo = |n: &str| {
        s.lines()
            .find(|l| l.starts_with(n))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0)
            >> 10
    };
    format!("RSS {} MB, pico {} MB", campo("VmRSS"), campo("VmHWM"))
}

/// O preço da compressão com várias camadas, a 4K, e os traços de Digital com elas a dormir. Pico medido: 8,4 G;
/// consumir ou não a imagem não muda a memória (`regua_do_fecho_memoria_*`).
#[test]
#[ignore = "régua: corre por nome com --ignored"]
fn regua_do_fecho_varias_camadas() {
    const PASSADAS: usize = 6;
    let (w, h) = (4096usize, 2560usize);
    let mut t = pintada(w, h, PASSADAS);
    println!("[camadas] camada 0 pintada: {}", memoria());
    let mut trocas = Vec::new();
    for c in 1..4 {
        t.add_raster_layer(format!("camada {c}")).expect("a camada");
        let t0 = Instant::now();
        t.on_tick(1000.0 / 60.0);
        trocas.push(t0.elapsed().as_secs_f64() * 1e3);
        for p in 0..PASSADAS {
            traco_inteiro(&mut t, w, (p as f32 + 0.5) * h as f32 / PASSADAS as f32);
        }
        println!(
            "[camadas] camada {c} pintada: {} · {} adormecidas, somam {} MB",
            memoria(),
            t.paint.wetpaint.adormecidas.len(),
            t.paint
                .wetpaint
                .adormecidas
                .iter()
                .map(|a| a.bytes.load(std::sync::atomic::Ordering::Relaxed))
                .sum::<usize>()
                >> 20
        );
    }
    let t0 = Instant::now();
    assert!(t.wet_adormece());
    let adormecer = t0.elapsed().as_secs_f64() * 1e3;
    t.set_paint_media(PaintMedia::Digital);
    let t0 = Instant::now();
    loop {
        let prontas = t
            .paint
            .wetpaint
            .adormecidas
            .iter()
            .all(|a| a.sess.lock().expect("o trinco").dormente.is_some());
        if prontas || t0.elapsed() > Duration::from_secs(120) {
            println!(
                "[camadas] {} adormecidas, comprimidas {prontas} em {:.0} ms depois de adormecer ({adormecer:.0} ms); somam {} MB",
                t.paint.wetpaint.adormecidas.len(),
                t0.elapsed().as_secs_f64() * 1e3,
                t.paint
                    .wetpaint
                    .adormecidas
                    .iter()
                    .map(|a| a.bytes.load(std::sync::atomic::Ordering::Relaxed))
                    .sum::<usize>()
                    >> 20
            );
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    println!("[camadas] criar a camada e o quadro seguinte: {trocas:.1?} ms");
    // Digital com as camadas a dormir × numa tela sem nada, intercalado.
    let mut vazia = PainterTool::default();
    vazia.abre_a_sprite(vec![255u8; w * h * 4], w as u32, h as u32);
    vazia.set_paint_media(PaintMedia::Digital);
    for f in [&mut t, &mut vazia] {
        f.set_brush_size_px(60.0);
    }
    let (mut a, mut b) = (Vec::new(), Vec::new());
    for r in 0..9usize {
        for v in 0..2 {
            let i = (v + r) % 2;
            let f = if i == 0 { &mut t } else { &mut vazia };
            let t0 = Instant::now();
            f.on_canvas_pointer(ponteiro(20.0, 50.0 + 20.0 * r as f32, PointerPhase::Down));
            for k in 1..=20 {
                f.on_canvas_pointer(ponteiro(
                    20.0 + 40.0 * k as f32,
                    50.0 + 20.0 * r as f32,
                    PointerPhase::Move,
                ));
                f.on_tick(1000.0 / 60.0);
            }
            f.on_canvas_pointer(ponteiro(820.0, 50.0 + 20.0 * r as f32, PointerPhase::Up));
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            if i == 0 { a.push(ms) } else { b.push(ms) }
        }
    }
    println!(
        "[camadas] traço Digital com {} camadas a dormir: {} ms | tela sem tinta: {} ms",
        t.paint.wetpaint.adormecidas.len(),
        resumo(&mut a),
        resumo(&mut b)
    );
    t.set_paint_media(PaintMedia::WetPaint);
    let t0 = Instant::now();
    t.wet_acorda();
    println!(
        "[camadas] acordar a da camada de cima com o pen-down: {:.0} ms · carga {}",
        t0.elapsed().as_secs_f64() * 1e3,
        carga()
    );
}

/// Desfazer e refazer a tinta viva, a tempo.
#[test]
#[ignore = "régua: corre por nome com --ignored"]
fn regua_do_fecho_desfazer_e_refazer() {
    for (w, h) in TAMANHOS {
        let mut t = pintada(w, h, 10);
        let (mut d, mut f) = (Vec::new(), Vec::new());
        for _ in 0..5 {
            for _ in 0..3 {
                let t0 = Instant::now();
                assert!(t.undo_last());
                d.push(t0.elapsed().as_secs_f64() * 1e3);
                assert!(t.paint.wetpaint.session.is_some(), "o desfazer");
                quadro(&mut t);
            }
            for _ in 0..3 {
                let t0 = Instant::now();
                assert!(t.redo_last());
                f.push(t0.elapsed().as_secs_f64() * 1e3);
                assert!(t.paint.wetpaint.session.is_some(), "o refazer");
                quadro(&mut t);
            }
        }
        println!(
            "[redo] {w}×{h}: desfazer {} ms | refazer {} ms",
            resumo(&mut d),
            resumo(&mut f)
        );
    }
    println!("[redo] carga {}", carga());
}

/// Porque é que a imagem continua no ar 90 quadros depois de soltar: quadro a quadro, antes e depois de ir e voltar.
#[test]
#[ignore = "sonda: corre por nome com --ignored"]
fn sonda_do_fecho_pendente_depois_de_assentar() {
    let (w, h) = (1280usize, 800usize);
    let mut t = pintada(w, h, 4);
    for vida in ["1.ª vida", "depois de ir e voltar"] {
        traco_inteiro(&mut t, w, 60.0);
        let mut linha = String::new();
        for q in 0..240 {
            quadro(&mut t);
            if q % 30 == 29 {
                let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
                linha += &format!(
                    " q{q}: pend {} dív {:?} passos {}/{};",
                    s.otica.pendente(),
                    s.otica.divida,
                    s.sim_steps(),
                    s.seen_steps
                );
            }
        }
        println!("[pendente] {vida}:{linha}");
        t.set_paint_media(PaintMedia::Digital);
        t.set_paint_media(PaintMedia::WetPaint);
        t.wet_acorda();
        quadro(&mut t);
    }
}

/// Wet Round a 4K, só ele (a régua geral estourou 1200 s): 4 passadas em vez de 10, 3 amostras em vez de 7, a água sem os
/// 90 quadros de espera. `caixa` = a parte da grade que a saída compõe.
#[test]
#[ignore = "régua: corre por nome com --ignored"]
fn regua_do_fecho_wet_round_a_4k() {
    use super::composite_contagem as contagem;
    let nome = "Watercolor/Fluid/Wet Round.ph2d-brush.png";
    for (w, h) in TAMANHOS {
        let nova = || {
            let mut t = pintada(w, h, 4);
            t.preset_aplicar(&preset(nome)).expect("o meio");
            t.set_brush_size_px(60.0);
            t
        };
        let mut viva = nova();
        let (mut dorme, mut acaba, mut caixa) = (Vec::new(), Vec::new(), Vec::new());
        let (gw, gh) = viva.paint.wetpaint.session.as_ref().expect("sessão").grid;
        for r in 0..3usize {
            traco_inteiro(&mut viva, w, 40.0 + 90.0 * r as f32);
            let antes = Arc::clone(&viva.canvas_rgba);
            let _ = contagem::take();
            let t0 = Instant::now();
            viva.set_paint_media(PaintMedia::Digital);
            dorme.push(t0.elapsed().as_secs_f64() * 1e3);
            let (mudou, delta) = antes.chunks(4).zip(viva.canvas_rgba.chunks(4)).fold(
                (0usize, 0u8),
                |(n, d), (p, q)| {
                    let m = (0..4).map(|k| p[k].abs_diff(q[k])).max().unwrap_or(0);
                    (n + usize::from(m > 0), d.max(m))
                },
            );
            println!("[wet round] {w}×{h} r{r}: a saída mudou {mudou} px, Δ máx {delta}");
            caixa.push(contagem::take().2 as f64 * 100.0 / (gw * gh) as f64);
            viva.set_paint_media(PaintMedia::WetPaint);
            viva.wet_acorda();
            quadro(&mut viva);
            if r < 2 {
                let mut cheia = nova();
                traco_inteiro(&mut cheia, w, 40.0 + 90.0 * r as f32);
                let t0 = Instant::now();
                cheia.wetpaint_end_session();
                cheia.set_paint_media(PaintMedia::Digital);
                acaba.push(t0.elapsed().as_secs_f64() * 1e3);
            }
        }
        println!(
            "[wet round] {w}×{h}: Dorme {} ms (caixa {caixa:.0?} % da grade) | AcabaCheia {} ms · carga {}",
            resumo(&mut dorme),
            resumo(&mut acaba),
            carga()
        );
    }
}

/// O controlo: sem ninguém consumir a imagem (só `on_tick`), a memória sobe? `consome` = a ponte do app, a régua de cá.
fn memoria_por_traco(consome: bool) {
    let (w, h) = (4096usize, 2560usize);
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; w * h * 4], w as u32, h as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_compor_por_quadro(true);
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o meio");
    t.set_brush_size_px(60.0);
    for p in 0..12 {
        let y = (p as f32 + 0.5) * h as f32 / 12.0;
        t.on_canvas_pointer(ponteiro(20.0, y, PointerPhase::Down));
        for k in 1..=30 {
            let x = 20.0 + k as f32 * (w as f32 - 40.0) / 30.0;
            t.on_canvas_pointer(ponteiro(x, y, PointerPhase::Move));
            std::thread::sleep(Duration::from_millis(16));
            t.on_tick(1000.0 / 60.0);
            if consome {
                let _ = t.take_preview_arc();
            }
        }
        t.on_canvas_pointer(ponteiro(w as f32 - 20.0, y, PointerPhase::Up));
        println!("[memória] consome {consome}, traço {p}: {}", memoria());
    }
}

#[test]
#[ignore = "régua: corre por nome com --ignored"]
fn regua_do_fecho_memoria_sem_consumir() {
    memoria_por_traco(false);
}

#[test]
#[ignore = "régua: corre por nome com --ignored"]
fn regua_do_fecho_memoria_consumindo() {
    memoria_por_traco(true);
}

/// ⭐ O desfazer para lá do tecto, a 4K: 10 traços de largura inteira, e o desfazer de um por um. Também o desfazer com
/// fotografia numa sessão que já enterrou (um Fill antes), e o Dry Canvas com o registo aberto.
#[test]
#[ignore = "régua: corre por nome com --ignored"]
fn regua_do_fecho_desfazer_para_la_do_tecto() {
    use super::undo_tinta_morre_tests::teto;
    let (w, h) = (4096usize, 2560usize);
    println!("[desfazer] carga antes {}", carga());
    teto::set(Some(1));
    let mut t = pintada(w, h, 10);
    println!("[desfazer] pintada, carga {}", carga());
    use super::undo_auditoria_tests::grade_inteira;
    // Variantes alternadas no mesmo processo, ABBA: 0 = caixa, 1 = a grade inteira de antes.
    let abba = |i: usize| (i + 1) % 4 < 2;
    let mut sem_passo = [Vec::new(), Vec::new()];
    for i in 0..9 {
        grade_inteira::set(abba(i));
        let t0 = Instant::now();
        assert!(t.undo_last());
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        println!("[desfazer] tecto {i} inteira {}: {ms:7.1} ms", abba(i));
        if i > 0 {
            sem_passo[usize::from(abba(i))].push(ms);
        }
        quadro(&mut t);
    }
    teto::set(None);
    println!(
        "[desfazer] para lá do tecto: caixa {} · inteira {} · carga {}",
        resumo(&mut sem_passo[0]),
        resumo(&mut sem_passo[1]),
        carga()
    );

    grade_inteira::set(false);
    let mut t = pintada(w, h, 4);
    t.set_rect_selection(100, 100, 200, 200);
    t.selection_color_fill();
    t.on_tick(1000.0 / 60.0);
    t.clear_selection();
    for p in 0..8 {
        traco_inteiro(&mut t, w, 200.0 + p as f32 * 280.0);
        t.on_tick(1000.0 / 60.0);
    }
    let mut foto = [Vec::new(), Vec::new()];
    for i in 0..8 {
        grade_inteira::set(abba(i));
        let t0 = Instant::now();
        assert!(t.undo_last());
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        println!("[desfazer] foto {i} inteira {}: {ms:7.1} ms", abba(i));
        foto[usize::from(abba(i))].push(ms);
        quadro(&mut t);
    }
    grade_inteira::set(false);
    println!(
        "[desfazer] com fotografia, já enterrou: caixa {} · inteira {} · carga {}",
        resumo(&mut foto[0]),
        resumo(&mut foto[1]),
        carga()
    );

    traco_inteiro(&mut t, w, 300.0);
    t.wet_bring_home();
    let bytes = |t: &PainterTool| {
        t.paint
            .wetpaint
            .session
            .as_ref()
            .map_or(0, |s| s.engine.bytes_do_historico(0))
    };
    let antes = bytes(&t);
    let t0 = Instant::now();
    t.wetpaint_dry_canvas();
    println!(
        "[desfazer] Dry Canvas: {:7.1} ms · registos {} → {} MB",
        t0.elapsed().as_secs_f64() * 1e3,
        antes >> 20,
        bytes(&t) >> 20
    );
}
