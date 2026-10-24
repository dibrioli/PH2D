//! Painter bridge — tool-concrete DOWNCAST query helpers, split out of
//! `painter_bridge.rs` to keep it under the HR-18 file-LOC cap.
//!
//! The helper needs the `PainterTool` concrete type, so it lives in an
//! allowlisted bridge file to keep the central dispatch free of tool-concrete
//! downcasts per the `architecture_no_downcast_to_concrete_tool_in_shell` gate.

use ph2d_editor_core::ToolRegistry;

/// **O Painter do registo, activo ou não** — a porta da shell para o `PainterTool` concreto por id.
pub fn o_painter(tools: &mut ToolRegistry) -> Option<&mut ph2d_tool_painter::PainterTool> {
    tools
        .tool_by_id_mut(&ph2d_editor_core::ToolId::new("painter"))?
        .as_any_mut()
        .downcast_mut()
}

/// **A imagem que um verbo SÓ-LEITURA deve levar da sprite `entity`** (RGBA8 straight) enquanto o
/// Painter a pinta sem a ter assado: a tinta vive no tool e a textura da sprite ainda é a de antes
/// (Enio 24/10: o «Export Image…» de um gato pintado saiu BRANCO; `BUGS_painter_fechados_41_a_46.md` #53). `None` = a sprite é a verdade
/// (outro documento, ou nada por assar — o leitor dela guarda os 16 bits).
pub fn imagem_por_assar(tools: &mut ToolRegistry, entity: u64) -> Option<(Vec<u8>, u32, u32)> {
    imagem_por_assar_do(o_painter(tools)?, entity)
}

/// [`imagem_por_assar`] sobre o tool já na mão.
#[must_use]
pub fn imagem_por_assar_do(
    p: &ph2d_tool_painter::PainterTool,
    entity: u64,
) -> Option<(Vec<u8>, u32, u32)> {
    (p.bound_document() == Some(entity) && p.has_unbaked_edits())
        .then(|| p.imagem_aplicada())
        .flatten()
}

/// **A tinta por assar do Painter e a sprite dela, lidas UMA vez por gesto** — o que um verbo
/// SÓ-LEITURA sobre sprites (exportar a folha, enviar ao quadro, fundir, o pivô ao conteúdo) leva no
/// lugar da textura da sprite pintada (`BUGS_painter_fechados_41_a_46.md` #53). Vazio = toda sprite é
/// a verdade.
#[derive(Default)]
pub struct PorAssar(Option<(u64, ph2d_render::SpriteImage)>);

impl PorAssar {
    /// A imagem da sprite `entity`: a tinta por assar, se é dela; senão `le_a_sprite()`.
    pub fn ou(
        &self,
        entity: u64,
        le_a_sprite: impl FnOnce() -> Option<ph2d_render::SpriteImage>,
    ) -> Option<ph2d_render::SpriteImage> {
        match &self.0 {
            Some((e, img)) if *e == entity => Some(img.clone()),
            _ => le_a_sprite(),
        }
    }

    /// O centro da caixa OPACA da sprite `entity` (alfa > 0), normalizado à imagem: `[u, v]` em `-0.5..0.5`,
    /// `v` para baixo (as linhas da imagem). A tinta por assar é conteúdo (#53) — o pivô ao conteúdo. `None`:
    /// sem pixels ou toda transparente.
    #[expect(clippy::cast_precision_loss, reason = "coordenadas de imagem < 2^24")]
    pub fn centro_do_conteudo(
        &self,
        entity: u64,
        le_a_sprite: impl FnOnce() -> Option<ph2d_render::SpriteImage>,
    ) -> Option<[f32; 2]> {
        let img = self.ou(entity, le_a_sprite)?;
        let w = img.width as usize;
        let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
        for (i, px) in img.pixels.as_chunks::<4>().0.iter().enumerate() {
            if px[3] > 0 {
                let (x, y) = (i % w, i / w);
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
            }
        }
        (x0 != usize::MAX).then(|| {
            let c = |a: usize, b: usize, n: u32| (a + b + 1) as f32 * 0.5 / n as f32 - 0.5;
            [c(x0, x1, img.width), c(y0, y1, img.height)]
        })
    }
}

/// [`PorAssar`] do Painter do registo (activo ou não).
pub fn por_assar(tools: &mut ToolRegistry) -> PorAssar {
    o_painter(tools).map_or_else(PorAssar::default, |p| por_assar_do(p))
}

/// [`PorAssar`] sobre o tool já na mão.
#[must_use]
pub fn por_assar_do(p: &ph2d_tool_painter::PainterTool) -> PorAssar {
    PorAssar(p.bound_document().and_then(|e| {
        let (px, w, h) = imagem_por_assar_do(p, e)?;
        let img = ph2d_render::SpriteImage::from_bytes(w, h, px, ph2d_render::AlphaMode::Straight);
        Some((e, img))
    }))
}

/// **A sprite cuja tinta por assar um verbo que a ESCREVE tem de assar ANTES** (#53): o Painter
/// pinta-a, há edições por assar e `alvo` a escolhe. Ler a tinta não basta a quem escreve: a textura
/// nova seria reescrita pelo assar seguinte (ou a tinta perdida no re-bind).
#[must_use]
pub fn a_assar_antes(
    p: &ph2d_tool_painter::PainterTool,
    alvo: impl Fn(u64) -> bool,
) -> Option<u64> {
    p.bound_document()
        .filter(|&e| p.has_unbaked_edits() && alvo(e))
}

/// Apply a Painter layers drag-reparent (W3 T3.8) emitted by the dispatch on Up
/// of an active layer-row drag. The active `PainterTool` reverses
/// `NodeId`→`LayerId` and applies `move_into_group` / reorder. Mirror of
/// [`painter_has_unflushed_strokes`]. No-op if the active tool is not the Painter.
/// `true` while the active Painter's **Deform Transform** gizmo is live (any of the 4 sub-modes —
/// Uniform / Free / Distort / Warp, with a lifted floating patch). The render loop then SUPPRESSES the
/// sprite gizmo for the frame: on a whole-image transform both gizmos occupy the same screen corners,
/// and a near-corner Down grabbed the SPRITE's scale handle instead of the deform's (Enio 2026-07-04,
/// "Transform uniform/free interferem no gizmo da sprite — inative o gizmo da sprite").
pub fn deform_transform_gizmo_active(tools: &mut ToolRegistry) -> bool {
    tools
        .active_mut()
        .and_then(|t| {
            t.as_any_mut()
                .downcast_mut::<ph2d_tool_painter::PainterTool>()
        })
        .is_some_and(|p| p.deform_gizmo_grab_margin_px() > 0.0)
}

pub fn apply_layer_reparent(
    tools: &mut ToolRegistry,
    dragged: ph2d_editor_core::NodeId,
    drop: ph2d_editor_core::interaction::PanelRowDrop,
) {
    if let Some(painter) = tools.active_mut().and_then(|t| {
        t.as_any_mut()
            .downcast_mut::<ph2d_tool_painter::PainterTool>()
    }) {
        painter.handle_layer_reparent(dragged, drop);
    }
}
