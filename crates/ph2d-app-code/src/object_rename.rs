//! ⭐⭐ **Renomear um objecto reescreve os scripts** (item 8, `docs/CodeEditor/07_arrastar_para_o_codigo.md`
//! §3.5; divergência do Godot, que parte em silêncio — `ferramentas/oraculo_arrastar/saida.txt`).
//!
//! Chamado no COMMIT do renomear da Hierarquia (a shell), antes do diff do quadro: o passo global do
//! renomear nasce com o elo pendente ([`ph2d_editor_core::undo_link`]) e cada texto mudado (a
//! biblioteca, a aba aberta, a estacionada) leva o mesmo elo — um Ctrl+Z desfaz o nome e os textos
//! juntos, pela costura do I10 ([`crate::link`]).

use crate::docs::CodeDocs;
use crate::link::Linked;
use ph2d_code_buffer::{Buffer, Edits, Range, Selection};
use ph2d_code_syntax::Syntax;
use ph2d_code_syntax::dropping::rename_edits;
use ph2d_ecs::{Entity, MasterRoot, World};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::undo_link::UndoLink;
use ph2d_script::{ProjectScripts, ScriptId};

/// O texto `before` com o nome reescrito (`None` = nenhum nome a mudar).
fn rewritten(before: &str, old: &str, new: &str, master: bool) -> Option<String> {
    let t = ph2d_code_buffer::Text::new(before);
    let edits = rename_edits(&t, &Syntax::new(&t), old, new, master);
    if edits.is_empty() {
        return None;
    }
    let mut s = before.to_owned();
    for (r, inner) in edits {
        s.replace_range(r, &inner);
    }
    Some(s)
}

/// O buffer com o nome reescrito num grupo de undo com o elo `link` (a selecção segue as edições).
fn rewrite_buffer(
    buffer: &mut Buffer,
    old: &str,
    new: &str,
    master: bool,
    link: UndoLink,
) -> Option<Edits> {
    let mut edits = rename_edits(buffer.text(), &Syntax::new(buffer.text()), old, new, master);
    let inner = edits.first()?.1.clone();
    edits.reverse();
    let shift = |b: usize| {
        let mut out = b;
        for (r, s) in &edits {
            if r.end <= b {
                out = out + s.len() - r.len();
            }
        }
        out
    };
    let keep = buffer.selection().clone();
    let kept: Vec<Range> = keep
        .ranges()
        .iter()
        .map(|r| Range::new(shift(r.anchor), shift(r.head)))
        .collect();
    let ranges: Vec<Range> = edits
        .iter()
        .map(|(r, _)| Range::new(r.start, r.end))
        .collect();
    buffer.seal_history();
    buffer.set_selection(Selection::new(ranges, 0));
    let done = buffer.insert(&inner);
    buffer.set_selection(Selection::new(kept, keep.primary_index()));
    buffer.link_last(link.0);
    buffer.seal_history();
    Some(done)
}

/// ⭐ O objecto `entity` passou de `old` a `new` (um mestre muda os `ph2d.spawn`; um objecto, os
/// `ph2d.object`/`ph2d.find_by_name`). Sem a família instalada ou sem nada a mudar, não faz nada.
pub fn object_renamed(
    hero: &mut HeroScreen,
    world: &mut World,
    entity: Entity,
    old: &str,
    new: &str,
) {
    if old == new {
        return;
    }
    let master = world.get::<MasterRoot>(entity).is_some();
    let Some(docs) = hero.documents.surfaces_mut().find_mut::<CodeDocs>() else {
        return;
    };
    let scripts: Vec<(ScriptId, String)> = world
        .get_resource::<ProjectScripts>()
        .map(|lib| lib.iter().map(|(id, s)| (id, s.text.clone())).collect())
        .unwrap_or_default();
    let link = UndoLink::fresh();
    let mut linked = false;
    for (id, before) in scripts {
        let after = rewritten(&before, old, new, master);
        let mut tab = false;
        if let Some(d) = docs.docs.iter_mut().find(|d| d.script == id) {
            if let Some(edits) = rewrite_buffer(&mut d.buffer, old, new, master, link) {
                d.edited(&edits, true);
                tab = true;
            }
        } else if let Some(p) = docs.parked.get_mut(&id) {
            tab = rewrite_buffer(&mut p.buffer, old, new, master, link).is_some();
        }
        if after.is_none() && !tab {
            continue;
        }
        let after = after.unwrap_or_else(|| before.clone());
        if after != before {
            world.resource_mut::<ProjectScripts>().set_text(id, &after);
            docs.reload.push(id);
        }
        docs.follow_synced(id, &after);
        docs.links.list.push(Linked::new(link, id, before, after));
        linked = true;
    }
    if linked {
        hero.undo_links.pending = Some(link);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_texto_e_o_buffer_reescrevem_se_e_um_undo_volta() {
        let src = "local p = ph2d.object(\"Porta\")\nlocal q = ph2d.find_by_name(\"Porta\")\n";
        assert_eq!(
            rewritten(src, "Porta", "Portão", false).as_deref(),
            Some("local p = ph2d.object(\"Portão\")\nlocal q = ph2d.find_by_name(\"Portão\")\n")
        );
        assert_eq!(
            rewritten(src, "Porta", "Portão", true),
            None,
            "um mestre não muda estes"
        );
        let mut b = Buffer::new(src);
        let fim = src.len();
        b.set_selection(Selection::single(Range::new(fim, fim)));
        let edits = rewrite_buffer(&mut b, "Porta", "Portão", false, UndoLink(7)).expect("mudou");
        assert_eq!(edits.len(), 2);
        assert_eq!(
            b.text().to_string(),
            rewritten(src, "Porta", "Portão", false).unwrap()
        );
        let novo_fim = b.text().len();
        assert_eq!(
            b.selection().primary().head,
            novo_fim,
            "o cursor do fim segue as edições (ç e ã têm 2 bytes)"
        );
        assert_eq!(b.history().top_link(false), Some(7), "o grupo leva o elo");
        b.undo();
        assert_eq!(b.text().to_string(), src, "um undo desfaz as duas");
    }
}
