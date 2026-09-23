//! DIAGNÓSTICO — **o custo da pilha é por EVENTO do rato, não por píxel do traço?**
//!
//! Report do dono (2026-09-23, cena `PH2D_COMPOSITE_SMOKE`): *«FPS cai para 1»*. Parado, o app lê
//! `60 fps` (medido com `PH2D_PAINT_PERF=1`); a queda é só a pintar. A sonda irmã
//! (`diag_preco_da_pilha`) corre o traço em passos de `2 px` e lê `~0,7 ms` por evento — mas um rato
//! entrega eventos a uma TAXA (125 Hz num rato comum, 1000 Hz num de jogo), não a uma distância, e
//! a shell entrega cada `CursorMoved` do traço livre ao pincel sem os juntar
//! (`painter_canvas_input.rs`). Se a composição da região custa o mesmo com o rato a andar `0,5 px`
//! ou `16 px`, o preço por SEGUNDO é `taxa × custo`, e acima de `1 ms` por evento um rato de
//! `1 kHz` pede mais CPU do que o segundo tem.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --release --lib \
//!     diag_passo_do_rato -- --ignored --nocapture --test-threads=1
//! ```
//!
//! ⛔⛔ **Estas sondas medem o build de TESTE, e ele não é o produto** (achado de 2026-09-23): sob
//! `cfg(test)` o traço paga o journal do undo do canvas e a espia do esfregão (`8 MB` copiados por
//! lote), que o app nunca paga. Elas servem para PARTIR o custo em fases e operações; **o número do
//! produto sai do exemplo `examples/mede_a_pilha.rs`**, que compila a biblioteca sem `cfg(test)`.

use super::*;
use ph2d_editor_core::tool::RasterEditTool;

const SIZE: u32 = 1024;
const Y: f32 = 512.0;
const X0: f32 = 152.0;
const X1: f32 = 872.0;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A pilha da foto do dono (a mesma da cena), no pincel da cena (`Size 0,4`).
fn pilha_do_dono() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    t.paint.brush.radius_px = super::brush_settings::size_norm_to_px(0.4);
    t.paint.brush.strength = 1.0;
    t.paint.brush.space_attenuation = false;
    t.paint.composite_enabled = true;
    for pos in 0..composite::N_CAMADAS {
        t.paint.composite[pos].strength = 0.0;
    }
    let dono = [
        (CompositeOp::Blur, 1.0f32, 2.048f32),
        (CompositeOp::Brush, 0.133, 0.574),
        (CompositeOp::Brush, 0.204, 1.002),
        (CompositeOp::Brush, 0.176, 1.221),
        (CompositeOp::Smear, 0.596, 1.0),
        (CompositeOp::Erase, 0.104, 1.0),
    ];
    t.paint.composite_len = dono.len();
    for (i, &(op, strength, size)) in dono.iter().enumerate() {
        t.paint.composite[i] = CompositeLayer {
            op,
            strength,
            size,
            ..CompositeLayer::default()
        };
    }
    t
}

/// O traço de `X0` a `X1` com o rato a andar `passo` px por evento, drenando a pré-visualização a
/// cada `drena` eventos (`0` = nunca) — o que a ponte do app faz uma vez por quadro. Devolve
/// (ms, eventos).
fn traco(t: &mut PainterTool, passo: f32, drena: u32) -> (f64, u32) {
    let ini = std::time::Instant::now();
    t.on_canvas_pointer(cp([X0, Y], PointerPhase::Down));
    let (mut x, mut n) = (X0, 0u32);
    while x < X1 {
        x += passo;
        t.on_canvas_pointer(cp([x.min(X1), Y], PointerPhase::Move));
        n += 1;
        if drena > 0 && n.is_multiple_of(drena) {
            let _ = t.take_preview_arc();
        }
    }
    t.on_canvas_pointer(cp([X1, Y], PointerPhase::Up));
    (ini.elapsed().as_secs_f64() * 1e3, n)
}

#[test]
#[ignore = "sonda de relógio: corre à mão, em --release e com a máquina calma"]
fn diag_passo_do_rato() {
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("\n  O PASSO DO RATO   (load {})", carga.trim());
    println!("  pilha do dono · canvas {SIZE}² · traço de {} px", X1 - X0);
    println!("  passo px | eventos |     ms | ms/evento | CPU pedida a 125 Hz | a 1000 Hz");
    println!("  ---------+---------+--------+-----------+---------------------+----------");
    for passo in [0.5f32, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0] {
        let mut melhor = (f64::MAX, 0);
        for _ in 0..3 {
            let mut t = pilha_do_dono();
            let r = traco(&mut t, passo, 0);
            if r.0 < melhor.0 {
                melhor = r;
            }
        }
        let (ms, n) = melhor;
        let por_ev = ms / f64::from(n);
        // ⭐ «CPU pedida»: o custo de UM segundo de eventos àquela taxa, em segundos. `> 100 %` é
        //    um rato que pede mais trabalho do que o segundo tem — o app deixa de acompanhar.
        println!(
            "  {passo:8.1} | {n:7} | {ms:6.1} | {por_ev:9.3} | {:18.0} % | {:7.0} %",
            por_ev * 125.0 / 10.0,
            por_ev * 1000.0 / 10.0
        );
    }
}

/// ⭐ **O MESMO traço com a composição por QUADRO** ([`super::composite_por_quadro`]): a ponte drena
/// uma vez por quadro, e um rato de `1 kHz` a `60 fps` entrega `16` eventos entre duas drenagens.
/// A coluna de antes é a mesma pilha a compor em cada evento — ALTERNADAS na mesma corrida, para a
/// contenção da máquina cair sobre os dois lados por igual.
#[test]
#[ignore = "sonda de relógio: corre à mão, em --release e com a máquina calma"]
fn diag_a_composicao_por_quadro() {
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("\n  A COMPOSIÇÃO POR QUADRO   (load {})", carga.trim());
    println!(
        "  pilha do dono · canvas {SIZE}² · traço de {} px · drena a cada 16 eventos",
        X1 - X0
    );
    println!(
        "  passo px | por evento ms | por quadro ms | ganho | CPU pedida a 1 kHz (antes → depois)"
    );
    println!(
        "  ---------+---------------+---------------+-------+------------------------------------"
    );
    for passo in [2.0f32, 4.0, 8.0, 16.0] {
        let (mut antes, mut depois, mut n) = (f64::MAX, f64::MAX, 0u32);
        for _ in 0..3 {
            let mut t = pilha_do_dono();
            t.set_compor_por_quadro(false);
            let (ms, ev) = traco(&mut t, passo, 16);
            antes = antes.min(ms);
            n = ev;
            let mut t = pilha_do_dono();
            t.set_compor_por_quadro(true);
            depois = depois.min(traco(&mut t, passo, 16).0);
        }
        let cpu = |ms: f64| ms / f64::from(n) * 1000.0 / 10.0;
        println!(
            "  {passo:8.1} | {antes:13.1} | {depois:13.1} | {:4.1}× | {:9.0} % → {:.0} %",
            antes / depois,
            cpu(antes),
            cpu(depois)
        );
    }
}

/// As fases do traço nas duas rotas — o que SOBRA quando a composição passa a uma por quadro.
#[test]
#[ignore = "sonda de relógio: corre à mão, em --release e com a máquina calma"]
fn diag_as_fases_por_quadro() {
    use super::composite_acumulado::fases;
    println!("\n  AS FASES   (pilha do dono · drena a cada 16 eventos)");
    println!("  passo | rota       |  total |    pre | acumular |  compor | cópias");
    for passo in [2.0f32, 16.0] {
        for por_quadro in [false, true] {
            let mut t = pilha_do_dono();
            t.set_compor_por_quadro(por_quadro);
            let _ = fases::take();
            let (ms, _) = traco(&mut t, passo, 16);
            let (us, _, _) = fases::take();
            println!(
                "  {passo:5.1} | {:10} | {ms:6.1} | {:6.1} | {:8.1} | {:7.1} | {:6.1}",
                if por_quadro {
                    "por quadro"
                } else {
                    "por evento"
                },
                us[0] as f64 / 1e3,
                us[1] as f64 / 1e3,
                us[2] as f64 / 1e3,
                us[3] as f64 / 1e3
            );
        }
    }
}

/// Dentro do COMPOR por quadro: quanto custa cada operação — o que atacar a seguir.
#[test]
#[ignore = "sonda de relógio: corre à mão, em --release e com a máquina calma"]
fn diag_as_operacoes_da_composicao() {
    use super::composite_acumulado::fases;
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "\n  AS OPERAÇÕES DO COMPOR   (load {} · drena a cada 16 eventos)",
        carga.trim()
    );
    for passo in [2.0f32, 8.0] {
        let mut melhor = [u64::MAX; 5];
        let mut acum = u64::MAX;
        for _ in 0..3 {
            let mut t = pilha_do_dono();
            t.set_compor_por_quadro(true);
            let _ = fases::take();
            let _ = fases::take_ops();
            let _ = traco(&mut t, passo, 16);
            let (us, _, _) = fases::take();
            let ops = fases::take_ops();
            acum = acum.min(us[fases::ACUMULAR]);
            for i in 0..5 {
                melhor[i] = melhor[i].min(ops[i]);
            }
        }
        println!("  passo {passo}: acumular {:.1} ms", acum as f64 / 1e3);
        for (i, n) in fases::OP_NOMES.iter().enumerate() {
            println!("    {n:10} {:7.1} ms", melhor[i] as f64 / 1e3);
        }
    }
}
