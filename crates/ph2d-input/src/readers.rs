//! **QUEM LÊ CADA ACÇÃO** — e quando uma tecla em duas acções é de propósito (plano InputMap §2).

use std::collections::{BTreeMap, BTreeSet};

use crate::action::{ActionId, Binding};
use crate::map::{
    InputMap, PLAYER_DASH, PLAYER_DOWN, PLAYER_GRAB, PLAYER_JUMP, PLAYER_MOVE_DOWN,
    PLAYER_MOVE_LEFT, PLAYER_MOVE_RIGHT, PLAYER_MOVE_UP,
};

/// Os nomes que o Platform Player lê (`ph2d-physics-ecs/src/player_input.rs`).
pub const PLATFORM_PLAYER_READS: [&str; 6] = [
    PLAYER_MOVE_LEFT,
    PLAYER_MOVE_RIGHT,
    PLAYER_JUMP,
    PLAYER_DOWN,
    PLAYER_DASH,
    PLAYER_GRAB,
];

/// Os nomes que o Top-Down Player lê.
pub const TOP_DOWN_PLAYER_READS: [&str; 4] = [
    PLAYER_MOVE_LEFT,
    PLAYER_MOVE_RIGHT,
    PLAYER_MOVE_UP,
    PLAYER_MOVE_DOWN,
];

/// Quem lê uma acção.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Reader {
    PlatformPlayer,
    TopDownPlayer,
    /// Um Trigger da cena com o nome da acção na sua Action.
    Trigger { object: String },
    /// Um script Luau do projecto que chama `ph2d.action_*`.
    Script { name: String },
}

impl Reader {
    /// A chave i18n do nome dos leitores estáticos; os outros mostram o nome do objecto/script.
    #[must_use]
    pub const fn label_key(&self) -> Option<&'static str> {
        match self {
            Self::PlatformPlayer => Some("input_map_tab.reader.platform_player"),
            Self::TopDownPlayer => Some("input_map_tab.reader.top_down_player"),
            Self::Trigger { .. } | Self::Script { .. } => None,
        }
    }

    /// Um mover: um objecto tem um só, então dois movers nunca correm juntos.
    #[must_use]
    pub const fn is_mover(&self) -> bool {
        matches!(self, Self::PlatformPlayer | Self::TopDownPlayer)
    }
}

/// O que a família junta da cena e do projecto.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReaderFacts {
    /// `(objecto, acção)` de cada Trigger.
    pub triggers: Vec<(String, String)>,
    /// `(nome do script, fonte)` de cada script Luau.
    pub scripts: Vec<(String, String)>,
}

/// Os leitores de cada nome de acção. Os estáticos entram sempre (são contrato do código).
#[must_use]
pub fn readers(facts: &ReaderFacts) -> BTreeMap<String, Vec<Reader>> {
    let mut out: BTreeMap<String, Vec<Reader>> = BTreeMap::new();
    let mut add = |name: &str, r: Reader| {
        let list = out.entry(name.to_owned()).or_default();
        if !list.contains(&r) {
            list.push(r);
        }
    };
    for n in PLATFORM_PLAYER_READS {
        add(n, Reader::PlatformPlayer);
    }
    for n in TOP_DOWN_PLAYER_READS {
        add(n, Reader::TopDownPlayer);
    }
    for (object, action) in &facts.triggers {
        add(
            action,
            Reader::Trigger {
                object: object.clone(),
            },
        );
    }
    for (name, src) in &facts.scripts {
        for action in action_names_in_script(src) {
            add(&action, Reader::Script { name: name.clone() });
        }
    }
    out
}

/// As funções Luau que lêem uma acção pelo nome.
const ACTION_FNS: [&str; 4] = [
    "ph2d.action_down",
    "ph2d.action_pressed",
    "ph2d.action_released",
    "ph2d.action_strength",
];

/// Os nomes de acção que um script lê: `ph2d.action_*("x")`, aspas simples ou duplas, fora de
/// comentários `--`.
#[must_use]
pub fn action_names_in_script(src: &str) -> BTreeSet<String> {
    let code = without_comments(src);
    let mut out = BTreeSet::new();
    for f in ACTION_FNS {
        let mut from = 0;
        while let Some(at) = code[from..].find(f) {
            let start = from + at;
            from = start + f.len();
            let before = code[..start].chars().next_back();
            if before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.') {
                continue;
            }
            if let Some(name) = first_string_arg(&code[from..]) {
                out.insert(name);
            }
        }
    }
    out
}

/// `("x")` / `('x')` / `"x"` logo a seguir ao nome da função.
fn first_string_arg(rest: &str) -> Option<String> {
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('(').map_or(rest, str::trim_start);
    let q = rest.chars().next().filter(|c| *c == '"' || *c == '\'')?;
    let body = &rest[1..];
    let end = body.find([q, '\n'])?;
    (body[end..].starts_with(q) && end > 0).then(|| body[..end].to_owned())
}

/// O fonte com cada comentário (`--` até ao fim da linha, ou `--[[ … ]]`) trocado por espaços;
/// as strings ficam.
fn without_comments(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    let mut quote: Option<u8> = None;
    while i < b.len() {
        let c = b[i];
        if let Some(q) = quote {
            if c == b'\\' && i + 1 < b.len() {
                out.push('\\');
                i += 1;
                let ch_len = src[i..].chars().next().map_or(1, char::len_utf8);
                out.push_str(&src[i..i + ch_len]);
                i += ch_len;
                continue;
            }
            if c == q || c == b'\n' {
                quote = None;
            }
        } else if c == b'"' || c == b'\'' {
            quote = Some(c);
        } else if c == b'-' && b.get(i + 1) == Some(&b'-') {
            let end = comment_end(src, i + 2);
            out.extend(src[i..end].chars().map(|ch| if ch == '\n' { '\n' } else { ' ' }));
            i = end;
            continue;
        }
        let ch_len = src[i..].chars().next().map_or(1, char::len_utf8);
        out.push_str(&src[i..i + ch_len]);
        i += ch_len;
    }
    out
}

/// Onde acaba um comentário que começa em `at` (logo depois do `--`).
fn comment_end(src: &str, at: usize) -> usize {
    let rest = &src[at..];
    if let Some(open) = rest.strip_prefix('[') {
        let eqs = open.bytes().take_while(|c| *c == b'=').count();
        if open[eqs..].starts_with('[') {
            let close = format!("]{}]", "=".repeat(eqs));
            return rest.find(&close).map_or(src.len(), |e| at + e + close.len());
        }
    }
    rest.find('\n').map_or(src.len(), |e| at + e)
}

/// Uma ligação presente em mais de uma acção.
#[derive(Clone, Debug, PartialEq)]
pub struct SharedBinding {
    pub binding: Binding,
    /// As acções que a partilham, pela ordem do mapa.
    pub actions: Vec<ActionId>,
    /// De propósito: todo par tem leitores, só de mover, e conjuntos disjuntos.
    pub on_purpose: bool,
}

/// Toda ligação presente em mais de uma acção, pela ordem das ligações.
#[must_use]
pub fn shared_bindings(
    map: &InputMap,
    readers: &BTreeMap<String, Vec<Reader>>,
) -> Vec<SharedBinding> {
    let mut by_binding: BTreeMap<Binding, Vec<ActionId>> = BTreeMap::new();
    for a in map.actions() {
        for b in &a.bindings {
            let ids = by_binding.entry(*b).or_default();
            if !ids.contains(&a.id) {
                ids.push(a.id);
            }
        }
    }
    let none: Vec<Reader> = Vec::new();
    let readers_of = |id: ActionId| {
        map.get(id)
            .and_then(|a| readers.get(&a.name))
            .unwrap_or(&none)
    };
    by_binding
        .into_iter()
        .filter(|(_, ids)| ids.len() > 1)
        .map(|(binding, actions)| {
            let on_purpose = actions.iter().enumerate().all(|(i, x)| {
                actions[i + 1..]
                    .iter()
                    .all(|y| apart(readers_of(*x), readers_of(*y)))
            });
            SharedBinding {
                binding,
                actions,
                on_purpose,
            }
        })
        .collect()
}

/// Os dois têm leitores, só de mover, e nenhum em comum.
fn apart(a: &[Reader], b: &[Reader]) -> bool {
    !a.is_empty()
        && !b.is_empty()
        && a.iter().chain(b).all(Reader::is_mover)
        && a.iter().all(|r| !b.contains(r))
}

#[cfg(test)]
#[path = "readers_tests.rs"]
mod tests;
