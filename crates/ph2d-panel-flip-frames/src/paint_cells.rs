//! As **células** — a tira propriamente dita.
//!
//! Cada célula é uma chave, e a sua LARGURA é a exposição (à TVPaint): o tempo é
//! visível como espaço. A tira **sempre cabe** (a escala é derivada do vão), então
//! nunca há barra de rolagem nem estado de pan escondido — zoom/pan é follow-up.
//!
//! A célula é um **botão canônico** (`paint_button`), não um retângulo improvisado:
//! ela é clicável, tem estados (hover/press) e semântica de a11y — e o rótulo que
//! ela carrega é o número que o animador lê, a EXPOSIÇÃO ("esse desenho fica 2
//! quadros"). A chave ATIVA vira o botão `Accent`; um breakdown (inbetween do
//! tween) fica em `Default` (discreto). Um desenho INSTANCIADO (a mesma arte em
//! várias chaves) ganha um ponto. O playhead é uma linha vertical no quadro
//! corrente — inclusive no meio de um hold, que é onde ele costuma estar.

use crate::ids;
use crate::state::FlipStripSnapshot;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Button, ButtonKind, ButtonState, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};

/// Largura máxima de um quadro em px (senão 3 chaves viram 3 painéis gigantes).
const MAX_PX_PER_FRAME: f32 = 26.0; // LITERAL-PX-OK: strip cell scale cap
/// Largura mínima visível de uma célula.
const MIN_CELL_W: f32 = 3.0; // LITERAL-PX-OK: strip minimum cell width
/// Lado do marcador de desenho instanciado.
const INSTANCE_DOT: f32 = 4.0; // LITERAL-PX-OK: instanced-drawing marker
/// Espessura da linha do playhead.
const PLAYHEAD_W: f32 = 2.0; // LITERAL-PX-OK: playhead line width
/// A célula só ganha rótulo se couber ~um glifo e meio (senão o número sai cortado).
const LABEL_FIT: f32 = 1.6; // LITERAL-PX-OK: fator de caber-o-glifo, não medida de design

pub(crate) fn paint(ctx: &mut PaintCtx, theme: Theme, area: Rect, snap: &FlipStripSnapshot) {
    if area.w <= 0.0 || area.h <= 0.0 {
        return;
    }
    // Trilho de fundo (a "pista" onde as células vivem).
    fill_rounded_rect(
        ctx.scene,
        area,
        Radius::Sm.px(),
        resolve(ColorToken::Bg2, theme),
    );

    if snap.cells.is_empty() {
        let font = TypeToken::Sm.px();
        let hint = if snap.has_layer {
            "No keys yet — draw on the canvas to create the first one."
        } else {
            "No Flip layer — add one in the Flip panel."
        };
        paint_text(
            ctx.text_system,
            ctx.scene,
            hint,
            area.x + Spacing::Sm.px(),
            area.y + (area.h - font) * 0.5,
            font,
            area.w,
            resolve(ColorToken::Text2, theme),
        );
        return;
    }

    // O vão: do primeiro quadro exposto ao fim da última exposição. A escala é
    // derivada dele, então a tira SEMPRE cabe na faixa.
    let first = snap.cells[0].key;
    let last = &snap.cells[snap.cells.len() - 1];
    let end = last.key.saturating_add(last.exposure.max(1) as i32);
    let total = (end - first).max(1) as f32;
    let inner = Rect::new(
        area.x + Spacing::Xs.px(),
        area.y + Spacing::Xs.px(),
        (area.w - Spacing::Xs.px() * 2.0).max(1.0),
        (area.h - Spacing::Xs.px() * 2.0).max(1.0),
    );
    let ppf = (inner.w / total).min(MAX_PX_PER_FRAME);
    let font = TypeToken::Sm.px();

    for (i, cell) in snap.cells.iter().enumerate() {
        let x = inner.x + (cell.key - first) as f32 * ppf;
        let w = (cell.exposure.max(1) as f32 * ppf - 1.0).max(MIN_CELL_W);
        let r = Rect::new(x, inner.y, w, inner.h);
        let id = ids::flip_cell_id(i);
        ctx.host.store_mut().register_if_absent(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        let st = ctx
            .host
            .store()
            .button_state(id)
            .unwrap_or(ButtonState::Normal);
        // O rótulo só entra quando cabe (numa tira longa a célula é uma lasca).
        let label = if w > font * LABEL_FIT {
            format!("{}", cell.exposure)
        } else {
            String::new()
        };
        let kind = if snap.current_key == Some(cell.key) {
            ButtonKind::Accent // a chave que está NA TELA
        } else {
            ButtonKind::Default
        };
        paint_button(
            &Button::new(id, label).state(st).kind(kind),
            r,
            ctx.scene,
            ctx.text_system,
            theme,
        );
        // Desenho instanciado (a MESMA arte em várias chaves): um ponto discreto.
        if cell.instanced && w > INSTANCE_DOT * 2.0 {
            let d = Rect::new(
                r.x + Spacing::Xs.px(),
                r.y + Spacing::Xs.px(),
                INSTANCE_DOT,
                INSTANCE_DOT,
            );
            fill_rounded_rect(
                ctx.scene,
                d,
                Radius::Sm.px(),
                resolve(ColorToken::Text2, theme),
            );
        }
        ctx.host.hit_index_mut().register(id, r);
    }

    // O playhead — uma linha no quadro corrente (que costuma estar NO MEIO de uma
    // exposição; é justamente aí que ele precisa ser visível).
    let px = inner.x + (snap.current_frame - first) as f32 * ppf;
    if px >= inner.x - 1.0 && px <= inner.x + inner.w + 1.0 {
        let line = Rect::new(px.max(inner.x), area.y, PLAYHEAD_W, area.h);
        fill_rounded_rect(
            ctx.scene,
            line,
            Radius::Sm.px(),
            resolve(ColorToken::BorderEmph, theme),
        );
    }
}
