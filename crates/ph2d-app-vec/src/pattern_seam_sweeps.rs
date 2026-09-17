//! ⭐⭐⭐ **AS VARREDURAS da costura do ladrilho** — a evidência que decidiu a W10 (plano 33).
//!
//! Elas **imprimem e não julgam**: o veredito foi escrito depois, com o número na mão, e vive no
//! irmão [`super`] (a lei) e no [§W10 do plano](../../../docs/Vector%20Module/33_plano_texture_pattern.md).
//!
//! ⚠️ **Ficam porque três curas foram REFUTADAS aqui**, e uma recusa medida só é uma recusa
//! enquanto o instrumento que a mediu continuar a correr. Quem propuser outra vez baixar a
//! qualidade das estampas, ou pôr um gutter no ladrilho, corre estas primeiro.
//!
//! Corra com `-- --ignored --nocapture`.

use super::*;
use ph2d_vector::{ImageQuality, StableImage};
use std::sync::Arc;

/// ⭐⭐⭐ **A MEDIÇÃO.** Imprime a tabela e não julga — o veredito é escrito depois, com o número na
/// mão. Corre com `-- --ignored --nocapture`.
#[test]
#[ignore = "needs a GPU adapter; measurement, run with --ignored --nocapture"]
fn measure_the_tile_seam() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter; skipping measure_the_tile_seam");
        return;
    };
    const PERIOD: u32 = 32;
    const REPS: u32 = 4;
    let (fine, fw, fh) = periodic_art(PERIOD, 1, 8);
    let (coarse, cw, ch) = periodic_art(PERIOD, REPS, 8);
    // ⭐ A prova de que o oráculo é o MESMO desenho: o largo é o estreito repetido, ao byte.
    for i in 0..coarse.len() {
        assert_eq!(
            coarse[i],
            fine[i % fine.len()],
            "o ladrilho largo nao e' o estreito repetido - o oraculo nao mede o amostrador"
        );
    }
    // ⛔⛔⛔ **O CONTROLO DA FIXTURA** — sem ele, uma arte lisa na costura imprime zeros e lê-se
    // como *"o defeito não existe"*. Foi exactamente o que a cosenoide fez.
    let last = fine[((fw - 1) * 4) as usize];
    let first = fine[0];
    assert!(
        last.abs_diff(first) > 100,
        "a fixtura nao tem o fenomeno: a arte vale {first} na primeira coluna e {last} na ultima, \
         logo NAO ha' contraste na fronteira do ladrilho - e' ali, e so' ali, que a costura vive"
    );
    let fine = StableImage::from_rgba(fine, fw, fh).expect("fine tile");
    let coarse = StableImage::from_rgba(coarse, cw, ch).expect("coarse tile");

    println!(
        "\n=== A COSTURA DO LADRILHO — periodo {PERIOD} px, oraculo = o mesmo padrao com periodo {} px ===",
        PERIOD * REPS
    );
    for (qname, q) in [
        ("Low   ", ImageQuality::Low),
        ("Medium", ImageQuality::Medium),
        ("High  ", ImageQuality::High),
    ] {
        for scale in [1.0_f64, 2.0, 4.0] {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let w = (f64::from(PERIOD * REPS) * scale) as u32;
            let h = 8u32;
            let a = render_tiled(&gpu, &fine, (w, h), scale, q);
            let b = render_tiled(&gpu, &coarse, (w, h), scale, q);
            let err = column_error(&a, &b, w, h);
            // ⚠️ A borda do ALVO é costura dos dois (x=0 e x=w), e a `x=0` do largo cai lá. Ignora
            // uma margem de 2 periodos de ecra' em cada ponta: o que sobra e' so' costura do
            // estreito, que e' o sujeito.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let margin = (f64::from(PERIOD) * scale) as u32;
            let inner = &err[margin as usize..(w - margin) as usize];
            let worst = inner.iter().copied().max().unwrap_or(0);
            let wrong = inner.iter().filter(|&&e| e > 1).count();
            let seams = (w - 2 * margin) / margin.max(1);
            let per_seam = if seams > 0 {
                wrong as f64 / f64::from(seams)
            } else {
                0.0
            };
            println!(
                "{qname} escala {scale:>4.1}x | pior desvio {worst:>3} niveis | \
                 colunas erradas {wrong:>4} de {:>4} | ~{per_seam:>5.1} px por costura",
                inner.len()
            );
            // ⭐ O PERFIL de uma costura: o que se vê é a FORMA da banda, não só a largura.
            if worst > 1 {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let seam = (f64::from(2 * PERIOD) * scale) as usize;
                let half = (margin as usize).min(8);
                let perfil: Vec<String> = err[seam - half..seam + half]
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect();
                println!("         costura em x={seam}: [{}]", perfil.join(" "));
            }
        }
    }
    println!();
}

/// ⭐⭐⭐ **O OUTRO LADO DO NEGÓCIO: o que o `High` COMPRA.**
///
/// Se o `High` só custasse costura, a escolha era trivial. Ele compra **fidelidade na ampliação**,
/// e escolher sem a medir seria decidir com meia régua.
///
/// # A régua
///
/// A verdade é a MESMA onda amostrada **4x mais fina**, desenhada à escala `1` — nenhum filtro tem
/// de inventar nada ali. O sujeito é a onda grossa desenhada a `4x`, que é exactamente o mesmo
/// desenho no mundo. `|sujeito - verdade|` **longe das costuras** é o erro de ampliação.
///
/// ⚠️ A fixtura é a **cosenoide**, e tem de ser: uma onda quadrada não tem interior para reproduzir
/// — o que ela mediria era o toque de sino do filtro numa aresta, que é outra pergunta.
#[test]
#[ignore = "needs a GPU adapter; measurement, run with --ignored --nocapture"]
fn measure_what_high_buys_in_the_interior() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter; skipping measure_what_high_buys_in_the_interior");
        return;
    };
    const K: u32 = 4;
    const W: u32 = 512;
    const H: u32 = 8;
    println!(
        "\n=== O QUE O `High` COMPRA — onda lisa ampliada {K}x, verdade = a mesma onda {K}x mais fina ==="
    );
    println!(
        "(`texels/periodo` e' a densidade de amostragem da ARTE: quanto menor, mais curvatura ha' ENTRE texels vizinhos)"
    );
    // ⚠️ Uma so' densidade nao decide nada: a 32 texels/periodo uma cosenoide e' quase LINEAR entre
    // vizinhos, e ali bilinear e bicubico coincidem por construcao. A varredura e' que mostra ONDE
    // — se e' que algures — o `High` se separa do `Medium`.
    for p in [4_u32, 8, 16, 32] {
        let (coarse, cw, chh) = art(p, 1, H, Wave::Cosine);
        let (dense, dw, dh) = art(p * K, 1, H, Wave::Cosine);
        let coarse = StableImage::from_rgba(coarse, cw, chh).expect("coarse");
        let dense = StableImage::from_rgba(dense, dw, dh).expect("dense");
        // A verdade: a onda fina, sem ampliacao nenhuma. `Low` chega — nao ha' o que interpolar.
        let truth = render_tiled(&gpu, &dense, (W, H), 1.0, ImageQuality::Low);
        for (qname, q) in [
            ("Low   ", ImageQuality::Low),
            ("Medium", ImageQuality::Medium),
            ("High  ", ImageQuality::High),
        ] {
            let got = render_tiled(&gpu, &coarse, (W, H), f64::from(K), q);
            let err = column_error(&got, &truth, W, H);
            // ⚠️ Longe das costuras: as duas imagens repetem com periodo `p*K` no ecra', e a
            // costura e' o assunto do OUTRO gate. Descontar +-p/2 de cada multiplo isola o interior.
            let periodo = p * K;
            let interior: Vec<u8> = err
                .iter()
                .enumerate()
                .filter(|(x, _)| {
                    let d = (*x as u32) % periodo;
                    d > p / 2 && d < periodo - p / 2
                })
                .map(|(_, e)| *e)
                .collect();
            let pior = interior.iter().copied().max().unwrap_or(0);
            let medio = f64::from(interior.iter().map(|&e| u32::from(e)).sum::<u32>())
                / interior.len() as f64;
            println!(
                "{p:>3} texels/periodo | {qname} | pior no interior {pior:>3} niveis | medio {medio:>5.1}"
            );
        }
    }
    println!();
}

/// ⭐⭐⭐ **A COSTURA NAS ARTES QUE O PRODUTO DE FACTO ASSA.**
///
/// A onda quadrada é o pior caso construído de propósito. Esta é a pergunta que decide se há
/// problema de produto: *quem, das artes reais, tem contraste NA FRONTEIRA?*
#[test]
#[ignore = "needs a GPU adapter; measurement, run with --ignored --nocapture"]
fn measure_the_seam_on_realistic_art() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter; skipping measure_the_seam_on_realistic_art");
        return;
    };
    const P: u32 = 32;
    const REPS: u32 = 4;
    const HH: u32 = 32;
    let quadrada = periodic_art(P, 1, HH).0;
    let casos: [(&str, Arc<Vec<u8>>); 3] = [
        ("motivo, caixa justa (forma assada)", disc_art(P, HH)),
        ("textura de bordo a bordo (imagem) ", noise_art(P, HH)),
        ("onda quadrada (o pior caso)       ", quadrada),
    ];
    println!("\n=== A COSTURA NAS ARTES REAIS — ladrilho {P}x{HH}, ampliado 4x ===");
    for (nome, art) in casos {
        let largo = repeat_x(&art, P, HH, REPS);
        let fino = StableImage::from_rgba(art, P, HH).expect("fino");
        let largo = StableImage::from_rgba(largo, P * REPS, HH).expect("largo");
        let (w, h) = (P * REPS * 4, HH);
        let a = render_tiled(&gpu, &fino, (w, h), 4.0, ImageQuality::High);
        let b = render_tiled(&gpu, &largo, (w, h), 4.0, ImageQuality::High);
        let err = column_error(&a, &b, w, h);
        let margem = (P * 4) as usize;
        let inner = &err[margem..err.len() - margem];
        let pior = inner.iter().copied().max().unwrap_or(0);
        let erradas = inner.iter().filter(|&&e| e > 1).count();
        println!("{nome} | pior {pior:>3} niveis | {erradas:>3} colunas erradas");
    }
    println!();
}

/// ⛔⛔⛔ **O CONTROLO QUE DECIDE SE O `Medium` VALE ALGUMA COISA.**
///
/// A 1.ª tabela desta sonda mediu `Medium` a `0` niveis na escala `1x` — e isso é verdade **só no
/// alinhamento inteiro**, onde o bilinear cai exactamente sobre um texel e não interpola nada.
/// Um `pan` de meio pixel tira-o de lá. *Uma vantagem que só existe num ponto de medida zero não é
/// uma vantagem.*
#[test]
#[ignore = "needs a GPU adapter; measurement, run with --ignored --nocapture"]
fn measure_the_seam_across_subpixel_offsets() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter; skipping measure_the_seam_across_subpixel_offsets");
        return;
    };
    const P: u32 = 32;
    const REPS: u32 = 4;
    let (fino, fw, fh) = periodic_art(P, 1, 8);
    let (largo, cw, ch) = periodic_art(P, REPS, 8);
    let fino = StableImage::from_rgba(fino, fw, fh).expect("fino");
    let largo = StableImage::from_rgba(largo, cw, ch).expect("largo");
    println!("\n=== O ALINHAMENTO IMPORTA? — pior desvio por deslocamento de sub-pixel ===");
    for scale in [1.0_f64, 2.0] {
        for (qname, q) in [
            ("Medium", ImageQuality::Medium),
            ("High  ", ImageQuality::High),
        ] {
            let mut linha = Vec::new();
            for off in [0.0_f64, 0.125, 0.25, 0.375, 0.5] {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let w = (f64::from(P * REPS) * scale) as u32;
                let h = 8u32;
                let a = render_tiled_at(&gpu, &fino, (w, h), scale, off, q);
                let b = render_tiled_at(&gpu, &largo, (w, h), scale, off, q);
                let err = column_error(&a, &b, w, h);
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let margem = (f64::from(P) * scale) as usize;
                let inner = &err[margem..err.len() - margem];
                let pior = inner.iter().copied().max().unwrap_or(0);
                let erradas = inner.iter().filter(|&&e| e > 1).count();
                linha.push(format!("{off:>5.3}px: {pior:>3}/{erradas:>2}col"));
            }
            println!("escala {scale:>4.1}x {qname} | {}", linha.join("  "));
        }
    }
    println!();
}

/// ⭐⭐⭐ **O CONTROLO QUE FECHA A INVESTIGAÇÃO: um ladrilho que FECHA tem costura?**
///
/// A hipótese, depois de três curas refutadas: *o grampo custa exactamente a DESCONTINUIDADE do
/// próprio ladrilho na volta*. Se for verdade, um ladrilho contínuo na fronteira não tem defeito
/// nenhum — e o produto **já entrega** a ferramenta que torna todo ladrilho contínuo: o modo
/// **Mirror**.
#[test]
#[ignore = "needs a GPU adapter; measurement, run with --ignored --nocapture"]
fn measure_whether_a_tile_that_closes_has_any_seam() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter; skipping measure_whether_a_tile_that_closes_has_any_seam");
        return;
    };
    const P: u32 = 32;
    const HH: u32 = 32;
    const REPS: u32 = 4;
    let quadrada = periodic_art(P, 1, HH).0;
    let casos: [(&str, Arc<Vec<u8>>, u32); 4] = [
        ("ruido CRU (nao fecha)          ", noise_art(P, HH), P),
        (
            "ruido ESPELHADO (fecha ao bit) ",
            mirrored_x(&noise_art(P, HH), P, HH),
            P * 2,
        ),
        ("onda quadrada CRUA             ", quadrada.clone(), P),
        (
            "onda quadrada ESPELHADA        ",
            mirrored_x(&quadrada, P, HH),
            P * 2,
        ),
    ];
    println!(
        "\n=== UM LADRILHO QUE FECHA TEM COSTURA? — High, escala 4x, deslocamento 0,5 px (o pior) ==="
    );
    for (nome, art, w_tile) in casos {
        // A descontinuidade do PROPRIO ladrilho na volta: |coluna 0 - coluna w-1|.
        let mut salto = 0u8;
        for y in 0..HH {
            let esq = ((y * w_tile) * 4) as usize;
            let dir = ((y * w_tile + w_tile - 1) * 4) as usize;
            for c in 0..3 {
                salto = salto.max(art[esq + c].abs_diff(art[dir + c]));
            }
        }
        let largo = repeat_x(&art, w_tile, HH, REPS);
        let fino = StableImage::from_rgba(art, w_tile, HH).expect("fino");
        let largo = StableImage::from_rgba(largo, w_tile * REPS, HH).expect("largo");
        let (w, h) = (w_tile * REPS * 4, HH);
        let a = render_tiled_at(&gpu, &fino, (w, h), 4.0, 0.5, ImageQuality::High);
        let b = render_tiled_at(&gpu, &largo, (w, h), 4.0, 0.5, ImageQuality::High);
        let err = column_error(&a, &b, w, h);
        let margem = (w_tile * 4) as usize;
        let inner = &err[margem..err.len() - margem];
        let pior = inner.iter().copied().max().unwrap_or(0);
        let erradas = inner.iter().filter(|&&e| e > 1).count();
        println!(
            "{nome} | salto do ladrilho {salto:>3} | costura medida {pior:>3} niveis | {erradas:>3} colunas"
        );
    }
    println!();
}

/// ⭐⭐⭐ **A CURVA salto -> costura.** Dois pontos dão uma recta que ninguém verificou; o limiar de
/// um aviso ao artista tem de sair de uma VARREDURA.
#[test]
#[ignore = "needs a GPU adapter; measurement, run with --ignored --nocapture"]
fn measure_the_seam_against_the_tiles_own_gap() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter; skipping measure_the_seam_against_the_tiles_own_gap");
        return;
    };
    const P: u32 = 32;
    const HH: u32 = 32;
    const REPS: u32 = 4;
    println!("\n=== A CURVA: o SALTO do ladrilho contra a COSTURA medida (High, 4x, 0,5 px) ===");
    for alvo in [0_u8, 2, 4, 8, 16, 32, 64, 128, 200] {
        let art = noise_with_gap(P, HH, alvo);
        let mut salto = 0u8;
        for y in 0..HH {
            let esq = ((y * P) * 4) as usize;
            let dir = ((y * P + P - 1) * 4) as usize;
            for c in 0..3 {
                salto = salto.max(art[esq + c].abs_diff(art[dir + c]));
            }
        }
        let largo = repeat_x(&art, P, HH, REPS);
        let fino = StableImage::from_rgba(art, P, HH).expect("fino");
        let largo = StableImage::from_rgba(largo, P * REPS, HH).expect("largo");
        let (w, h) = (P * REPS * 4, HH);
        let a = render_tiled_at(&gpu, &fino, (w, h), 4.0, 0.5, ImageQuality::High);
        let b = render_tiled_at(&gpu, &largo, (w, h), 4.0, 0.5, ImageQuality::High);
        let err = column_error(&a, &b, w, h);
        let margem = (P * 4) as usize;
        let inner = &err[margem..err.len() - margem];
        let pior = inner.iter().copied().max().unwrap_or(0);
        let erradas = inner.iter().filter(|&&e| e > 1).count();
        println!(
            "salto pedido {alvo:>3} | salto real {salto:>3} | costura {pior:>3} niveis | {erradas:>3} colunas"
        );
    }
    println!();
}
