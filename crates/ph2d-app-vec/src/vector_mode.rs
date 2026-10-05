//! ⭐⭐ **VECTOR ▸ EDIT** — cada FORMA é um objecto, e o Edit abre a ferramenta do vetor sobre ela
//! (spec/06 F3 ▸ Vector, 2.ª volta; escolha do dono, 05/10: *«nada aparece no canvas ou na hierarquia
//! até que o usuário crie alguma forma ou linha. Ao desenhar algo, o objeto aparece na hierarquia no
//! modo edit»*, e a 2.ª forma é OUTRO objecto).
//!
//! - **Add ▸ Vector Drawing** não cria entidade: arma a ferramenta ([`EditTarget::arm`]) — ela chega
//!   à mão em Object, com o painel inteiro.
//! - **A forma que NASCE com a ferramenta na mão pede o Edit sobre ela** ([`ModeFamily::wants`]): a
//!   1.ª depois do Add, e cada nova num Edit (o Edit PASSA a ela — a rede `still_holds` larga a
//!   anterior e o pedido entra na nova, no mesmo quadro).
//! - **Edit** = a ferramenta `vector` na mão (qualquer `DrawMode`). ⭐⭐ **O Edit é do TIPO** (dono,
//!   05/10: *«todos os objetos daquele tipo estão em modo de edição»*, [`ModeFamily::holds_the_whole_kind`]):
//!   toda forma se agarra e é parte do Edit; escolher um objecto de outro tipo volta a Object.
//! - **A forma trancada que MORRE** (o Soldar consome os traços, a caneta junta dois caminhos) passa o
//!   Edit à [herdeira](ModeFamily::heir) — a última forma viva da caneta.
//! - **Object** = a ferramenta sai da mão.
//! - ⭐ **O ALVO** mora no [`crate::state::VecState`], fora da `VecScene` (que entra no undo).

use ph2d_component_desc::ObjectKind;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor_core::object_mode::{ActiveMode, ModeRequest, ObjectMode};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::mode_drive::ModeFamily;
use ph2d_editor_core::{ToolId, ToolRegistry};
use ph2d_tool_vector::VectorTool;
use ph2d_vec_scene::VecPathId;

use crate::state::VecState;

/// O id da ferramenta do vetor no registo.
const VECTOR: &str = "vector";

/// ⭐ **O que o Edit tem em mãos.**
#[derive(Default)]
pub struct EditTarget {
    /// As formas do Edit — a activa à frente. Vazio em Object.
    pub objects: Vec<u64>,
    /// O quadro anterior seguia um Edit do vetor.
    following: bool,
    /// O *Add ▸ Vector Drawing* pediu a ferramenta na mão (consumido quando não há modo).
    armed: bool,
    /// As formas assentadas no quadro anterior (fora de gesto) — `None` antes do 1.º quadro. Uma
    /// forma fora desta lista NASCEU.
    known: Option<Vec<u64>>,
}

impl EditTarget {
    /// *Add ▸ Vector Drawing*: a ferramenta vem à mão sem objecto; a 1.ª forma pede o Edit.
    pub fn arm(&mut self) {
        self.armed = true;
    }

    /// A ferramenta ainda espera vir à mão.
    #[must_use]
    pub fn armed(&self) -> bool {
        self.armed
    }
}

/// ⭐ **Num Edit do TIPO, o clique do Node que não acerta no vetor escolhe o objecto de OUTRO tipo
/// por baixo** (dono, 05/10: *«ao clicar num objeto de outro tipo, o objeto deve ser selecionado mas
/// em modo object»*): o 1.º dos `hits` (a porta única do pick) que não é forma. `None` = nada de
/// outro tipo ali — o Node desselecciona, como sempre. Os modos de DESENHAR não perguntam: desenhar
/// por cima de uma imagem é o uso.
#[must_use]
pub fn another_kind_under(
    whole_kind: bool,
    hits: &[u64],
    map: &ph2d_vec_entities::entities::VecEntityMap,
) -> Option<u64> {
    if !whole_kind {
        return None;
    }
    hits.iter().copied().find(|b| !map.values().any(|v| v == b))
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

/// ⭐⭐ **A LEI do «tem em mãos»**, pura: Edit, a forma é o alvo e a ferramenta está na mão.
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

/// ⭐⭐ **A LEI do nascimento**, pura: a forma nova que pede o Edit — UMA só nasceu fora de gesto
/// desde o quadro anterior (desenhar, a booleana, colar uma); várias de uma vez (carregar, colar
/// muitas) não pedem, e antes do 1.º quadro (`known` = `None`) nada nasceu.
pub(crate) fn newborn(shapes: &[u64], known: Option<&[u64]>, drawing: &[u64]) -> Option<u64> {
    let known = known?;
    let mut new = shapes
        .iter()
        .filter(|b| !known.contains(b) && !drawing.contains(b));
    let first = *new.next()?;
    new.next().is_none().then_some(first)
}

/// ⭐⭐ **A LEI da herdeira**, pura: a forma trancada (`objects[0]`) morreu e outra continua o Edit —
/// a última viva da caneta (a rede que o Soldar deixa), senão uma do Edit, senão qualquer forma (o
/// Edit é do tipo). `None` = a trancada vive, ou não resta forma.
pub(crate) fn heir(objects: &[u64], shapes: &[u64], pen: &[u64]) -> Option<u64> {
    let locked = objects.first()?;
    if shapes.contains(locked) {
        return None;
    }
    let alive = |b: &&u64| shapes.contains(b);
    pen.iter()
        .rev()
        .find(alive)
        .or_else(|| objects.iter().find(alive))
        .or_else(|| shapes.last())
        .copied()
}

/// ⭐ **A família**, construída em cada quadro com o que ela lê do mundo.
pub struct Family<'a> {
    vec: &'a mut VecState,
    /// Cada forma viva — os objectos desta família.
    shapes: Vec<u64>,
    /// As formas em gesto (a mão ainda as escreve).
    drawing: Vec<u64>,
    /// As formas que a caneta tem seleccionadas, a última no fim.
    pen: Vec<u64>,
    /// Este quadro o Edit veio pela porta antiga ([`ModeFamily::wants`] pela caneta): só aí a
    /// selecção da caneta junta — noutra entrada ela pode ser a de um Edit velho.
    via_pen: bool,
}

impl<'a> Family<'a> {
    /// Lê as formas de `sim`.
    pub fn new(vec: &'a mut VecState, sim: &SimWorld) -> Self {
        let alive = |b: u64| sim.world().get_entity(Entity::from_bits(b)).is_ok();
        let shapes: Vec<u64> = vec
            .entities
            .values()
            .copied()
            .filter(|b| alive(*b))
            .collect();
        let bits_of = |ids: &[VecPathId]| -> Vec<u64> {
            ids.iter()
                .filter_map(|id| vec.entities.get(id).copied())
                .collect()
        };
        let drawing = bits_of(&ph2d_vec_entities::transform::gesture_paths(
            &vec.pen,
            &vec.shape,
            &vec.pencil,
        ));
        let pen = bits_of(vec.pen.selected_paths());
        Self {
            vec,
            shapes,
            drawing,
            pen,
            via_pen: false,
        }
    }

    fn is_shape(&self, bits: u64) -> bool {
        self.shapes.contains(&bits)
    }
}

impl ModeFamily for Family<'_> {
    fn modes(&self) -> &'static [(ObjectKind, ObjectMode)] {
        &[(ObjectKind::Vector, ObjectMode::Edit)]
    }

    fn holds(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry) -> bool {
        let is_target = self.vec.edit.objects.first() == Some(&entity) && self.is_shape(entity);
        holds(mode, is_target, in_hand(tools))
    }

    fn enter(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry) -> bool {
        self.enter_with(mode, entity, &[], tools)
    }

    fn joins(&self, mode: ObjectMode) -> bool {
        mode == ObjectMode::Edit
    }

    fn parts_take_the_object_gizmo(&self, mode: ObjectMode) -> bool {
        mode == ObjectMode::Edit
    }

    fn holds_the_whole_kind(&self, mode: ObjectMode) -> bool {
        mode == ObjectMode::Edit
    }

    fn heir(&mut self, mode: ObjectMode, locked: u64, tools: &mut ToolRegistry) -> Option<u64> {
        let edit = &self.vec.edit;
        if mode != ObjectMode::Edit
            || !edit.following
            || edit.objects.first() != Some(&locked)
            || !in_hand(tools)
        {
            return None;
        }
        let h = heir(&edit.objects, &self.shapes, &self.pen)?;
        let shapes = &self.shapes;
        let edit = &mut self.vec.edit;
        edit.objects.retain(|b| *b != h && shapes.contains(b));
        edit.objects.insert(0, h);
        Some(h)
    }

    fn enter_with(
        &mut self,
        mode: ObjectMode,
        entity: u64,
        joined: &[u64],
        tools: &mut ToolRegistry,
    ) -> bool {
        if mode != ObjectMode::Edit || !self.is_shape(entity) {
            return false;
        }
        if !in_hand(tools) && !tools.set_active(&ToolId::new(VECTOR)) {
            return false;
        }
        let mut objects = vec![entity];
        let pen: &[u64] = if self.via_pen { &self.pen } else { &[] };
        for b in joined.iter().chain(pen) {
            if self.is_shape(*b) && !objects.contains(b) {
                objects.push(*b);
            }
        }
        // A caneta fica só com as formas que entram: uma selecção velha (de um Edit anterior)
        // mostraria nós numa forma que o artista não escolheu, e a herdeira sairia dela.
        let entities = &self.vec.entities;
        let keep: Vec<VecPathId> = self
            .vec
            .pen
            .selected_paths()
            .iter()
            .copied()
            .filter(|id| entities.get(id).is_some_and(|b| objects.contains(b)))
            .collect();
        if keep.len() != self.vec.pen.selected_paths().len() {
            self.vec.pen.select_many(&keep);
        }
        self.pen.retain(|b| objects.contains(b));
        self.vec.edit.objects = objects;
        self.vec.edit.armed = false;
        true
    }

    /// ⚠️ Não larga a ferramenta: o Edit pode estar só a PASSAR à forma que nasceu (ver o
    /// cabeçalho) — quem a larga é o [`ModeFamily::follow`], quando nenhum Edit do vetor ficou.
    fn leave(&mut self, _: ObjectMode, _: u64, _: &mut ToolRegistry) {
        self.vec.edit.objects.clear();
    }

    fn follow(&mut self, current: Option<ActiveMode>, tools: &mut ToolRegistry) {
        let ours = current
            .filter(|a| a.mode == ObjectMode::Edit && self.is_shape(a.entity))
            .map(|a| a.entity);
        let shapes = &self.shapes;
        let edit = &mut self.vec.edit;
        if let Some(o) = ours {
            if edit.objects.first() != Some(&o) {
                edit.objects = vec![o];
            }
            edit.objects.retain(|b| shapes.contains(b));
        } else if releases(false, edit.following) {
            // ⭐ O Edit caiu porque as formas dele DESAPARECERAM (a booleana, apagar — o quadro nem
            // chama o `leave`: uma entidade morta não tem tipo): a ferramenta FICA na mão, e a forma
            // que a booleana deixa, que só ganha entidade no quadro seguinte, pede o Edit então.
            let gone = !edit.objects.is_empty() && edit.objects.iter().all(|b| !shapes.contains(b));
            edit.objects.clear();
            if !gone && in_hand(tools) {
                tools.activate_default();
            }
        }
        edit.following = ours.is_some();
        // *Add ▸ Vector Drawing*: a ferramenta vem à mão quando nenhum modo a disputa (um pedido de
        // Object pode estar a caminho), com a caneta sem selecção — senão a porta antiga entraria
        // no Edit de uma forma velha.
        if edit.armed && current.is_none() {
            edit.armed = false;
            self.vec.pen.select_many(&[]);
            if !in_hand(tools) {
                tools.set_active(&ToolId::new(VECTOR));
            }
        }
        let drawing = &self.drawing;
        self.vec.edit.known = Some(
            self.shapes
                .iter()
                .copied()
                .filter(|b| !drawing.contains(b))
                .collect(),
        );
    }

    fn wants(&mut self, tools: &mut ToolRegistry) -> Option<(u64, ObjectMode)> {
        if !in_hand(tools) {
            return None;
        }
        if let Some(b) = newborn(&self.shapes, self.vec.edit.known.as_deref(), &self.drawing) {
            return Some((b, ObjectMode::Edit));
        }
        if !adopt(true, self.vec.edit.following) {
            return None;
        }
        self.via_pen = true;
        Some((*self.pen.last()?, ObjectMode::Edit))
    }

    /// ⭐ **Todas as outras formas** — o Edit é do tipo; `None` = uma forma só (o cadeado é exacto).
    fn parts(&mut self, entity: u64) -> Option<Vec<u64>> {
        if !self.is_shape(entity) {
            return None;
        }
        let parts: Vec<u64> = self
            .shapes
            .iter()
            .copied()
            .filter(|b| *b != entity)
            .collect();
        (!parts.is_empty()).then_some(parts)
    }
}

/// ⭐ **O smoke do modo** — `PH2D_OBJECT_MODE_SMOKE=6`: *Add ▸ Vector Drawing* (nada nasce: a
/// ferramenta vem à mão), um rectângulo (nasce objecto, em Edit), uma elipse por cima dele (OUTRO
/// objecto, e o Edit passa a ela), a booleana UNION pelo botão do painel (nasce UMA forma, e o Edit
/// passa a ela) e o seletor aberto — a foto mostra um objecto só na Hierarquia, em Edit, COM o gizmo.
/// `PH2D_OBJECT_MODE_SMOKE=7` (os furos, 05/10): duas linhas que se cruzam pela CANETA (o Edit passa
/// à 2.ª) e o *Weld* do painel — a solda consome a 2.ª, e a foto mostra a REDE em Edit, com o gizmo.
/// Corre uma vez.
pub fn smoke_step(
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    hero: &mut HeroScreen,
) -> Option<ModeRequest> {
    use ph2d_vec_scene::ShapeKind;
    use std::sync::atomic::{AtomicU8, Ordering};
    // 0 = por ler · 1 = o Add · 2 = a ferramenta na mão · 3 = o rectângulo · 4 = a elipse ·
    // 5 = a booleana · 6 = o Edit na forma dela · 10 = o seletor · 9 = feito. A cena 7 troca 3–6 por
    // 13 = a 1.ª linha · 14 = a 2.ª · 15 = o Weld · 16 = o Edit na rede.
    static STAGE: AtomicU8 = AtomicU8::new(0);
    static WELD: AtomicU8 = AtomicU8::new(0);
    let stage = match STAGE.load(Ordering::Relaxed) {
        0 => {
            let scene = std::env::var("PH2D_OBJECT_MODE_SMOKE");
            let want = match scene.as_deref() {
                Ok("6" | "7") => 1,
                _ => 9,
            };
            WELD.store(u8::from(scene.as_deref() == Ok("7")), Ordering::Relaxed);
            STAGE.store(want, Ordering::Relaxed);
            want
        }
        s => s,
    };
    let weld = WELD.load(Ordering::Relaxed) == 1;
    let go = |next| STAGE.store(next, Ordering::Relaxed);
    let edit = hero.gizmo.mode.current() == ObjectMode::Edit;
    let shapes = vec.entities.len();
    // A forma mais nova (os ids crescem): é nela que o Edit tem de estar.
    let newest = vec.entities.values().last().copied();
    match stage {
        1 if hero.gizmo.mode.current() == ObjectMode::Object => {
            hero.store.set_command_pick(crate::object_add::VECTOR.id());
            go(2);
        }
        // ⚠️ Pelo espelho da ferramenta activa, e não pelo `armed`: o quadro do Add traz o pedido
        // de Object, e nesse quadro este passo não corre — o `armed` nasce e morre sem ser visto.
        2 if hero.image_edit.active_tool_id == Some(VECTOR) => go(if weld { 13 } else { 3 }),
        13 if !edit => {
            draw_line(vec, scene, [-1.6, -0.4], [0.2, 1.0]);
            go(14);
        }
        14 if edit && shapes == 1 => {
            draw_line(vec, scene, [-1.6, 1.0], [0.2, -0.4]);
            go(15);
        }
        15 if edit && shapes == 2 && vec.edit.objects.first().copied() == newest => {
            let both: Vec<VecPathId> = vec.entities.keys().copied().collect();
            vec.pen.select_many(&both);
            hero.bus
                .push(ph2d_editor_core::action_bus::EditorAction::ToolPanelEvent(
                    ph2d_editor_core::tool::PanelEvent::Click(
                        ph2d_panel_vector::ids::VECTOR_PATH_WELD,
                    ),
                ));
            go(16);
        }
        // A rede fica no traço mais ao fundo (o mais VELHO): é nele que o Edit tem de estar.
        16 if edit && shapes == 1 && vec.edit.objects.first() == vec.entities.values().next() => {
            go(10);
        }
        3 if !edit => {
            draw_shape(vec, scene, ShapeKind::Rectangle, [-1.2, 0.0], 0.7);
            go(4);
        }
        4 if edit && shapes == 1 => {
            draw_shape(vec, scene, ShapeKind::Ellipse, [-0.6, 0.4], 0.7);
            go(5);
        }
        5 if edit && shapes == 2 && vec.edit.objects.first().copied() == newest => {
            let both: Vec<VecPathId> = vec.entities.keys().copied().collect();
            vec.pen.select_many(&both);
            hero.bus
                .push(ph2d_editor_core::action_bus::EditorAction::ToolPanelEvent(
                    ph2d_editor_core::tool::PanelEvent::Click(
                        ph2d_panel_vector::ids::VECTOR_BOOL_UNION,
                    ),
                ));
            go(6);
        }
        6 if edit && shapes == 1 && vec.edit.objects.first().copied() == newest => go(10),
        10 if edit => {
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

/// Desenha um traço aberto de dois cliques pela CANETA (o mesmo gesto do artista) e acaba-o.
fn draw_line(vec: &mut VecState, scene: &mut ph2d_vec_scene::VecScene, a: [f64; 2], b: [f64; 2]) {
    for p in [a, b] {
        vec.pen.on_press(scene, p, 0.01, false, &mut |q| q);
        vec.pen.on_release();
    }
    vec.pen.finish();
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

/// Os gates dos FUROS do Edit (o Soldar, a caneta que continua, o Width e o Trim) — irmão pelo tecto
/// de LOC (HR-18).
#[cfg(test)]
#[path = "vector_mode_furos_tests.rs"]
mod furos_tests;
