//! **A porta única de "onde está o caminho-guia, e como se percorre por arco?"**.
//!
//! Um caminho-guia é um `VecPath` normal — geometria LOCAL + pose no `Transform` (ADR-0111). Para
//! algo o *cavalgar* (o texto em caminho, o Pattern Along Path, e o que vier), ele é lido **cozido
//! e assado em MUNDO** e virado num [`ArcPath`]. Essa resolução tem três cuidados que discordar é
//! bug:
//!
//! 1. **cozido**, não a fonte crua — senão o que cavalga ignora as Live Corners e a pilha de
//!    efeitos do guia (ADR-0121/0132);
//! 2. **assado pela pose de mundo** do guia — senão o padrão assenta onde o caminho NÃO está, e
//!    mover o caminho deixa de mover o que o cavalga (a metade visível da feature);
//! 3. `total > 0` — um guia degenerado (menos de dois vértices, comprimento zero) não tem por onde
//!    ser percorrido, e quem divide pelo total tem de o saber.
//!
//! O texto e o pattern faziam (ou fariam) esta MESMA pergunta. Duas cópias divergem no dia em que
//! uma ganha um cuidado e a outra não — por isso mora aqui, e o `vec_text_ride::guide_of` delega.

use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_vec_scene::VecScene;
use ph2d_vec_scene::arc_path::ArcPath;

use crate::vec_entities::VecEntityMap;

/// O [`ArcPath`] do caminho-guia `guide_path_id`, **cozido e em MUNDO**.
///
/// `None` — e quem cavalga cai no comportamento reto/solto — em três casos honestos e silenciosos
/// de propósito: o caminho **não existe** na cena (id pendurado após apagar o guia) · o caminho é
/// **degenerado** (`from_contour` devolve `None`) · o comprimento é **zero**.
#[must_use]
pub(crate) fn guide_arc(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    guide_path_id: u64,
) -> Option<ArcPath> {
    let src = scene.paths().iter().find(|p| p.id == guide_path_id)?;
    // A APARÊNCIA do caminho: `cooked()` resolve o raio de quina (sem raio, empresta a fonte, custo
    // zero). Ler a fonte crua faria o padrão ignorar as Live Corners do guia.
    let mut world = src.cooked().into_owned();
    // ...assada pela pose de MUNDO do guia (ou identidade se a entidade sumiu).
    let pose = map
        .get(&guide_path_id)
        .map(|&bits| Entity::from_bits(bits))
        .filter(|&e| sim.world().get_entity(e).is_ok())
        .map_or_else(Transform::default, |e| {
            crate::vec_transform::world_transform(sim, e)
        });
    ph2d_vec_scene::bake_xform(&mut world, &crate::vec_transform::xform_of_transform(pose));
    let arc = ArcPath::from_contour(&world.verts, world.closed)?;
    (arc.total() > 0.0).then_some(arc)
}
