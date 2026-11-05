//! **O anel do cursor é o que a Borracha toca** (smoke do dono 05/11): ela herdava a tabela de largura da tinta do Wet
//! Round, e tocava 49 px com anel de 30. As ferramentas da água têm a tabela delas, medida no preset.
use super::same_as_paper_volume_tests::{H, W, ponto, traco_com};
use crate::tool::PainterTool;
use crate::tool::paint::WetTool;
use crate::tool::paint::wetpaint_preset::de_fabrica_ou_candidato;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};

#[test]
fn o_raio_da_borracha_e_o_do_anel_e_o_das_outras_e_o_da_lei() {
    for (fio, com_lei) in [
        ("eraser", false),
        ("wet_paint", true),
        (WetTool::Smear.fio(), true),
        (WetTool::Blend.fio(), true),
    ] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
        t.set_paint_tool_mode("wetpaint");
        t.set_paint_tool_mode(fio);
        t.set_brush_size_px(30.0);
        let igual = t.stroke_spec().radius_px == t.cursor_dab().0;
        assert_eq!(igual, !com_lei, "{fio}");
    }
}

/// De ponta a ponta: um clique da Borracha não muda nada fora do círculo, contra a mesma tinta sem o clique.
#[test]
fn o_clique_da_borracha_fica_dentro_do_anel() {
    const RAIO: f32 = 15.0;
    let tinta = || {
        let (_, mut t) = traco_com(&de_fabrica_ou_candidato("Gouache Opaque"), |_| {});
        for _ in 0..900 {
            t.on_tick(1000.0 / 60.0);
        }
        t
    };
    let (mut t, mut controlo) = (tinta(), tinta());
    t.set_paint_tool_mode("eraser");
    t.set_brush_size_px(RAIO);
    t.on_canvas_pointer(ponto(200.0, PointerPhase::Down));
    t.on_canvas_pointer(ponto(200.0, PointerPhase::Up));
    for _ in 0..8 {
        t.on_tick(1000.0 / 60.0);
        controlo.on_tick(1000.0 / 60.0);
    }
    let (mut fora, mut dentro) = (0, 0);
    for (i, (a, b)) in t
        .canvas_rgba
        .chunks(4)
        .zip(controlo.canvas_rgba.chunks(4))
        .enumerate()
    {
        if a.iter().zip(b).any(|(p, q)| p.abs_diff(*q) > 2) {
            let (dx, dy) = ((i % W) as f32 - 200.0, (i / W) as f32 - 120.0);
            if dx.hypot(dy) > RAIO + 1.5 {
                fora += 1;
            } else {
                dentro += 1;
            }
        }
    }
    assert!(dentro > 0, "premissa: a Borracha apagou");
    assert_eq!(fora, 0, "píxeis mudados fora do anel ({dentro} dentro)");
}
