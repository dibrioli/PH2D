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
}

#[cfg(test)]
#[path = "vec_text_ride_tests.rs"]
mod tests;
