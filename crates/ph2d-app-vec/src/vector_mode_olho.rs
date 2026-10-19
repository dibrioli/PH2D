//! ⭐ **A cena 36** (`PH2D_OBJECT_MODE_SMOKE=36`, F22 — dono 19/10): O OLHO DIZ «MISTO»
//! (`docs/UI_New_and_Simple/vetor_nova_ui/33_plano_F22_o_gesto_vence_a_maquina.md` §7). Duas formas
//! vermelhas com a MESMA pilha (uma camada extra de contorno amarelo), a camada visível na da
//! esquerda e escondida na da direita, as duas selecionadas ⇒ a linha da camada aparece e o olho
//! mostra «—».

use crate::state::VecState;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_vec_scene::{Paint, PaintEntry, Rgba8, ShapeKind, StrokeSpec};

const FORMAS: [([f64; 2], f64); 2] = [([-1.2, 0.0], 0.8), ([1.2, 0.0], 0.8)];
const VERMELHO: Rgba8 = Rgba8::new(220, 40, 40, 255);
const PRETO: Rgba8 = Rgba8::new(20, 20, 20, 255);
const AMARELO: Rgba8 = Rgba8::new(250, 200, 0, 255);

/// O primeiro estágio — no buraco entre a cena 10 (`30..=32`) e a 11 (`40..`).
pub(crate) const FIRST: u8 = 33;
const ESTILO: u8 = FIRST + FORMAS.len() as u8;
pub(crate) const LAST: u8 = ESTILO + 1;
const _: () = assert!(LAST < 40);

/// A pilha da forma `i`: a mesma estrutura nas duas; o olho aceso só na primeira.
fn camadas(i: usize, doc: ph2d_editor_core::DocPx) -> Vec<PaintEntry> {
    let mut e = PaintEntry::stroke(StrokeSpec::new(AMARELO, doc.to_world(6.0)));
    e.enabled = i == 0;
    vec![e]
}

/// Um passo da cena 36; devolve o próximo estágio, se mudou.
pub fn step(
    stage: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    hero: &mut HeroScreen,
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
                if let Some(p) = scene.path_mut(id) {
                    p.fill = Some(Paint::Solid(VERMELHO));
                    p.stroke = Some(StrokeSpec::new(PRETO, doc.to_world(2.0)));
                    p.paints = camadas(i, doc);
                }
            }
            hero.apply_event(WidgetEvent::Click(
                ph2d_tool_vector::ids::VECTOR_MODE_SELECT,
            ));
            Some(LAST)
        }
        // A troca de ferramenta reduz a seleção: as duas são escolhidas DEPOIS dela.
        LAST if hero.store.tool_bars().is_some_and(|b| {
            b.column
                .iter()
                .flatten()
                .any(|x| x.on && x.id == ph2d_tool_vector::ids::VECTOR_MODE_SELECT)
        }) =>
        {
            let ids: Vec<_> = vec.entities.keys().copied().collect();
            vec.pen.select_many(&[*ids.get(1)?, *ids.first()?]);
            Some(9)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixture contém o fenómeno: a mesma estrutura, o olho diferente.
    #[test]
    fn as_duas_pilhas_diferem_so_no_olho() {
        let doc = ph2d_editor_core::DocPx::DEFAULT;
        let (a, b) = (camadas(0, doc), camadas(1, doc));
        assert_eq!(a.len(), b.len());
        assert_ne!(a[0].enabled, b[0].enabled);
        assert_eq!(a[0].swatch_color(), b[0].swatch_color());
    }
}
