//! ⭐⭐⭐ **DESENHAR A PILHA DE APARÊNCIA** — as camadas de tinta por cima do `fill`+`stroke` de
//! base (v20 do schema, estudo 42 item 4). Módulo irmão de [`super`] pelo tecto de LOC, e o corte é
//! por RESPONSABILIDADE: ali desenha-se **o chão** de uma forma; aqui, o que está **por cima dele**.
//!
//! # ⭐ A geometria é a MESMA, e é isso que torna a pilha barata
//!
//! Uma camada extra não re-tessela nada: ela reusa o [`PathTess`] que o chão já construiu e só
//! troca a TINTA (e, num contorno, a largura — que o Vello aplica no `stroke`, não na geometria).
//! ⇒ N camadas custam N emissões, e **zero** construções de caminho.
//!
//! # A composição de CADA camada
//!
//! Cada entrada tem opacidade e mistura próprias, e elas compõem-se com o que já está pintado
//! **dentro** da forma. ⚠️ Isso é diferente da opacidade e da mistura do OBJECTO (v19), que compõem
//! a forma inteira com o que está **atrás** dela — e é por isso que a base não precisa das suas: a
//! camada de baixo já é composta pela do objecto.
//!
//! ⛔ **Uma camada neutra (opaca e `Normal`) NÃO abre camada de composição**, e é isso que mantém
//! byte-idêntico o desenho de toda forma cuja pilha só acrescenta tintas simples.
//!
//! # ⛔ O que uma camada extra NÃO carrega, e é declarado
//!
//! **Padrão e pincel.** A arte de um ladrilho e a de um pincel são memoizadas pela forma ANFITRIÃ
//! (`VecPathId -> …`), e uma camada não é uma forma — dar-lhe uma chave própria é a wave que o
//! censo `the_artless_draw_routes_are_declared` já nomeia para as outras três rotas. Uma camada com
//! padrão ou pincel desenha a **cor de recurso** dele, que é a mesma resposta honesta que o chão dá
//! enquanto o ladrilho não resolve.

use ph2d_vec_scene::{PaintKind, VecPath};
use ph2d_vector::{Affine, VectorScene};

use crate::{PathTess, blend, fill_brush, fill_rule, path_bounds_under, stroke_draw};

/// **Desenha as camadas ligadas**, de baixo para cima. No-op sem pilha — o caminho comum.
pub(crate) fn draw_extra_paints(
    path: &VecPath,
    tess: &PathTess,
    transform: Affine,
    target: &mut VectorScene,
) {
    if path.paints.is_empty() {
        return;
    }
    for e in path.paints.iter().filter(|e| e.is_active()) {
        let aberta = open_layer(target, path, transform, e.opacity.get(), e.blend);
        match &e.kind {
            PaintKind::Fill(paint) => {
                if let Some(fp) = tess.fill_bp.as_ref() {
                    target.inner_mut().fill(
                        fill_rule(path),
                        transform,
                        &fill_brush(paint, path),
                        None,
                        fp,
                    );
                }
            }
            // ⛔ Os dois `None` (ladrilho, arte de pincel) são a fronteira declarada no cabeçalho
            // deste módulo, e o censo `the_artless_draw_routes_are_declared` conta-os.
            PaintKind::Stroke(s) => {
                stroke_draw::draw_one_stroke(path, s, tess, transform, target, None, None);
            }
        }
        if aberta {
            target.pop_layer();
        }
    }
}

/// Abre a camada de composição desta entrada, se ela precisar de uma. Devolve se abriu.
///
/// ⚠️ **O rectângulo sai da MESMA porta que dimensiona o scratch do FX** ([`path_bounds_under`]),
/// que desde a v20 inclui o transbordo do contorno **mais gordo da pilha** — uma camada com um
/// traço de `20` sobre um de `2` seria recortada por uma caixa medida na base.
fn open_layer(
    target: &mut VectorScene,
    path: &VecPath,
    transform: Affine,
    alpha: f32,
    mode: ph2d_vec_scene::BlendMode,
) -> bool {
    if alpha >= 1.0 && blend::is_neutral(mode) {
        return false;
    }
    let Some(rect) = path_bounds_under(path, transform) else {
        return false; // sem caixa não há o que compor
    };
    // Um modo que a placa não exprime não se desenha, mas a OPACIDADE dele desenha-se — a mesma
    // resposta que a camada do OBJECTO dá, e o `is_neutral` já a declara.
    let vb = blend::vello_blend(mode).unwrap_or_default();
    target.push_object_layer(&rect, vb, alpha);
    true
}

#[cfg(test)]
#[path = "stack_draw_tests.rs"]
mod tests;
