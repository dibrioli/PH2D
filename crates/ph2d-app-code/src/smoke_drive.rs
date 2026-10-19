//! ⭐ **A porta das FOTOS da cena 3** (`PH2D_CODE_SMOKE=3 PH2D_CODE_SMOKE_PASSO=<n>`): o roteiro do
//! smoke conduzido pelas teclas REAIS da aba até ao fim do passo `n`, e pára — cada foto é o passo que
//! diz ser. A tela virtual das fotos ignora eventos sintéticos (`fotografa_cena.sh`), logo o gesto
//! entra pela porta da tecla do artista (`DocumentSurface::key`) e o cursor anda pelo `go_to` da
//! paleta `:`. O teste da cena (`ph2d-app-components`, `a_cena_3_conduzida`) conduz o MESMO roteiro e
//! confere as agulhas no texto da cena.

use crate::CodeDocs;
use ph2d_editor_core::doc_surface::{DocumentSurface, Key, KeyInput};
use ph2d_host::Modifiers;
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
}

const NONE: Modifiers = Modifiers {
    shift: false,
    ctrl: false,
    alt: false,
    meta: false,
};
const ALT: Modifiers = Modifiers { alt: true, ..NONE };
const CTRL: Modifiers = Modifiers { ctrl: true, ..NONE };
const SAVE: Gesture = Gesture::Key(Key::Char('s'), CTRL);

/// ⭐ O roteiro: `STEPS[k]` é o passo `k + 1` do smoke do dono, gesto a gesto.
pub const STEPS: &[&[Gesture]] = &[
    // 1 — o cursor no «ampiltude» sublinhado, Alt+Enter.
    &[
        Gesture::Wait(Until::Problem("ampiltude")),
        Gesture::Go {
            needle: "self.ampiltude",
            ahead: 8,
        },
        Gesture::Key(Key::Enter, ALT),
        Gesture::Wait(Until::Actions),
    ],
    // 2 — Enter aplica, Ctrl+S: as formas mexem-se.
    &[
        Gesture::Key(Key::Enter, NONE),
        SAVE,
        Gesture::Wait(Until::Saved),
    ],
    // 3 — uma linha nova depois do «2)», `ph2d.`: a lista abre com a API.
    &[
        Gesture::Go {
            needle: "-- 2) escreva aqui",
            ahead: 0,
        },
        Gesture::Key(Key::End, NONE),
        Gesture::Key(Key::Enter, NONE),
        Gesture::Type("ph2d."),
        Gesture::Wait(Until::ListHas("set")),
    ],
    // 4 — `se` deixa `set` em cima; Enter escolhe: o esqueleto com buracos.
    &[
        Gesture::Type("se"),
        Gesture::Wait(Until::ListTop("set")),
        Gesture::Key(Key::Enter, NONE),
        Gesture::Wait(Until::Snippet),
    ],
    // 5 — os três buracos, Tab entre eles, Ctrl+S: as formas giram.
    &[
        Gesture::Type("self.id"),
        Gesture::Key(Key::Tab, NONE),
        Gesture::Type("rotation"),
        Gesture::Key(Key::Tab, NONE),
        Gesture::Type("self.t"),
        Gesture::Key(Key::Tab, NONE),
        SAVE,
        Gesture::Wait(Until::Saved),
    ],
    // 6 — F2 sobre o nome da propriedade: a caixinha diz quantos objectos guardam um valor.
    &[
        Gesture::Go {
            needle: "\"amplitude\"",
            ahead: 3,
        },
        Gesture::Key(Key::F(2), NONE),
        Gesture::Wait(Until::RenameBox),
    ],
    // 7 — o nome novo, Enter: o script e o valor da estrela mudam juntos.
    &[
        Gesture::Type("height"),
        Gesture::Key(Key::Enter, NONE),
        Gesture::Wait(Until::Saved),
    ],
];

/// O passo pedido pelo ambiente (lido uma vez); só com a cena 3.
#[must_use]
pub fn armed_step() -> Option<usize> {
    static STEP: OnceLock<Option<usize>> = OnceLock::new();
    *STEP.get_or_init(|| {
        let n = std::env::var(STEP_ENV).ok()?.trim().parse().ok()?;
        (crate::smoke::armed_level() == Some(3) && n <= STEPS.len()).then_some(n)
    })
}

/// O condutor: onde vai, e desde quando espera.
#[derive(Debug)]
pub struct Conductor {
    target: usize,
    step: usize,
    gesture: usize,
    /// Quantas letras do `Type` em curso já entraram.
    typed: usize,
    since: Option<Instant>,
    failed: bool,
}

impl Conductor {
    /// Conduz até ao fim do passo `target` (1-based; 0 = nada).
    #[must_use]
    pub fn new(target: usize) -> Self {
        Self {
            target: target.min(STEPS.len()),
            step: 0,
            gesture: 0,
            typed: 0,
            since: None,
            failed: false,
        }
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
/// `true` UMA vez, quando o roteiro acabou e a foto pede a cena ([`VIEW_ENV`]).
pub(crate) fn frame(docs: &mut CodeDocs) -> bool {
    static DRIVE: OnceLock<Mutex<Conductor>> = OnceLock::new();
    static SHOWN: AtomicBool = AtomicBool::new(false);
    let Some(n) = armed_step() else { return false };
    let mut c = DRIVE
        .get_or_init(|| Mutex::new(Conductor::new(n)))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    drive(docs, &mut c);
    c.finished()
        && !c.failed()
        && std::env::var(VIEW_ENV).is_ok_and(|v| v.trim() == "cena")
        && !SHOWN.swap(true, Ordering::Relaxed)
}

/// Um quadro do condutor: os gestos até à próxima espera ou ao fim do passo entram já, na aba do
/// 1.º script aberto.
pub fn drive(docs: &mut CodeDocs, c: &mut Conductor) {
    let Some(&id) = docs.open_scripts().first() else {
        return;
    };
    while !c.finished() {
        let gestures = STEPS[c.step];
        let Some(&g) = gestures.get(c.gesture) else {
            c.step += 1;
            c.gesture = 0;
            println!("[code-smoke] passo {} feito", c.step);
            // O passo acaba com o quadro: o seguinte começa no próximo (o estado de cada um vê-se).
            return;
        };
        if let Gesture::Wait(until) = g {
            if !holds(docs, id, until) {
                let since = *c.since.get_or_insert_with(Instant::now);
                if since.elapsed() > PATIENCE {
                    eprintln!(
                        "[code-smoke] passo {}: {until:?} não chegou em {PATIENCE:?} — a cena pára aqui",
                        c.step + 1
                    );
                    c.failed = true;
                }
                return;
            }
            c.since = None;
        } else if let Gesture::Type(s) = g {
            if let Some(ch) = s.chars().nth(c.typed) {
                key(docs, id, Key::Other, NONE, Some(&ch.to_string()));
                c.typed += 1;
                return;
            }
            c.typed = 0;
        } else {
            act(docs, id, g);
        }
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
    }
}

fn act(docs: &mut CodeDocs, id: ScriptId, g: Gesture) {
    match g {
        Gesture::Go { needle, ahead } => {
            let Some(text) = docs.text_of(id) else { return };
            let Some(at) = text.find(needle).map(|b| b + ahead) else {
                return;
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
            key(docs, id, k, mods, s.as_deref());
        }
        Gesture::Type(_) | Gesture::Wait(_) => {}
    }
}

fn key(docs: &mut CodeDocs, id: ScriptId, key: Key, mods: Modifiers, text: Option<&str>) {
    // A aba não molda texto na tecla (`CodeDocs::key` ignora o sistema de texto).
    let mut ts = TextSystem::without_system_fonts();
    let input = KeyInput {
        key,
        mods,
        pressed: true,
        text,
        paste: None,
    };
    let _ = docs.key(id.0, &mut ts, input);
}
