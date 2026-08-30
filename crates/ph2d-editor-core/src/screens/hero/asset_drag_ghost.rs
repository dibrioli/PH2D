//! ⭐⭐ **A VOZ DO ARRASTO** — o fantasma que segue o cursor (plano `docs/Components/07`, B4).
//!
//! # Ele é o primeiro do app, e a ausência tinha nome
//!
//! Medido em 2026-08-30: **nada neste editor segue o cursor como coisa carregada.** O indicador do
//! reparentar é ancorado na LINHA; o menu radial fixa o centro de propósito (*«ele não se
//! move»*); e o aviso de largar ficheiro do sistema **recebe o cursor e deita-o fora** — a
//! assinatura dele é literalmente `_cursor_px`, e o cartão desenha-se no meio do ecrã.
//!
//! ⇒ um arrasto que sai de um painel precisa de dizer **o que vai na mão**, senão o gesto é uma
//! aposta: o artista solta e descobre.
//!
//! # ⚠️ Onde ele é pintado, e por quê
//!
//! No fim do `paint_hero_screen`, **depois** do aviso de largar ficheiro — a cena do chrome é a
//! última a ser composta, então o que se desenha aqui fica por cima de tudo, inclusive dos painéis
//! e da tela. Um fantasma por baixo de um painel seria pior que nenhum.
//!
//! ⛔ **Ele não regista rectângulo nenhum** no `HitIndex`: uma coisa que segue o cursor não pode
//! ser clicável — ela taparia sempre o alvo que a mão está a procurar.

use crate::interaction::drag_payload::InFlightDrag;
use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Deslocamento do cartão em relação ao cursor.
///
/// ⚠️ **Positivo nos dois eixos, de propósito**: o fantasma fica **abaixo e à direita** da ponta do
/// cursor, que é onde a seta não tapa. Centrá-lo no cursor esconderia precisamente o pixel que o
/// artista está a mirar.
const GHOST_OFFSET_PX: f32 = 14.0; // LITERAL-PX-OK: geometria do fantasma, não medida de design

/// Pinta o que vai na mão. No-op se não há arrasto, ou se ele **ainda não armou** (o gesto pode
/// acabar por ser um clique, e um fantasma que pisca a cada toque é ruído).
pub(super) fn paint_asset_drag_ghost(
    drag: Option<InFlightDrag>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let Some(drag) = drag.filter(|d| d.armed) else {
        return;
    };
    let label = drag.payload.kind_label();
    let font = TypeToken::Sm.px();
    let pad = Spacing::Sm.px();
    let w = text_system.layout(label, font, f32::INFINITY).width() + pad * 2.0;
    let h = font + pad * 2.0;
    let rect = Rect::new(
        drag.cursor.0 + GHOST_OFFSET_PX,
        drag.cursor.1 + GHOST_OFFSET_PX,
        w,
        h,
    );
    fill_rounded_rect(
        scene,
        rect,
        Radius::Sm.px(),
        resolve(ColorToken::BgElev, theme),
    );
    stroke_rounded_rect(
        scene,
        rect,
        Radius::Sm.px(),
        StrokeToken::Default.px(),
        resolve(ColorToken::Accent, theme),
    );
    crate::paint::paint_text(
        text_system,
        scene,
        label,
        rect.x + pad,
        rect.y + pad,
        font,
        w,
        resolve(ColorToken::Text1, theme),
    );
}
