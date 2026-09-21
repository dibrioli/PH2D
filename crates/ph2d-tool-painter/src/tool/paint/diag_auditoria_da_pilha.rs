//! AUDITORIA 2026-09-21 — **o preço e o carimbo da recomposição regional**, sobre a pilha do DONO.
//!
//! ⛔ Report do dono (com foto): *«a performance ficou ruim… pinceladas rápidas quase travam a
//! tool»* e *«temos problemas com o carimbo que fica retangular; provavelmente o principal culpado
//! é Smear»*.
//!
//! As duas sondas deste ficheiro existem porque **as réguas que já existiam não contêm o
//! fenómeno**:
//!
//! * o [`super::diag_preco_da_pilha`] mede um traço **RECTO**, que é o regime em que a premissa do
//!   módulo (*«o número de lotes replayados não cresce com o traço»*) é verdadeira;
//! * o gate [`super::composite_pilha_tests::a_ordem_e_da_pilha_e_nao_da_taxa_do_rato`] põe o Smear
//!   **no TOPO** sobre um Brush, e a pilha do dono tem-no **no FUNDO** com dois Brushes por cima.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --release --lib \
//!     diag_auditoria -- --ignored --nocapture --test-threads=1
//! ```
//!
//! ⚠️ `--release` (em debug esta crate lê `~20×` mais lento) e com `/proc/loadavg` ao lado.

use super::*;
use ph2d_editor_core::tool::RasterEditTool;

const SIZE: u32 = 1024;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn tela(raio: f32) -> PainterTool {
    tela_com(raio, 255)
}

/// ⚠️ **O `alfa` é um parâmetro porque a tela do dono está VAZIA** (as guias aparecem por baixo) —
/// e um canvas branco OPACO é o ponto neutro do alfa, onde metade das leis de composição não
/// discrimina nada.
fn tela_com(raio: f32, alfa: u8) -> PainterTool {
    let mut t = PainterTool::default();
    let mut px = vec![255u8; (SIZE * SIZE * 4) as usize];
    for p in px.chunks_exact_mut(4) {
        p[3] = alfa;
    }
    t.set_source(px, SIZE, SIZE);
    t.paint.brush.radius_px = raio;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.strength = 1.0;
    t.paint.brush.space_attenuation = false;
    t.paint.composite_enabled = true;
    for pos in 0..composite::N_CAMADAS {
        t.paint.composite[pos] = CompositeLayer::default();
    }
    t
}

/// **A pilha da foto do dono** (2026-09-21), posição a posição.
fn pilha_do_dono(t: &mut PainterTool) {
    t.paint.composite[0] = CompositeLayer {
        op: CompositeOp::Blur,
        strength: 0.362,
        size: 1.0,
        hardness: Some(0.0),
        ..CompositeLayer::default()
    };
    t.paint.composite[1] = CompositeLayer {
        op: CompositeOp::Erase,
        strength: 0.0,
        size: 0.439,
        ..CompositeLayer::default()
    };
    t.paint.composite[2] = CompositeLayer {
        op: CompositeOp::Brush,
        strength: 0.294,
        size: 1.0,
        hardness: Some(0.0),
        color: Some([1.0, 0.0, 0.0]),
        ..CompositeLayer::default()
    };
    t.paint.composite[3] = CompositeLayer {
        op: CompositeOp::Brush,
        strength: 0.290,
        size: 1.904,
        hardness: Some(0.0),
        color: Some([0.0, 0.0, 0.0]),
        ..CompositeLayer::default()
    };
    t.paint.composite[4] = CompositeLayer {
        op: CompositeOp::Smear,
        strength: 0.559,
        size: 1.577,
        hardness: Some(1.0),
        ..CompositeLayer::default()
    };
}

/// Um caminho RECTO de `n` passos de 2 px.
fn caminho_recto(n: usize) -> Vec<[f32; 2]> {
    (0..=n).map(|i| [152.0 + i as f32 * 2.0, 512.0]).collect()
}

/// Um RABISCO — o MESMO comprimento de caminho, confinado a uma caixa de 240 px.
///
/// ⭐ É esta a diferença que a sonda existe para medir: num traço recto as caixas dos lotes
/// afastam-se umas das outras; num rabisco **todas se tocam**, e a janela do replay deixa de ser
/// `~2/spacing` e passa a ser *o traço inteiro*.
fn caminho_rabisco(n: usize) -> Vec<[f32; 2]> {
    let mut v = Vec::with_capacity(n + 1);
    let (cx, cy, r) = (512.0f32, 512.0f32, 110.0f32);
    for i in 0..=n {
        // Lissajous 3:2 — passa repetidamente pela mesma vizinhança.
        let s = i as f32 * 2.0 / 90.0;
        v.push([cx + r * (3.0 * s).sin(), cy + r * (2.0 * s).cos()]);
    }
    v
}

fn corre(t: &mut PainterTool, pts: &[[f32; 2]]) {
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for p in &pts[1..] {
        t.on_canvas_pointer(cp(*p, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
}

fn conta_zerada() {
    super::composite_pilha::CONTA_DA_PILHA.with(|c| c.set((0, 0, 0)));
}
fn conta() -> (u64, u64, u64) {
    super::composite_pilha::CONTA_DA_PILHA.with(std::cell::Cell::get)
}

/// ⭐⭐⭐ **O PREÇO: recto contra rabisco.**
#[test]
#[ignore = "sonda de relógio: corre à mão, em --release e com a máquina calma"]
fn diag_auditoria_o_preco() {
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("\n  O PREÇO DA RECOMPOSIÇÃO REGIONAL   (load {})", carga.trim());
    println!("  canvas {SIZE}² · pilha do dono (Blur·Brush·Brush·Smear) · passo 2 px\n");
    println!("  caminho  | passos |     ms |  ms/ev | lotes replayados | área reescrita (Mpx)");
    println!("  ---------+--------+--------+--------+------------------+---------------------");
    for n in [60usize, 120, 240, 480] {
        for (nome, pts) in [("recto", caminho_recto(n)), ("rabisco", caminho_rabisco(n))] {
            let mut t = tela(24.0);
            pilha_do_dono(&mut t);
            conta_zerada();
            let ini = std::time::Instant::now();
            corre(&mut t, &pts);
            let ms = ini.elapsed().as_secs_f64() * 1e3;
            let (n_rec, lotes, area) = conta();
            let medio = if n_rec == 0 { 0.0 } else { lotes as f64 / n_rec as f64 };
            println!(
                "  {nome:8} | {n:6} | {ms:6.1} | {:6.2} | {lotes:6} ({medio:5.1}/lote) | {:8.1}",
                ms / n as f64,
                area as f64 / 1e6
            );
        }
    }
}

/// ⭐⭐⭐ **O CARIMBO: a rota regional contra a recomposição GLOBAL.**
///
/// O A/B corre o MESMO fluxo de eventos nas duas rotas — só o LIMITE muda. Um `|Δ|` diferente de
/// zero diz que a região recomposta **não é grande o suficiente**, e o mapa diz onde.
#[test]
#[ignore = "sonda: corre à mão, em --release"]
fn diag_auditoria_o_carimbo() {
    println!("\n  O CARIMBO — regional contra recomposição GLOBAL (|Δ| sobre 255)\n");
    println!("  caminho  | pilha                          | pior |  médio | px ≠ 0 | maior caixa");
    println!("  ---------+--------------------------------+------+--------+--------+------------");
    let casos: [(&str, fn(&mut PainterTool)); 5] = [
        ("pilha do dono (Smear no FUNDO)", pilha_do_dono),
        ("Smear no FUNDO sob 1 Brush", |t| {
            t.paint.composite[0] = CompositeLayer {
                op: CompositeOp::Brush,
                strength: 1.0,
                color: Some([1.0, 0.0, 0.0]),
                ..CompositeLayer::default()
            };
            t.paint.composite[1] = CompositeLayer {
                op: CompositeOp::Smear,
                strength: 1.0,
                ..CompositeLayer::default()
            };
        }),
        ("Smear no TOPO sobre 1 Brush", |t| {
            t.paint.composite[0] = CompositeLayer {
                op: CompositeOp::Smear,
                strength: 1.0,
                ..CompositeLayer::default()
            };
            t.paint.composite[1] = CompositeLayer {
                op: CompositeOp::Brush,
                strength: 1.0,
                color: Some([1.0, 0.0, 0.0]),
                ..CompositeLayer::default()
            };
        }),
        ("Blur no TOPO sobre 1 Brush", |t| {
            t.paint.composite[0] = CompositeLayer {
                op: CompositeOp::Blur,
                strength: 1.0,
                ..CompositeLayer::default()
            };
            t.paint.composite[1] = CompositeLayer {
                op: CompositeOp::Brush,
                strength: 1.0,
                color: Some([1.0, 0.0, 0.0]),
                ..CompositeLayer::default()
            };
        }),
        ("dois Brushes (CONTROLO)", |t| {
            t.paint.composite[0] = CompositeLayer {
                op: CompositeOp::Brush,
                strength: 1.0,
                color: Some([1.0, 0.0, 0.0]),
                ..CompositeLayer::default()
            };
            t.paint.composite[1] = CompositeLayer {
                op: CompositeOp::Brush,
                strength: 1.0,
                color: Some([0.0, 0.0, 1.0]),
                ..CompositeLayer::default()
            };
        }),
    ];
    for (nome_c, pts) in [
        ("recto", caminho_recto(120)),
        ("rabisco", caminho_rabisco(120)),
    ] {
        for (nome, monta) in casos {
            let img = |global: bool| {
                super::composite_pilha::RECOMPOSICAO_GLOBAL.with(|c| c.set(global));
                let mut t = tela(24.0);
                monta(&mut t);
                corre(&mut t, &pts);
                super::composite_pilha::RECOMPOSICAO_GLOBAL.with(|c| c.set(false));
                (*t.canvas_rgba).clone()
            };
            let a = img(true);
            let b = img(false);
            let (soma, pior, nz, caixa) = diff(&a, &b);
            println!(
                "  {nome_c:8} | {nome:30} | {pior:4} | {:6.2} | {nz:6} | {caixa}",
                soma as f64 / a.len() as f64
            );
        }
    }
}

/// `|Δ|` somado, pior, quantos bytes diferem, e a CAIXA que os contém.
fn diff(a: &[u8], b: &[u8]) -> (u64, u8, u64, String) {
    let (mut soma, mut pior, mut nz) = (0u64, 0u8, 0u64);
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    for (i, (&x, &y)) in a.iter().zip(b.iter()).enumerate() {
        let d = x.abs_diff(y);
        soma += u64::from(d);
        pior = pior.max(d);
        if d != 0 {
            nz += 1;
            let px = (i / 4) as u32 % SIZE;
            let py = (i / 4) as u32 / SIZE;
            x0 = x0.min(px);
            y0 = y0.min(py);
            x1 = x1.max(px);
            y1 = y1.max(py);
        }
    }
    let caixa = if nz == 0 {
        "—".to_string()
    } else {
        format!("{x0},{y0} {}×{}", x1 - x0 + 1, y1 - y0 + 1)
    };
    (soma, pior, nz, caixa)
}

/// Um traço RÁPIDO e caótico: o mesmo rabisco entregue **1 ponto em cada `salto`** — que é o que
/// um ponteiro faz quando a mão corre (menos amostras, cada uma um salto grande).
fn caminho_rapido(n: usize, salto: usize) -> Vec<[f32; 2]> {
    caminho_rabisco(n)
        .into_iter()
        .step_by(salto)
        .collect()
}

/// ⭐ **A FOTO** — pinta o rabisco com a pilha do dono e grava o canvas em PPM, para o defeito ser
/// OLHADO em vez de deduzido. Convém `magick <ppm> <png>` a seguir.
#[test]
#[ignore = "sonda: grava ficheiros; corre à mão"]
fn diag_auditoria_a_foto() {
    let destino = std::env::var("PH2D_AUDIT_DUMP").unwrap_or_else(|_| "/tmp".to_string());
    for raio in [40.0f32] {
        for salto in [1usize, 8, 20] {
            for global in [false, true] {
                super::composite_pilha::RECOMPOSICAO_GLOBAL.with(|c| c.set(global));
                let mut t = tela(raio);
                pilha_do_dono(&mut t);
                corre(&mut t, &caminho_rapido(240, salto));
                super::composite_pilha::RECOMPOSICAO_GLOBAL.with(|c| c.set(false));
                let nome = format!(
                    "{destino}/pilha_s{salto}_{}.ppm",
                    if global { "global" } else { "regional" }
                );
                grava_ppm(&nome, &t.canvas_rgba);
                println!("  gravado {nome}");
            }
        }
    }
}

fn grava_ppm(caminho: &str, rgba: &[u8]) {
    let mut v = format!("P6\n{SIZE} {SIZE}\n255\n").into_bytes();
    // Compor sobre branco, que é o que o ecrã mostra.
    for px in rgba.chunks_exact(4) {
        let a = f32::from(px[3]) / 255.0;
        for k in 0..3 {
            v.push((f32::from(px[k]) * a + 255.0 * (1.0 - a)).round() as u8);
        }
    }
    std::fs::write(caminho, v).unwrap();
}

/// ⭐⭐⭐ **A PREMISSA DA JANELA, para a pilha** — o gémeo do
/// [`super::measure_window_premise`]: *o rectângulo sujo CONTÉM todo pixel que mudou?*
///
/// Se não contiver, o ecrã serve estado velho ali — e a fronteira do que não subiu é **um
/// rectângulo**, que é exactamente o artefacto do report. Esta é a metade que o A/B
/// regional-contra-global **não pode** ver: aquele compara dois `canvas_rgba`, este pergunta o que
/// deles chega ao ecrã.
#[test]
#[ignore = "sonda: corre à mão"]
fn diag_auditoria_a_janela_do_ecra() {
    println!("\n  A PREMISSA DA JANELA — pixels mudados FORA do rectângulo sujo\n");
    println!("  caminho  | eventos | eventos com fuga | pior fuga (px) | maior caixa de fuga");
    println!("  ---------+---------+------------------+----------------+--------------------");
    for (nome, pts) in [
        ("recto", caminho_recto(120)),
        ("rabisco", caminho_rabisco(120)),
        ("rápido", caminho_rapido(240, 20)),
    ] {
        let mut t = tela(40.0);
        pilha_do_dono(&mut t);
        t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
        let (mut n_ev, mut com_fuga, mut pior) = (0u32, 0u32, 0u64);
        let mut caixa = String::from("—");
        for p in &pts[1..] {
            let antes = (*t.canvas_rgba).clone();
            t.dirty_rect = None;
            t.on_canvas_pointer(cp(*p, PointerPhase::Move));
            n_ev += 1;
            let sujo = t.dirty_rect;
            let (mut fuga, mut fx0, mut fy0, mut fx1, mut fy1) =
                (0u64, u32::MAX, u32::MAX, 0u32, 0u32);
            for i in (0..antes.len()).step_by(4) {
                if antes[i..i + 4] == t.canvas_rgba[i..i + 4] {
                    continue;
                }
                let (px, py) = ((i / 4) as u32 % SIZE, (i / 4) as u32 / SIZE);
                let dentro = sujo.is_some_and(|r| {
                    px >= r.x && px < r.x + r.w && py >= r.y && py < r.y + r.h
                });
                if !dentro {
                    fuga += 1;
                    fx0 = fx0.min(px);
                    fy0 = fy0.min(py);
                    fx1 = fx1.max(px);
                    fy1 = fy1.max(py);
                }
            }
            if fuga > 0 {
                com_fuga += 1;
                if fuga > pior {
                    pior = fuga;
                    caixa = format!("{fx0},{fy0} {}×{}", fx1 - fx0 + 1, fy1 - fy0 + 1);
                }
            }
        }
        t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
        println!("  {nome:8} | {n_ev:7} | {com_fuga:16} | {pior:14} | {caixa}");
    }
}

/// ⭐⭐⭐ **A TELA VAZIA** — o A/B e a foto sobre um canvas com **alfa 0**, que é o do report.
#[test]
#[ignore = "sonda: corre à mão"]
fn diag_auditoria_a_tela_vazia() {
    let destino = std::env::var("PH2D_AUDIT_DUMP").unwrap_or_else(|_| "/tmp".to_string());
    println!("\n  A TELA VAZIA (alfa 0) — regional contra GLOBAL\n");
    println!("  caminho  | alfa |  pior |  médio | px ≠ 0 | caixa da diferença");
    println!("  ---------+------+-------+--------+--------+-------------------");
    for (nome, pts) in [
        ("rabisco", caminho_rabisco(240)),
        ("rápido", caminho_rapido(240, 20)),
    ] {
        for alfa in [255u8, 0] {
            let img = |global: bool| {
                super::composite_pilha::RECOMPOSICAO_GLOBAL.with(|c| c.set(global));
                let mut t = tela_com(40.0, alfa);
                pilha_do_dono(&mut t);
                corre(&mut t, &pts);
                super::composite_pilha::RECOMPOSICAO_GLOBAL.with(|c| c.set(false));
                (*t.canvas_rgba).clone()
            };
            let a = img(true);
            let b = img(false);
            let (soma, pior, nz, caixa) = diff(&a, &b);
            println!(
                "  {nome:8} | {alfa:4} | {pior:5} | {:6.2} | {nz:6} | {caixa}",
                soma as f64 / a.len() as f64
            );
            if alfa == 0 {
                grava_ppm(&format!("{destino}/vazia_{nome}_regional.ppm"), &b);
                grava_ppm(&format!("{destino}/vazia_{nome}_global.ppm"), &a);
            }
        }
    }
}

/// A diferença **que se VÊ**: os dois lados compostos sobre branco, canal a canal.
fn diff_visivel(a: &[u8], b: &[u8]) -> (u8, f64, u64, String) {
    let sobre_branco = |px: &[u8], k: usize| {
        let al = f32::from(px[3]) / 255.0;
        f32::from(px[k]).mul_add(al, 255.0 * (1.0 - al))
    };
    let (mut soma, mut pior, mut nz) = (0f64, 0u8, 0u64);
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    for (i, (pa, pb)) in a.chunks_exact(4).zip(b.chunks_exact(4)).enumerate() {
        let mut d_px = 0u8;
        for k in 0..3 {
            let d = (sobre_branco(pa, k) - sobre_branco(pb, k)).abs().round() as u8;
            soma += f64::from(d);
            d_px = d_px.max(d);
        }
        if d_px > 0 {
            pior = pior.max(d_px);
            nz += 1;
            let (px, py) = (i as u32 % SIZE, i as u32 / SIZE);
            x0 = x0.min(px);
            y0 = y0.min(py);
            x1 = x1.max(px);
            y1 = y1.max(py);
        }
    }
    let caixa = if nz == 0 {
        "—".to_string()
    } else {
        format!("{x0},{y0} {}×{}", x1 - x0 + 1, y1 - y0 + 1)
    };
    (pior, soma / (a.len() as f64 / 4.0 * 3.0), nz, caixa)
}

/// ⭐⭐⭐ **O QUE SE VÊ** — a mesma A/B, mas com os dois lados compostos sobre branco.
#[test]
#[ignore = "sonda: corre à mão"]
fn diag_auditoria_o_que_se_ve() {
    let destino = std::env::var("PH2D_AUDIT_DUMP").unwrap_or_else(|_| "/tmp".to_string());
    println!("\n  O QUE SE VÊ — regional contra GLOBAL, composto sobre branco\n");
    println!("  caminho  | alfa | pior |  médio | px visíveis | caixa");
    println!("  ---------+------+------+--------+-------------+-------------------");
    for (nome, pts) in [
        ("rabisco", caminho_rabisco(240)),
        ("rápido", caminho_rapido(240, 20)),
    ] {
        for alfa in [255u8, 0] {
            let img = |global: bool| {
                super::composite_pilha::RECOMPOSICAO_GLOBAL.with(|c| c.set(global));
                let mut t = tela_com(40.0, alfa);
                pilha_do_dono(&mut t);
                corre(&mut t, &pts);
                super::composite_pilha::RECOMPOSICAO_GLOBAL.with(|c| c.set(false));
                (*t.canvas_rgba).clone()
            };
            let a = img(true);
            let b = img(false);
            let (pior, medio, nz, caixa) = diff_visivel(&a, &b);
            println!("  {nome:8} | {alfa:4} | {pior:4} | {medio:6.3} | {nz:11} | {caixa}");
            if alfa == 0 && nz > 0 {
                grava_ppm(&format!("{destino}/ve_{nome}_regional.ppm"), &b);
                grava_ppm(&format!("{destino}/ve_{nome}_global.ppm"), &a);
            }
        }
    }
}

/// ⭐⭐⭐ **O CONTROLO POSITIVO: cada camada da pilha do dono MOVE barro?**
///
/// ⚠️ Sem ele todo A/B deste ficheiro pode estar a comparar duas corridas de uma camada que **não
/// corre** — e duas imagens iguais leem-se como *«a lei está certa»*.
#[test]
#[ignore = "sonda: corre à mão"]
fn diag_auditoria_cada_camada_esta_viva() {
    println!("\n  CADA CAMADA ESTÁ VIVA?  (a pilha do dono, calando UMA de cada vez)\n");
    println!("  calada     | px diferentes | pior | warp.active");
    println!("  -----------+---------------+------+------------");
    let pts = caminho_rabisco(120);
    let base = {
        let mut t = tela_com(40.0, 0);
        pilha_do_dono(&mut t);
        corre(&mut t, &pts);
        (*t.canvas_rgba).clone()
    };
    for (pos, nome) in [
        (0usize, "Blur"),
        (2, "Brush verm"),
        (3, "Brush preto"),
        (4, "Smear"),
    ] {
        let mut t = tela_com(40.0, 0);
        pilha_do_dono(&mut t);
        t.paint.composite[pos].strength = 0.0;
        corre(&mut t, &pts);
        let (pior, _, nz, _) = diff_visivel(&base, &t.canvas_rgba);
        println!(
            "  {nome:10} | {nz:13} | {pior:4} | {}",
            t.paint.warp.active
        );
    }
}

/// ⭐⭐⭐ **A TELA COM ARTE** — a fixtura que o Smear precisa.
///
/// ⛔⛔ O Smear da pilha do dono está na posição de BAIXO, logo corre PRIMEIRO, sobre o `pre`. Numa
/// tela vazia não há nada para esfregar e a camada é **inerte** (medido: calá-la muda `0` pixels,
/// `warp.active = false`). O dono pinta sobre arte que já lá está — e é essa a fixtura.
fn tela_com_arte(raio: f32, alfa: u8) -> PainterTool {
    let mut t = tela_com(raio, alfa);
    // Arte de fundo: quatro traços de pincel simples, composite DESLIGADO.
    t.paint.composite_enabled = false;
    t.paint.brush.color = [0.15, 0.35, 0.75];
    for k in 0..4 {
        let y = 390.0 + k as f32 * 65.0;
        t.on_canvas_pointer(cp([380.0, y], PointerPhase::Down));
        let mut x = 380.0;
        while x < 650.0 {
            x += 6.0;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([650.0, y], PointerPhase::Up));
    }
    t.paint.brush.color = [0.0, 0.0, 0.0];
    t.paint.composite_enabled = true;
    t
}

/// ⭐⭐⭐ **O CONTROLO POSITIVO sobre a fixtura CERTA**, e o A/B regional-contra-global nela.
#[test]
#[ignore = "sonda: corre à mão"]
fn diag_auditoria_sobre_arte() {
    let destino = std::env::var("PH2D_AUDIT_DUMP").unwrap_or_else(|_| "/tmp".to_string());
    let pts = caminho_rabisco(120);
    println!("\n  (a) CADA CAMADA ESTÁ VIVA? — tela COM ARTE\n");
    println!("  calada      | px diferentes | pior | warp.active");
    println!("  ------------+---------------+------+------------");
    let base = {
        let mut t = tela_com_arte(40.0, 0);
        pilha_do_dono(&mut t);
        corre(&mut t, &pts);
        (*t.canvas_rgba).clone()
    };
    for (pos, nome) in [
        (0usize, "Blur"),
        (2, "Brush verm"),
        (3, "Brush preto"),
        (4, "Smear"),
    ] {
        let mut t = tela_com_arte(40.0, 0);
        pilha_do_dono(&mut t);
        t.paint.composite[pos].strength = 0.0;
        corre(&mut t, &pts);
        let (pior, _, nz, _) = diff_visivel(&base, &t.canvas_rgba);
        println!("  {nome:11} | {nz:13} | {pior:4} | {}", t.paint.warp.active);
    }

    println!("\n  (b) REGIONAL contra GLOBAL — o que se VÊ\n");
    println!("  caminho  | pior |  médio | px visíveis | caixa");
    println!("  ---------+------+--------+-------------+-------------------");
    for (nome, pts) in [
        ("rabisco", caminho_rabisco(120)),
        ("rápido", caminho_rapido(240, 20)),
    ] {
        let img = |global: bool| {
            super::composite_pilha::RECOMPOSICAO_GLOBAL.with(|c| c.set(global));
            let mut t = tela_com_arte(40.0, 0);
            pilha_do_dono(&mut t);
            corre(&mut t, &pts);
            super::composite_pilha::RECOMPOSICAO_GLOBAL.with(|c| c.set(false));
            (*t.canvas_rgba).clone()
        };
        let a = img(true);
        let b = img(false);
        let (pior, medio, nz, caixa) = diff_visivel(&a, &b);
        println!("  {nome:8} | {pior:4} | {medio:6.3} | {nz:11} | {caixa}");
        grava_ppm(&format!("{destino}/arte_{nome}_regional.ppm"), &b);
        grava_ppm(&format!("{destino}/arte_{nome}_global.ppm"), &a);
    }
}

/// ⭐⭐⭐ **A ATRIBUIÇÃO: qual dos dois limites do Smear é o artefacto.**
///
/// Uma ablação de cada vez, contra a recomposição GLOBAL como verdade.
#[test]
#[ignore = "sonda: corre à mão"]
fn diag_auditoria_atribuicao_do_smear() {
    println!("\n  ATRIBUIÇÃO — cada ablação contra a recomposição GLOBAL\n");
    println!("  caminho  | ablação                        | pior |  médio | px visíveis");
    println!("  ---------+--------------------------------+------+--------+------------");
    for (nome_c, pts) in [
        ("rabisco", caminho_rabisco(120)),
        ("rápido", caminho_rapido(240, 20)),
    ] {
        let img = |global: bool, sem_limite: bool, base_grande: bool| {
            super::composite_pilha::RECOMPOSICAO_GLOBAL.with(|c| c.set(global));
            super::composite_pilha::SMEAR_SEM_LIMITE.with(|c| c.set(sem_limite));
            super::composite_pilha::SMEAR_BASE_GRANDE.with(|c| c.set(base_grande));
            let mut t = tela_com_arte(40.0, 0);
            pilha_do_dono(&mut t);
            corre(&mut t, &pts);
            super::composite_pilha::RECOMPOSICAO_GLOBAL.with(|c| c.set(false));
            super::composite_pilha::SMEAR_SEM_LIMITE.with(|c| c.set(false));
            super::composite_pilha::SMEAR_BASE_GRANDE.with(|c| c.set(false));
            (*t.canvas_rgba).clone()
        };
        let verdade = img(true, false, false);
        for (nome, sl, bg) in [
            ("nenhuma (o que shipa)", false, false),
            ("só: base sobre a caixa GRANDE", false, true),
            ("só: render SEM limite", true, false),
            ("as duas", true, true),
        ] {
            let (pior, medio, nz, _) = diff_visivel(&verdade, &img(false, sl, bg));
            println!("  {nome_c:8} | {nome:30} | {pior:4} | {medio:6.3} | {nz:11}");
        }
    }
}

/// ⭐⭐⭐ **A FASE DOMINANTE** — sem ela optimiza-se a aritmética errada.
#[test]
#[ignore = "sonda de relógio: corre à mão, em --release e com a máquina calma"]
fn diag_auditoria_a_fase_dominante() {
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("\n  A FASE DOMINANTE  (load {})\n", carga.trim());
    println!(
        "  caminho  | passos |  total ms | guardar+pre |  Brush |  Smear |   Blur | restaurar"
    );
    println!(
        "  ---------+--------+-----------+-------------+--------+--------+--------+----------"
    );
    for (nome, pts) in [
        ("recto", caminho_recto(240)),
        ("rabisco", caminho_rabisco(240)),
        ("rápido", caminho_rapido(480, 20)),
    ] {
        super::composite_pilha::FASES_DA_PILHA.with(|c| c.set([0.0; 6]));
        let mut t = tela_com_arte(40.0, 0);
        pilha_do_dono(&mut t);
        super::composite_pilha::FASES_DA_PILHA.with(|c| c.set([0.0; 6]));
        let ini = std::time::Instant::now();
        corre(&mut t, &pts);
        let ms = ini.elapsed().as_secs_f64() * 1e3;
        let f = super::composite_pilha::FASES_DA_PILHA.with(std::cell::Cell::get);
        let pct = |v: f64| format!("{:5.1}%", 100.0 * v / (ms * 1e3));
        println!(
            "  {nome:8} | {:6} | {ms:9.1} | {:>11} | {:>6} | {:>6} | {:>6} | {:>9}",
            pts.len(),
            pct(f[0]),
            pct(f[1]),
            pct(f[3]),
            pct(f[4]),
            pct(f[5])
        );
    }
}

/// ⭐⭐⭐ **O TECTO DA CURA** — quanto do relógio é o REPLAY.
///
/// ⚠️ A ablação entrega a imagem ERRADA de propósito (é a ordem por LOTE de volta). O que ela mede
/// é o relógio: é o tecto de qualquer desenho que substitua o replay por ACUMULAÇÃO, que é o que o
/// [`super::stamp_color_cache::PerLayerStroke`] já faz para as camadas de Shape.
#[test]
#[ignore = "sonda de relógio: corre à mão, em --release e com a máquina calma"]
fn diag_auditoria_o_tecto_da_cura() {
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("\n  O TECTO DA CURA  (load {})\n", carga.trim());
    println!("  caminho  | passos | com replay | sem replay | ganho");
    println!("  ---------+--------+------------+------------+-------");
    for (nome, pts) in [
        ("recto", caminho_recto(240)),
        ("rabisco", caminho_rabisco(240)),
        ("rabisco+", caminho_rabisco(480)),
        ("rápido", caminho_rapido(480, 20)),
    ] {
        let corre_com = |so_novo: bool| {
            super::composite_pilha::JANELA_SO_O_NOVO.with(|c| c.set(so_novo));
            let mut t = tela_com_arte(40.0, 0);
            pilha_do_dono(&mut t);
            let ini = std::time::Instant::now();
            corre(&mut t, &pts);
            let ms = ini.elapsed().as_secs_f64() * 1e3;
            super::composite_pilha::JANELA_SO_O_NOVO.with(|c| c.set(false));
            ms
        };
        let com = corre_com(false);
        let sem = corre_com(true);
        println!(
            "  {nome:8} | {:6} | {com:10.1} | {sem:10.1} | {:5.1}×",
            pts.len(),
            com / sem
        );
    }
}
