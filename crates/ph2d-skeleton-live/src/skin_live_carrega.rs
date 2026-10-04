//! ⭐⭐ **A forma presa com efeitos VIVOS coze-os** (A4, 2026-10-04), no quadro, antes da pele — um
//! projecto anterior à F51 tem formas presas com a pilha viva (desenhada pela lei F50), e o painel de
//! uma forma presa já não a mostra: a lei do dono «presa não tem efeitos» falhava por omissão. A
//! fonte guardada passa a ser a que o Bind de hoje faria, com o desenho de antes AO BIT (gate
//! `skin_live_carrega_tests`). Uma forma sem campo (bind anterior a 2026-09-20) ou com um offset de
//! camada (`Estilo::NaoServe`) fica como está: cozê-la mudaria o desenho. ⚠️ Corre por QUADRO (e
//! não só ao abrir) para valer a qualquer origem — abrir, desfazer, colar —; depois da 1.ª vez o
//! custo é ver uma pilha vazia por forma presa, sem clonar nada.

use super::*;

/// Coze os efeitos vivos de toda forma presa da cena; devolve quantas cozeu. Uma pilha só de efeitos
/// DESLIGADOS sai sem cozer (como no Bind).
pub fn coze_os_efeitos_presos(sim: &mut SimWorld, scene: &mut VecScene) -> usize {
    let alvos: Vec<(Entity, VecPathId)> = sim
        .world()
        .iter_entities()
        .filter(|er| er.get::<SkinBind>().is_some())
        .filter_map(|er| Some((er.id(), er.get::<VecPathRef>()?.0)))
        .filter(|(_, id)| scene.path(*id).is_some_and(|p| !p.effects.is_empty()))
        .collect();
    if alvos.is_empty() {
        return 0;
    }
    let index = bone_index(sim);
    let mut feitos = 0;
    for (e, id) in alvos {
        let (Some(viva), Some(mut skin)) =
            (scene.path(id), sim.world().get::<SkinBind>(e).cloned())
        else {
            continue;
        };
        let pilha = match crate::skin_desenho::estilo_de(viva) {
            crate::skin_desenho::Estilo::Efeitos(p) => p,
            crate::skin_desenho::Estilo::Serve => {
                if let Some(p) = scene.path_mut(id) {
                    p.effects.clear();
                }
                continue;
            }
            crate::skin_desenho::Estilo::NaoServe => continue,
        };
        // ⚠️ Uma fonte JÁ cozida que voltou a ter pilha (colar, desfazer) coze outra vez: o desenho
        // da F50 sobre ela é o mesmo, e a lei é «presa não tem efeitos» (mutação: a guarda sobrevivia).
        let Some(g) = crate::skinned_mesh::le(&skin.source) else {
            continue;
        };
        let eixos = eixos_do_bind(sim, &skin, &index);
        let Some(novo) = crate::skin_desenho::coze_para_guardar(&g, &pilha, &eixos) else {
            continue;
        };
        let Some(bytes) = crate::skinned_mesh::grava(&novo) else {
            continue;
        };
        skin.source = bytes;
        sim.world_mut().entity_mut(e).insert(skin);
        if let Some(p) = scene.path_mut(id) {
            p.replace_geometry(novo.path);
            p.effects.clear();
        }
        feitos += 1;
    }
    feitos
}

#[cfg(test)]
#[path = "skin_live_carrega_tests.rs"]
mod tests;
