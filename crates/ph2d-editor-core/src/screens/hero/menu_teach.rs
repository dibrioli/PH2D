//! ⭐ **O CHIP QUE ENSINA O ATALHO** (versão C, dono 12/11): na 3.ª vez que o artista escolhe um
//! comando PELO MENU, a faixa diz «Ctrl+E is Export Sheet…» ao lado da busca. Some ao usar a tecla
//! ou no «×» — e aí nunca volta para esse comando. Um chip de cada vez.
//!
//! «Pelo menu» = o clique (ou o Enter) numa linha da cascata da barra, a porta única
//! [`super::menu_bar::close_on_row_click`]. A tecla do atalho é o texto à direita do rótulo
//! («Save · Ctrl+S»), lido como acorde ([`chord_of_text`]); um rótulo cujo lado direito não é um
//! acorde (a pasta de um projeto recente) não ensina nada.

use super::HeroScreen;
use super::menu_faixa::Faixa;
use super::menu_memory::TEACH_AT;
use crate::ids;
use crate::interaction::{HitIndex, TeachChip, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, paint_text_elided, resolve};
use crate::zones::Rect;
use ph2d_host::Modifiers;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, INLINE_ICON_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Um acorde lido: os modificadores e a tecla (maiúscula, ou o nome dela: `F5`, `Space`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chord {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub key: String,
}

/// «Ctrl+Shift+S» ⇒ o acorde; `None` se o texto não é um acorde (uma pasta, uma frase).
#[must_use]
pub fn chord_of_text(text: &str) -> Option<Chord> {
    let mut c = Chord {
        ctrl: false,
        shift: false,
        alt: false,
        key: String::new(),
    };
    let parts: Vec<&str> = text.trim().split('+').collect();
    let (key, mods) = parts.split_last()?;
    for m in mods {
        match *m {
            "Ctrl" | "Cmd" => c.ctrl = true,
            "Shift" => c.shift = true,
            "Alt" => c.alt = true,
            _ => return None,
        }
    }
    let key = key.trim();
    let tecla = key.chars().count() == 1 && key.chars().all(|ch| ch.is_ascii_alphanumeric());
    let nomeada = matches!(key, "Space" | "Tab" | "Enter" | "Delete")
        || (key.len() <= 3
            && key.starts_with('F')
            && key.len() > 1
            && key[1..].chars().all(|ch| ch.is_ascii_digit()));
    if !(tecla || nomeada) {
        return None;
    }
    c.key = key.to_ascii_uppercase();
    Some(c)
}

/// O acorde de uma tecla que DESCEU (o normalizador do Input Map: toda tecla tem nome).
#[must_use]
pub fn chord_of_key(key: ph2d_input::Key, m: Modifiers) -> Chord {
    Chord {
        ctrl: m.ctrl || m.meta,
        shift: m.shift,
        alt: m.alt,
        key: key.label().to_ascii_uppercase(),
    }
}

/// O rótulo da linha `id` num menu aberto da cascata (a tabela ou as linhas da aba).
fn open_row_label(store: &WidgetStore, id: ph2d_a11y::NodeId) -> Option<String> {
    store.menu_chain().iter().find_map(|req| {
        super::menu_rows::menu_rows(req.kind)
            .iter()
            .find(|r| r.0 == id)
            .map(|r| r.1.tr().to_owned())
            .or_else(|| {
                store
                    .doc_menu_rows(req.kind)
                    .find(|(rid, _)| *rid == id)
                    .map(|(_, l)| l.to_owned())
            })
    })
}

/// ⭐ **`id` foi escolhido numa linha da cascata** — conta, e na [`TEACH_AT`]ª vez mostra o chip
/// (se o rótulo traz um atalho e o artista ainda não o aprendeu). Chamado ANTES de o menu fechar.
pub fn picked_from_menu(hero: &mut HeroScreen, id: ph2d_a11y::NodeId) {
    let Some(label) = open_row_label(&hero.store, id) else {
        return;
    };
    let n = hero.menu_memory.picked_from_menu(id);
    if n < TEACH_AT || hero.menu_memory.knows(id) {
        return;
    }
    let Some((name, key)) = label.rsplit_once(" \u{00b7} ") else {
        return;
    };
    if chord_of_text(key).is_none() {
        return;
    }
    hero.store.set_teach_chip(Some(TeachChip {
        id,
        key: key.trim().to_owned(),
        name: name.trim().to_owned(),
    }));
}

/// O artista fechou o chip: já sabe, e ele não volta.
pub fn dismiss(hero: &mut HeroScreen) {
    if let Some(chip) = hero.store.teach_chip() {
        let id = chip.id;
        hero.menu_memory.learn(id);
    }
    hero.store.set_teach_chip(None);
}

/// Uma tecla desceu: se é o atalho que o chip ensina, o artista aprendeu-o.
pub fn heard_key(hero: &mut HeroScreen, key: ph2d_input::Key, m: Modifiers) {
    let Some(chip) = hero.store.teach_chip() else {
        return;
    };
    if chord_of_text(&chip.key) == Some(chord_of_key(key, m)) {
        dismiss(hero);
    }
}

/// Onde o chip cabe: no vão entre a busca e o jogo, encostado à busca. `None` = não cabe.
fn chip_rect(f: &Faixa, text_system: &mut TextSystem, texto: &str) -> Option<(Rect, Rect)> {
    if f.lupa {
        return None;
    }
    let gap = super::document_tabs::gap_px();
    let de = f.find.x + f.find.w + gap;
    let ate = f.jogo.first().map_or(f.find.x + f.find.w, |(_, _, r)| r.x) - gap;
    let pad = Spacing::Sm.px();
    let x_w = INLINE_ICON_PX + pad;
    // Inteiro ou nada: um chip cortado não ensina a tecla.
    let w = pad + text_system.prefix_width(texto, TypeToken::Xs.px()) + pad + x_w;
    if w > ate - de {
        return None;
    }
    let h = f.find.h - Spacing::Xs.px() * 2.0;
    let r = Rect::new(de, f.find.y + (f.find.h - h) * 0.5, w, h);
    let x = Rect::new(r.x + r.w - x_w, r.y, x_w, r.h);
    Some((r, x))
}

/// Pinta o chip (se há um e cabe) e regista o «×».
pub fn paint(
    f: &Faixa,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let Some(chip) = store.teach_chip() else {
        return;
    };
    let texto = ph2d_i18n::tr_with(
        "chrome.menu_bar.teach",
        &[("key", &chip.key), ("name", &chip.name)],
    );
    let Some((r, x)) = chip_rect(f, text_system, &texto) else {
        return;
    };
    let raio = crate::paint::frame_radius(theme, Radius::Full.px().min(r.h * 0.5));
    fill_rounded_rect(scene, r, raio, resolve(ColorToken::AccentSoft, theme));
    let font = TypeToken::Xs.px();
    let pad = Spacing::Sm.px();
    let room = (x.x - r.x - pad).max(0.0);
    let tinta = resolve(ColorToken::Accent, theme);
    paint_text_elided(
        text_system,
        scene,
        &texto,
        r.x + pad,
        r.y + (r.h - font) * 0.5,
        font,
        room,
        tinta,
    );
    let hover = matches!(
        store.button_state(ids::MENUBAR_TEACH_CLOSE),
        Some(crate::widget::ButtonState::Hovered | crate::widget::ButtonState::Pressed)
    );
    let lado = INLINE_ICON_PX;
    let glifo = Rect::new(
        x.x + (x.w - lado - pad) * 0.5,
        x.y + (x.h - lado) * 0.5,
        lado,
        lado,
    );
    let cor = if hover {
        tinta
    } else {
        resolve(ColorToken::Text2, theme)
    };
    paint_icon(
        scene,
        crate::icons::IconId::Close,
        glifo,
        cor,
        StrokeToken::Default.px(),
    );
    hit_index.register(ids::MENUBAR_TEACH, r);
    hit_index.register(ids::MENUBAR_TEACH_CLOSE, x);
}

#[cfg(test)]
#[path = "menu_teach_tests.rs"]
mod tests;
