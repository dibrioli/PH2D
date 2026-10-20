//! **Sonda (ignorada): o MESMO gesto nos quatro meios, gravado em PPM** — a metade NOSSA da comparação
//! com o oráculo Rebelle 8 (`docs/Painter/NewFeaturesWetPaint/`, 2026-10-07).
//!
//! Os gestos são os de `sonda_gestos` (os mesmos que o Rebelle recebe pela porta WebSocket). A régua
//! é a mesma dos dois lados: `docs/Painter/NewFeaturesWetPaint/ferramentas/mede_traco.py`.
//!
//! ```text
//! PH2D_SONDA_OUT=/tmp/x PH2D_SONDA_SIZE=28 cargo test -p ph2d-tool-painter --profile smoke --test it \
//!     sonda_tracos_por_meio -- --ignored --nocapture
//! ```
//!
//! Escreve `<meio>.ppm` (P6) e imprime o FNV-1a de cada um — o PNG faz-se fora, com `magick`.
//!
//! **Por preset do Wet Painter** (W1): `PH2D_SONDA_PRESET=<nome>|todos` pinta a mesma cena com cada
//! preset de fábrica (`assets/brushes/WetPainter/`), no relógio fixo, e escreve `<nome>.ppm`:
//!
//! ```text
//! PH2D_SONDA_OUT=/tmp/x PH2D_SONDA_PRESET=todos cargo test -p ph2d-tool-painter --profile smoke \
//!     --test it sonda_tracos_por_meio::os_presets_do_wet_painter -- --ignored --nocapture
//! ```
//! O preset que não fixa o tamanho pinta ao raio `PH2D_SONDA_SIZE` (ou 40, o dos meios).

use super::sonda_gestos::{fnv1a, pinta, pinta_cena, tela};
use ph2d_tool_painter::PaintMedia;
use std::io::Write;

/// O tamanho de fábrica da sonda; `PH2D_SONDA_SIZE` sobrepõe-no (o Rebelle recebe `size_px` 40).
const BRUSH_PX: f32 = 40.0;

fn escreve_ppm(path: &std::path::Path, rgba: &[u8], w: u32, h: u32) {
    assert_eq!(rgba.len(), (w * h * 4) as usize, "run_full devolve RGBA8");
    let mut f = std::fs::File::create(path).expect("ppm");
    write!(f, "P6\n{w} {h}\n255\n").unwrap();
    let mut rgb = Vec::with_capacity((w * h * 3) as usize);
    for px in rgba.as_chunks::<4>().0 {
        rgb.extend_from_slice(&px[..3]);
    }
    f.write_all(&rgb).unwrap();
}

#[test]
#[ignore = "sonda: grava PPMs para a comparação com o oráculo; corre-se à mão"]
fn os_mesmos_gestos_nos_quatro_meios() {
    let out = std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| {
        std::env::temp_dir()
            .join("ph2d_sonda_tracos")
            .display()
            .to_string()
    });
    let size = std::env::var("PH2D_SONDA_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(BRUSH_PX);
    std::fs::create_dir_all(&out).unwrap();
    for (nome, meio) in [
        ("digital", PaintMedia::Digital),
        ("watercolor", PaintMedia::Watercolor),
        ("impasto", PaintMedia::Impasto),
        ("wetpaint", PaintMedia::WetPaint),
    ] {
        let t0 = std::time::Instant::now();
        let (px, w, h) = pinta(meio, size);
        let p = std::path::Path::new(&out).join(format!("{nome}.ppm"));
        escreve_ppm(&p, &px, w, h);
        println!(
            "[sonda] {nome:<10} {w}x{h} fnv {:#018x} → {} ({:.1} s)",
            fnv1a(&px),
            p.display(),
            t0.elapsed().as_secs_f32()
        );
    }
}

#[test]
#[ignore = "sonda: grava um PPM por preset do Wet Painter; corre-se à mão"]
fn os_presets_do_wet_painter() {
    let Ok(qual) = std::env::var("PH2D_SONDA_PRESET") else {
        println!("[sonda] PH2D_SONDA_PRESET=<nome>|todos");
        return;
    };
    let out = std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| {
        std::env::temp_dir()
            .join("ph2d_sonda_presets")
            .display()
            .to_string()
    });
    let size = std::env::var("PH2D_SONDA_SIZE")
        .ok()
        .and_then(|v| v.parse::<f32>().ok());
    std::fs::create_dir_all(&out).unwrap();
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let bib = ph2d_brush_presets::Biblioteca::carregar(&raiz, None);
    assert!(bib.erros.is_empty(), "{:?}", bib.erros);
    let mut n = 0;
    // Só os do Wet Paint (os outros meios têm presets irmãos desde a J3).
    for e in bib.entradas.iter().filter(|e| e.preset.parametros.media == ph2d_tool_painter::MEIO_DO_PRESET) {
        let nome = &e.preset.parametros.name;
        if qual != "todos" && qual != *nome {
            continue;
        }
        n += 1;
        let mut t = tela();
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(&e.preset).is_empty());
        if let Some(r) = size.or(e
            .preset
            .parametros
            .volumes
            .size_px
            .is_none()
            .then_some(BRUSH_PX))
        {
            t.set_brush_size_px(r);
        }
        let t0 = std::time::Instant::now();
        let (px, w, h) = pinta_cena(&mut t);
        let p = std::path::Path::new(&out).join(format!("{nome}.ppm"));
        escreve_ppm(&p, &px, w, h);
        println!(
            "[sonda] {nome:<14} {w}x{h} fnv {:#018x} → {} ({:.1} s)",
            fnv1a(&px),
            p.display(),
            t0.elapsed().as_secs_f32()
        );
    }
    assert!(n > 0, "nenhum preset de fábrica chamado {qual:?}");
}
