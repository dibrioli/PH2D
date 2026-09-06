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
    dilated: Option<&crate::DilatedPaints>,
) {
    if path.paints.is_empty() {
        return;
    }
    for (i, e) in path
        .paints
        .iter()
        .enumerate()
        .filter(|(_, e)| e.is_active())
    {
        let aberta = open_layer(target, path, transform, e.opacity.get(), e.blend);
        // ⭐⭐⭐ **ONDE esta camada desenha** (v21). ⚠️ A translação vem **por fora** (`transform *
        // translate`, e não `translate * transform`): o deslocamento é LOCAL, logo tem de sofrer a
        // pose da forma — rodar a forma 90° tem de rodar a sombra com ela, senão a sombra descola.
        //
        // ⭐ E é aqui que se vê por que ele é GRÁTIS: a tesselação (`tess`) é a MESMA, e o Vello já
        // recebia um transform em toda camada. O custo somado é uma multiplicação de afins.
        let onde = camada_xf(transform, e.offset);
        // ⭐⭐⭐ **A GEOMETRIA DESTA CAMADA** (v22): a dilatada, se a shell a cozeu; senão a da
        // forma. ⚠️ O índice é o do DOCUMENTO (`enumerate` ANTES do filtro) — desarmar a camada `1`
        // não pode fazer a `3` desenhar a silhueta da `2`.
        //
        // ⛔ **Uma camada dilatada cuja geometria NÃO chegou desenha a silhueta da forma**, e não
        // nada: a shell só falha em cozer quando a booleana panica, e *voltar à forma lê-se como
        // «o offset não pegou»; desaparecer lê-se como «apaguei a camada»*.
        //
        // ⭐ **Tesselar o dilatado corre AQUI** (`~0,13 µs`, como qualquer forma); o que é caro — o
        // offset — já veio memoizado da shell. É essa divisão que deixa o traço dilatado passar
        // pela MESMA porta do de base, com o tracejado AJUSTADO ao comprimento novo de graça.
        let proprio = dilated
            .and_then(|d| d.get(&(path.id, i)))
            .map(|p| (p, crate::path_tess(p)));
        let (geo, gtess) = proprio.as_ref().map_or((path, tess), |(p, t)| (*p, t));
        match &e.kind {
            PaintKind::Fill(paint) => {
                if let Some(fp) = gtess.fill_bp.as_ref() {
                    target.inner_mut().fill(
                        fill_rule(geo),
                        onde,
                        &fill_brush(paint, geo),
                        None,
                        fp,
                    );
                }
            }
            // ⛔ Os dois `None` (ladrilho, arte de pincel) são a fronteira declarada no cabeçalho
            // deste módulo, e o censo `the_artless_draw_routes_are_declared` conta-os.
            // ⭐ Um CONTORNO dilatado percorre a silhueta crescida/encolhida — o anel de CAD —, e
            // passa por AQUI: o que muda é a geometria, não a porta.
            PaintKind::Stroke(s) => {
                stroke_draw::draw_one_stroke(geo, s, gtess, onde, target, None, None);
            }
        }
        if aberta {
            target.pop_layer();
        }
    }
}

/// **O transform de UMA camada** — a pose da forma, e depois o deslocamento dela.
///
/// ⚠️ **A ordem é load-bearing e o caso de omissão não a testa:** com `offset = [0, 0]` as duas
/// ordens dão o mesmo afim, então uma fixtura sem deslocamento fica verde sobre a errada. O que a
/// separa é uma forma **rodada ou escalada** — ali, `translate ∘ transform` deixa a sombra a andar
/// no eixo do ECRÃ enquanto a forma roda por baixo dela.
///
/// ⭐ O neutro é o afim de sempre, **ao bit** — é o `if` que mantém byte-idêntico o desenho de toda
/// pilha que não desloca nada.
fn camada_xf(transform: Affine, offset: [f64; 2]) -> Affine {
    if offset[0] == 0.0 && offset[1] == 0.0 {
        return transform;
    }
    transform * Affine::translate((offset[0], offset[1]))
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
