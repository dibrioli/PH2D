//! ⭐⭐ **FLIP ▸ DRAW · EDIT** — os modos que esta família declara e como eles abrem o Flip SOBRE o
//! desenho da entidade activa (spec/06 F3; D6: *desenho Flip → Object · Draw · Edit*, a correcção 1
//! do dono: o Flip tem modo PRÓPRIO, como o Grease Pencil).
//!
//! - **Draw** = a ferramenta Flip na mão com uma das ferramentas de pôr/tirar tinta
//!   ([`FlipMode::DRAW_TOOLS`]); **Edit** = com uma das que mexem no traço que já existe
//!   ([`FlipMode::EDIT_TOOLS`]). O painel só mostra as do modo em curso.
//! - **Object** = a ferramenta Flip sai da mão: o gizmo do objecto é o da ferramenta de omissão.
//! - ⭐ **O ALVO** ([`ph2d_flip::FlipTarget::object`]) é escrito AQUI e só aqui: todo gesto, painel e
//!   pré-visualização cai no desenho do modo — com dois desenhos, o outro fica intocado.
//!
//! ⭐ **Duas portas para chegar ao modo, uma regra** ([`ModeFamily::wants`]):
//! - um desenho que NASCE pelo menu Add pede o Draw (escolha do dono, 03/10: nasce vazio, o próximo
//!   gesto é desenhar);
//! - a ferramenta Flip que chega à mão por uma porta antiga (as cenas `PH2D_FLIP_*_SMOKE`) pede o modo
//!   da ferramenta dela — e a ferramenta que troca de grupo (Draw ↔ Edit) também. Assim o modo é a
//!   verdade, e nenhuma porta deixa a ferramenta na mão sem o seletor o dizer.

use ph2d_component_desc::ObjectKind;
use ph2d_editor_core::object_mode::{ActiveMode, ObjectMode};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::mode_drive::ModeFamily;
use ph2d_editor_core::{ToolId, ToolRegistry};
use ph2d_flip::{FlipDoc, FlipObjectId};
use ph2d_tool_flip::{FlipMode, FlipTool};

use crate::state::FlipState;

/// O id da ferramenta Flip no registo.
const FLIP: &str = "flip";

/// ⭐ **A família**, construída em cada quadro com o que ela empresta: o estado de edição do Flip
/// (o alvo, o mapa desenho↔entidade) e o documento (para saber que desenhos estão vivos).
pub struct Family<'a> {
    state: &'a mut FlipState,
    doc: &'a FlipDoc,
}

impl<'a> Family<'a> {
    /// Empresta o estado e o documento deste quadro.
    pub fn new(state: &'a mut FlipState, doc: &'a FlipDoc) -> Self {
        Self { state, doc }
    }

    /// O desenho vivo de `entity`.
    fn drawing_of(&self, entity: u64) -> Option<FlipObjectId> {
        let (oid, _) = self.state.entities.iter().find(|(_, b)| **b == entity)?;
        self.doc.object(*oid).map(|o| o.id)
    }

    fn bits_of(&self, oid: FlipObjectId) -> Option<u64> {
        self.doc.object(oid)?;
        self.state.entities.get(&oid).copied()
    }
}

/// A ferramenta do Flip na mão, se a ferramenta activa é a Flip.
fn tool_in_hand(tools: &mut ToolRegistry) -> Option<FlipMode> {
    tools
        .active_mut()
        .filter(|t| t.id() == ToolId::new(FLIP))
        .and_then(|t| t.as_any_mut().downcast_mut::<FlipTool>())
        .map(|t| t.mode())
}

/// ⭐⭐ **A LEI do «tem em mãos»**, pura: o desenho do modo é o alvo, e a ferramenta na mão é do
/// modo.
pub(crate) fn holds(mode: ObjectMode, is_target: bool, tool: Option<FlipMode>) -> bool {
    matches!(mode, ObjectMode::Draw | ObjectMode::Edit)
        && is_target
        && tool.is_some_and(|t| t.object_mode() == mode)
}

/// ⭐⭐ **A LEI da porta antiga**, pura: a ferramenta na mão cujo modo o quadro anterior não
/// seguia pede-o (`following` = o modo do Flip que o quadro anterior seguia).
pub(crate) fn adopt(tool: Option<FlipMode>, following: Option<ObjectMode>) -> Option<ObjectMode> {
    let want = tool?.object_mode();
    (following != Some(want)).then_some(want)
}

/// ⭐⭐ **A LEI do «a ferramenta segue o modo»**, pura: `true` = a ferramenta sai da mão. Só quando
/// um modo do Flip ACABOU — a ferramenta que acabou de chegar por uma porta antiga espera pelo
/// pedido dela ([`adopt`]), em vez de ser arrancada antes de o desenho ter entidade.
pub(crate) fn releases(now: Option<ObjectMode>, following: Option<ObjectMode>) -> bool {
    now.is_none() && following.is_some()
}

impl ModeFamily for Family<'_> {
    fn modes(&self) -> &'static [(ObjectKind, ObjectMode)] {
        &[
            (ObjectKind::Flip, ObjectMode::Draw),
            (ObjectKind::Flip, ObjectMode::Edit),
        ]
    }

    fn holds(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry) -> bool {
        let is_target = self
            .drawing_of(entity)
            .is_some_and(|oid| self.state.target.object == Some(oid));
        holds(mode, is_target, tool_in_hand(tools))
    }

    fn enter(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry) -> bool {
        let Some(oid) = self.drawing_of(entity) else {
            return false;
        };
        let first_tool = FlipMode::tools_of(mode).first().copied();
        let Some(first_tool) = first_tool else {
            return false;
        };
        if tool_in_hand(tools).is_none() && !tools.set_active(&ToolId::new(FLIP)) {
            return false;
        }
        if self.state.target.object != Some(oid) {
            // Outro desenho: a camada e as chaves marcadas eram do anterior (o multiframe agiria
            // à distância sobre quadros que o artista já não vê).
            self.state.target.layer = None;
            self.state.strip.selection.clear();
        }
        self.state.target.object = Some(oid);
        if self.state.born == Some(oid) {
            self.state.born = None;
        }
        if let Some(tool) = tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<FlipTool>())
            && tool.mode().object_mode() != mode
        {
            let last = self.state.last_tool.get(&mode).copied();
            tool.set_mode(last.unwrap_or(first_tool));
        }
        true
    }

    fn leave(&mut self, _: ObjectMode, _: u64, tools: &mut ToolRegistry) {
        self.state.target.object = None;
        if tool_in_hand(tools).is_some() {
            tools.activate_default();
        }
    }

    fn follow(&mut self, current: Option<ActiveMode>, tools: &mut ToolRegistry) {
        let ours = current
            .filter(|a| matches!(a.mode, ObjectMode::Draw | ObjectMode::Edit))
            .and_then(|a| Some((a.mode, self.drawing_of(a.entity)?)));
        let tool = tool_in_hand(tools);
        if let Some((mode, oid)) = ours {
            self.state.target.object = Some(oid);
            if let Some(t) = tool.filter(|t| t.object_mode() == mode) {
                self.state.last_tool.insert(mode, t);
            }
        } else if releases(None, self.state.following) {
            self.state.target.object = None;
            if tool.is_some() {
                tools.activate_default();
            }
        }
        self.state.following = ours.map(|(m, _)| m);
    }

    fn wants(&mut self, tools: &mut ToolRegistry) -> Option<(u64, ObjectMode)> {
        if let Some(born) = self.state.born {
            // Espera a entidade do desenho (a ponte pode só a criar no quadro seguinte).
            let bits = self.bits_of(born)?;
            self.state.born = None;
            return Some((bits, ObjectMode::Draw));
        }
        // A porta antiga: o desenho é o alvo vivo, senão o primeiro — o que a cena que pôs a
        // ferramenta na mão acabou de criar.
        let want = adopt(tool_in_hand(tools), self.state.following)?;
        let oid = self
            .state
            .target
            .drawing(self.doc)
            .or_else(|| self.doc.objects().first())?
            .id;
        Some((self.bits_of(oid)?, want))
    }
}

/// ⭐ **O smoke do modo** — `PH2D_OBJECT_MODE_SMOKE=4` acrescenta um desenho Flip pela porta do menu
/// Add (ele nasce em Draw) e abre o seletor *Mode* sobre ele, com as três faces. É como a foto do
/// passo do smoke o apanha (`docs/Components/ferramentas/fotografa_cena.sh`). Corre uma vez.
pub fn smoke_step(state: &FlipState, hero: &mut HeroScreen) {
    use std::sync::atomic::{AtomicU8, Ordering};
    // 0 = por ler · 1 = pedir o desenho · 2 = abrir o seletor em Draw · 9 = feito.
    static STAGE: AtomicU8 = AtomicU8::new(0);
    let stage = match STAGE.load(Ordering::Relaxed) {
        0 => {
            let want = match std::env::var("PH2D_OBJECT_MODE_SMOKE").as_deref() {
                Ok("4") => 1,
                _ => 9,
            };
            STAGE.store(want, Ordering::Relaxed);
            want
        }
        s => s,
    };
    match stage {
        1 => {
            if state.entities.is_empty() {
                hero.store.set_command_pick(crate::object_add::FLIP.id());
            }
            STAGE.store(2, Ordering::Relaxed);
        }
        2 if hero.gizmo.mode.current() == ObjectMode::Draw => {
            // O chip só se abre depois de pintado (o pulldown ancora-se no rect dele).
            let chip = ph2d_editor_core::ids::area_menu_button(0);
            if hero.hit_index.rect_for(chip).is_some() {
                STAGE.store(9, Ordering::Relaxed);
                hero.apply_event(ph2d_editor_core::interaction::WidgetEvent::Click(chip));
            }
        }
        _ => {}
    }
}

#[cfg(test)]
#[path = "flip_mode_tests.rs"]
mod tests;
