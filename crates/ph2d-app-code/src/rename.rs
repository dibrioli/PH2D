//! ⭐ **I10 — renomear (F2)**: a caixinha por baixo do nome, com ele escrito e todo seleccionado (o
//! VS Code 1.142.0, `ferramentas/medicoes_onda2/I10_rename_e_undo.md` §2); Enter aplica num passo de
//! undo, Esc cancela. Os nomes do código vão ao `luau-lsp`; os campos do `self` e as `ph2d.property`
//! são nossos — e uma propriedade declarada leva consigo o valor que cada objecto guarda no Inspector,
//! num passo ligado à fila global ([`crate::link`]).
//!
//! A validação é NOSSA, antes de perguntar: o servidor aceita `end` e nomes que já existem, e MORRE
//! sobre um membro (§1 da medição); o VS Code aceita `1x` e `class` e parte o código (§2).

use crate::doc::CodeDoc;
use crate::edit::{FieldKey, field_key};
use ph2d_code_buffer::Text;
use ph2d_code_buffer::{Buffer, Change, ChangeSet, EditKind, Range as Cursor, Selection};
use ph2d_code_intel::{Answer, Ticket};
use ph2d_code_syntax::{LUAU_KEYWORDS, NameAt, Syntax, is_identifier};
use ph2d_editor_core::undo_link::UndoLink;
use ph2d_i18n::tr;
use ph2d_script::ScriptId;
use ph2d_script::api_table::HOOKS;
use std::collections::BTreeSet;
use std::ops::Range;

pub(crate) struct Renaming {
    /// O nome sob o cursor (bytes do buffer): a caixinha alinha-se por ele.
    pub word: Range<usize>,
    pub old: String,
    /// `None` = não há o que renomear aqui: a caixinha só diz porquê ([`Self::why`]).
    pub kind: Option<Kind>,
    /// O nome novo (uma linha).
    pub field: Buffer,
    pub why: Option<Why>,
    /// Enter num nome do código: o nome a mandar ao servidor e o pedido em voo.
    pub sent: Option<(String, Option<Ticket>)>,
    /// Os objectos do script (a família lê-os do mundo a cada quadro, [`crate::link`]).
    pub scene: Option<Scene>,
}

pub(crate) enum Kind {
    /// Um nome do código: o servidor segue-o.
    Name,
    /// Um campo do `self` (com a declaração, se é uma `ph2d.property`): todos os sítios dele.
    Field {
        sites: Vec<Range<usize>>,
        declared: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Why {
    Nothing,
    Member,
    Hook,
    Escapes,
    Invalid,
    Reserved,
    InUse,
    Taken,
    Offline,
    Server(String),
}

impl Why {
    pub(crate) fn text(&self, name: &str) -> String {
        let key = match self {
            Self::Nothing => "code.rename.nothing",
            Self::Member => "code.rename.member",
            Self::Hook => "code.rename.hook",
            Self::Escapes => "code.rename.escapes",
            Self::Invalid => "code.rename.invalid",
            Self::Reserved => "code.rename.reserved",
            Self::InUse => "code.rename.in_use",
            Self::Taken => "code.rename.taken",
            Self::Offline => "code.rename.offline",
            Self::Server(s) => return s.clone(),
        };
        tr(key).replace("{name}", name)
    }
}

/// Os objectos que correm o script: quantos guardam um valor da propriedade a renomear, e os nomes
/// que algum já guarda (um valor órfão com o nome novo seria apagado pela mudança).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Scene {
    pub objects: usize,
    pub taken: BTreeSet<String>,
}

/// Um rename de propriedade que mexe também na cena: a família aplica-o no quadro, com o mundo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SceneRename {
    pub script: ScriptId,
    pub old: String,
    pub new: String,
    pub link: UndoLink,
}

pub(crate) fn hook_names() -> Vec<&'static str> {
    HOOKS.iter().map(|h| h.ident).collect()
}

/// Todos os sítios do campo `name` do `self` e da `ph2d.property` dele; `None` = o `self` foge (ou o
/// texto tem um erro de sintaxe): não dá para os ver todos.
pub(crate) fn field_sites(
    syntax: &Syntax,
    text: &Text,
    name: &str,
) -> Option<(Vec<Range<usize>>, bool)> {
    let fields = syntax.self_fields(text, &hook_names())?;
    let decls: Vec<Range<usize>> = syntax
        .property_declarations(text)
        .into_iter()
        .filter(|(n, _)| n == name)
        .map(|(_, r)| r)
        .collect();
    let declared = !decls.is_empty();
    let mut sites: Vec<Range<usize>> = fields
        .sites
        .into_iter()
        .filter(|(n, _)| n == name)
        .map(|(_, r)| r)
        .chain(decls)
        .collect();
    sites.sort_by_key(|r| r.start);
    sites.dedup();
    Some((sites, declared))
}

/// O texto `text` com a propriedade `old` renomeada para `new` (declaração e campos do `self`);
/// `None` = o texto não a declara, ou não dá para ver todos os sítios.
pub(crate) fn rename_property_in(text: &str, old: &str, new: &str) -> Option<String> {
    let t = Text::new(text);
    let syntax = Syntax::new(&t);
    let (sites, declared) = field_sites(&syntax, &t, old)?;
    if !declared {
        return None;
    }
    let mut out = text.to_owned();
    for r in sites.iter().rev() {
        out.replace_range(r.clone(), new);
    }
    Some(out)
}

impl CodeDoc {
    /// F2: a caixinha sobre o nome no cursor primário.
    pub(crate) fn start_rename(&mut self) {
        self.actions = None;
        self.complete.close();
        // A pergunta é a árvore EXACTA (renomear é raro; a tecla nunca paga isto).
        self.syntax.fresh(self.buffer.text());
        let t = self.buffer.text();
        let head = self.buffer.selection().primary().head;
        let hooks = hook_names();
        let (word, kind, why) = match self.syntax.name_at(t, head, &hooks) {
            None => (head..head, None, Some(Why::Nothing)),
            Some(NameAt::Member(r)) => (r, None, Some(Why::Member)),
            Some(NameAt::Hook(r)) => (r, None, Some(Why::Hook)),
            Some(NameAt::Name(r)) => (r, Some(Kind::Name), None),
            Some(NameAt::SelfField(name, r)) => match field_sites(&self.syntax, t, &name) {
                Some((sites, declared)) => (r, Some(Kind::Field { sites, declared }), None),
                None => (r, None, Some(Why::Escapes)),
            },
        };
        let old = t.slice(word.clone());
        let mut field = Buffer::new(&old);
        field.select_all();
        self.rename = Some(Renaming {
            word,
            old,
            kind,
            field,
            why,
            sent: None,
            scene: None,
        });
    }

    /// Uma tecla com a caixinha aberta. Devolve o texto a copiar (Ctrl+C no campo).
    pub(crate) fn rename_key(&mut self, k: FieldKey<'_>) -> Option<String> {
        let r = self.rename.as_mut()?;
        if r.kind.is_none() {
            // Só a razão: qualquer tecla a fecha.
            self.rename = None;
            return None;
        }
        let out = field_key(&mut r.field, k);
        if out.escape {
            self.rename = None;
        } else if out.enter {
            self.rename_commit();
        } else if out.changed {
            r.sent = None;
            self.rename_check();
        }
        out.copy
    }

    /// A razão por que o nome escrito não serve (ou nenhuma).
    pub(crate) fn rename_check(&mut self) {
        let Some(r) = self.rename.as_mut() else {
            return;
        };
        let new = r.field.text().to_string();
        let t = self.buffer.text();
        r.why = if new == r.old {
            None
        } else if LUAU_KEYWORDS.contains(&new.as_str())
            || ph2d_script::props::RESERVED_NAMES.contains(&new.as_str())
        {
            Some(Why::Reserved)
        } else if !is_identifier(&new) {
            Some(Why::Invalid)
        } else {
            match &r.kind {
                Some(Kind::Name) if self.syntax.name_in_use(t, &new) => Some(Why::InUse),
                Some(Kind::Field { .. }) => {
                    let props = self.syntax.declared_properties(t);
                    let fields = self.syntax.self_fields(t, &hook_names());
                    if props.contains(&new)
                        || r.scene.as_ref().is_some_and(|s| s.taken.contains(&new))
                    {
                        Some(Why::Taken)
                    } else if fields.is_some_and(|f| f.sites.iter().any(|(n, _)| *n == new)) {
                        Some(Why::InUse)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        };
    }

    fn rename_commit(&mut self) {
        self.rename_check();
        let Some(r) = self.rename.as_mut() else {
            return;
        };
        let new = r.field.text().to_string();
        if r.why.is_some() {
            return;
        }
        if new == r.old {
            self.rename = None;
            return;
        }
        match &r.kind {
            Some(Kind::Name) => r.sent = Some((new, None)),
            Some(Kind::Field { sites, declared }) => {
                // A propriedade só se aplica com os objectos lidos (o valor de cada um vai com ela).
                if *declared && r.scene.is_none() {
                    return;
                }
                let (sites, declared) = (sites.clone(), *declared);
                let word = r.word.clone();
                self.rename = None;
                let edits: Vec<(Range<usize>, String)> =
                    sites.into_iter().map(|s| (s, new.clone())).collect();
                let old = self.buffer.text().slice(word.clone());
                if self.apply_rename(&edits, &word) && declared {
                    let link = UndoLink::fresh();
                    self.buffer.link_last(link.0);
                    self.scene_rename = Some(SceneRename {
                        script: self.script,
                        old,
                        new,
                        link,
                    });
                }
            }
            None => self.rename = None,
        }
    }

    /// A resposta do servidor ao rename de um nome do código.
    pub(crate) fn rename_answer(&mut self, answer: Answer) {
        let Some(r) = self.rename.as_mut() else {
            return;
        };
        r.sent = None;
        let t = self.buffer.text();
        match answer {
            Answer::Rename(Some(edits))
                if edits
                    .iter()
                    .all(|(range, _)| t.slice(range.clone()) == r.old) =>
            {
                let word = r.word.clone();
                self.rename = None;
                self.apply_rename(&edits, &word);
            }
            Answer::Failed(msg) => r.why = Some(Why::Server(msg)),
            _ => r.why = Some(Why::Server(tr("code.rename.failed").to_owned())),
        }
    }

    /// A ajuda e a razão da caixinha (a pintura).
    pub(crate) fn rename_note(&self) -> (String, Option<String>) {
        let Some(r) = &self.rename else {
            return (String::new(), None);
        };
        let shown = if r.kind.is_some() {
            r.field.text().to_string()
        } else {
            r.old.clone()
        };
        let error = r.why.as_ref().map(|w| w.text(&shown));
        let objects = match (&r.kind, &r.scene) {
            (Some(Kind::Field { declared: true, .. }), Some(s)) => s.objects,
            _ => 0,
        };
        let hint = match objects {
            0 => tr("code.rename.hint").to_owned(),
            1 => tr("code.rename.hint_objects_one").to_owned(),
            n => tr("code.rename.hint_objects").replace("{count}", &n.to_string()),
        };
        (hint, error)
    }

    /// O servidor não responde: a caixinha di-lo e fica aberta.
    pub(crate) fn rename_offline(&mut self) {
        if let Some(r) = self.rename.as_mut().filter(|r| r.sent.is_some()) {
            r.sent = None;
            r.why = Some(Why::Offline);
        }
    }

    /// As trocas num passo de undo; o cursor fica no mesmo sítio do nome renomeado.
    fn apply_rename(&mut self, edits: &[(Range<usize>, String)], word: &Range<usize>) -> bool {
        let head = self.buffer.selection().primary().head;
        let changes = edits
            .iter()
            .map(|(range, insert)| Change {
                range: range.clone(),
                insert: insert.clone(),
            })
            .collect();
        let Some(cs) = ChangeSet::new(changes) else {
            return false;
        };
        // O início do nome sob o cursor, depois das trocas antes dele.
        let shift: isize = edits
            .iter()
            .filter(|(r, _)| r.end <= word.start)
            .map(|(r, s)| s.len() as isize - r.len() as isize)
            .sum();
        let start = word.start.saturating_add_signed(shift);
        let new_len = edits.first().map_or(0, |(_, s)| s.len());
        let at = start + head.saturating_sub(word.start).min(new_len);
        self.buffer.seal_history();
        let applied = self.buffer.apply(
            cs,
            Some(Selection::single(Cursor::cursor(at))),
            EditKind::Other,
        );
        self.buffer.seal_history();
        self.edited(&applied, true);
        self.reveal();
        !applied.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_propriedade_renomeada_num_texto_leva_a_declaracao_e_os_campos_e_mais_nada() {
        let src = "ph2d.property(\"amplitude\", 1)\nfunction update(self, dt)\n  local amplitude = \"amplitude\"\n  print(self.amplitude, self[\"amplitude\"], amplitude)\nend\n";
        let out = rename_property_in(src, "amplitude", "altura").unwrap();
        assert_eq!(
            out,
            "ph2d.property(\"altura\", 1)\nfunction update(self, dt)\n  local amplitude = \"amplitude\"\n  print(self.altura, self[\"altura\"], amplitude)\nend\n"
        );
        assert_eq!(rename_property_in(src, "speed", "x"), None, "não declarada");
        assert_eq!(
            rename_property_in(
                "ph2d.property(\"a\", 1)\nfunction update(self) ajuda(self) end\n",
                "a",
                "b"
            ),
            None,
            "o self foge"
        );
    }
}
