//! 📏 **Sonda: quanto peso o `hint` tira do texto do Inspector** (CodeEditor 15/10).
//!
//! O `CrispHeavyPlus` tinha `hint: false` com a razão *«o autohinter achata o peso extra»*. Esta
//! sonda mede-o: o mesmo rótulo pela porta do produto (`paint_text`), no preset de fábrica, hint
//! OFF × ON escrito na codificação (variante de execução, mesmo processo, duas voltas).
//!
//! Corre: `PH2D_GPU=1 bash scripts/cargo-test-narrow.sh ph2d-app-components mede_o_peso_do_texto_com_hint -- --ignored --nocapture`
//! Saída: `docs/CodeEditor/ferramentas/medicoes/peso_hint.txt` + `peso_*.png` (×4).

use super::texto_na_placa::{Alfa, Letra, estilo_do_dono, grava_png, placa, rasteriza};
use ph2d_text::TextSystem;
use ph2d_tokens::{TextRendering, UiTextStyle};
use std::fmt::Write as _;

const TEXTO: &str = "Script  Browse  amplitude  speed  line 3 · Expected identifier";
const CAIXA: (f32, f32) = (420.0, 24.0);
const ESCALAS: [f32; 4] = [1.0, 1.25, 1.5, 2.0];

struct Medida {
    tinta: f32,
    media: f32,
    solida: f32,
    pico_col: f32,
    pico_lin: f32,
    linhas: Vec<f32>,
}

fn mede(a: &Alfa) -> Medida {
    let (w, h) = (a.w as usize, a.h as usize);
    let com: Vec<f32> = a.a.iter().copied().filter(|&v| v > 0.02).collect();
    let picos = |n: usize, f: &dyn Fn(usize, usize) -> f32, m: usize| {
        let v: Vec<f32> = (0..n)
            .map(|i| (0..m).map(|j| f(i, j)).fold(0.0, f32::max))
            .filter(|&p| p > 0.02)
            .collect();
        v.iter().sum::<f32>() / v.len() as f32
    };
    Medida {
        tinta: com.iter().sum(),
        media: com.iter().sum::<f32>() / com.len() as f32,
        solida: com.iter().filter(|&&v| v >= 0.75).count() as f32 / com.len() as f32,
        pico_col: picos(w, &|c, r| a.a[r * w + c], h),
        pico_lin: picos(h, &|r, c| a.a[r * w + c], w),
        linhas: a.linhas().into_iter().filter(|&t| t > 0.0).collect(),
    }
}

#[test]
#[ignore = "sonda de placa (PH2D_GPU=1): mede, não vigia"]
fn mede_o_peso_do_texto_com_hint() {
    let max = ((CAIXA.0 * 2.0) as u32, (CAIXA.1 * 2.0) as u32);
    let Some(mut placa) = placa(max) else {
        eprintln!("sem placa — a sonda não mede nada");
        return;
    };
    let mut ts = TextSystem::without_system_fonts();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/CodeEditor/ferramentas/medicoes");
    let letras = [
        ("dono", Letra::Ui(estilo_do_dono())),
        ("inter", Letra::Ui(UiTextStyle::default())),
    ];
    let mut vs = Vec::new();
    for &s in &ESCALAS {
        for (nome, letra) in letras {
            for hint in [false, true] {
                vs.push((nome, letra, s, hint));
            }
        }
    }
    let mut primeira = Vec::new();
    let mut tabela = String::from(
        "variante                 tinta   Δtinta  média  sólida  pico/col pico/lin  tinta por linha de píxel\n",
    );
    for volta in 0..2 {
        let ordem: Vec<usize> = if volta == 0 {
            (0..vs.len()).collect()
        } else {
            (0..vs.len()).rev().collect()
        };
        let mut todas = vec![Vec::new(); vs.len()];
        for i in ordem {
            let (nome, letra, s, hint) = vs[i];
            let a = rasteriza(
                &mut placa,
                &mut ts,
                letra,
                (TextRendering::CrispHeavyPlus, Some(hint)),
                s,
                TEXTO,
                CAIXA,
            );
            if volta == 0 && s <= 1.25 && nome == "dono" {
                let f = format!(
                    "peso_dono_s{:.0}_hint{}.png",
                    s * 100.0,
                    if hint { "on" } else { "off" }
                );
                grava_png(&a, 4, &dir.join(f));
            }
            todas[i] = a.a;
        }
        if volta == 0 {
            primeira = todas;
        } else {
            let difere: Vec<String> = (0..vs.len())
                .map(|i| {
                    let (nome, _, s, hint) = vs[i];
                    let d = primeira[i]
                        .iter()
                        .zip(&todas[i])
                        .map(|(a, b)| (a - b).abs())
                        .fold(0.0, f32::max);
                    (d, format!("s{s} {nome} hint={hint}: Δmax {d:.3}"))
                })
                // ⚠️ 1 LSB (1/255) muda quando o passe acabou de mudar de tamanho — medido 15/10.
                .filter(|(d, _)| *d > 1.5 / 255.0)
                .map(|(_, t)| t)
                .collect();
            assert!(
                difere.is_empty(),
                "a rasterização não é determinística: {difere:#?}"
            );
        }
    }
    let mut off = None;
    for (i, (nome, _, s, hint)) in vs.iter().enumerate() {
        let (w, h) = ((CAIXA.0 * s).ceil() as u32, (CAIXA.1 * s).ceil() as u32);
        let m = mede(&Alfa {
            w,
            h,
            a: primeira[i].clone(),
        });
        let delta = match (hint, off) {
            (true, Some(t)) => format!("{:+5.1} %", 100.0 * (m.tinta / t - 1.0)),
            _ => "      ".into(),
        };
        if !hint {
            off = Some(m.tinta);
        }
        let linhas: Vec<String> = m.linhas.iter().map(|v| format!("{v:.1}")).collect();
        let _ = writeln!(
            tabela,
            "s{s:.2} {nome:<5} hint={:<3}   {:7.1} {delta} {:5.3}  {:5.3}   {:5.3}    {:5.3}    [{}]",
            if *hint { "on" } else { "off" },
            m.tinta,
            m.media,
            m.solida,
            m.pico_col,
            m.pico_lin,
            linhas.join(" ")
        );
    }
    print!("{tabela}");
    std::fs::write(dir.join("peso_hint.txt"), &tabela).unwrap();
}
