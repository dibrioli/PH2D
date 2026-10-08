//! ⭐ **Os MODIFICADORES NA TELA** (F5, D10) — Shift · Alt · Ctrl num canto da área, do lado da mão
//! livre: **segurar = segurar, tocar = travar**, e travado fica SEMPRE à vista (o lembrete do
//! *Command Controller* do Affinity e do *Touch Shortcut* da Adobe — estudo §3).
//!
//! ⚠️ É a porta VISÍVEL do gesto do dedo extra: aparece no primeiro toque da sessão (ou com algo
//! travado), e o rato também o trava com um clique.

use super::{button_px, gap, paint_button, paint_panel};
use crate::interaction::touch::CHIPS;
use crate::screens::hero::HeroScreen;
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, TypeToken};
use ph2d_vector::VectorScene;

/// As chaves dos rótulos, na ordem de [`CHIPS`].
const LABELS: [&str; 3] = [
    "chrome.touch.shift",
    "chrome.touch.alt",
    "chrome.touch.ctrl",
];

/// ⭐ **Onde cai cada botão** — no canto de BAIXO da área, do lado da coluna ① e logo a seguir a
/// ela (a coluna de um módulo desce até ao fundo num iPad mini). `left_handed` espelha.
#[must_use]
pub fn rects(area: Rect, has_column: bool, left_handed: bool) -> [Rect; 3] {
    let (s, g, m) = (button_px(), gap(), Spacing::Md.px());
    let w = 2.0 * s;
    let x0 = area.x
        + m
        + if has_column {
            s + Spacing::Sm.px()
        } else {
            0.0
        };
    let y = area.y + area.h - m - s;
    std::array::from_fn(|i| {
        #[allow(clippy::cast_precision_loss)]
        let r = Rect::new(x0 + i as f32 * (w + g), y, w, s);
        if left_handed {
            super::mirrored(area, r)
        } else {
            r
        }
    })
}

/// Pinta o widget, se o toque já apareceu nesta sessão ou há algo travado.
pub fn paint(hero: &mut HeroScreen, area: Rect, scene: &mut VectorScene, text: &mut TextSystem) {
    let touch = hero.store.touch().clone();
    if !touch.widget_visible() {
        return;
    }
    let theme = hero.theme;
    let has_column = hero.store.tool_bars().is_some_and(|b| !b.column.is_empty());
    let rs = rects(area, has_column, hero.view.ui_mirrored);
    paint_panel(scene, rs.iter().copied(), theme);
    let locked = touch.locked();
    let on = [locked.shift, locked.alt, locked.ctrl];
    for ((&id, r), (key, lock)) in CHIPS.iter().zip(rs).zip(LABELS.iter().zip(on)) {
        let fg = paint_button(
            scene,
            &mut hero.hit_index,
            &hero.store,
            theme,
            super::Btn {
                id,
                rect: r,
                on: lock || touch.chip_held(id),
                tint: ColorToken::Text2,
            },
        );
        crate::paint::paint_text_centered(
            text,
            scene,
            ph2d_i18n::tr(key),
            r,
            TypeToken::Sm.px(),
            fg,
        );
        hero.store
            .set_tooltip(id, ph2d_i18n::tr("chrome.touch.tip").to_owned());
    }
}

#[cfg(test)]
#[path = "touch_mods_tests.rs"]
mod tests;
