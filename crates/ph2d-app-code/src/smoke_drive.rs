//! ⭐ **A porta das FOTOS das cenas 3 a 5, 10 e 11** (`PH2D_CODE_SMOKE=3|4|5|10|11 PH2D_CODE_SMOKE_PASSO=<n>`): o
//! roteiro do smoke conduzido pelas teclas REAIS da aba até ao fim do passo `n`, e pára — cada foto é
//! o passo que diz ser. A tela virtual das fotos ignora eventos sintéticos (`fotografa_cena.sh`), logo
//! o gesto entra pelas portas do artista (`DocumentSurface::key`, `pointer_move`, `pointer`), o cursor
//! anda pelo `go_to` da paleta `:` e o Ctrl carregado vai para o `WidgetStore` como o da janela. Os
//! testes das cenas (na shell, `shells/desktop/tests/it/a_cena_{3,4,5}_do_editor_conduzida.rs`)
//! conduzem os MESMOS roteiros e conferem as agulhas no texto da cena. Os roteiros moram em
//! `ph2d-code-smoke` (a 15 em `ph2d-code-scene-arena`). As teclas `Key`/`Type` vão para o script;
//! `TabKey`/`TabType` para a aba ACTIVA da família (a página de ajuda, quando é ela).

use crate::help::page_of;

use crate::CodeDocs;
use ph2d_code_help::{Anchor, catalog};
use ph2d_editor_core::doc_surface::PointerLook;
use ph2d_editor_core::doc_surface::{DocumentSurface, Key, KeyInput, PointerInput};
use ph2d_host::{Modifiers, PointerButton, PointerKind};
use ph2d_script::ScriptId;
use ph2d_text::TextSystem;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

pub use ph2d_code_scene_arena::steps;
pub use ph2d_code_smoke::{
    Gesture, NONE, STEP_ENV, STEPS, STEPS_4, STEPS_5, STEPS_10, STEPS_11, STEPS_12, STEPS_13,
    Until, VIEW_ENV,
};

/// Quanto um passo espera pelo editor (o assistente, a pausa do aviso) antes de desistir e dizê-lo.
const PATIENCE: Duration = Duration::from_secs(30);

/// O passo pedido pelo ambiente (lido uma vez) e a cena dele; só com as cenas que têm roteiro.
#[must_use]
pub fn armed_step() -> Option<(u32, usize)> {
    static STEP: OnceLock<Option<(u32, usize)>> = OnceLock::new();
    *STEP.get_or_init(|| {
        let n = std::env::var(STEP_ENV).ok()?.trim().parse().ok()?;
        let level = crate::smoke::armed_level()?;
        (n <= steps(level)?.len()).then_some((level, n))
    })
}

/// O condutor: onde vai, e desde quando espera.
#[derive(Debug)]
pub struct Conductor {
    steps: &'static [&'static [Gesture]],
    target: usize,
    step: usize,
    gesture: usize,
    /// Quantas letras do `Type` em curso já entraram.
    typed: usize,
    since: Option<Instant>,
    failed: bool,
    /// O Ctrl que o roteiro tem carregado (`None` = o roteiro nunca lhe tocou).
    ctrl: Option<bool>,
}

impl Conductor {
    /// Conduz a cena 3 até ao fim do passo `target` (1-based; 0 = nada).
    #[must_use]
    pub fn new(target: usize) -> Self {
        Self::scene(STEPS, target)
    }

    /// Conduz o roteiro `steps` até ao fim do passo `target` (1-based; 0 = nada).
    #[must_use]
    pub fn scene(steps: &'static [&'static [Gesture]], target: usize) -> Self {
        Self {
            steps,
            target: target.min(steps.len()),
            step: 0,
            gesture: 0,
            typed: 0,
            since: None,
            failed: false,
            ctrl: None,
        }
    }

    /// O Ctrl que o roteiro tem carregado (`None` = nunca lhe tocou): quem conduz di-lo ao
    /// `WidgetStore` (`set_cmd_held`), como a janela.
    #[must_use]
    pub fn ctrl(&self) -> Option<bool> {
        self.ctrl
    }

    /// Quantos passos estão feitos.
    #[must_use]
    pub fn steps_done(&self) -> usize {
        self.step
    }

    #[must_use]
    pub fn finished(&self) -> bool {
        self.step >= self.target || self.failed
    }

    #[must_use]
    pub fn failed(&self) -> bool {
        self.failed
    }
}

/// O condutor do processo, armado pelo ambiente (do [`crate::frame`]; inerte sem a variável).
/// `.0` = `true` UMA vez, quando o roteiro acabou e a foto pede a cena ([`VIEW_ENV`]); `.1` = o Ctrl
/// que o roteiro tem carregado ([`Conductor::ctrl`]).
pub(crate) fn frame(docs: &mut CodeDocs) -> (bool, Option<bool>) {
    static DRIVE: OnceLock<Mutex<Conductor>> = OnceLock::new();
    static SHOWN: AtomicBool = AtomicBool::new(false);
    preset_breakpoint(docs);
    let Some((level, n)) = armed_step() else {
        return (false, None);
    };
    let Some(steps) = steps(level) else {
        return (false, None);
    };
    let mut c = DRIVE
        .get_or_init(|| Mutex::new(Conductor::scene(steps, n)))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    drive(docs, &mut c);
    let show = c.finished()
        && !c.failed()
        && std::env::var(VIEW_ENV).is_ok_and(|v| v.trim() == "cena")
        && !SHOWN.swap(true, Ordering::Relaxed);
    (show, c.ctrl)
}

/// A mão do roteiro neste quadro: um gesto entrou (a shell conta-o como entrada, como a do artista).
static ACTED: AtomicBool = AtomicBool::new(false);
/// Um [`Gesture::WindowUndo`] à espera de a shell o mandar pelo teclado da janela.
static WINDOW_UNDO: AtomicBool = AtomicBool::new(false);

/// A shell, antes do desfazer do quadro: `(houve gesto, Ctrl+Z da janela pedido)`, e esquece-os.
#[must_use]
pub fn take_hand() -> (bool, bool) {
    (
        ACTED.swap(false, Ordering::Relaxed),
        WINDOW_UNDO.swap(false, Ordering::Relaxed),
    )
}

/// Um quadro do condutor: os gestos até à próxima espera ou ao fim do passo entram já, na aba do
/// 1.º script aberto.
pub fn drive(docs: &mut CodeDocs, c: &mut Conductor) {
    let Some(&id) = docs.open_scripts().first() else {
        return;
    };
    while !c.finished() {
        let gestures = c.steps[c.step];
        let Some(&g) = gestures.get(c.gesture) else {
            c.step += 1;
            c.gesture = 0;
            println!("[code-smoke] passo {} feito", c.step);
            // O passo acaba com o quadro: o seguinte começa no próximo (o estado de cada um vê-se).
            return;
        };
        // Uma espera, ou um gesto cujo alvo ainda não está pintado (o rato, o clique).
        let ready = match g {
            Gesture::Wait(until) => holds(docs, id, until),
            Gesture::Type(s) | Gesture::TabType(s) => {
                let tab = match g {
                    Gesture::TabType(_) => active(docs, id),
                    _ => id.0,
                };
                if let Some(ch) = s.chars().nth(c.typed) {
                    key(docs, tab, Key::Other, NONE, Some(&ch.to_string()));
                    ACTED.store(true, Ordering::Relaxed);
                    c.typed += 1;
                    return;
                }
                c.typed = 0;
                true
            }
            Gesture::Ctrl(held) => {
                c.ctrl = Some(held);
                true
            }
            Gesture::Frame => {
                c.gesture += 1;
                return;
            }
            Gesture::WindowUndo => {
                WINDOW_UNDO.store(true, Ordering::Relaxed);
                true
            }
            _ => act(docs, id, g),
        };
        if !ready {
            let since = *c.since.get_or_insert_with(Instant::now);
            if since.elapsed() > PATIENCE {
                eprintln!(
                    "[code-smoke] passo {}: {g:?} não chegou em {PATIENCE:?} — a cena pára aqui",
                    c.step + 1
                );
                c.failed = true;
            }
            return;
        }
        c.since = None;
        c.gesture += 1;
        if !matches!(g, Gesture::Wait(_)) {
            ACTED.store(true, Ordering::Relaxed);
        }
    }
}

fn holds(docs: &mut CodeDocs, id: ScriptId, until: Until) -> bool {
    match until {
        Until::Problem(s) => docs.problems_of(id).iter().any(|p| p.message.contains(s)),
        Until::ListHas(s) => docs
            .completion_of(id)
            .is_some_and(|(l, _)| l.iter().any(|x| x == s)),
        Until::ListTop(s) => docs
            .completion_of(id)
            .is_some_and(|(l, _)| l.first().is_some_and(|x| x == s)),
        Until::Actions => docs.actions_of(id).is_some(),
        Until::Snippet => docs.snippet_of(id).is_some(),
        Until::RenameBox => docs
            .docs
            .iter()
            .find(|d| d.script == id)
            .and_then(|d| d.rename.as_ref())
            .is_some_and(|r| {
                r.scene.is_some()
                    || !matches!(
                        r.kind,
                        Some(crate::rename::Kind::Field { declared: true, .. })
                    )
            }),
        Until::Saved => !docs.is_dirty(id) && docs.rename_of(id).is_none(),
        Until::Problems(n) => docs.problems_of(id).len() >= n,
        Until::Reach(true) => {
            docs.reach_settled_of(id) == Some(true) && matches!(docs.link_of(id), (Some(_), true))
        }
        Until::Reach(false) => {
            docs.reach_settled_of(id) == Some(false) && docs.link_of(id) == (None, false)
        }
        Until::ProblemRows => docs.band_of(id).is_some_and(|b| !b.problems.is_empty()),
        Until::Has(s) => docs.text_of(id).is_some_and(|t| t.contains(s)),
        Until::Page { page, member } => {
            let tab = docs.help_tab_id(page);
            tab.is_some()
                && docs.active == tab
                && docs.help_probe(page).is_some_and(|p| {
                    p.title == ph2d_code_help::page(page).map_or("", |c| c.title.as_str())
                        && p.top_member == crate::help::member_id(page, member)
                })
        }
        Until::Script => docs.active == Some(id.0),
        Until::Stopped(s) => docs.docs.iter().any(|d| {
            d.script == id
                && d.is_stopped_here()
                && d.debug.card.as_ref().is_some_and(|c| c.title.contains(s))
        }),
        Until::Changed(key) => docs.docs.iter().any(|d| {
            d.script == id
                && d.debug.card.as_ref().is_some_and(|c| {
                    c.locals
                        .iter()
                        .chain(&c.this)
                        .any(|r| r.key == key && r.old.is_some())
                })
        }),
        Until::Peeking => docs.bench.peeking().is_some(),
        Until::CellIs(h, c, v) => docs.bancada.who.cell(h, c).is_some_and(|x| x.text == v),
        Until::Live => docs
            .docs
            .iter()
            .any(|d| d.script == id && !d.is_stopped_here() && !d.debug.live.ghosts.is_empty()),
        Until::Checked => docs.assist_diagnostics(id).1.is_some(),
        Until::Found => docs
            .active
            .and_then(page_of)
            .and_then(|p| docs.help_find_count_text(catalog()[p].id))
            .is_some_and(|t| t.starts_with("1 of")),
        Until::Plain { needle, ahead } => {
            let at = docs
                .text_of(id)
                .and_then(|t| t.find(needle))
                .map(|b| b + ahead);
            let look = at
                .and_then(|at| docs.glyph_center(id, at))
                .and_then(|(x, y)| docs.pointer_look(id.0, x, y));
            docs.link_of(id) == (None, false) && look.is_some_and(|l| l != PointerLook::Hand)
        }
    }
}

/// A aba activa da família (o script, se nenhuma das dela o é).
fn active(docs: &CodeDocs, id: ScriptId) -> u64 {
    docs.active
        .filter(|t| page_of(*t).is_some() || *t == id.0)
        .unwrap_or(id.0)
}

/// Carregar e largar o botão em `(x, y)` da aba `tab`, com `mods`, no relógio da família.
fn click(docs: &mut CodeDocs, tab: u64, (x, y): (f32, f32), mods: Modifiers) {
    let now_ns = docs.clock.0;
    let mut ts = TextSystem::without_system_fonts();
    for kind in [PointerKind::Down, PointerKind::Up] {
        let input = PointerInput {
            text: &mut ts,
            mods,
            now_ns,
            primary_held: kind == PointerKind::Down,
        };
        let _ = docs.pointer(tab, input, kind, PointerButton::Primary, x, y);
    }
}

/// Um gesto que entra num quadro; `false` = o alvo ainda não está pintado (tenta-se no seguinte).
fn act(docs: &mut CodeDocs, id: ScriptId, g: Gesture) -> bool {
    match g {
        Gesture::Go { needle, ahead } => {
            let Some(text) = docs.text_of(id) else {
                return true;
            };
            let Some(at) = text.find(needle).map(|b| b + ahead) else {
                return true;
            };
            let line_start = text[..at].rfind('\n').map_or(0, |b| b + 1);
            let line = text[..at].matches('\n').count();
            let col = text[line_start..at].chars().count();
            docs.go_to(id.0, line, col);
        }
        Gesture::Key(k, mods) => {
            let s = match k {
                Key::Char(ch) => Some(ch.to_string()),
                _ => None,
            };
            key(docs, id.0, k, mods, s.as_deref());
        }
        Gesture::DropObject { needle, name } => {
            let at = docs.text_of(id).and_then(|t| t.find(needle));
            let Some((x, y)) = at.and_then(|at| docs.glyph_center(id, at)) else {
                return false;
            };
            let names = [name.to_owned()];
            let at = ph2d_editor_core::doc_surface::DropAt {
                x,
                y,
                what: ph2d_editor_core::doc_surface::Dropping::Objects(&names),
                mods: NONE,
            };
            docs.drop(id.0, at);
        }
        Gesture::Point { needle, ahead } => {
            let at = docs
                .text_of(id)
                .and_then(|t| t.find(needle))
                .map(|b| b + ahead);
            let Some((x, y)) = at.and_then(|at| docs.glyph_center(id, at)) else {
                if let Some(at) = at {
                    docs.scroll_to(id, at);
                }
                return false;
            };
            let mut ts = TextSystem::without_system_fonts();
            let input = PointerInput {
                text: &mut ts,
                mods: NONE,
                now_ns: 1,
                primary_held: false,
            };
            let _ = docs.pointer_move(id.0, input, x, y);
        }
        Gesture::PointTabMenuRow(_) | Gesture::PointAim(_) => {
            let r = match g {
                Gesture::PointAim(aim) => docs.bench.target(aim),
                Gesture::PointTabMenuRow(k) => docs.bar_painted().rows.get(k).map(|r| r.1),
                _ => None,
            };
            let Some(r) = r else {
                return false;
            };
            let mut ts = TextSystem::without_system_fonts();
            let input = PointerInput {
                text: &mut ts,
                mods: NONE,
                now_ns: 1,
                primary_held: false,
            };
            let _ = docs.pointer_move(id.0, input, r.x + r.w * 0.5, r.y + r.h * 0.5);
        }
        Gesture::ClickProblem(_)
        | Gesture::ClickBandTab(_)
        | Gesture::ClickTabMenu(_)
        | Gesture::ClickGear
        | Gesture::ClickDebug(_)
        | Gesture::ClickTabMenuRow(_)
        | Gesture::ClickDebugRow(_)
        | Gesture::ClickAim(_)
        | Gesture::ClickMark { .. } => {
            let band = docs.band_of(id);
            let bar = docs.bar_painted();
            let doc = docs.docs.iter().find(|d| d.script == id);
            let r = match g {
                Gesture::ClickDebug(k) => {
                    doc.and_then(|d| d.debug.view.painted.buttons.get(k).map(|b| b.1))
                }
                Gesture::ClickTabMenuRow(k) => bar.rows.get(k).map(|r| r.1),
                Gesture::ClickDebugRow(key) => doc.and_then(|d| d.debug.view.chevron_of(key)),
                Gesture::ClickMark { needle } => doc.and_then(|d| d.mark_rect_of(needle)),
                Gesture::ClickProblem(k) => band.and_then(|b| b.problems.get(k).map(|p| p.rect)),
                Gesture::ClickBandTab(k) => band.and_then(|b| b.tabs.get(k).map(|t| t.1)),
                Gesture::ClickTabMenu(k) => bar.titles.get(k).map(|t| t.1),
                Gesture::ClickGear => bar.gear,
                Gesture::ClickAim(aim) => docs.bench.target(aim),
                _ => None,
            };
            let Some(r) = r else {
                return false;
            };
            let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
            let mut ts = TextSystem::without_system_fonts();
            for kind in [PointerKind::Down, PointerKind::Up] {
                let input = PointerInput {
                    text: &mut ts,
                    mods: NONE,
                    now_ns: 1,
                    primary_held: kind == PointerKind::Down,
                };
                let _ = docs.pointer(id.0, input, kind, PointerButton::Primary, x, y);
            }
        }
        Gesture::Click {
            needle,
            ahead,
            mods,
        } => {
            let at = docs
                .text_of(id)
                .and_then(|t| t.find(needle))
                .map(|b| b + ahead);
            let Some(p) = at.and_then(|at| docs.glyph_center(id, at)) else {
                if let Some(at) = at {
                    docs.scroll_to(id, at);
                }
                return false;
            };
            click(docs, id.0, p, mods);
        }
        Gesture::RightClickDebugHeader => {
            let doc = docs.docs.iter().find(|d| d.script == id);
            let header = doc.and_then(|d| d.debug.view.painted.table.as_ref().map(|t| t.0.header));
            let Some(r) = header else { return false };
            if !docs.context_open(id.0, r.x + r.w * 0.5, r.y + r.h * 0.5) {
                return false;
            }
        }
        Gesture::RightClick { needle, ahead } => {
            let at = docs
                .text_of(id)
                .and_then(|t| t.find(needle))
                .map(|b| b + ahead);
            if !at.is_some_and(|at| docs.context_open_at(id.0, at)) {
                return false;
            }
        }
        Gesture::TabKey(k, mods) => {
            let s = match k {
                Key::Char(ch) => Some(ch.to_string()),
                _ => None,
            };
            key(docs, active(docs, id), k, mods, s.as_deref());
        }
        Gesture::Link { page, to, member } => {
            let want = Anchor {
                page: to,
                member: Some(member),
            };
            let Some(tab) = docs.help_tab_id(page).filter(|t| docs.active == Some(*t)) else {
                return false;
            };
            let Some(r) = docs.help_probe(page).and_then(|p| {
                p.links
                    .iter()
                    .find(|(n, _, _)| docs.help_link_target(page, *n) == Some(want))
                    .map(|l| l.1)
            }) else {
                return false;
            };
            click(docs, tab, (r.x + r.w * 0.5, r.y + r.h * 0.5), NONE);
        }
        Gesture::Type(_)
        | Gesture::TabType(_)
        | Gesture::Wait(_)
        | Gesture::Ctrl(_)
        | Gesture::WindowUndo
        | Gesture::Frame => {}
    }
    true
}

fn key(docs: &mut CodeDocs, tab: u64, key: Key, mods: Modifiers, text: Option<&str>) {
    // A aba não molda texto na tecla (`CodeDocs::key` ignora o sistema de texto).
    let mut ts = TextSystem::without_system_fonts();
    let input = KeyInput {
        key,
        mods,
        pressed: true,
        text,
        paste: None,
    };
    let _ = docs.key(tab, &mut ts, input);
}

/// Cena 12: o ponto de parada que a cena traz, posto UMA vez quando a aba do `depura.luau` abre
/// (a cena mora na `ph2d-app-components`, que não chega ao `ScriptHost`).
fn preset_breakpoint(docs: &mut CodeDocs) {
    static DONE: AtomicBool = AtomicBool::new(false);
    let needle = match crate::smoke::armed_level() {
        Some(12) => ph2d_code_smoke::steps::DEPURA_BREAK,
        Some(14) => ph2d_code_smoke::steps::QUADROS_BREAK,
        _ => return,
    };
    if DONE.load(Ordering::Relaxed) {
        return;
    }
    let Some(d) = docs
        .docs
        .iter_mut()
        .find(|d| d.buffer.text().to_string().contains(needle))
    else {
        return;
    };
    let text = d.buffer.text().to_string();
    let Some(at) = text.find(needle) else { return };
    let line = text[..at].matches('\n').count();
    d.toggle_breakpoint(line);
    DONE.store(true, Ordering::Relaxed);
}
