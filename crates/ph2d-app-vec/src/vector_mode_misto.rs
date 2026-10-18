//! ⭐ **As cenas 31 e 32** (`PH2D_OBJECT_MODE_SMOKE=31|32`, F20 — dono 18/10): A SELEÇÃO DE DUAS
//! DIZ «MISTO» E A ESPESSURA FALA NA UNIDADE DO DOCUMENTO
//! (`docs/UI_New_and_Simple/vetor_nova_ui/31_plano_F20_misto_e_espessura_do_documento.md` §7). No
//! layout *Vetor*, uma forma VERMELHA (traço preto de 2, Round/Round, tracejado) e uma AZUL (traço
//! branco de 6, Butt/Miter, por dentro), as duas selecionadas.
//!
//! - **31**: Fill e Stroke em «misto» no painel e na barra; Width, Cap e Join sem valor aceso.
//! - **32**: a 31 e a Width mudada pelo painel para 9 ⇒ as duas mais grossas, cada uma com a SUA cor,
//!   e o «W» lê 9 (no zoom da cena, 98 %, a lei antiga mostrava 8,86).

use crate::state::VecState;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_vec_scene::{
    DashPattern, LineCap, LineJoin, Paint, Rgba8, ShapeKind, StrokeAlign, StrokeSpec,
};

/// As duas formas: o centro (m), a meia-largura (m), o preenchimento e o traço.
const FORMAS: [([f64; 2], f64); 2] = [([-1.2, 0.0], 0.8), ([1.2, 0.0], 0.8)];
const VERMELHO: Rgba8 = Rgba8::new(220, 40, 40, 255);
const AZUL: Rgba8 = Rgba8::new(40, 120, 240, 255);
const PRETO: Rgba8 = Rgba8::new(20, 20, 20, 255);
const BRANCO: Rgba8 = Rgba8::new(240, 240, 245, 255);
/// A Width que a cena 32 escreve pelo painel, em px do documento.
pub(crate) const WIDTH_PX: f64 = 9.0;

/// O primeiro estágio — no buraco entre a cena 22 (`220..=224`) e a 23 (`230..`).
pub(crate) const FIRST: u8 = 225;
const ESTILO: u8 = FIRST + FORMAS.len() as u8;
const PICK: u8 = ESTILO + 1;
pub(crate) const LAST: u8 = PICK + 1;
// Os estágios cabem no buraco e não pisam os vizinhos.
const _: () = assert!(crate::vector_mode_alinhar::LAST < FIRST);
const _: () = assert!(LAST < crate::vector_mode_controle::FIRST);

/// O estilo da forma `i` (0 = a vermelha, 1 = a azul), em px do documento.
fn estilo(i: usize, doc: ph2d_editor_core::DocPx) -> (Paint, StrokeSpec) {
    if i == 0 {
        let mut s = StrokeSpec::new(PRETO, doc.to_world(2.0));
        (s.cap, s.join) = (LineCap::Round, LineJoin::Round);
        s.dash = Some(DashPattern::single(2.0, 1.0));
        (Paint::Solid(VERMELHO), s)
    } else {
        let mut s = StrokeSpec::new(BRANCO, doc.to_world(6.0));
        s.align = StrokeAlign::Inner;
        (Paint::Solid(AZUL), s)
    }
}

/// Um passo das cenas 31/32 (`espessura` = a 32); devolve o próximo estágio, se mudou.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    hero: &mut HeroScreen,
    espessura: bool,
) -> Option<u8> {
    let feitas = vec.entities.len();
    match stage {
        s if (FIRST..ESTILO).contains(&s) && feitas == usize::from(s - FIRST) => {
            let (at, half) = FORMAS[usize::from(s - FIRST)];
            crate::vector_mode::draw_shape(vec, scene, ShapeKind::Rectangle, at, half);
            Some(s + 1)
        }
        ESTILO if feitas == FORMAS.len() => {
            let doc = ph2d_editor_core::DocPx::of(&hero.project);
            let ids: Vec<_> = vec.entities.keys().copied().collect();
            for (i, id) in ids.into_iter().enumerate() {
                let (fill, stroke) = estilo(i, doc);
                if let Some(p) = scene.path_mut(id) {
                    p.fill = Some(fill);
                    p.stroke = Some(stroke);
                }
            }
            hero.apply_event(WidgetEvent::Click(
                ph2d_tool_vector::ids::VECTOR_MODE_SELECT,
            ));
            Some(PICK)
        }
        // A troca de ferramenta reduz a seleção: as duas são escolhidas DEPOIS dela, a vermelha
        // por último (o primário).
        PICK if hero.store.tool_bars().is_some_and(|b| {
            b.column
                .iter()
                .flatten()
                .any(|x| x.on && x.id == ph2d_tool_vector::ids::VECTOR_MODE_SELECT)
        }) =>
        {
            let ids: Vec<_> = vec.entities.keys().copied().collect();
            vec.pen.select_many(&[*ids.get(1)?, *ids.first()?]);
            Some(if espessura { LAST } else { 9 })
        }
        LAST if matches!(vec.pen.selected_paths(), [_, _]) => {
            let id = ph2d_tool_vector::ids::VECTOR_WIDTH;
            if let Some(ph2d_editor_core::InteractiveState::Slider { value, .. }) =
                hero.store.get_mut(id)
            {
                *value = ph2d_tool_vector::params::px_to_slider(WIDTH_PX);
            }
            hero.store
                .set_number_value(ph2d_tool_vector::ids::VECTOR_WIDTH_NUM, WIDTH_PX);
            hero.apply_event(WidgetEvent::ValueChanged(id));
            Some(9)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vec_scene::StrokePaint;

    /// As duas formas da cena diferem em tudo o que o painel mostra — senão o «misto» não aparece.
    #[test]
    fn as_duas_formas_diferem() {
        let doc = ph2d_editor_core::DocPx::DEFAULT;
        let (fa, sa) = estilo(0, doc);
        let (fb, sb) = estilo(1, doc);
        assert_ne!(fa, fb);
        assert_ne!(sa.paint, sb.paint);
        assert!(matches!(sa.paint, StrokePaint::Solid(_)));
        assert_ne!(sa.width, sb.width);
        assert_ne!(sa.cap, sb.cap);
        assert_ne!(sa.join, sb.join);
    }
}
