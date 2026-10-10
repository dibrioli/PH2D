//! ⭐ **A equipa e a saída no ecrã** (MiroClone W8, `docs/MiroClone/06_equipa_e_saida.md`): comentar
//! (K1–K10), as versões (R1–R5), a apresentação (P1–P6) e o exportar/importar (E1–E7, X1–X4). Módulo
//! filho da navegação ([`super`]): a shell só compõe, e o diálogo de ficheiro é da família do quadro
//! (o pedido em `Team::io`).

use std::time::Instant;

use super::super::HeroScreen;
use crate::documents::team::{ExportChoice, Format, IoRequest, OpenThread, Scope};
use crate::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use crate::widget::ButtonState;
use ph2d_a11y::NodeId;
use ph2d_board_model::{BoardId, Pin, ThreadId};

#[path = "board_team_paint.rs"]
mod paint;
#[path = "board_team_panels.rs"]
mod panels;
#[path = "board_team_show.rs"]
pub mod show;

pub use paint::{bubble_rect, paint};

/// Um botão de tamanho fixo da equipa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeamButton {
    /// K5 — resolver ou reabrir o fio aberto.
    CardResolve,
    CardClose,
    /// O fundo do cartão do fio (toma o clique).
    CardSurface,
    ListClose,
    /// K5 — os resolvidos à vista.
    ListShowResolved,
    /// K10 — mudar o nome de quem escreve.
    ListAuthor,
    ListSurface,
    VersionsSave,
    VersionsClose,
    VersionsSurface,
    ExportPng,
    ExportSvg,
    ExportPdf,
    ExportExcalidraw,
    ScopeBoard,
    ScopeSelection,
    ScopeFrames,
    SizeSmall,
    SizeMedium,
    SizeLarge,
    /// E7 — fundo transparente.
    Transparent,
    /// Exportar (o diálogo de ficheiro é da família do quadro).
    ExportGo,
    ExportClose,
    ExportSurface,
    ShowPrev,
    ShowNext,
    ShowExit,
    /// A apresentação inteira toma o clique (nada por baixo se carrega).
    ShowSurface,
}

const ALL: [TeamButton; 28] = [
    TeamButton::CardResolve,
    TeamButton::CardClose,
    TeamButton::CardSurface,
    TeamButton::ListClose,
    TeamButton::ListShowResolved,
    TeamButton::ListAuthor,
    TeamButton::ListSurface,
    TeamButton::VersionsSave,
    TeamButton::VersionsClose,
    TeamButton::VersionsSurface,
    TeamButton::ExportPng,
    TeamButton::ExportSvg,
    TeamButton::ExportPdf,
    TeamButton::ExportExcalidraw,
    TeamButton::ScopeBoard,
    TeamButton::ScopeSelection,
    TeamButton::ScopeFrames,
    TeamButton::SizeSmall,
    TeamButton::SizeMedium,
    TeamButton::SizeLarge,
    TeamButton::Transparent,
    TeamButton::ExportGo,
    TeamButton::ExportClose,
    TeamButton::ExportSurface,
    TeamButton::ShowPrev,
    TeamButton::ShowNext,
    TeamButton::ShowExit,
    TeamButton::ShowSurface,
];

/// O que um id derivado é.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    Button(TeamButton),
    /// A bolha de um fio no quadro.
    Bubble(ThreadId),
    /// A linha de um fio na lista.
    Row(ThreadId),
    /// K4 — editar / apagar a mensagem `n` do fio aberto.
    Edit(usize),
    Delete(usize),
    /// R2/R3 — restaurar, apagar, renomear a versão `n`.
    Restore(usize),
    DropVersion(usize),
    RenameVersion(usize),
}

/// ⛔ Bits 44–47 diferentes dos da navegação e da facilitação (`0x4e61_1…`–`0x4e61_5…`): o idioma.
const SALT: u64 = 0x4e61_6000_0000_0000;
const SALT_BUBBLE: u64 = 0x4e61_7000_0000_0000;
const SALT_ROW: u64 = 0x4e61_8000_0000_0000;
const SALT_MSG: u64 = 0x4e61_9000_0000_0000;
const SALT_VER: u64 = 0x4e61_a000_0000_0000;

/// O id de um alvo da equipa.
#[must_use]
pub fn node(h: Hit) -> NodeId {
    let pair = |kind: u64, n: usize| (kind << 32) | n as u64;
    NodeId(match h {
        Hit::Button(b) => SALT ^ (ALL.iter().position(|x| *x == b).expect("na tabela") as u64 + 1),
        Hit::Bubble(t) => SALT_BUBBLE ^ t.0,
        Hit::Row(t) => SALT_ROW ^ t.0,
        Hit::Edit(n) => SALT_MSG ^ pair(1, n),
        Hit::Delete(n) => SALT_MSG ^ pair(2, n),
        Hit::Restore(n) => SALT_VER ^ pair(1, n),
        Hit::DropVersion(n) => SALT_VER ^ pair(2, n),
        Hit::RenameVersion(n) => SALT_VER ^ pair(3, n),
    })
}

/// O alvo de um id (`None` = não é da equipa). Cada descodificador exige os bits acima do que carrega
/// a zero (a lição da W5: saltos parecidos liam-se uns pelos outros).
#[must_use]
pub fn hit_of(id: NodeId) -> Option<Hit> {
    let low = |salt: u64| {
        let v = id.0 ^ salt;
        (v >> 40 == 0 && v != 0).then_some(v)
    };
    if let Some(v) = low(SALT) {
        return ALL.get(usize::try_from(v - 1).ok()?).map(|b| Hit::Button(*b));
    }
    if let Some(v) = low(SALT_BUBBLE) {
        return Some(Hit::Bubble(ThreadId(v)));
    }
    if let Some(v) = low(SALT_ROW) {
        return Some(Hit::Row(ThreadId(v)));
    }
    let split = |v: u64| (v >> 32, usize::try_from(v & 0xffff_ffff).unwrap_or(usize::MAX));
    if let Some((k, n)) = low(SALT_MSG).map(split) {
        return match k {
            1 => Some(Hit::Edit(n)),
            2 => Some(Hit::Delete(n)),
            _ => None,
        };
    }
    if let Some((k, n)) = low(SALT_VER).map(split) {
        return match k {
            1 => Some(Hit::Restore(n)),
            2 => Some(Hit::DropVersion(n)),
            3 => Some(Hit::RenameVersion(n)),
            _ => None,
        };
    }
    None
}

fn tooltip_key(b: TeamButton) -> &'static str {
    match b {
        TeamButton::CardResolve => "board.comment.resolve",
        TeamButton::CardClose | TeamButton::ListClose => "board.comment.close",
        TeamButton::ListShowResolved => "board.comment.show_resolved",
        TeamButton::ListAuthor => "board.comment.author_tip",
        TeamButton::VersionsSave => "board.version.save",
        TeamButton::VersionsClose | TeamButton::ExportClose => "board.team.close",
        TeamButton::ExportPng => "board.export.png",
        TeamButton::ExportSvg => "board.export.svg",
        TeamButton::ExportPdf => "board.export.pdf",
        TeamButton::ExportExcalidraw => "board.export.excalidraw",
        TeamButton::ScopeBoard => "board.export.scope_board",
        TeamButton::ScopeSelection => "board.export.scope_selection",
        TeamButton::ScopeFrames => "board.export.scope_frames",
        TeamButton::SizeSmall => "board.export.small",
        TeamButton::SizeMedium => "board.export.medium",
        TeamButton::SizeLarge => "board.export.large",
        TeamButton::Transparent => "board.export.transparent",
        TeamButton::ExportGo => "board.export.go",
        TeamButton::ShowPrev => "board.present.prev",
        TeamButton::ShowNext => "board.present.next",
        TeamButton::ShowExit => "board.present.exit",
        TeamButton::CardSurface
        | TeamButton::ListSurface
        | TeamButton::VersionsSurface
        | TeamButton::ExportSurface
        | TeamButton::ShowSurface => "board.team.surface",
    }
}

pub(crate) fn register(store: &mut WidgetStore, id: NodeId) {
    if store.button_state(id).is_none() {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// Regista os botões fixos (pintado ⇒ clicável). Chamado pelo `board_nav::populate`.
pub fn populate(store: &mut WidgetStore) {
    for b in ALL {
        let id = node(Hit::Button(b));
        register(store, id);
        store.set_tooltip(id, ph2d_i18n::tr(tooltip_key(b)));
    }
}

/// O instante de agora em segundos UNIX (as mensagens e as versões guardam-no; a vista mostra «há
/// quanto tempo», que não pede fuso).
#[must_use]
pub fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

/// «agora», «há 5 min», «há 3 h», «há 2 d».
#[must_use]
pub fn ago(at: i64, now: i64) -> String {
    let s = (now - at).max(0);
    let n = |v: i64| v.to_string();
    match s {
        0..60 => ph2d_i18n::tr("board.ago.now").to_owned(),
        60..3600 => ph2d_i18n::tr_with("board.ago.min", &[("n", &n(s / 60))]),
        3600..86_400 => ph2d_i18n::tr_with("board.ago.hour", &[("n", &n(s / 3600))]),
        _ => ph2d_i18n::tr_with("board.ago.day", &[("n", &n(s / 86_400))]),
    }
}

/// K10 — o nome de quem escreve, ou «Me» enquanto não há nenhum.
fn me(hero: &HeroScreen) -> String {
    let a = hero.documents.live.team.author.trim();
    if a.is_empty() {
        ph2d_i18n::tr("board.comment.me").to_owned()
    } else {
        a.to_owned()
    }
}

/// K1 — o botão «Comment» da barra curta e a tecla `C`: arma ou desarma a ferramenta.
pub fn toggle_comment_tool(hero: &mut HeroScreen) {
    let t = &mut hero.documents.live.team;
    t.commenting = !t.commenting;
    if t.commenting {
        t.hide_comments = false;
    }
}

/// K8 — o botão «Comments» do canto: a lista.
pub fn toggle_list(hero: &mut HeroScreen) {
    let t = &mut hero.documents.live.team;
    t.list_open = !t.list_open;
}

/// Abre o fio `t` do quadro activo no cartão: lido por mim (K6), o compositor pronto para responder.
pub fn open_thread(hero: &mut HeroScreen, t: ThreadId) {
    let author = me(hero);
    let Some(board) = hero.documents.active_board_mut() else {
        return;
    };
    let id = board.id;
    board.team.mark_read(t, &author);
    let team = &mut hero.documents.live.team;
    team.open.insert(id, OpenThread::Thread(t));
    team.editing = None;
    super::open_field(&mut hero.store, crate::ids::BOARD_COMMENT_INPUT, String::new());
}

fn close_card(hero: &mut HeroScreen) {
    if let Some(id) = hero.documents.active() {
        hero.documents.live.team.open.remove(&id);
    }
    hero.documents.live.team.editing = None;
    if hero.store.focus_id() == Some(crate::ids::BOARD_COMMENT_INPUT) {
        hero.store.set_focus(None);
    }
}

/// ⭐ K1/K2 — com a ferramenta armada, um carregar no quadro abre um fio NOVO preso ao objecto de
/// cima sob ele (o mesmo pick do clique), ou ao ponto. `true` = consumido (nada chega ao editor).
pub fn pointer_down(hero: &mut HeroScreen, x: f32, y: f32) -> bool {
    if !hero.documents.live.team.commenting {
        return false;
    }
    let area = super::super::board_view::area_of(hero.last_canvas);
    let theme = hero.theme;
    let Some((board, live)) = hero.documents.active_parts() else {
        return false;
    };
    let world = board.camera.to_world(area, [f64::from(x), f64::from(y)]);
    let p = ph2d_board_edit::Pointer {
        world,
        px: 1.0 / board.camera.zoom,
        mods: ph2d_board_edit::Mods::default(),
    };
    let ed = super::super::board_view::editor(&mut live.editor, theme);
    let on = ed.hit(&board.doc, p);
    let pin = Pin::at(&board.doc, on, world);
    live.team.open.insert(board.id, OpenThread::New(pin));
    live.team.editing = None;
    super::open_field(&mut hero.store, crate::ids::BOARD_COMMENT_INPUT, String::new());
    true
}

/// ⭐ K4 — o compositor publicou: um fio novo, uma resposta, ou a edição da minha mensagem.
fn post(hero: &mut HeroScreen) {
    let text = super::field_text(&hero.store, crate::ids::BOARD_COMMENT_INPUT);
    let author = me(hero);
    let now = now_unix();
    let editing = hero.documents.live.team.editing.take();
    let Some(board) = hero.documents.active_board_mut() else {
        return;
    };
    let id = board.id;
    let open = hero.documents.live.team.open.get(&id).copied();
    let Some(board) = hero.documents.active_board_mut() else {
        return;
    };
    match open {
        Some(OpenThread::New(pin)) => {
            if let Some(t) = board.team.open(pin, &author, &text, now) {
                hero.documents
                    .live
                    .team
                    .open
                    .insert(id, OpenThread::Thread(t));
            }
        }
        Some(OpenThread::Thread(t)) => match editing {
            Some(n) => _ = board.team.edit(t, n, &author, &text),
            None => _ = board.team.reply(t, &author, &text, now),
        },
        None => return,
    }
    super::open_field(&mut hero.store, crate::ids::BOARD_COMMENT_INPUT, String::new());
}

/// ⭐ K1/K8 — as teclas da equipa, antes das do editor: `C` arma a ferramenta, `Shift+C` mostra/esconde
/// os comentários, `Esc` larga a ferramenta e o cartão. `Some` = tomada.
pub fn key(
    hero: &mut HeroScreen,
    k: super::super::board_keys::BoardKey,
    mods: ph2d_host::Modifiers,
) -> Option<super::super::board_keys::KeyOutcome> {
    use super::super::board_keys::{BoardKey, KeyOutcome};
    let taken = Some(KeyOutcome {
        consumed: true,
        copy: None,
    });
    let ctrl = mods.ctrl || mods.meta || mods.alt;
    match k {
        BoardKey::Char('c' | 'C') if !ctrl && mods.shift => {
            let t = &mut hero.documents.live.team;
            t.hide_comments = !t.hide_comments;
            taken
        }
        BoardKey::Char('c') if !ctrl => {
            // Com uma nota seleccionada a letra escreve nela (o idioma do Miro, o mesmo do `M`).
            let (board, live) = hero.documents.active_parts()?;
            if live
                .editor
                .as_ref()
                .is_some_and(|e| e.would_type_into_note(&board.doc))
            {
                return None;
            }
            toggle_comment_tool(hero);
            taken
        }
        BoardKey::Escape => {
            let active = hero.documents.active();
            let t = &mut hero.documents.live.team;
            let open = active.is_some_and(|b| t.open.contains_key(&b));
            if !(t.commenting || open) {
                return None;
            }
            t.commenting = false;
            close_card(hero);
            taken
        }
        _ => None,
    }
}

/// ⭐ As linhas do menu da aba (W8): apresentar, versões, exportar, importar. `false` = não é daqui.
pub fn menu(hero: &mut HeroScreen, board: BoardId, id: NodeId) -> bool {
    let ids = crate::ids::CTX_MENU_BOARD_PRESENT;
    if id == ids {
        hero.documents.activate(Some(board));
        show::start(hero);
    } else if id == crate::ids::CTX_MENU_BOARD_VERSIONS {
        hero.documents.activate(Some(board));
        panels::open_versions(hero);
    } else if id == crate::ids::CTX_MENU_BOARD_EXPORT {
        hero.documents.activate(Some(board));
        hero.documents.live.team.export = Some(ExportChoice::default());
    } else if id == crate::ids::CTX_MENU_BOARD_IMPORT {
        hero.documents.live.team.io = Some(IoRequest::Import);
    } else {
        return false;
    }
    true
}

/// O texto de um aviso da equipa (W5: nunca só o terminal) — a família do quadro também o usa.
pub fn notice(hero: &mut HeroScreen, key: &'static str) {
    hero.documents.live.notice = Some((key, Instant::now()));
}

/// ⭐ Os cliques e os campos da equipa. Corre no pré-despacho (ids derivados).
pub fn apply_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    use crate::ids::{BOARD_AUTHOR_INPUT, BOARD_COMMENT_INPUT, BOARD_VERSION_INPUT};
    match event {
        WidgetEvent::Submit(id) if id == BOARD_COMMENT_INPUT => {
            post(hero);
            return true;
        }
        WidgetEvent::Cancel(id) if id == BOARD_COMMENT_INPUT => {
            close_card(hero);
            return true;
        }
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id) if id == BOARD_AUTHOR_INPUT => {
            let name = super::field_text(&hero.store, BOARD_AUTHOR_INPUT);
            let name = name.trim();
            let t = &mut hero.documents.live.team;
            if !name.is_empty() && name != t.author {
                t.author = name.to_owned();
                t.author_dirty = true;
            }
            t.editing_author = false;
            return true;
        }
        WidgetEvent::Cancel(id) if id == BOARD_AUTHOR_INPUT => {
            hero.documents.live.team.editing_author = false;
            return true;
        }
        WidgetEvent::Submit(id) if id == BOARD_VERSION_INPUT => {
            panels::version_submit(hero);
            return true;
        }
        WidgetEvent::Cancel(id) if id == BOARD_VERSION_INPUT => {
            hero.documents.live.team.renaming_version = None;
            return true;
        }
        WidgetEvent::Blur(id) | WidgetEvent::TextChanged(id)
            if id == BOARD_COMMENT_INPUT || id == BOARD_VERSION_INPUT =>
        {
            return true;
        }
        _ => {}
    }
    let (WidgetEvent::Click(id) | WidgetEvent::DoubleClick(id)) = event else {
        return false;
    };
    let Some(h) = hit_of(id) else {
        return false;
    };
    let double = matches!(event, WidgetEvent::DoubleClick(_));
    match h {
        Hit::Bubble(t) | Hit::Row(t) => {
            if matches!(h, Hit::Row(_)) {
                paint::go_to_thread(hero, t);
            }
            open_thread(hero, t);
        }
        Hit::Edit(n) => paint::begin_edit(hero, n),
        Hit::Delete(n) => {
            let author = me(hero);
            let open = hero.documents.active().and_then(|b| hero.documents.live.team.open.get(&b).copied());
            if let (Some(OpenThread::Thread(t)), Some(board)) = (open, hero.documents.active_board_mut())
                && board.team.delete(t, n, &author)
                && board.team.thread(t).is_none()
            {
                close_card(hero);
            }
        }
        Hit::Restore(n) | Hit::DropVersion(n) | Hit::RenameVersion(n) => {
            panels::version_click(hero, h, n, double);
        }
        Hit::Button(b) => button(hero, b),
    }
    true
}

fn button(hero: &mut HeroScreen, b: TeamButton) {
    let active = hero.documents.active();
    let t = &mut hero.documents.live.team;
    let set = |c: &mut Option<ExportChoice>, f: &dyn Fn(&mut ExportChoice)| {
        if let Some(c) = c.as_mut() {
            f(c);
        }
    };
    match b {
        TeamButton::CardResolve => {
            let open = active.and_then(|id| t.open.get(&id).copied());
            let show = t.show_resolved;
            if let (Some(OpenThread::Thread(th)), Some(board)) = (open, hero.documents.active_board_mut()) {
                let now = !board.team.thread(th).is_some_and(|x| x.resolved);
                board.team.set_resolved(th, now);
                // K5 — resolvido esconde-se do quadro: o cartão fecha (a não ser com «Show resolved»).
                if now && !show {
                    close_card(hero);
                }
            }
        }
        TeamButton::CardClose => close_card(hero),
        TeamButton::ListClose => t.list_open = false,
        TeamButton::ListShowResolved => t.show_resolved = !t.show_resolved,
        TeamButton::ListAuthor => {
            t.editing_author = true;
            let name = t.author.clone();
            super::open_field(&mut hero.store, crate::ids::BOARD_AUTHOR_INPUT, name);
        }
        TeamButton::VersionsSave => panels::version_submit(hero),
        TeamButton::VersionsClose => {
            t.versions_open = false;
            t.renaming_version = None;
        }
        TeamButton::ExportPng => set(&mut t.export, &|c| c.format = Format::Png),
        TeamButton::ExportSvg => set(&mut t.export, &|c| c.format = Format::Svg),
        TeamButton::ExportPdf => set(&mut t.export, &|c| c.format = Format::Pdf),
        TeamButton::ExportExcalidraw => set(&mut t.export, &|c| c.format = Format::Excalidraw),
        TeamButton::ScopeBoard => set(&mut t.export, &|c| c.scope = Scope::Board),
        TeamButton::ScopeSelection => set(&mut t.export, &|c| c.scope = Scope::Selection),
        TeamButton::ScopeFrames => set(&mut t.export, &|c| c.scope = Scope::Frames),
        TeamButton::SizeSmall => set(&mut t.export, &|c| c.size = 0),
        TeamButton::SizeMedium => set(&mut t.export, &|c| c.size = 1),
        TeamButton::SizeLarge => set(&mut t.export, &|c| c.size = 2),
        TeamButton::Transparent => set(&mut t.export, &|c| c.transparent = !c.transparent),
        TeamButton::ExportGo => panels::export_go(hero),
        TeamButton::ExportClose => t.export = None,
        TeamButton::ShowPrev => show::step(hero, false),
        TeamButton::ShowNext => show::step(hero, true),
        TeamButton::ShowExit => show::stop(hero),
        TeamButton::CardSurface
        | TeamButton::ListSurface
        | TeamButton::VersionsSurface
        | TeamButton::ExportSurface
        | TeamButton::ShowSurface => {}
    }
}

#[cfg(test)]
#[path = "board_team_tests.rs"]
mod tests;
