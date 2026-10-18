//! **As alças da forma VIVA no canvas** (F9) — o lado do mundo da lei de
//! [`ph2d_tool_vector::shape_handles`]: qual forma, onde as alças ficam em MUNDO, o press que agarra
//! uma, o arrasto que escreve, o `Esc` que repõe e o duplo-clique que volta à fábrica.
//!
//! ⭐ **Uma porta só de escrita:** o arrasto escreve pelo MESMO [`crate::shape_params::edit_selected_shape`]
//! que os campos numéricos usam — re-cozinha no lugar, guarda id, estilo e `Transform`. Uma alça é
//! só outra mão sobre os mesmos campos, nunca uma segunda mutação da forma. O passo de undo é o do
//! diff do quadro (o botão premido segura-o até soltar: um arrasto = um passo).
//!
//! ⭐ **O alvo é o dos campos** ([`crate::shape_params::shape_field_target`], com o modo e o latch do
//! catálogo — [`Gate`]): as alças vivem só no modo Forma (F10), e armado no catálogo ninguém as tem.
//! Alça e campo falam sempre da MESMA forma.

use ph2d_ecs::{Entity, SimWorld, VecShape};
use ph2d_tool_vector::params::DrawMode;
use ph2d_tool_vector::shape_handles::{self as law, HandleDrag, Mods};
use ph2d_tool_vector::shape_handles_face as face;
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_entities::transform::{world_transform, xform_of_transform};
use ph2d_vec_scene::{ShapeKind, ShapeValues, VecPath, VecPathId, VecScene, Xform};

use crate::shape_params::{edit_selected_shape, shape_field_target};

/// O raio de acerto de uma alça, em px de ecrã: o LADO da alça da caixa
/// (`ph2d_editor_core::HANDLE_SIZE_PX`) — o mesmo alcance que a mão já conhece.
pub const HIT_PX: f64 = ph2d_editor_core::HANDLE_SIZE_PX as f64;

/// Onde uma alça estaciona, em px: o círculo de acerto dela encosta no quadrado da alça da caixa
/// (meio lado + o raio de acerto) — as duas nunca disputam o mesmo ponto.
pub const PARK_PX: f64 = ph2d_editor_core::HANDLE_SIZE_PX as f64 * 0.5 + HIT_PX;

/// ⭐ **De quem são as alças** (F10): o modo da ferramenta e o latch do «armado no catálogo»
/// (`VecState::shape_armed`) — a MESMA pergunta que os campos fazem.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Gate {
    pub mode: DrawMode,
    pub armed: bool,
    /// A régua das faixas dos campos `Px` (raios, cantos) — a do painel (plano 31).
    pub doc: ph2d_editor_core::DocPx,
}

impl Gate {
    /// O modo Forma, desarmado — a forma seleccionada (a acabada de desenhar) tem alças.
    pub const SHAPE: Self = Self {
        mode: DrawMode::Shape,
        armed: false,
        doc: ph2d_editor_core::DocPx::DEFAULT,
    };

    /// O portão do quadro, lido do estado vetorial.
    #[must_use]
    pub fn of(vec: &crate::state::VecState) -> Self {
        Self {
            mode: vec.draw_config.mode,
            armed: vec.shape_armed,
            doc: vec.doc,
        }
    }
}

/// `true` = os NÓS da selecção não se desenham: no modo Forma NUNCA (F13, o dono: «shape não deve
/// mostrar nós como a ferramenta node» — nem armado, nem numa forma sem alças: nada no modo Forma
/// os arrasta), e onde quer que as alças de uma forma viva estejam à vista (F10: as âncoras laranja
/// tapavam as alças verdes na foto da cena 16).
#[must_use]
pub fn hides_the_nodes(vec: &crate::state::VecState, sim: &SimWorld) -> bool {
    vec.draw_config.mode == DrawMode::Shape
        || Target::of(sim, &vec.entities, vec.pen.selected_paths(), Gate::of(vec)).is_some()
}

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
    /// A escala de mundo (a caneta do traço mede-se sob ela — a caixa do gizmo).
    scale: [f64; 2],
    doc: ph2d_editor_core::DocPx,
}

impl Target {
    fn of(sim: &SimWorld, map: &VecEntityMap, selection: &[VecPathId], gate: Gate) -> Option<Self> {
        if !crate::overlay::vec_overlay_plan(true, gate.mode).shape_handles {
            return None;
        }
        let (id, entity, kind, values) =
            shape_field_target(sim, map, selection, gate.mode, gate.armed)?;
        let Some(VecShape::Param { w, h, .. }) = sim.world().get::<VecShape>(entity) else {
            return None;
        };
        if law::handles_of(kind).is_empty() {
            return None;
        }
        let pose = world_transform(sim, entity);
        let xf = xform_of_transform(pose);
        Some(Self {
            id,
            entity,
            kind,
            values,
            w: *w,
            h: *h,
            inv: xf.inverse()?,
            xf,
            scale: [f64::from(pose.scale.x), f64::from(pose.scale.y)],
            doc: gate.doc,
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

    /// ⭐ (F14b) Quanto o que a forma PINTA passa da curva pela DIREITA, em local — o traço (a
    /// junta, o miter, o Outer), pela MESMA conta da caixa do gizmo
    /// (`ph2d_vec_render::path_intrinsic_visual_bbox`, a *Use Preview Bounds*): as alças da caixa
    /// vivem na caixa visual, e a fila de ⇕ mede-se dela (F13 §7.2: com meia caneta ≥ 9 px o ⇕
    /// encostava na alça do meio).
    fn paint_margin(&self, scene: &VecScene) -> f64 {
        let Some(p) = scene.path(self.id) else {
            return 0.0;
        };
        let bare = VecPath {
            stroke: None,
            ..p.clone()
        };
        match (
            ph2d_vec_render::path_intrinsic_visual_bbox(p, self.scale),
            ph2d_vec_render::path_intrinsic_visual_bbox(&bare, self.scale),
        ) {
            (Some((_, painted)), Some((_, curve))) => (painted[0] - curve[0]).max(0.0),
            _ => 0.0,
        }
    }

    /// As pegas (onde cada alça se desenha e agarra), em LOCAL. A fila de ⇕ fica à direita da
    /// caixa VISUAL ([`Self::paint_margin`]).
    fn grips(&self, scene: &VecScene, px_to_world: f64) -> Vec<Option<[f64; 2]>> {
        let ck = self.cook();
        let park = self.local_len(PARK_PX, px_to_world);
        let margin = self.paint_margin(scene);
        law::handles_of(self.kind)
            .iter()
            .map(|h| {
                let g =
                    law::handle_grip(self.kind, h, &self.values, self.half(), self.doc, park, &ck);
                if h.spot == law::Spot::Rail {
                    g.map(|p| [p[0] + margin, p[1]])
                } else {
                    g
                }
            })
            .collect()
    }
}

/// O que se desenha: as alças em MUNDO (com a cara e a raiz da haste de cada uma — F11), o trilho
/// da que está quente (agarrada ou sob o cursor) e o círculo tracejado do raio AGARRADO.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShapeHandlesView {
    pub points: Vec<[f64; 2]>,
    pub faces: Vec<ph2d_vec_render::HandleFace>,
    pub stems: Vec<Option<[f64; 2]>>,
    pub hot: Option<usize>,
    pub track: Vec<[f64; 2]>,
    pub ring: Vec<[f64; 2]>,
}

/// A cara que se PINTA para a cara que a lei DÁ — o render não conhece a lei (o compilador cobra
/// uma cara nova dos dois lados).
fn face_of(g: face::Glyph) -> ph2d_vec_render::HandleFace {
    use ph2d_vec_render::HandleFace as F;
    match g {
        face::Glyph::Corner => F::Corner,
        face::Glyph::Count => F::Count,
        face::Glyph::Angle => F::Angle,
        face::Glyph::Radius => F::Radius,
    }
}

/// Os pontos do círculo tracejado do raio.
const RING_N: usize = 96;

/// As alças da forma seleccionada (fora do `gate`, nenhuma). `grabbed` (o arrasto vivo) manda no
/// quente; sem ele, a alça sob `hover` (mundo) — a guia aparece ANTES do arrasto, como no Affinity.
#[must_use]
#[expect(
    clippy::too_many_arguments,
    reason = "o alvo (mundo, cena, mapa, selecção, portão) e o quadro (escala, agarrada, cursor)"
)]
pub fn view(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    selection: &[VecPathId],
    gate: Gate,
    px_to_world: f64,
    grabbed: Option<usize>,
    hover: Option<[f64; 2]>,
) -> Option<ShapeHandlesView> {
    let t = Target::of(sim, map, selection, gate)?;
    let grips = t.grips(scene, px_to_world);
    let world: Vec<Option<[f64; 2]>> = grips.iter().map(|g| g.map(|p| t.xf.apply(p))).collect();
    let hot_handle = grabbed.or_else(|| hover.and_then(|w| hit(&world, w, HIT_PX * px_to_world)));
    let track = hot_handle
        .and_then(|k| law::handles_of(t.kind).get(k))
        .map(|h| {
            law::track(t.kind, h, &t.values, t.half(), t.doc, &t.cook())
                .into_iter()
                .map(|p| t.xf.apply(p))
                .collect()
        })
        .unwrap_or_default();
    let (hs, ck) = (law::handles_of(t.kind), t.cook());
    let truth = |h: &law::HandleDesc| law::handle_point(h, &t.values, t.half(), &ck);
    let (mut points, mut faces, mut stems) = (Vec::new(), Vec::new(), Vec::new());
    // `hot` conta nos pontos DESENHADOS: uma alça sem ponto (a quina de uma elipse inteira) não
    // entra, e o índice da tabela desalinhava o halo.
    let mut hot = None;
    for (k, (h, g)) in hs.iter().zip(&world).enumerate() {
        let Some(p) = *g else { continue };
        if hot_handle == Some(k) {
            hot = Some(points.len());
        }
        points.push(p);
        faces.push(face_of(face::glyph_of(h)));
        stems.push(
            law::is_arc_angle(h)
                .then(|| truth(h))
                .flatten()
                .map(|q| t.xf.apply(q)),
        );
    }
    // O círculo do raio só DURANTE o arrasto (o R2 do Illustrator), pelo ponto verdadeiro.
    let ring = grabbed
        .and_then(|k| hs.get(k))
        .and_then(|h| truth(h).map(|at| face::radius_ring(h, at, t.half(), RING_N)))
        .unwrap_or_default()
        .into_iter()
        .map(|p| t.xf.apply(p))
        .collect();
    Some(ShapeHandlesView {
        points,
        faces,
        stems,
        hot,
        track,
        ring,
    })
}

/// ⭐ (F11) **A ETIQUETA** — `«Points: 7»`, `«Inner: 0.38»`: o campo traduzido e o número na
/// unidade dele (os comprimentos pela `LengthDisplay` do projecto, a da ficha da caixa). Da alça
/// AGARRADA; sem arrasto, da que está sob `hover` (mundo) — F12: com a fila de ⇕ iguais, o nome
/// diz qual é qual antes do arrasto. `None` fora de uma alça.
#[must_use]
pub fn readout(
    vec: &crate::state::VecState,
    sim: &SimWorld,
    scene: &VecScene,
    px_to_world: f64,
    display: ph2d_editor_core::LengthDisplay,
    hover: Option<[f64; 2]>,
) -> Option<String> {
    let t = Target::of(sim, &vec.entities, vec.pen.selected_paths(), Gate::of(vec))?;
    let k = match vec.shape_handles.grab {
        Some(g) => (t.id == g.id).then_some(g.handle())?,
        None => {
            let world: Vec<Option<[f64; 2]>> = t
                .grips(scene, px_to_world)
                .iter()
                .map(|g| g.map(|p| t.xf.apply(p)))
                .collect();
            hit(&world, hover?, HIT_PX * px_to_world)?
        }
    };
    let h = law::handles_of(t.kind).get(k)?;
    let name = |l: &'static str| ph2d_panel_vector::nomes_do_motor::campo(l).to_owned();
    let length = |w: f64| {
        let z = if px_to_world > 0.0 {
            1.0 / px_to_world
        } else {
            1.0
        };
        format!("{} {}", display.text_at_zoom(w, z), display.suffix())
    };
    Some(face::readout(t.kind, h, &t.values, &name, &length))
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
/// alça ali — o clique segue para a caixa ou para o desenho de uma forma nova.
#[must_use]
pub fn press(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    selection: &[VecPathId],
    gate: Gate,
    w: [f64; 2],
    px_to_world: f64,
) -> Option<ShapeHandleGrab> {
    let t = Target::of(sim, map, selection, gate)?;
    let grips = t.grips(scene, px_to_world);
    let world: Vec<Option<[f64; 2]>> = grips.iter().map(|g| g.map(|p| t.xf.apply(p))).collect();
    let k = hit(&world, w, HIT_PX * px_to_world)?;
    // A pega mede-se ao ponto VERDADEIRO, não ao estacionado: agarrar a alça estacionada e não
    // mexer não arredonda nada (o desvio da estação viaja com a mão). As de ÂNGULO medem-se ao
    // ponto DESENHADO (recolhido no raio dela: o mesmo ângulo) — a translação não torce o ângulo.
    let h = &law::handles_of(t.kind)[k];
    let truth = if law::reads_only_the_angle(h) {
        grips[k]?
    } else {
        law::handle_point(h, &t.values, t.half(), &t.cook())?
    };
    Some(ShapeHandleGrab {
        id: t.id,
        entity_bits: t.entity.to_bits(),
        drag: HandleDrag::start(k, t.values, t.inv.apply(w), truth, t.half()),
    })
}

/// O arrasto: leva a alça para `w` (mundo) e escreve os campos pela porta única. `false` = a forma
/// agarrada deixou de ser o alvo (outro escritor mexeu na seleção) — o chamador larga o arrasto.
pub fn drag(
    world: World<'_>,
    grab: &mut ShapeHandleGrab,
    w: [f64; 2],
    mods: Mods,
    px_to_world: f64,
) -> bool {
    let World {
        sim,
        scene,
        map,
        selection,
        gate,
    } = world;
    let Some(t) = Target::of(sim, map, selection, gate) else {
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
        t.doc,
        &t.cook(),
    );
    write(sim, scene, map, selection, gate, new)
}

/// O `Esc` (ou o botão direito) a meio: os campos voltam aos do press.
pub fn cancel(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    selection: &[VecPathId],
    gate: Gate,
    grab: &ShapeHandleGrab,
) -> bool {
    match Target::of(sim, map, selection, gate) {
        Some(t) if t.id == grab.id => write(sim, scene, map, selection, gate, grab.drag.at_press),
        _ => false,
    }
}

/// O DUPLO-CLIQUE numa alça: os campos dela voltam ao valor de fábrica (o Affinity). `false` =
/// nenhuma alça em `w`.
pub fn reset_at(world: World<'_>, w: [f64; 2], px_to_world: f64) -> bool {
    let World {
        sim,
        scene,
        map,
        selection,
        gate,
    } = world;
    let Some(g) = press(sim, scene, map, selection, gate, w, px_to_world) else {
        return false;
    };
    let Some(t) = Target::of(sim, map, selection, gate) else {
        return false;
    };
    let new = law::reset(t.kind, g.handle(), &t.values);
    write(sim, scene, map, selection, gate, new)
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

/// O mundo que as portas tocam (a shell parte o `gfx`; o resto vem do `VecState`).
pub struct World<'a> {
    pub sim: &'a mut SimWorld,
    pub scene: &'a mut VecScene,
    pub map: &'a VecEntityMap,
    pub selection: &'a [VecPathId],
    pub gate: Gate,
}

impl ShapeHandleInput {
    /// O press em `w` (mundo) às `now_ns`: sobre uma alça, agarra-a — ou, se é o 2.º press na
    /// MESMA alça dentro da janela do duplo-clique, repõe-a. `true` = consumido (o clique não segue
    /// para a caixa nem para o pick).
    pub fn press(&mut self, world: World<'_>, w: [f64; 2], px_to_world: f64, now_ns: u128) -> bool {
        let Some(g) = press(
            world.sim,
            world.scene,
            world.map,
            world.selection,
            world.gate,
            w,
            px_to_world,
        ) else {
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
            reset_at(world, w, px_to_world);
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
        if drag(world, &mut g, w, mods, px_to_world) {
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
        cancel(
            world.sim,
            world.scene,
            world.map,
            world.selection,
            world.gate,
            &g,
        );
        true
    }
}

/// O desenho das alças da forma seleccionada (por cima do gizmo): a quente é a agarrada ou a que
/// está sob `hover` (mundo), e o trilho dela é a guia. Nada fora do [`Gate`] do `vec` — nem com
/// `shown` apagado (o plano do overlay).
///
/// ⭐ (F11) Publica também no `hero` o que o Illustrator faz DURANTE o arrasto de um widget: a caixa
/// só com o CONTORNO (`outline_only` — as alças dela somem) e a ETIQUETA do número junto do cursor
/// (`handle_readout`; F12: também sob o cursor, sem arrasto). Fora de uma alça, apagam-se.
#[allow(clippy::too_many_arguments)] // o mesmo quadro que a shell parte para cada overlay
pub fn draw(
    vec: &crate::state::VecState,
    sim: &SimWorld,
    scene: &VecScene,
    px_to_world: f64,
    hover: Option<[f64; 2]>,
    shown: bool,
    camera: ph2d_vector::Affine,
    hero: &mut ph2d_editor_core::HeroScreen,
    target: &mut ph2d_vector::VectorScene,
) {
    let grabbed = vec.shape_handles.grab.map(|g| g.handle()).filter(|_| shown);
    hero.gizmo.outline_only = grabbed.is_some();
    hero.gizmo.handle_readout = hover
        .filter(|_| shown)
        .zip(readout(
            vec,
            sim,
            scene,
            px_to_world,
            ph2d_editor_core::LengthDisplay::of(&hero.project),
            hover,
        ))
        .map(|(w, text)| {
            let p = camera * ph2d_vector::Point::new(w[0], w[1]);
            #[expect(clippy::cast_possible_truncation, reason = "px de ecrã")]
            (text, [p.x as f32, p.y as f32])
        });
    if !shown {
        return;
    }
    let selection = vec.pen.selected_paths();
    if let Some(v) = view(
        sim,
        scene,
        &vec.entities,
        selection,
        Gate::of(vec),
        px_to_world,
        grabbed,
        hover,
    ) {
        let marks: Vec<ph2d_vec_render::HandleMark> = v
            .points
            .iter()
            .zip(&v.faces)
            .zip(&v.stems)
            .map(|((&at, &face), &stem)| ph2d_vec_render::HandleMark { at, face, stem })
            .collect();
        ph2d_vec_render::draw_shape_handles(
            &marks, v.hot, &v.track, &v.ring, camera, hero.theme, target,
        );
    }
}

/// As portas de [`ShapeHandleInput`] sobre o [`crate::state::VecState`] inteiro — a shell parte só
/// o `gfx` (o mundo e a cena); a seleção, o mapa e o estado vêm do `vec`.
pub mod on {
    use super::{Gate, Mods, SimWorld, VecScene, World};
    use crate::state::VecState;

    fn split<'a>(
        vec: &'a mut VecState,
        sim: &'a mut SimWorld,
        scene: &'a mut VecScene,
    ) -> (&'a mut super::ShapeHandleInput, World<'a>) {
        let gate = Gate::of(vec);
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
            gate,
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
    gate: Gate,
    new: ShapeValues,
) -> bool {
    edit_selected_shape(
        sim,
        scene,
        map,
        selection,
        gate.mode,
        gate.armed,
        |_, values| {
            *values = new;
            true
        },
    )
}

#[cfg(test)]
#[path = "shape_handles_live_tests.rs"]
mod tests;

/// As alças no MODO FORMA (F10) pela cadeia do produto: o gesto da ferramenta, a seleção da forma
/// nova, o nascimento vivo, o quadro dos campos (o latch) e as portas da shell.
#[cfg(test)]
#[path = "shape_mode_handles_tests.rs"]
mod shape_mode_tests;
