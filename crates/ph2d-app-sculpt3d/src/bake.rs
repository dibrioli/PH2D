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
//! rig, a acendida, a codificação com que ele viaja no arquivo — mora na [`ph2d_form_donation::baked_form`], que
//! **não está atrás da feature `sculpt3d`**.
//!
//! ⚠️ **Essa divisão É a rota A.** O `docs/3D/02.2` promete que o G-buffer é gerado uma vez, vira
//! canal do sprite, e a malha some do build; um módulo único deixaria a acendida cair junto com a
//! feature, e a promessa viraria prosa. O corte é literalmente *o que precisa da malha* × *o que
//! precisa só dos canais*.

use std::collections::BTreeMap;

use ph2d_ecs::{BakedForm as BakedFormId, Entity, SimWorld};
use ph2d_gpu::GpuContext;
use ph2d_render::SpriteRenderer;

use ph2d_form_donation::baked_form::{BakedForm, PassesDaLuz, light, rig_stamp};

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
    passes: &mut PassesDaLuz,
    next_id: &mut u32,
    gpu: &GpuContext,
    entity_bits: u64,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    // ⚠️⚠️ **O leitor da fonte chega por ASSINATURA, e é PREGUIÇOSO de propósito** — um sprite
    // já assado reúsa o `base` e nunca chega a perguntar, e a leitura custa uma volta ao
    // device. A porta que a faz (`hero_intents::texture_edit`) é da shell e ela é que sabe
    // ler um sprite; o que ESTA função precisa são os **pixels**, não a capacidade de os ir
    // buscar. ⭐ E foi só por causa dela que o `AssetDb` e o mapa de atlas estavam nesta
    // assinatura: nenhuma outra linha deste ficheiro os lê, e os dois saíram com ela.
    ler_fonte: &mut dyn FnMut(
        &mut SimWorld,
        &mut SpriteRenderer,
    ) -> Option<ph2d_render::SpriteImage>,
) -> Result<(u32, u32), String> {
    let entity = Entity::from_bits(entity_bits);
    // ⚠️ **RE-ASSAR NÃO LÊ A TELA DE VOLTA**, e essa lei mudou de casa em 2026-09-21: ela passou a
    // ter um SEGUNDO leitor — o visor, que desde a mesma data pinta a MESMA matéria que este gesto
    // vai acender ([`super::albedo`], onde o `61×` que a obrigou está medido). Depois do primeiro
    // bake os pixels do sprite são `base × luz`; lê-los como fonte faria o segundo bake acender o
    // que já está aceso, e o objeto escureceria a cada gesto.
    let (base, size) =
        super::albedo::materia_para(forms, entity_bits, &mut || ler_fonte(sim, renderer))?;
    // ⚠️ **O SLOT é do bake e não da matéria** — um sprite já assado reusa a textura que já tem
    // (nenhuma textura nova por bake); o visor não tem slot nenhum, e é por isso que esta linha
    // ficou aqui em vez de viajar com a lei.
    let texture_id = match forms.get(&entity_bits) {
        Some(b) => b.texture_id,
        None => renderer.acquire_individual_empty(size.0, size.1),
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
    light(gpu, renderer, passes, &rig, &bake)?;
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
/// [`ph2d_form_donation::baked_form::relight_stale`], roda por frame **sem a feature**, e é o que faz um objeto
/// reaberto acender.
#[allow(clippy::too_many_arguments)]
pub fn drain(
    scene: &mut Sculpt3dScene,
    forms: &mut BTreeMap<u64, BakedForm>,
    passes: &mut PassesDaLuz,
    next_id: &mut u32,
    gpu: &GpuContext,
    want_bake: bool,
    selected: Option<u64>,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    // ⚠️ **Um FACTO, colhido antes** — ver o corpo: ele tem de ser lido ANTES do bake, porque o
    // bake substitui a textura e depois já não há 16 bits que ver. A porta é
    // `hero_intents::texture_edit::holds_sixteen_bit`, da shell.
    lost_precision: bool,
    // Ver `bake_one`.
    ler_fonte: &mut dyn FnMut(
        &mut SimWorld,
        &mut SpriteRenderer,
    ) -> Option<ph2d_render::SpriteImage>,
) -> Option<String> {
    if !want_bake {
        return None;
    }
    // **A PRECISÃO QUE A DOAÇÃO CUSTA** (plano `docs/Sprite_projeto/18` W7).
    //
    // ⚠️ O `base` do objeto assado é `Vec<u8>` e o passe de luz escreve numa textura de 8 bits — a
    // conversão é **correcta** (o produto `base × luz` é aritmética sobre a cor), mas era **muda**.
    // Aqui ela custa mais que numa ferramenta: o sprite passa a ser função da luz, e a próxima
    // leitura já não devolve a imagem original.
    //
    // ⚠️ Lido ANTES do bake, porque o bake substitui a textura — depois já não há 16 bits que ver.
    // (A leitura é da shell desde a W2/L3-B; ver o parâmetro.)
    let note = if lost_precision {
        " -- convertido para RGBA8: a forma assada e' de 8 bits"
    } else {
        ""
    };
    Some(match selected {
        Some(bits) => match bake_one(
            scene, forms, passes, next_id, gpu, bits, sim, renderer, ler_fonte,
        ) {
            Ok((w, h)) => format!(
                "[sculpt3d] ASSADO no sprite ({w}x{h}){note} -- mova a lampada (Q/E/R/F) e ele \
                 RE-ACENDE; apague a peca que ele continua aceso, e SALVE que ele volta aceso"
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
/// [`ph2d_form_donation::baked_form::relight_stale`], que vê o carimbo divergir e faz o trabalho; separar as duas
/// é o que deixa a acendida fora da feature.
pub fn follow_live_rig(forms: &mut BTreeMap<u64, BakedForm>, rig: &ph2d_light::LightRig) {
    for b in forms.values_mut() {
        b.rig = *rig;
    }
}

impl Sculpt3dScene {
    /// **O artista MEXEU na lâmpada desde a última pergunta?**
    ///
    /// ⚠️ **É a diferença entre *a cena existe* e *o artista fez alguma coisa*, e ela vale a arte
    /// do documento.** O [`follow_live_rig`] re-autora o rig de TODO objeto assado, e o campo que
    /// ele sobrescreve é o rig **AUTORADO no bake** — cujo próprio doc-comment diz que ele viaja no
    /// arquivo justamente para *"reabrir o projeto não acender o objeto com o rig DEFAULT"*.
    ///
    /// Enquanto a cena 3D só nascia atrás de uma variável de ambiente, as duas frases não se
    /// encontravam. Com o pill SCULPT **um clique cria uma cena com o rig default**, e sem esta
    /// borda todo sprite já assado do documento era re-aceso com uma luz que ninguém escolheu —
    /// que é, da cadeira do artista, *"o Bake to Sprite disparando sozinho"* (Enio, 2026-08-09).
    ///
    /// ⚠️ **Testemunha, não enumeração:** quem move o rig são as quatro teclas de luz, o painel, e
    /// uma cena de smoke que traz rig próprio. Comparar carimbos cobre os três — e o quarto.
    pub fn take_rig_edge(&mut self) -> bool {
        let now = ph2d_form_donation::baked_form::rig_stamp(&self.rig);
        now != std::mem::replace(&mut self.rig_was, now)
    }
}

/// **As duas luzes, medidas e pinadas.** Módulo irmão, e não parte de um `mod tests`, porque tudo o
/// que mora nele precisa de um adapter de GPU — e porque ele carrega a medição que refutou o report
/// do aro, ao lado do gate que a torna executável.
#[cfg(test)]
#[path = "bake_light.rs"]
mod light_measure;

/// **O GESTO INTEIRO, num device de verdade** — ver o módulo. Irmão do
/// `light_measure` pela mesma razão: tudo o que mora nele precisa de um adapter.
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
