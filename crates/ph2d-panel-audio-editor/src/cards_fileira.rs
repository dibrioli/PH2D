//! A fileira de parâmetro num cartão: rótulo ao lado, a barra do cartão e a leitura do motor. O id continua o do
//! `Slider`: o store e o despacho não mudam, e a leitura continua uma string do motor (a curva é de cada efeito).

use super::*;
use crate::clipped_hits::ClippedHits;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::paint_text_elided;
use ph2d_editor_core::panel::cards::linhas_extra::CardBar;
use ph2d_editor_core::widget::{DEFAULT_LABEL_W, default_chip_w};
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// O envelope do corpo num cartão: quem corta é o recorte da lista.
pub(crate) const SEM_RECORTE: Rect = Rect {
    x: -ALCANCE,
    y: -ALCANCE,
    w: ALCANCE * 2.0,
    h: ALCANCE * 2.0,
};
/// Um quarto do maior `f32`: `x + w` não pode dar 0 (com `MIN + MAX` nenhum alvo cabia e nada se clicava).
const ALCANCE: f32 = f32::MAX / 4.0; // LITERAL-PX-OK: fração do domínio do f32, não uma medida

/// Uma fileira de parâmetro. `vivo = false` pinta e não regista, como no painel de antes.
#[allow(clippy::too_many_arguments)]
pub(crate) fn fileira_no_cartao(
    ink: CardInk,
    (x, y, w): (f32, f32, f32),
    label: &str,
    norm: f32,
    valor: Option<&str>,
    id: NodeId,
    vivo: bool,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut ClippedHits,
) -> f32 {
    let centro = |px: f32| y + (m::FIELD_H - px) * 0.5;
    paint_text_elided(
        text_system,
        scene,
        label,
        x,
        centro(m::LABEL_PX),
        m::LABEL_PX,
        DEFAULT_LABEL_W,
        ink.color(ColorToken::CardLabel),
    );
    let lido = valor.map_or_else(|| format!("{:.2}", f64::from(norm)), str::to_owned);
    let col_valor = default_chip_w();
    let tw = text_system.prefix_width(&lido, m::VALUE_PX).ceil();
    paint_text_elided(
        text_system,
        scene,
        &lido,
        (x + w - tw).max(x + w - col_valor),
        centro(m::VALUE_PX),
        m::VALUE_PX,
        col_valor,
        ink.color(ColorToken::KanbanCardFg),
    );
    let bx = x + DEFAULT_LABEL_W + m::FIELD_GAP;
    let barra = Rect::new(
        bx,
        y,
        (x + w - col_valor - m::FIELD_GAP - bx).max(0.0),
        m::FIELD_H,
    );
    hit_index.com_recorte(|store, hits| {
        let mut mudo = HitIndex::default();
        let alvo = if vivo { hits } else { &mut mudo };
        CardCtx::flutuante(scene, text_system, store, alvo, ink, x, w).bar(
            &CardBar {
                id,
                value: norm,
                vertical: false,
                ticks: &[],
            },
            barra,
        );
    });
    y + m::FIELD_H + m::FIELD_ROW_GAP
}
