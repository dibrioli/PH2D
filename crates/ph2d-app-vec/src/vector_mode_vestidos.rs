//! ⭐ **As cenas 27 e 28** (`PH2D_OBJECT_MODE_SMOKE=27|28`, F18 — dono 17/10): O DESENHO É O CORPO
//! DO WIDGET (`docs/UI_New_and_Simple/vetor_nova_ui/29_plano_F18_o_desenho_e_o_corpo_do_widget.md`
//! §7). Começam pela 25 inteira (o botão azul feito componente, a pele *Button*, Default e Hover
//! gravados) e acrescentam uma fila de quatro formas, cada uma pintada de uma cor e vestida de um
//! tipo pelo chip do painel: Toggle verde, Slider roxo, Checkbox claro, campo de texto escuro.
//!
//! - **27**: tudo em repouso, o botão seleccionado — cada widget na cor do artista, o rótulo
//!   legível sobre ela.
//! - **28**: a 27 e o *Show* do Hover ⇒ o botão fica LARANJA e o rótulo reescolhe a tinta.

use crate::state::VecState;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::widget::WidgetKind;
use ph2d_panel_vector::ids;
use ph2d_vec_scene::{Paint, Rgba8, ShapeKind, VecPathId};

/// A fila: centro (m) de cada forma, a cor e o tipo que veste.
const FILA: [([f64; 2], [u8; 3], WidgetKind); 4] = [
    ([-2.4, -1.4], [52, 168, 83], WidgetKind::Toggle),
    ([-0.8, -1.4], [124, 77, 255], WidgetKind::Slider),
    ([0.8, -1.4], [236, 236, 228], WidgetKind::Checkbox),
    ([2.4, -1.4], [32, 36, 44], WidgetKind::TextInput),
];
/// Meias medidas `[x, y]` de cada forma da fila.
const MEIA: [f64; 2] = [0.65, 0.25];
/// O índice do papel Hover em `StateRole::ALL`.
const PAPEL_HOVER: usize = 1;
/// O que a cena 25 deixa: o original, a estrela e a cópia vestida.
const DA_CENA_25: usize = 3;

/// O primeiro estágio — no buraco entre a cena 21 (`140..=213`) e a 22 (`220..`).
pub(crate) const FIRST: u8 = 214;
const VESTE: u8 = FIRST + 1;
pub(crate) const LAST: u8 = VESTE + FILA.len() as u8;

/// O caminho único seleccionado.
fn so_um(vec: &VecState) -> Option<VecPathId> {
    match vec.pen.selected_paths() {
        [only] => Some(*only),
        _ => None,
    }
}

/// O tipo que esta forma veste, se veste.
fn veste(vec: &VecState, sim: &ph2d_ecs::SimWorld, id: VecPathId) -> Option<WidgetKind> {
    let &bits = vec.entities.get(&id)?;
    let w = sim
        .world()
        .get::<ph2d_ecs::VecWidget>(ph2d_ecs::Entity::from_bits(bits))?;
    WidgetKind::from_code(w.kind)
}

/// As formas da fila, pela ordem de [`FILA`] (as mais novas do mapa).
fn fila(vec: &VecState) -> Vec<VecPathId> {
    let ids: Vec<_> = vec.entities.keys().copied().collect();
    ids.get(DA_CENA_25..).map(<[_]>::to_vec).unwrap_or_default()
}

/// Um passo das cenas 27/28 (`hover` = a 28); devolve o próximo estágio, se mudou.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    sim: &ph2d_ecs::SimWorld,
    hero: &mut HeroScreen,
    hover: bool,
) -> Option<u8> {
    let feitas = vec.entities.len();
    match stage {
        // Uma forma por quadro: a ferramenta guarda UM pedido de nascimento vivo (`pending_live`),
        // e quatro gestos no mesmo quadro deixariam as três primeiras como caminhos mortos.
        FIRST
            if (DA_CENA_25..DA_CENA_25 + FILA.len()).contains(&feitas)
                && vec.shape.pending_live().is_none() =>
        {
            let i = feitas - DA_CENA_25;
            let (c, _, _) = FILA[i];
            let k = ph2d_vec_edit::ShapeConstraint::default();
            let kind = ShapeKind::RoundRect;
            let a = [c[0] - MEIA[0], c[1] - MEIA[1]];
            vec.shape.on_press(scene, kind, kind.defaults(), a, 0.01, k);
            vec.shape.on_drag(scene, [c[0] + MEIA[0], c[1] + MEIA[1]], k);
            vec.shape.on_release(scene);
            (i + 1 == FILA.len()).then_some(VESTE)
        }
        // Uma forma por estágio: pinta, selecciona e — com a seleção já assente — o chip do tipo.
        s if (VESTE..LAST).contains(&s) && feitas == DA_CENA_25 + FILA.len() => {
            let i = usize::from(s - VESTE);
            let formas = fila(vec);
            let (_, cor, kind) = FILA[i];
            let id = formas[i];
            if i > 0 && veste(vec, sim, formas[i - 1]) != Some(FILA[i - 1].2) {
                return None;
            }
            if so_um(vec) != Some(id) {
                scene.path_mut(id)?.fill =
                    Some(Paint::Solid(Rgba8::new(cor[0], cor[1], cor[2], 255)));
                vec.pen.select_many(&[id]);
                return None;
            }
            let chip = WidgetKind::ALL.iter().position(|&k| k == kind)?;
            hero.apply_event(WidgetEvent::Click(ids::vector_widget_kind_id(chip)));
            Some(s + 1)
        }
        LAST => {
            let formas = fila(vec);
            if veste(vec, sim, *formas.last()?) != Some(FILA[FILA.len() - 1].2) {
                return None;
            }
            let botao = vec
                .entities
                .keys()
                .copied()
                .find(|&id| veste(vec, sim, id) == Some(WidgetKind::Button))?;
            if so_um(vec) != Some(botao) {
                vec.pen.select_many(&[botao]);
                return None;
            }
            if hover {
                hero.apply_event(WidgetEvent::Click(ids::vector_state_apply_id(PAPEL_HOVER)));
            }
            Some(9)
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "vector_mode_vestidos_tests.rs"]
mod tests;
