//! ⭐ **A cauda (L5) com as pontas da L18/L19** (revisão do bloco L7/L8/L5, achados 2 e 4): a re-pintura da cauda
//! ao levantar entra pela porta do DESPACHO — as corridas da ponta de N imagens chegam a ela — e re-anda o trilho
//! da ponta de píxel com o lote ANCORADO que o pen-up gravou: o último píxel do pixel-perfect não se perde.

use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{LinhaDaPonta, PaintMedia, PainterTool};

const W: usize = 640;
const H: usize = 80;
const LADO: u32 = 24;
/// Um recto horizontal em y = 40, de x 20 a ~620, numa tela branca; a tela depois do pen-up.
fn recto(mut t: PainterTool) -> Vec<u8> {
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, 40.0],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(20.0, PointerPhase::Down));
    let mut x = 20.0;
    while x < 620.0 {
        x += 4.0;
        t.on_canvas_pointer(cp(x, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(x, PointerPhase::Up));
    t.run_full().0
}

/// Uma imagem `LADO × LADO` preta: o quadrado cheio (`barra = false`) ou só a BARRA horizontal do meio (um terço).
fn silhueta(barra: bool) -> Vec<u8> {
    (0..LADO * LADO)
        .flat_map(|i| {
            let y = i / LADO;
            let dentro = !barra || (LADO / 3..2 * LADO / 3).contains(&y);
            if dentro { [0, 0, 0, 255] } else { [0, 0, 0, 0] }
        })
        .collect()
}

/// As colunas de `x0..x1` em que só a barra pinta: o meio do traço com tinta e as linhas de cima sem.
fn colunas_de_barra(px: &[u8], x0: usize, x1: usize) -> usize {
    let tinta = |x: usize, y: usize| px[(y * W + x) * 4 + 3] > 0 && px[(y * W + x) * 4] < 200;
    (x0..x1)
        .filter(|&x| tinta(x, 40) && !tinta(x, 33) && !tinta(x, 47))
        .count()
}

/// ⭐ **A cauda da ponta de N imagens guarda as imagens**: em SEQUÊNCIA (quadrado, barra, quadrado, barra…), a cauda
/// re-pintada ao levantar (ponta romba: não estreita, só re-carimba) ainda tem as dabs de BARRA. Vermelho antes da
/// cura: a re-pintura saltava o despacho (as corridas por imagem) e a cauda saía toda com a imagem 0, o quadrado.
#[test]
fn a_cauda_da_ponta_de_n_imagens_guarda_as_imagens() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_size_px(10.0);
    t.set_brush_spacing(1.0);
    t.set_brush_color_srgb8([0, 0, 0]);
    t.set_brush_shape_image_rgba(&silhueta(false), LADO, LADO, None);
    t.toggle_brush_shape_alpha_from_image();
    t.add_brush_shape_hose_image_rgba(&silhueta(true), LADO, LADO);
    t.set_ponta(LinhaDaPonta::HoseMode, 1.0); // em SEQUÊNCIA
    t.set_brush_taper_end(8.0);
    t.set_brush_taper_tip_end(1.0);
    let px = recto(t);
    let corpo = colunas_de_barra(&px, 60, 300);
    let cauda = colunas_de_barra(&px, 480, 610);
    assert!(corpo > 0, "fixtura: o corpo tem dabs de barra");
    assert!(
        cauda > 0,
        "a cauda saiu só com a imagem 0 (o quadrado): corpo {corpo} · cauda {cauda}"
    );
}

/// Uma ESCADA (de (20, 10) a (620, 70): um degrau a cada 10 px) — o pixel-perfect segura sempre o último píxel à
/// espera do seguinte; a tela depois do pen-up.
fn escada(mut t: PainterTool) -> Vec<u8> {
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, 10.0 + (x - 20.0) * 0.1],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(20.0, PointerPhase::Down));
    let mut x = 20.0;
    while x < 617.0 {
        x += 3.0;
        t.on_canvas_pointer(cp(x, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(x, PointerPhase::Up));
    t.run_full().0
}

/// ⭐ **Pixel-perfect + cauda: o último píxel sai**. Com a ponta da cauda ROMBA a re-pintura não estreita: a escada
/// com cauda é a escada sem cauda píxel a píxel, o fim do trilho incluído. Vermelho antes da cura: a re-pintura andava o trilho
/// com `fim = false` e o píxel pendente nunca saía.
#[test]
fn a_cauda_da_ponta_de_pixel_guarda_o_ultimo_pixel() {
    let ultimo = |end: f32| {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
        t.set_paint_media(PaintMedia::Digital);
        t.set_brush_size_px(0.5); // 1 px: o pixel-perfect só age no diâmetro 1
        t.set_ponta(LinhaDaPonta::Pixel, 2.0);
        // A ponta da cauda ROMBA (1): a cauda re-pinta-se ao levantar mas não estreita — o que mudar no fim é do
        // trilho, não da largura.
        t.set_brush_taper_end(end);
        t.set_brush_taper_tip_end(1.0);
        let px = escada(t);
        px.chunks_exact(4)
            .map(|p| p[0] < 128)
            .collect::<Vec<bool>>()
    };
    let (sem, com) = (ultimo(0.0), ultimo(3.0));
    let fim = |v: &[bool]| {
        (0..W)
            .filter(|&x| (0..H).any(|y| v[y * W + x]))
            .max()
            .unwrap()
    };
    assert_eq!(fim(&com), fim(&sem), "a cauda perdeu o fim do trilho");
    let n = com.iter().zip(&sem).filter(|(a, b)| a != b).count();
    assert_eq!(
        n, 0,
        "a cauda romba re-pintou {n} píxeis diferentes do traço sem cauda"
    );
}
