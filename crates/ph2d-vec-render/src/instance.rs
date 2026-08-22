//! **A camada de INSTÂNCIA de Motion** (ADR-0154) — módulo irmão pelo teto de LOC. O corte é por
//! assunto: aqui o desenho de UMA forma de instância e o LOTE que compartilha geometria; no
//! `lib.rs` fica o motor de path ([`crate::draw_path`]/[`crate::path_tess`]) que o `dispatch` da
//! cena usa. A metade cara (tesselar) é feita uma vez por geometria e reusada por instância —
//! o congelamento das 160k estrelas.

use ph2d_vec_scene::VecPath;
use ph2d_vector::{Affine, Brush, Color, VectorScene};

use crate::{PathTess, build_contours, draw_path_with, fill_rule, path_tess};

/// **Desenha UMA forma de instância de Motion** (ADR-0154) — a porta pública que o
/// passe vetorial de Motion usa. O `VecPath` é geometria **PURA** (a cor não mora
/// nele: instâncias iguais compartilham a MESMA geometria content-cached no
/// `VecPathStore`), então o preenchimento usa o `tint` da instância, não o `fill`
/// do path. Espelha o ramo de FILL do [`draw_path`](crate::draw_path) (a mesma `build_contours` +
/// `fill_rule`) e omite o traço — uma forma de Motion é uma silhueta preenchida.
/// `transform` já leva a geometria LOCAL à tela (o basis da instância ∘ câmera).
/// A `fill_rule` vem do path, então um anel (`EvenOdd`) desenha o furo.
pub fn draw_shape_instance(
    path: &VecPath,
    transform: Affine,
    tint: [f32; 4],
    target: &mut VectorScene,
) {
    let tess = tessellate_shape_instance(path);
    draw_shape_instance_tessellated(path, &tess, transform, tint, target);
}

/// A [`PathTess`] de uma instância de Motion — a metade CARA de [`draw_shape_instance`], separada
/// para que um lote de instâncias da MESMA geometria a construa uma vez ([`draw_shared_instances`]).
///
/// Duas espécies de vetor vivo entram por aqui. Um PRIMITIVO `source.shape` (ADR-0154) não carrega
/// tinta autorada (`fill`/`stroke` ambos `None`, ex.: `ellipse` é `..VecPath::default()`): ele SÓ
/// tem silhueta, então tessela só o preenchimento (o `tint` da instância o pinta no desenho). Um
/// vetor-DOCUMENTO `source.object` carrega o próprio fill/stroke, então tessela pela mesma
/// [`path_tess`] que o [`draw_path`](crate::draw_path) usa (o preenchimento e o traço quando difere).
pub(crate) fn tessellate_shape_instance(path: &VecPath) -> PathTess {
    if path.fill.is_some() {
        path_tess(path)
    } else {
        // Primitivo: a silhueta é do `tint` da instância, e o traço (se houver) é do próprio
        // path. (Sem `count_cook` — o caminho antigo cozia cru aqui, e os contadores do
        // `encode_cost_tests` têm de bater byte-a-byte com ele.)
        let cooked = path.cooked();
        // ⚠️ **O preenchimento leva só os contornos FECHADOS** (`Some(true)`) — é isso que
        // faz um caminho APARADO (Trim) desenhar só o traço, sem um `if` que pergunte se
        // houve trim: um trecho de contorno não tem interior, e fechá-lo implicitamente
        // desenharia a corda.
        let fill_bp = build_contours(&cooked, Some(true));
        // O traço leva TODOS os contornos. Sem contorno aberto ele é **o mesmo desenho** do
        // preenchimento ⇒ partilham uma construção só (a mesma economia do [`path_tess`]).
        let open =
            (0..cooked.contour_count()).any(|c| cooked.contour(c).is_some_and(|(_, cl)| !cl));
        let stroke_bp = (path.stroke.is_some() && open).then(|| build_contours(&cooked, None));
        PathTess {
            fill_bp: Some(fill_bp),
            stroke_bp,
            dash: crate::dash_of(&cooked, path.stroke.as_ref()),
        }
    }
}

/// Desenha UMA instância de Motion a partir da geometria JÁ TESSELADA (`tess`) — a metade barata,
/// que só emite os comandos Vello e não constrói nada. Ramifica exatamente como
/// [`draw_shape_instance`]: um vetor-documento (com fill/stroke) honra a tinta autorada dele pela
/// [`draw_path_with`]; um primitivo (sem paint) é preenchido com o `tint` da instância.
///
/// ⚠️ Um vetor-documento vivo NÃO é re-tingido a jusante (as cores são as do desenho); a tile
/// assada era tingível — a troca nomeada de virar vivo. Fiar `tint` pelo fill/stroke do
/// [`draw_path`](crate::draw_path) é o follow-up.
pub(crate) fn draw_shape_instance_tessellated(
    path: &VecPath,
    tess: &PathTess,
    transform: Affine,
    tint: [f32; 4],
    target: &mut VectorScene,
) {
    if path.fill.is_some() {
        draw_path_with(path, tess, transform, target);
    } else {
        let fill_bp = tess
            .fill_bp
            .as_ref()
            .expect("primitivo => fill_bp construido");
        let brush = Brush::Solid(Color::new(tint));
        target
            .inner_mut()
            .fill(fill_rule(path), transform, &brush, None, fill_bp);
        // ⚠️ **O traço vem DEPOIS, e não em vez do preenchimento** (medido 2026-08-21). Um
        // primitivo com `stroke_width > 0` ia pela rota do DOCUMENTO, que só preenche quando
        // `path.fill.is_some()` — e um primitivo tem `fill: None` de propósito, porque a cor
        // dele é o `tint` da instância. A forma ficava **oca** no instante em que o artista
        // mexia na largura do traço, e o `motion.tint` a jusante deixava de pintar coisa
        // nenhuma. Pela porta ÚNICA do traço ([`crate::draw_stroke_with`]), então tracejado,
        // pontas e alinhamento são os mesmos de um caminho de documento.
        crate::draw_stroke_with(path, tess, transform, target);
    }
}

/// **Desenha um LOTE de instâncias que compartilham geometria por handle, tesselando cada handle
/// DISTINTO uma ÚNICA vez** (ADR-0154 — o congelamento das 160k estrelas). `instances` produz
/// `(handle, transform, tint)` por instância; `resolve(handle)` mapeia o handle ao `VecPath`
/// armazenado. As instâncias de um mesmo handle reusam a [`PathTess`] cacheada, então N cópias da
/// mesma estrela pagam UMA tesselação em vez de N.
///
/// É a PORTA de lote do produto (o passe vetorial de Motion), e o que a torna correta é o gate:
/// desenhar por aqui produz o MESMO encode que N [`draw_shape_instance`] independentes
/// ([[feedback_two_doors_to_the_same_question_diverge]]). O cache é POR FRAME — o handle é estável
/// enquanto o conteúdo não muda, mas re-tesselar um punhado de geometrias distintas por frame é
/// grátis, e um cache por-frame não guarda um `BezPath` velho de um handle reciclado.
pub fn draw_shared_instances<'p>(
    instances: impl IntoIterator<Item = (u32, Affine, [f32; 4])>,
    resolve: impl Fn(u32) -> Option<&'p VecPath>,
    target: &mut VectorScene,
) {
    let mut cache: std::collections::BTreeMap<u32, PathTess> = std::collections::BTreeMap::new();
    for (handle, transform, tint) in instances {
        let Some(path) = resolve(handle) else {
            continue; // handle sem geometria (um cook adiantado) desenha nada
        };
        let tess = cache
            .entry(handle)
            .or_insert_with(|| tessellate_shape_instance(path));
        draw_shape_instance_tessellated(path, tess, transform, tint, target);
    }
}

#[cfg(test)]
#[path = "instance_tests.rs"]
mod tests;
