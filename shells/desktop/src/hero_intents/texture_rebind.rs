//! **A CAUDA de "esta sprite passou a ter pixels próprios"** — e a pergunta que ela faz a cada
//! chamador.
//!
//! ⚠️ **Irmão de [`super::texture_edit`] por CAP de LOC** (HR-18, 600): dar um nome à pergunta da
//! janela de amostragem levou aquele ficheiro a 669. *Cortar para o irmão é a cura.*
//!
//! O corte também é por responsabilidade: o `texture_edit.rs` é o **funil de leitura e commit** das
//! ferramentas de imagem; aqui ficam as invariantes de **re-alojamento** — o que muda no `Sprite`
//! quando os bytes dele passam a ter outro dono, e as duas coisas que morrem nessa passagem (a
//! janela de amostragem, às vezes; a autoria de folha, sempre).

use ph2d_ecs::{Entity, SimWorld};
use ph2d_render::{Sprite, SpriteSource};

/// **O que acontece à janela de amostragem (`Sprite::region_enabled`) quando os pixels são
/// re-ligados.** A resposta depende de os bytes novos serem *outra imagem* ou *a mesma imagem*.
///
/// ⚠️ **Isto é um parâmetro NOMEADO e não um `bool`** porque a resposta certa é diferente para
/// cada chamador, e um `true` solto no sítio da chamada não diz de quê. Foi exatamente esta
/// pergunta que a porta partilhada respondeu errado durante um dia inteiro.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum SamplingWindow {
    /// **Os bytes novos são OUTRA imagem** — uma edição de ferramenta. Se o sprite era uma região
    /// de uma folha, o que subiu é a imagem inteira dele, e amostrá-la pela janela antiga mostraria
    /// um recorte arbitrário dessa imagem nova. A janela morre.
    Dies,
    /// **Os bytes novos são A MESMA imagem** noutra precisão — a conversão 8↔16. O
    /// `region_rect` é expresso em **pixels da fonte** e a conversão preserva conteúdo *e*
    /// dimensões (`acquire_individual_16(width, height, …)` usa as do próprio asset), por isso a
    /// janela continua a apontar exatamente para os mesmos pixels. Ela sobrevive.
    Survives,
}

/// **A cauda do [`commit_edited_texture`] — as invariantes de "esta sprite passou a ter pixels
/// próprios", num sítio só.**
///
/// ⚠️ Extraída em 2026-08-20 (plano `docs/Sprite_projeto/18` W5) porque a conversão de precisão
/// precisa **quase** exatamente delas e escrevê-las outra vez seria pedir que as duas cópias
/// concordassem para sempre. Duas delas só falham **depois de fechar e reabrir o projeto**, que é o
/// pior sítio para descobrir uma divergência.
///
/// # ⚠️ A invariante que NÃO era comum, e o que ela custou
///
/// A extração trouxe junto `region_enabled = false`, que é a resposta certa para o chamador
/// original (edição de ferramenta) e **errada** para a conversão de precisão: esta sobe a *mesma*
/// imagem, e apagava o recorte do artista sem aviso — contra o contrato escrito no próprio
/// `precision_convert.rs` (*«`8 → 16 → 8` … tem de devolver a mesma sprite»*). Encontrado pela
/// auditoria de 2026-08-21 ([`docs/Sprite_projeto/20`](../../../../docs/Sprite_projeto/20_auditoria_do_inspector_2026-08-21.md) §4.1).
///
/// **A lei:** *uma porta partilhada por dois chamadores herda a regra de um deles.* O que é comum
/// fica no corpo; o que difere entra pela porta com **nome** — daí o [`SamplingWindow`].
pub(crate) fn rebind_to_individual(
    entity: Entity,
    sim: &mut SimWorld,
    texture_id: u32,
    pixels_id: ph2d_asset::AssetId,
    new_size_world: [f32; 2],
    premultiplied: bool,
    window: SamplingWindow,
) {
    // ⭐⭐ **OS PIXELS SÃO DA RECEITA, e por isso a edição sobe até ela** (Enio, 2026-08-26).
    // Ver [`write_through_targets`]: sem esta linha o artista pinta uma cópia e as irmãs ficam
    // como estavam, que foi exatamente o report.
    for target in write_through_targets(sim, entity) {
        rebind_one(
            target,
            sim,
            texture_id,
            pixels_id,
            new_size_world,
            premultiplied,
            window,
        );
    }
}

/// ⭐⭐ **A cadeia que uma edição de pixels percorre**: a entidade tocada, e — se ela é peça de uma
/// **instância** — a peça do MESTRE de que nasceu, e assim por diante.
///
/// # ⭐⭐⭐ A subida é do modo LIGADO, e só dele (Enio, 2026-08-27)
///
/// > *«No modelo Blender há os dois modos: Duplicate e Duplicate Linked.»*
///
/// A cadeia só sobe enquanto a peça tem [`ph2d_ecs::LinkedArt`] — o `Alt+D`. Sem a marca, pintar
/// uma cópia é uma edição **dela**, o passe de sync lê *«só a instância mexeu»* e captura um
/// override, que é o que *Instantiate* promete.
///
/// # ⚠️ Das duas razões originais, uma MORREU e ninguém reconferiu a nota
///
/// Este bloco dizia (F4.5) que a subida valia para **toda** cópia, por duas razões:
///
/// 1. **Uma imagem é um ASSET, não uma propriedade** — em todo motor 2D pintar a textura muda quem
///    a usa. ⭐ Continua verdade, e é exactamente o que **dividir a arte** quer dizer: virou o modo,
///    em vez de ser uma lei do app.
/// 2. ⛔ **«A receita está ESCONDIDA, então pintá-la não é alcançável por gesto nenhum»** — era
///    verdade na F4.5 e **deixou de o ser na F4.6**: escolher a linha da receita põe-na na tela
///    (`MasterEditing`), e o artista pinta-a directamente. *Quem move o número que tornava algo
///    inalcançável tem de reconferir a nota.*
///
/// ⛔⛔ E enquanto valia para todas, ela era **metade de uma incoerência**: a tinta de uma cópia
/// subia e a **geometria vetorial da mesma cópia** não (`instance_sync_docs` capturava override),
/// sem nada na tela a explicar a diferença. Hoje as duas leem a mesma marca.
///
/// ⚠️ **Não é um override, e é por construção:** ao escrever no mestre o passe seguinte lê *«o
/// mestre mexeu-se»* e leva os bytes a **todas** as instâncias; a que foi pintada já os tem, logo
/// `want == have` e ela não é reescrita. O ponto fixo do sync fica intacto.
///
/// ⛔ **A FRONTEIRA, nomeada:** para pintar UMA cópia ligada diferente das outras, *Detach from
/// Master* primeiro — ou tê-la criado com *Instantiate* em vez de *Instantiate Linked*.
///
/// ⚠️ **Sem entidade repetida**, e a guarda não é um tecto numérico: um elo corrompido que
/// apontasse para trás daria um laço infinito dentro de um commit de ferramenta, e um número
/// máximo de saltos transformaria isso numa contagem que ninguém sabe explicar.
fn write_through_targets(sim: &mut SimWorld, entity: Entity) -> Vec<Entity> {
    let by_id: std::collections::BTreeMap<u64, Entity> = {
        let mut q = sim.world_mut().query::<(Entity, &ph2d_ecs::StableId)>();
        q.iter(sim.world()).map(|(e, s)| (s.0, e)).collect()
    };
    let mut out = vec![entity];
    let mut cur = entity;
    // ⭐⭐⭐ **A subida é do modo LIGADO, e só dele** (Enio, 2026-08-27).
    //
    // ⚠️ Uma das duas razões desta subida MORREU e ninguém reconferiu a nota: *«a receita está
    // escondida, então pintá-la não é alcançável por gesto nenhum»* era verdade na F4.5 e deixou
    // de o ser na F4.6 — hoje escolher a linha da receita põe-na na tela, e o artista pinta-a
    // directamente. Sobra a razão 1 (*uma imagem é um asset*), e ela não é uma lei do app: é o que
    // uma cópia **LIGADA** quer dizer. *Quem move o número que tornava algo inalcançável tem de
    // reconferir a nota.*
    //
    // ⛔ Sem a marca, a subida punha a tinta de UMA cópia em TODAS — e a geometria vetorial da
    // mesma cópia fazia o contrário, no mesmo objeto, sem nada na tela a explicar a diferença.
    while sim.world().get::<ph2d_ecs::LinkedArt>(cur).is_some()
        && let Some(link) = sim.world().get::<ph2d_ecs::InstanceOf>(cur).copied()
        && let Some(&up) = by_id.get(&link.master)
        && !out.contains(&up)
    {
        out.push(up);
        cur = up;
    }
    out
}

/// As invariantes de re-alojamento aplicadas a UMA entidade — o corpo de sempre.
fn rebind_one(
    entity: Entity,
    sim: &mut SimWorld,
    texture_id: u32,
    pixels_id: ph2d_asset::AssetId,
    new_size_world: [f32; 2],
    premultiplied: bool,
    window: SamplingWindow,
) {
    if let Some(mut sprite) = sim.world_mut().get_mut::<Sprite>(entity) {
        sprite.source = SpriteSource::Individual { texture_id };
        sprite.size = new_size_world;
        sprite.premultiplied = premultiplied;
    }
    if window == SamplingWindow::Dies {
        // ⚠️ **A janela morre RETIRANDO o componente** (ADR-0164 F1 passo 6). Antes isto punha
        // `region_enabled = false` e deixava o `region_rect` autorado ao lado, de propósito —
        // hoje esse par deixou de ser exprimível, e «não há região» diz-se ausentando-a.
        sim.world_mut()
            .entity_mut(entity)
            .remove::<ph2d_ecs::SpriteRegion>();
    }
    // Stamped AFTER the sprite write and unconditionally: an entity whose `Sprite` vanished
    // mid-frame gets no stamp because `insert` on a dead entity is the caller's bug, so guard on
    // the same lookup the write used.
    if sim.world().get::<Sprite>(entity).is_some() {
        sim.world_mut()
            .entity_mut(entity)
            .insert(ph2d_ecs::SpritePixels(pixels_id));
        // ⚠️ **E a AUTORIA morre com ela.** `SpriteSheetRef` diz *"os meus pixels são a região R da
        // folha F"*, e isso deixou de ser verdade: os pixels agora são próprios, com nome durável
        // (o `SpritePixels` acima). Deixá-lo faria o `restore_sprite_sheets` re-ligar o sprite à
        // folha no load seguinte e **apagar a edição** — o defeito só apareceria depois de fechar
        // e reabrir o projeto, que é o pior sítio para o descobrir.
        drop_sheet_authorship(entity, sim);
    }
}

/// **A sprite deixou de ser uma região de uma folha — a autoria tem de morrer junto.**
///
/// `SpriteSheetRef` diz *"os meus pixels são a região R da folha F"*. Assim que os pixels passam a
/// ter outro dono — carimbo próprio (`SpritePixels`) ou uma célula do atlas partilhado — a
/// afirmação fica falsa, e deixá-la faz o `restore_sprite_sheets`
/// ([`crate::project_sprite_pixels`], que corre **incondicionalmente** para toda entidade que
/// carregue o componente) re-ligar a sprite à folha no load seguinte e **apagar a edição**.
///
/// ⚠️ **O defeito só aparece depois de fechar e reabrir o projeto** — o pior sítio para o
/// descobrir. É por isso que isto é uma PORTA e não uma linha repetida: em 2026-08-21 a auditoria
/// encontrou o `demote_to_atlas` (o caminho Individual → Atlas) sem ela, enquanto o caminho oposto
/// a tinha desde o primeiro dia. *Uma invariante que dois sítios têm de lembrar é uma invariante
/// que um deles vai esquecer.*
pub(crate) fn drop_sheet_authorship(entity: Entity, sim: &mut SimWorld) {
    sim.world_mut()
        .entity_mut(entity)
        .remove::<ph2d_ecs::SpriteSheetRef>();
}

#[cfg(test)]
mod write_through_tests {
    use super::{SamplingWindow, rebind_to_individual, write_through_targets};
    use ph2d_ecs::{Entity, InstanceOf, SimWorld, StableId};
    use ph2d_render::Sprite;

    /// Uma peça de instância e a peça da receita de que ela nasceu.
    ///
    /// ⚠️ **`linked` é o argumento porque a lei tem DOIS lados desde 2026-08-27** — uma fixtura que
    /// só soubesse construir um deles é uma fixtura sem o fenómeno para o outro.
    fn a_copy_and_its_recipe(sim: &mut SimWorld, linked: bool) -> (Entity, Entity) {
        let recipe = sim
            .world_mut()
            .spawn((Sprite::individual(1, [1.0, 1.0], [1.0; 4]), StableId(42)))
            .id();
        let copy = sim
            .world_mut()
            .spawn((
                Sprite::individual(2, [1.0, 1.0], [1.0; 4]),
                StableId(43),
                InstanceOf { master: 42 },
            ))
            .id();
        if linked {
            sim.world_mut().entity_mut(copy).insert(ph2d_ecs::LinkedArt);
        }
        (copy, recipe)
    }

    /// ⭐⭐⭐ **Pintar uma cópia LIGADA escreve na RECEITA** — e o passe leva-a às irmãs.
    ///
    /// ⚠️ **Este gate afirmava isto de TODA cópia** (F4.5, com a razão *«uma imagem é um asset»*),
    /// e era metade de uma incoerência que o Enio nomeou em 2026-08-27: a tinta subia e a
    /// geometria vetorial da MESMA cópia não. Hoje as duas leem a mesma marca, e quem escolhe é o
    /// artista no gesto (*Instantiate* contra *Instantiate Linked*). O irmão
    /// [`painting_an_unlinked_copy_keeps_it_to_itself`] guarda o outro lado.
    ///
    /// (Mutação: devolver `vec![entity]` em `write_through_targets` ⇒ RED.)
    #[test]
    fn painting_a_linked_copy_writes_through_to_the_recipe() {
        let mut sim = SimWorld::new();
        let (copy, recipe) = a_copy_and_its_recipe(&mut sim, true);
        let pixels = ph2d_asset::AssetId::from_bytes(b"tinta");
        rebind_to_individual(
            copy,
            &mut sim,
            9,
            pixels,
            [2.0, 2.0],
            false,
            SamplingWindow::Dies,
        );
        for (who, e) in [("a copia", copy), ("a receita", recipe)] {
            assert_eq!(
                sim.world().get::<ph2d_ecs::SpritePixels>(e).map(|p| p.0),
                Some(pixels),
                "{who} ficou sem o nome duravel dos pixels"
            );
            assert_eq!(
                sim.world().get::<Sprite>(e).map(|s| s.size),
                Some([2.0, 2.0]),
                "{who} ficou com o tamanho antigo"
            );
        }
    }

    /// ⭐⭐⭐ **E uma cópia NÃO ligada pinta-se sozinha** — o outro lado da mesma lei.
    ///
    /// É o `Shift+D`: *Instantiate* dá arte própria, e o que o artista pinta nela é dela. A receita
    /// fica intacta, e o passe de sync captura a excepção pela porta de sempre.
    ///
    /// ⚠️ **Os dois lados no mesmo módulo, de propósito.** Enquanto só existia o lado de cima, a
    /// única forma de ter uma cópia com tinta própria era *Detach* — soltá-la da receita para
    /// sempre —, e a geometria vetorial da mesma cópia já fazia o contrário sem ninguém ter
    /// escolhido.
    ///
    /// (Mutação: subir a cadeia sem olhar ao `LinkedArt` ⇒ RED.)
    #[test]
    fn painting_an_unlinked_copy_keeps_it_to_itself() {
        let mut sim = SimWorld::new();
        let (copy, recipe) = a_copy_and_its_recipe(&mut sim, false);
        rebind_to_individual(
            copy,
            &mut sim,
            9,
            ph2d_asset::AssetId::from_bytes(b"so' desta copia"),
            [2.0, 2.0],
            false,
            SamplingWindow::Dies,
        );
        assert!(
            sim.world().get::<ph2d_ecs::SpritePixels>(recipe).is_none(),
            "uma copia com arte PROPRIA escreveu na receita — o *Instantiate* deixou de se \
             distinguir do *Instantiate Linked*"
        );
        // Controlo POSITIVO: a cópia recebeu mesmo a tinta, senão o gate passaria sobre um no-op.
        assert!(
            sim.world().get::<ph2d_ecs::SpritePixels>(copy).is_some(),
            "a copia tambem nao foi pintada — o gate acima nao mede nada"
        );
    }

    /// ⛔ **A FRONTEIRA, nomeada: uma cópia DESTACADA pinta-se sozinha.**
    ///
    /// *Destacar* apaga o `InstanceOf`, e é isso — e só isso — que faz a cadeia parar. É a resposta
    /// a *«e se eu quiser esta cópia diferente?»*.
    ///
    /// (Mutação: subir a cadeia sem olhar ao `InstanceOf` ⇒ RED.)
    #[test]
    fn a_detached_copy_paints_only_itself() {
        let mut sim = SimWorld::new();
        let (copy, recipe) = a_copy_and_its_recipe(&mut sim, true);
        sim.world_mut().entity_mut(copy).remove::<InstanceOf>();
        rebind_to_individual(
            copy,
            &mut sim,
            9,
            ph2d_asset::AssetId::from_bytes(b"so' minha"),
            [2.0, 2.0],
            false,
            SamplingWindow::Dies,
        );
        assert!(
            sim.world().get::<ph2d_ecs::SpritePixels>(recipe).is_none(),
            "uma copia destacada escreveu na receita — o Detach deixou de significar alguma coisa"
        );
    }

    /// ⚠️ **Um elo que aponta para trás não faz a cadeia rodar para sempre** — a guarda é *«sem
    /// entidade repetida»*, e não um tecto de saltos.
    ///
    /// (Mutação: apagar o `!out.contains(&up)` ⇒ o teste nunca termina.)
    #[test]
    fn a_link_that_points_back_does_not_loop() {
        let mut sim = SimWorld::new();
        let (copy, recipe) = a_copy_and_its_recipe(&mut sim, true);
        // A receita a dizer-se cópia da cópia — só um ficheiro corrompido faz isto.
        //
        // ⚠️ **E LIGADA também**, senão a travessia parava nela por falta da marca e a guarda de
        // repetição não chegava a ser exercida: a mutação que a apaga passaria. *Uma condição nova
        // a montante pode tornar inobservável a metade que o gate existe para medir.*
        sim.world_mut()
            .entity_mut(recipe)
            .insert((InstanceOf { master: 43 }, ph2d_ecs::LinkedArt));
        let chain = write_through_targets(&mut sim, copy);
        assert_eq!(chain, vec![copy, recipe], "a cadeia repetiu uma entidade");
    }
}

#[cfg(test)]
mod sampling_window_tests {
    use super::*;

    /// Uma sprite `Individual` com janela de amostragem autorada e ligada.
    fn sprite_with_a_live_window(sim: &mut SimWorld) -> (Entity, [f32; 4]) {
        let rect = [8.0, 8.0, 64.0, 64.0];
        let sprite = Sprite::individual(7, [1.0, 1.0], [1.0; 4]);
        // ⭐ A janela é um componente, e a PRESENÇA dele é o antigo `region_enabled`.
        let e = sim
            .world_mut()
            .spawn((sprite, ph2d_ecs::SpriteRegion::individual(rect)))
            .id();
        (e, rect)
    }

    fn rebind(sim: &mut SimWorld, e: Entity, window: SamplingWindow) {
        rebind_to_individual(
            e,
            sim,
            42,
            ph2d_asset::AssetId::from_bytes(&[0u8; 32]),
            [1.0, 1.0],
            false,
            window,
        );
    }

    /// **A conversão de precisão PRESERVA o recorte** — o contrato que
    /// `precision_convert.rs` declara por escrito: *«`8 → 16 → 8` … tem de devolver a mesma
    /// sprite»*.
    ///
    /// ⚠️ Este é o gate do defeito de 2026-08-21: a cauda partilhada apagava
    /// `region_enabled` para os dois chamadores, porque a extração trouxe junto a regra do
    /// chamador original. A sprite de imagem própria perdia o recorte do artista **sem aviso**,
    /// e o toast dizia apenas «Format · RGBA16».
    #[test]
    fn a_precision_swap_keeps_the_authored_sampling_window() {
        let mut sim = SimWorld::default();
        let (e, rect) = sprite_with_a_live_window(&mut sim);
        rebind(&mut sim, e, SamplingWindow::Survives);
        let after = sim.world().get::<ph2d_ecs::SpriteRegion>(e).copied();
        assert!(
            after.is_some(),
            "a troca de precisao apagou a janela de amostragem — ela sobe A MESMA imagem noutra \
             precisao, entao o recorte continua a apontar para os mesmos pixels"
        );
        assert_eq!(
            after.expect("a janela").rect,
            rect,
            "a janela sobreviveu mas mudou de sitio"
        );
    }

    /// **Uma edição de ferramenta MATA o recorte** — e este é o controlo positivo que impede a
    /// cura de virar «nunca apagar».
    ///
    /// ⚠️ Sem ele, trocar o argumento por `Survives` em toda parte passaria despercebido — e aí a
    /// peça de folha voltaria a amostrar a imagem nova por uma janela que já não descreve nada,
    /// que é o bug das «múltiplas repetições» de 2026-08-19.
    #[test]
    fn a_tool_edit_still_kills_the_window() {
        let mut sim = SimWorld::default();
        let (e, _) = sprite_with_a_live_window(&mut sim);
        rebind(&mut sim, e, SamplingWindow::Dies);
        assert!(
            sim.world().get::<ph2d_ecs::SpriteRegion>(e).is_none(),
            "a edicao de ferramenta manteve a janela — os pixels novos sao OUTRA imagem, e a \
             janela antiga recortaria um pedaco arbitrario dela"
        );
    }

    /// **As duas respostas são de facto diferentes.** Um `enum` de duas variantes que o corpo
    /// ignorasse deixaria os dois testes acima verdes por acidente.
    #[test]
    fn the_two_answers_are_not_the_same_answer() {
        let mut sim = SimWorld::default();
        let (a, _) = sprite_with_a_live_window(&mut sim);
        let (b, _) = sprite_with_a_live_window(&mut sim);
        rebind(&mut sim, a, SamplingWindow::Survives);
        rebind(&mut sim, b, SamplingWindow::Dies);
        let ra = sim.world().get::<ph2d_ecs::SpriteRegion>(a).is_some();
        let rb = sim.world().get::<ph2d_ecs::SpriteRegion>(b).is_some();
        assert_ne!(
            ra, rb,
            "`SamplingWindow` nao muda nada — o parametro existe e o corpo nao o le"
        );
    }
}
