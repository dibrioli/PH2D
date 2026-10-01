//! ⏱️⭐⭐⭐⭐ **A OCLUSÃO NO TEMPO** — a sonda que mede o quadro de MOVIMENTO a girar
//! (`ph2d_field_gpu::ceu_tempo`; ordem do dono de 2026-09-29: *«o render ainda não está em tempo
//! real — somos uma game engine»*).
//!
//! Para cada cena: um quadro ASSENTE (grava o histórico) e depois uma rotação de `QUADROS` quadros de
//! movimento, `PASSO_GRAUS` cada à volta do eixo vertical. Mede o relógio de cada quadro com o
//! histórico e o do produto de antes (a oclusão a passo `2`), e no fim compara a imagem com a
//! oclusão EXACTA (passo `1`, sem histórico) na mesma câmara.

use super::super::super::super::contexto;

const QUADROS: usize = 16;
const PASSO_GRAUS: f32 = 3.0;

fn cena_e_luz(
    cena: u32,
) -> (
    ph2d_field::FieldDoc,
    ph2d_field_eval::hybrid::Registry,
    [ph2d_field_render::PointLamp; 1],
    Option<ph2d_field_render::Ground>,
) {
    let doc = crate::smoke::scene(cena);
    let reg = crate::smoke::sampled_registry();
    // ⚠️ A luz fica FIXA em mundo: orbitar não pode trocar a luz, senão o quadro mediria outra cena.
    let luz = [crate::gpu_frame::tests_lampada(
        &ph2d_field_render::Orbit::default(),
    )];
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    (doc, reg, luz, chao)
}

#[allow(clippy::too_many_arguments)]
fn quadro(
    t: &crate::gpu_frame::SharedTracer,
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &ph2d_field_render::Orbit,
    luz: &[ph2d_field_render::PointLamp],
    chao: Option<ph2d_field_render::Ground>,
    assente: bool,
    sonda: crate::gpu_frame::Sonda,
    w: u32,
    h: u32,
) -> (Vec<u8>, f64) {
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let pres = ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default());
    let t0 = std::time::Instant::now();
    let img = crate::gpu_frame::paint_com(
        t,
        doc,
        reg,
        cam,
        luz,
        &surfaces,
        &pres,
        [40, 40, 40, 255],
        chao,
        w,
        h,
        assente,
        sonda,
    )
    .expect("o pintor")
    .rgba;
    (img, t0.elapsed().as_secs_f64() * 1e3)
}

fn mediana(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// ⏱️ **Sonda: o quadro de movimento a GIRAR, com e sem o histórico da oclusão.**
#[test]
#[ignore = "sonda de GPU"]
fn diag_o_ceu_no_tempo() {
    const W: u32 = 1920;
    const H: u32 = 1080;
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador");
        return;
    };
    let cenas: Vec<u32> = std::env::var("PH2D_SONDA_CENAS")
        .ok()
        .map(|v| v.split(',').filter_map(|c| c.trim().parse().ok()).collect())
        .unwrap_or_else(|| vec![28, 5, 11, 30]);
    println!(
        "\n  {}\n  cena · antes (passo 2) ms · NO TEMPO ms [mediana dos {QUADROS}] · 1.º quadro · SEM CÉU ms · contra a exacta: canais >2 B · >8 B · máx B",
        contexto()
    );
    let com = crate::gpu_frame::Sonda {
        ceu_no_tempo: true,
        ..crate::gpu_frame::Sonda::default()
    };
    let antes = crate::gpu_frame::Sonda {
        ceu_no_tempo: false,
        ..crate::gpu_frame::Sonda::default()
    };
    // `PH2D_SONDA_ABLA=ricochete,bordas` tira também esses do chão — a escada de ablação.
    let abla = std::env::var("PH2D_SONDA_ABLA").unwrap_or_default();
    let sem_ceu = crate::gpu_frame::Sonda {
        sem_ceu: true,
        ricochete: !abla.contains("ricochete"),
        bordas: !abla.contains("bordas"),
        ..crate::gpu_frame::Sonda::default()
    };
    let exacta = crate::gpu_frame::Sonda {
        ceu_no_tempo: false,
        ceu_passo: 1,
        ..crate::gpu_frame::Sonda::default()
    };
    // ⚠️ A sequência corre `REPETE` vezes e fica o MÍNIMO das medianas: a placa é partilhada com o
    // ecrã do dono e com outras janelas, e uma corrida só variou `±30 %` entre duas iguais.
    let repete = std::env::var("PH2D_SONDA_REPETE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(3)
        .max(1);
    for cena in cenas {
        let (doc, reg, luz, chao_da_cena) = cena_e_luz(cena);
        let mut melhor = (f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut chao = f64::INFINITY;
        let mut ultima = Vec::new();
        let mut cam = ph2d_field_render::Orbit::default();
        let mut minimos: std::collections::BTreeMap<&'static str, (f64, u32)> =
            std::collections::BTreeMap::new();
        for _ in 0..repete {
            cam = ph2d_field_render::Orbit::default();
            if let Ok(mut g) = t.lock() {
                g.esquece_o_ceu();
            }
            // Aquecer: compilar os pipelines das três leis fora do relógio.
            // ⚠️ `com` DUAS vezes: o 1.º grava (não há tabela) e só o 2.º compila os passes do movimento.
            for s in [com, com, antes, exacta] {
                let _ = quadro(t, &doc, &reg, &cam, &luz, chao_da_cena, false, s, W, H);
            }
            let _ = quadro(t, &doc, &reg, &cam, &luz, chao_da_cena, true, com, W, H);
            let reinicios = || t.lock().map_or(0, |g| g.ceu_tempo_reinicios());
            let r0 = reinicios();
            // ⏱️ Esvaziar o relógio por passe: o relatório abaixo é SÓ dos quadros de movimento.
            let relogio = || {
                t.lock()
                    .map_or_else(|_| Vec::new(), |mut g| g.cronometro_relatorio())
            };
            let _ = relogio();
            let mut tempos = Vec::new();
            let mut antes_ms = Vec::new();
            let quadros = std::env::var("PH2D_SONDA_QUADROS")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(QUADROS);
            for _ in 0..quadros {
                let graus = std::env::var("PH2D_SONDA_GRAUS")
                    .ok()
                    .and_then(|v| v.parse::<f32>().ok())
                    .unwrap_or(PASSO_GRAUS);
                // `PH2D_SONDA_ZOOM=<factor>` troca o giro por um ZOOM (`0,97` aproxima, `1,03` afasta).
                match std::env::var("PH2D_SONDA_ZOOM")
                    .ok()
                    .and_then(|v| v.parse::<f32>().ok())
                {
                    Some(z) => cam.half_extent *= z,
                    None => cam.turn_world([0.0, 1.0, 0.0], graus.to_radians()),
                }
                let (img, ms) = quadro(t, &doc, &reg, &cam, &luz, chao_da_cena, false, com, W, H);
                tempos.push(ms);
                ultima = img;
            }
            println!(
                "  (cena {cena}: o histórico recomeçou {} vezes em {QUADROS} quadros; quadro a quadro ms: {})",
                reinicios() - r0,
                tempos
                    .iter()
                    .map(|t| format!("{t:.1}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            // ⏱️ O MÍNIMO por passe entre as repetições: a placa é partilhada com o ecrã e a mesma
            // corrida varia `±20 %` de uma volta para a outra.
            for (rotulo, ms, n) in relogio() {
                let e = minimos.entry(rotulo).or_insert((f64::INFINITY, n));
                e.0 = e.0.min(ms);
            }
            for _ in 0..5 {
                antes_ms
                    .push(quadro(t, &doc, &reg, &cam, &luz, chao_da_cena, false, antes, W, H).1);
            }
            let mut sem = Vec::new();
            for _ in 0..5 {
                sem.push(
                    quadro(
                        t,
                        &doc,
                        &reg,
                        &cam,
                        &luz,
                        chao_da_cena,
                        false,
                        sem_ceu,
                        W,
                        H,
                    )
                    .1,
                );
            }
            chao = chao.min(mediana(sem));
            melhor.0 = melhor.0.min(mediana(antes_ms));
            melhor.1 = melhor.1.min(mediana(tempos.clone()));
            melhor.2 = melhor.2.min(tempos[0]);
        }
        if !minimos.is_empty() {
            let soma: f64 = minimos
                .iter()
                .filter(|p| {
                    !p.0.starts_with("n-") && !p.0.starts_with("espera") && !p.0.starts_with("cpu")
                })
                .map(|p| p.1.0)
                .sum();
            println!(
                "  (cena {cena}: placa por passe, MÍNIMO de {repete} — soma dos passes {soma:.2} ms)"
            );
            for (rotulo, (ms, n)) in &minimos {
                if rotulo.starts_with("n-") {
                    println!("      {rotulo:>14} · {ms:>9.0} por quadro · ×{n}");
                } else {
                    println!("      {rotulo:>14} · {ms:>7.3} ms · ×{n}");
                }
            }
        }
        let (ref_img, _) = quadro(t, &doc, &reg, &cam, &luz, chao_da_cena, false, exacta, W, H);
        if let Ok(dir) = std::env::var("PH2D_SONDA_DIR") {
            for (nome, img) in [("tempo", &ultima), ("exacta", &ref_img)] {
                let mut ppm = format!("P6\n{W} {H}\n255\n").into_bytes();
                for px in img.as_chunks::<4>().0 {
                    ppm.extend_from_slice(&px[..3]);
                }
                std::fs::write(format!("{dir}/ceu_{cena}_{nome}.ppm"), ppm).expect("grava");
            }
        }
        let d: Vec<u8> = ultima
            .iter()
            .zip(&ref_img)
            .map(|(a, b)| a.abs_diff(*b))
            .collect();
        let a2 = d.iter().filter(|x| **x > 2).count();
        let a8 = d.iter().filter(|x| **x > 8).count();
        let max = d.iter().copied().max().unwrap_or(0);
        println!(
            "  {cena:>4} · {:>9.1} · {:>9.1} · {:>7.1} · {chao:>7.1} · {a2:>9} · {a8:>7} · {max:>5}",
            melhor.0, melhor.1, melhor.2
        );
    }
}
