//! **O AUTO LAYOUT, vivo** — a moldura empilha os filhos, e as posições são derivadas por frame.
//!
//! O **8º produtor** de [`LiveGeometry`], e o **terceiro que TRANSFORMA o mapa** em vez de o
//! estender (os outros dois são o [`crate::align_live`] e o [`crate::bool_live`]). A razão é a
//! mesma: os cinco produtores que o `render_loop` funde com `extend` são mutuamente exclusivos —
//! uma forma é offset OU pattern OU contour —, e **o layout não é membro dessa família**: ele é um
//! componente do PAI, então cada filho que ele coloca pode ter, ele próprio, o seu offset vivo.
//! Fundido por `extend` ele apagaria esses offsets em silêncio.
//!
//! # A lei (ADR-0153)
//!
//! > **O passe publica ONDE as coisas ficam. Ele não escreve ONDE elas estão.**
//!
//! Nada aqui toca `Transform`. O undo deste editor é por DIFF do mundo ECS, então escrever a pose
//! derivada faria **cada frame de um redimensionamento virar um passo de undo** — e o layout
//! brigaria com o arrasto do artista dentro do mesmo frame.
//!
//! # A conversão de eixo é UMA, e é aqui
//!
//! O motor fala **CSS** (`y` para baixo, origem no canto superior-esquerdo da moldura); o documento
//! é **Y-up**. A troca acontece em [`world_target`] e em mais lugar nenhum — meia troca de eixo
//! espalhada por dois sítios é a forma de a moldura empilhar para cima num dia e para baixo noutro.
//!
//! # Posição e tamanho viajam pelo MESMO canal
//!
//! Um filho colocado ganha um afim `translate ∘ scale` aplicado à geometria de MUNDO dele **e à de
//! toda a sub-árvore dele** — é o que faz um grupo inteiro andar junto com a caixa que o contém.
//! O `scale` só é ≠ 1 quando o `grow`/`shrink` do filho de facto mudou o tamanho, e o neutro
//! (ninguém pediu nada) é a identidade: o mapa fica **intocado**, e o mundo pré-layout é
//! byte-idêntico.

use ph2d_ecs::{Entity, SimWorld, VecLayout, VecLayoutItem};
use ph2d_vec_layout::{Align, Dir, FrameStyle, ItemStyle, Justify, Node, Solved};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms, Xform, bake_xform, xform_of};

use crate::vec_entities::VecEntityMap;

/// Teto da caminhada de ancestrais — defesa contra hierarquia ciclada, não limite de produto (o
/// mesmo número, e pelo mesmo motivo, que o `vec_entities::MAX_DEPTH`).
const MAX_DEPTH: usize = 64;

/// **A geometria de MUNDO de um caminho neste frame** — o que o mapa já diz, ou a fonte assada.
///
/// A mesma lei do `align_live`/`bool_live`: o layout coloca *o que se DESENHA*, e não *o que está
/// guardado* — um filho com offset vivo tem de andar com o offset dele junto.
fn world_of(
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    id: VecPathId,
) -> Vec<VecPath> {
    match live.get(&id) {
        Some(items) => items.clone(),
        None => match scene.paths().iter().find(|p| p.id == id) {
            Some(p) => {
                let mut world = p.cooked().into_owned();
                bake_xform(&mut world, &xform_of(xforms, id));
                vec![world]
            }
            None => Vec::new(),
        },
    }
}

/// A caixa de MUNDO de uma lista de caminhos já assados. `None` quando não há geometria.
fn bbox_of(items: &[VecPath]) -> Option<([f64; 2], [f64; 2])> {
    let mut out: Option<([f64; 2], [f64; 2])> = None;
    for p in items {
        let Some((lo, hi)) = ph2d_vec_scene::curve_bbox_in_frame(p, 1.0, 0.0) else {
            continue;
        };
        out = Some(match out {
            None => (lo, hi),
            Some((a, b)) => (
                [a[0].min(lo[0]), a[1].min(lo[1])],
                [b[0].max(hi[0]), b[1].max(hi[1])],
            ),
        });
    }
    out
}

/// **A tradução do vocabulário do DOCUMENTO para o do MOTOR.**
///
/// ⚠️ Porta ÚNICA, e os `match` são exaustivos de propósito: uma direção nova no documento sem
/// tradução aqui **não compila**, em vez de cair num `_ =>` que a desenharia como uma linha.
fn frame_style(l: &VecLayout) -> FrameStyle {
    use ph2d_ecs::{LayoutAlign as A, LayoutDir as D, LayoutJustify as J};
    FrameStyle {
        dir: match l.dir {
            D::Row => Dir::Row,
            D::Column => Dir::Column,
            D::RowWrap => Dir::RowWrap,
        },
        gap: l.gap,
        pad: l.pad,
        align: match l.align {
            A::Start => Align::Start,
            A::Center => Align::Center,
            A::End => Align::End,
            A::Stretch => Align::Stretch,
        },
        justify: match l.justify {
            J::Start => Justify::Start,
            J::Center => Justify::Center,
            J::End => Justify::End,
            J::SpaceBetween => Justify::SpaceBetween,
            J::SpaceAround => Justify::SpaceAround,
        },
    }
}

fn item_style(it: Option<&VecLayoutItem>) -> ItemStyle {
    it.map_or_else(ItemStyle::default, |i| ItemStyle {
        grow: i.grow,
        shrink: i.shrink,
        basis: i.basis,
    })
}

/// Um nó recolhido: o índice na fatia do motor, a entidade, e os caminhos que ele MOVE.
struct Collected {
    /// Todos os caminhos da sub-árvore deste nó — é o conjunto que anda junto com a caixa dele.
    paths: Vec<VecPathId>,
    /// A caixa de mundo que ele ocupa HOJE.
    bbox: ([f64; 2], [f64; 2]),
    /// A entidade deste nó e a moldura que o COLOCA — `None` na raiz, que não é colocada por
    /// ninguém. É o que o gesto de reordenar consulta depois (ver [`FlowSlots`]).
    who: Option<(Entity, Entity)>,
}

/// **O canto superior-esquerdo, em MUNDO, do retângulo que o motor resolveu.**
///
/// ⚠️ Aqui mora a troca de eixo, e ela é a única do passe: o motor mede `y` para BAIXO a partir do
/// topo da moldura, o documento mede para CIMA. `top - y` é a conversão inteira.
fn world_target(frame_bbox: ([f64; 2], [f64; 2]), solved: Solved) -> ([f64; 2], [f64; 2]) {
    let (lo, hi) = frame_bbox;
    let left = lo[0] + solved[0];
    let top = hi[1] - solved[1];
    ([left, top - solved[3]], [left + solved[2], top])
}

/// O afim que leva a caixa `from` à caixa `to` (translada e escala em torno do canto
/// superior-esquerdo). Escala degenerada (caixa de largura ou altura zero) vira `1` — mover uma
/// coisa sem tamanho é legítimo, esticá-la não significa nada.
fn fit(from: ([f64; 2], [f64; 2]), to: ([f64; 2], [f64; 2])) -> Xform {
    let (fw, fh) = (from.1[0] - from.0[0], from.1[1] - from.0[1]);
    let (tw, th) = (to.1[0] - to.0[0], to.1[1] - to.0[1]);
    let sx = if fw.abs() > 1e-9 { tw / fw } else { 1.0 };
    let sy = if fh.abs() > 1e-9 { th / fh } else { 1.0 };
    // Canto superior-esquerdo: x mínimo, y MÁXIMO (o mundo é Y-up).
    let (fx, fy) = (from.0[0], from.1[1]);
    let (tx, ty) = (to.0[0], to.1[1]);
    Xform([sx, 0.0, 0.0, sy, tx - sx * fx, ty - sy * fy])
}

/// **Onde o último passe PÔS os filhos de uma moldura** — a régua que o gesto de reordenar lê.
///
/// ⚠️ Ela é PUBLICADA por quem colocou, e nunca re-derivada por quem arrasta. Um gesto que
/// recalculasse as posições seria a segunda resposta a *"onde este filho está?"*, e as duas
/// divergiriam no primeiro `grow` — o artista veria a forma numa posição e o slot ser escolhido
/// por outra.
pub(crate) struct FlowSlots {
    /// O eixo PRINCIPAL do fluxo: `true` = X (linha e quebra-linha), `false` = Y (coluna).
    pub(crate) main_x: bool,
    /// Os filhos na ORDEM do fluxo, cada um com o seu centro no eixo principal (mundo).
    pub(crate) kids: Vec<(Entity, f64)>,
}

/// **O auto layout de toda a cena.** Roda DEPOIS da booleana (ele coloca *o que os filhos de facto
/// desenham*) e ANTES do alinhamento (que recorta a faixa do traço na largura AUTORADA — escalar
/// depois de a recortar mudaria a espessura dela).
#[derive(Default)]
pub(crate) struct LayoutLive {
    /// Quantos filhos o último passe colocou.
    placed: usize,
    /// Por moldura que flui (bits da entidade), onde os filhos dela ficaram.
    ///
    /// ⚠️ `BTreeMap` e não `HashMap`: a ordem de iteração é parte do que o editor guarda, e o
    /// `HashMap` é banido por lint neste repo justamente por não a ter.
    slots: std::collections::BTreeMap<u64, FlowSlots>,
    /// **A POSE que cada caminho colocado recebeu** — o mesmo afim que foi assado na geometria,
    /// publicado também como número.
    ///
    /// ⚠️ Assar serve a quem DESENHA; quem **aponta** e quem **anota** (as âncoras do modo Node,
    /// a caixa do gizmo, o hit-test) não desenha geometria nenhuma — lê a pose autorada, que não
    /// se mexeu. Sem esta tabela as âncoras ficam no lugar de origem e o clique procura a forma
    /// onde ela já não está.
    ///
    /// ⚠️ Os dois saem do MESMO `x`, uma linha um do outro: é isso que impede a pose publicada de
    /// divergir da geometria assada.
    poses: std::collections::BTreeMap<VecPathId, Xform>,
}

impl LayoutLive {
    /// ⚠️ **Hoje o único leitor é o gate**, e por isso o acessor é `cfg(test)`: um `pub` sem
    /// chamador não é código morto silencioso, é uma segunda resposta à espera de alguém a chamar
    /// (a lição do `warp_axis` no Painter). O segundo leitor nasce com a seção do painel, que
    /// precisa dele para decidir se oferece o Apply.
    #[cfg(test)]
    pub(crate) fn placed(&self) -> usize {
        self.placed
    }

    /// **A tabela de poses deste frame**, para o `VecViewState` que a shell publica.
    pub(crate) fn poses(&self) -> Vec<(VecPathId, Xform)> {
        self.poses.iter().map(|(id, x)| (*id, *x)).collect()
    }

    /// Onde o último passe pôs os filhos desta moldura — `None` se ela não flui.
    pub(crate) fn slots_of(&self, frame: Entity) -> Option<&FlowSlots> {
        self.slots.get(&frame.to_bits())
    }

    /// Uma régua montada à mão — **só para os gates do gesto de reordenar**.
    ///
    /// ⚠️ Ela existe para o gate poder afirmar o GESTO sem montar uma cena vetorial inteira; o
    /// produto nunca a chama, e o `cfg(test)` é o que impede um segundo produtor de posições de
    /// nascer ao lado do passe (a lei do ADR-0153: a régua é publicada por quem coloca).
    #[cfg(test)]
    pub(crate) fn with_slots(frame: Entity, main_x: bool, kids: Vec<(Entity, f64)>) -> Self {
        let mut me = Self::default();
        me.slots.insert(frame.to_bits(), FlowSlots { main_x, kids });
        me
    }

    pub(crate) fn recook(
        &mut self,
        scene: &VecScene,
        sim: &SimWorld,
        map: &VecEntityMap,
        xforms: &VecXforms,
        live: &mut LiveGeometry,
    ) {
        self.placed = 0;
        self.slots.clear();
        self.poses.clear();
        for frame in outermost_flowing_frames(scene, sim, map) {
            self.lay_out(scene, sim, xforms, live, frame);
        }
    }

    /// Uma moldura (e tudo o que flui dentro dela).
    fn lay_out(
        &mut self,
        scene: &VecScene,
        sim: &SimWorld,
        xforms: &VecXforms,
        live: &mut LiveGeometry,
        frame: Entity,
    ) {
        let w = sim.world();
        let mut nodes: Vec<Node> = Vec::new();
        let mut collected: Vec<Collected> = Vec::new();

        // A raiz: a própria moldura. O tamanho dela é o da caixa que ela DESENHA (o retângulo vivo
        // que a carrega, ADR-0153 W0) — nunca um segundo número guardado ao lado.
        let Some(root_paths) = own_paths(sim, scene, frame) else {
            return;
        };
        let root_world = world_of_all(scene, xforms, live, &root_paths);
        let Some(root_bbox) = bbox_of(&root_world) else {
            return;
        };
        let Some(root_layout) = w.get::<VecLayout>(frame) else {
            return;
        };
        nodes.push(Node {
            parent: None,
            frame: Some(frame_style(root_layout)),
            item: ItemStyle::default(),
            size: [
                root_bbox.1[0] - root_bbox.0[0],
                root_bbox.1[1] - root_bbox.0[1],
            ],
        });
        collected.push(Collected {
            paths: Vec::new(), // a raiz não se move: ela é a régua
            bbox: root_bbox,
            who: None,
        });

        // Os filhos, em largura, na ordem da HIERARQUIA — é ela que o artista vê e reordena.
        let mut queue: Vec<(Entity, usize)> = vec![(frame, 0)];
        while let Some((parent, parent_idx)) = queue.pop() {
            let Some(kids) = w.get::<ph2d_ecs::Children>(parent) else {
                continue;
            };
            for &kid in kids.iter() {
                let flows_here = w.get::<VecLayout>(kid).is_some();
                // ⚠️ **Uma moldura que FLUI mede-se e move-se por SI, nunca pela sub-árvore.**
                //
                // Os filhos dela viram nós próprios, com transformação própria; incluí-los aqui
                // aplicaria a deles DUAS vezes — a do pai a mover a sub-árvore inteira, e a
                // própria — e a cada frame os netos fugiriam mais para fora da moldura. Um nó que
                // NÃO flui é o oposto: nada lá dentro é nó, então ele carrega a sub-árvore toda.
                let paths = if flows_here {
                    own_paths(sim, scene, kid).unwrap_or_default()
                } else {
                    crate::vec_entities::subtree_paths(sim, scene, kid)
                };
                if paths.is_empty() {
                    continue;
                }
                let items = world_of_all(scene, xforms, live, &paths);
                let Some(bbox) = bbox_of(&items) else {
                    continue;
                };
                nodes.push(Node {
                    parent: Some(parent_idx),
                    frame: w.get::<VecLayout>(kid).map(frame_style),
                    item: item_style(w.get::<VecLayoutItem>(kid)),
                    size: [bbox.1[0] - bbox.0[0], bbox.1[1] - bbox.0[1]],
                });
                let idx = nodes.len() - 1;
                collected.push(Collected {
                    paths,
                    bbox,
                    who: Some((kid, parent)),
                });
                if flows_here {
                    queue.push((kid, idx));
                }
            }
        }
        if nodes.len() < 2 {
            return; // uma moldura sem filhos não coloca nada
        }

        let Ok(solved) = ph2d_vec_layout::solve(&nodes) else {
            // O motor recusou (uma árvore que este colector não devia produzir): a arte fica como
            // está, em vez de piscar para posições que ninguém pediu.
            return;
        };

        for (i, c) in collected.iter().enumerate().skip(1) {
            let target = world_target(root_bbox, solved[i]);
            // **Publica ONDE este filho ficou**, antes de qualquer early-out: o gesto de reordenar
            // precisa da régua mesmo quando a colocação não moveu nada (uma moldura já arrumada é
            // exactamente onde o artista vai arrastar).
            if let Some((kid, parent)) = c.who
                && let Some(l) = w.get::<VecLayout>(parent)
            {
                let main_x = !matches!(l.dir, ph2d_ecs::LayoutDir::Column);
                let centre = if main_x {
                    (target.0[0] + target.1[0]) * 0.5
                } else {
                    (target.0[1] + target.1[1]) * 0.5
                };
                self.slots
                    .entry(parent.to_bits())
                    .or_insert_with(|| FlowSlots {
                        main_x,
                        kids: Vec::new(),
                    })
                    .kids
                    .push((kid, centre));
            }
            let x = fit(c.bbox, target);
            if is_identity(&x) {
                continue; // nada a fazer: não paga sequer a cópia
            }
            for &id in &c.paths {
                let mut items = world_of(scene, xforms, live, id);
                for p in &mut items {
                    bake_xform(p, &x);
                }
                live.insert(id, items);
                // O MESMO afim, como número: quem não desenha geometria precisa dele.
                self.poses.insert(id, x);
            }
            self.placed += 1;
        }
    }
}

fn is_identity(x: &Xform) -> bool {
    let e = 1e-9;
    (x.0[0] - 1.0).abs() < e
        && x.0[1].abs() < e
        && x.0[2].abs() < e
        && (x.0[3] - 1.0).abs() < e
        && x.0[4].abs() < e
        && x.0[5].abs() < e
}

/// A geometria de mundo de vários caminhos, concatenada.
fn world_of_all(
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    ids: &[VecPathId],
) -> Vec<VecPath> {
    ids.iter()
        .flat_map(|&id| world_of(scene, xforms, live, id))
        .collect()
}

/// Os caminhos da PRÓPRIA entidade (não da sub-árvore) — a moldura mede-se por si.
fn own_paths(sim: &SimWorld, scene: &VecScene, e: Entity) -> Option<Vec<VecPathId>> {
    let id = sim.world().get::<ph2d_ecs::VecPathRef>(e)?.0;
    scene.paths().iter().find(|p| p.id == id)?;
    Some(vec![id])
}

/// **As molduras que fluem e que não estão DENTRO de outra que flui.**
///
/// ⚠️ Uma moldura aninhada não entra nesta lista: ela é resolvida como parte da árvore da
/// ancestral, e é isso que faz o `grow` dela ser honrado antes de ela colocar os próprios filhos.
/// Processá-la também por fora seria colocá-la duas vezes, com a segunda a ler a caixa que a
/// primeira acabou de mover.
fn outermost_flowing_frames(scene: &VecScene, sim: &SimWorld, map: &VecEntityMap) -> Vec<Entity> {
    let w = sim.world();
    let mut found: Vec<Entity> = Vec::new();
    for path in scene.paths() {
        let Some(&bits) = map.get(&path.id) else {
            continue;
        };
        let mut cur = Entity::from_bits(bits);
        for _ in 0..MAX_DEPTH {
            if w.get::<VecLayout>(cur).is_some() && !found.contains(&cur) {
                found.push(cur);
            }
            match w.get::<ph2d_ecs::ChildOf>(cur) {
                Some(c) => cur = c.parent(),
                None => break,
            }
        }
    }
    // Fica só quem NÃO tem ancestral que flui.
    found.retain(|&e| !has_flowing_ancestor(sim, e));
    // Ordem estável entre frames.
    found.sort_unstable();
    found
}

fn has_flowing_ancestor(sim: &SimWorld, e: Entity) -> bool {
    let w = sim.world();
    let mut cur = e;
    for _ in 0..MAX_DEPTH {
        match w.get::<ph2d_ecs::ChildOf>(cur) {
            Some(c) => {
                cur = c.parent();
                if w.get::<VecLayout>(cur).is_some() {
                    return true;
                }
            }
            None => return false,
        }
    }
    false
}

#[cfg(test)]
#[path = "layout_live_tests.rs"]
mod tests;
