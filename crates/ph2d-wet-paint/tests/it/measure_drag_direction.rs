//! **O ARRASTO DO BICO DEPENDE DO SENTIDO DO TRAÇO?** — a sonda do item 1 da fila
//! ([doc 44 §1](../../../../docs/Painter/44_a_fila_depois_da_linha.md)).
//!
//! O passo 5 do `transfer_paint` (o arrasto) lê `susp`/`sett`/`film`/`wet` na janela da âncora
//! ANTERIOR, nos mesmos planos que o próprio laço escreve em ordem de raster. Num traço para a
//! DIREITA a origem `si = i − d` já foi reescrita neste laço (Gauss-Seidel); num traço para a
//! ESQUERDA `si = i + d` ainda não foi (lê o valor de antes, que é o que uma leitura sem ordem —
//! Jacobi — leria sempre).
//!
//! ⚠️ **Por que não um teste de ESPELHO:** o papel e as cerdas têm ruído com semente que não é
//! simétrico, e o mesmo traço espelhado com o arrasto DESLIGADO já diverge `67 %` da mudança que o
//! traço faz (medido 2026-09-29) — o fundo afoga o sinal.
//!
//! ⇒ a sonda **grava** o grid final de um traço para a direita e de um para a esquerda num
//! directório (`PH2D_DRAG_DUMP=<dir>`), e a comparação é feita contra a MESMA sonda corrida com o
//! arrasto trocado por uma leitura sem ordem (instrumento no doc 44 §1). Previsão que torna a
//! medição honesta: **para a esquerda as duas leis dão os MESMOS bits** (é o CONTROLO); para a
//! direita, a diferença é exactamente o que a ordem compra.
//!
//! ```text
//! PH2D_DRAG_DUMP=<dir> bash scripts/ph2d-run.sh cargo test -p ph2d-wet-paint --release \
//!   --test it measure_drag_direction -- --ignored --nocapture
//! ```

use std::io::Write;

use ph2d_wet_paint::painter::Engine;

const W: usize = 400;
const H: usize = 200;

/// Tinta molhada com estrutura (dentes em `x`, onda em `y`): um campo chato daria o mesmo
/// resultado sob qualquer ordem de leitura e mediria o vazio.
fn molhado() -> Engine {
    let mut e = Engine::new(W, H);
    e.sliders.water = 1.0;
    e.sliders.size = 0.5;
    let g = e.active_grid_mut();
    let s = g.s;
    for y in 0..g.h + 2 {
        for x in 0..g.w + 2 {
            let i = x + y * s;
            if i >= g.susp.len() {
                continue;
            }
            let dentes = if (x % 9) < 4 { 1.4 } else { 0.7 };
            let fy = 1.0 + 0.3 * ((y as f64) * 0.21).sin();
            g.susp[i] = (400.0 * dentes * fy) as f32;
            g.sett[i] = (150.0 * (2.1 - dentes) * fy) as f32;
            g.film[i] = (0.6 * dentes) as f32;
            g.wet[i] = (120.0 + 60.0 * dentes) as u8;
            let c = ((x % 40) as f32) * 5.0;
            g.susp_rgb[i] = [c, 200.0 - c, 90.0];
            g.sett_rgb[i] = [60.0, c, 180.0 - c];
        }
    }
    e
}

/// Um traço horizontal com o relógio de 40 Hz, SEM simulação (ela fica parada com o ponteiro em
/// baixo): o que muda o grid é só o depósito e o transfer.
fn traco(e: &mut Engine, x0: f64, x1: f64, y: f64, passo: f64) {
    e.pointer_down(x0, y, None);
    let n = ((x1 - x0).abs() / passo).ceil() as usize;
    for k in 1..=n {
        let t = (k as f64 / n as f64).min(1.0);
        e.pointer_frame(x0 + (x1 - x0) * t, y);
    }
}

fn grava(dir: &std::path::Path, nome: &str, e: &Engine) {
    let g = e.active_grid();
    let mut f = std::fs::File::create(dir.join(nome)).expect("criar o ficheiro do grid");
    for plano in [&g.susp, &g.sett, &g.film] {
        for v in plano.iter() {
            f.write_all(&v.to_le_bytes()).expect("gravar");
        }
    }
    for c in g.susp_rgb.iter() {
        for v in c {
            f.write_all(&v.to_le_bytes()).expect("gravar");
        }
    }
    f.write_all(&g.wet).expect("gravar");
}

#[test]
#[ignore = "sonda: o arrasto depende do sentido do traço? (doc 44 §1)"]
fn measure_drag_direction() {
    let Ok(dir) = std::env::var("PH2D_DRAG_DUMP") else {
        println!("PH2D_DRAG_DUMP não definido: nada a gravar");
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&dir).expect("criar o directório");
    let y = (H / 2) as f64;
    // Dois passos por quadro: o curto (a janela anterior sobrepõe-se quase toda) e o longo.
    for passo in [3.0, 12.0] {
        let mut direita = molhado();
        traco(&mut direita, 80.0, 320.0, y, passo);
        grava(&dir, &format!("direita_{passo}.bin"), &direita);
        let mut esquerda = molhado();
        traco(&mut esquerda, 320.0, 80.0, y, passo);
        grava(&dir, &format!("esquerda_{passo}.bin"), &esquerda);
    }
    grava(&dir, "antes.bin", &molhado());
    println!("gravado em {}", dir.display());
}
