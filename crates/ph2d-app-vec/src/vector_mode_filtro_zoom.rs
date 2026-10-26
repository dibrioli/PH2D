//! ⭐ **A cena 41** (`PH2D_OBJECT_MODE_SMOKE=41`, F34 — achado do dono no smoke F33: zoom muito perto
//! numa forma com filtro = a forma some e o FPS cai). Uma Nuvem GIGANTE com o Bevel + Drop Shadow da
//! cena 40: no zoom de abertura ela já passa do lado de textura da placa (8192), que é o zoom perto
//! do dono sem precisar de mexer na câmera. Só a borda direita cabe na vista; ela tem de aparecer,
//! com relevo, e a roda (perto e longe) não pode travar. Plano: `docs/UI_New_and_Simple/F34_plano.md`.

use crate::state::VecState;
use ph2d_ecs::{FxOp, Name, SimWorld};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_vec_scene::{Paint, Rgba8, ShapeKind, VecPathId, VecScene};

/// O estágio — UM só, logo depois do da cena 40 (`45`) e antes da 12 (`50..`).
pub(crate) const FIRST: u8 = crate::vector_mode_filtro_efeito::LAST + 1;
pub(crate) const LAST: u8 = FIRST;
const _: () = assert!(LAST < 50);

const VERDE: Rgba8 = Rgba8::new(120, 190, 100, 255);
/// O tamanho e o centro (unidades da ferramenta de forma, que mede o RAIO): 12 000 px de lado a
/// 100 %, e a borda direita a +3 (300 px à direita do centro da vista). Medido na foto da F34.
const TAMANHO: f64 = 60.0;
const CENTRO: [f64; 2] = [3.0 - TAMANHO, 0.0];

/// Um quadro da cena 41; `Some(9)` = acabou.
pub fn step(
    vec: &mut VecState,
    scene: &mut VecScene,
    sim: &mut SimWorld,
    _hero: &mut HeroScreen,
) -> Option<u8> {
    if vec.entities.is_empty() {
        crate::vector_mode::draw_shape(vec, scene, ShapeKind::Cloud, CENTRO, TAMANHO);
        return None;
    }
    let ids: Vec<VecPathId> = vec.entities.keys().copied().collect();
    let id = *ids.first()?;
    let p = scene.path_mut(id)?;
    p.fill = Some(Paint::solid(VERDE));
    p.stroke = None;
    sim.world_mut()
        .entity_mut(ph2d_ecs::Entity::from_bits(*vec.entities.get(&id)?))
        .insert(Name::new(FxOp::spec(FxOp::BEVEL).name));
    let filtro = crate::vector_mode_filtro_efeito::filtro();
    ph2d_vec_entities::filter::set_filter(sim, &vec.entities, &ids, Some(filtro));
    eprintln!(
        "[cena 41] uma Nuvem de {TAMANHO} com Bevel + Drop Shadow, maior que a textura da placa"
    );
    Some(9)
}
