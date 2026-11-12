//! B40.A5: o pincel Mask com seleção copia e recorta só a caixa dos dabs, ao bit da rota antiga (tela
//! inteira), nos quatro modos.

use super::mascara::{ROTA_ANTIGA, TEXELS_DO_LOTE};
use crate::tool::PainterTool;
use crate::tool::paint::mask::mask_probe::mask_tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn traco_mascara(t: &mut PainterTool, modo: u8, dy: f32) {
    t.set_mask_brush(modo);
    t.on_canvas_pointer(cp([300.0, 260.0 + dy], PointerPhase::Down));
    for k in 1..=12u8 {
        let f = f32::from(k);
        t.on_canvas_pointer(cp(
            [300.0 + f * 6.0, 260.0 + dy + f * 9.0],
            PointerPhase::Move,
        ));
    }
    t.on_canvas_pointer(cp([372.0, 368.0 + dy], PointerPhase::Up));
    let _ = t.take_preview_arc();
}

fn com_selecao(size: u32) -> PainterTool {
    let mut t = mask_tool(size);
    // A borda da seleção atravessa o traço: metade dos dabs cai fora dela.
    t.set_rect_selection(0, 0, 330, size);
    t
}

/// Por lote, texels copiados para o recorte + texels revistos pelo recorte: a pegada dos dabs, nunca a tela.
#[test]
fn o_mask_com_selecao_copia_e_recorta_so_a_caixa_dos_dabs() {
    let size = 1024u32;
    let mut t = com_selecao(size);
    TEXELS_DO_LOTE.with(|c| c.set(0));
    traco_mascara(&mut t, 0, 0.0);
    let pior = TEXELS_DO_LOTE.with(std::cell::Cell::get);
    let tela = (size * size) as usize;
    assert!(
        pior > 0,
        "controle: o traço com seleção passou pela rota do recorte"
    );
    // Controle (o vermelho de antes): a rota antiga copia e recorta a tela inteira por lote.
    let mut antigo = com_selecao(size);
    TEXELS_DO_LOTE.with(|c| c.set(0));
    ROTA_ANTIGA.with(|c| c.set(true));
    traco_mascara(&mut antigo, 0, 0.0);
    ROTA_ANTIGA.with(|c| c.set(false));
    assert_eq!(TEXELS_DO_LOTE.with(std::cell::Cell::get), 2 * tela);
    assert!(
        pior * 16 <= tela,
        "o pior lote mexeu em {pior} texels (copiados + recortados) numa tela de {tela}"
    );
}

#[test]
fn o_mask_com_selecao_na_caixa_e_a_rota_antiga_ao_bit() {
    for modo in 0..=3u8 {
        let mut novo = com_selecao(512);
        let mut antigo = com_selecao(512);
        // Uma demão de Paint ao lado, para Erase/Blur/Smear terem o que mexer.
        traco_mascara(&mut novo, 0, 24.0);
        traco_mascara(&mut novo, modo, 0.0);
        ROTA_ANTIGA.with(|c| c.set(true));
        traco_mascara(&mut antigo, 0, 24.0);
        traco_mascara(&mut antigo, modo, 0.0);
        ROTA_ANTIGA.with(|c| c.set(false));
        let (a, b) = (
            &novo.paint.mask_scratch_rgba,
            &antigo.paint.mask_scratch_rgba,
        );
        assert!(
            a.iter().any(|&v| v < 128),
            "controle: o modo {modo} deixou alguma protecção"
        );
        assert!(
            a[..] == b[..],
            "modo {modo}: o rascunho difere da rota antiga"
        );
        assert!(
            novo.canvas_rgba[..] == antigo.canvas_rgba[..],
            "modo {modo}: a tela difere"
        );
    }
}

/// ms por traço (12 lotes) com Mask + seleção a 4096², rota antiga × caixa intercaladas, mín. de 7.
#[test]
#[ignore = "sonda de tempo"]
fn sonda_mask_com_selecao_4096() {
    let mut t = com_selecao(4096);
    let (mut antiga, mut caixa) = (f64::MAX, f64::MAX);
    for _ in 0..7 {
        for velha in [true, false] {
            ROTA_ANTIGA.with(|c| c.set(velha));
            let t0 = std::time::Instant::now();
            traco_mascara(&mut t, 0, 0.0);
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            let m = if velha { &mut antiga } else { &mut caixa };
            *m = m.min(ms);
        }
    }
    ROTA_ANTIGA.with(|c| c.set(false));
    println!("Mask+seleção 4096², traço de 12 lotes: antiga {antiga:.1} ms · caixa {caixa:.1} ms");
}
