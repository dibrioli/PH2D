//! ⭐⭐ **VECTOR ▸ EDIT** — o modo que o OBJECTO vetorial declara e como ele abre a ferramenta do
//! vetor DENTRO dele (spec/06 F3; escolha do dono, 04/10: *«ao clicar nele cria-se um objeto vazio e
//! entra-se no modo edit do vector com o menu exatamente como era antigamente»*).
//!
//! - **Edit** = a ferramenta `vector` na mão (qualquer `DrawMode`, o painel inteiro) com o objecto em
//!   mãos ([`EditTarget`]). Tudo o que nasce nesse Edit é filho dele
//!   ([`ph2d_vec_entities::entities::object::adopt_loose`]); só as formas dele se agarram e mostram
//!   nós ([`EditTarget::editing`] → `VecViewState::editing`). As formas são as PARTES do modo.
//! - **Object** = a ferramenta sai da mão; o clique numa forma selecciona o objecto inteiro.
//! - ⭐ **Multi-objecto** ([`ModeFamily::joins`]): `Tab` com vários objectos = Edit de todos.
//! - ⭐ **O ALVO** mora no [`crate::state::VecState`], fora da `VecScene` (que entra no undo).
//!
//! **Duas portas para o Edit, uma regra** ([`ModeFamily::wants`]): o objecto que NASCE pelo menu Add
//! pede-o, e a ferramenta `vector` que chega à mão sem o modo (a aba de cima, as cenas
//! `PH2D_*_SMOKE`) pede-o sobre o objecto da forma seleccionada.

use ph2d_component_desc::ObjectKind;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor_core::object_mode::{ActiveMode, ModeRequest, ObjectMode};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::mode_drive::ModeFamily;
use ph2d_editor_core::{ToolId, ToolRegistry};
use ph2d_tool_vector::VectorTool;
use ph2d_vec_entities::entities::{VecEntityMap, object_of};
use ph2d_vec_scene::VecPathId;

use crate::state::VecState;

/// O id da ferramenta do vetor no registo.
const VECTOR: &str = "vector";

/// ⭐ **O que o Edit tem em mãos.**
#[derive(Default)]
pub struct EditTarget {
    /// Os objectos do Edit — o activo à frente. Vazio em Object.
    pub objects: Vec<u64>,
    /// O quadro anterior seguia um Edit do vetor.
    following: bool,
    /// O objecto que nasceu pelo menu Add e ainda não pediu o Edit.
    born: Option<u64>,
}

impl EditTarget {
    /// O objecto `bits` acabou de nascer: pede o Edit no próximo quadro do modo.
    pub fn born(&mut self, bits: u64) {
        self.born = Some(bits);
    }

    /// O objecto onde nasce o que se desenha agora — `None` em Object.
    #[must_use]
    pub fn object(&self) -> Option<Entity> {
        self.objects.first().map(|b| Entity::from_bits(*b))
    }

    /// ⭐ **As formas do Edit, para a vista** — as dos objectos em mãos e as soltas (que entram no
    /// objecto quando o gesto acabar). `None` = Object; `Some` vazio = um objecto ainda sem formas.
    #[must_use]
    pub fn editing(&self, sim: &SimWorld, map: &VecEntityMap) -> Option<Vec<VecPathId>> {
        if self.objects.is_empty() {
            return None;
        }
        let ours = |bits: u64| {
            object_of(sim, Entity::from_bits(bits))
                .is_none_or(|o| self.objects.contains(&o.to_bits()))
        };
        Some(
            map.iter()
                .filter(|(_, b)| ours(**b))
                .map(|(id, _)| *id)
                .collect(),
        )
    }
}

/// A ferramenta do vetor, se é a que está na mão.
pub(crate) fn tool_mut(tools: &mut ToolRegistry) -> Option<&mut VectorTool> {
    tools
        .active_mut()
        .filter(|t| t.id() == ToolId::new(VECTOR))
        .and_then(|t| t.as_any_mut().downcast_mut::<VectorTool>())
}

fn in_hand(tools: &mut ToolRegistry) -> bool {
    tool_mut(tools).is_some()
}

/// ⭐⭐ **A LEI do «tem em mãos»**, pura: Edit, o objecto é o alvo e a ferramenta está na mão.
pub(crate) fn holds(mode: ObjectMode, is_target: bool, tool_in_hand: bool) -> bool {
    mode == ObjectMode::Edit && is_target && tool_in_hand
}

/// ⭐⭐ **A LEI da porta antiga**, pura: a ferramenta na mão que o quadro anterior não seguia pede o
/// modo.
pub(crate) fn adopt(tool_in_hand: bool, following: bool) -> bool {
    tool_in_hand && !following
}

/// ⭐⭐ **A LEI do «a ferramenta segue o modo»**, pura: `true` = o Edit ACABOU neste quadro.
pub(crate) fn releases(ours_now: bool, following: bool) -> bool {
    !ours_now && following
}

/// ⭐ **A família**, construída em cada quadro com o que ela lê do mundo.
pub struct Family<'a> {
    vec: &'a mut VecState,
    /// Cada objecto vetorial, com tudo o que vive debaixo dele.
    objects: Vec<(u64, Vec<u64>)>,
    /// As formas sem objecto (entram num no próximo assentamento).
    loose: Vec<u64>,
    /// Os objectos das formas que a caneta tem seleccionadas, a da última no fim.
    pen_owners: Vec<u64>,
    /// Este quadro o Edit veio pela porta antiga ([`ModeFamily::wants`] pela caneta): só aí os
    /// [`Self::pen_owners`] juntam — noutra entrada a caneta pode ter a selecção de um Edit velho.
    via_pen: bool,
}

/// Teto de nós de uma sub-árvore (defesa contra save corrompido, não limite de produto).
const MAX_NODES: usize = 4096;

/// Tudo o que vive debaixo de `root`.
fn descendants(sim: &SimWorld, root: Entity) -> Vec<u64> {
    let w = sim.world();
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if out.len() >= MAX_NODES {
            break;
        }
        if let Some(kids) = w.get::<ph2d_ecs::Children>(e) {
            for k in kids.iter().copied() {
                out.push(k.to_bits());
                stack.push(k);
            }
        }
    }
    out
}

impl<'a> Family<'a> {
    /// Lê os objectos e as soltas de `sim`.
    pub fn new(vec: &'a mut VecState, sim: &mut SimWorld) -> Self {
        let world = sim.world_mut();
        let mut q = world.query::<(Entity, &ph2d_ecs::VecObject)>();
        let roots: Vec<Entity> = q.iter(world).map(|(e, _)| e).collect();
        let objects = roots
            .into_iter()
            .map(|o| (o.to_bits(), descendants(sim, o)))
            .collect();
        let alive = |b: u64| sim.world().get_entity(Entity::from_bits(b)).is_ok();
        let loose = vec
            .entities
            .values()
            .copied()
            .filter(|b| alive(*b) && object_of(sim, Entity::from_bits(*b)).is_none())
            .collect();
        let mut pen_owners: Vec<u64> = Vec::new();
        for id in vec.pen.selected_paths() {
            let Some(b) = vec.entities.get(id) else {
                continue;
            };
            if let Some(o) = object_of(sim, Entity::from_bits(*b)).map(Entity::to_bits) {
                pen_owners.retain(|x| *x != o);
                pen_owners.push(o);
            }
        }
        Self {
            vec,
            objects,
            loose,
            pen_owners,
            via_pen: false,
        }
    }

    fn is_object(&self, bits: u64) -> bool {
        self.objects.iter().any(|(o, _)| *o == bits)
    }
}

impl ModeFamily for Family<'_> {
    fn modes(&self) -> &'static [(ObjectKind, ObjectMode)] {
        &[(ObjectKind::Vector, ObjectMode::Edit)]
    }

    fn holds(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry) -> bool {
        let is_target = self.vec.edit.objects.first() == Some(&entity) && self.is_object(entity);
        holds(mode, is_target, in_hand(tools))
    }

    fn enter(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry) -> bool {
        self.enter_with(mode, entity, &[], tools)
    }

    fn joins(&self, mode: ObjectMode) -> bool {
        mode == ObjectMode::Edit
    }

    fn enter_with(
        &mut self,
        mode: ObjectMode,
        entity: u64,
        joined: &[u64],
        tools: &mut ToolRegistry,
    ) -> bool {
        if mode != ObjectMode::Edit || !self.is_object(entity) {
            return false;
        }
        if !in_hand(tools) && !tools.set_active(&ToolId::new(VECTOR)) {
            return false;
        }
        let mut objects = vec![entity];
        let pen: &[u64] = if self.via_pen { &self.pen_owners } else { &[] };
        for b in joined.iter().chain(pen) {
            if self.is_object(*b) && !objects.contains(b) {
                objects.push(*b);
            }
        }
        self.vec.edit.objects = objects;
        self.vec.edit.born = self.vec.edit.born.filter(|b| *b != entity);
        true
    }

    fn leave(&mut self, _: ObjectMode, _: u64, tools: &mut ToolRegistry) {
        self.vec.edit.objects.clear();
        if in_hand(tools) {
            tools.activate_default();
        }
    }

    fn follow(&mut self, current: Option<ActiveMode>, tools: &mut ToolRegistry) {
        let ours = current
            .filter(|a| a.mode == ObjectMode::Edit && self.is_object(a.entity))
            .map(|a| a.entity);
        let objects = &self.objects;
        let edit = &mut self.vec.edit;
        if let Some(o) = ours {
            if edit.objects.first() != Some(&o) {
                edit.objects = vec![o];
            }
            edit.objects.retain(|b| objects.iter().any(|(x, _)| x == b));
        } else if releases(false, edit.following) {
            edit.objects.clear();
            if in_hand(tools) {
                tools.activate_default();
            }
        }
        edit.following = ours.is_some();
    }

    fn wants(&mut self, tools: &mut ToolRegistry) -> Option<(u64, ObjectMode)> {
        if let Some(b) = self.vec.edit.born.take()
            && self.is_object(b)
        {
            return Some((b, ObjectMode::Edit));
        }
        if !adopt(in_hand(tools), self.vec.edit.following) {
            return None;
        }
        self.via_pen = true;
        Some((*self.pen_owners.last()?, ObjectMode::Edit))
    }

    fn parts(&mut self, entity: u64) -> Option<Vec<u64>> {
        let edit = &self.vec.edit.objects;
        if edit.first() != Some(&entity) {
            return None;
        }
        let mut parts: Vec<u64> = Vec::new();
        for o in edit {
            if *o != entity {
                parts.push(*o);
            }
            if let Some((_, under)) = self.objects.iter().find(|(x, _)| x == o) {
                parts.extend(under);
            }
        }
        parts.extend(&self.loose);
        Some(parts)
    }

    fn owner_of(&mut self, bits: u64) -> Option<u64> {
        if self.is_object(bits) {
            return None;
        }
        self.objects
            .iter()
            .find(|(_, under)| under.contains(&bits))
            .map(|(o, _)| *o)
    }
}

/// ⭐⭐ **Em Object o clique nomeia o OBJECTO** (spec/06 F3: *«clicar numa forma selecciona o objecto
/// inteiro»*) — cada forma de `hits` sobe ao objecto vetorial dela, sem repetidos e na ordem. Em Edit
/// (`view.editing` armado) a lista fica: as formas escolhem-se uma a uma. A porta do clique, do laço
/// e do realce.
pub fn lift_to_objects(sim: &SimWorld, view: &ph2d_vec_scene::VecViewState, hits: &mut Vec<u64>) {
    if view.editing.is_some() {
        return;
    }
    let mut out: Vec<u64> = Vec::with_capacity(hits.len());
    for b in hits.iter() {
        let o = object_of(sim, Entity::from_bits(*b)).map_or(*b, Entity::to_bits);
        if !out.contains(&o) {
            out.push(o);
        }
    }
    *hits = out;
}

/// ⭐ **O smoke do modo** — `PH2D_OBJECT_MODE_SMOKE=6`: *Add ▸ Vector Object* (nasce em Edit),
/// um rectângulo e uma elipse por cima, a booleana UNION pelo botão do painel, `Tab`; depois um
/// segundo objecto com uma estrela, em Edit com o Node, e o seletor aberto — a foto mostra os nós
/// SÓ da estrela, e a união do primeiro intocada. Corre uma vez.
pub fn smoke_step(
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    sim: &SimWorld,
    hero: &mut HeroScreen,
) -> Option<ModeRequest> {
    use ph2d_vec_scene::ShapeKind;
    use std::sync::atomic::{AtomicU8, Ordering};
    // 0 = por ler · 1 = o 1.º objecto · 2 = o rectângulo · 3 = a elipse · 4 = a booleana ·
    // 5 = o `Tab` · 6 = o 2.º objecto · 7 = a estrela · 8 = o Node · 10 = o seletor · 9 = feito.
    static STAGE: AtomicU8 = AtomicU8::new(0);
    let stage = match STAGE.load(Ordering::Relaxed) {
        0 => {
            let want = match std::env::var("PH2D_OBJECT_MODE_SMOKE").as_deref() {
                Ok("6") => 1,
                _ => 9,
            };
            STAGE.store(want, Ordering::Relaxed);
            want
        }
        s => s,
    };
    let go = |next| STAGE.store(next, Ordering::Relaxed);
    // Toda forma já assentou num objecto (a regra das soltas corre no assentamento da árvore).
    let settled = vec
        .entities
        .values()
        .all(|b| object_of(sim, Entity::from_bits(*b)).is_some());
    let editing =
        settled && hero.gizmo.mode.current() == ObjectMode::Edit && vec.edit.objects.len() == 1;
    let shapes = vec.entities.len();
    match stage {
        1 | 6 if hero.gizmo.mode.current() == ObjectMode::Object => {
            hero.store
                .set_command_pick(crate::object_add::VECTOR_OBJECT.id());
            go(stage + 1);
        }
        2 if editing => {
            draw_shape(vec, scene, ShapeKind::Rectangle, [-1.6, 0.0], 0.7);
            go(3);
        }
        3 if editing && shapes == 1 => {
            draw_shape(vec, scene, ShapeKind::Ellipse, [-1.1, 0.4], 0.6);
            go(4);
        }
        4 if editing && shapes == 2 => {
            let ids: Vec<VecPathId> = vec.entities.keys().copied().collect();
            vec.pen.select_many(&ids);
            hero.bus
                .push(ph2d_editor_core::action_bus::EditorAction::ToolPanelEvent(
                    ph2d_editor_core::tool::PanelEvent::Click(
                        ph2d_panel_vector::ids::VECTOR_BOOL_UNION,
                    ),
                ));
            go(5);
        }
        5 if editing && shapes == 1 => {
            go(6);
            return Some(ModeRequest::Toggle);
        }
        7 if editing && shapes == 1 => {
            draw_shape(vec, scene, ShapeKind::Star, [1.4, 0.0], 0.7);
            go(8);
        }
        8 if editing && shapes == 2 => {
            // A estrela na mão do Node: os nós dela aparecem, e os da união (outro objecto) não.
            let ours: Vec<VecPathId> = vec
                .entities
                .iter()
                .filter(|(_, b)| object_of(sim, Entity::from_bits(**b)) == vec.edit.object())
                .map(|(id, _)| *id)
                .collect();
            vec.pen.select_many(&ours);
            hero.bus
                .push(ph2d_editor_core::action_bus::EditorAction::ToolPanelEvent(
                    ph2d_editor_core::tool::PanelEvent::Click(
                        ph2d_tool_vector::ids::VECTOR_MODE_NODE,
                    ),
                ));
            go(10);
        }
        10 if editing => {
            // O chip só se abre depois de pintado (o pulldown ancora-se no rect dele).
            let chip = ph2d_editor_core::ids::area_menu_button(0);
            if hero.hit_index.rect_for(chip).is_some() {
                go(9);
                hero.apply_event(ph2d_editor_core::interaction::WidgetEvent::Click(chip));
            }
        }
        _ => {}
    }
    None
}

/// Desenha uma forma pela FERRAMENTA de forma (o mesmo commit do gesto do artista), de lado `2 ×
/// half` centrada em `at` (mundo, metros), com os valores de fábrica dela.
fn draw_shape(
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    kind: ph2d_vec_scene::ShapeKind,
    at: [f64; 2],
    half: f64,
) {
    let c = ph2d_vec_edit::ShapeConstraint::default();
    let px_to_world = 0.01;
    let corner = [at[0] - half, at[1] - half];
    vec.shape
        .on_press(scene, kind, kind.defaults(), corner, px_to_world, c);
    vec.shape.on_drag(scene, [at[0] + half, at[1] + half], c);
    vec.shape.on_release(scene);
}

#[cfg(test)]
#[path = "vector_mode_tests.rs"]
mod tests;
