//! **AS ÂNCORAS, vivas** — a segunda metade do passe de layout (plano UI/UX W3), e o ARRANJO do HUD
//! depois delas (ADR-0194). Filho do [`super`] pela lei da porta única. O porquê (régua local, rotação
//! aproximada, neutro exacto): [`as_ancoras_vivas_porque.md`](../../../docs/UI_New_and_Simple/medicoes/as_ancoras_vivas_porque.md).

use ph2d_ecs::{Children, Entity, SimWorld, VecAnchors, VecFrame, VecLayout};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{VecPathId, VecScene, VecXforms, bake_xform, curve_bbox_in_frame};

use super::LayoutLive;
use ph2d_vec_entities::entities::VecEntityMap;

/// **A caixa LOCAL de uma moldura** — `[x0, y0, x1, y1]`, Y-up, sem pose nenhuma.
///
/// ⚠️ **Porta única, e ela é load-bearing.** Esta é a MESMA função que mede a régua na hora de
/// armar a regra (`vec_anchor_edit`) e que a lê na hora de a honrar (aqui). Duas medições da
/// *"caixa da moldura"* — uma que coza a forma viva e outra que não, por exemplo — poriam a
/// criança meio delta ao lado no primeiro redimensionamento, e nada na tela diria porquê
/// (`feedback_derived_coordinate_seed_must_match_sample`).
#[must_use]
pub(crate) fn frame_local_box(scene: &VecScene, id: VecPathId) -> Option<[f64; 4]> {
    let p = scene.paths().iter().find(|p| p.id == id)?;
    let (lo, hi) = curve_bbox_in_frame(&p.cooked(), 1.0, 0.0)?;
    Some([lo[0], lo[1], hi[0], hi[1]])
}

/// A moldura que ANCORA este filho — `None` se o pai não é moldura, ou se ele FLUI.
///
/// ⚠️ A recusa do fluxo é a lei do plano feita função: um filho de moldura ou está num fluxo ou
/// está ancorado. Ela é perguntada aqui (para HONRAR) e pelo `vec_anchor_edit` (para OFERECER a
/// seção) — se o painel oferecesse a regra onde o passe a ignora, o artista teria quatro chips que
/// acendem e não movem um pixel.
#[must_use]
pub(crate) fn anchoring_frame(sim: &SimWorld, kid: Entity) -> Option<Entity> {
    let w = sim.world();
    let parent = w.get::<ph2d_ecs::ChildOf>(kid)?.parent();
    w.get::<VecFrame>(parent)?;
    if w.get::<VecLayout>(parent).is_some() {
        return None; // o fluxo já o colocou; a âncora não tem voto
    }
    Some(parent)
}

/// **A moldura MEDIDA neste frame** — a caixa local (a régua contra a qual a regra compara) e a
/// razão que leva um deslocamento local à tela. Um valor só porque as duas nunca fazem sentido
/// separadas: a segunda existe para converter a primeira.
#[derive(Clone, Copy)]
struct Measured {
    now: [f64; 4],
    scale: [f64; 2],
}

/// A razão de uma dimensão de mundo para a local. Degenerada (local de tamanho zero) vira `1`:
/// esticar uma coisa sem tamanho não quer dizer nada, e mover uma continua legítimo.
fn ratio(world: f64, local: f64) -> f64 {
    if local.abs() > 1e-9 {
        world / local
    } else {
        1.0
    }
}

impl LayoutLive {
    /// **Todas as molduras que ancoram alguém.** Roda DEPOIS do fluxo: uma moldura ancorada dentro
    /// de um fluxo já foi colocada, e é a caixa colocada que os filhos dela têm de ler.
    pub(super) fn anchor_all(
        &mut self,
        scene: &VecScene,
        sim: &SimWorld,
        map: &VecEntityMap,
        xforms: &VecXforms,
        live: &mut LiveGeometry,
    ) {
        for frame in anchoring_frames(scene, sim, map) {
            self.anchor_frame(scene, sim, xforms, live, frame);
        }
    }

    /// Uma moldura: mede-se, e depois move cada filho ancorado pela regra dele.
    fn anchor_frame(
        &mut self,
        scene: &VecScene,
        sim: &SimWorld,
        xforms: &VecXforms,
        live: &mut LiveGeometry,
        frame: Entity,
    ) {
        let w = sim.world();
        // A moldura mede-se por SI (o retângulo vivo que ela é), nunca pela sub-árvore — senão o
        // filho que ela move entraria na medida que decide o quanto ele se move.
        let Some(own) = super::own_paths(sim, scene, frame) else {
            return;
        };
        let Some(&id) = own.first() else {
            return;
        };
        let Some(now) = frame_local_box(scene, id) else {
            return;
        };
        let Some((wlo, whi)) = super::bbox_of(&super::world_of_all(scene, xforms, live, &own))
        else {
            return;
        };
        let m = Measured {
            now,
            // Local -> mundo, por eixo. Ver o ⚠️ do cabeçalho sobre rotação.
            scale: [
                ratio(whi[0] - wlo[0], now[2] - now[0]),
                ratio(whi[1] - wlo[1], now[3] - now[1]),
            ],
        };
        let Some(kids) = w.get::<Children>(frame) else {
            return;
        };
        for &kid in kids.iter() {
            self.anchor_kid(scene, sim, xforms, live, kid, m);
        }
    }

    /// Um filho: a caixa que ele tem hoje, a caixa que a regra pede, e o afim entre as duas.
    ///
    /// ⚠️ A regra é lida AQUI, e não passada pelo chamador: quem não tem regra sai na primeira
    /// linha, e o laço de cima fica sendo só *"para cada filho"*. É também o que mantém a lista de
    /// argumentos honesta — a caixa e a razão viajam SEMPRE juntas (são a moldura medida NESTE
    /// frame), então elas são um valor só.
    fn anchor_kid(
        &mut self,
        scene: &VecScene,
        sim: &SimWorld,
        xforms: &VecXforms,
        live: &mut LiveGeometry,
        kid: Entity,
        m: Measured,
    ) {
        let Some(a) = sim.world().get::<VecAnchors>(kid) else {
            return;
        };
        let (now, s) = (m.now, m.scale);
        // A sub-árvore INTEIRA anda com ele: um grupo ancorado é uma coisa só.
        let paths = ph2d_vec_entities::entities::subtree_paths(sim, scene, kid);
        if paths.is_empty() {
            return;
        }
        let items = super::world_of_all(scene, xforms, live, &paths);
        let Some((lo, hi)) = super::bbox_of(&items) else {
            return;
        };
        let [dmin, dmax] = a.delta_local(now);
        let target = (
            [lo[0] + dmin[0] * s[0], lo[1] + dmin[1] * s[1]],
            [hi[0] + dmax[0] * s[0], hi[1] + dmax[1] * s[1]],
        );
        if self.move_subtree(scene, xforms, live, &paths, (lo, hi), target) {
            self.anchored += 1;
        }
    }

    /// A sub-árvore de `(lo, hi)` para `target`; `false` = identidade (moldura do tamanho da régua: nem a cópia).
    fn move_subtree(
        &mut self,
        scene: &VecScene,
        xforms: &VecXforms,
        live: &mut LiveGeometry,
        paths: &[VecPathId],
        from: super::Box2,
        target: super::Box2,
    ) -> bool {
        let x = super::fit(from, target);
        if super::is_identity(&x) {
            return false;
        }
        for &id in paths {
            let mut items = super::world_of(scene, xforms, live, id);
            for p in &mut items {
                bake_xform(p, &x);
            }
            live.insert(id, items);
            self.add_pose(id, x);
        }
        true
    }
}

/// **As molduras que ancoram alguém** — as que têm pelo menos um filho com regra que elas de facto
/// honram. Ordem estável entre frames (o `sort`), como a irmã do fluxo.
fn anchoring_frames(scene: &VecScene, sim: &SimWorld, map: &VecEntityMap) -> Vec<Entity> {
    let w = sim.world();
    let mut found: Vec<Entity> = Vec::new();
    for path in scene.paths() {
        let Some(&bits) = map.get(&path.id) else {
            continue;
        };
        let e = Entity::from_bits(bits);
        if w.get::<VecAnchors>(e).is_none() {
            continue;
        }
        if let Some(frame) = anchoring_frame(sim, e)
            && !found.contains(&frame)
        {
            found.push(frame);
        }
    }
    found.sort_unstable();
    found
}

#[cfg(test)]
#[path = "layout_live_anchors_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "layout_live_anchors_canvas_tests.rs"]
mod canvas_tests;

/// O canvas do HUD ancora pela MESMA lei do filho ([`LayoutLive::anchor_kid`]); a selecção mora em
/// [`ph2d_app_components::hud_anchors::ancorados`]. Porquê: o doc do cabeçalho.
impl LayoutLive {
    /// Todos os canvas de HUD que ancoram alguém (disjuntos das molduras: um filho tem UM pai).
    pub(super) fn anchor_canvases(
        &mut self,
        scene: &VecScene,
        sim: &SimWorld,
        map: &VecEntityMap,
        xforms: &VecXforms,
        live: &mut LiveGeometry,
    ) {
        for a in ph2d_app_components::hud_anchors::ancorados(sim, map, self.vista) {
            let m = Measured {
                now: a.now,
                scale: a.scale,
            };
            self.anchor_kid(scene, sim, xforms, live, a.kid, m);
        }
    }
}

/// O arranjo dos irmãos (ADR-0194): a SELECÇÃO e o solver moram em `ph2d_app_components::hud_arranjo`; aqui só se
/// publica a pose, pelo mesmo braço das âncoras.
impl LayoutLive {
    pub(super) fn arrange_canvases(
        &mut self,
        scene: &VecScene,
        sim: &SimWorld,
        map: &VecEntityMap,
        xforms: &VecXforms,
        live: &mut LiveGeometry,
    ) {
        let subtree = |kid| ph2d_vec_entities::entities::subtree_paths(sim, scene, kid);
        let caixa = |kid| super::bbox_of(&super::world_of_all(scene, xforms, live, &subtree(kid)));
        for m in self.arranjos.arranja(sim, map, self.vista, caixa) {
            self.move_subtree(scene, xforms, live, &subtree(m.kid), m.de, m.para);
        }
    }
}
