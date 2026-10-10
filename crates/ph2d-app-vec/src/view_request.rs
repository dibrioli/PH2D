//! ⭐ **Um pedido de enquadramento vira um destino da câmera** (F8) — o botão VIEW, o `F`/`Home`, o
//! menu *Ver* e o beliscão rápido. A shell junta as fontes do quadro ([`Sources`]); a lei é a
//! `ph2d_render::camera_frame` e as caixas são as [`crate::frame_boxes`]. Os que enquadram VOAM
//! (o tique é da shell); o centrar salta, como sempre (as cenas de smoke contam com isso).

use crate::frame_boxes;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor_core::ViewFocusKind;
use ph2d_host::WindowSize;
use ph2d_render::camera_frame::{self, Area, ViewFlight};
use ph2d_render::{Camera2d, WorldBbox};
use ph2d_vec_scene::{VecScene, VecXforms};

/// ⭐ **A vista entre quadros**: o voo, e o pedido ADIADO — um pedido resolve-se no quadro SEGUINTE
/// ao que o fez, quando o que nasceu nesse quadro (uma cena montada e enquadrada de uma vez) já foi
/// desenhado e entra na caixa (medido: as mídias presas a ossos saíam cortadas).
#[derive(Clone, Copy, Debug, Default)]
pub struct ViewState {
    pub flight: ViewFlight,
    held: Option<ViewFocusKind>,
}

impl ViewState {
    /// Guarda o pedido deste quadro e devolve o do anterior.
    pub fn defer(&mut self, now: Option<ViewFocusKind>) -> Option<ViewFocusKind> {
        std::mem::replace(&mut self.held, now)
    }
}

/// O que o quadro tem à mão para responder a um pedido.
pub struct Sources<'a> {
    pub sim: &'a SimWorld,
    pub present: &'a mut ph2d_ecs::World,
    pub scene: &'a VecScene,
    pub xf: &'a VecXforms,
    /// As formas que a ordem do quadro desenha.
    pub drawn_paths: Vec<u64>,
    /// As caixas desenhadas das famílias que esta crate não conhece (o Flip).
    pub others: Vec<WorldBbox>,
    /// A caixa de UM objecto dessas famílias.
    pub other_box: &'a mut dyn FnMut(Entity) -> Option<WorldBbox>,
    /// A seleção: a primária e as extras.
    pub selection: Vec<u64>,
    /// A área onde o conteúdo pode ficar, a janela da câmera e a margem — px físicos.
    pub area: ph2d_editor_core::zones::Rect,
    pub window: WindowSize,
    pub margin: f32,
}

/// Responde a `kind`: põe o voo a caminho (ou centra a câmera) e devolve a chave do aviso, `None`
/// quando o pedido é silencioso (o beliscão ao contrário, como no Procreate).
pub fn resolve(
    kind: ViewFocusKind,
    src: Sources<'_>,
    camera: &mut Camera2d,
    flight: &mut ViewFlight,
    track: Option<f32>,
) -> Option<&'static str> {
    let Sources {
        sim,
        present,
        scene,
        xf,
        drawn_paths,
        others,
        other_box,
        selection,
        area,
        window,
        margin,
    } = src;
    let (x, y, w, h) = (area.x, area.y, area.w, area.h);
    let target = |b| camera_frame::frame(b, Area { x, y, w, h }, window, margin);
    let mut selected = || {
        let mut acc = None;
        for bits in &selection {
            let e = Entity::from_bits(*bits);
            frame_boxes::grow(
                &mut acc,
                frame_boxes::object_box(sim, present, scene, xf, e, other_box),
            );
        }
        acc
    };
    Some(match kind {
        ViewFocusKind::All => {
            let b = frame_boxes::scene_box(present, scene, xf, drawn_paths, others);
            let (to, key) = match b.and_then(target) {
                Some(v) => (v, "shell.view.view_all"),
                None => (
                    Camera2d::default().view(),
                    "shell.view.view_all_empty_scene",
                ),
            };
            flight.go_remembering(to, track);
            key
        }
        ViewFocusKind::FrameSelected => match selected().and_then(target) {
            Some(v) => {
                flight.go(v, track);
                "shell.view.view_selected"
            }
            None => "shell.view.view_selected_no",
        },
        ViewFocusKind::Selected => {
            let b = selected();
            camera.center = b.map_or([0.0, 0.0], |b| b.center_half().0);
            if b.is_some() {
                "shell.view.view_selected"
            } else {
                "shell.view.view_selected_no"
            }
        }
        // Ainda não há objecto-câmera: a origem.
        ViewFocusKind::Camera => {
            camera.center = [0.0, 0.0];
            "shell.view.view_camera_origin"
        }
        ViewFocusKind::Previous => {
            flight.go_back(track);
            return None;
        }
    })
}

#[cfg(test)]
#[path = "view_request_tests.rs"]
mod tests;
