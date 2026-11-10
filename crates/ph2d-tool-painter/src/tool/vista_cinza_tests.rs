//! B10.A1 — a vista cinza da máscara pela caixa suja é a vista inteira ao bit, e um traço sobe só a caixa.

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};

const L: u32 = 96;

/// O oráculo: o laço serial de tela inteira de antes (verbatim do `layers/model.rs`).
fn oraculo(t: &PainterTool) -> Option<Vec<u8>> {
    let src = t.mask_grayscale_view_source()?;
    let (w, h) = t.source_size;
    let n = (w as usize) * (h as usize);
    let mut out = vec![0u8; n * 4];
    for i in 0..n {
        let g = (crate::compositor::mask_value(src, i) * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8;
        out[i * 4] = g;
        out[i * 4 + 1] = g;
        out[i * 4 + 2] = g;
        out[i * 4 + 3] = 255;
    }
    Some(out)
}

fn traco(t: &mut PainterTool, de: [f32; 2], a: [f32; 2]) {
    let p = |pos, phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(de, PointerPhase::Down));
    t.on_canvas_pointer(p(
        [(de[0] + a[0]) * 0.5, (de[1] + a[1]) * 0.5],
        PointerPhase::Move,
    ));
    t.on_canvas_pointer(p(a, PointerPhase::Move));
    t.on_canvas_pointer(p(a, PointerPhase::Up));
}

/// O dreno e a comparação com o oráculo; devolve a caixa que subiu.
fn drena_e_confere(t: &mut PainterTool, passo: &str) -> Option<(u32, u32, u32, u32)> {
    let (buf, w, h) = t.take_preview_arc().expect("o olho aberto drena");
    assert_eq!((w, h), (L, L));
    assert_eq!(
        t.preview_drain_diag().0,
        DrainBranch::Gray,
        "{passo}: o ramo"
    );
    let o = oraculo(t).expect("a máscara à vista");
    let dif = buf.iter().zip(&o).filter(|(a, b)| a != b).count();
    assert_eq!(dif, 0, "{passo}: {dif} bytes ≠ do cinza inteiro");
    t.take_preview_upload_bbox()
}

#[test]
fn a_vista_cinza_pela_caixa_e_a_inteira_ao_bit() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (L * L * 4) as usize], L, L);
    t.add_raster_layer_with_pixels("A", vec![120u8; (L * L * 4) as usize])
        .expect("A");
    let mascara = t.add_mask_to_active().expect("a máscara activa");
    t.toggle_mask_view_grayscale(mascara);
    assert_eq!(
        drena_e_confere(&mut t, "1.º quadro"),
        None,
        "o 1.º sobe inteiro"
    );

    let mut caixas = Vec::new();
    for (k, (de, a)) in [
        ([10.0, 10.0], [30.0, 14.0]),
        ([60.0, 70.0], [80.0, 50.0]),
        ([5.0, 90.0], [90.0, 92.0]),
    ]
    .into_iter()
    .enumerate()
    {
        traco(&mut t, de, a);
        caixas.push(drena_e_confere(&mut t, &format!("traço {k}")));
    }
    assert!(t.undo_last(), "controlo: há o que desfazer");
    drena_e_confere(&mut t, "Ctrl+Z");
    traco(&mut t, [40.0, 40.0], [44.0, 46.0]);
    caixas.push(drena_e_confere(&mut t, "traço depois do Ctrl+Z"));
    // Fechar e reabrir o olho: o cache normal não serve de cinza.
    t.toggle_mask_view_grayscale(mascara);
    t.take_preview_arc();
    t.toggle_mask_view_grayscale(mascara);
    assert_eq!(drena_e_confere(&mut t, "reaberto"), None);

    let inteiras = caixas.iter().filter(|c| c.is_none()).count();
    assert_eq!(
        inteiras,
        0,
        "{inteiras} de {} traços subiram a tela inteira (queria só a caixa)",
        caixas.len()
    );
}

/// Sonda (`--profile smoke`, `--ignored`): o cinza inteiro a 4K × a caixa de um traço, intercalados, mín. de 7.
#[test]
#[ignore = "sonda de tempo"]
fn sonda_vista_cinza_4k() {
    let (w, h) = (3840u32, 2160u32);
    let src = vec![77u8; (w * h * 4) as usize];
    let mut out = vec![0u8; src.len()];
    let caixa = Region {
        x: 1800,
        y: 1000,
        w: 96,
        h: 96,
    };
    let (mut inteira, mut pela_caixa) = (f64::MAX, f64::MAX);
    for _ in 0..7 {
        let t0 = std::time::Instant::now();
        super::vista_cinza::cinza_na_caixa(&src, w, &mut out, Region { x: 0, y: 0, w, h });
        inteira = inteira.min(t0.elapsed().as_secs_f64() * 1e3);
        let t0 = std::time::Instant::now();
        super::vista_cinza::cinza_na_caixa(&src, w, &mut out, caixa);
        pela_caixa = pela_caixa.min(t0.elapsed().as_secs_f64() * 1e3);
    }
    eprintln!("[vista-cinza 4K] inteira {inteira:.2} ms · caixa 96² {pela_caixa:.3} ms");
}
