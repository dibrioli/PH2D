//! **As alças da forma VIVA no canvas** (F9) — o lado do mundo da lei de
//! [`ph2d_tool_vector::shape_handles`]: qual forma, onde as alças ficam em MUNDO, o press que agarra
//! uma, o arrasto que escreve, o `Esc` que repõe e o duplo-clique que volta à fábrica.
//!
//! ⭐ **Uma porta só de escrita:** o arrasto escreve pelo MESMO [`crate::shape_params::edit_selected_shape`]
//! que os campos numéricos usam — re-cozinha no lugar, guarda id, estilo e `Transform`. Uma alça é
//! só outra mão sobre os mesmos campos, nunca uma segunda mutação da forma. O passo de undo é o do
//! diff do quadro (o botão premido segura-o até soltar: um arrasto = um passo).
//!
//! O alvo é o de [`crate::shape_params::panel_shape_target`] — a 1.ª forma paramétrica da seleção,
//! o mesmo objeto cujos números a fila ② mostra.

use ph2d_ecs::{Entity, SimWorld, VecShape};
use ph2d_tool_vector::params::DrawMode;
use ph2d_tool_vector::shape_handles::{self as law, HandleDrag, Mods};
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_entities::transform::{world_transform, xform_of_transform};
use ph2d_vec_scene::{ShapeKind, ShapeValues, VecPath, VecPathId, VecScene, Xform};

use crate::shape_params::{edit_selected_shape, panel_shape_target};

/// O raio de acerto de uma alça, em px de ecrã: o LADO da alça da caixa
/// (`ph2d_editor_core::HANDLE_SIZE_PX`) — o mesmo alcance que a mão já conhece.
pub const HIT_PX: f64 = ph2d_editor_core::HANDLE_SIZE_PX as f64;

/// Onde uma alça estaciona, em px: o círculo de acerto dela encosta no quadrado da alça da caixa
/// (meio lado + o raio de acerto) — as duas nunca disputam o mesmo ponto.
pub const PARK_PX: f64 = ph2d_editor_core::HANDLE_SIZE_PX as f64 * 0.5 + HIT_PX;

/// A forma paramétrica das alças e o que é preciso para a cozinhar e a pôr no mundo.
struct Target {
    id: VecPathId,
    entity: Entity,
    kind: ShapeKind,
    values: ShapeValues,
    w: f64,
    h: f64,
    xf: Xform,
    inv: Xform,
}

impl Target {
    fn of(sim: &SimWorld, map: &VecEntityMap, selection: &[VecPathId]) -> Option<Self> {
        let (id, entity, kind, values) = panel_shape_target(sim, map, selection)?;
        let Some(VecShape::Param { w, h, .. }) = sim.world().get::<VecShape>(entity) else {
            return None;
        };
        if law::handles_of(kind).is_empty() {
            return None;
        }
        let xf = xform_of_transform(world_transform(sim, entity));
        Some(Self {
            id,
            entity,
            kind,
            values,
            w: *w,
            h: *h,
            inv: xf.inverse()?,
            xf,
        })
    }

    fn half(&self) -> [f64; 2] {
        [self.w.abs() * 0.5, self.h.abs() * 0.5]
    }

    fn cook(&self) -> impl Fn(&ShapeValues) -> VecPath + '_ {
        let (kind, w, h) = (self.kind.as_u16(), self.w, self.h);
        move |values: &ShapeValues| {
            crate::shape_live::recook_shape(&VecShape::Param {
                kind,
                w,
                h,
                values: *values,
            })
            .unwrap_or_default()
        }
    }

    /// `px` de ecrã em unidades LOCAIS (o objeto pode estar escalado).
    fn local_len(&self, px: f64, px_to_world: f64) -> f64 {
        let v = self.inv.apply_vec([px * px_to_world, 0.0]);
        v[0].hypot(v[1])
    }

    /// As pegas (onde cada alça se desenha e agarra), em LOCAL.
    fn grips(&self, px_to_world: f64) -> Vec<Option<[f64; 2]>> {
        let ck = self.cook();
        let park = self.local_len(PARK_PX, px_to_world);
        law::handles_of(self.kind)
            .iter()
            .map(|h| {
                law::handle_grip(
                    self.kind,
                    h,
                    &self.values,
                    self.half(),
                    px_to_world,
                    park,
                    &ck,
                )
            })
            .collect()
    }
}

/// O que se desenha: as alças em MUNDO e o trilho da que está quente (agarrada ou sob o cursor).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShapeHandlesView {
    pub points: Vec<[f64; 2]>,
    pub hot: Option<usize>,
    pub track: Vec<[f64; 2]>,
}

/// As alças da forma seleccionada. `grabbed` (o arrasto vivo) manda no quente; sem ele, a alça sob
/// `hover` (mundo) — a guia aparece ANTES do arrasto, como no Affinity.
#[must_use]
pub fn view(
    sim: &SimWorld,
    map: &VecEntityMap,
    selection: &[VecPathId],
    px_to_world: f64,
    grabbed: Option<usize>,
    hover: Option<[f64; 2]>,
) -> Option<ShapeHandlesView> {
    let t = Target::of(sim, map, selection)?;
    let grips = t.grips(px_to_world);
    let world: Vec<Option<[f64; 2]>> = grips.iter().map(|g| g.map(|p| t.xf.apply(p))).collect();
    let hot = grabbed.or_else(|| hover.and_then(|w| hit(&world, w, HIT_PX * px_to_world)));
    let track = hot
        .and_then(|k| law::handles_of(t.kind).get(k))
        .map(|h| {
            law::track(t.kind, h, &t.values, t.half(), px_to_world, &t.cook())
                .into_iter()
                .map(|p| t.xf.apply(p))
                .collect()
        })
        .unwrap_or_default();
    Some(ShapeHandlesView {
        points: world.into_iter().flatten().collect(),
        hot,
        track,
    })
}

/// A alça sob `w` (mundo): a ÚLTIMA da lista que o alcança (é a que se desenha por cima).
fn hit(world: &[Option<[f64; 2]>], w: [f64; 2], r: f64) -> Option<usize> {
    world.iter().enumerate().rev().find_map(|(k, p)| {
        let p = (*p)?;
        ((p[0] - w[0]).hypot(p[1] - w[1]) <= r).then_some(k)
    })
}

/// Um arrasto vivo de alça. Runtime-only (o resultado é documento; o arrasto não).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ShapeHandleGrab {
    pub id: VecPathId,
    pub entity_bits: u64,
    pub drag: HandleDrag,
}

impl ShapeHandleGrab {
    /// A alça agarrada (o índice na tabela da forma).
    #[must_use]
    pub fn handle(&self) -> usize {
        self.drag.handle
    }
}

/// ⭐ **O press**: se `w` (mundo) cai numa alça da forma seleccionada, agarra-a. `None` = nenhuma
/// alça ali — o clique segue para a caixa, o pick, o marquee.
#[must_use]
pub fn press(
    sim: &SimWorld,
    map: &VecEntityMap,
    selection: &[VecPathId],
    w: [f64; 2],
    px_to_world: f64,
) -> Option<ShapeHandleGrab> {
    let t = Target::of(sim, map, selection)?;
    let grips = t.grips(px_to_world);
    let world: Vec<Option<[f64; 2]>> = grips.iter().map(|g| g.map(|p| t.xf.apply(p))).collect();
    let k = hit(&world, w, HIT_PX * px_to_world)?;
    // A pega mede-se ao ponto VERDADEIRO, não ao estacionado: agarrar a alça estacionada e não
    // mexer não arredonda nada (o desvio da estação viaja com a mão).
    let truth = law::handle_point(&law::handles_of(t.kind)[k], &t.values, t.half(), &t.cook())?;
    Some(ShapeHandleGrab {
        id: t.id,
        entity_bits: t.entity.to_bits(),
        drag: HandleDrag::start(k, t.values, t.inv.apply(w), truth, t.half()),
    })
}

/// O arrasto: leva a alça para `w` (mundo) e escreve os campos pela porta única. `false` = a forma
/// agarrada deixou de ser o alvo (outro escritor mexeu na seleção) — o chamador larga o arrasto.
pub fn drag(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    selection: &[VecPathId],
    grab: &mut ShapeHandleGrab,
    w: [f64; 2],
    mods: Mods,
    px_to_world: f64,
) -> bool {
    let Some(t) = Target::of(sim, map, selection) else {
        return false;
    };
    if t.id != grab.id {
        return false;
    }
    let new = law::drag(
        t.kind,
        &mut grab.drag,
        t.inv.apply(w),
        mods,
        t.half(),
        px_to_world,
        &t.cook(),
    );
    write(sim, scene, map, selection, new)
}

/// O `Esc` (ou o botão direito) a meio: os campos voltam aos do press.
pub fn cancel(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    selection: &[VecPathId],
    grab: &ShapeHandleGrab,
) -> bool {
    match Target::of(sim, map, selection) {
        Some(t) if t.id == grab.id => write(sim, scene, map, selection, grab.drag.at_press),
        _ => false,
    }
}

/// O DUPLO-CLIQUE numa alça: os campos dela voltam ao valor de fábrica (o Affinity). `false` =
/// nenhuma alça em `w`.
pub fn reset_at(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    selection: &[VecPathId],
    w: [f64; 2],
    px_to_world: f64,
) -> bool {
    let Some(g) = press(sim, map, selection, w, px_to_world) else {
        return false;
    };
    let Some(t) = Target::of(sim, map, selection) else {
        return false;
    };
    let new = law::reset(t.kind, g.handle(), &t.values);
    write(sim, scene, map, selection, new)
}

/// ⭐ **A entrada das alças** — o que a shell guarda entre eventos (`VecState::shape_handles`) e as
/// quatro portas que ela chama: o press, o arrasto, o soltar e o `Esc`. Runtime-only.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ShapeHandleInput {
    /// O arrasto vivo, se algum.
    pub grab: Option<ShapeHandleGrab>,
    /// O último press numa alça (instante, alça, forma) — o par do duplo-clique: o canvas não
    /// emite `DoubleClick` (é evento de widget), então ele conta-se aqui.
    last: Option<(u128, usize, VecPathId)>,
}

/// O mundo que as portas tocam (a shell parte o `gfx` nestes três).
pub struct World<'a> {
    pub sim: &'a mut SimWorld,
    pub scene: &'a mut VecScene,
    pub map: &'a VecEntityMap,
    pub selection: &'a [VecPathId],
}

impl ShapeHandleInput {
    /// O press em `w` (mundo) às `now_ns`: sobre uma alça, agarra-a — ou, se é o 2.º press na
    /// MESMA alça dentro da janela do duplo-clique, repõe-a. `true` = consumido (o clique não segue
    /// para a caixa nem para o pick).
    pub fn press(&mut self, world: World<'_>, w: [f64; 2], px_to_world: f64, now_ns: u128) -> bool {
        let Some(g) = press(world.sim, world.map, world.selection, w, px_to_world) else {
            self.last = None;
            return false;
        };
        let double = self.last.is_some_and(|(t, k, id)| {
            k == g.handle()
                && id == g.id
                && now_ns.saturating_sub(t) < ph2d_editor_core::interaction::DOUBLE_CLICK_WINDOW_NS
        });
        if double {
            self.last = None;
            self.grab = None;
            reset_at(
                world.sim,
                world.scene,
                world.map,
                world.selection,
                w,
                px_to_world,
            );
        } else {
            self.last = Some((now_ns, g.handle(), g.id));
            self.grab = Some(g);
        }
        true
    }

    /// O arrasto (o cursor em `w`). `false` = nenhum arrasto vivo (o evento segue adiante). Um
    /// arrasto cuja forma saiu do alvo é largado.
    pub fn drag(&mut self, world: World<'_>, w: [f64; 2], mods: Mods, px_to_world: f64) -> bool {
        let Some(mut g) = self.grab else {
            return false;
        };
        if drag(
            world.sim,
            world.scene,
            world.map,
            world.selection,
            &mut g,
            w,
            mods,
            px_to_world,
        ) {
            self.grab = Some(g);
        } else {
            self.grab = None;
        }
        true
    }

    /// O soltar: o arrasto acaba (o passo de undo fecha com o botão). `true` = havia arrasto.
    pub fn release(&mut self) -> bool {
        self.grab.take().is_some()
    }

    /// O `Esc` a meio do arrasto: os campos voltam aos do press e o arrasto acaba. `true` =
    /// consumido.
    pub fn escape(&mut self, world: World<'_>) -> bool {
        let Some(g) = self.grab.take() else {
            return false;
        };
        cancel(world.sim, world.scene, world.map, world.selection, &g);
        true
    }
}

/// O desenho das alças da forma seleccionada (por cima do gizmo): a quente é a agarrada ou a que
/// está sob `hover` (mundo), e o trilho dela é a guia. Nada sem uma forma com alças na seleção.
pub fn draw(
    vec: &crate::state::VecState,
    sim: &SimWorld,
    px_to_world: f64,
    hover: Option<[f64; 2]>,
    camera: ph2d_vector::Affine,
    theme: ph2d_tokens::Theme,
    target: &mut ph2d_vector::VectorScene,
) {
    let grabbed = vec.shape_handles.grab.map(|g| g.handle());
    let selection = vec.pen.selected_paths();
    if let Some(v) = view(sim, &vec.entities, selection, px_to_world, grabbed, hover) {
        ph2d_vec_render::draw_shape_handles(&v.points, v.hot, &v.track, camera, theme, target);
    }
}

/// As portas de [`ShapeHandleInput`] sobre o [`crate::state::VecState`] inteiro — a shell parte só
/// o `gfx` (o mundo e a cena); a seleção, o mapa e o estado vêm do `vec`.
pub mod on {
    use super::{Mods, SimWorld, VecScene, World};
    use crate::state::VecState;

    fn split<'a>(
        vec: &'a mut VecState,
        sim: &'a mut SimWorld,
        scene: &'a mut VecScene,
    ) -> (&'a mut super::ShapeHandleInput, World<'a>) {
        let VecState {
            shape_handles,
            entities,
            pen,
            ..
        } = vec;
        let world = World {
            sim,
            scene,
            map: entities,
            selection: pen.selected_paths(),
        };
        (shape_handles, world)
    }

    /// [`super::ShapeHandleInput::press`].
    pub fn press(
        vec: &mut VecState,
        sim: &mut SimWorld,
        scene: &mut VecScene,
        w: [f64; 2],
        px_to_world: f64,
        now_ns: u128,
    ) -> bool {
        let (input, world) = split(vec, sim, scene);
        input.press(world, w, px_to_world, now_ns)
    }

    /// [`super::ShapeHandleInput::drag`].
    pub fn drag(
        vec: &mut VecState,
        sim: &mut SimWorld,
        scene: &mut VecScene,
        w: [f64; 2],
        mods: Mods,
        px_to_world: f64,
    ) -> bool {
        let (input, world) = split(vec, sim, scene);
        input.drag(world, w, mods, px_to_world)
    }

    /// [`super::ShapeHandleInput::escape`].
    pub fn escape(vec: &mut VecState, sim: &mut SimWorld, scene: &mut VecScene) -> bool {
        let (input, world) = split(vec, sim, scene);
        input.escape(world)
    }
}

fn write(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    selection: &[VecPathId],
    new: ShapeValues,
) -> bool {
    edit_selected_shape(
        sim,
        scene,
        map,
        selection,
        DrawMode::Select,
        false,
        |_, values| {
            *values = new;
            true
        },
    )
}

#[cfg(test)]
#[path = "shape_handles_live_tests.rs"]
mod tests;
