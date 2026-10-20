//! ⭐ **A porta das FOTOS das cenas 3 a 5** (`PH2D_CODE_SMOKE=3|4|5 PH2D_CODE_SMOKE_PASSO=<n>`): o
//! roteiro do smoke conduzido pelas teclas REAIS da aba até ao fim do passo `n`, e pára — cada foto é
//! o passo que diz ser. A tela virtual das fotos ignora eventos sintéticos (`fotografa_cena.sh`), logo
//! o gesto entra pelas portas do artista (`DocumentSurface::key`, `pointer_move`, `pointer`), o cursor
//! anda pelo `go_to` da paleta `:` e o Ctrl carregado vai para o `WidgetStore` como o da janela. Os
//! testes das cenas (na shell, `shells/desktop/tests/it/a_cena_{3,4,5}_do_editor_conduzida.rs`)
//! conduzem os MESMOS roteiros e conferem as agulhas no texto da cena. Os roteiros moram em
//! `smoke_steps.rs`. As
//! teclas `Key`/`Type` vão para o script; `TabKey`/`TabType` para a aba ACTIVA da família (a página
//! de ajuda, quando é ela).

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

/// O passo até onde a cena 3 se conduz sozinha (0 = nada, o estado em que o dono a recebe).
pub const STEP_ENV: &str = "PH2D_CODE_SMOKE_PASSO";

/// `cena` = acabado o roteiro, a aba `Scene` passa à frente (o clique do dono na aba): a foto das
/// formas, que a aba do script tapa.
pub const VIEW_ENV: &str = "PH2D_CODE_SMOKE_FOTO";

/// Quanto um passo espera pelo editor (o assistente, a pausa do aviso) antes de desistir e dizê-lo.
const PATIENCE: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gesture {
    /// O cursor `ahead` bytes depois do início da 1.ª `needle` do texto (o `:` da paleta).
    Go {
        needle: &'static str,
        ahead: usize,
    },
    Key(Key, Modifiers),
    /// Uma letra por quadro, como a mão: a gramática assenta entre as teclas (num quadro só, a
    /// letra depois de um Enter num comentário ainda se lia como comentário e a lista não abria).
    Type(&'static str),
    Wait(Until),
    /// O rato (sem botão) sobre o carácter `ahead` bytes depois da 1.ª `needle`.
    Point {
        needle: &'static str,
        ahead: usize,
    },
    /// O Ctrl carregado (`true`) ou largado, como a janela o diz ao `WidgetStore`.
    Ctrl(bool),
    /// Um clique na linha `k` da lista de problemas pintada.
    ClickProblem(usize),
    /// O resto do passo entra no quadro seguinte (cada Tab vê-se).
    Frame,
    /// Um clique com `mods` sobre o carácter `ahead` bytes depois da 1.ª `needle` (o Ctrl+clique).
    Click {
        needle: &'static str,
        ahead: usize,
        mods: Modifiers,
    },
    /// Uma tecla na aba ACTIVA da família (a página de ajuda, ou o script).
    TabKey(Key, Modifiers),
    /// Uma letra por quadro na aba activa (o campo da busca da página).
    TabType(&'static str),
    /// Um clique no link pintado da página `page` que leva a `to` no membro `member`.
    Link {
        page: &'static str,
        to: &'static str,
        member: &'static str,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Until {
    /// Um problema cuja mensagem contém o texto.
    Problem(&'static str),
    /// A lista aberta com este rótulo nela.
    ListHas(&'static str),
    /// A lista aberta com este rótulo em PRIMEIRO.
    ListTop(&'static str),
    /// As acções do Alt+Enter abertas.
    Actions,
    /// Um esqueleto com buracos activo.
    Snippet,
    /// A caixinha do F2 aberta e, numa `ph2d.property`, com os objectos lidos (a ajuda conta-os).
    RenameBox,
    /// A aba sem ● e sem caixinha.
    Saved,
    /// Pelo menos `n` problemas na calha.
    Problems(usize),
    /// O assistente respondeu sobre o nome sob o rato: `true` = o Ctrl+clique alcança-o e o
    /// sublinhado foi pintado; `false` = não alcança, e nada se pintou.
    Reach(bool),
    /// A lista de problemas pintada com linhas.
    ProblemRows,
    /// O texto contém isto.
    Has(&'static str),
    /// A página `page` é a aba activa e pintou `member` no topo.
    Page {
        page: &'static str,
        member: &'static str,
    },
    /// O script é a aba activa.
    Script,
    /// O assistente já verificou o script (publicou os diagnósticos dele).
    Checked,
    /// A busca da página activa contou (o «1 of N»).
    Found,
    /// Nada sublinhado e o ponteiro sobre o carácter `ahead` depois de `needle` não é a mão.
    Plain { needle: &'static str, ahead: usize },
}

#[path = "smoke_steps.rs"]
mod roteiros;
use roteiros::NONE;
pub use roteiros::{STEPS, STEPS_4, STEPS_5};

/// O roteiro da cena `level` (`None` = a cena não tem fotos conduzidas).
#[must_use]
pub fn steps(level: u32) -> Option<&'static [&'static [Gesture]]> {
    match level {
        3 => Some(STEPS),
        4 => Some(STEPS_4),
        5 => Some(STEPS_5),
        _ => None,
    }
}

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
        Gesture::Point { needle, ahead } => {
            let at = docs
                .text_of(id)
                .and_then(|t| t.find(needle))
                .map(|b| b + ahead);
            let Some((x, y)) = at.and_then(|at| docs.glyph_center(id, at)) else {
                return false;
            };
            let mut ts = TextSystem::without_system_fonts();
            let input = PointerInput {
                text: &mut ts,
                mods: NONE,
                now_ns: 1,
            };
            let _ = docs.pointer_move(id.0, input, x, y);
        }
        Gesture::ClickProblem(k) => {
            let Some(r) = docs
                .band_of(id)
                .and_then(|b| b.problems.get(k).map(|p| p.rect))
            else {
                return false;
            };
            let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
            let mut ts = TextSystem::without_system_fonts();
            for kind in [PointerKind::Down, PointerKind::Up] {
                let input = PointerInput {
                    text: &mut ts,
                    mods: NONE,
                    now_ns: 1,
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
                return false;
            };
            click(docs, id.0, p, mods);
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
