//! O modelo do depurador na aba (Onda D): o cartão Stopped e o texto fantasma de uma linha, a
//! partir do que a VM do `ph2d-script` relata. A família (`ph2d-app-code::debug`) pinta-os.

use ph2d_i18n::{tr, tr_with};
use ph2d_script::{DebugStop, LineLive, StopKind};

/// As variáveis que o cartão mostra antes do «+N more» (a altura do cartão; o resto rola na pilha
/// do Luau, não no ecrã).
const VARS_SHOWN: usize = 8;
/// Os caracteres de um valor ao vivo no fim da linha (legibilidade; o resto é «…»).
const GHOST_CHARS: usize = 60;

/// O que a aba pede ao jogo parado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ask {
    Continue,
    NextLine,
}

/// O cartão de baixo do código: parado (cabeçalho com a fila e os botões, variáveis em grelha,
/// pilha) ou de quem são os valores ao vivo.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DebugCard {
    pub title: String,
    /// «2 more waiting», o chip do cabeçalho.
    pub waiting: Option<String>,
    /// A mensagem do erro (Pause on Error).
    pub error: Option<String>,
    /// `(nome, valor)`; o «+N more» é um nome sem valor.
    pub vars: Vec<(String, String)>,
    /// A pilha já escrita (`update · 14`).
    pub stack: Option<String>,
    /// `false` = só o rótulo dos valores ao vivo.
    pub stopped: bool,
    /// O Next line (escondido num erro).
    pub can_step: bool,
}

/// «x = 412, step = 1  ×3  printed 2×» de uma linha.
pub fn ghost_text(live: &LineLive) -> String {
    let mut parts: Vec<String> = Vec::new();
    let vals: Vec<String> = live
        .values
        .iter()
        .map(|v| format!("{} = {}", v.name, v.value))
        .collect();
    if !vals.is_empty() {
        parts.push(vals.join(", "));
    }
    if live.hits > 1 {
        parts.push(tr_with("code.debug.hits", &[("count", &live.hits)]));
    }
    if live.prints > 0 {
        parts.push(tr_with("code.debug.printed", &[("count", &live.prints)]));
    }
    let s = parts.join("  ");
    if s.chars().count() > GHOST_CHARS {
        let mut cut: String = s.chars().take(GHOST_CHARS).collect();
        cut.push_str(tr("code.fold.ellipsis"));
        cut
    } else {
        s
    }
}

/// O cartão «Stopped» de uma paragem (`who` = o nome do objecto).
pub fn stop_card(stop: &DebugStop, who: &str) -> DebugCard {
    let line = stop.frame.as_ref().map_or(0, |f| f.line);
    let (title, error) = match &stop.why {
        StopKind::Error(m) => (
            tr_with(
                "code.debug.stopped_error",
                &[("line", &line), ("who", &who)],
            ),
            Some(m.clone()),
        ),
        StopKind::Breakpoint | StopKind::Step => (
            tr_with("code.debug.stopped", &[("line", &line), ("who", &who)]),
            None,
        ),
    };
    let waiting =
        (stop.queued > 0).then(|| tr_with("code.debug.waiting", &[("count", &stop.queued)]));
    let mut vars = Vec::new();
    let mut stack = None;
    if let Some(f) = &stop.frame {
        // Com os campos do `self` à mostra, o «self = table» não diz nada.
        let all: Vec<(String, String)> = f
            .locals
            .iter()
            .filter(|v| f.this.is_empty() || v.name != "self")
            .map(|v| (v.name.clone(), v.value.clone()))
            .chain(
                f.this
                    .iter()
                    .map(|v| (format!("self.{}", v.name), v.value.clone())),
            )
            .collect();
        let more = all.len().saturating_sub(VARS_SHOWN);
        vars.extend(all.into_iter().take(VARS_SHOWN));
        if more > 0 {
            vars.push((
                tr_with("code.debug.more_vars", &[("count", &more)]),
                String::new(),
            ));
        }
        if !f.stack.is_empty() {
            let calls: Vec<String> = f
                .stack
                .iter()
                .map(|(name, n)| {
                    let name = if name.is_empty() {
                        tr("code.debug.top_level")
                    } else {
                        name.as_str()
                    };
                    tr_with("code.debug.frame", &[("function", &name), ("line", n)])
                })
                .collect();
            stack = Some(calls.join(tr("code.debug.stack_sep")));
        }
    }
    DebugCard {
        title,
        waiting,
        error,
        vars,
        stack,
        stopped: true,
        can_step: !matches!(stop.why, StopKind::Error(_)),
    }
}
