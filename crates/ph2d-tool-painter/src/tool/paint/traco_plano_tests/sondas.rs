//! As SONDAS (`#[ignore]`) da cauda do plano do traço — filho de `traco_plano_tests` por LOC.

use super::*;

/// Sonda (dono 25/10): a cauda com o Continuous, o Accumulate desligado e o Flow baixo «buga de vários modos» — discos
/// e degraus. Grava a prancha em `PH2D_SONDA_DIR` e imprime a tinta no eixo (`255 − R` sobre branco).
#[test]
#[ignore = "sonda: a cauda com o Continuous (dono 25/10)"]
fn sonda_cauda_continuous() {
    let dir =
        std::path::PathBuf::from(std::env::var("PH2D_SONDA_DIR").unwrap_or_else(|_| "/tmp".into()));
    let (w, h) = (600u32, 110u32);
    let mut prancha = Vec::new();
    // (nome, cauda, levanta, blend, continuous, accumulate, flow, strength)
    let casos = [
        ("cauda", 3.0, true, BrushBlend::Mix, true, false, 0.3, 0.6),
        (
            "cauda aberta",
            3.0,
            false,
            BrushBlend::Mix,
            true,
            false,
            0.3,
            0.6,
        ),
        (
            "multiply",
            0.0,
            true,
            BrushBlend::Multiply,
            true,
            false,
            0.3,
            0.6,
        ),
        ("mix", 0.0, true, BrushBlend::Mix, true, false, 0.3, 0.6),
        (
            "cauda sem cont",
            3.0,
            true,
            BrushBlend::Mix,
            false,
            false,
            0.3,
            0.6,
        ),
        (
            "plano S1",
            0.0,
            false,
            BrushBlend::Multiply,
            true,
            false,
            0.3,
            1.0,
        ),
        ("mix S1", 0.0, false, BrushBlend::Mix, true, false, 0.3, 1.0),
        (
            "plano sem cont",
            0.0,
            false,
            BrushBlend::Multiply,
            false,
            false,
            0.3,
            0.6,
        ),
        (
            "mix sem cont",
            0.0,
            false,
            BrushBlend::Mix,
            false,
            false,
            0.3,
            0.6,
        ),
        (
            "plano acc",
            0.0,
            false,
            BrushBlend::Multiply,
            false,
            true,
            1.0,
            0.6,
        ),
        (
            "mix acc",
            0.0,
            false,
            BrushBlend::Mix,
            false,
            true,
            1.0,
            0.6,
        ),
        (
            "cauda acc translúcida",
            3.0,
            true,
            BrushBlend::Mix,
            false,
            true,
            0.3,
            0.6,
        ),
        (
            "acc translúcida",
            0.0,
            true,
            BrushBlend::Mix,
            false,
            true,
            0.3,
            0.6,
        ),
    ];
    for (nome, cauda, levanta, blend, cont, acc, flow, forca) in casos {
        let mut b = pincel(20.0);
        b.hardness = 0.3;
        b.spacing = 0.4;
        b.continuous_spacing = cont;
        b.accumulate = acc;
        b.flow = flow;
        b.strength = forca;
        b.blend = blend;
        b.taper.end = cauda;
        let mut t = tool(w, h, [255; 4], b);
        let pts: Vec<[f32; 2]> = (0..=110).map(|k| [50.0 + 4.6 * k as f32, 55.0]).collect();
        traco(&mut t, &pts, levanta);
        let eixo: Vec<u8> = (0..w)
            .step_by(8)
            .map(|x| 255 - t.canvas_rgba[((55 * w + x) * 4) as usize])
            .collect();
        println!("[cauda] {nome}: {eixo:?}");
        prancha.extend_from_slice(&t.canvas_rgba);
    }
    let png = super::super::composite_look::png_rgba(&prancha, w, h * casos.len() as u32);
    std::fs::write(dir.join("cauda_continuous.png"), png).unwrap();
}

/// Sonda (25/10): a cauda nas 4 combinações (Continuous, Accumulate) a Flow 0,3 — o pico do corpo, o pico da
/// entrada da janela (420..460), o pico da cauda e o eixo dela.
#[test]
#[ignore = "sonda: a cauda nas 4 combinações"]
fn sonda_cauda_quatro() {
    for (cont, acc) in [(true, false), (false, false), (true, true), (false, true)] {
        let com = eixo_do_recto(recto_translucido(cont, acc, 0.6, BrushBlend::Mix, 3.0));
        let sem = eixo_do_recto(recto_translucido(cont, acc, 0.6, BrushBlend::Mix, 0.0));
        let corpo = (60..410).map(|x| sem[x]).max().unwrap();
        let entrada = (410..460).map(|x| com[x]).max().unwrap();
        let cauda = (436..600).map(|x| com[x]).max().unwrap();
        let sobe = com[436..600]
            .windows(2)
            .map(|w| i32::from(w[1]) - i32::from(w[0]))
            .max()
            .unwrap();
        let env = envoltoria(&com[410..600]);
        let pico = env.iter().enumerate().max_by_key(|(_, v)| **v).unwrap().0;
        let env_sobe = env[pico..]
            .windows(2)
            .map(|w| i32::from(w[1]) - i32::from(w[0]))
            .max()
            .unwrap_or(0);
        println!(
            "[quatro] C{cont} A{acc}: corpo {corpo} entrada {entrada} cauda {cauda} sobe {sobe} env_sobe {env_sobe}\n  {:?}",
            &com[400..600]
        );
    }
}
