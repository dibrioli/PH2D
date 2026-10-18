//! ⭐ **As cenas 29 e 30** (`PH2D_OBJECT_MODE_SMOKE=29|30`, F19 — dono 18/10): O PAINEL LÊ A FORMA
//! (`docs/UI_New_and_Simple/vetor_nova_ui/30_plano_F19_o_painel_le_a_forma.md` §7). Começam pela 25
//! inteira (o botão azul feito componente, a pele *Button*, Default e Hover gravados), regravam o
//! Hover com o TRAÇO também mudado (amarelo, mais grosso, tracejado, pontas redondas) e mostram-no.
//!
//! - **29**: o *Show* do Hover ⇒ o canvas, o painel (Fill, Stroke, Width, Dash, Cap/Join) e a barra
//!   de controlo dizem o mesmo.
//! - **30**: a 29 e a Width mudada pelo painel ⇒ o botão continua laranja e amarelo, mais grosso.

use crate::state::VecState;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_panel_vector::ids;
use ph2d_vec_scene::{DashPattern, LineCap, LineJoin, Paint, Rgba8, VecPathId};

const AZUL: Rgba8 = Rgba8::new(40, 120, 240, 255);
const LARANJA: Rgba8 = Rgba8::new(240, 140, 40, 255);
const AMARELO: Rgba8 = Rgba8::new(250, 210, 60, 255);
/// O índice do papel em `StateRole::ALL`.
const DEFAULT: usize = 0;
const PAPEL_HOVER: usize = 1;
/// A Width que a cena 30 escreve pelo painel, em px de ecrã.
pub(crate) const WIDTH_PX: f64 = 9.0;

/// O primeiro estágio — depois da cena 24 (`240..=247`).
pub(crate) const FIRST: u8 = 250;
const DEFAULT_DE_VOLTA: u8 = FIRST + 1;
const SHOW: u8 = DEFAULT_DE_VOLTA + 1;
pub(crate) const LAST: u8 = SHOW + 1;

fn so_um(vec: &VecState) -> Option<VecPathId> {
    match vec.pen.selected_paths() {
        [only] => Some(*only),
        _ => None,
    }
}

/// A forma está assente nesta cor (fim da transição, nunca a meio dela).
fn assente(scene: &ph2d_vec_scene::VecScene, id: VecPathId, c: Rgba8) -> bool {
    scene
        .path(id)
        .is_some_and(|p| p.fill == Some(Paint::Solid(c)))
}

/// Um passo das cenas 29/30 (`espessura` = a 30); devolve o próximo estágio, se mudou.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    hero: &mut HeroScreen,
    espessura: bool,
) -> Option<u8> {
    let botao = so_um(vec)?;
    match stage {
        // O Hover regravado com a família inteira, depois de o Show do Default da 25 assentar.
        FIRST if assente(scene, botao, AZUL) => {
            let p = scene.path_mut(botao)?;
            p.fill = Some(Paint::Solid(LARANJA));
            let s = p.stroke.as_mut()?;
            s.paint = ph2d_vec_scene::StrokePaint::Solid(AMARELO);
            s.width *= 2.0;
            s.cap = LineCap::Round;
            s.join = LineJoin::Round;
            s.dash = Some(DashPattern::single(2.0, 1.0));
            hero.apply_event(WidgetEvent::Click(ids::vector_state_record_id(PAPEL_HOVER)));
            Some(DEFAULT_DE_VOLTA)
        }
        DEFAULT_DE_VOLTA => {
            hero.apply_event(WidgetEvent::Click(ids::vector_state_apply_id(DEFAULT)));
            Some(SHOW)
        }
        // Só depois de o Default ASSENTAR: o Show do Hover parte de uma cena azul.
        SHOW if assente(scene, botao, AZUL) => {
            hero.apply_event(WidgetEvent::Click(ids::vector_state_apply_id(PAPEL_HOVER)));
            Some(if espessura { LAST } else { 9 })
        }
        LAST if assente(scene, botao, LARANJA) => {
            let id = ph2d_tool_vector::ids::VECTOR_WIDTH;
            if let Some(ph2d_editor_core::InteractiveState::Slider { value, .. }) =
                hero.store.get_mut(id)
            {
                *value = ph2d_tool_vector::params::px_to_slider(WIDTH_PX);
            }
            // O arrasto real leva a caixa ligada junto (o despacho espelha barra ↔ chip).
            hero.store
                .set_number_value(ph2d_tool_vector::ids::VECTOR_WIDTH_NUM, WIDTH_PX);
            hero.apply_event(WidgetEvent::ValueChanged(id));
            Some(9)
        }
        _ => None,
    }
}
