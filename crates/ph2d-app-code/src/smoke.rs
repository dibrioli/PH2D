//! A cena de smoke do editor de scripts (`PH2D_CODE_SMOKE`), inerte sem a variável.
//!
//! - **1** — sobre uma cena com scripts (ex.: `PH2D_SCRIPT_SMOKE=1`): o script do objecto escolhido
//!   (ou, sem escolha, o primeiro que um objecto corre) abre numa aba, uma vez.
//!
//! ⚠️ A cena própria do E15 (formas, simulação, três objectos no mesmo script) ainda não existe.

use crate::docs::CodeDocs;
use ph2d_ecs::{Entity, World};
use ph2d_script::{LuauScript, ProjectScripts, ScriptId, ScriptSource};
use std::sync::OnceLock;

/// O maior nível que [`stage`] responde (contado no `match` dele).
pub const CENAS: u32 = 1;

pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[ph2d_app_host::SmokeRouter {
    env: "PH2D_CODE_SMOKE",
    max_level: CENAS,
}];

/// O nível pedido, lido por `get` (a porta injectável: os testes não mexem no ambiente).
#[must_use]
pub fn armed_level_in(get: impl Fn(&str) -> Option<String>) -> Option<u32> {
    get(ROUTERS[0].env)?
        .trim()
        .parse()
        .ok()
        .filter(|n| (1..=CENAS).contains(n))
}

/// O nível do processo (lido uma vez).
pub(crate) fn armed_level() -> Option<u32> {
    static LEVEL: OnceLock<Option<u32>> = OnceLock::new();
    *LEVEL.get_or_init(|| armed_level_in(|k| std::env::var(k).ok()))
}

/// Encena o nível `level` (uma vez por sessão). `selected` = o objecto escolhido na cena.
pub(crate) fn stage(level: u32, world: &mut World, docs: &mut CodeDocs, selected: Option<u64>) {
    if docs.smoke_done {
        return;
    }
    match level {
        1 => {
            let mut q = world.query::<(Entity, &LuauScript)>();
            let users: Vec<(u64, ScriptId)> = q
                .iter(world)
                .filter_map(|(e, c)| match c.source {
                    ScriptSource::Library(id) => Some((e.to_bits(), id)),
                    _ => None,
                })
                .collect();
            let id = users
                .iter()
                .find(|(b, _)| Some(*b) == selected)
                .or_else(|| users.first())
                .map(|(_, id)| *id);
            let Some(lib) = world.get_resource::<ProjectScripts>() else {
                return;
            };
            if let Some((id, d)) = id.and_then(|id| lib.get(id).map(|d| (id, d))) {
                docs.open(id, &d.name, &d.text);
                docs.smoke_done = true;
            }
        }
        _ => docs.smoke_done = true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_nivel_le_se_pela_porta_e_fora_da_faixa_e_inerte() {
        let com = |v: &'static str| move |k: &str| (k == "PH2D_CODE_SMOKE").then(|| v.to_owned());
        assert_eq!(armed_level_in(com("1")), Some(1));
        assert_eq!(armed_level_in(com("2")), None);
        assert_eq!(armed_level_in(com("x")), None);
        assert_eq!(armed_level_in(|_| None), None);
    }

    /// Nível 1: abre o script do objecto ESCOLHIDO (não o primeiro), uma vez só.
    #[test]
    fn o_nivel_1_abre_o_script_do_escolhido_uma_vez() {
        use ph2d_editor_core::doc_surface::DocumentSurface;
        let mut sim = ph2d_ecs::SimWorld::new();
        let w = sim.world_mut();
        let (a, b) = {
            let mut lib = w.get_resource_or_init::<ProjectScripts>();
            (lib.insert("a.luau", "-- a"), lib.insert("b.luau", "-- b"))
        };
        w.spawn(LuauScript::library(a));
        let escolhido = w.spawn(LuauScript::library(b)).id().to_bits();
        let mut docs = CodeDocs::default();
        stage(1, w, &mut docs, Some(escolhido));
        assert_eq!(docs.open_scripts(), [b]);
        docs.close(b.0);
        stage(1, w, &mut docs, Some(escolhido));
        assert!(docs.open_scripts().is_empty(), "uma vez por sessão");
        // Sem escolha: o primeiro que um objecto corre.
        let mut docs = CodeDocs::default();
        stage(1, w, &mut docs, None);
        assert_eq!(docs.open_scripts().len(), 1);
    }
}
