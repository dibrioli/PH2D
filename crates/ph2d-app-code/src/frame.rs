//! ⭐ **O quadro da família** — a ÚNICA porta entre o editor e a cena, chamada pela shell uma vez por
//! quadro: abre o que o cartão pediu, entrega o texto salvo à `ScriptLibrary` (e pede a gravação do
//! projeto pelo MESMO caminho do File ▸ Save), lê as recargas (o selo), o console e o diagnóstico.

use crate::doc::CodeDoc;
use crate::docs::{CodeDocs, SaveKind};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_panel_code::Badge;
use ph2d_script::{
    ProjectScripts, ReloadOutcome, ScriptEditRequests, ScriptHost, ScriptId, ScriptInfo,
};

/// Um quadro do editor de scripts. `host` = a VM (`None` = o app corre sem scripts); `toasts` = os
/// avisos do ecrã (o projecto trocado com textos por gravar diz onde eles ficaram).
pub fn frame(
    sim: &mut SimWorld,
    mut host: Option<&mut ScriptHost>,
    hero: &mut HeroScreen,
    toasts: &mut ToastQueue,
) {
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
    let clock = (hero.store.clock_ns(), hero.motion.reduced_motion());
    let Some(docs) = hero.documents.surfaces_mut().find_mut::<CodeDocs>() else {
        return;
    };
    docs.set_clock(clock.0, clock.1);
    docs.follow_active(our_active);
    let world = sim.world_mut();
    let mut wanted = world
        .get_resource_mut::<ScriptEditRequests>()
        .map(|mut r| std::mem::take(&mut r.0))
        .unwrap_or_default();
    if let Some(lib) = world.get_resource::<ProjectScripts>() {
        if docs.follow_epoch(lib.epoch(), toasts) {
            wanted.clear(); // pedidos de ids do projecto anterior
        }
        for id in wanted {
            if let Some(d) = lib.get(id) {
                docs.open(id, &d.name, &d.text);
            }
        }
        follow_library(docs, lib);
    }
    docs.apply_list(world);
    if let Some(lib) = world.get_resource::<ProjectScripts>() {
        follow_library(docs, lib);
    }
    let ask = save(docs, world, host.as_deref_mut(), menu_save, our_active);
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

/// Os salvares pedidos (Ctrl+S na aba; File ▸ Save / Ctrl+S com QUALQUER aba activa, e então também
/// as abas fechadas com ●): o texto vai para a biblioteca — a cena recarrega no quadro seguinte, sem disco. ⭐ Gravar o projeto aplica
/// também toda aba com ● (decisão do dono: «salvar = guardado»; um texto partido deixa a versão
/// anterior a correr — é o motor). Devolve o pedido de gravar o projeto.
fn save(
    docs: &mut CodeDocs,
    world: &mut ph2d_ecs::World,
    mut host: Option<&mut ScriptHost>,
    menu: bool,
    active: Option<u64>,
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
        let by_menu = menu && (d.buffer.is_dirty() || active == Some(d.script.0));
        if kind.is_some() || by_menu {
            saving.push(i);
        }
    }
    // As abas FECHADAS com ● também (a lista pinta-as com ●): gravadas, deixam de estar estacionadas.
    let parked: Vec<ScriptId> = if menu {
        docs.parked.keys().copied().collect()
    } else {
        Vec::new()
    };
    if saving.is_empty() && parked.is_empty() {
        return ask;
    }
    let mut lib = world.get_resource_or_init::<ProjectScripts>();
    for id in parked {
        if let Some(d) = docs.parked.remove(&id) {
            lib.set_text(id, &d.buffer.text().to_string());
        }
    }
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
        d.set_saved_diag(match h.scene_info(d.script) {
            Some(ScriptInfo::Broken(diag)) => Some(diag.clone()),
            _ => None,
        });
    }
}
