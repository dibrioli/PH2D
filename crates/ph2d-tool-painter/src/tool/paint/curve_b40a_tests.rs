//! **B40a** — o Trim com Offset 0 não apaga o laço da Curva (T1-27) e o Free Hand guarda o laço
//! inteiro de uma tela 4K (B40.7). A prancha do Trim grava em `docs/Painter/missao_4k/pranchas/B40a/`.

use super::{TRIM_SEM_GUARDA, freehand_capture_cap, trim_offset_spine};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{
    CanvasPaintTool as _, CanvasPointer, PointerPhase, RasterEditTool as _, Tool as _,
};
use ph2d_painter_brush::{BrushSpec, Falloff, StrokeMethod};

/// Um «α» aberto (a reta entra, dá a volta e cruza-se a si mesma em ~(45, 56,7)) e um «8» fechado.
const ALFA: [[f32; 2]; 9] = [
    [10.0, 60.0],
    [40.0, 60.0],
    [55.0, 50.0],
    [55.0, 35.0],
    [45.0, 28.0],
    [35.0, 35.0],
    [35.0, 50.0],
    [50.0, 60.0],
    [80.0, 60.0],
];
const OITO: [[f32; 2]; 10] = [
    [20.0, 50.0],
    [35.0, 38.0],
    [48.0, 48.0],
    [52.0, 52.0],
    [70.0, 65.0],
    [85.0, 50.0],
    [70.0, 35.0],
    [52.0, 48.0],
    [48.0, 52.0],
    [35.0, 62.0],
];

#[test]
fn o_trim_com_offset_zero_nao_apaga_o_laco_da_curva() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 100 * 100 * 4], 100, 100);
    for (pts, closed) in [(&ALFA[..], false), (&OITO[..], true)] {
        t.paint.offset_trim = false;
        let pristino = t.offset_curve_spine(pts, &[], closed, 0.0);
        assert!(
            trim_offset_spine(&pristino, closed).len() + 4 < pristino.len(),
            "a fixtura cruza-se: o Trim cortaria o laço (closed={closed})"
        );
        t.paint.offset_trim = true;
        assert_eq!(
            t.offset_curve_spine(pts, &[], closed, 0.0),
            pristino,
            "Offset 0 não tem auto-interseções a cortar: o laço desenhado fica (closed={closed})"
        );
        let aparado = t.offset_curve_spine(pts, &[], closed, 3.0);
        t.paint.offset_trim = false;
        assert_ne!(
            aparado,
            t.offset_curve_spine(pts, &[], closed, 3.0),
            "com Offset ≠ 0 o Trim continua a correr (closed={closed})"
        );
    }
}

#[test]
fn o_laco_a_mao_livre_de_uma_tela_4k_nao_perde_pontos() {
    // O contorno de uma tela 3840×2160 ao espaçamento mínimo da captura (2 px): 6 000 pontos.
    let (w, h) = (3840u32, 2160u32);
    let precisa = (2 * (w + h) / 2) as usize;
    assert!(
        4096 < precisa,
        "o tecto fixo de 4 096 perdia pontos: {precisa}"
    );
    assert!(
        freehand_capture_cap((w, h)) >= 2 * precisa,
        "dois contornos cabem"
    );
    assert_eq!(
        freehand_capture_cap((512, 512)),
        4096,
        "telas pequenas ficam no piso de antes"
    );
}

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn fundo(w: u32, h: u32, blocos: bool) -> Vec<u8> {
    let cores = [
        [200u8, 60, 50],
        [60, 150, 210],
        [240, 200, 60],
        [90, 170, 90],
    ];
    let mut v = vec![255u8; (w * h * 4) as usize];
    if blocos {
        for (i, p) in v.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let (x, y) = (i as u32 % w, i as u32 / w);
            p[..3].copy_from_slice(&cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)]);
        }
    }
    v
}

/// O «α» à mão livre com Trim ligado e Offset 0, pelo caminho `antigo` (Trim sem a guarda) ou o novo.
fn cena(w: u32, h: u32, qual: u8, antigo: bool) -> Vec<u8> {
    TRIM_SEM_GUARDA.set(antigo);
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    // O mesmo desenho (1080 px de lado) nas duas telas: a 4K a Free Hand pousa-o no meio.
    let k = 10.8;
    t.paint.brush = BrushSpec {
        radius_px: 1.5 * k,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.05, 0.1, 0.5],
        space_attenuation: false,
        stabilizer: 0.0,
        stroke_method: StrokeMethod::FreeHand,
        ..Default::default()
    };
    t.paint.offset_trim = true;
    let (ox, oy) = ((w as f32 - 100.0 * k) / 2.0, (h as f32 - 100.0 * k) / 2.0);
    let at = |p: [f32; 2]| [ox + p[0] * k, oy + p[1] * k];
    t.on_canvas_pointer(cp(at(ALFA[0]), PointerPhase::Down));
    for win in ALFA.windows(2) {
        for s in 1..=4 {
            let f = s as f32 / 4.0;
            let p = [
                win[0][0] + (win[1][0] - win[0][0]) * f,
                win[0][1] + (win[1][1] - win[0][1]) * f,
            ];
            t.on_canvas_pointer(cp(at(p), PointerPhase::Move));
        }
    }
    t.on_canvas_pointer(cp(at(ALFA[8]), PointerPhase::Up));
    TRIM_SEM_GUARDA.set(false);
    t.on_tick(1000.0 / 60.0);
    t.invalidate_composite();
    t.take_preview_arc().expect("o preview").0.to_vec()
}

/// Antes | depois | o que muda (vermelho), no recorte central de lado `h`.
fn monta(a: &[u8], d: &[u8], w: u32, h: u32) -> image::RgbaImage {
    let lado = h;
    let x0 = (w - lado) / 2;
    let gap = 8u32;
    let mut par =
        image::RgbaImage::from_pixel(3 * lado + 2 * gap, lado, image::Rgba([40, 40, 40, 255]));
    for y in 0..lado {
        for x in 0..lado {
            let i = ((y * w + x0 + x) * 4) as usize;
            let m = (0..4)
                .map(|c| a[i + c].abs_diff(d[i + c]))
                .max()
                .unwrap_or(0);
            let q = 255 - (u16::from(m) * 32).min(255) as u8;
            par.put_pixel(x, y, image::Rgba([a[i], a[i + 1], a[i + 2], 255]));
            par.put_pixel(
                x + lado + gap,
                y,
                image::Rgba([d[i], d[i + 1], d[i + 2], 255]),
            );
            par.put_pixel(x + 2 * (lado + gap), y, image::Rgba([255, q, q, 255]));
        }
    }
    par
}

#[test]
#[ignore = "prancha B40a: grava as PNG do Trim em docs/Painter/missao_4k/pranchas/B40a/"]
fn prancha_b40a_trim_com_offset_zero_guarda_o_laco() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B40a"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, fundo_nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            let a = cena(w, h, qual, true);
            let d = cena(w, h, qual, false);
            let pintou = d
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|p| p[0] < 40 && p[2] > 100)
                .count();
            assert!(
                a != d,
                "{res} {fundo_nome}: o Trim antigo tem de cortar o laço ({pintou} px de tinta)"
            );
            monta(&a, &d, w, h)
                .save(format!(
                    "{dir}/{res}_{fundo_nome}_trim_offset_zero_antes_depois.png"
                ))
                .expect("png");
        }
    }
}
