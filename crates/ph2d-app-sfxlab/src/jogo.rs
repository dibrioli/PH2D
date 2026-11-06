//! **Use in Game e Create sound:** o som aprovado chega ao objeto. A ligação é escrita no mundo, e a fila de desfazer da
//! cena a vê pelo diff; o som fica no projeto, sem efeito no jogo (`docs/SfxLab/04_W3_plano.md` §2).

use bevy_ecs::prelude::Resource;
use bevy_ecs::world::World;
use ph2d_ecs::{
    Entity, Name, ProjectSounds, StableId, assign_missing_stable_ids, entity_with_stable_id,
    stable_id_of,
};
use ph2d_i18n::tr_with;
use ph2d_sfx_lab::aprovar::{AprovarErro, aprovar};
use ph2d_sfx_lab::ligacao::{liga, objetos_da_cena};
use ph2d_sfx_recipe::SfxLibrary;

use crate::aba::Aba;
use crate::docs::SfxLabDocs;

/// O Create sound do Inspector, à espera do `service`, que é quem tem as abas.
#[derive(Resource, Default)]
struct Pedidos(Vec<StableId>);

/// A porta do cartão Audio Source 2D: pede uma aba nova ligada a este objeto. O que se guarda é a identidade dele.
pub fn pede_som_para(world: &mut World, objeto: Entity) {
    assign_missing_stable_ids(world);
    if let Some(id) = stable_id_of(world, objeto) {
        world
            .get_resource_or_insert_with(Pedidos::default)
            .0
            .push(id);
    }
}

fn usa(a: &mut Aba, world: &mut World, objeto: Entity) {
    if !world.contains_resource::<SfxLibrary>() {
        world.insert_resource(SfxLibrary::default());
    }
    let mut sons = world.remove_resource::<ProjectSounds>().unwrap_or_default();
    let feito = world
        .get_resource_mut::<SfxLibrary>()
        .ok_or(AprovarErro::SemReceita)
        .and_then(|mut lib| aprovar(&mut lib, &mut sons, &a.nome));
    world.insert_resource(sons);
    match feito {
        Ok(som) => {
            let aprovado = world
                .get_resource::<SfxLibrary>()
                .and_then(|l| l.recipes.get(&a.nome))
                .and_then(|r| r.approved.clone());
            a.receita.approved = aprovado;
            a.escrita = Some(a.receita.clone());
            a.recusa = None;
            liga(world, objeto, &som);
        }
        Err(e) => {
            a.recusa = Some(tr_with(
                "sfxlab.use.failed",
                &[("reason", &format!("{e:?}"))],
            ));
        }
    }
}

impl SfxLabDocs {
    /// Depois de a receita estar escrita no mundo: quem aprova lê de lá.
    pub(crate) fn atende_o_jogo(&mut self, world: &mut World) {
        if let Some(p) = world.remove_resource::<Pedidos>() {
            for id in p.0 {
                let objeto = entity_with_stable_id(world, id)
                    .and_then(|e| world.get::<Name>(e))
                    .map(|n| n.as_str().trim())
                    .filter(|n| !n.is_empty());
                let aba = match objeto {
                    Some(nome) => self.novo_para(nome),
                    None => self.novo(),
                };
                if let Some(a) = self.aba_mut(aba) {
                    a.ligado = Some(id);
                }
            }
        }
        for a in &mut self.abas {
            if !std::mem::take(&mut a.quer_usar) {
                continue;
            }
            let vivo = a.ligado.and_then(|id| entity_with_stable_id(world, id));
            match vivo {
                Some(objeto) => {
                    a.objeto_sumiu = false;
                    usa(a, world, objeto);
                }
                None => {
                    a.objeto_sumiu = a.ligado.take().is_some();
                    a.objetos = objetos_da_cena(world);
                    a.menu_objetos = true;
                    a.menu_ini = 0;
                }
            }
        }
    }
}
