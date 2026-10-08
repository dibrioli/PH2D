//! **Sonda (ignorada): a largura que se VÊ para cada tamanho pedido**, nos quatro meios — a lei que a
//! W0 da linha Wet Painter corrige no Wet Paint (decisão do dono, 07/10: «o tamanho é a largura que se
//! vê», em todas as mídias). Imprime uma tabela `tamanho → largura medida` por meio.
//!
//! ```text
//! cargo test -p ph2d-tool-painter --profile smoke --test it sonda_largura -- --ignored --nocapture
//! ```

use super::sonda_gestos::{largura_mediana, traco_recto};
use ph2d_tool_painter::PaintMedia;

const TAMANHOS: [f32; 12] = [
    2.0, 4.0, 8.0, 12.0, 20.0, 28.0, 40.0, 60.0, 90.0, 140.0, 200.0, 300.0,
];

#[test]
#[ignore = "sonda: a lei tamanho → largura de hoje; corre-se à mão"]
fn a_largura_que_se_ve_por_tamanho() {
    let meios = [
        ("digital", PaintMedia::Digital),
        ("watercolor", PaintMedia::Watercolor),
        ("impasto", PaintMedia::Impasto),
        ("wetpaint", PaintMedia::WetPaint),
    ];
    print!("[largura] {:>7}", "tamanho");
    for (nome, _) in meios {
        print!(" {nome:>11}");
    }
    println!();
    // `PH2D_SONDA_OUT=<dir>` grava cada traço em PPM, para conferir esta régua contra `mede_traco.py`.
    let out = std::env::var("PH2D_SONDA_OUT").ok();
    for size in TAMANHOS {
        print!("[largura] {size:>7}");
        for (nome, meio) in meios {
            let (px, w, h) = traco_recto(meio, size, 480);
            print!(" {:>11.1}", largura_mediana(&px, w, h));
            if let Some(dir) = &out {
                let mut f = Vec::with_capacity((w * h * 3) as usize + 32);
                f.extend_from_slice(format!("P6\n{w} {h}\n255\n").as_bytes());
                px.chunks_exact(4)
                    .for_each(|p| f.extend_from_slice(&p[..3]));
                std::fs::create_dir_all(dir).unwrap();
                std::fs::write(format!("{dir}/{nome}_{size}.ppm"), f).unwrap();
            }
        }
        println!();
    }
}

/// A lei densa do Wet Paint: raio pedido → largura vista logo depois do traço, aos 8 s e aos 20 s
/// (a água corre e seca: a largura que se promete é a que fica).
#[test]
#[ignore = "sonda: a lei densa raio → largura do Wet Paint; corre-se à mão"]
fn a_lei_densa_do_wet_paint() {
    println!(
        "[densa] {:>7} {:>8} {:>8} {:>8}",
        "raio", "t=0", "t=8s", "t=20s"
    );
    let mut r = 1.0f32;
    while r <= 512.0 {
        let l: Vec<f32> = [0u32, 480, 1200]
            .iter()
            .map(|&k| {
                let (px, w, h) = traco_recto(PaintMedia::WetPaint, r, k);
                largura_mediana(&px, w, h)
            })
            .collect();
        println!("[densa] {r:>7.2} {:>8.1} {:>8.1} {:>8.1}", l[0], l[1], l[2]);
        r = (r * 1.25).max(r + 1.0);
    }
}

/// O degrau do Wet Paint entre os raios 12 e 19 (a largura fica em 10 px e salta para 18): varredura
/// fina, com o perfil transversal da coluna do meio impresso (cobertura × 100, de 2 em 2 px).
#[test]
#[ignore = "sonda: o degrau da largura do Wet Paint; corre-se à mão"]
fn o_degrau_do_wet_paint() {
    let mut r = 10.0f32;
    while r <= 25.0 {
        let (px, w, h) = traco_recto(PaintMedia::WetPaint, r, 480);
        let x = w / 2;
        let perfil: Vec<String> = (0..h)
            .step_by(2)
            .map(|y| {
                let i = ((y * w + x) * 4) as usize;
                let c = px[i..i + 3].iter().map(|&v| 255 - v).max().unwrap_or(0);
                (u32::from(c) * 100 / 255).to_string()
            })
            .filter(|s| s != "0")
            .collect();
        println!(
            "[degrau] {r:>5.1} larg {:>5.1}  perfil {}",
            largura_mediana(&px, w, h),
            perfil.join(" ")
        );
        r += 0.5;
    }
}
