//! ⭐ **As linhas do traço como camada e do fim do traço viajam na macro** (L5 · L7 · L8, ADR-0186): o tecto
//! (Glaze), as orlas molhada/queimada e a largura, o «On empty», a cauda (`end`, `tip_end`) e o fall-off são
//! portas INTERNAS (`macro_track/portas_internas.rs`) — o painel chega-lhes pelo `handle_panel_event`, que
//! grava. Este gate prova a decisão pelos dois lados: postas ANTES de armar, viajam no estado inicial (o
//! preset); postas A GRAVAR, viajam como passos. Nos dois casos o tool que toca acaba com o MESMO pincel e a
//! MESMA tela ao byte. Controlo: sem as linhas a tela é outra (o caso mede-as).

use super::macro_comum::{Gravador, cp, texels_diferentes, toca};
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase, RasterEditTool, Tool};
use ph2d_tool_painter::{PaintMedia, PainterTool, ids};

const N: u32 = 160;

fn ferramenta() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_size_px(14.0);
    t.set_brush_color_srgb8([200, 40, 30]);
    t
}

fn quadro(t: &mut PainterTool, g: Option<&mut Gravador>) {
    let _ = t.take_preview_arc();
    t.on_tick(1000.0 / 60.0);
    if let Some(g) = g {
        g.quadro(t);
    }
}

/// As nove linhas, pela porta do painel (a do produto), cada uma fora do seu neutro.
fn linhas(t: &mut PainterTool, mut g: Option<&mut Gravador>) {
    let mut evs: Vec<PanelEvent> = ids::PAINTER_TRACO_FIELDS
        .into_iter()
        .zip([0.3, 0.5, 0.4, 6.0, 1.0])
        .map(|(id, v)| PanelEvent::SetValue(id, v))
        .collect();
    evs.extend([
        PanelEvent::SetValue(ids::PAINTER_TAPER_END, 2.0),
        PanelEvent::SetValue(ids::PAINTER_TAPER_TIP_END, 0.2),
        PanelEvent::SetValue(ids::PAINTER_TAPER_FALL_OFF, 6.0),
        PanelEvent::SetValue(ids::PAINTER_TAPER_FALL_OFF_START, 2.0),
    ]);
    for ev in evs {
        t.handle_panel_event(ev);
        quadro(t, g.as_deref_mut());
    }
}

/// Um traço de `de` a `ate` (px), um evento por quadro.
fn traco(t: &mut PainterTool, mut g: Option<&mut Gravador>, de: [f32; 2], ate: [f32; 2]) {
    const AMOSTRAS: usize = 16;
    for i in 0..=AMOSTRAS {
        let f = i as f32 / AMOSTRAS as f32;
        let fase = match i {
            0 => PointerPhase::Down,
            AMOSTRAS => PointerPhase::Up,
            _ => PointerPhase::Move,
        };
        let p = [de[0] + (ate[0] - de[0]) * f, de[1] + (ate[1] - de[1]) * f];
        t.on_canvas_pointer(cp(p, fase));
        quadro(t, g.as_deref_mut());
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Quando {
    AntesDeArmar,
    AGravar,
    Nunca,
}

/// Devolve `(pincel e tela do tool que gravou, pincel e tela do tool que tocou)`.
type Lado = (
    (
        ph2d_painter_brush::taper::Taper,
        ph2d_painter_brush::TracoComposto,
        bool,
    ),
    Vec<u8>,
);

fn lado(t: &mut PainterTool) -> Lado {
    let s = t.brush_settings();
    ((s.taper, s.traco, s.blend_sobre_vazio), t.run_full().0)
}

fn grava_e_toca(quando: Quando) -> (Lado, Lado) {
    let mut t = ferramenta();
    if quando == Quando::AntesDeArmar {
        linhas(&mut t, None);
    }
    let mut g = Gravador::arma(&mut t);
    if quando == Quando::AGravar {
        linhas(&mut t, Some(&mut g));
    }
    // Dois traços que se cruzam: o tecto e as orlas mordem no cruzamento; a cauda e o fall-off no fim.
    traco(&mut t, Some(&mut g), [10.0, 40.0], [150.0, 120.0]);
    traco(&mut t, Some(&mut g), [10.0, 120.0], [150.0, 40.0]);
    let doc = g.para(&mut t);
    let original = lado(&mut t);
    let mut toc = toca(&doc).expect("a reprodução não diverge");
    (original, lado(toc.tool_mut()))
}

#[test]
fn as_linhas_do_traco_viajam_na_macro() {
    for quando in [Quando::AntesDeArmar, Quando::AGravar] {
        let ((pa, ta), (pb, tb)) = grava_e_toca(quando);
        assert!(pa.2 && pa.1.tecto < 1.0 && pa.0.fall_off > 0.0, "{quando:?}: as linhas não chegaram ao pincel");
        assert_eq!(pa, pb, "{quando:?}: o tool que toca acaba com outro pincel");
        assert_eq!(texels_diferentes(&ta, &tb), 0, "{quando:?}: a tela tocada é outra");
    }
}

/// O controlo: sem as linhas a tela é outra — o gate mede-as.
#[test]
fn sem_as_linhas_a_tela_e_outra() {
    let ((_, com), _) = grava_e_toca(Quando::AGravar);
    let ((_, sem), _) = grava_e_toca(Quando::Nunca);
    assert!(texels_diferentes(&com, &sem) > 0, "as linhas não mudam a tela");
}
