//! O gesto de **arrastar os cantos da gaiola** do Envelope (ADR-0129, Fatia 1) — o lado do HOST.
//!
//! A parte pura (que canto, e até onde) mora em `ph2d_vec_envelope::{nearest_corner,
//! move_corner_convex}`; aqui é o adaptador fino que lê a seleção, o componente ECS [`VecEnvelope`]
//! e o cursor, e ESCREVE `corners` de volta no componente. É o padrão do [`crate::blend_live`] (um
//! gesto de Node que toca o ECS), **não** o do `PenTool` (que só conhece o `VecScene`) — porque a
//! gaiola vive num componente, não como âncora de um path da cena.
//!
//! A alça é PRÓPRIA e vive no modo Node (ADR-0129 §3.3): o gizmo de sprite não a toca. Um gizmo
//! sobre a geometria de mundo que o [`crate::envelope_live::recook`] reescreve a cada frame dobraria
//! — a lição de 5 tentativas revertidas do Blend (ADR-0128).
//!
//! # Undo sai de graça
//!
//! O arrasto roda com o botão pressionado, e `App::post_frame_undo` suprime passos enquanto
//! `held_button` está `Some`. Os N frames do arrasto não viram N passos; ao soltar, o [`VecEnvelope`]
//! alterado (que viaja no `WorldSnapshot`) vira **um** passo no diff global. Nada a instrumentar aqui.

use ph2d_ecs::{Entity, SimWorld, VecEnvelope};
use ph2d_vec_render::{ENVELOPE_HANDLE_R_PX, EnvelopeCageView};
use ph2d_vec_scene::{VecPathId, Xform};

use crate::vec_entities::VecEntityMap;

/// Os 4 cantos da gaiola do path `id`, em coordenadas **LOCAIS** (como vivem no componente) —
/// `None` se ele não é um envelope (ou sumiu). Quem os leva ao MUNDO é [`path_world_xform`].
#[must_use]
pub(crate) fn corners_of(
    sim: &SimWorld,
    map: &VecEntityMap,
    id: VecPathId,
) -> Option<[[f64; 2]; 4]> {
    let bits = *map.get(&id)?;
    sim.world()
        .get::<VecEnvelope>(Entity::from_bits(bits))
        .map(|env| env.corners)
}

/// O afim LOCAL→MUNDO da entidade do path `id` — a MESMA pose que `vec_transform::build` publica
/// (ADR-0111), calculada por-entidade aqui para não depender de o `VecXforms` do frame já estar na
/// mão. É por ele que a gaiola (cantos LOCAIS) se desenha e se hit-testa no MUNDO, e é essa pose que
/// o gizmo do Select move (Fatia 2). `None` se a entidade sumiu.
fn path_world_xform(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<Xform> {
    let bits = *map.get(&id)?;
    let entity = Entity::from_bits(bits);
    Some(crate::vec_transform::xform_of_transform(
        crate::vec_transform::world_transform(sim, entity),
    ))
}

/// Os 4 cantos LOCAIS levados ao MUNDO pela pose.
#[must_use]
fn to_world(local: [[f64; 2]; 4], xf: &Xform) -> [[f64; 2]; 4] {
    std::array::from_fn(|i| xf.apply(local[i]))
}

/// **Pressão no modo Node:** se a forma selecionada é um envelope e um canto da gaiola está sob o
/// cursor, arma o arrasto (`*drag = Some((id, canto))`) e devolve `true` — o host então PULA o
/// `PenTool`. Fora disso devolve `false` e o pen segue como hoje (seleção / edição de âncora).
///
/// Hit-test no MUNDO: os cantos LOCAIS sobem pela pose ([`path_world_xform`]) e o cursor (mundo) é
/// comparado a eles. `px_to_world` converte o raio da bolinha (px, do renderer) para o alcance em
/// mundo — a MESMA constante que o desenho usa, para o dedo e a tela concordarem.
#[must_use]
pub(crate) fn press(
    sim: &SimWorld,
    map: &VecEntityMap,
    selected: Option<VecPathId>,
    world_pt: [f64; 2],
    px_to_world: f64,
    drag: &mut Option<(VecPathId, usize)>,
) -> bool {
    let Some(id) = selected else { return false };
    let Some(local) = corners_of(sim, map, id) else {
        return false;
    };
    let Some(xf) = path_world_xform(sim, map, id) else {
        return false;
    };
    let world = to_world(local, &xf);
    let radius = ENVELOPE_HANDLE_R_PX * px_to_world;
    match ph2d_vec_envelope::nearest_corner(&world, world_pt, radius) {
        Some(corner) => {
            *drag = Some((id, corner));
            true
        }
        None => false,
    }
}

/// **Move durante o arrasto:** leva o canto agarrado para `world_pt`, mas só se a gaiola continuar
/// convexa ([`ph2d_vec_envelope::move_corner_convex`]); não-convexo ⇒ o canto **para na fronteira**
/// (os cantos não mudam neste frame; o §5 mantém o horizonte fora da gaiola). Devolve `true`
/// enquanto há um arrasto vivo — o host consome o Move —, tenha o canto andado ou não.
///
/// O cursor está em MUNDO e a gaiola vive em LOCAL: o ponto desce pela pose INVERSA antes do
/// `move_corner_convex`, então mover um canto sob pose girada/escalada segue o dedo. Convexidade é
/// invariante a afim, logo checá-la em local é o mesmo que em mundo. Pose não-invertível: consome o
/// Move e não escreve.
#[must_use]
pub(crate) fn drag(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    active: Option<(VecPathId, usize)>,
    world_pt: [f64; 2],
) -> bool {
    let Some((id, corner)) = active else {
        return false;
    };
    let Some(&bits) = map.get(&id) else {
        return true; // arrasto vivo, mas a entidade sumiu: consome e espera o release
    };
    let entity = Entity::from_bits(bits);
    let (Some(xf), Some(local)) = (
        path_world_xform(sim, map, id),
        sim.world()
            .get::<VecEnvelope>(entity)
            .map(|env| env.corners),
    ) else {
        return true;
    };
    let Some(inv) = xf.inverse() else {
        return true; // pose degenerada: nada a mover com sentido
    };
    let local_pt = inv.apply(world_pt);
    if let Some(next) = ph2d_vec_envelope::move_corner_convex(local, corner, local_pt)
        && let Some(mut env) = sim.world_mut().get_mut::<VecEnvelope>(entity)
    {
        env.corners = next;
    }
    true
}

/// A gaiola a desenhar neste frame, se a forma selecionada é um envelope — os cantos já em MUNDO
/// (cantos LOCAIS levados pela pose). O canto sob arrasto (se pertencer à seleção) sai marcado
/// `dragging` — a bolinha cheia.
#[must_use]
pub(crate) fn view(
    sim: &SimWorld,
    map: &VecEntityMap,
    selected: Option<VecPathId>,
    active: Option<(VecPathId, usize)>,
) -> Option<EnvelopeCageView> {
    let id = selected?;
    let local = corners_of(sim, map, id)?;
    let xf = path_world_xform(sim, map, id)?;
    let dragging = active.filter(|(d, _)| *d == id).map(|(_, c)| c);
    Some(EnvelopeCageView {
        corners: to_world(local, &xf),
        dragging,
    })
}

#[cfg(test)]
#[path = "envelope_gesture_tests.rs"]
mod tests;
