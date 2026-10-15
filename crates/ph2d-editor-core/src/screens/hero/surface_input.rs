//! ⭐ **A entrada de um documento de FAMÍLIA** ([`crate::doc_surface`]) — chamada no TOPO das portas
//! por onde o quadro recebe rato, roda, teclado e composição: com uma aba de família activa, ela vem
//! primeiro e a cena nunca vê o evento que ela tomou. `None` = não há aba de família activa (segue a
//! porta do quadro como antes).

use super::board_keys::{BoardKey, KeyOutcome as BoardOutcome};
use super::board_view::Input;
use super::HeroScreen;
use crate::doc_surface::{Compose, Key, PointerInput};
use ph2d_host::{Modifiers, PointerButton, PointerKind};
use ph2d_text::TextSystem;

fn key_of(k: BoardKey) -> Key {
    match k {
        BoardKey::Char(c) => Key::Char(c),
        BoardKey::Enter => Key::Enter,
        BoardKey::Escape => Key::Escape,
        BoardKey::Backspace => Key::Backspace,
        BoardKey::Delete => Key::Delete,
        BoardKey::Left => Key::Left,
        BoardKey::Right => Key::Right,
        BoardKey::Up => Key::Up,
        BoardKey::Down => Key::Down,
        BoardKey::Home => Key::Home,
        BoardKey::End => Key::End,
        BoardKey::Space => Key::Space,
        BoardKey::Tab => Key::Tab,
        BoardKey::PageUp => Key::PageUp,
        BoardKey::PageDown => Key::PageDown,
        BoardKey::Other => Key::Other,
    }
}

/// A roda: o Shift vem da cache que a shell actualiza a cada mudança de modificadores.
pub(super) fn wheel(hero: &mut HeroScreen, x: f32, y: f32, dy: f32) -> Option<bool> {
    let shift = hero.store.shift_held();
    let (f, id) = hero.documents.active_surface_mut()?;
    Some(f.wheel(id, x, y, dy, shift))
}

/// Os comandos da aba de família activa (vazio sem ela).
#[must_use]
pub fn commands(hero: &HeroScreen) -> Vec<crate::doc_surface::DocCommand> {
    let Some(tab) = hero.documents.surface_active() else {
        return Vec::new();
    };
    hero.documents
        .surfaces()
        .families
        .get(usize::from(tab.family))
        .map(|f| f.commands(tab.id))
        .unwrap_or_default()
}

/// Um clique (linha de menu ou escolha do `Ctrl+K`) num comando da aba activa. `true` = tomado.
pub(super) fn command(hero: &mut HeroScreen, event: crate::interaction::WidgetEvent) -> bool {
    let crate::interaction::WidgetEvent::Click(cmd) = event else {
        return false;
    };
    let taken = hero
        .documents
        .active_surface_mut()
        .is_some_and(|(f, id)| f.command(id, cmd));
    if taken && hero.store.context_menu().is_some() {
        hero.store.close_context_menu();
    }
    taken
}

pub(super) fn pointer(
    hero: &mut HeroScreen,
    input: Input<'_>,
    kind: PointerKind,
    button: PointerButton,
    x: f32,
    y: f32,
) -> Option<bool> {
    let (f, id) = hero.documents.active_surface_mut()?;
    let pi = PointerInput { text: input.text, mods: input.mods, now_ns: input.now_ns };
    let taken = f.pointer(id, pi, kind, button, x, y);
    // Carregar no documento tira o foco a um campo da interface: senão o texto iria para lá.
    if taken && matches!(kind, PointerKind::Down) && hero.store.focus_id().is_some() {
        hero.store.set_focus(None);
    }
    Some(taken)
}

pub(super) fn pointer_move(hero: &mut HeroScreen, input: Input<'_>, x: f32, y: f32) -> Option<bool> {
    let (f, id) = hero.documents.active_surface_mut()?;
    let pi = PointerInput { text: input.text, mods: input.mods, now_ns: input.now_ns };
    Some(f.pointer_move(id, pi, x, y))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn key(
    hero: &mut HeroScreen,
    ts: &mut TextSystem,
    k: BoardKey,
    mods: Modifiers,
    pressed: bool,
    text: Option<&str>,
    paste: Option<String>,
) -> Option<BoardOutcome> {
    let (f, id) = hero.documents.active_surface_mut()?;
    let out = f.key(id, ts, key_of(k), mods, pressed, text, paste);
    Some(BoardOutcome { consumed: out.consumed, copy: out.copy })
}

/// Composição (IME / tecla morta / carácter confirmado) — só sem campo da interface focado.
pub(super) fn compose(hero: &mut HeroScreen, c: Compose<'_>) -> Option<bool> {
    if hero.store.focus_id().is_some() {
        return None;
    }
    let (f, id) = hero.documents.active_surface_mut()?;
    Some(f.compose(id, c))
}

/// Desfazer/refazer do menu Edit com a aba de família activa.
pub fn undo(hero: &mut HeroScreen, redo: bool) -> Option<bool> {
    let (f, id) = hero.documents.active_surface_mut()?;
    Some(f.undo(id, redo))
}

/// O nó `Window` do ecrã; com uma aba de família activa, o `Document` dela é o filho.
#[must_use]
pub(super) fn window_node(hero: &HeroScreen, viewport: crate::zones::Rect) -> ph2d_a11y::Node {
    let doc = surface_a11y(hero).first().map(|(id, _)| *id);
    let b = ph2d_a11y::NodeBuilder::new(ph2d_a11y::Role::Window)
        .label(ph2d_i18n::tr("chrome.hero.ph2d_editor"))
        .bounds(
            f64::from(viewport.x),
            f64::from(viewport.y),
            f64::from(viewport.w),
            f64::from(viewport.h),
        );
    match doc {
        Some(d) => b.children(vec![d]).build(),
        None => b.build(),
    }
}

fn surface_a11y(hero: &HeroScreen) -> Vec<(ph2d_a11y::NodeId, ph2d_a11y::Node)> {
    let Some(tab) = hero.documents.surface_active() else {
        return Vec::new();
    };
    hero.documents
        .surfaces()
        .families
        .get(usize::from(tab.family))
        .map(|f| f.a11y(tab.id))
        .unwrap_or_default()
}

/// ⭐ **A árvore de acessibilidade do ecrã** (E12): o `Window` na raiz de `tree` e, com uma aba de
/// família activa, os nós dela (o `Document` filho do `Window`, um `TextRun` por linha visível). Um
/// nó que deixou de existir (a linha saiu da vista ao rolar, a aba fechou) sai da árvore.
pub fn sync_a11y(hero: &HeroScreen, tree: &mut ph2d_a11y::Tree, viewport: crate::zones::Rect) {
    let root = tree.root();
    let mut nodes = surface_a11y(hero);
    nodes.insert(0, (root, window_node(hero, viewport)));
    let keep: std::collections::BTreeSet<ph2d_a11y::NodeId> = nodes.iter().map(|(id, _)| *id).collect();
    let stale: Vec<ph2d_a11y::NodeId> = tree.ids().filter(|id| !keep.contains(id)).collect();
    for id in stale {
        tree.remove(id);
    }
    for (id, n) in nodes {
        if tree.get(id) != Some(&n) {
            tree.insert(id, n);
        }
    }
}
