//! O que a composição passa às portas de aplicar, nos testes: o registo de componentes do produto (os
//! que a família «mundo» põe), o remapeador das referências e o Motion como família injectada.

use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components};
use ph2d_ecs::{Entity, World};
use ph2d_editor_core::macro_edits::{CampoDaEtapa, ErroDaMacroView, EtapaView, FamiliaDaMacro};
use ph2d_editor_core::macro_etapas::{EtapasDaFamilia, erros};
use ph2d_macro::{Etapa, Event, LeitorDaFamilia, MacroTime, Track};
use std::collections::BTreeMap;

/// O registo como a shell o monta (`build_component_registry`), nas famílias que estes testes usam.
pub(crate) fn registo() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    ph2d_physics_ecs::register_physics_components(&mut r);
    ph2d_render::register_render_components(&mut r);
    r
}

/// A tabela da casa (`ph2d_app_components::instance_refs::REMAPPERS`), pelos mesmos remapeadores.
pub(crate) fn remapeia(w: &mut World, es: &[Entity], m: &BTreeMap<u64, u64>) -> usize {
    ph2d_ecs::remap_instance_of(w, es, m)
        + ph2d_physics_ecs::remap_joint_refs(w, es, m)
        + ph2d_physics_ecs::remap_wheel_refs(w, es, m)
}

/// Um recurso que não está no `AssetDb`.
pub(crate) fn sem_recurso(_: &[u8; 32]) -> Option<Vec<u8>> {
    None
}

struct Leitor;
impl LeitorDaFamilia for Leitor {
    fn etapas(&self, ev: &[Event]) -> Vec<Etapa> {
        (0..ev.len()).map(|i| Etapa { inicio: i, fim: i }).collect()
    }
    fn e_relogio(&self, _: &[u8]) -> bool {
        false
    }
    fn espera(&self, _: &[Event], _: usize, _: MacroTime, _: u64, _: i64) -> Vec<Event> {
        Vec::new()
    }
    fn desloca(&self, _: &[u8], _: i64) -> Option<Vec<u8>> {
        None
    }
    fn sai_da_gravacao(&self) -> Option<Vec<u8>> {
        None
    }
}

/// O Motion como a composição o injecta: aqui só a chave e a família contam.
pub(crate) struct EtapasDoMotion;
impl EtapasDaFamilia for EtapasDoMotion {
    fn familia(&self) -> FamiliaDaMacro {
        FamiliaDaMacro::Motion
    }
    fn chave(&self) -> &'static str {
        "motion"
    }
    fn leitor(&self) -> &'static dyn LeitorDaFamilia {
        &Leitor
    }
    fn vistas(&self, _: usize, _: &Track) -> Result<Vec<EtapaView>, ErroDaMacroView> {
        Ok(Vec::new())
    }
    fn edita(&self, _: &mut Track, _: usize, _: &CampoDaEtapa) -> Result<(), ErroDaMacroView> {
        Err(erros::sem_etapa())
    }
}

pub(crate) const COM_MOTION: ph2d_app_macros::etapas::Injectadas<'static> = &[&EtapasDoMotion];
