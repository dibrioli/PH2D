//! **O SPRAY no Wet Paint** (J6 I-11, `docs/Painter/Pinceis/exames/aguarela_texturas_j6.md`): as gotas caem na nuvem
//! do traço, e o preset do spray grava-se dentro da fatia de memória.
//!
//! Os dois defeitos (28/10): (1) a tabela da largura de um spray lia o diâmetro de UMA gota por coluna — a promessa
//! pintava com um raio do motor ~20× o pedido, gotas grandes pela tela inteira; (2) cada marca nascia uma lane nova,
//! nunca solta, cada uma com a sua janela de rasto — a tabela (64 raios em paralelo) passava os 24 G.

use crate::familias::aguarela::EM_ESTUDO_J6;
use crate::sonda_gestos::{AZUL, Relogio, W, gesto, passa_o_tempo};
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_tool_painter::PainterTool;

/// O «Spatter Watercolor» (o spray do carimbo, 6 marcas, partícula 0,04) com o `jitter`/`jitter_scale` dados.
fn salpico(jitter: Option<(f64, f64)>) -> ph2d_brush_presets::Preset {
    let r = EM_ESTUDO_J6
        .iter()
        .find(|r| r.nome == "Spatter Watercolor")
        .expect("a receita");
    let mut p = ph2d_brush_presets::Preset {
        parametros: crate::presets_de_fabrica::parametros(r),
        origem: ph2d_brush_presets::Origem::Fabrica,
        miniatura: ph2d_brush_presets::Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![255; 4],
        },
        texturas: Default::default(),
    };
    if let Some((j, s)) = jitter {
        p.parametros
            .brush
            .insert("jitter".into(), serde_json::json!(j));
        p.parametros
            .brush
            .insert("jitter_scale".into(), serde_json::json!(s));
    }
    p
}

/// Um recto horizontal de raio `raio` (o do artista) a meio de uma tela 1000×640.
fn recto(p: &ph2d_brush_presets::Preset, raio: f32) -> (Vec<u8>, u32, u32) {
    let h = 640u32;
    let cy = h as f32 * 0.5;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * h * 4) as usize], W, h);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.set_brush_size_px(raio);
    t.set_brush_color_srgb8(AZUL);
    gesto(&mut t, &[[200.0, cy], [800.0, cy]], 3.0, |_| 1.0);
    passa_o_tempo(&mut t, 60, Relogio::DeUmaVez);
    t.run_full()
}

/// As distâncias à linha do traço de cada píxel com tinta (`> 8` em algum canal), ordenadas.
fn distancias(px: &[u8], w: u32, h: u32) -> Vec<f32> {
    let cy = h as f32 * 0.5;
    let mut d: Vec<f32> = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let i = ((y * w + x) * 4) as usize;
            px[i..i + 3].iter().any(|&v| 255 - v > 8)
        })
        .map(|(_, y)| (y as f32 - cy).abs())
        .collect();
    d.sort_by(f32::total_cmp);
    d
}

/// ⭐ **A gota cai na nuvem do traço** (defeito 1): o preset do spray com a tabela que o GERADOR mede (aqui seis raios
/// do motor, os da faixa) pinta o traço de 40 px com quase toda a tinta a menos de 2 larguras da linha. Medido 28/10:
/// com a régua por coluna (a de antes) a tabela dava ~8 px ao raio 40 e a tinta caía a ~300 px (nada perto do traço).
#[test]
fn a_gota_do_spray_cai_na_nuvem_do_traco() {
    let mut p = salpico(None);
    p.parametros.width_table = [2.0f32, 4.0, 8.0, 16.0, 32.0, 64.0]
        .iter()
        .map(|&r| {
            [
                f64::from(r),
                f64::from(crate::presets_de_fabrica::largura_de(&p, r, true)),
            ]
        })
        .collect();
    let raio = 20.0;
    let (px, w, h) = recto(&p, raio);
    let d = distancias(&px, w, h);
    assert!(d.len() > 200, "o traço pinta ({} px com tinta)", d.len());
    let p99 = d[d.len() * 99 / 100];
    println!(
        "[gota] tabela {:?} · {} px · p99 {p99} · max {}",
        p.parametros.width_table,
        d.len(),
        d[d.len() - 1]
    );
    assert!(
        p99 <= 4.0 * raio,
        "99 % da tinta a ≤ 2 larguras (80 px) do traço de 40 px: p99 {p99}"
    );
}

/// O pico de memória do processo (`VmHWM`), MiB.
fn pico_mib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .unwrap()
        .lines()
        .find_map(|l| l.strip_prefix("VmHWM:"))
        .and_then(|v| v.trim().trim_end_matches("kB").trim().parse::<u64>().ok())
        .map_or(0, |k| k / 1024)
}

/// ⭐ **As gotas não juntam rastos** (defeito 2): o recto que mais marcas pousa na tabela (raio do motor 4, cru) com o
/// jitter alto (0,8 · 0,95) deixa o motor com UM rasto de lane. Medido 28/10: antes 645 rastos (≥ 123² células × os
/// planos cada, ~0,35 MiB) e 277 MiB de pico só nesse raio — 64 raios em paralelo passavam a fatia de 24 G (SIGKILL,
/// 3/3); depois 1 rasto, 31 MiB, e a tabela inteira com pico ≤ 4,1 GiB.
#[test]
fn as_gotas_do_spray_nao_juntam_rastos() {
    let p = salpico(Some((0.8, 0.95)));
    let raio = 4.0;
    let h = ((raio * 6.0) as u32 + 80).max(120);
    let cy = h as f32 * 0.5;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * h * 4) as usize], W, h);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&p).is_empty());
    t.set_wet_largura_crua(1.0);
    t.set_brush_size_px(raio);
    t.set_brush_color_srgb8(AZUL);
    gesto(&mut t, &[[100.0, cy], [900.0, cy]], 3.0, |_| 1.0);
    let mut janelas = Vec::new();
    t.wet_sonda_motor(|e| {
        janelas = e
            .lane_trails
            .iter()
            .map(ph2d_wet_paint::trail::Trail::window_half_for_measure)
            .collect();
    });
    println!(
        "[gotas] rastos {} · maior meia-largura {:?}",
        janelas.len(),
        janelas.iter().max()
    );
    assert!(
        janelas.len() <= 2,
        "uma gota pousa e solta a lane: {} rastos",
        janelas.len()
    );
}

/// A memória de UM raio cru da tabela, ou da tabela inteira (`PH2D_SONDA_TABELA`), por variante
/// (`PH2D_SONDA_JITTER` = "j,s", `PH2D_SONDA_RAIO` = "r;r;…"): um processo por variante.
#[test]
#[ignore = "sonda de memória"]
fn sonda_memoria() {
    let v = std::env::var("PH2D_SONDA_JITTER").unwrap_or_else(|_| "0.7,0.9".into());
    let (j, s) = v.split_once(',').unwrap();
    let p = salpico(Some((j.parse().unwrap(), s.parse().unwrap())));
    if std::env::var_os("PH2D_SONDA_TABELA").is_some() {
        let t0 = std::time::Instant::now();
        let t = crate::presets_de_fabrica::tabela_de_largura(&p);
        println!(
            "[mem] tabela jitter {j} escala {s}: pico {} MiB em {:.1} s; {:?}",
            pico_mib(),
            t0.elapsed().as_secs_f32(),
            t.iter().step_by(8).collect::<Vec<_>>()
        );
        return;
    }
    let raios = std::env::var("PH2D_SONDA_RAIO").unwrap_or_else(|_| "1100".into());
    for raio in raios.split(';').map(|r| r.parse::<f32>().unwrap()) {
        let t0 = std::time::Instant::now();
        let l = crate::presets_de_fabrica::largura_de(&p, raio, true);
        println!(
            "[mem] jitter {j} escala {s} raio {raio}: largura {l} pico {} MiB em {:.1} s",
            pico_mib(),
            t0.elapsed().as_secs_f32()
        );
    }
}

/// O recto do spray em PPM (`PH2D_SONDA_OUT`), a tinta longe e a mais longe; `PH2D_SONDA_PRESET` = um ficheiro.
#[test]
#[ignore = "sonda: cargo test -p ph2d-tool-painter --profile smoke --test it o_spray_do_wet::sonda -- --ignored --nocapture"]
fn sonda() {
    let out = std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| "/tmp".into());
    let raios = std::env::var("PH2D_SONDA_RAIO").unwrap_or_else(|_| "10;40".into());
    for raio in raios.split(';').map(|r| r.parse::<f32>().unwrap()) {
        let p = match std::env::var("PH2D_SONDA_PRESET") {
            Ok(f) => ph2d_brush_presets::ler(&std::fs::read(f).unwrap()).unwrap(),
            Err(_) => salpico(None),
        };
        let (px, w, h) = recto(&p, raio);
        let d = distancias(&px, w, h);
        let longe = d.iter().filter(|&&x| x > 3.0 * raio).count();
        println!(
            "[sonda] raio {raio}: tinta {} px, longe (> 3 r) {longe}, dmax {}",
            d.len(),
            d.last().copied().unwrap_or(0.0)
        );
        let mut ppm = format!("P6 {w} {h} 255\n").into_bytes();
        ppm.extend(
            px.as_chunks::<4>()
                .0
                .iter()
                .flat_map(|c| [c[0], c[1], c[2]]),
        );
        std::fs::write(format!("{out}/spray_wet_{raio}.ppm"), ppm).unwrap();
    }
}
