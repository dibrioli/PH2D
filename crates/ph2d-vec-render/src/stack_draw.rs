//! ⭐⭐⭐ **DESENHAR A PILHA DE APARÊNCIA** — o preenchimento de base e cada camada (v20 do schema,
//! estudo 42 item 4). Módulo irmão de [`super`] pelo tecto de LOC; quem decide a ORDEM é o
//! `draw_path_with` de lá, pela [`VecPath::draw_order`] (F45: a base tem posição na pilha).
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

use ph2d_vec_scene::{Paint, PaintKind, StackItem, VecPath};
use ph2d_vector::{Affine, Shape, VectorScene};

use crate::{PathTess, blend, fill_brush, fill_rule, path_bounds_under, stroke_draw};

/// **Desenha o preenchimento de BASE** — o padrão (com o ladrilho deste quadro), o multi-ponto ou a
/// tinta lisa/degradê. No-op sem preenchimento.
pub(crate) fn draw_base_fill(
    path: &VecPath,
    tess: &PathTess,
    transform: Affine,
    target: &mut VectorScene,
    tile: Option<&crate::PatternTile>,
) {
    if let Some(fill) = &path.fill {
        let fp = tess.fill_bp.as_ref().expect("fill => fill_bp construido");
        // ⭐ **O padrão desenha a IMAGEM quando o ladrilho existe, e a `fallback` quando não.**
        // As duas metades são desenho certo: a segunda é o que o artista vê enquanto a arte carrega,
        // ou quando a forma-fonte desapareceu — e ⛔ desenhar NADA seria pior (uma forma invisível
        // não se distingue de um preenchimento vazio).
        if let (Paint::Pattern(pat), Some(t)) = (fill, tile) {
            crate::pattern::fill_pattern(
                target,
                fp,
                fill_rule(path),
                transform,
                &t.image,
                // ⚠️ A caixa vem do desenho de PREENCHIMENTO (espaço das âncoras, como a
                // colocação): é ela que o `Clamp` enquadra. `bounding_box` de um `BezPath` vazio
                // devolve uma caixa degenerada, e o `placement_in` recusa-a e cai na autorada.
                pat.placement_in(t.cells, t.tile_px, {
                    let b = fp.bounding_box();
                    ([b.x0, b.y0], [b.x1, b.y1])
                }),
                pat.mode,
                t.quality,
                pat.alpha,
            );
        } else if let Paint::MultiPoint { points } = fill {
            // ⚠️ `path`, não `cooked`: o `fill_multipoint` mede a caixa dos pontos de controle da
            // forma AUTORADA (`control_point_bounds`). Passar o cozido moveria o gradiente de toda
            // forma com quina viva ou efeito — mudança de aparência dentro de um fix de custo.
            crate::fill_multipoint(target, fp, path, points, transform);
        } else {
            // `VectorScene::fill_path` assume NonZero; um compound precisa da
            // regra do path (EvenOdd vaza o contorno de dentro).
            target.inner_mut().fill(
                fill_rule(path),
                transform,
                &fill_brush(fill, path),
                None,
                fp,
            );
        }
    }
}

/// ⭐ **Os traços POR CIMA dos Filtros** (F47, dono 02/11) — redesenha, vectoriais e na ordem da pilha,
/// só os traços com [`ph2d_vec_scene::StrokeSpec::above_effects`]; o `dispatch` chama-o logo depois de
/// pôr a imagem de FX da forma. A imagem continua a levá-los (a sombra nasce da forma inteira), então o
/// traço nítido cobre exactamente o filtrado. No-op (sem tesselar) se nenhum traço o pede.
pub(crate) fn draw_above_effects(
    path: &VecPath,
    transform: Affine,
    target: &mut VectorScene,
    derived: crate::Derived<'_>,
) {
    if !path.has_stroke_above_effects() {
        return;
    }
    let tess = crate::path_tess(path);
    for it in path.draw_order().filter(|it| path.is_above_effects(*it)) {
        match it {
            StackItem::BaseStroke => stroke_draw::draw_stroke_with(
                path,
                &tess,
                transform,
                target,
                derived.stroke_tile,
                derived.brush_art,
            ),
            StackItem::Entry(i) => draw_entry(path, i, &tess, transform, target, derived.dilated),
            StackItem::BaseFill => {}
        }
    }
}

/// **Desenha a camada `i`** da pilha, se ela está ligada. ⚠️ `i` é o índice do DOCUMENTO — desarmar a
/// camada `1` não pode fazer a `3` desenhar a silhueta da `2`.
pub(crate) fn draw_entry(
    path: &VecPath,
    i: usize,
    tess: &PathTess,
    transform: Affine,
    target: &mut VectorScene,
    dilated: Option<&crate::DilatedPaints>,
) {
    let Some(e) = path.paints.get(i).filter(|e| e.is_active()) else {
        return;
    };
    let aberta = open_layer(target, path, transform, e.opacity.get(), e.blend);
    // ⭐⭐⭐ **ONDE esta camada desenha** (v21). ⚠️ A translação vem **por fora** (`transform *
    // translate`, e não `translate * transform`): o deslocamento é LOCAL, logo tem de sofrer a
    // pose da forma — rodar a forma 90° tem de rodar a sombra com ela, senão a sombra descola.
    //
    // ⭐ E é aqui que se vê por que ele é GRÁTIS: a tesselação (`tess`) é a MESMA, e o Vello já
    // recebia um transform em toda camada. O custo somado é uma multiplicação de afins.
    let onde = camada_xf(transform, e.offset);
    // ⭐⭐⭐ **A GEOMETRIA DESTA CAMADA** (v22): a dilatada, se a shell a cozeu; senão a da
    // forma.
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
                target
                    .inner_mut()
                    .fill(fill_rule(geo), onde, &fill_brush(paint, geo), None, fp);
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

/// **O transform de UMA camada** — a pose da forma, e depois o deslocamento dela.
///
/// ⚠️ **A ordem é load-bearing e o caso de omissão não a testa:** com `offset = [0, 0]` as duas
/// ordens dão o mesmo afim, então uma fixtura sem deslocamento fica verde sobre a errada. O que a
/// separa é uma forma **rodada ou escalada** — ali, `translate ∘ transform` deixa a sombra a andar
/// no eixo do ECRÃ enquanto a forma roda por baixo dela.
///
/// ⭐ O neutro é o afim de sempre, **ao bit** — é o `if` que mantém byte-idêntico o desenho de toda
/// pilha que não desloca nada.
pub(crate) fn camada_xf(transform: Affine, offset: [f64; 2]) -> Affine {
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
