//! O modelo do depurador na aba (Onda D, plano 14): o quadro Stopped e o texto fantasma de uma
//! linha, a partir do que a VM do `ph2d-script` relata.

use ph2d_i18n::{tr, tr_with};
use ph2d_script::{DebugStop, LineLive, StopKind, ValueKind, Var};
use std::collections::BTreeMap;

/// Os caracteres de um valor ao vivo no fim da linha (legibilidade; o resto é «…»).
const GHOST_CHARS: usize = 60;

/// O que a aba pede ao jogo parado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ask {
    Continue,
    NextLine,
}

/// O estado do quadro: a cor e a palavra da faixa de cima.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardState {
    /// Os valores ao vivo (o jogo corre).
    #[default]
    Live,
    /// Um ponto de parada ou o fim de um Next line.
    Paused,
    /// Pause on Error.
    Error,
}

/// Um objecto no quadro: o nome e a cor dele na Bancada (`ColorToken::layer`), se a tem.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WhoTag {
    pub name: String,
    pub color: Option<usize>,
}

/// Os dois grupos do Stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VarGroup {
    Locals,
    This,
}

/// Uma linha de variável, já achatada: as filhas de uma tabela vêm a seguir com `depth` maior.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VarRow {
    /// O caminho (`dt`, `self.t`, `self.pos.x`): estável entre passos — guarda a dobra e o «mudou».
    pub key: String,
    pub name: String,
    pub depth: u8,
    pub value: String,
    pub kind: ValueKind,
    /// As entradas de uma tabela (o `{N fields}`).
    pub field_count: u32,
    /// Tem campos descidos (mostra ▸).
    pub expandable: bool,
    /// O valor do passo anterior, se mudou.
    pub old: Option<String>,
}

/// Uma chamada da pilha: a função e a linha (1-based).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackEntry {
    pub function: String,
    pub line: u32,
}

/// ⭐ **O quadro de baixo do código** (plano 14): parado (faixa com o estado, a linha, a função, o
/// objecto e os botões; Locals e self em árvore com o tipo; Call stack e Waiting here) ou de quem
/// são os valores ao vivo.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DebugCard {
    pub state: CardState,
    /// «Stopped at line 14 — on Circle» / «Live values: Circle» (a faixa compacta e a leitura).
    pub title: String,
    pub line: u32,
    /// A função do topo (vazia no topo do script).
    pub function: String,
    pub who: WhoTag,
    /// Quem espera a vez, pela ordem.
    pub waiting: Vec<WhoTag>,
    /// A mensagem do erro (Pause on Error).
    pub error: Option<String>,
    pub locals: Vec<VarRow>,
    pub this: Vec<VarRow>,
    pub stack: Vec<StackEntry>,
    /// O Next line (escondido num erro).
    pub can_step: bool,
}

impl DebugCard {
    /// Parado (não os valores ao vivo).
    #[must_use]
    pub fn stopped(&self) -> bool {
        self.state != CardState::Live
    }

    /// Os dois grupos, pela ordem do quadro.
    #[must_use]
    pub fn groups(&self) -> [(VarGroup, &[VarRow]); 2] {
        [
            (VarGroup::Locals, &self.locals),
            (VarGroup::This, &self.this),
        ]
    }

    /// O quadro em texto (as sondas e a árvore de acessibilidade): quem espera, cada variável
    /// (`chave = valor (was antigo)`), a pilha (com o rótulo `stack`) e quem espera, um a um.
    #[must_use]
    pub fn text_rows(&self, stack: &str) -> Vec<String> {
        let mut rows: Vec<String> = self.waiting_text().into_iter().collect();
        rows.extend(self.locals.iter().chain(&self.this).map(|r| {
            let v = crate::shown_value(r);
            match &r.old {
                Some(old) => format!("{} = {v} (was {old})", r.key),
                None => format!("{} = {v}", r.key),
            }
        }));
        let frames = self.stack.iter();
        rows.extend(frames.map(|s| format!("{stack}  {} {}", s.function, s.line)));
        rows.extend(self.waiting.iter().map(|w| format!("waiting {}", w.name)));
        rows
    }

    /// «2 more waiting» (o chip da faixa), se alguém espera.
    #[must_use]
    pub fn waiting_text(&self) -> Option<String> {
        (!self.waiting.is_empty())
            .then(|| tr_with("code.debug.waiting", &[("count", &self.waiting.len())]))
    }

    /// «update() · line 14» / «(top of script) · line 14».
    #[must_use]
    pub fn place(&self) -> String {
        let f = if self.function.is_empty() {
            tr("code.debug.top_level").to_owned()
        } else {
            tr_with("code.debug.call", &[("function", &self.function)])
        };
        tr_with(
            "code.debug.place",
            &[("line", &self.line), ("function", &f)],
        )
    }

    /// Os valores de cada caminho (o que o próximo passo compara).
    #[must_use]
    pub fn values(&self) -> BTreeMap<String, String> {
        self.locals
            .iter()
            .chain(&self.this)
            .map(|r| (r.key.clone(), r.value.clone()))
            .collect()
    }
}

/// O tipo escrito da coluna Type (o nome Luau).
#[must_use]
pub fn kind_name(kind: ValueKind) -> &'static str {
    tr(match kind {
        ValueKind::Nil => "code.debug.kind.nil",
        ValueKind::Boolean => "code.debug.kind.boolean",
        ValueKind::Number => "code.debug.kind.number",
        ValueKind::Vector => "code.debug.kind.vector",
        ValueKind::String => "code.debug.kind.string",
        ValueKind::Table => "code.debug.kind.table",
        ValueKind::Function => "code.debug.kind.function",
        ValueKind::Coroutine => "code.debug.kind.coroutine",
        ValueKind::Buffer => "code.debug.kind.buffer",
        ValueKind::Userdata => "code.debug.kind.userdata",
    })
}

/// O valor como a coluna Value o mostra: uma tabela é `{N fields}`.
#[must_use]
pub fn shown_value(r: &VarRow) -> String {
    if r.kind == ValueKind::Table {
        tr_with("code.debug.fields", &[("count", &r.field_count)])
    } else {
        r.value.clone()
    }
}

/// ⭐ **«Mudou desde o passo anterior»**: a memória dos valores do objecto parado. Enquanto o jogo
/// está parado os valores não mudam, logo valores diferentes do MESMO objecto e script = um passo
/// (Next line, ou o Continue que voltou a parar); outro objecto começa de novo.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StepMemory {
    owner: Option<(u64, u64)>,
    current: BTreeMap<String, String>,
    before: BTreeMap<String, String>,
}

impl StepMemory {
    /// Regista os valores de `owner` (bits do objecto, id do script) e devolve os do passo anterior.
    pub fn observe(
        &mut self,
        owner: (u64, u64),
        now: BTreeMap<String, String>,
    ) -> &BTreeMap<String, String> {
        if self.owner != Some(owner) {
            self.owner = Some(owner);
            self.before.clear();
            self.current = now;
        } else if self.current != now {
            self.before = std::mem::replace(&mut self.current, now);
        }
        &self.before
    }
}

/// As linhas de `vars` debaixo de `parent` (`None` nas locais, `self` no self), filhas a seguir.
fn flatten(vars: &[Var], parent: Option<&str>, depth: u8, out: &mut Vec<VarRow>) {
    for v in vars {
        let key = match parent {
            None => v.name.clone(),
            Some(p) if v.name.starts_with('[') => format!("{p}{}", v.name),
            Some(p) => format!("{p}.{}", v.name),
        };
        out.push(VarRow {
            key: key.clone(),
            name: v.name.clone(),
            depth,
            value: v.value.clone(),
            kind: v.kind,
            field_count: v.field_count,
            expandable: !v.fields.is_empty(),
            old: None,
        });
        flatten(&v.fields, Some(&key), depth.saturating_add(1), out);
    }
}

/// ⭐ O quadro «Stopped» de uma paragem: `who`/`waiting` com nome e cor, `before` = os valores do
/// passo anterior ([`StepMemory`]) — o que difere leva o antigo.
#[must_use]
pub fn stop_card(
    stop: &DebugStop,
    who: WhoTag,
    waiting: Vec<WhoTag>,
    before: &BTreeMap<String, String>,
) -> DebugCard {
    let line = stop.frame.as_ref().map_or(0, |f| f.line);
    let (state, title, error) = match &stop.why {
        StopKind::Error(m) => (
            CardState::Error,
            tr_with(
                "code.debug.stopped_error",
                &[("line", &line), ("who", &who.name)],
            ),
            Some(m.clone()),
        ),
        StopKind::Breakpoint | StopKind::Step => (
            CardState::Paused,
            tr_with("code.debug.stopped", &[("line", &line), ("who", &who.name)]),
            None,
        ),
    };
    let mut locals = Vec::new();
    let mut this = Vec::new();
    let mut stack = Vec::new();
    let mut function = String::new();
    if let Some(f) = &stop.frame {
        function.clone_from(&f.function);
        // Com os campos do `self` no grupo dele, o «self = table» das locais não diz nada.
        let shown: Vec<Var> = f
            .locals
            .iter()
            .filter(|v| f.this.is_empty() || v.name != "self")
            .cloned()
            .collect();
        flatten(&shown, None, 0, &mut locals);
        flatten(&f.this, Some("self"), 0, &mut this);
        stack = f
            .stack
            .iter()
            .map(|(name, n)| StackEntry {
                function: name.clone(),
                line: *n,
            })
            .collect();
    }
    for r in locals.iter_mut().chain(this.iter_mut()) {
        if let Some(old) = before.get(&r.key)
            && *old != r.value
            && r.kind != ValueKind::Table
        {
            r.old = Some(old.clone());
        }
    }
    DebugCard {
        state,
        title,
        line,
        function,
        who,
        waiting,
        error,
        locals,
        this,
        stack,
        can_step: !matches!(stop.why, StopKind::Error(_)),
    }
}

/// O quadro dos valores ao vivo de `who` (o jogo corre).
#[must_use]
pub fn live_card(who: WhoTag) -> DebugCard {
    DebugCard {
        state: CardState::Live,
        title: tr_with("code.debug.live", &[("who", &who.name)]),
        who,
        ..DebugCard::default()
    }
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
