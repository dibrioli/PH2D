//! O pino do cabeçalho do seletor (à esquerda do `×`): fixado = o glifo aceso (`Accent`) e um
//! Down fora deixa de o fechar (`WidgetStore::picker_pinned`).
//!
//! ⚠️ Não importa `crate::interaction`: o registo no `HitIndex` fica em quem chama (`paint.rs`, que já
//! tem o `use`) — a aresta `widget → interaction` só encolhe (`architecture_the_foundation_modules_form_a_dag`).

use crate::icons::IconId;
use crate::paint::{paint_icon, resolve};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// Pinta o pino em `slot`; devolve se o pintou (`id == 0` = sem pino) — quem chama regista o MESMO rect.
pub(super) fn paint_pin(
    scene: &mut VectorScene,
    theme: Theme,
    fixado: bool,
    id: NodeId,
    slot: Rect,
) -> bool {
    if id.0 == 0 {
        return false;
    }
    let cor = if fixado {
        ColorToken::Accent
    } else {
        ColorToken::Text3
    };
    paint_icon(
        scene,
        IconId::Pin,
        slot,
        resolve(cor, theme),
        StrokeToken::Default.px(),
    );
    true
}
