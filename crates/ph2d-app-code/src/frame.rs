//! ⭐ **O quadro da família** — a ÚNICA porta entre o editor e a cena, chamada pela shell uma vez por
//! quadro: abre o que o cartão pediu, entrega o texto salvo à `ScriptLibrary` (e pede a gravação do
//! projeto pelo MESMO caminho do File ▸ Save), lê as recargas (o selo), o console e o diagnóstico.

use crate::doc::CodeDoc;
use crate::docs::{CodeDocs, SaveKind};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_panel_code::Badge;
use ph2d_script::{
    ProjectScripts, ReloadOutcome, ScriptEditRequests, ScriptHost, ScriptId, ScriptInfo,
};

/// Um quadro do editor de scripts. `host` = a VM (`None` = o app corre sem scripts).
pub fn frame(sim: &mut SimWorld, mut host: Option<&mut ScriptHost>, hero: &mut HeroScreen) {
    let menu_save = hero.file_menu.save || hero.file_menu.save_as;
    let active = hero.documents.surface_active();
    let our_active = active.and_then(|t| {
        let ours = hero
            .documents
            .surfaces_mut()
            .get(t.family)
            .is_some_and(|f| f.as_any().is::<CodeDocs>());
        ours.then_some(t.id)
    });
    let Some(docs) = hero.documents.surfaces_mut().find_mut::<CodeDocs>() else {
        return;
    };
    let world = sim.world_mut();
    let wanted = world
        .get_resource_mut::<ScriptEditRequests>()
        .map(|mut r| std::mem::take(&mut r.0))
        .unwrap_or_default();
    if let Some(lib) = world.get_resource::<ProjectScripts>() {
        for id in wanted {
            if let Some(d) = lib.get(id) {
                docs.open(id, &d.name, &d.text);
            }
        }
        follow_library(docs, lib);
    }
    let ask = save(
        docs,
        world,
        host.as_deref_mut(),
        our_active.filter(|_| menu_save),
    );
    if let Some(h) = host {
        hear(docs, h);
    }
    match ask {
        Some(SaveKind::Save) => hero.file_menu.save = true,
        Some(SaveKind::SaveAs) => hero.file_menu.save_as = true,
        None => {}
    }
}

/// A biblioteca manda no que a aba não mexeu: um script que sumiu (outro projeto) fecha a aba; um
/// nome novo muda o título; um texto mudado por fora entra numa aba SEM alterações por salvar. Uma
/// aba com ● nunca perde o texto do artista.
fn follow_library(docs: &mut CodeDocs, lib: &ProjectScripts) {
    docs.docs.retain(|d| lib.get(d.script).is_some());
    docs.parked.retain(|id, _| lib.get(*id).is_some());
    for d in &mut docs.docs {
        let Some(s) = lib.get(d.script) else { continue };
        if d.title != s.name {
            d.title.clone_from(&s.name);
        }
        if !d.buffer.is_dirty() && d.synced != s.text {
            let fresh = CodeDoc::new(d.script, &s.name, &s.text);
            *d = fresh;
        }
    }
}

/// Os salvares pedidos (Ctrl+S na aba; File ▸ Save com a aba activa): o texto vai para a biblioteca
/// — a cena recarrega no quadro seguinte, sem disco. Devolve o pedido de gravar o projeto.
fn save(
    docs: &mut CodeDocs,
    world: &mut ph2d_ecs::World,
    mut host: Option<&mut ScriptHost>,
    menu: Option<u64>,
) -> Option<SaveKind> {
    let mut ask = None;
    let mut saving = Vec::new();
    for (i, d) in docs.docs.iter_mut().enumerate() {
        let kind = d.save.take();
        if let Some(k) = kind {
            ask = Some(if ask == Some(SaveKind::SaveAs) {
                SaveKind::SaveAs
            } else {
                k
            });
        }
        if kind.is_some() || menu == Some(d.script.0) {
            saving.push(i);
        }
    }
    if saving.is_empty() {
        return ask;
    }
    let mut lib = world.get_resource_or_init::<ProjectScripts>();
    for i in saving {
        let d = &mut docs.docs[i];
        let text = d.buffer.text().to_string();
        let changed = lib.set_text(d.script, &text);
        d.synced = text;
        d.buffer.mark_saved();
        let users = host.as_deref().map_or(0, |h| h.objects_using(d.script));
        d.badge = (users == 0).then_some(Badge::Saved);
        if !changed && let Some(h) = host.as_deref_mut() {
            h.request_reload(d.script);
        }
    }
    ask
}

/// O que a cena disse desde o último quadro: o selo de cada recarga, o console, o diagnóstico.
fn hear(docs: &mut CodeDocs, h: &mut ScriptHost) {
    for r in h.drain_reloads() {
        let badge = match r.outcome {
            ReloadOutcome::Reloaded => Badge::Reloaded { objects: r.objects },
            ReloadOutcome::Refused(d) => Badge::Refused { line: d.line },
        };
        if let Some(d) = docs.docs.iter_mut().find(|d| d.script == r.script) {
            d.badge = Some(badge);
        }
    }
    for l in h.drain_console() {
        if l.script == ScriptId::NONE {
            continue;
        }
        if docs.consoles.entry(l.script).or_default().push(&l)
            && let Some(d) = docs.docs.iter_mut().find(|d| d.script == l.script)
        {
            d.console_view.follow();
        }
    }
    for d in &mut docs.docs {
        d.saved_diag = match h.scene_info(d.script) {
            Some(ScriptInfo::Broken(diag)) => Some(diag.clone()),
            _ => None,
        };
    }
}
