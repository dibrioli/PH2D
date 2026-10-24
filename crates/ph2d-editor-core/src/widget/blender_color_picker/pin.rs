//! O pino do cabeçalho do seletor (à esquerda do `×`): fixado = o glifo aceso (`Accent`) e um
//! Down fora deixa de o fechar ([`crate::interaction::WidgetStore::picker_pinned`]).

use crate::icons::IconId;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{paint_icon, resolve};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// Pinta e regista o pino em `slot` — o rect que se pinta é o que se clica. `id == 0` = sem pino.
pub(super) fn paint_pin(
    scene: &mut VectorScene,
    theme: Theme,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    id: NodeId,
    slot: Rect,
) {
    if id.0 == 0 {
        return;
    }
    let cor = if store.picker_pinned() {
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
    hit_index.register(id, slot);
}
