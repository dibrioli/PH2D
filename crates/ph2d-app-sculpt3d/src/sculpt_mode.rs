//! ⭐⭐ **SCULPT ▸ SCULPT · PAINT** — os modos que esta família declara e como eles abrem a escultura
//! SOBRE a peça da entidade activa (spec/06 F3; D6: *malha 3D → Object · Sculpt · Paint*).
//!
//! - **Sculpt** = o barro na tela, com a peça desta entidade activa e PRESA ([`Sculpt3dScene::preso`]:
//!   a mira do pen-down não salta para outra peça — o Sculpt do Blender só toca o objecto activo).
//! - **Paint** = o mesmo, com o Painter em mãos: ele prende a tela da vista e pousa a tinta na peça
//!   ([`crate::painter_na_malha`]). ⛔ O IMG e a aba do Painter deixaram de ser porta.
//! - **Object** = o barro sai da tela pela porta do módulo ([`Sculpt3dScene::toggle_clay`]: para a
//!   LUZ, a ordem do ciclo).
//!
//! ⭐ **O barro SEGUE o modo** ([`ModeFamily::follow`]): fora de um modo desta família ele sai, e
//! dentro a peça activa volta à presa. Assim nenhuma porta antiga (o `D`, o MODEL que toma o
//! canvas, um load) deixa o barro na tela sem o modo o dizer, nem o modo a dizer barro sem ele.
//!
//! ⭐ **A peça que NASCE entra em Sculpt** (escolha do dono, 03/10: fora do modo ela não se desenha):
//! a cena marca-a ao nascer ([`Sculpt3dScene::pede_o_modo`]) e o quadro do modo selecciona a
//! entidade e entra ([`ModeFamily::wants`]) — pelas cinco portas de nascer, sem uma linha na shell.

use ph2d_component_desc::ObjectKind;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::object_mode::{ActiveMode, ObjectMode};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::mode_drive::ModeFamily;
use ph2d_editor_core::{ToolId, ToolRegistry};
use ph2d_tool_painter::PainterTool;

use crate::Sculpt3dScene;
use crate::entities::{SculptEntityMap, world_map};
use crate::objects::ObjectId;

/// O id do Painter no registo de ferramentas.
const PAINTER: &str = "painter";

/// ⭐ **A família**, construída em cada quadro com o que ela empresta: a cena (se há) e o mapa
/// peça↔entidade lido do mundo.
pub struct Family<'a> {
    scene: Option<&'a mut Sculpt3dScene>,
    pieces: SculptEntityMap,
}

impl<'a> Family<'a> {
    /// Lê o mapa peça↔entidade do mundo — sem cena, não há peça a mapear.
    pub fn new(sim: &mut SimWorld, scene: Option<&'a mut Sculpt3dScene>) -> Self {
        let pieces = if scene.is_some() {
            world_map(sim)
        } else {
            SculptEntityMap::new()
        };
        Self { scene, pieces }
    }

    fn piece_of(&self, entity: u64) -> Option<ObjectId> {
        self.pieces
            .iter()
            .find(|(_, bits)| **bits == entity)
            .map(|(piece, _)| ObjectId(*piece))
    }
}

/// O que a cena e o registo têm em mãos agora — os factos que [`holds`] lê.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Hands {
    /// O barro está na tela.
    pub(crate) clay: bool,
    /// A peça da entidade existe na cena.
    pub(crate) piece: bool,
    /// O Painter está em mãos.
    pub(crate) painter: bool,
}

/// ⭐⭐ **A LEI do «tem em mãos»**, pura: o barro na tela com a peça viva; no Sculpt, sem o Painter;
/// no Paint, com ele.
pub(crate) fn holds(mode: ObjectMode, h: Hands) -> bool {
    h.clay
        && h.piece
        && match mode {
            ObjectMode::Sculpt => !h.painter,
            ObjectMode::Paint => h.painter,
            _ => false,
        }
}

/// O que o barro faz para seguir o modo — a resposta de [`follow`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Follow {
    /// Um modo desta família tem a peça: ela fica presa e activa.
    Hold(ObjectId),
    /// Nenhum modo a tem: o barro sai e a presa solta-se.
    Release,
    /// Uma peça nasceu e espera a entidade dela: nada muda ainda.
    Wait,
}

/// ⭐⭐ **A LEI do «o barro segue o modo»**, pura. `current` é o modo em curso com a peça da
/// entidade dele (se ela for desta cena); `born` = uma peça nasceu e pede o modo.
pub(crate) fn follow(current: Option<(ObjectMode, Option<ObjectId>)>, born: bool) -> Follow {
    match current {
        Some((ObjectMode::Sculpt | ObjectMode::Paint, Some(piece))) => Follow::Hold(piece),
        _ if born => Follow::Wait,
        _ => Follow::Release,
    }
}

fn painter_in_hand(tools: &ToolRegistry) -> bool {
    tools
        .active()
        .is_some_and(|t| t.id() == ToolId::new(PAINTER))
}

/// O Painter em mãos está PRESO à tela da peça (e não sobre uma imagem).
fn painter_on_the_piece(tools: &mut ToolRegistry) -> bool {
    tools
        .active_mut()
        .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
        .is_some_and(|p| p.on_screen_canvas())
}

impl ModeFamily for Family<'_> {
    fn modes(&self) -> &'static [(ObjectKind, ObjectMode)] {
        &[
            (ObjectKind::Sculpt3D, ObjectMode::Sculpt),
            (ObjectKind::Sculpt3D, ObjectMode::Paint),
        ]
    }

    fn holds(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry) -> bool {
        let piece = self.piece_of(entity);
        let Some(scene) = self.scene.as_deref() else {
            return false;
        };
        let hands = Hands {
            clay: scene.clay_on_screen(),
            piece: piece.and_then(|p| scene.index_of(p)).is_some(),
            painter: painter_in_hand(tools),
        };
        holds(mode, hands)
    }

    fn enter(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry) -> bool {
        let piece = self.piece_of(entity);
        let Some(scene) = self.scene.as_deref_mut() else {
            return false;
        };
        let Some((piece, i)) = piece.and_then(|p| Some((p, scene.index_of(p)?))) else {
            return false;
        };
        scene.active = i;
        scene.preso = Some(piece);
        scene.pede_o_modo = false;
        if !scene.clay_on_screen() {
            scene.toggle_clay();
        }
        match mode {
            ObjectMode::Sculpt => {
                if painter_in_hand(tools) {
                    tools.activate_default();
                }
                true
            }
            ObjectMode::Paint => painter_in_hand(tools) || tools.set_active(&ToolId::new(PAINTER)),
            ObjectMode::Object | ObjectMode::Draw | ObjectMode::Edit => false,
        }
    }

    fn leave(&mut self, _: ObjectMode, _: u64, tools: &mut ToolRegistry) {
        if let Some(scene) = self.scene.as_deref_mut() {
            scene.preso = None;
            if scene.clay_on_screen() {
                scene.toggle_clay();
            }
        }
        if painter_in_hand(tools) {
            tools.activate_default();
        }
    }

    fn follow(&mut self, current: Option<ActiveMode>, tools: &mut ToolRegistry) {
        let current = current.map(|a| (a.mode, self.piece_of(a.entity)));
        let Some(scene) = self.scene.as_deref_mut() else {
            return;
        };
        match follow(current, scene.pede_o_modo) {
            Follow::Hold(piece) => {
                scene.preso = Some(piece);
                if let Some(i) = scene.index_of(piece) {
                    scene.active = i;
                }
            }
            Follow::Release => {
                scene.preso = None;
                if scene.clay_on_screen() {
                    scene.toggle_clay();
                }
                // O Painter que pintava a peça não fica em mãos sobre nada.
                if painter_on_the_piece(tools) {
                    tools.activate_default();
                }
            }
            Follow::Wait => {}
        }
    }

    fn wants(&mut self, _: &mut ToolRegistry) -> Option<(u64, ObjectMode)> {
        let scene = self.scene.as_deref_mut()?;
        if !scene.pede_o_modo {
            return None;
        }
        let piece = scene.objects.get(scene.active)?.id;
        let bits = *self.pieces.get(&piece.0)?;
        scene.pede_o_modo = false;
        Some((bits, ObjectMode::Sculpt))
    }
}

/// ⭐ **O smoke do modo** — `PH2D_OBJECT_MODE_SMOKE=3` acrescenta uma Esfera pela porta do menu Add
/// (ela nasce em Sculpt) e abre o seletor *Mode* sobre ela, com as três faces. É como a foto do passo
/// do smoke o apanha (`docs/Components/ferramentas/fotografa_cena.sh`). Corre uma vez.
pub fn smoke_step(sim: &mut SimWorld, hero: &mut HeroScreen) {
    use std::sync::atomic::{AtomicU8, Ordering};
    // 0 = por ler · 1 = pedir a esfera · 2 = abrir o seletor em Sculpt · 9 = feito.
    static STAGE: AtomicU8 = AtomicU8::new(0);
    let stage = match STAGE.load(Ordering::Relaxed) {
        0 => {
            let want = match std::env::var("PH2D_OBJECT_MODE_SMOKE").as_deref() {
                Ok("3") => 1,
                _ => 9,
            };
            STAGE.store(want, Ordering::Relaxed);
            want
        }
        s => s,
    };
    match stage {
        1 => {
            if world_map(sim).is_empty() {
                hero.store.set_command_pick(crate::object_add::SPHERE.id());
            }
            STAGE.store(2, Ordering::Relaxed);
        }
        2 if hero.gizmo.mode.current() == ObjectMode::Sculpt => {
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
#[path = "sculpt_mode_tests.rs"]
mod tests;
