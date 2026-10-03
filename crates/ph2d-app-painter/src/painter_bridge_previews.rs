//! **Os previews do painel do pincel** — as imagens que a ferramenta publica para as pré-visualizações
//! (o Grain, o Shape, o Paper e o Flow do Ragged Edge), cada uma só quando a VERSÃO dela muda (o `Vec`
//! pesado é clonado na mudança, nunca por quadro). Cortado do `painter_bridge` (tecto de LOC), onde os
//! três primeiros eram três cópias do mesmo bloco.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Corre `publica` só quando `versao` difere da última vista em `ultima`.
fn se_mudou(ultima: &AtomicU64, versao: u64, publica: impl FnOnce()) {
    if ultima.swap(versao, Ordering::Relaxed) != versao {
        publica();
    }
}

/// Uma luminância emprestada da ferramenta → a cópia partilhável que o painel guarda.
fn copia(img: Option<(&[u8], u32, u32)>) -> Option<(Arc<Vec<u8>>, u32, u32)> {
    img.map(|(lum, w, h)| (Arc::new(lum.to_vec()), w, h))
}

/// Publica os quatro previews do pincel. Chamado uma vez por quadro com o Painter ativo.
pub fn publica_previews(painter: &ph2d_tool_painter::PainterTool) {
    static TEXTURA: AtomicU64 = AtomicU64::new(u64::MAX);
    static SHAPE: AtomicU64 = AtomicU64::new(u64::MAX);
    static PAPEL: AtomicU64 = AtomicU64::new(u64::MAX);
    static FLUXO: AtomicU64 = AtomicU64::new(u64::MAX);
    se_mudou(&TEXTURA, painter.brush_texture_image_version(), || {
        ph2d_panel_painter_layers::set_current_brush_texture_image(copia(
            painter.brush_texture_image(),
        ));
    });
    se_mudou(&SHAPE, painter.brush_shape_image_version(), || {
        ph2d_panel_painter_layers::set_current_brush_shape_image(copia(
            painter.brush_shape_image(),
        ));
    });
    se_mudou(&PAPEL, painter.brush_paper_image_version(), || {
        ph2d_panel_painter_layers::set_current_brush_paper_image(copia(
            painter.brush_paper_image(),
        ));
    });
    se_mudou(&FLUXO, painter.brush_flow_image_version(), || {
        ph2d_panel_painter_layers::set_current_brush_flow_image(copia(painter.brush_flow_image()));
    });
}
