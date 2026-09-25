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
    // ⭐⭐⭐⭐ **ONDE A ARTE DESTE SPRITE ESTÁ NO ECRÃ**, `[x, y, w, h]` em pixels de janela — o
    // que faz o assado cair onde o artista pôs o objecto. Ver
    // [`Sculpt3dScene::enquadramento_do_sprite`]: quem sabe isto é a shell (a câmera 2D e o afim
    // partilhado são dela), quem sabe onde a vista 3D está é esta crate, e nenhuma das duas sabe
    // as duas coisas. `None` = a vista inteira, o comportamento de antes.
    rect_do_sprite: Option<[f32; 4]>,
) -> Result<(u32, u32, usize), String> {
    let entity = Entity::from_bits(entity_bits);
    // ⚠️ **RE-ASSAR NÃO LÊ A TELA DE VOLTA**, e essa lei mudou de casa em 2026-09-21: ela passou a
    // ter um SEGUNDO leitor — o visor, que desde a mesma data pinta a MESMA matéria que este gesto
    // vai acender ([`super::albedo`], onde o `61×` que a obrigou está medido). Depois do primeiro
    // bake os pixels do sprite são `base × luz`; lê-los como fonte faria o segundo bake acender o
    // que já está aceso, e o objeto escureceria a cada gesto.
    let (mut base, size) =
        super::albedo::materia_para(forms, entity_bits, &mut || ler_fonte(sim, renderer))?;
    // ⚠️ **O SLOT é do bake e não da matéria** — um sprite já assado reusa a textura que já tem
    // (nenhuma textura nova por bake); o visor não tem slot nenhum, e é por isso que esta linha
    // ficou aqui em vez de viajar com a lei.
    let texture_id = match forms.get(&entity_bits) {
        Some(b) => b.texture_id,
        None => renderer.acquire_individual_empty(size.0, size.1),
    };
    // ⭐⭐ **RE-ASSAR PRESERVA A LEI DA PEÇA** — ela é uma escolha do ARTISTA sobre este objecto
    // (`BakedForm::lei`), como o slot acima, e não uma consequência do gesto. Sem esta linha um
    // `Shift+B` devolvia toda peça ao valor de fábrica **em silêncio**, que é o defeito que o
    // campo gravado existe para impedir: *o artista escolheria a lei uma vez e perdê-la-ia no
    // gesto seguinte*.
    let lei = lei_ao_assar(forms.get(&entity_bits));
    // ⚠️ **O enquadramento é derivado DEPOIS de a matéria dizer o `size`**, e a ordem é a mesma
    // razão que já prende a silhueta lá abaixo: o `size` é a extensão em que a forma é pedida.
    let recorte = scene
        .enquadramento_do_sprite(rect_do_sprite)
        .map(super::recorte::a_guardar);
    let planes = scene
        .form_plane_for(gpu, size, super::recorte::a_usar(recorte, size))
        .ok_or_else(|| ph2d_i18n::tr("app.sculpt3d.bake.sem_malha_para_doar").to_string())?;
    // ⭐⭐⭐⭐ **UM SPRITE VAZIO VESTE A SILHUETA DA PEÇA** (report do dono, 21/09) — ver
    // [`super::albedo::veste_a_forma`], onde a lei, a cerca e o *porquê do branco* estão escritos.
    //
    // ⚠️ **Ela mora AQUI e não no [`super::albedo::materia_para`], e a ordem é a razão:** a
    // silhueta que falta é a **cobertura** do G-buffer, que só existe depois do `form_plane_for`
    // logo acima. A matéria é lida antes porque ela decide o TAMANHO com que a forma é
    // rasterizada — *não há como perguntar à forma antes de saber em que extensão a pedir*.
    //
    // ⚠️ **E ela escreve no `base`, que é o que fica GRAVADO** — logo re-assar é estável: o
    // segundo `Shift+B` reusa esta matéria já vestida e não a compõe outra vez.
    let vestidos =
        super::albedo::veste_a_forma(&mut base, super::albedo::Cobertura::DaForma(&planes.normal));
    let rig = *scene.rig();
    let bake = BakedForm {
        size,
        base,
        form: planes.normal,
        form_occ: planes.occlusion,
        texture_id,
        rig,
        lit_with: None,
        lei,
        // ⭐⭐⭐⭐ **A resposta vem do VESTIR, e é o mesmo gesto que a decide** — ver o doc do campo:
        // se o sprite chegou sem arte, o que ele mostra é a peça 3D, logo a silhueta dele tem de
        // ser re-derivada por quadro em vez de ficar congelada neste bake.
        materia_da_forma: vestidos > 0,
        // ⚠️ **CONGELADO aqui**, e é isso que faz o catavento não saltar: a rota B re-rasteriza
        // por quadro e tem de reproduzir o enquadramento deste gesto, nunca o do ecrã de agora —
        // senão a peça deslizava dentro do sprite ao arrastar o canvas 2D.
        recorte,
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
    // ⭐⭐⭐⭐ **E O OBJECTO PASSA A SER VIRÁVEL** (report do dono, 21/09: *«ao assar com a sprite
    // transparente, não aparece no Inspector os controlos da sprite 3D»*).
    //
    // ⚠️ A secção `Live Mesh` só é pintada COM o componente (ADR-0166), e até aqui só a cena `=53`
    // o semeava — logo o artista que assava a peça dele tinha a forma 3D no objecto e **nenhuma
    // superfície para a virar**. *Um motor com a lei certa e o artista sem lhe chegar lê-se, da
    // cadeira dele, como um motor sem a lei.*
    //
    // ⭐ **E é barato:** o componente nasce em `yaw = pitch = spin = 0`, que é a identidade da rota
    // B **ao bit** — quem não quiser virar nada não vê diferença nenhuma no que desenha.
    //
    // ⚠️ **Só na PRIMEIRA vez** (`get` antes de `insert`): re-assar não pode devolver a pose ao
    // zero, senão um `Shift+B` apagava o giro que o artista tinha posto — o mesmo argumento do
    // `lei_ao_assar` logo acima, e do slot.
    marca_como_viravel(sim, entity);
    forms.insert(
        entity_bits,
        BakedForm {
            lit_with: Some(rig_stamp(&rig)),
            ..bake
        },
    );
    Ok((size.0, size.1, vestidos))
}

/// ⭐⭐ **COM QUE LEI ESTE BAKE FICA** — a do objecto, se ele já existe; a de fábrica, se nasce.
///
/// ⚠️ **Ela é uma função e não duas linhas lá em cima por uma razão desta casa:** o gesto de assar
/// pede um `GpuContext`, um `SpriteRenderer` e um mapa de atlas, logo *não é alcançável de um teste*
/// — e o que esta decisão afirma não tem um pixel dentro. Escrita lá dentro, a mutação que a troca
/// por `Lei::default()` passava a suíte inteira, e o sintoma seria o pior: *o artista escolhe a lei
/// uma vez e perde-a no `Shift+B` seguinte*, em silêncio.
///
/// ⛔ **Re-assar NÃO é escolher** — é a mesma lei do `texture_id` logo acima dela: o slot e a lei
/// são propriedades do OBJECTO, e o gesto reusa as duas.
fn lei_ao_assar(anterior: Option<&BakedForm>) -> ph2d_form_donation::lei_da_luz::Lei {
    anterior.map_or_else(ph2d_form_donation::lei_da_luz::Lei::default, |b| b.lei)
}

#[cfg(test)]
mod lei_ao_assar_tests {
    use ph2d_form_donation::lei_da_luz::Lei;

    /// ⭐ **As DUAS metades, porque as curas são opostas** — e a fixtura escolhe a lei que NÃO é a
    /// de fábrica, senão *«preservou»* e *«nasceu de novo»* leem-se igual.
    #[test]
    fn re_assar_preserva_a_lei_e_um_bake_novo_nasce_na_de_fabrica() {
        let anterior = ph2d_form_donation::baked_form::BakedForm {
            size: (1, 1),
            base: vec![0; 4],
            form: vec![0.0; 4],
            form_occ: Vec::new(),
            texture_id: 0,
            rig: ph2d_light::LightRig::default(),
            lit_with: None,
            lei: Lei::Tinta,
            materia_da_forma: false,
            recorte: None,
        };
        assert_ne!(
            Lei::Tinta,
            Lei::default(),
            "controlo: a fixtura tem de carregar a lei que NÃO é a de fábrica"
        );
        assert_eq!(
            super::lei_ao_assar(Some(&anterior)),
            Lei::Tinta,
            "re-assar preserva a escolha do artista — ela é do OBJECTO, não do gesto"
        );
        assert_eq!(
            super::lei_ao_assar(None),
            Lei::default(),
            "um objecto que nasce assado nasce na lei que o visor mostra"
        );
    }
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

/// **ASSAR TORNA O OBJECTO VIRÁVEL** — o [`ph2d_ecs::Mesh3D`] na primeira vez, e nunca depois.
///
/// ⚠️ Ver a chamada, no [`bake_one`]: o que isto compra é a **secção do Inspector**, e o que o
/// `get` primeiro protege é a pose que o artista já pôs.
pub(crate) fn marca_como_viravel(sim: &mut SimWorld, entity: Entity) {
    let world = sim.world_mut();
    if world.get::<ph2d_ecs::Mesh3D>(entity).is_some() {
        return;
    }
    if let Ok(mut e) = world.get_entity_mut(entity) {
        e.insert(ph2d_ecs::Mesh3D::default());
    }
}

/// ⭐⭐⭐ **O QUE O GESTO DIZ, E COM QUE CARA** — o veredito de [`drain`].
///
/// ⛔⛔ **Ele existe por causa de uma FOTO** (report do dono, 21/09): a recusa do bake aparecia na
/// tela com o ✓ **VERDE** de sucesso, porque a fase fazia `Toast::success(line)` sobre a `String`
/// que esta porta devolvia — *o chamador não tinha como saber se o gesto tinha corrido*.
/// ⚠️ Um aviso que anuncia uma falha com a cara de um sucesso é pior do que nenhum: ele ensina o
/// artista a não ler os avisos.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Veredito {
    /// O sprite ficou aceso pela forma.
    Assado(String),
    /// Nada foi assado, e a frase diz porquê (e, onde há cura, qual é).
    Recusado(String),
}

impl Veredito {
    /// A frase, para quem só a quer imprimir.
    #[must_use]
    pub fn frase(&self) -> &str {
        match self {
            Self::Assado(s) | Self::Recusado(s) => s,
        }
    }

    /// `true` quando o gesto correu — o que decide a CARA do aviso.
    #[must_use]
    pub fn assou(&self) -> bool {
        matches!(self, Self::Assado(_))
    }
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
    // Ver `bake_one`: o rectângulo que a arte do sprite ocupa na JANELA, ou `None` quando não há
    // quem o saiba (uma cena sem canvas publicado — todo gate headless desta casa).
    rect_do_sprite: Option<[f32; 4]>,
) -> Option<Veredito> {
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
        ph2d_i18n::tr("app.sculpt3d.bake.convertido_para_rgba8")
    } else {
        ""
    };
    Some(match selected {
        Some(bits) => Veredito::do_gesto(
            bake_one(
                scene,
                forms,
                passes,
                next_id,
                gpu,
                bits,
                sim,
                renderer,
                ler_fonte,
                rect_do_sprite,
            ),
            note,
        ),
        None => {
            Veredito::Recusado(ph2d_i18n::tr("app.sculpt3d.bake.selecione_um_sprite").to_string())
        }
    })
}

impl Veredito {
    /// ⭐⭐⭐ **A TRADUÇÃO do que o gesto devolveu para o que o artista vê** — pura, e é essa a
    /// razão de ela ser uma função.
    ///
    /// ⛔⛔ **Ela foi extraída por uma MUTAÇÃO SOBREVIVENTE** (21/09): trocar o
    /// `Err(e) => Veredito::Recusado(…)` por `Assado(…)` passava os **dois** gates de GPU do gesto
    /// e a suíte inteira — porque os dois só percorrem o caminho que ASSA, e **o braço do erro
    /// nunca era exercitado por ninguém**. ⚠️ *Um gate que só corre o caminho feliz não afirma
    /// nada sobre a cara de uma falha* — e a cara de uma falha era, exactamente, o report.
    ///
    /// ⭐ Escrita dentro do [`drain`] ela só era alcançável com um `GpuContext`, e a lei desta
    /// casa di-lo por extenso: *quando um gate precisa de um device para medir uma decisão que não
    /// tem pixel nenhum, a lei está no sítio errado*.
    ///
    /// **Mutações que devem sangrar:** `Err(…) => Assado` · `Ok((.., 0))` trocado com o braço do
    /// vestido · a chave da frase.
    pub(crate) fn do_gesto(feito: Result<(u32, u32, usize), String>, note: &str) -> Self {
        match feito {
            // ⭐⭐ **DUAS frases e não uma com um `if` dentro**: quando o sprite estava vazio o
            // que aconteceu foi outra coisa — ele **vestiu a peça** —, e o artista tem de o saber
            // para não pintar por cima sem perceber porquê (ver [`super::albedo::veste_a_forma`]).
            Ok((w, h, 0)) => Self::Assado(ph2d_i18n::tr_with(
                "app.sculpt3d.bake.assado",
                &[
                    ("w", &w.to_string()),
                    ("h", &h.to_string()),
                    ("note", &note),
                ],
            )),
            Ok((w, h, vestidos)) => Self::Assado(ph2d_i18n::tr_with(
                "app.sculpt3d.bake.assado_vestido",
                &[
                    ("w", &w.to_string()),
                    ("h", &h.to_string()),
                    ("note", &note),
                    ("vestidos", &vestidos.to_string()),
                ],
            )),
            Err(e) => Self::Recusado(ph2d_i18n::tr_with(
                "app.sculpt3d.bake.nao_assou",
                &[("e", &e)],
            )),
        }
    }
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
