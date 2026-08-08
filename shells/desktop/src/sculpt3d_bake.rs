//! **O OBJETO MISTO (O2), o lado do GESTO** — assar a forma da malha num sprite da cena.
//!
//! Módulo FILHO de [`super`] (`#[path]`), irmão da [`super::donation`]: lá a forma acende **a tela
//! do Painter** (o objetivo 1 — `docs/3D/05.2`), aqui ela acende **um objeto da cena** (o objetivo
//! 2 — `docs/3D/02.2`). São duas perguntas diferentes e a segunda tem uma propriedade que a primeira
//! não tem: **o resultado sobrevive à malha**.
//!
//! ## O que mora aqui e o que NÃO mora
//!
//! Aqui fica só o que **precisa da malha**: ler os pixels do sprite, pedir o G-buffer à cena, e
//! montar o objeto assado. Tudo o que um objeto assado precisa **depois** — os canais, o carimbo do
//! rig, a acendida, a codificação com que ele viaja no arquivo — mora na [`crate::baked_form`], que
//! **não está atrás da feature `sculpt3d`**.
//!
//! ⚠️ **Essa divisão É a rota A.** O `docs/3D/02.2` promete que o G-buffer é gerado uma vez, vira
//! canal do sprite, e a malha some do build; um módulo único deixaria a acendida cair junto com a
//! feature, e a promessa viraria prosa. O corte é literalmente *o que precisa da malha* × *o que
//! precisa só dos canais*.

use std::collections::BTreeMap;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{BakedForm as BakedFormId, Entity, SimWorld};
use ph2d_gpu::GpuContext;
use ph2d_render::{ImpastoLightPass, SpriteRenderer};

use crate::baked_form::{BakedForm, light, rig_stamp};

use super::Sculpt3dScene;

/// **O GESTO** — assa a forma no sprite selecionado.
///
/// ⚠️ **A câmera é a do ESCULTOR**, a mesma decisão da doação: a pose em que o artista deixou o
/// modelo É a pose sobre a qual ele quer o objeto aceso. Não há enquadramento novo a inventar, e
/// inventar um faria o sprite acender por uma vista que ninguém escolheu.
///
/// ⚠️ **O sprite passa a ser `Individual`.** Ele deixa de compartilhar a célula de atlas: os pixels
/// dele agora são função de `base × luz`, e um atlas é justamente o lugar onde pixels são
/// compartilhados. É a mesma conversão que Trim / Make Square / Bg Removal fazem, pela mesma porta.
///
/// ⚠️ **E ele ganha um [`BakedFormId`]** — a identidade estável sem a qual o save não teria a quem
/// devolver os canais: os bits da entidade são id de ALOCAÇÃO e morrem no restore.
#[allow(clippy::too_many_arguments)]
fn bake_one(
    scene: &mut Sculpt3dScene,
    forms: &mut BTreeMap<u64, BakedForm>,
    pass: &mut Option<ImpastoLightPass>,
    next_id: &mut u32,
    gpu: &GpuContext,
    entity_bits: u64,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
) -> Result<(u32, u32), String> {
    let entity = Entity::from_bits(entity_bits);
    // ⚠️ **RE-ASSAR NÃO LÊ A TELA DE VOLTA.** Depois do primeiro bake os pixels do sprite são
    // `base × luz`; lê-los como fonte faria o segundo bake acender o que já está aceso, e o objeto
    // escureceria a cada gesto — a composição que o `base` existe para impedir. Um sprite já assado
    // reusa o `base` **e o slot**: nenhuma textura nova por bake.
    let previous = forms.get(&entity_bits).map(|b| (b.size, b.texture_id));
    let (size, base, texture_id) = match previous {
        Some((size, texture_id)) => {
            let base = forms[&entity_bits].base.clone();
            (size, base, texture_id)
        }
        None => {
            let src = crate::hero_intents::texture_edit::read_sprite_source(
                entity,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
            )
            .ok_or_else(|| "nao consegui ler os pixels do sprite selecionado".to_string())?;
            // ⚠️ **Alpha DIREITO.** O passe multiplica a cor pela luz, e num buffer pré-multiplicado
            // a multiplicação aconteceria sobre `cor × alpha` — o resultado escureceria pela borda
            // do recorte, que é a assinatura clássica de tratar premultiplicado como direto.
            let straight = src.image.into_straight();
            let size = (straight.width, straight.height);
            if size.0 == 0 || size.1 == 0 {
                return Err("o sprite selecionado nao tem pixels".into());
            }
            let id = renderer.acquire_individual_empty(size.0, size.1);
            (size, straight.pixels, id)
        }
    };
    let planes = scene
        .form_plane_for(gpu, size)
        .ok_or_else(|| "a cena nao tem malha para doar".to_string())?;
    let rig = *scene.rig();
    let bake = BakedForm {
        size,
        base,
        form: planes.normal,
        form_occ: planes.occlusion,
        texture_id,
        rig,
        lit_with: None,
    };
    light(gpu, renderer, pass, &rig, &bake)?;
    // Só DEPOIS de a luz ter chegado ao slot: apontar o sprite para uma textura vazia e falhar
    // deixaria o objeto invisível, que é pior que o gesto não ter acontecido.
    if let Some(mut sprite) = sim.world_mut().get_mut::<ph2d_render::Sprite>(entity) {
        sprite.source = ph2d_render::SpriteSource::Individual { texture_id };
        // O passe devolve alpha direto, como recebeu.
        sprite.premultiplied = false;
    }
    // A identidade estável, carimbada no MESMO gesto que cria os canais — um objeto assado sem ela
    // é um objeto que o save não sabe devolver.
    stamp_identity(sim, entity, next_id);
    forms.insert(
        entity_bits,
        BakedForm {
            lit_with: Some(rig_stamp(&rig)),
            ..bake
        },
    );
    Ok(size)
}

/// Carimba (ou preserva) o [`BakedFormId`] deste sprite e devolve o id.
///
/// ⚠️ **Re-assar NÃO cunha id novo.** O documento é keyed pelo id, e trocá-lo a cada gesto deixaria
/// no arquivo um documento órfão por bake — invisível, e crescendo.
fn stamp_identity(sim: &mut SimWorld, entity: Entity, next_id: &mut u32) -> u32 {
    let world = sim.world_mut();
    if let Some(id) = world.get::<BakedFormId>(entity) {
        return id.0;
    }
    let id = *next_id;
    *next_id = next_id.saturating_add(1);
    if let Ok(mut e) = world.get_entity_mut(entity) {
        e.insert(BakedFormId(id));
    }
    id
}

/// **A PORTA do gesto** — assa o que o artista pediu.
///
/// ⚠️ **A RE-ACENDIDA não está aqui**, e a ausência é o desenho: ela é
/// [`crate::baked_form::relight_stale`], roda por frame **sem a feature**, e é o que faz um objeto
/// reaberto acender.
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain(
    scene: &mut Sculpt3dScene,
    forms: &mut BTreeMap<u64, BakedForm>,
    pass: &mut Option<ImpastoLightPass>,
    next_id: &mut u32,
    gpu: &GpuContext,
    want_bake: bool,
    selected: Option<u64>,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
) -> Option<String> {
    if !want_bake {
        return None;
    }
    Some(match selected {
        Some(bits) => match bake_one(
            scene,
            forms,
            pass,
            next_id,
            gpu,
            bits,
            sim,
            renderer,
            asset_db,
            atlas_asset_map,
        ) {
            Ok((w, h)) => format!(
                "[sculpt3d] ASSADO no sprite ({w}x{h}) -- mova a lampada (Q/E/R/F) e ele RE-ACENDE; \
                 apague a peca que ele continua aceso, e SALVE que ele volta aceso"
            ),
            Err(e) => format!("[sculpt3d] nao assou: {e}"),
        },
        None => "[sculpt3d] nao assou: selecione um SPRITE antes (a forma acende ELE)".into(),
    })
}

/// **ENQUANTO A CENA EXISTE, A LUZ DELA AUTORA** — empurra o rig vivo para os objetos assados.
///
/// ⚠️ Sem isto, mover a lâmpada com `Q/E/R/F` deixaria os objetos assados com a luz de quando foram
/// assados: eles guardam uma **cópia** do rig justamente porque sobrevivem à cena, e uma cópia que
/// ninguém re-autora é uma cópia que congela. É a metade que o *"um documento, uma iluminação"*
/// (`docs/3D/05.2`) exige enquanto os dois existem.
///
/// ⚠️ **Ele não ACENDE nada** — só re-autora. Quem acende é a
/// [`crate::baked_form::relight_stale`], que vê o carimbo divergir e faz o trabalho; separar as duas
/// é o que deixa a acendida fora da feature.
pub(crate) fn follow_live_rig(forms: &mut BTreeMap<u64, BakedForm>, rig: &ph2d_light::LightRig) {
    for b in forms.values_mut() {
        b.rig = *rig;
    }
}

/// **As duas luzes, medidas e pinadas.** Módulo irmão, e não parte de um `mod tests`, porque tudo o
/// que mora nele precisa de um adapter de GPU — e porque ele carrega a medição que refutou o report
/// do aro, ao lado do gate que a torna executável.
#[cfg(test)]
#[path = "sculpt3d_bake_light.rs"]
mod light_measure;

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::SimWorld;

    /// **RE-ASSAR NÃO CUNHA IDENTIDADE NOVA.**
    ///
    /// ⚠️ O modo de falha de errar isto é silencioso e cresce: o documento é keyed pelo id, então um
    /// id novo por gesto deixa no arquivo um documento órfão por bake — ninguém vê, e o projeto
    /// engorda em planos do tamanho do sprite.
    #[test]
    fn re_baking_the_same_sprite_keeps_its_identity() {
        let mut sim = SimWorld::new();
        let e = sim.world_mut().spawn(()).id();
        let mut next = 0u32;

        let first = stamp_identity(&mut sim, e, &mut next);
        assert_eq!(first, 0, "o primeiro objeto assado leva o id 0");
        assert_eq!(next, 1, "e o proximo livre anda");

        let again = stamp_identity(&mut sim, e, &mut next);
        assert_eq!(again, first, "re-assar reusa a identidade");
        assert_eq!(next, 1, "e NAO consome um id novo");

        let other = sim.world_mut().spawn(()).id();
        assert_eq!(
            stamp_identity(&mut sim, other, &mut next),
            1,
            "outro sprite leva o proximo id"
        );
    }
}
