//! ⭐ **As cenas 33 e 34** (`PH2D_OBJECT_MODE_SMOKE=33|34`, F21 — dono 19/10): A COR ESCOLHIDA
//! CHEGA A TODAS, E A PILHA DE CAMADAS DIZ «MISTO»
//! (`docs/UI_New_and_Simple/vetor_nova_ui/32_plano_F21_o_misto_completo.md` §7). No layout *Vetor*,
//! uma forma VERMELHA cheia (com uma camada extra: um contorno amarelo) e uma forma VAZIA (sem
//! preenchimento, traço azul, sem camadas extra), as duas selecionadas, a vermelha o primário.
//!
//! - **33**: o Fill diz «—»; a pilha diz «Mixed layers» (as estruturas diferem).
//! - **34**: a 33 e o seletor do Fill tocado no vermelho EXATO do primário ⇒ as duas vermelhas (a
//!   vazia ganhou preenchimento opaco) e o Fill deixa de ser «—».

use crate::state::VecState;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_vec_scene::{Paint, PaintEntry, Rgba8, ShapeKind, StrokeSpec};

/// As duas formas: o centro (m) e a meia-largura (m).
const FORMAS: [([f64; 2], f64); 2] = [([-1.2, 0.0], 0.8), ([1.2, 0.0], 0.8)];
pub(crate) const VERMELHO: Rgba8 = Rgba8::new(220, 40, 40, 255);
const PRETO: Rgba8 = Rgba8::new(20, 20, 20, 255);
const AZUL: Rgba8 = Rgba8::new(40, 120, 240, 255);
const AMARELO: Rgba8 = Rgba8::new(250, 200, 0, 255);

/// O primeiro estágio — no buraco entre a cena 23 (`230..=234`) e a 24 (`240..`).
pub(crate) const FIRST: u8 = 235;
const ESTILO: u8 = FIRST + FORMAS.len() as u8;
const PICK: u8 = ESTILO + 1;
pub(crate) const LAST: u8 = PICK + 1;
const _: () = assert!(crate::vector_mode_controle::LAST < FIRST);
const _: () = assert!(LAST < crate::vector_mode_tracejado::FIRST);

/// O estilo da forma `i` (0 = a vermelha cheia, 1 = a vazia), em px do documento.
fn estilo(i: usize, doc: ph2d_editor_core::DocPx) -> (Option<Paint>, StrokeSpec, Vec<PaintEntry>) {
    if i == 0 {
        let extra = vec![PaintEntry::stroke(StrokeSpec::new(
            AMARELO,
            doc.to_world(4.0),
        ))];
        (
            Some(Paint::Solid(VERMELHO)),
            StrokeSpec::new(PRETO, doc.to_world(2.0)),
            extra,
        )
    } else {
        (None, StrokeSpec::new(AZUL, doc.to_world(3.0)), Vec::new())
    }
}

/// Um passo das cenas 33/34 (`pinta` = a 34); devolve o próximo estágio, se mudou.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    hero: &mut HeroScreen,
    pinta: bool,
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
                let (fill, stroke, extra) = estilo(i, doc);
                if let Some(p) = scene.path_mut(id) {
                    p.fill = fill;
                    p.stroke = Some(stroke);
                    p.paints = extra;
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
            Some(if pinta { LAST } else { 9 })
        }
        // O clique na amostra Fill abre o seletor nela; o toque é no vermelho do primário.
        LAST if matches!(vec.pen.selected_paths(), [_, _]) => {
            let picker = ph2d_editor_core::ids::INSP_BLENDER_PICKER;
            hero.store
                .open_picker_on(ph2d_tool_vector::ids::VECTOR_FILL_SWATCH);
            hero.store.set_blender_value(
                picker,
                ph2d_tokens::ColorValue::from_rgba8(VERMELHO.r, VERMELHO.g, VERMELHO.b, VERMELHO.a),
            );
            hero.store.note_colour_pick(picker);
            Some(9)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixture contém o fenómeno: uma cheia e uma vazia, com pilhas de ESTRUTURAS diferentes.
    #[test]
    fn as_duas_formas_contem_o_misto() {
        let doc = ph2d_editor_core::DocPx::DEFAULT;
        let (fa, _, pa) = estilo(0, doc);
        let (fb, _, pb) = estilo(1, doc);
        assert!(fa.is_some() && fb.is_none());
        assert_ne!(pa.len(), pb.len());
    }
}
