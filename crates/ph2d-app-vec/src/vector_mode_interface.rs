//! ⭐ **As cenas 25 e 26** (`PH2D_OBJECT_MODE_SMOKE=25|26`, F17 — dono 16/10): AS ABAS DRAWING E
//! INTERFACE (`docs/UI_New_and_Simple/vetor_nova_ui/28_plano_F17_abas_desenho_interface.md` §7). No
//! layout *Vetor*, um BOTÃO (rectângulo arredondado, cor viva) e uma estrela comum ao lado. O botão
//! vira interface pelos MESMOS cliques do painel: *Create* (prefab — a seleção segue para a cópia),
//! *Wear* (a pele nasce *Button*), o estado *Default* gravado, a cor de *Hover* gravada por cima, e o
//! *Show* do Default de volta.
//!
//! - **25**: fica na aba Drawing, a de nascença — o botão seleccionado acende o ponto na aba
//!   Interface (`state::seleccao_tem_interface`).
//! - **26**: a 25 e o clique no interruptor do fim da coluna ① ⇒ aba Interface e o palco tingido.
//!
//! Duas cenas porque o roteiro de foto fotografa o estado FINAL.

use crate::state::VecState;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_panel_vector::ids;
use ph2d_vec_scene::{Paint, Rgba8, ShapeKind, VecPathId};

/// O botão: centro (m) e meias medidas `[x, y]` — mais largo que alto.
const BOTAO: ([f64; 2], [f64; 2]) = ([-1.2, 0.0], [0.9, 0.35]);
/// A estrela: centro (m) e meia medida.
const ESTRELA: ([f64; 2], f64) = ([1.4, 0.0], 0.6);
/// A cor do botão em repouso e a do Hover (a pose gravada por cima).
const REPOUSO: [u8; 3] = [40, 120, 240];
const HOVER: [u8; 3] = [240, 140, 40];
/// A cor da estrela.
const COR_ESTRELA: [u8; 3] = [246, 196, 64];
/// O índice do papel em `StateRole::ALL`.
const DEFAULT: usize = 0;
const PAPEL_HOVER: usize = 1;

/// O primeiro estágio — um por gesto, no buraco entre a cena 19 (`110..=120`) e a 20 (`130..`).
pub(crate) const FIRST: u8 = 121;
const DESENHA_ESTRELA: u8 = FIRST + 1;
const CORES: u8 = DESENHA_ESTRELA + 1;
const PICK: u8 = CORES + 1;
const CREATE: u8 = PICK + 1;
const WEAR: u8 = CREATE + 1;
const GRAVA_DEFAULT: u8 = WEAR + 1;
const GRAVA_HOVER: u8 = GRAVA_DEFAULT + 1;
pub(crate) const LAST: u8 = GRAVA_HOVER + 1;

fn solido(c: [u8; 3]) -> Option<Paint> {
    Some(Paint::Solid(Rgba8::new(c[0], c[1], c[2], 255)))
}

/// O caminho único seleccionado.
fn so_um(vec: &VecState) -> Option<VecPathId> {
    match vec.pen.selected_paths() {
        [only] => Some(*only),
        _ => None,
    }
}

/// O papel `i` já está gravado no hospedeiro seleccionado (o que a shell publica ao painel).
fn gravado(i: usize) -> bool {
    ph2d_panel_vector::state::ui_states_state().is_some_and(|s| s.recorded[i])
}

/// Um passo das cenas 25/26 (`interface` = a 26); devolve o próximo estágio, se mudou.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    sim: &ph2d_ecs::SimWorld,
    hero: &mut HeroScreen,
    interface: bool,
) -> Option<u8> {
    let feitas = vec.entities.len();
    let botao = vec.entities.keys().next().copied();
    match stage {
        FIRST if feitas == 0 => {
            // Pela ferramenta de forma, com cantos de medidas diferentes em x e y.
            let ((c, h), kind) = (BOTAO, ShapeKind::RoundRect);
            let k = ph2d_vec_edit::ShapeConstraint::default();
            let a = [c[0] - h[0], c[1] - h[1]];
            vec.shape.on_press(scene, kind, kind.defaults(), a, 0.01, k);
            vec.shape.on_drag(scene, [c[0] + h[0], c[1] + h[1]], k);
            vec.shape.on_release(scene);
            Some(DESENHA_ESTRELA)
        }
        DESENHA_ESTRELA if feitas == 1 => {
            let (at, half) = ESTRELA;
            crate::vector_mode::draw_shape(vec, scene, ShapeKind::Star, at, half);
            Some(CORES)
        }
        CORES if feitas == 2 => {
            let ids: Vec<_> = vec.entities.keys().copied().collect();
            for (id, cor) in ids.into_iter().zip([REPOUSO, COR_ESTRELA]) {
                if let Some(p) = scene.path_mut(id) {
                    p.fill = solido(cor);
                }
            }
            hero.apply_event(WidgetEvent::Click(
                ph2d_tool_vector::ids::VECTOR_MODE_SELECT,
            ));
            Some(PICK)
        }
        // A troca de ferramenta reduz a seleção: o botão é escolhido DEPOIS dela.
        PICK if hero.store.tool_bars().is_some_and(|b| {
            b.column
                .iter()
                .flatten()
                .any(|x| x.on && x.id == ph2d_tool_vector::ids::VECTOR_MODE_SELECT)
        }) =>
        {
            vec.pen.select_many(&[botao?]);
            Some(CREATE)
        }
        CREATE if so_um(vec) == botao => {
            hero.apply_event(WidgetEvent::Click(ids::VECTOR_COMPONENT_CREATE));
            Some(WEAR)
        }
        // O *Create* põe uma cópia no lugar e a seleção segue para ela.
        WEAR if ph2d_panel_vector::state::seleccao_tem_interface()
            && so_um(vec).is_some_and(|s| Some(s) != botao) =>
        {
            hero.apply_event(WidgetEvent::Click(ids::VECTOR_WIDGET_WEAR));
            Some(GRAVA_DEFAULT)
        }
        GRAVA_DEFAULT
            if so_um(vec)
                .and_then(|s| vec.entities.get(&s))
                .is_some_and(|&b| {
                    sim.world()
                        .get::<ph2d_ecs::VecWidget>(ph2d_ecs::Entity::from_bits(b))
                        .is_some()
                }) =>
        {
            hero.apply_event(WidgetEvent::Click(ids::vector_state_record_id(DEFAULT)));
            Some(GRAVA_HOVER)
        }
        GRAVA_HOVER if gravado(DEFAULT) => {
            let copia = so_um(vec)?;
            scene.path_mut(copia)?.fill = solido(HOVER);
            hero.apply_event(WidgetEvent::Click(ids::vector_state_record_id(PAPEL_HOVER)));
            Some(LAST)
        }
        LAST if gravado(PAPEL_HOVER) => {
            hero.apply_event(WidgetEvent::Click(ids::vector_state_apply_id(DEFAULT)));
            if interface {
                hero.apply_event(WidgetEvent::Click(ids::VECTOR_VIEW_INTERFACE));
            }
            Some(9)
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "vector_mode_interface_tests.rs"]
mod tests;
