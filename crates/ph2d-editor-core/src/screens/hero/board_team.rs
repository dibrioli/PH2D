//! ⭐ **A equipa e a saída no ecrã** (MiroClone W8, `docs/MiroClone/06_equipa_e_saida.md`): comentar
//! (K1–K10), as versões (R1–R5), a apresentação (P1–P6) e o exportar/importar (E1–E7, X1–X4). Módulo
//! filho da navegação ([`super`]): a shell só compõe, e o diálogo de ficheiro é da família do quadro
//! (o pedido em `Team::io`).

use std::time::Instant;

use super::super::HeroScreen;
use crate::documents::team::{Confirm, ExportChoice, Format, IoRequest, OpenThread, Scope};
use crate::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use crate::widget::ButtonState;
use ph2d_a11y::NodeId;
use ph2d_board_model::{BoardId, Pin, TeamOp, ThreadId};
use ph2d_host::PointerButton;

#[path = "board_team_bubbles.rs"]
mod bubbles;
/// W11 — o quadro ao vivo (o painel «Live», os avatares, seguir, os cursores alheios).
#[path = "board_live.rs"]
pub mod live_board;
/// W23 B14 — o `@` do compositor.
#[path = "board_team_mention.rs"]
pub mod mention;
/// W23 B14 — o aviso, a bolinha e o «por ler para mim».
#[path = "board_team_notify.rs"]
pub mod notify;
#[path = "board_team_paint.rs"]
mod paint;
#[path = "board_team_panels.rs"]
mod panels;
#[path = "board_team_show.rs"]
pub mod show;

pub use bubbles::{bubble_down, bubble_drag, bubble_rect, bubble_up};
pub use paint::paint;

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
    /// A lista ou as versões não cabem: as filas de antes / de depois.
    PageUp,
    PageDown,
    /// W23 B14 — a lista só com os fios PARA mim.
    ListForYou,
}

const ALL: [TeamButton; 31] = [
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
    TeamButton::PageUp,
    TeamButton::PageDown,
    TeamButton::ListForYou,
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
    /// W23 B14 — a pessoa `n` da lista do `@`.
    Mention(usize),
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
        Hit::Mention(n) => SALT_MSG ^ pair(3, n),
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
        return ALL
            .get(usize::try_from(v - 1).ok()?)
            .map(|b| Hit::Button(*b));
    }
    if let Some(v) = low(SALT_BUBBLE) {
        return Some(Hit::Bubble(ThreadId(v)));
    }
    if let Some(v) = low(SALT_ROW) {
        return Some(Hit::Row(ThreadId(v)));
    }
    let split = |v: u64| {
        (
            v >> 32,
            usize::try_from(v & u64::from(u32::MAX)).unwrap_or(usize::MAX),
        )
    };
    if let Some((k, n)) = low(SALT_MSG).map(split) {
        return match k {
            1 => Some(Hit::Edit(n)),
            2 => Some(Hit::Delete(n)),
            3 => Some(Hit::Mention(n)),
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
        TeamButton::ListForYou => "board.comment.for_you_tip",
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
        TeamButton::PageUp => "board.team.page_up",
        TeamButton::PageDown => "board.team.page_down",
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
    live_board::populate(store);
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

/// K1 — outra ferramenta escolhida (barra ou tecla): a de comentar larga-se (uma ferramenta de cada
/// vez, o idioma da barra).
pub fn disarm_comment_tool(hero: &mut HeroScreen) {
    hero.documents.live.team.commenting = false;
}

/// Um painel do canto de cada vez: abrir um fecha os outros (a lista, as versões, o exportar).
pub(super) fn only_panel(t: &mut crate::documents::team::Team) {
    t.list_open = false;
    t.versions_open = false;
    t.export = None;
    t.renaming_version = None;
    t.editing_author = false;
    t.confirm = None;
    t.from = 0;
}

/// K8 — o botão «Comments» do canto: a lista.
pub fn toggle_list(hero: &mut HeroScreen) {
    let t = &mut hero.documents.live.team;
    let open = !t.list_open;
    only_panel(t);
    t.list_open = open;
}

/// O y do texto de `font` px centrado numa fila de altura `row` que começa em `y`.
pub(super) fn text_y(y: f32, row: f32, font: f32) -> f32 {
    y + (row - font) / 2.0
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
    super::open_field(
        &mut hero.store,
        crate::ids::BOARD_COMMENT_INPUT,
        String::new(),
    );
}

/// W23 B14 — o «Show» de um aviso: o quadro do fio activo, a vista nele, o cartão aberto (lido).
pub(crate) fn show_thread(hero: &mut HeroScreen, board: BoardId, t: ThreadId) {
    hero.documents.activate(Some(board));
    hero.documents.live.team.commenting = false;
    paint::go_to_thread(hero, t);
    open_thread(hero, t);
}

fn close_card(hero: &mut HeroScreen) {
    hero.documents.live.team.calling.clear();
    if let Some(id) = hero.documents.active() {
        hero.documents.live.team.open.remove(&id);
    }
    hero.documents.live.team.editing = None;
    hero.documents.live.team.confirm = None;
    if hero.store.focus_id() == Some(crate::ids::BOARD_COMMENT_INPUT) {
        hero.store.set_focus(None);
    }
}

/// ⭐ K1/K2 — com a ferramenta armada, um carregar PRIMÁRIO no quadro abre um fio NOVO preso ao
/// objecto de cima sob ele (o mesmo pick do clique), ou ao ponto. O botão do meio, o direito e o
/// `Espaço`+arrastar continuam a deslocar a vista. `true` = consumido (nada chega ao editor).
pub fn pointer_down(hero: &mut HeroScreen, x: f32, y: f32, button: PointerButton) -> bool {
    let live = &hero.documents.live;
    if !live.team.commenting || button != PointerButton::Primary || live.space {
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
    super::open_field(
        &mut hero.store,
        crate::ids::BOARD_COMMENT_INPUT,
        String::new(),
    );
    true
}

/// ⭐ K4 — o compositor publicou: um fio novo, uma resposta, ou a edição da minha mensagem.
fn post(hero: &mut HeroScreen) {
    let text = super::field_text(&hero.store, crate::ids::BOARD_COMMENT_INPUT);
    let author = me(hero);
    let now = now_unix();
    let editing = hero.documents.live.team.editing.take();
    let calling = std::mem::take(&mut hero.documents.live.team.calling);
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
            if let Some(t) = board.team.open_calling(pin, &author, &text, &calling, now) {
                hero.documents
                    .live
                    .team
                    .open
                    .insert(id, OpenThread::Thread(t));
            }
        }
        Some(OpenThread::Thread(t)) => match editing {
            Some((th, m)) if th == t => {
                _ = board.team.edit_calling(t, m, &author, &text, &calling);
            }
            _ => _ = board.team.reply_calling(t, &author, &text, &calling, now),
        },
        None => return,
    }
    super::open_field(
        &mut hero.store,
        crate::ids::BOARD_COMMENT_INPUT,
        String::new(),
    );
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
        let t = &mut hero.documents.live.team;
        only_panel(t);
        t.export = Some(ExportChoice::default());
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
            // W23 B14 — com a lista do `@` aberta, o `Enter` escolhe a pessoa.
            if mention::is_open(hero) {
                mention::submit(hero);
            } else {
                post(hero);
            }
            return true;
        }
        WidgetEvent::Cancel(id) if id == BOARD_COMMENT_INPUT => {
            // `Esc` no compositor larga tudo: o cartão E a ferramenta (o `Esc` do quadro faz o mesmo).
            hero.documents.live.team.commenting = false;
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
    // Qualquer clique desarma o apagar; só o 2.º no MESMO alvo apaga.
    let armed = hero.documents.live.team.confirm.take();
    match h {
        Hit::Bubble(t) | Hit::Row(t) => {
            if matches!(h, Hit::Row(_)) {
                paint::go_to_thread(hero, t);
            }
            open_thread(hero, t);
        }
        Hit::Edit(n) => paint::begin_edit(hero, n),
        Hit::Mention(n) => mention::choose(hero, n),
        Hit::Delete(n) => delete_message(hero, n, armed),
        Hit::Restore(n) | Hit::DropVersion(n) | Hit::RenameVersion(n) => {
            panels::version_click(hero, h, n, double, armed);
        }
        Hit::Button(b) => button(hero, b),
    }
    true
}

/// K4 — o 1.º clique no lixo da linha `n` arma a mensagem que lá está, o 2.º apaga-a. A edição em
/// curso segue a SUA mensagem pelo id: só a apagada a larga.
fn delete_message(hero: &mut HeroScreen, n: usize, armed: Option<Confirm>) {
    let author = me(hero);
    let open = hero
        .documents
        .active()
        .and_then(|b| hero.documents.live.team.open.get(&b).copied());
    let Some(OpenThread::Thread(t)) = open else {
        return;
    };
    let Some(board) = hero.documents.active_board_mut() else {
        return;
    };
    let Some(msg) = board
        .team
        .thread(t)
        .and_then(|th| th.messages.get(n))
        .map(|m| m.id)
    else {
        return;
    };
    if armed != Some(Confirm::Message(t, msg)) {
        hero.documents.live.team.confirm = Some(Confirm::Message(t, msg));
        return;
    }
    if !board.team.delete(t, msg, &author) {
        return;
    }
    if board.team.thread(t).is_none() {
        close_card(hero);
        return;
    }
    let team = &mut hero.documents.live.team;
    if team.editing == Some((t, msg)) {
        team.editing = None;
    }
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
            if let (Some(OpenThread::Thread(th)), Some((board, live))) =
                (open, hero.documents.active_parts())
            {
                let now = !board.team.thread(th).is_some_and(|x| x.resolved);
                // Um passo do desfazer do quadro (o dono, 10/10: «não temos undo para Resolve»).
                let history = live.histories.entry(board.id).or_default();
                history.apply_team(&mut board.team, TeamOp::Resolve(th, now));
                // K5 — resolvido esconde-se do quadro: o cartão fecha (a não ser com «Show resolved»).
                if now && !show {
                    close_card(hero);
                }
            }
        }
        TeamButton::CardClose => close_card(hero),
        TeamButton::ListClose => t.list_open = false,
        TeamButton::ListShowResolved => t.show_resolved = !t.show_resolved,
        TeamButton::ListForYou => t.for_you = !t.for_you,
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
        TeamButton::PageUp => t.from = t.from.saturating_sub(t.page.max(1)),
        TeamButton::PageDown => t.from += t.page.max(1),
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
