//! ⭐ B15.5 — a dobra através dos ajustes (Gaussiano/Nitidez) materializa-se na JANELA que a luz lê, dilatada pela
//! SOMA dos halos dos filtros; a borda que cai na borda da tela é a da tela. Ao bit com a dobra da tela inteira.

use super::super::Region;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_effects::adjustments::{
    AdjustmentKind, AdjustmentParams, GaussianBlurParams, SharpenParams,
};

/// Papel com relevo e duas cristas que se cruzam (o relevo varia em toda a janela, logo um halo curto erra nela).
fn cristas(w: u32, h: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_substrate_depth(1.0);
    t.set_substrate_roughness(0.5);
    t.toggle_brush_impasto();
    let at = |x: f32, y: f32, phase| CanvasPointer {
        pos: [x * w as f32, y * h as f32],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.set_brush_size_px(h as f32 * 0.15);
    t.on_canvas_pointer(at(0.05, 0.3, PointerPhase::Down));
    t.on_canvas_pointer(at(0.5, 0.55, PointerPhase::Move));
    t.on_canvas_pointer(at(0.95, 0.7, PointerPhase::Up));
    t.add_raster_layer("cima").expect("camada de cima");
    t.set_brush_size_px(h as f32 * 0.08);
    t.paint.brush.strength = 0.5;
    t.on_canvas_pointer(at(0.6, 0.02, PointerPhase::Down));
    t.on_canvas_pointer(at(0.45, 0.5, PointerPhase::Move));
    t.on_canvas_pointer(at(0.55, 0.98, PointerPhase::Up));
    t.reconcile_substrate();
    t
}

fn ajuste(t: &mut PainterTool, p: AdjustmentParams) {
    let kind = match p {
        AdjustmentParams::Sharpen(_) => AdjustmentKind::Sharpen,
        _ => AdjustmentKind::GaussianBlur,
    };
    let id = t.layers.add_adjustment(kind).expect("ajuste");
    t.layers.adjustment_mut(id).expect("ajuste").params = p;
}

fn gauss(radius: f32) -> AdjustmentParams {
    AdjustmentParams::GaussianBlur(GaussianBlurParams { radius })
}

fn nitidez(radius: f32, amount: f32) -> AdjustmentParams {
    AdjustmentParams::Sharpen(SharpenParams {
        amount,
        radius,
        mask_edges: false,
    })
}

/// A janela == a tela, ao bit, em cada píxel que a luz lê (a região e a moldura de 1 px), e a luz idem.
fn janela_e_a_tela(t: &PainterTool, r: Region, caso: &str) {
    let tela = t.impasto_fields().expect("campos");
    assert!(tela.folded.is_some(), "{caso}: o ajuste não dobrou");
    let jan = t.impasto_fields_para(r).expect("campos");
    let d = jan.folded.as_ref().expect("dobrou");
    let interior = r.x > 0 && r.y > 0 && r.x + r.w < t.source_size.0 && r.y + r.h < t.source_size.1;
    if interior {
        assert!(
            d.w < tela.width || d.h < tela.height,
            "{caso}: a dobra de {r:?} fez a tela inteira ({}×{})",
            d.w,
            d.h
        );
    }
    let mut lidos = 0usize;
    for y in i64::from(r.y) - 1..=i64::from(r.y + r.h) {
        for x in i64::from(r.x) - 1..=i64::from(r.x + r.w) {
            let (a, b) = (jan.height_at(x, y), tela.height_at(x, y));
            assert_eq!(
                a.to_bits(),
                b.to_bits(),
                "{caso}: altura em ({x},{y}): {a} × {b}"
            );
            let (a, b) = (jan.cover_at(x, y), tela.cover_at(x, y));
            assert_eq!(
                a.to_bits(),
                b.to_bits(),
                "{caso}: corpo em ({x},{y}): {a} × {b}"
            );
            lidos += 1;
        }
    }
    assert!(lidos > 0);
    // Um cinzento opaco: a luz o escurece ou realça onde houver relevo (a camada activa é transparente em quase tudo).
    let base = [180u8, 170, 160, 255].repeat((r.w * r.h) as usize);
    let (mut nova, mut tela_px) = (base.clone(), base.clone());
    t.apply_impasto_light(&mut nova, r);
    t.apply_impasto_light_por_pixel(&mut tela_px, r);
    assert!(nova != base, "{caso}: a fixture não acendeu nada em {r:?}");
    assert!(
        nova == tela_px,
        "{caso}: a luz da janela difere da da tela em {r:?}"
    );
}

/// ⭐⭐ **A dobra filtrada da janela dilatada é a da tela, ao bit** (B15.5). Gaussiano r 1 · 20 · 256, a Nitidez, e
/// Gaussiano + Nitidez empilhados (o halo é a SOMA); regiões no meio, encostadas à borda e no canto.
/// *Mutação (vista vermelha): o halo pelo MAIOR raio em vez da soma ⇒ o caso «empilhados» reprova.*
#[test]
fn a_dobra_filtrada_da_janela_dilatada_e_a_da_tela() {
    let (w, h) = (600u32, 560u32);
    let meio = Region {
        x: 290,
        y: 270,
        w: 24,
        h: 18,
    };
    let regioes = [
        meio,
        Region {
            x: 0,
            y: 200,
            w: 30,
            h: 20,
        },
        Region {
            x: w - 17,
            y: h - 13,
            w: 17,
            h: 13,
        },
    ];
    let casos: [(&str, Vec<AdjustmentParams>); 5] = [
        ("gauss r1", vec![gauss(1.0)]),
        ("gauss r20", vec![gauss(20.0)]),
        ("gauss r256", vec![gauss(256.0)]),
        ("nitidez r6", vec![nitidez(6.0, 1.5)]),
        (
            "empilhados",
            vec![gauss(20.0), nitidez(7.0, 2.0), gauss(1.5)],
        ),
    ];
    for (caso, ajustes) in casos {
        let mut t = cristas(w, h);
        for p in ajustes {
            ajuste(&mut t, p);
        }
        let so_o_meio = caso == "gauss r256";
        for &r in regioes.iter().take(if so_o_meio { 1 } else { 3 }) {
            janela_e_a_tela(&t, r, caso);
        }
    }
}
