//! **O Reset de uma secção devolve o pincel com que ESTE modo, NESTE meio, nasce — e nunca troca o
//! meio** (decisão do dono, 2026-10-04, depois do censo dos controlos: doc 45 §2.1).
//!
//! O censo mediu, na fábrica de cada meio, Resets que MUDAVAM a pintura: o da Aquarela, do Impasto e
//! do Wet Paint desligavam o meio (o pincel voltava a Digital); o do Stroke no Wet Paint punha o
//! Spacing em `0,1` (o pincel da água nasce com `0,025`); o da Shape no Impasto punha o Falloff do
//! Digital (o Impasto nasce com Sphere). A régua dos Resets era o `BrushSpec::default()` — o pincel
//! de NENHUM modo em particular.

use super::super::media::PaintMedia;
use super::*;
use crate::ids::*;
use ph2d_editor_core::tool::PanelEvent;

/// Todo Reset de secção do painel do pincel.
const RESETS: [ph2d_a11y::NodeId; 12] = [
    PAINTER_IMPASTO_RESET,
    PAINTER_BRUSH_RANDOMIZE_RESET,
    PAINTER_BRUSH_TEXTURE_RESET,
    PAINTER_BRUSH_COLOR_RAMP_RESET,
    PAINTER_BRUSH_STROKE_RESET,
    PAINTER_BRUSH_TILING_RESET,
    PAINTER_WATERCOLOR_RESET,
    PAINTER_WATERCOLOR_PAPER_RESET,
    PAINTER_WETPAINT_RESET,
    PAINTER_SHAPE_RESET,
    PAINTER_SHAPE_RAMP_RESET,
    PAINTER_BRUSH_SYMMETRY_RESET,
];

const MEIOS: [PaintMedia; 4] = [
    PaintMedia::Digital,
    PaintMedia::Watercolor,
    PaintMedia::Impasto,
    PaintMedia::WetPaint,
];

/// Um pincel acabado de nascer no meio `media`, com a ferramenta `modo` da barra (ou o pincel).
fn nascido(media: PaintMedia, modo: Option<&str>) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_paint_media(media);
    if let Some(m) = modo {
        t.set_paint_tool_mode(m);
    }
    t
}

fn retrato(t: &PainterTool) -> String {
    format!("{:?}", t.brush_settings())
}

/// ⭐ **Na fábrica, nenhum Reset muda o pincel** — em cada meio, com o pincel e com as ferramentas
/// da barra que têm pincel próprio (Smear/Blur/Clone nascem com Spacing `0,05`).
///
/// **Mutações que sangram:** voltar a escrever `b.watercolor = d.watercolor` no Reset da Aquarela;
/// o Reset do Stroke ler o `BrushSpec::default()`; o do Wet Paint desarmar.
#[test]
fn na_fabrica_nenhum_reset_muda_o_pincel() {
    let mut falhas = Vec::new();
    for media in MEIOS {
        for modo in [None, Some("smear"), Some("blur"), Some("clone")] {
            for id in RESETS {
                let mut t = nascido(media, modo);
                let (antes, meio) = (retrato(&t), t.paint_media());
                t.handle_panel_event(PanelEvent::Click(id));
                if retrato(&t) != antes || t.paint_media() != meio {
                    falhas.push(format!("{media:?} · {modo:?} · Reset {id:?}"));
                }
            }
        }
    }
    assert!(
        falhas.is_empty(),
        "{} Reset(s) mudam o pincel ACABADO DE NASCER:\n{}",
        falhas.len(),
        falhas.join("\n")
    );
}

/// ⭐ **O Reset de um meio guarda o meio e repõe os valores DELE** — mexe-se num valor da secção, o
/// Reset devolve-o ao de fábrica, e o pincel continua no mesmo meio.
#[test]
fn o_reset_de_um_meio_guarda_o_meio_e_repoe_os_valores_dele() {
    for (media, mexe, reset) in [
        (
            PaintMedia::Watercolor,
            PanelEvent::SetValue(PAINTER_WATERCOLOR_EDGE, 5.0),
            PAINTER_WATERCOLOR_RESET,
        ),
        (
            PaintMedia::Impasto,
            PanelEvent::SetValue(PAINTER_IMPASTO_DEPTH, 0.9),
            PAINTER_IMPASTO_RESET,
        ),
        (
            PaintMedia::WetPaint,
            PanelEvent::SetValue(PAINTER_WETPAINT_WATER, 0.9),
            PAINTER_WETPAINT_RESET,
        ),
    ] {
        let fabrica = retrato(&nascido(media, None));
        let mut t = nascido(media, None);
        t.handle_panel_event(mexe);
        assert_ne!(retrato(&t), fabrica, "{media:?}: o valor nem mexeu");
        t.handle_panel_event(PanelEvent::Click(reset));
        assert_eq!(t.paint_media(), media, "{media:?}: o Reset trocou o meio");
        assert_eq!(
            retrato(&t),
            fabrica,
            "{media:?}: o Reset não repôs a fábrica"
        );
    }
}

/// **A fábrica guarda o meio de AGORA** — a porta que todo Reset lê ([`PainterTool::spec_de_fabrica`])
/// nunca devolve um pincel noutro meio, para o Reset que um dia copiar a fábrica INTEIRA.
///
/// **Mutação que sangra:** tirar `s.watercolor = b.watercolor` (ou a do impasto) da fábrica.
#[test]
fn a_fabrica_guarda_o_meio_escolhido() {
    for media in MEIOS {
        let t = nascido(media, None);
        let f = t.spec_de_fabrica();
        assert_eq!(
            (f.watercolor, f.impasto),
            (t.paint.brush.watercolor, t.paint.brush.impasto),
            "{media:?}: a fábrica devolveu o pincel noutro meio"
        );
    }
}
