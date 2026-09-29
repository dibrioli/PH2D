//! ⏱️⭐⭐⭐⭐ **A OCLUSÃO A PASSO NO QUADRO DE MOVIMENTO** — a sonda que escolhe o passo
//! (`docs/Render3d/03_o_plano.md` §W9; `ph2d_field_gpu::trace::MarchSetup::ceu_passo`).
//!
//! O report do dono (2026-09-24): *«render de boa qualidade, movimentação mais fluida, mas queda de
//! resolução do modelo»*. A oclusão é `60`–`80 %` do Render a mexer, e é ela que obriga o laço do
//! movimento a encolher a tela. A pergunta: marchar os cones num pixel de cada `n × n` e
//! reconstruir os outros guiado pela forma compra quanto, e quanto move a imagem?

/// Uma cena, o quadro de MOVIMENTO pela porta do produto, com o passo da oclusão dado.
fn quadro(t: &crate::gpu_frame::SharedTracer, cena: u32, passo: u32, w: u32, h: u32) -> Vec<u8> {
    let cam = ph2d_field_render::Orbit::default();
    let doc = crate::smoke::scene(cena);
    let reg = crate::smoke::sampled_registry();
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let pres = ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default());
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    crate::gpu_frame::paint_com(
        t,
        &doc,
        &reg,
        &cam,
        &luz,
        &surfaces,
        &pres,
        [40, 40, 40, 255],
        chao,
        w,
        h,
        false,
        crate::gpu_frame::Sonda {
            ceu_passo: passo,
            ..crate::gpu_frame::Sonda::default()
        },
    )
    .expect("o pintor")
    .rgba
}

/// ⏱️ **Sonda: relógio e imagem do quadro de movimento, passo `1` contra `2`, `3` e `4`.** A imagem
/// compara-se byte a byte com a do passo `1` (a oclusão em todo pixel). Com `PH2D_SONDA_DIR` grava
/// o nó nos dois passos e a diferença ampliada `8×`.
#[test]
#[ignore = "sonda de GPU"]
fn diag_o_ceu_a_passo() {
    const W: u32 = 1920;
    const H: u32 = 1080;
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador");
        return;
    };
    println!(
        "\n  {}\n  cena · passo · ms [mín 5] · canais >2 B · >8 B · p99 B · máx B",
        super::super::super::super::contexto()
    );
    for cena in [5u32, 28, 1, 11, 26, 27, 29, 30] {
        let base = quadro(t, cena, 1, W, H);
        for passo in [1u32, 2, 3, 4] {
            let _ = quadro(t, cena, passo, W, H);
            let mut ms = f64::INFINITY;
            for _ in 0..5 {
                let t0 = std::time::Instant::now();
                let _ = quadro(t, cena, passo, W, H);
                ms = ms.min(t0.elapsed().as_secs_f64() * 1e3);
            }
            let img = quadro(t, cena, passo, W, H);
            let mut d: Vec<u8> = img.iter().zip(&base).map(|(a, b)| a.abs_diff(*b)).collect();
            let (a2, a8) = (
                d.iter().filter(|x| **x > 2).count(),
                d.iter().filter(|x| **x > 8).count(),
            );
            d.sort_unstable();
            let p99 = d[d.len() * 99 / 100];
            let max = d[d.len() - 1];
            println!(
                "  {cena:4} · {passo:5} · {ms:>7.2} · {a2:>9} · {a8:>7} · {p99:>5} · {max:>5}"
            );
        }
    }
    if let Ok(dir) = std::env::var("PH2D_SONDA_DIR") {
        let grava = |nome: &str, rgba: &[u8]| {
            let mut ppm = format!("P6\n{W} {H}\n255\n").into_bytes();
            for px in rgba.as_chunks::<4>().0 {
                ppm.extend_from_slice(&px[..3]);
            }
            let caminho = format!("{dir}/ceu_passo_{nome}.ppm");
            std::fs::write(&caminho, ppm).expect("grava");
            println!("gravado {caminho}");
        };
        // `PH2D_SONDA_CENA` escolhe a cena gravada (o nó por omissão).
        let cena = std::env::var("PH2D_SONDA_CENA")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(28u32);
        let um = quadro(t, cena, 1, W, H);
        let dois = quadro(t, cena, 2, W, H);
        let dif: Vec<u8> = um
            .iter()
            .zip(&dois)
            .map(|(a, b)| a.abs_diff(*b).saturating_mul(8))
            .collect();
        grava("1", &um);
        grava("2", &dois);
        grava("dif8x", &dif);
    }
}

/// Uma cena no quadro de MOVIMENTO com o passo da oclusão e a borda re-amostrada dados.
fn quadro_bissecto(
    t: &crate::gpu_frame::SharedTracer,
    cena: u32,
    passo: u32,
    bordas: bool,
    w: u32,
    h: u32,
) -> Vec<u8> {
    let cam = ph2d_field_render::Orbit::default();
    let doc = crate::smoke::scene(cena);
    let reg = crate::smoke::sampled_registry();
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let pres = ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default());
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    crate::gpu_frame::paint_com(
        t,
        &doc,
        &reg,
        &cam,
        &luz,
        &surfaces,
        &pres,
        [40, 40, 40, 255],
        chao,
        w,
        h,
        false,
        crate::gpu_frame::Sonda {
            ceu_passo: passo,
            bordas,
            ..crate::gpu_frame::Sonda::default()
        },
    )
    .expect("o pintor")
    .rgba
}

/// ⏱️ **Sonda: o CONTORNO PONTILHADO da foto de 25/09** — o quadro de movimento do nó a `960×540`
/// (o tamanho que o laço escolhe para ele) nas quatro combinações de passo da oclusão × borda
/// re-amostrada, com a diferença contra `(passo 1, borda ligada)`. Com `PH2D_SONDA_DIR` grava as
/// quatro imagens.
#[test]
#[ignore = "sonda de GPU"]
fn diag_o_contorno_pontilhado() {
    const W: u32 = 960;
    const H: u32 = 540;
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador");
        return;
    };
    let cena = std::env::var("PH2D_SONDA_CENA")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(28u32);
    let base = quadro_bissecto(t, cena, 1, true, W, H);
    println!("\n  cena {cena} · passo · borda · canais >2 B · >8 B · máx B");
    for (passo, bordas) in [(1u32, true), (2, true), (1, false), (2, false)] {
        let img = quadro_bissecto(t, cena, passo, bordas, W, H);
        let d: Vec<u8> = img.iter().zip(&base).map(|(a, b)| a.abs_diff(*b)).collect();
        let a2 = d.iter().filter(|x| **x > 2).count();
        let a8 = d.iter().filter(|x| **x > 8).count();
        let max = d.iter().copied().max().unwrap_or(0);
        println!("  {passo:5} · {bordas:5} · {a2:>9} · {a8:>7} · {max:>5}");
        if let Ok(dir) = std::env::var("PH2D_SONDA_DIR") {
            let mut ppm = format!("P6\n{W} {H}\n255\n").into_bytes();
            for px in img.as_chunks::<4>().0 {
                ppm.extend_from_slice(&px[..3]);
            }
            let caminho = format!("{dir}/pontilhado_{cena}_p{passo}_b{}.ppm", u8::from(bordas));
            std::fs::write(&caminho, ppm).expect("grava");
            println!("  gravado {caminho}");
        }
    }
}
