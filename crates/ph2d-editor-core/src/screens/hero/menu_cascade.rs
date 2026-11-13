//! ⭐ **A CASCATA dos menus da barra** (line/BarraDeMenus, onda 2): o pai fica aberto, o filho abre
//! ao lado, ao PAIRAR; o teclado anda por eles e o `Alt` sozinho abre o «Menu».
//!
//! Uma porta para «esta linha abre submenu?» ([`opens_submenu`]) — o pintor (o «›»), o Down (não
//! fecha), o pairar e as setas perguntam aqui. Quem ABRE continua a ser o dono da linha em
//! `chrome/*`, por `Click(id)`: pairar re-toca o clique, não há segunda tabela `linha → menu`.

use super::HeroScreen;
use super::context_menu_overlay::pad_y;
use crate::ids;
use crate::interaction::dispatch::keymap as k;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent, WidgetStore};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_host::{KeyEvent, KeyKind, PointerEvent, PointerKind};
use ph2d_tokens::{ROW_H_PX, Spacing};

/// Os dois `Alt` (`ph2d-input-winit`).
const ALT_KEYS: [u32; 2] = [0xF714, 0xF715];

/// **Esta linha abre um submenu?** As cascatas da barra, Preferences, Theme e toda linha das
/// Preferências (cada uma abre a lista dos valores dela).
#[must_use]
pub fn opens_submenu(id: NodeId) -> bool {
    super::menu_bar::cascade_of(id).is_some()
        || id == ids::MENUBAR_EDIT_PREFERENCES
        || id == ids::MENUBAR_VIEW_THEME
        || super::menu_rows::menu_rows(ContextMenuKind::SettingsMenu)
            .iter()
            .any(|(rid, ..)| *rid == id)
}

/// As linhas que este menu mostra agora (as da tabela que valem, as do módulo e as da aba).
fn level_rows(store: &WidgetStore, kind: ContextMenuKind) -> Vec<NodeId> {
    super::menu_rows::menu_rows(kind)
        .iter()
        .map(|(id, ..)| *id)
        .filter(|id| !store.menu_row_is_hidden(kind, *id))
        .chain(store.menu_contrib(kind).iter().filter_map(|e| e.node_id()))
        .chain(store.doc_menu_rows(kind).map(|(id, _)| id))
        .collect()
}

/// Em que nível da cascata aberta esta linha está (0 = a raiz).
#[must_use]
pub fn level_of_row(store: &WidgetStore, id: NodeId) -> Option<usize> {
    store
        .menu_chain()
        .iter()
        .position(|req| level_rows(store, req.kind).contains(&id))
}

/// **O Down em `id` é do menu aberto (não o fecha)?** — a única porta do despacho para cá. A linha
/// que abre um submenu é do menu (o pai fica, o filho abre ao lado); num menu da barra, TODA linha
/// dele: quem fecha é o clique ao soltar ([`super::menu_bar::close_on_row_click`]), a mesma porta do
/// Enter — senão o rato fechava no Down e a escolha pelo menu chegava sem menu (o chip que ensina o
/// atalho nunca contava um clique de rato, smoke do dono 12/11).
#[must_use]
pub fn down_is_of_the_menu(store: &WidgetStore, kind: ContextMenuKind, id: NodeId) -> bool {
    (opens_submenu(id) && level_of_row(store, id).is_some())
        || (super::menu_bar::is_bar_menu(kind) && level_of_row(store, id).is_some())
}

/// A cascata aberta é da barra (nasceu do «Menu» ou de um dos cinco)?
fn bar_chain(store: &WidgetStore) -> Vec<ContextMenuRequest> {
    let chain = store.menu_chain();
    if chain
        .first()
        .is_some_and(|r| super::menu_bar::is_bar_menu(r.kind))
    {
        chain
    } else {
        Vec::new()
    }
}

/// ⭐ **Abre `kind` como filho de `row`**: ao lado da linha (à esquerda se à direita não couber),
/// com a primeira linha dele à altura dela; o pai fica. Fora de uma cascata, abre sozinho.
pub fn open_cascade(hero: &mut HeroScreen, row: NodeId, kind: ContextMenuKind) {
    let (x, y) = match hero.hit_index.rect_for(row) {
        Some(r) => {
            let folga = Spacing::Xs.px();
            let vp = hero.last_viewport;
            let w = super::menu_barra_linha::largura(&hero.store, kind);
            let (direita, esquerda) = (r.x + r.w + folga, r.x - folga - w);
            let cabe = direita + w <= vp.x + vp.w;
            let x = if cabe || esquerda < vp.x {
                direita
            } else {
                esquerda
            };
            (x, r.y - pad_y())
        }
        None => super::chrome::cascade_anchor(hero, row),
    };
    let req = ContextMenuRequest { x, y, kind };
    match level_of_row(&hero.store, row) {
        Some(depth) => hero.store.open_submenu_at(depth, row, req),
        None => hero.store.open_context_menu(req),
    }
}

/// O rectângulo que um nível ocupa: a união das linhas dele no quadro pintado.
fn level_rect(hero: &HeroScreen, kind: ContextMenuKind) -> Option<Rect> {
    crate::widget::cromo_barra::uniao(
        level_rows(&hero.store, kind)
            .into_iter()
            .filter_map(|id| hero.hit_index.rect_for(id)),
    )
}

/// Quantas posições o rasto guarda: o vértice do triângulo é a mais velha. Uma amostra só não chega
/// — um rato real anda aos degraus de 1 px, e um degrau vertical (0, 1) cai fora de qualquer
/// triângulo (o defeito do smoke do dono, 12/11).
const RASTO: usize = 4;

/// Quanto o ponteiro parado numa irmã espera antes de ela abrir o submenu dela, em segundos: o
/// token de duração da casa (240 ms; o «delay» do jQuery menu-aim e do macOS anda pelos 300).
const ESPERA_S: f64 = ph2d_tokens::Duration::Default.secs() as f64;

/// ⭐ **O ponteiro vai A CAMINHO do filho?** — dentro do triângulo entre onde estava (o mais velho
/// do rasto) e a borda próxima do filho, com folga de duas linhas em cima e em baixo. A diagonal por
/// cima das irmãs não troca de submenu.
fn aiming(antes: [f32; 2], agora: [f32; 2], filho: Rect) -> bool {
    if antes == agora {
        return true;
    }
    let x = if filho.x >= antes[0] {
        filho.x
    } else {
        filho.x + filho.w
    };
    let folga = ROW_H_PX * 2.0;
    let (a, b) = ([x, filho.y - folga], [x, filho.y + filho.h + folga]);
    let lado = |p: [f32; 2], q: [f32; 2], r: [f32; 2]| {
        (p[0] - r[0]) * (q[1] - r[1]) - (q[0] - r[0]) * (p[1] - r[1])
    };
    let (d1, d2, d3) = (
        lado(agora, antes, a),
        lado(agora, a, b),
        lado(agora, b, antes),
    );
    let neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(neg && pos)
}

/// ⭐ **O ponteiro mexeu-se com a cascata aberta**: a linha sob ele abre o submenu dela, ou fecha
/// os níveis abaixo do dela — salvo se ele vai a caminho do submenu aberto: aí a troca espera
/// [`ESPERA_S`] parado na linha ([`tick`]). `antes` é onde o ponteiro estava antes deste evento.
pub fn pointer(hero: &mut HeroScreen, antes: Option<[f32; 2]>, event: PointerEvent) {
    if event.kind == PointerKind::Down {
        hero.store.menu_alt_disarm();
    }
    if event.kind != PointerKind::Move {
        return;
    }
    let chain = bar_chain(&hero.store);
    if chain.is_empty() {
        return;
    }
    let agora = [event.x, event.y];
    let aim = hero.store.menu_aim();
    if aim.rasto.is_empty()
        && let Some(a) = antes
    {
        aim.rasto.push_back(a);
    }
    let vertice = aim.rasto.front().copied();
    aim.rasto.push_back(agora);
    while aim.rasto.len() > RASTO {
        aim.rasto.pop_front();
    }
    let Some(hot) = hero.store.hot_id() else {
        return;
    };
    let Some(level) = level_of_row(&hero.store, hot) else {
        hero.store.menu_aim().pendente = None;
        return;
    };
    if hero.store.menu_open_row(level) == Some(hot) {
        // De volta à linha-mãe: o filho dela fica, os netos saem.
        hero.store.menu_aim().pendente = None;
        hero.store.close_submenus_below(level + 1);
        return;
    }
    if let (Some(filho), Some(v)) = (chain.get(level + 1), vertice)
        && let Some(r) = level_rect(hero, filho.kind)
        && aiming(v, agora, r)
    {
        // A caminho do filho: a irmã só abre se o ponteiro ficar nela.
        let aim = hero.store.menu_aim();
        if aim.pendente.map(|(id, _)| id) != Some(hot) {
            aim.pendente = Some((hot, ESPERA_S));
        }
        return;
    }
    troca(hero, hot, level);
}

/// A linha `hot` do nível `level` toma a vez: abre o submenu dela, ou fecha os de baixo.
fn troca(hero: &mut HeroScreen, hot: NodeId, level: usize) {
    hero.store.menu_aim().pendente = None;
    if opens_submenu(hot) && !hero.store.menu_row_is_unavailable(hot) {
        hero.apply_event(WidgetEvent::Click(hot));
    } else {
        hero.store.close_submenus_below(level);
    }
}

/// O relógio da mira (uma vez por quadro, `dt` de parede): o ponteiro que ficou [`ESPERA_S`] numa
/// irmã a caminho do filho troca para ela.
pub fn tick(hero: &mut HeroScreen, dt: f64) {
    let Some((row, falta)) = hero.store.menu_aim().pendente else {
        return;
    };
    let falta = falta - dt;
    if falta > 0.0 {
        hero.store.menu_aim().pendente = Some((row, falta));
        return;
    }
    hero.store.menu_aim().pendente = None;
    if hero.store.hot_id() == Some(row)
        && let Some(level) = level_of_row(&hero.store, row)
        && hero.store.menu_open_row(level) != Some(row)
    {
        troca(hero, row, level);
    }
}

/// Abre o «Menu» por baixo do botão dele, ou fecha-o se já está aberto.
pub fn toggle_root(hero: &mut HeroScreen) {
    if !bar_chain(&hero.store).is_empty() {
        hero.store.close_context_menu();
        return;
    }
    let Some(r) = hero.hit_index.rect_for(ids::MENUBAR_W) else {
        return;
    };
    hero.store.open_context_menu(ContextMenuRequest {
        x: r.x,
        y: r.y + r.h + Spacing::Xs.px(),
        kind: ContextMenuKind::MenuBarRoot,
    });
    hero.store.set_hot(Some(ids::MENUBAR_FILE));
}

/// A porta da SHELL, antes de todo atalho dela (que só passa ao hero algumas teclas, e só ao
/// descer): a mesma lei de [`key`], com o `Alt` dito à parte (o mapa de teclas do editor não o tem).
pub fn shell_key(
    hero: &mut HeroScreen,
    keycode: Option<u32>,
    alt: bool,
    (pressed, repeat): (bool, bool),
    modifiers: ph2d_host::Modifiers,
) -> bool {
    let kind = match (pressed, repeat) {
        (true, true) => KeyKind::Repeat,
        (true, false) => KeyKind::Down,
        _ => KeyKind::Up,
    };
    let keycode = if alt {
        ALT_KEYS[0]
    } else {
        keycode.unwrap_or_default()
    };
    // O Esc segue o caminho de sempre (fecha o menu no despacho).
    key(
        hero,
        KeyEvent {
            keycode,
            modifiers,
            kind,
            timestamp_ns: 0,
        },
    )
}

/// ⭐ **O teclado dos menus da barra.** `Alt` sozinho abre/fecha o «Menu»; aberto, ↑ ↓ andam pelas
/// linhas, → abre o submenu, ← volta ao pai, Enter escolhe. Devolve `true` se a tecla era do menu.
pub fn key(hero: &mut HeroScreen, event: KeyEvent) -> bool {
    let alt = ALT_KEYS.contains(&event.keycode);
    match event.kind {
        KeyKind::Down if alt => {
            let m = event.modifiers;
            if !(m.ctrl || m.shift || m.meta) {
                hero.store.menu_alt_arm();
            }
            return false;
        }
        KeyKind::Up if alt => {
            let abre = hero.store.menu_alt_fire();
            if abre {
                toggle_root(hero);
            }
            return abre;
        }
        KeyKind::Repeat if alt => return false,
        KeyKind::Up => return false,
        _ => hero.store.menu_alt_disarm(),
    }
    let chain = bar_chain(&hero.store);
    let Some(ponta) = chain.last() else {
        return false;
    };
    let depth = chain.len() - 1;
    let rows: Vec<NodeId> = level_rows(&hero.store, ponta.kind)
        .into_iter()
        .filter(|id| !hero.store.menu_row_is_unavailable(*id))
        .collect();
    let at = hero
        .store
        .hot_id()
        .and_then(|h| rows.iter().position(|r| *r == h));
    match event.keycode {
        k::KEY_ARROW_DOWN | k::KEY_ARROW_UP if !rows.is_empty() => {
            let n = rows.len();
            let next = match (at, event.keycode == k::KEY_ARROW_DOWN) {
                (Some(i), true) => (i + 1) % n,
                (Some(i), false) => (i + n - 1) % n,
                (None, true) => 0,
                (None, false) => n - 1,
            };
            hero.store.set_hot(Some(rows[next]));
        }
        k::KEY_ARROW_RIGHT | k::KEY_ENTER => {
            let Some(row) = at.map(|i| rows[i]) else {
                return event.keycode == k::KEY_ARROW_RIGHT;
            };
            let abre = opens_submenu(row);
            if abre || event.keycode == k::KEY_ENTER {
                hero.apply_event(WidgetEvent::Click(row));
            }
            if abre && let Some(nova) = hero.store.context_menu() {
                let first = level_rows(&hero.store, nova.kind)
                    .into_iter()
                    .find(|id| !hero.store.menu_row_is_unavailable(*id));
                hero.store.set_hot(first);
            }
        }
        k::KEY_ARROW_LEFT => {
            if depth > 0 {
                let mae = hero.store.menu_open_row(depth - 1);
                hero.store.close_submenus_below(depth - 1);
                hero.store.set_hot(mae);
            }
        }
        _ => return false,
    }
    true
}

#[cfg(test)]
#[path = "menu_cascade_tests.rs"]
mod tests;
