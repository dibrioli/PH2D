//! A tabela viva: a de fábrica mais o delta do utilizador, com a lei do conflito.

use std::collections::BTreeSet;

use crate::factory::{FACTORY, SAME_MEANING, reserved};
use crate::{Chord, Context, PixelAction};

/// Porque uma tecla não se atribui.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conflict {
    /// Tecla do app (chave i18n do dono): recusada, nunca roubada.
    Reserved(&'static str),
    /// Já é de outra acção no mesmo contexto: quem chama pergunta, e então `assign_replacing`.
    Taken(PixelAction),
}

/// O que o `from_text` não aplicou.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextReport {
    /// Ids que esta versão não conhece (linha saltada).
    pub unknown_ids: Vec<String>,
    /// Linhas ou teclas que não se leram, ou teclas do app (saltadas).
    pub rejected: Vec<String>,
}

/// As teclas de cada acção.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keymap {
    binds: Vec<(PixelAction, Chord, Context)>,
}

impl Default for Keymap {
    fn default() -> Self {
        Self::factory()
    }
}

fn refusal(action: PixelAction, chord: Chord, ctx: Context) -> Option<Conflict> {
    if let Some(n) = reserved(chord) {
        return Some(Conflict::Reserved(n));
    }
    SAME_MEANING
        .iter()
        .find(|(c, a, _)| ctx == Context::Normal && *c == chord && *a != action)
        .map(|(_, _, n)| Conflict::Reserved(n))
}

impl Keymap {
    #[must_use]
    pub fn factory() -> Self {
        Self {
            binds: FACTORY.to_vec(),
        }
    }

    /// Todas as ligações, na ordem em que entraram.
    #[must_use]
    pub fn bindings(&self) -> &[(PixelAction, Chord, Context)] {
        &self.binds
    }

    /// A acção da tecla: a do contexto, senão a de `Normal` (que, a gesto a meio, só vale se a
    /// acção o permite).
    #[must_use]
    pub fn resolve(&self, chord: Chord, ctx: Context) -> Option<PixelAction> {
        let find = |c: Context| {
            self.binds
                .iter()
                .find(|(_, k, x)| *k == chord && *x == c)
                .map(|(a, _, _)| *a)
        };
        find(ctx).or_else(|| {
            if ctx == Context::Normal {
                return None;
            }
            find(Context::Normal).filter(|a| ctx != Context::Stroking || a.while_stroking())
        })
    }

    #[must_use]
    pub fn keys_of(&self, action: PixelAction) -> Vec<(Chord, Context)> {
        self.binds
            .iter()
            .filter(|(a, _, _)| *a == action)
            .map(|(_, k, c)| (*k, *c))
            .collect()
    }

    fn owner(&self, chord: Chord, ctx: Context) -> Option<PixelAction> {
        self.binds
            .iter()
            .find(|(_, k, c)| *k == chord && *c == ctx)
            .map(|(a, _, _)| *a)
    }

    /// Junta a tecla à acção. Tecla do app: recusada. De outra acção: `Taken`, nada muda.
    pub fn assign(
        &mut self,
        action: PixelAction,
        chord: Chord,
        ctx: Context,
    ) -> Result<(), Conflict> {
        if let Some(c) = refusal(action, chord, ctx) {
            return Err(c);
        }
        match self.owner(chord, ctx) {
            Some(a) if a == action => Ok(()),
            Some(a) => Err(Conflict::Taken(a)),
            None => {
                self.binds.push((action, chord, ctx));
                Ok(())
            }
        }
    }

    /// Como `assign`, mas tira a tecla a quem a tinha (depois de o utilizador dizer «Replace»).
    pub fn assign_replacing(
        &mut self,
        action: PixelAction,
        chord: Chord,
        ctx: Context,
    ) -> Result<(), Conflict> {
        if let Some(c) = refusal(action, chord, ctx) {
            return Err(c);
        }
        self.binds.retain(|(_, k, c)| !(*k == chord && *c == ctx));
        self.binds.push((action, chord, ctx));
        Ok(())
    }

    /// Tira a tecla da acção (em qualquer contexto).
    pub fn remove(&mut self, action: PixelAction, chord: Chord) {
        self.binds
            .retain(|(a, k, _)| !(*a == action && *k == chord));
    }

    /// Repõe só esta acção (as teclas de fábrica dela que outra tomou voltam a ser dela).
    pub fn reset(&mut self, action: PixelAction) {
        self.binds.retain(|(a, _, _)| *a != action);
        for &(a, k, c) in FACTORY.iter().filter(|(a, _, _)| *a == action) {
            self.binds.retain(|(_, k2, c2)| !(*k2 == k && *c2 == c));
            self.binds.push((a, k, c));
        }
    }

    pub fn reset_all(&mut self) {
        *self = Self::factory();
    }

    fn set_of(binds: &[(PixelAction, Chord, Context)], a: PixelAction) -> BTreeSet<String> {
        binds
            .iter()
            .filter(|(x, _, _)| *x == a)
            .map(|(_, k, c)| fmt_key(*k, *c))
            .collect()
    }

    /// O delta: uma linha por acção que difere da fábrica (`id = Ctrl+T, Esc @Selection` ou
    /// `id = -`), na ordem do diálogo.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for &a in PixelAction::ALL {
            let mine = Self::set_of(&self.binds, a);
            if mine == Self::set_of(FACTORY, a) {
                continue;
            }
            let keys: Vec<String> = self
                .keys_of(a)
                .into_iter()
                .map(|(k, c)| fmt_key(k, c))
                .collect();
            let rhs = if keys.is_empty() {
                "-".to_owned()
            } else {
                keys.join(", ")
            };
            out.push_str(a.id());
            out.push_str(" = ");
            out.push_str(&rhs);
            out.push('\n');
        }
        out
    }

    /// A fábrica com o delta por cima. Cada linha é a palavra do utilizador: a tecla que ela dá
    /// sai de quem a tinha. Ids desconhecidos e teclas do app saltam-se e reportam-se.
    #[must_use]
    pub fn from_text(text: &str) -> (Self, TextReport) {
        let mut map = Self::factory();
        let mut rep = TextReport::default();
        for line in text.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((id, rhs)) = line.split_once('=') else {
                rep.rejected.push(line.to_owned());
                continue;
            };
            let Some(action) = PixelAction::from_id(id.trim()) else {
                rep.unknown_ids.push(id.trim().to_owned());
                continue;
            };
            map.binds.retain(|(a, _, _)| *a != action);
            let rhs = rhs.trim();
            if rhs == "-" {
                continue;
            }
            for part in rhs.split(',') {
                match parse_key(part) {
                    Some((k, c)) => {
                        if map.assign_replacing(action, k, c).is_err() {
                            rep.rejected.push(part.trim().to_owned());
                        }
                    }
                    None => rep.rejected.push(part.trim().to_owned()),
                }
            }
        }
        (map, rep)
    }
}

fn fmt_key(k: Chord, c: Context) -> String {
    if c == Context::Normal {
        k.to_string()
    } else {
        format!("{k} @{}", c.id())
    }
}

fn parse_key(s: &str) -> Option<(Chord, Context)> {
    let s = s.trim();
    let (k, c) = match s.split_once('@') {
        Some((k, c)) => (k, Context::from_id(c.trim())?),
        None => (s, Context::Normal),
    };
    Some((k.trim().parse().ok()?, c))
}
