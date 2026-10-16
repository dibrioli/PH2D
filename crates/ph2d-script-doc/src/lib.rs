//! **Os scripts DO PROJETO** (ADR-0188 §Decisão 2) — `ScriptId → { nome único, texto }`.
//!
//! O `LuauScript` de um objecto aponta um [`ScriptId`]; vários objectos partilham o mesmo script. O
//! texto vive FORA do `ProjectState` (molde `motion`/`timeline`): o editor terá histórico próprio, e o
//! diff global por quadro não vê cada tecla.
//!
//! ⚠️ Um script que nenhum objecto aponta FICA — é conteúdo do artista; apagar é um gesto explícito.
//!
//! # O ficheiro (molde do `PH2DMACRO`, [`ph2d_macro::rec`])
//!
//! ```text
//! ficheiro  = MAGIC · major:u16le · minor:u16le · registo(Library)
//! Library   : 1 next (varint) · 2 Script* (repetido)
//! Script    : 1 id (varint ≥ 1) · 2 name (utf8) · 3 text (utf8)
//! ```
//! Etiqueta desconhecida salta-se; MAJOR diferente recusa-se ([`DocError::MajorMismatch`]).

use std::collections::BTreeMap;

use ph2d_macro::rec::{Record, as_str, as_varint, fields};
use serde::{Deserialize, Serialize};

pub use ph2d_macro::CodecError;

pub const MAGIC: &[u8; 11] = b"PH2DSCRIPTS";
pub const MAJOR: u16 = 1;
pub const MINOR: u16 = 0;

/// A identidade durável de um script no projeto. Monotónica; `0` = nenhum.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct ScriptId(pub u64);

impl ScriptId {
    pub const NONE: Self = Self(0);
}

/// Um script: o nome (único na biblioteca — é o nome do chunk nas mensagens do Luau) e o texto.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScriptDoc {
    pub name: String,
    pub text: String,
}

/// **A biblioteca de scripts do projeto.**
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptLibrary {
    next: u64,
    scripts: BTreeMap<ScriptId, ScriptDoc>,
}

impl Default for ScriptLibrary {
    fn default() -> Self {
        Self {
            next: 1,
            scripts: BTreeMap::new(),
        }
    }
}

/// Porque um blob de biblioteca não se lê.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocError {
    NotAScriptLibrary,
    MajorMismatch {
        found: u16,
        supported: u16,
    },
    Codec(CodecError),
    /// Um script sem id, com id `0`, repetido, ou com nome repetido.
    BadScript {
        id: u64,
    },
    /// ⛔ O contador de ids chegou ao fim da faixa (um id `u64::MAX`, ou o próprio contador lá):
    /// o próximo script novo receberia um id que já está vivo. Ficheiro estragado — recusado.
    IdsExhausted {
        id: u64,
    },
}

impl std::fmt::Display for DocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IdsExhausted { id } => {
                write!(f, "script id {id} is the last possible one, the file is damaged")
            }
            other => write!(f, "{other:?}"),
        }
    }
}

impl std::error::Error for DocError {}

impl From<CodecError> for DocError {
    fn from(e: CodecError) -> Self {
        Self::Codec(e)
    }
}

impl ScriptLibrary {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.scripts.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.scripts.is_empty()
    }

    #[must_use]
    pub fn get(&self, id: ScriptId) -> Option<&ScriptDoc> {
        self.scripts.get(&id)
    }

    #[must_use]
    pub fn by_name(&self, name: &str) -> Option<ScriptId> {
        self.scripts
            .iter()
            .find_map(|(id, d)| (d.name == name).then_some(*id))
    }

    pub fn iter(&self) -> impl Iterator<Item = (ScriptId, &ScriptDoc)> {
        self.scripts.iter().map(|(id, d)| (*id, d))
    }

    /// Acrescenta um script com um nome ÚNICO derivado de `name` («bob.luau» → «bob 2.luau»).
    pub fn insert(&mut self, name: &str, text: impl Into<String>) -> ScriptId {
        let id = ScriptId(self.next);
        self.next += 1;
        let name = self.unique_name(name, None);
        self.scripts.insert(
            id,
            ScriptDoc {
                name,
                text: text.into(),
            },
        );
        id
    }

    /// Troca o texto. `true` = mudou.
    pub fn set_text(&mut self, id: ScriptId, text: &str) -> bool {
        match self.scripts.get_mut(&id) {
            Some(d) if d.text != text => {
                text.clone_into(&mut d.text);
                true
            }
            _ => false,
        }
    }

    /// Renomeia para um nome único; devolve o nome final.
    pub fn rename(&mut self, id: ScriptId, name: &str) -> Option<String> {
        if !self.scripts.contains_key(&id) {
            return None;
        }
        let name = self.unique_name(name, Some(id));
        let d = self.scripts.get_mut(&id)?;
        d.name.clone_from(&name);
        Some(name)
    }

    pub fn remove(&mut self, id: ScriptId) -> Option<ScriptDoc> {
        self.scripts.remove(&id)
    }

    /// `wanted` se livre; senão «stem N.ext» com o menor `N ≥ 2` livre. Um «stem N» pedido conta a
    /// partir do mesmo stem («bob 2.luau» ocupado → «bob 3.luau»).
    fn unique_name(&self, wanted: &str, except: Option<ScriptId>) -> String {
        let wanted = wanted.trim();
        let wanted = if wanted.is_empty() {
            "script.luau"
        } else {
            wanted
        };
        let taken = |n: &str| {
            self.scripts
                .iter()
                .any(|(id, d)| Some(*id) != except && d.name == n)
        };
        if !taken(wanted) {
            return wanted.to_owned();
        }
        let (stem, ext) = match wanted.rfind('.') {
            Some(i) if i > 0 => wanted.split_at(i),
            _ => (wanted, ""),
        };
        let base = match stem.rsplit_once(' ') {
            Some((b, n)) if !b.is_empty() && n.parse::<u32>().is_ok() => b,
            _ => stem,
        };
        (2u64..)
            .map(|n| format!("{base} {n}{ext}"))
            .find(|c| !taken(c))
            .expect("há sempre um número livre")
    }

    /// O blob versionado.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(16);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&MAJOR.to_le_bytes());
        out.extend_from_slice(&MINOR.to_le_bytes());
        let mut lib = Record::new();
        lib.varint(1, self.next);
        for (id, d) in &self.scripts {
            let mut s = Record::new();
            s.varint(1, id.0).str(2, &d.name).str(3, &d.text);
            lib.bytes(2, &s.finish());
        }
        out.extend_from_slice(&lib.finish());
        out
    }

    /// Lê um blob. `[]` = biblioteca vazia (um projeto sem scripts).
    ///
    /// # Errors
    /// Cabeçalho alheio, MAJOR diferente, registo partido ou script mal formado.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DocError> {
        if bytes.is_empty() {
            return Ok(Self::default());
        }
        let rest = bytes
            .strip_prefix(MAGIC)
            .ok_or(DocError::NotAScriptLibrary)?;
        if rest.len() < 4 {
            return Err(CodecError::Truncated.into());
        }
        let major = u16::from_le_bytes([rest[0], rest[1]]);
        let body = &rest[4..];
        if major != MAJOR {
            return Err(DocError::MajorMismatch {
                found: major,
                supported: MAJOR,
            });
        }
        let mut lib = Self::default();
        for (tag, field) in fields(body)? {
            match tag {
                1 => lib.next = as_varint(field)?,
                2 => {
                    let (id, doc) = read_script(field)?;
                    if id == 0
                        || lib.scripts.contains_key(&ScriptId(id))
                        || lib.by_name(&doc.name).is_some()
                    {
                        return Err(DocError::BadScript { id });
                    }
                    lib.scripts.insert(ScriptId(id), doc);
                }
                _ => {}
            }
        }
        // O contador nunca volta a entregar um id vivo: sem um id livre depois do maior (nem do
        // contador gravado), o ficheiro está estragado.
        let max = lib.scripts.keys().next_back().map_or(0, |i| i.0);
        let after = max.checked_add(1).ok_or(DocError::IdsExhausted { id: max })?;
        lib.next = lib.next.max(after);
        if lib.next == u64::MAX {
            return Err(DocError::IdsExhausted { id: lib.next });
        }
        Ok(lib)
    }
}

fn read_script(bytes: &[u8]) -> Result<(u64, ScriptDoc), DocError> {
    let mut id = 0;
    let mut doc = ScriptDoc::default();
    for (tag, f) in fields(bytes)? {
        match tag {
            1 => id = as_varint(f)?,
            2 => as_str(f)?.clone_into(&mut doc.name),
            3 => as_str(f)?.clone_into(&mut doc.text),
            _ => {}
        }
    }
    Ok((id, doc))
}

#[cfg(test)]
mod tests;
