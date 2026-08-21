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
    if let Some(mut sprite) = sim.world_mut().get_mut::<Sprite>(entity) {
        sprite.source = SpriteSource::Individual { texture_id };
        sprite.size = new_size_world;
        sprite.premultiplied = premultiplied;
        if window == SamplingWindow::Dies {
            // O `region_rect` fica como está de propósito (é ignorado enquanto `region_enabled` é
            // falso, e zerá-lo só acrescentaria uma escrita que ninguém lê).
            sprite.region_enabled = false;
        }
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
mod sampling_window_tests {
    use super::*;

    /// Uma sprite `Individual` com janela de amostragem autorada e ligada.
    fn sprite_with_a_live_window(sim: &mut SimWorld) -> (Entity, [f32; 4]) {
        let rect = [8.0, 8.0, 64.0, 64.0];
        let mut sprite = Sprite::individual(7, [1.0, 1.0], [1.0; 4]);
        sprite.region_enabled = true;
        sprite.region_rect = rect;
        let e = sim.world_mut().spawn((sprite,)).id();
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
        let after = sim.world().get::<Sprite>(e).copied().expect("sprite");
        assert!(
            after.region_enabled,
            "a troca de precisao apagou a janela de amostragem — ela sobe A MESMA imagem noutra \
             precisao, entao o recorte continua a apontar para os mesmos pixels"
        );
        assert_eq!(
            after.region_rect, rect,
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
        let after = sim.world().get::<Sprite>(e).copied().expect("sprite");
        assert!(
            !after.region_enabled,
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
        let ra = sim
            .world()
            .get::<Sprite>(a)
            .copied()
            .expect("a")
            .region_enabled;
        let rb = sim
            .world()
            .get::<Sprite>(b)
            .copied()
            .expect("b")
            .region_enabled;
        assert_ne!(
            ra, rb,
            "`SamplingWindow` nao muda nada — o parametro existe e o corpo nao o le"
        );
    }
}
