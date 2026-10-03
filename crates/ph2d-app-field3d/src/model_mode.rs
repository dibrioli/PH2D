//! ⭐⭐ **MODEL ▸ EDIT** — o modo que a peça sólida declara e como ele abre o módulo de modelagem
//! SOBRE a peça da entidade activa (spec/06 F3; D6: *peça sólida / SDF → Object · Edit*).
//!
//! - **Edit** = o painel `model3d` aberto (é ele que arma o módulo, `smoke::set_armed_by_panel`) com
//!   a peça da entidade EM MÃOS ([`target`] → [`crate::scene::root_in_hand`]: a que se coze, desenha
//!   e edita). As formas dela e as luzes da cena são PARTES ([`ModeFamily::parts`]): seleccioná-las
//!   não tira o modo, e outra peça fica intocada.
//! - **Object** = o painel fecha e nenhuma peça fica em mãos.
//! - ⭐ **O painel SEGUE o modo** ([`ModeFamily::follow`]): um Edit que acabou por outra porta (o X
//!   do painel, outra ferramenta que tomou o canvas, a peça apagada) fecha-o.
//! - ⭐ **Duas portas para chegar ao modo, uma regra** ([`ModeFamily::wants`]): a peça que NASCE pelo
//!   menu Add pede o Edit (escolha do dono, 03/10: fora dele não se desenha), e o painel aberto por
//!   uma porta antiga (um projecto com peça, as cenas `PH2D_FIELD_SMOKE`) pede-o sobre a peça em
//!   mãos — nenhuma cena antiga teve de ser editada.
//!
//! ⛔ O estado não mora no mundo: o mundo entra no diff do undo, e entrar num modo viraria passo de
//! histórico (a lição do `FlipTarget`).

use std::cell::Cell;

use ph2d_component_desc::ObjectKind;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToolRegistry;
use ph2d_editor_core::object_mode::{ActiveMode, ObjectMode};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::mode_drive::ModeFamily;

/// O que o modo guarda entre quadros.
#[derive(Clone, Copy, Debug, Default)]
struct State {
    /// A peça em edição (bits da raiz).
    target: Option<u64>,
    /// A peça que nasceu pelo menu Add e ainda não pediu o modo.
    born: Option<u64>,
    /// Um Edit do Model estava em curso no quadro anterior.
    following: bool,
    /// O pedido ao painel, tirado pela shell ([`take_panel_request`]).
    panel: Option<bool>,
    /// A porta antiga já pediu o Edit desde que o painel abriu.
    asked: bool,
    /// A última peça que entrou em Edit — trocar de peça enquadra a nova.
    last: Option<u64>,
}

thread_local! {
    static STATE: Cell<State> = const { Cell::new(State {
        target: None,
        born: None,
        following: false,
        panel: None,
        asked: false,
        last: None,
    }) };
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|c| {
        let mut s = c.get();
        let r = f(&mut s);
        c.set(s);
        r
    })
}

/// ⭐ **A peça em edição** — `None` em Object.
#[must_use]
pub fn target() -> Option<u64> {
    STATE.with(|c| c.get().target)
}

/// A peça `root` acabou de nascer pelo menu Add: ela pede o Edit no próximo quadro do modo.
pub fn born(root: u64) {
    with(|s| s.born = Some(root));
}

/// ⭐ **O pedido ao painel** (`true` abrir, `false` fechar), tirado uma vez — a shell escreve a
/// visibilidade, que é o que arma o módulo.
pub fn take_panel_request() -> Option<bool> {
    with(|s| s.panel.take())
}

/// Só para gates: repõe o estado, para que dois gates na mesma thread não se contaminem.
#[cfg(any(test, feature = "test-support"))]
pub fn forget() {
    STATE.with(|c| c.set(State::default()));
}

/// ⭐⭐ **A LEI do «tem em mãos»**, pura: Edit, a peça é a do modo, e o painel está aberto.
pub(crate) fn holds(mode: ObjectMode, is_target: bool, panel_open: bool) -> bool {
    mode == ObjectMode::Edit && is_target && panel_open
}

/// ⭐⭐ **A LEI do «o painel segue o modo»**, pura: `true` = um Edit do Model ACABOU (por qualquer
/// porta) e o módulo larga a peça e fecha o painel.
pub(crate) fn releases(ours_now: bool, following: bool) -> bool {
    following && !ours_now
}

/// ⭐⭐ **A LEI de quem pede o Edit**, pura: a peça nascida, senão — uma vez por abertura — a peça em
/// mãos de um painel que uma porta antiga abriu sem o modo (e que não está a fechar).
pub(crate) fn wanted(
    born: Option<u64>,
    in_hand: Option<u64>,
    panel_open: bool,
    closing: bool,
    following: bool,
    asked: bool,
) -> Option<u64> {
    born.or_else(|| (panel_open && !closing && !following && !asked).then_some(in_hand)?)
}

/// ⭐ **A família**, construída em cada quadro com o que ela lê do mundo: as peças, as partes de
/// cada uma, e se o painel está aberto.
pub struct Family {
    /// As raízes `FieldObject`, com as partes de cada uma (as formas dela).
    pieces: Vec<(u64, Vec<u64>)>,
    /// As luzes da cena — partes do Edit de QUALQUER peça (a luz é da cena, não da peça).
    lights: Vec<u64>,
    in_hand: Option<u64>,
    panel_open: bool,
}

impl Family {
    /// Lê as peças e as luzes de `sim`. `panel_open` = o painel `model3d` está visível.
    pub fn new(sim: &mut SimWorld, panel_open: bool) -> Self {
        let world = sim.world_mut();
        let in_hand = crate::scene::root_in_hand(world).map(bevy_ecs::entity::Entity::to_bits);
        let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldObject)>();
        let roots: Vec<bevy_ecs::entity::Entity> = q.iter(world).map(|(e, _)| e).collect();
        let pieces = roots
            .into_iter()
            .map(|r| {
                let parts = ph2d_field_ecs::walk(world, r)
                    .into_iter()
                    .map(|(e, _)| e.to_bits())
                    .filter(|b| *b != r.to_bits())
                    .collect();
                (r.to_bits(), parts)
            })
            .collect();
        let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldLight)>();
        let lights = q.iter(world).map(|(e, _)| e.to_bits()).collect();
        Self {
            pieces,
            lights,
            in_hand,
            panel_open,
        }
    }

    fn is_piece(&self, bits: u64) -> bool {
        self.pieces.iter().any(|(r, _)| *r == bits)
    }
}

impl ModeFamily for Family {
    fn modes(&self) -> &'static [(ObjectKind, ObjectMode)] {
        &[(ObjectKind::Model3D, ObjectMode::Edit)]
    }

    fn holds(&mut self, mode: ObjectMode, entity: u64, _: &mut ToolRegistry) -> bool {
        let is_target = self.is_piece(entity) && target() == Some(entity);
        holds(mode, is_target, self.panel_open)
    }

    fn enter(&mut self, mode: ObjectMode, entity: u64, _: &mut ToolRegistry) -> bool {
        if mode != ObjectMode::Edit || !self.is_piece(entity) {
            return false;
        }
        let other = with(|s| {
            let other = s.last.is_some_and(|l| l != entity);
            *s = State {
                target: Some(entity),
                born: s.born.filter(|b| *b != entity),
                following: true,
                panel: Some(true),
                asked: true,
                last: Some(entity),
            };
            other
        });
        // Outra peça: a câmera era da anterior.
        if other {
            crate::smoke::ask_frame_the_part();
        }
        true
    }

    fn leave(&mut self, _: ObjectMode, _: u64, _: &mut ToolRegistry) {
        with(|s| {
            s.target = None;
            s.following = false;
            s.panel = Some(false);
        });
    }

    fn follow(&mut self, current: Option<ActiveMode>, _: &mut ToolRegistry) {
        let ours = current.filter(|a| a.mode == ObjectMode::Edit && self.is_piece(a.entity));
        let panel_open = self.panel_open;
        with(|s| {
            if releases(ours.is_some(), s.following) {
                s.target = None;
                s.following = false;
                if panel_open {
                    s.panel = Some(false);
                }
            }
            if let Some(a) = ours {
                s.target = Some(a.entity);
                s.following = true;
            }
            if !panel_open && s.panel != Some(true) {
                s.asked = false;
            }
        });
    }

    fn wants(&mut self, _: &mut ToolRegistry) -> Option<(u64, ObjectMode)> {
        let born = with(|s| s.born).filter(|b| self.is_piece(*b));
        let (in_hand, panel_open) = (self.in_hand, self.panel_open);
        let bits = with(|s| {
            // Uma peça nascida que já não existe (desfeita) não pede nada.
            s.born = born;
            let closing = s.panel == Some(false);
            let bits = wanted(born, in_hand, panel_open, closing, s.following, s.asked)?;
            s.born = None;
            s.asked = true;
            Some(bits)
        })?;
        Some((bits, ObjectMode::Edit))
    }

    fn parts(&mut self, entity: u64) -> Option<Vec<u64>> {
        let (_, shapes) = self.pieces.iter().find(|(r, _)| *r == entity)?;
        Some(shapes.iter().chain(&self.lights).copied().collect())
    }

    fn owner_of(&mut self, bits: u64) -> Option<u64> {
        self.pieces
            .iter()
            .find(|(_, shapes)| shapes.contains(&bits))
            .map(|(r, _)| *r)
    }
}

/// ⭐ **O smoke do modo** — `PH2D_OBJECT_MODE_SMOKE=5` acrescenta um Model pela porta do menu Add
/// (ele nasce em Edit) e abre o seletor *Mode* sobre ele, com as duas faces. É como a foto do passo
/// do smoke o apanha (`docs/Components/ferramentas/fotografa_cena.sh`). Corre uma vez.
pub fn smoke_step(hero: &mut HeroScreen) {
    use std::sync::atomic::{AtomicU8, Ordering};
    // 0 = por ler · 1 = pedir o Model · 2 = abrir o seletor em Edit · 9 = feito.
    static STAGE: AtomicU8 = AtomicU8::new(0);
    let stage = match STAGE.load(Ordering::Relaxed) {
        0 => {
            let want = match std::env::var("PH2D_OBJECT_MODE_SMOKE").as_deref() {
                Ok("5") => 1,
                _ => 9,
            };
            STAGE.store(want, Ordering::Relaxed);
            want
        }
        s => s,
    };
    match stage {
        1 => {
            hero.store.set_command_pick(crate::object_add::MODEL.id());
            STAGE.store(2, Ordering::Relaxed);
        }
        2 if hero.gizmo.mode.current() == ObjectMode::Edit => {
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
#[path = "model_mode_tests.rs"]
mod tests;
