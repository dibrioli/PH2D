//! **Onde um texto assenta** — a porta única que resolve *"este texto é reto, ou cavalga
//! alguma coisa?"*.
//!
//! Módulo irmão de [`crate::vec_text_object`] (o objeto e o re-cook) e de [`crate::vec_glyph`]
//! (o layout). A pergunta é feita **num sítio** porque quem RE-COZINHA, quem decide se o
//! caminho-guia é PINTADO e quem um dia desenhar as alças têm de dar a mesma resposta — duas
//! portas divergem no dia em que uma delas ganha um cuidado e a outra não.
//!
//! # O espaço: um texto vinculado é MUNDO, e vive na IDENTIDADE
//!
//! O caminho-guia é um `VecPath` normal — geometria LOCAL + pose no `Transform` (ADR-0111).
//! Para o texto o cavalgar, o caminho é lido **cozido e assado em MUNDO** (o mesmo passo 1 do
//! [`crate::envelope_live`]), e o texto que sai daí já é geometria de mundo.
//!
//! Logo o `Transform` do texto tem de ser a **identidade**: uma pose por cima aplicaria a
//! transformação duas vezes. Isso não é efeito colateral a tolerar — é o desenho, e o
//! `connector_live` já o escreveu por nós:
//!
//! > *"Ele vive na identidade, e é isso que o torna (corretamente) não-arrastável pelo gizmo:
//! > arrastar um conector não quer dizer nada."*
//!
//! **Mover um texto em caminho não quer dizer nada — o que se move é o caminho.** É o que o
//! Illustrator faz (lá o texto e o caminho são literalmente um objeto só), e é o que o
//! `settle_origins` já respeita sem saber que existe (ele pula toda entidade com `VecShape`).

use ph2d_ecs::{Entity, SimWorld, Transform, VecTextPath};
use ph2d_vec_scene::VecScene;
use ph2d_vec_scene::arc_path::ArcPath;

use crate::vec_entities::VecEntityMap;
use crate::vec_glyph::TextPlacement;

/// O raio da bolinha da alça, em px de tela — o mesmo alcance para o desenho e o hit-test, pela
/// conversão `× px_to_world` na fronteira. **Maior** que as alças de nó (`ENVELOPE_HANDLE_R_PX`
/// 6.0, `connector::HANDLE_R_PX` 7.0): a alça de uma relação é uma ficha grande, não um ponto de
/// geometria, e espelha o `text_handle::HANDLE_R_PX` do renderer (os dois têm de concordar).
pub(crate) const HANDLE_R_PX: f64 = 10.0; // LITERAL-PX-OK: raio de alça, medida de INTERAÇÃO

/// O caminho-guia + o vínculo que o escolheu, prontos para serem cavalgados.
pub(crate) struct Guide {
    arc: ArcPath,
    link: VecTextPath,
}

/// O caminho-guia de um texto, já **cozido e em MUNDO**.
///
/// `None` — e o texto cai no layout reto — em três casos, todos honestos e todos silenciosos
/// de propósito: a entidade **não tem vínculo** · o caminho-guia **foi apagado** (o id fica
/// pendurado, e um texto que perde a curva tem de voltar a ser texto, não sumir) · o caminho é
/// **degenerado** (menos de dois vértices, ou comprimento zero).
#[must_use]
pub(crate) fn guide_of(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    entity: Entity,
) -> Option<Guide> {
    let link = *sim.world().get::<VecTextPath>(entity)?;
    let src = scene.paths().iter().find(|p| p.id == link.path)?;
    // A APARÊNCIA do caminho — `cooked()` resolve o raio de quina (sem raio ele empresta a
    // fonte, custo zero). Ler a fonte crua faria o texto ignorar as Live Corners do guia.
    let mut world = src.cooked().into_owned();
    // ...assada pela pose de MUNDO do guia. Sem isto o texto assentaria onde o caminho NÃO
    // está — e mover o caminho deixaria de mover o texto, que é a metade visível da feature.
    let pose = map
        .get(&link.path)
        .map(|&bits| Entity::from_bits(bits))
        .filter(|&e| sim.world().get_entity(e).is_ok())
        .map_or_else(Transform::default, |e| {
            crate::vec_transform::world_transform(sim, e)
        });
    ph2d_vec_scene::bake_xform(&mut world, &crate::vec_transform::xform_of_transform(pose));
    let arc = ArcPath::from_contour(&world.verts, world.closed)?;
    (arc.total() > 0.0).then_some(Guide { arc, link })
}

impl Guide {
    /// O assentamento que o layout consome.
    ///
    /// ⚠️ O `start_offset` é guardado como **fração** e vira comprimento **aqui**. A conversão
    /// mora nesta única porta de propósito: um número que metade do código lê como fração e a
    /// outra metade como distância é o bug que não dá erro em lado nenhum.
    #[must_use]
    pub(crate) fn placement(&self) -> TextPlacement<'_> {
        TextPlacement::OnPath {
            path: &self.arc,
            start_offset: f64::from(self.link.start_offset) * self.arc.total(),
            flip: self.link.flip,
        }
    }

    /// O ponto de MUNDO onde a alça de arrasto assenta — onde o texto **começa** no caminho.
    ///
    /// É o ponto do arco em `start_offset` (a mesma fração×total que o `placement` usa, pela
    /// mesma porta): a alça marca de onde a palavra parte, que é o que o `start_offset` diz. O
    /// ponto é o alvo do hit-test e o que o overlay desenha — perguntá-lo aqui, e não recomputá-lo
    /// nos dois sítios, é o que impede a alça de ser desenhada num sítio e agarrada noutro.
    #[must_use]
    pub(crate) fn handle_world(&self) -> [f64; 2] {
        let s = f64::from(self.link.start_offset) * self.arc.total();
        self.arc.frame_at(s).0
    }

    /// A fração de arco do ponto de mundo `p` mais próximo do caminho — o que um arrasto da alça
    /// escreve em `start_offset`. `NaN`-safe (o [`ArcPath::closest_arc`] devolve sempre um arco
    /// válido; a divisão pelo total é guardada em [`Self`], que só nasce com total > 0).
    #[must_use]
    pub(crate) fn frac_at_world(&self, p: [f64; 2]) -> f32 {
        (self.arc.closest_arc(p) / self.arc.total()) as f32
    }
}

#[cfg(test)]
#[path = "vec_text_ride_tests.rs"]
mod tests;

/// **O par que o gesto de prender exige: um TEXTO e um caminho que não seja ele.**
///
/// Porta única — quem PUBLICA se o botão aparece e quem HONRA o clique perguntam a esta
/// mesma função. Perguntar em dois sítios é como nasce o botão que aparece e recusa.
///
/// `None` quando a seleção não é exatamente isso: sem texto, sem segunda forma, ou com mais de
/// duas. Duas é o número certo e não uma limitação — *"prender este texto àquela curva"* nomeia
/// exatamente dois objetos, e três não diz qual delas é a curva.
#[must_use]
pub(crate) fn link_candidate(
    sim: &SimWorld,
    map: &VecEntityMap,
    selection: &[ph2d_vec_scene::VecPathId],
) -> Option<(ph2d_vec_scene::VecPathId, Entity, ph2d_vec_scene::VecPathId)> {
    if selection.len() != 2 {
        return None;
    }
    let (text, entity, _) = crate::vec_text_object::selected_text_object(sim, map, selection)?;
    let guide = *selection.iter().find(|&&id| id != text)?;
    // O guia não pode ser ele mesmo um texto: prender texto a texto não quer dizer nada, e o
    // `find` acima escolheria arbitrariamente qual dos dois manda.
    let guide_is_text = map
        .get(&guide)
        .map(|&b| Entity::from_bits(b))
        .and_then(|e| sim.world().get::<ph2d_ecs::VecShape>(e))
        .is_some_and(|s| matches!(s, ph2d_ecs::VecShape::Text(_)));
    (!guide_is_text).then_some((text, entity, guide))
}

/// Prende o texto da seleção ao outro objeto dela. `true` se prendeu.
pub(crate) fn link(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    selection: &[ph2d_vec_scene::VecPathId],
) -> bool {
    let Some((text, entity, guide)) = link_candidate(sim, map, selection) else {
        return false;
    };
    sim.world_mut().entity_mut(entity).insert(VecTextPath {
        path: guide,
        start_offset: 0.0,
        flip: false,
    });
    recook(sim, scene, map, text, entity);
    true
}

/// Solta o texto da seleção do caminho dele. `true` se soltou.
///
/// O caminho FICA. Soltar é remover o componente — nada é destruído, e é isso que torna
/// prender uma porta de duas mãos (o argumento do `Release` do Envelope e do Blend).
pub(crate) fn detach(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    selection: &[ph2d_vec_scene::VecPathId],
) -> bool {
    let Some((text, entity, _)) = crate::vec_text_object::selected_text_object(sim, map, selection)
    else {
        return false;
    };
    if sim.world().get::<VecTextPath>(entity).is_none() {
        return false;
    }
    sim.world_mut().entity_mut(entity).remove::<VecTextPath>();
    recook(sim, scene, map, text, entity);
    true
}

/// Edita o vínculo do texto da seleção (offset, lado) e re-cozinha. `true` se havia vínculo.
pub(crate) fn edit(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    selection: &[ph2d_vec_scene::VecPathId],
    f: impl FnOnce(&mut VecTextPath),
) -> bool {
    let Some((text, entity, _)) = crate::vec_text_object::selected_text_object(sim, map, selection)
    else {
        return false;
    };
    let Some(mut link) = sim.world().get::<VecTextPath>(entity).copied() else {
        return false;
    };
    f(&mut link);
    sim.world_mut().entity_mut(entity).insert(link);
    recook(sim, scene, map, text, entity);
    true
}

/// O vínculo VIVO do texto da seleção, para o painel publicar os controles no lugar certo.
#[must_use]
pub(crate) fn current(
    sim: &SimWorld,
    map: &VecEntityMap,
    selection: &[ph2d_vec_scene::VecPathId],
) -> Option<VecTextPath> {
    let (_, entity, _) = crate::vec_text_object::selected_text_object(sim, map, selection)?;
    sim.world().get::<VecTextPath>(entity).copied()
}

/// Re-cozinha o texto com os params que ele já tem — o vínculo mudou, o texto não.
///
/// Passa pela porta de sempre (`recook_text_object`), que é quem sabe resolver o guia e impor
/// a identidade. Um re-cook próprio aqui seria a segunda resposta a *"como um texto vira
/// geometria"*, e divergiria no dia em que uma delas ganhasse um cuidado.
fn recook(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    text: ph2d_vec_scene::VecPathId,
    entity: Entity,
) {
    if let Some(ph2d_ecs::VecShape::Text(params)) =
        sim.world().get::<ph2d_ecs::VecShape>(entity).cloned()
    {
        crate::vec_text_object::recook_text_object(sim, scene, map, text, entity, params);
    }
}

/// Um comando de vínculo vindo do painel — um clique, um comando.
#[derive(Clone, Copy, Debug)]
pub(crate) enum TextPathCmd {
    /// Prender o texto da seleção à outra forma dela.
    Link,
    /// Soltar (o caminho fica).
    Detach,
    /// Trocar o lado.
    Flip(bool),
}

/// A ALÇA de arrasto do texto em caminho: onde ela está, e como um arrasto a move (W5).
///
/// # Uma alça, e não três
///
/// O plano (22 §5.3) pedia *in / center / out*, copiando os três colchetes do Illustrator. Duas
/// dessas três não existem no nosso modelo, e uma alça que não faz nada é pior que uma que falta:
///
/// - **out** é a extensão do *container* de texto — o colchete onde o texto para e transborda.
///   **Não temos container:** o texto flui livre pela curva, sem recorte. A alça não teria o que
///   mover.
/// - **center-flip** vira o texto quando arrastado *através* da curva. É espacial, mas fiddly: ao
///   ajustar o offset o artista cruzaria a linha por acidente e o texto piscaria de lado. O lado
///   é uma escolha explícita, e o toggle **This side / Other side** do painel a faz sem
///   ambiguidade.
/// - **in** é onde o texto *começa* — o `start_offset`. Espacial, e a única das três que é uma
///   posição a arrastar. É esta.
///
/// # No modo SELECT, grande e colorida (Enio, smoke)
///
/// ⚠️ Nasceu no Node e pequena, e ali se confundia com as âncoras dos outros paths (que são
/// anéis pequenos). Mudou para o **Select** — onde não há âncoras, e o gizmo de sprite é inócuo
/// sobre um texto vinculado (ele vive na identidade) — e virou uma **ficha grande e sólida**
/// (`text_handle::draw_text_handle`), que lê como a alça primária de uma relação, não como um
/// ponto de geometria.
///
/// # Não é uma segunda porta para o Offset — é a MESMA
///
/// O painel já tem o slider de Offset. Isto **não** o duplica no sentido ruim: é o mesmo número
/// (`start_offset`), editado de dois modos, e ambos resolvem pela MESMA `edit(…, |l| l.start_offset)`
/// — a regra das duas-portas exige que concordem, e concordam por construção. É o precedente das
/// alças de gradiente e do gizmo de Transform: para uma grandeza **espacial**, arrastar na tela é
/// uma interação legitimamente diferente de um slider, não um segundo modelo dela.
pub(crate) mod handle {
    use super::{Guide, VecEntityMap, VecScene, guide_of};
    use ph2d_ecs::SimWorld;
    use ph2d_vec_scene::VecPathId;

    /// O ponto de MUNDO da alça do texto selecionado — `None` se não há texto vinculado na
    /// seleção. O overlay o desenha e o hit-test o compara ao cursor: a MESMA fonte, senão a
    /// alça seria pintada num sítio e agarrada noutro.
    #[must_use]
    pub(crate) fn world(
        sim: &SimWorld,
        scene: &VecScene,
        map: &VecEntityMap,
        selection: &[VecPathId],
    ) -> Option<[f64; 2]> {
        selected_guide(sim, scene, map, selection).map(|g| g.handle_world())
    }

    /// **Pressão sobre a alça (modo Select):** se a alça está sob o cursor, arma o arrasto e
    /// devolve `true` — o host então PULA o picking/gizmo. `radius` é o alcance em mundo (o raio
    /// da bolinha em px × `px_to_world`), a MESMA conversão que o desenho usa, para o dedo e a
    /// tela concordarem.
    #[must_use]
    pub(crate) fn press(
        sim: &SimWorld,
        scene: &VecScene,
        map: &VecEntityMap,
        selection: &[VecPathId],
        world_pt: [f64; 2],
        radius: f64,
        armed: &mut bool,
    ) -> bool {
        let Some(h) = world(sim, scene, map, selection) else {
            return false;
        };
        let hit = (h[0] - world_pt[0]).hypot(h[1] - world_pt[1]) <= radius;
        if hit {
            *armed = true;
        }
        hit
    }

    /// Arrasta a alça para o cursor: projeta o ponto no caminho, converte em fração e escreve
    /// `start_offset`. No-op (`false`) sem arrasto armado — a disciplina do `vec_pen_drag_move`.
    ///
    /// Passa pela porta `edit`, a MESMA do slider de Offset: a alça e o slider escrevem o mesmo
    /// número pela mesma função, então não podem divergir.
    pub(crate) fn drag(
        sim: &mut SimWorld,
        scene: &mut VecScene,
        map: &VecEntityMap,
        selection: &[VecPathId],
        world_pt: [f64; 2],
        armed: bool,
    ) -> bool {
        if !armed {
            return false;
        }
        let Some(frac) =
            selected_guide(sim, scene, map, selection).map(|g| g.frac_at_world(world_pt))
        else {
            return false;
        };
        super::edit(sim, scene, map, selection, |l| l.start_offset = frac)
    }

    /// O [`Guide`] do texto vinculado da seleção — o guia cozido e em mundo, pronto para a alça.
    fn selected_guide(
        sim: &SimWorld,
        scene: &VecScene,
        map: &VecEntityMap,
        selection: &[VecPathId],
    ) -> Option<Guide> {
        let (_, entity, _) = crate::vec_text_object::selected_text_object(sim, map, selection)?;
        guide_of(sim, scene, map, entity)
    }
}
