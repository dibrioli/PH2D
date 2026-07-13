//! O realce das faces do **Shape Builder** — módulo irmão (LOC cap).
//!
//! Sem isto o Shape Builder é adivinhação: o artista arrasta o cursor sobre um emaranhado
//! de formas sobrepostas e só descobre o que pegou depois de soltar. **O realce é a
//! feature** — o resto é a booleana que já existia.
//!
//! ## As três camadas, e a de cima é a que faltava
//!
//! - **Véu da face sob o cursor** (o hover): fino. Diz "é ESTA que você vai pegar".
//! - **Véu das faces já pintadas**: mais forte, com contorno. Dizem "estas já são suas" — e
//!   o contorno é o que separa duas faces adjacentes (dois véus da mesma cor encostados
//!   viram um borrão só).
//! - **As SILHUETAS das formas de origem, por CIMA de tudo.** Esta é a que faltava, e sem
//!   ela a ferramenta é inutilizável: as faces seguem as bordas das formas sobrepostas, e
//!   uma forma coberta por outra **não aparece na tela** — o artista vê um véu recortado por
//!   contornos que não correspondem a nada que ele enxergue. Foi exatamente o que o Enio
//!   fotografou. É também o que o Illustrator faz (a arte selecionada mostra o traço de
//!   seleção dela mesmo sob o realce).
//!
//! ## Tudo que tem ESPESSURA é desenhado em espaço de TELA
//!
//! A convenção da crate (ver [`crate::draw_corner_handles`]): o ponto sobe pelo afim, a
//! espessura NÃO. Um `Stroke::new(1.5)` sob o afim mundo→tela não é uma borda de 1,5 px —
//! é uma borda de **1,5 unidades de MUNDO**, que no zoom default são ~150 px de tinta opaca
//! cobrindo a arte. Era esse o "véu estranho e grande".
//!
//! **Subtrair pinta com outra cor.** Um gesto que APAGA e um que UNE não podem parecer a
//! mesma coisa: o artista solta o botão e descobre o que fez. A cor é a única coisa que ele
//! olha durante o arrasto.

use ph2d_tokens::{ColorToken, Theme};
use ph2d_vec_scene::VecPath;
use ph2d_vector::{Affine, Brush, Color as VelloColor, Stroke, VectorScene};

/// Opacidade do véu da face sob o cursor (ela ainda não é sua).
const HOVER_ALPHA: f32 = 0.18;
/// Opacidade do véu de uma face já pintada. Baixa **de propósito**: um véu opaco o bastante
/// para ser visto é opaco o bastante para esconder a arte — quem diz "esta é sua" é o
/// contorno, não a tinta.
const MARKED_ALPHA: f32 = 0.30;
/// Espessura da borda de uma face pintada, em **pixels de tela**.
const EDGE_PX: f64 = 1.5;
/// Espessura da silhueta de uma forma de origem, em **pixels de tela**.
const SILHOUETTE_PX: f64 = 1.0;

/// Desenha o realce. `sources` = as formas de origem (as silhuetas); `hover` = a face sob o
/// cursor; `marked` = as já pintadas. Tudo em MUNDO (a shell já assou); `transform` leva
/// mundo→tela.
///
/// `subtract` troca a cor: o gesto que apaga não pode parecer o que une.
pub fn draw_build_faces(
    sources: &[VecPath],
    hover: Option<&VecPath>,
    marked: &[VecPath],
    subtract: bool,
    transform: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    if sources.is_empty() {
        return;
    }
    // UNIR = o acento (é o que se ganha). SUBTRAIR = o token de perigo (é o que se perde).
    // Sem essa distinção o artista arrasta às cegas e só descobre no release.
    let base = if subtract {
        ColorToken::Danger
    } else {
        ColorToken::Accent
    };
    let c = base.resolve(theme);
    let tint = |a: f32| VelloColor::from_rgba8(c.r, c.g, c.b, (a * 255.0) as u8);
    let edge = ColorToken::BorderEmph.resolve(theme);
    let silhouette = VelloColor::from_rgba8(edge.r, edge.g, edge.b, edge.a);

    for m in marked {
        target.inner_mut().fill(
            crate::fill_rule(m),
            transform,
            &Brush::Solid(tint(MARKED_ALPHA)),
            None,
            &crate::build_bezpath(m),
        );
    }
    if let Some(h) = hover {
        target.inner_mut().fill(
            crate::fill_rule(h),
            transform,
            &Brush::Solid(tint(HOVER_ALPHA)),
            None,
            &crate::build_bezpath(h),
        );
    }
    // As bordas das pintadas e, **por último, as silhuetas** — todas em TELA. O véu é uma
    // camada de cima; sem redesenhar as formas que definem as regiões, a arte coberta some e
    // o realce fica pairando sobre nada.
    for (bp, w, edge) in edge_strokes(sources, marked, transform) {
        let color = if edge { silhouette } else { tint(1.0) };
        target.inner_mut().stroke(
            &Stroke::new(w),
            Affine::IDENTITY,
            &Brush::Solid(color),
            None,
            &bp,
        );
    }
}

/// Os traços do realce: **a geometria já em coordenadas de TELA, a largura já em PIXELS**.
/// `true` = é uma silhueta de forma de origem; `false` = a borda de uma face pintada.
///
/// A regra desta crate mora aqui, e é a que o gate mede: **o ponto sobe pelo afim, a
/// espessura NÃO.** Um `Stroke::new(1.5)` emitido sob o afim mundo→tela não é uma borda de
/// 1,5 px — é uma borda de 1,5 unidades de MUNDO, que no zoom default são ~150 px de tinta
/// opaca por cima da arte. Era esse o "véu estranho e grande" do smoke reprovado.
pub(crate) fn edge_strokes(
    sources: &[VecPath],
    marked: &[VecPath],
    transform: Affine,
) -> Vec<(ph2d_vector::BezPath, f64, bool)> {
    let mut out = Vec::with_capacity(marked.len() + sources.len());
    for m in marked {
        out.push((transform * crate::build_bezpath(m), EDGE_PX, false));
    }
    for s in sources {
        out.push((transform * crate::build_bezpath(s), SILHOUETTE_PX, true));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector::Shape;

    fn tri() -> VecPath {
        use ph2d_vec_scene::VecVertex;
        VecPath {
            verts: [[0.0, 0.0], [2.0, 0.0], [1.0, 2.0]]
                .map(VecVertex::corner)
                .to_vec(),
            closed: true,
            ..VecPath::default()
        }
    }

    /// **O gate do véu.** Dar zoom não pode engrossar a borda do realce: a espessura é em
    /// PIXELS e a geometria é que sobe pelo afim. A 1ª versão passava o afim mundo→tela ao
    /// `Stroke`, e a borda de "1,5" virava 1,5 unidades de mundo — ~150 px de tinta opaca
    /// cobrindo exatamente a arte que o artista precisa ver.
    #[test]
    fn the_highlight_edges_are_pixels_and_the_geometry_is_world() {
        let (src, mk) = (vec![tri()], vec![tri()]);
        let at = |zoom: f64| edge_strokes(&src, &mk, Affine::scale(zoom));

        let (a, b) = (at(1.0), at(8.0));
        assert_eq!(a.len(), 2, "a borda da face + a silhueta da fonte");
        // A espessura é a MESMA nos dois zooms…
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.1, y.1, "a largura não muda com o zoom (é px)");
            assert_eq!(x.2, y.2);
        }
        // …e a geometria é que cresce (senão o realce não seguiria a forma).
        let w = |v: &Vec<(ph2d_vector::BezPath, f64, bool)>| v[0].0.bounding_box().width();
        assert!(
            (w(&b) - 8.0 * w(&a)).abs() < 1e-9,
            "a forma sobe pelo afim: {} vs {}",
            w(&b),
            w(&a)
        );
    }

    /// Sem formas de origem não há o que realçar — e o realce não pode desenhar no vazio (o
    /// modo Build sem arranjo é o clique de SELEÇÃO, não um véu órfão).
    #[test]
    fn nothing_is_drawn_without_sources() {
        assert!(edge_strokes(&[], &[tri()], Affine::IDENTITY).len() == 1);
        let mut scene = VectorScene::new();
        draw_build_faces(
            &[],
            Some(&tri()),
            &[tri()],
            false,
            Affine::IDENTITY,
            Theme::Forge,
            &mut scene,
        );
        assert_eq!(scene.inner().encoding().n_paths, 0, "nada foi emitido");
    }
}
