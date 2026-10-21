//! ⭐ **I10 — a metade da família do marcador do undo global** ([`ph2d_editor_core::undo_link`]).
//!
//! Renomear uma `ph2d.property` muda três coisas: o texto da aba, o texto da biblioteca (o Inspector
//! lê as declarações dela — sem isto o valor de cada objecto ficava órfão até ao Ctrl+S) e as chaves
//! `own` de cada objecto do script. O grupo do texto leva um [`UndoLink`]; a mudança das chaves é um
//! passo da fila global com o mesmo elo (o diff apanha-as; o elo pendente fá-lo nascer mesmo sem
//! objectos). A fila global é a dona: um Ctrl+Z na aba pede-lhe o passo ([`UndoLinks::ask`]), um na
//! cena desfá-lo directamente, e nos dois casos a família aplica a sua metade quando o lê em
//! [`UndoLinks::done`]: o texto da aba e o da biblioteca.

use crate::docs::CodeDocs;
use crate::rename::{Scene, SceneRename, rename_property_in};
use ph2d_ecs::World;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::toast::Toast;
use ph2d_editor_core::undo_link::{Refusal, UndoLink, UndoLinks};
use ph2d_i18n::tr;
use ph2d_script::{LuauScript, ProjectScripts, ScriptHost, ScriptId, ScriptSource};

/// Um passo ligado vivo: o script e o texto da biblioteca antes e depois.
pub(crate) struct Linked {
    link: UndoLink,
    script: ScriptId,
    before: String,
    after: String,
}

impl Linked {
    pub(crate) fn new(link: UndoLink, script: ScriptId, before: String, after: String) -> Self {
        Self {
            link,
            script,
            before,
            after,
        }
    }
}

#[derive(Default)]
pub(crate) struct Links {
    pub(crate) list: Vec<Linked>,
}

/// Um quadro: o mundo da família (`world`), a costura com a fila global (`links`).
pub(crate) fn frame(
    docs: &mut CodeDocs,
    world: &mut World,
    links: &mut UndoLinks,
    mut host: Option<&mut ScriptHost>,
    toasts: &mut ToastQueue,
) {
    scene_facts(docs, world);
    for i in 0..docs.docs.len() {
        if let Some(job) = docs.docs[i].scene_rename.take()
            && !apply(docs, &job, world, links, host.as_deref_mut())
        {
            docs.docs[i].buffer.unlink(job.link.0);
        }
    }
    for (l, redo) in std::mem::take(&mut links.done) {
        undone(docs, l, redo, world, host.as_deref_mut());
    }
    // O elo que a fila global já não guarda: o grupo fica um passo de texto como os outros.
    let dead: Vec<UndoLink> = docs
        .links
        .list
        .iter()
        .map(|x| x.link)
        .filter(|l| !links.is_alive(*l))
        .collect();
    for l in dead {
        for d in &mut docs.docs {
            d.buffer.unlink(l.0);
        }
        for p in docs.parked.values_mut() {
            p.buffer.unlink(l.0);
        }
        docs.links.list.retain(|x| x.link != l);
    }
    for (side, set) in links.held.iter_mut().enumerate() {
        set.clear();
        let redo = side == 1;
        for x in &docs.links.list {
            let mut buffers = docs
                .docs
                .iter()
                .map(|d| &d.buffer)
                .chain(docs.parked.values().map(|p| &p.buffer));
            let held = buffers.any(|b| {
                b.history().holds_link(x.link.0, redo)
                    && b.history().top_link(redo) != Some(x.link.0)
            });
            if held {
                set.insert(x.link);
            }
        }
    }
    // O Ctrl+Z da aba sobre um grupo ligado vai à fila global; um elo sem passo vivo já é texto.
    for d in &mut docs.docs {
        if let Some((l, redo)) = d.link_ask.take() {
            if docs.links.list.iter().any(|x| x.link.0 == l) {
                links.ask = Some((UndoLink(l), redo));
            } else {
                d.buffer.unlink(l);
                d.undo_text(redo);
            }
        }
    }
    if let Some(r) = links.refused.take() {
        toasts.push(Toast::warning(tr(match r {
            Refusal::FamilyAhead => "code.undo.family_ahead",
            Refusal::GlobalAhead => "code.undo.global_ahead",
        })));
    }
}

/// Os objectos do script de cada caixinha de rename de propriedade aberta.
fn scene_facts(docs: &mut CodeDocs, world: &mut World) {
    for d in &mut docs.docs {
        let script = d.script;
        let Some(r) = d.rename.as_mut() else { continue };
        if !matches!(
            r.kind,
            Some(crate::rename::Kind::Field { declared: true, .. })
        ) {
            continue;
        }
        let mut s = Scene::default();
        let mut q = world.query::<&LuauScript>();
        for c in q.iter(world) {
            if c.source == ScriptSource::Library(script) {
                s.objects += usize::from(c.own.contains_key(&r.old));
                s.taken
                    .extend(c.own.keys().filter(|k| **k != r.old).cloned());
            }
        }
        if r.scene.as_ref() != Some(&s) {
            r.scene = Some(s);
            d.rename_check();
        }
    }
}

/// A metade da cena de um rename de propriedade: a biblioteca e as chaves `own`. `false` = a
/// biblioteca não declara a propriedade (nova, por salvar): o rename fica só no texto da aba.
fn apply(
    docs: &mut CodeDocs,
    job: &SceneRename,
    world: &mut World,
    links: &mut UndoLinks,
    host: Option<&mut ScriptHost>,
) -> bool {
    let Some(lib) = world.get_resource::<ProjectScripts>() else {
        return false;
    };
    let Some(before) = lib.get(job.script).map(|s| s.text.clone()) else {
        return false;
    };
    let Some(after) = rename_property_in(&before, &job.old, &job.new) else {
        return false;
    };
    let mut q = world.query::<&mut LuauScript>();
    for mut c in q.iter_mut(world) {
        if c.source == ScriptSource::Library(job.script)
            && let Some(v) = c.own.remove(&job.old)
        {
            c.own.insert(job.new.clone(), v);
        }
    }
    set_library(docs, job.script, &after, world, host);
    links.pending = Some(job.link);
    docs.links.list.push(Linked {
        link: job.link,
        script: job.script,
        before,
        after,
    });
    true
}

/// A fila global desfez (`redo = false`) ou refez o passo com o elo `l`: o texto de cada aba e da
/// biblioteca que o elo levou segue-o (o renomear de um objecto liga VÁRIOS scripts, item 8).
fn undone(
    docs: &mut CodeDocs,
    l: UndoLink,
    redo: bool,
    world: &mut World,
    mut host: Option<&mut ScriptHost>,
) {
    let texts: Vec<(ScriptId, String)> = docs
        .links
        .list
        .iter()
        .filter(|x| x.link == l)
        .map(|x| {
            (
                x.script,
                if redo {
                    x.after.clone()
                } else {
                    x.before.clone()
                },
            )
        })
        .collect();
    // Do lado de onde o passo sai: o undo antes de desfazer, o redo antes de refazer.
    for d in docs
        .docs
        .iter_mut()
        .filter(|d| d.buffer.history().top_link(redo) == Some(l.0))
    {
        d.undo_text(redo);
        d.reveal();
    }
    for p in docs
        .parked
        .values_mut()
        .filter(|p| p.buffer.history().top_link(redo) == Some(l.0))
    {
        if redo {
            p.buffer.redo();
        } else {
            p.buffer.undo();
        }
    }
    for (script, text) in texts {
        set_library(docs, script, &text, world, host.as_deref_mut());
    }
}

/// O texto da biblioteca muda por um passo ligado: a aba do script concorda com ele, e a cena
/// recarrega.
fn set_library(
    docs: &mut CodeDocs,
    script: ScriptId,
    text: &str,
    world: &mut World,
    host: Option<&mut ScriptHost>,
) {
    world
        .get_resource_or_init::<ProjectScripts>()
        .set_text(script, text);
    docs.follow_synced(script, text);
    if let Some(h) = host {
        h.request_reload(script);
    }
}

impl CodeDocs {
    /// O texto da biblioteca do script passou a ser `text` por um passo ligado: a aba (aberta ou
    /// estacionada) concorda com ele — sem isto o `follow_library` trocá-la-ia por uma nova.
    pub(crate) fn follow_synced(&mut self, script: ScriptId, text: &str) {
        if let Some(d) = self.docs.iter_mut().find(|d| d.script == script) {
            d.synced = text.to_owned();
            if d.buffer.text().to_string() == text {
                d.buffer.mark_saved();
            }
        } else if let Some(p) = self.parked.get_mut(&script) {
            p.synced = text.to_owned();
            if p.buffer.text().to_string() == text {
                p.buffer.mark_saved();
            }
        }
    }
}
