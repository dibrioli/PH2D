//! **AS ÂNCORAS, vivas** — a segunda metade do passe de layout (plano UI/UX W3).
//!
//! Módulo FILHO do [`super`], e a razão não é o teto de LOC: é a **lei da porta única**. O plano
//! diz que um filho de moldura *"ou está num fluxo ou está ancorado, nunca os dois"*, e um segundo
//! passe teria uma segunda opinião sobre a pose derivada. Aqui o dono da tabela de poses continua
//! a ser UM ([`super::LayoutLive`]), e este ficheiro é só o outro braço dele.
//!
//! # O que uma âncora responde, e o fluxo não
//!
//! O fluxo empilha: ele decide a posição de TODOS os filhos a partir de uma regra do pai. A âncora
//! é o oposto — o filho está onde o artista o pôs, e a regra diz apenas **o que acontece quando a
//! moldura muda de tamanho**. É o HUD: a vida no canto de cima, a pontuação colada no canto
//! oposto, a barra que estica no meio.
//!
//! # A régua é LOCAL, e o resto é uma razão
//!
//! [`ph2d_ecs::VecAnchors::delta_local`] devolve o deslocamento em unidades **locais** da moldura.
//! Para o levar ao mundo este passe usa a **razão entre a caixa de mundo e a caixa local da própria
//! moldura** — e essa razão traz consigo, de graça, tudo o que estiver entre as duas: a pose do
//! ancestral, a escala do `Transform`, e até a colocação que o fluxo acabou de dar à moldura, se
//! ela própria for um item de fluxo.
//!
//! ⚠️ **Ela é exacta enquanto o mapa local→mundo for alinhado aos eixos, e aproximada sob
//! ROTAÇÃO** — a caixa de mundo de um retângulo rodado é maior que ele. É a mesma aproximação
//! declarada que a caixa do gizmo carrega ([`crate::vec_gizmo_view::fold_layout_pose`]), e pela
//! mesma razão: o que se compara são caixas alinhadas aos eixos, e um retângulo rodado não é uma.
//!
//! # O neutro é EXACTO, não *"quase zero"*
//!
//! Moldura do tamanho da régua ⇒ `delta_local` dá `0.0` nos dois eixos por subtracção de iguais ⇒
//! a caixa-alvo é a caixa de agora ⇒ o afim é a identidade ⇒ o passe **nem paga a cópia da
//! geometria**. Um documento sem `VecAnchors` é byte-idêntico ao de antes desta wave; um COM
//! âncoras e sem redimensionamento também.

use ph2d_ecs::{Children, Entity, SimWorld, VecAnchors, VecFrame, VecLayout};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{VecPathId, VecScene, VecXforms, bake_xform, curve_bbox_in_frame};

use super::LayoutLive;
use crate::vec_entities::VecEntityMap;

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
        let paths = crate::vec_entities::subtree_paths(sim, scene, kid);
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
        let x = super::fit((lo, hi), target);
        if super::is_identity(&x) {
            return; // moldura do tamanho da régua: nada a fazer, nem a cópia
        }
        for &id in &paths {
            let mut items = super::world_of(scene, xforms, live, id);
            for p in &mut items {
                bake_xform(p, &x);
            }
            live.insert(id, items);
            self.add_pose(id, x);
        }
        self.anchored += 1;
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
