//! ⭐⭐ **O PAINEL A MOSTRAR A PILHA DA PEÇA 3D** (`docs/3D/30` §4 — a W3).
//!
//! Com o Painter a pintar uma peça esculpida, a lista é a pilha da PEÇA (a ponte publica-a pelo
//! mesmo [`crate::set_current_layers`]) e cada gesto vai para ela. O que a peça ainda não oferece
//! aparece DESLIGADO — nunca registado, logo nunca um clique mudo — e as frases do fundo dizem porquê,
//! junto com a última recusa que a escultura devolveu.

use std::cell::{Cell, RefCell};

use ph2d_a11y::NodeId;
use ph2d_editor_core::paint::{paint_text_block, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, TypeToken};
use ph2d_tool_painter::ids as tids;
use ph2d_tool_painter::{AdjustmentKind, LayerKind, LayerStack};

thread_local! {
    static ON_PIECE: Cell<bool> = const { Cell::new(false) };
    static REFUSAL: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Publica (ponte, por quadro) se a lista é a pilha da peça 3D.
pub fn set_current_layers_on_piece(on: bool) {
    ON_PIECE.with(|c| c.set(on));
}

/// Publica (ponte, por quadro) a frase da última recusa da pilha da peça (`None` = nenhuma).
pub fn set_current_piece_refusal(why: Option<String>) {
    REFUSAL.with(|c| *c.borrow_mut() = why);
}

/// A lista é a pilha da peça?
#[must_use]
pub(crate) fn on_piece() -> bool {
    ON_PIECE.with(Cell::get)
}

/// ⭐⭐ **Este botão da barra serve na peça, com esta pilha?** — fora da peça, todos servem.
#[must_use]
pub(crate) fn offered(id: NodeId, stack: Option<&LayerStack>) -> bool {
    if !on_piece() {
        return true;
    }
    let Some(s) = stack else {
        return false;
    };
    let activa = s.active().and_then(|a| s.get(a));
    let pintura = activa.filter(|l| matches!(l.kind, LayerKind::Raster(_)));
    if id == tids::PAINTER_LAYERS_ADD || id == tids::PAINTER_LAYERS_ADD_ADJUSTMENT {
        true
    } else if id == tids::PAINTER_LAYERS_DUPLICATE || id == tids::PAINTER_LAYERS_CLIP {
        pintura.is_some()
    } else if id == tids::PAINTER_LAYERS_MASK {
        pintura.is_some_and(|l| l.mask.is_none())
    } else if id == tids::PAINTER_LAYERS_DELETE {
        activa.is_some_and(|l| s.root().last() != Some(&l.id))
    } else {
        false
    }
}

/// ⭐ **Este ajuste serve na peça?** — os que leem a vizinhança da imagem ainda não (W6).
#[must_use]
pub(crate) fn adjustment_offered(kind: AdjustmentKind) -> bool {
    !on_piece() || !kind.reads_the_image_layout()
}

/// ⭐⭐ **As frases da peça**, por baixo da lista: a última recusa, a activa que não é de pintura, e
/// o que está desligado aqui. Devolve o `y` depois delas.
pub(crate) fn paint_piece_notes(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    stack: Option<&LayerStack>,
    x: f32,
    mut y: f32,
    w: f32,
) -> f32 {
    if !on_piece() {
        return y;
    }
    let font = TypeToken::Base.px();
    let gap = ph2d_tokens::control_gap_px();
    let activa_nao_pinta = stack.is_some_and(|s| {
        !s.active()
            .and_then(|a| s.get(a))
            .is_some_and(|l| matches!(l.kind, LayerKind::Raster(_)))
    });
    let recusa = REFUSAL.with(|c| c.borrow().clone());
    let mut linhas: Vec<(String, ColorToken)> = Vec::new();
    if let Some(r) = recusa {
        linhas.push((r, ColorToken::Danger));
    }
    if stack.is_none() {
        linhas.push((
            tr("panel.painter_layers.piece.no_plane").to_owned(),
            ColorToken::Text2,
        ));
    } else {
        if activa_nao_pinta {
            linhas.push((
                tr("panel.painter_layers.piece.active_not_paint").to_owned(),
                ColorToken::Text2,
            ));
        }
        linhas.push((
            tr("panel.painter_layers.piece.off_here").to_owned(),
            ColorToken::Text2,
        ));
    }
    for (texto, cor) in linhas {
        y += paint_text_block(
            ctx.text_system,
            ctx.scene,
            &texto,
            x,
            y,
            font,
            w,
            resolve(cor, theme),
        ) + gap;
    }
    y
}
