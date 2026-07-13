//! ADR-0114 W6 — **o realce da seleção** (Edit Mode).
//!
//! Uma seleção que não se VÊ não existe: o usuário clica, nada muda na tela, e ele conclui
//! que a ferramenta está quebrada (é a mesma lição do "pintado ≠ populado" — só que aqui o
//! seam é o canvas, não o painel).
//!
//! **É overlay, não render de traço.** O realce é desenhado no `vector_scene` (a cena
//! Vello composta sobre o canvas neste frame, como o anel do pincel em `flip_cursor`), e
//! **não** re-rasterizado pelo passe do Flip. Duas razões:
//!
//! - o realce é **chrome**, não arte: ele não pode entrar no `pack`, não pode ir para o
//!   PNG exportado, e não pode participar do depth do desenho;
//! - a espessura dele é em **px de TELA** e constante — como a do gizmo. Se ele fosse
//!   geometria de documento, aproximar a câmera o engrossaria e ele cobriria a linha que
//!   está tentando destacar.
//!
//! ⚠️ **E é por isso que a geometria sai daqui já em px de TELA, com o `stroke` desenhando
//! sob `Affine::IDENTITY`.** No Vello o transform do `stroke` **multiplica a espessura**:
//! entregar o afim mundo→tela como transform (o 1º corte) transformou 2 px em
//! `2 × px_por_unidade_de_mundo` — centenas de pixels, e o realce virou um borrão que
//! cobria o desenho inteiro (smoke do Enio, 2026-07-13). O anel do pincel (`flip_cursor`)
//! sempre desenhou assim, em tela, exatamente por isto.
//!
//! A pose do objeto e a câmera entram numa matriz só (`world_to_screen_affine ∘
//! local→mundo`) — a MESMA que o render usa —, mas aplicada aos PONTOS.

use ph2d_flip::FlipDoc;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vec_scene::Xform;
use ph2d_vector::{BezPath, Point, VectorScene};

/// Espessura do contorno de realce, em px de tela.
const HALO_PX: f64 = 2.0; // LITERAL-PX-OK: chrome de overlay, espessura de tela

/// Espessura da caixa do marquee, em px de tela (mais fina que o realce: ela é um gesto
/// em curso, não um estado).
const MARQUEE_PX: f64 = 1.0; // LITERAL-PX-OK: chrome de overlay, espessura de tela

/// Cor do realce (âmbar do editor) — chrome, não arte.
const HALO_RGBA: [f32; 4] = [1.0, 0.72, 0.2, 0.95]; // LITERAL-COLOR-OK: overlay de selecao

/// Desenha o contorno de realce sobre cada traço selecionado do desenho VISÍVEL.
///
/// `l2w` é o afim LOCAL→mundo do objeto (a pose do gizmo). Nada é desenhado fora do modo
/// Edit: o realce é a linguagem DESSE modo, e deixá-lo aceso nos outros faria a seleção
/// parecer um estado global que o Draw/Erase respeitam — o que não é verdade (só o Sculpt
/// e o painel a consultam).
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_flip_selection(
    active: bool,
    editing: bool,
    doc: &FlipDoc,
    playhead: &ph2d_core::Playhead,
    active_layer: Option<ph2d_flip::LayerId>,
    l2w: &Xform,
    camera: &Camera2d,
    window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    if !active || !editing {
        return;
    }
    let Some((oid, _lid, did)) = crate::flip_select::visible_drawing(doc, playhead, active_layer)
    else {
        return;
    };
    let Some(drawing) = doc.object(oid).and_then(|o| o.drawing(did)) else {
        return;
    };
    if !drawing.any_selected() {
        return;
    }

    use ph2d_vector::{Affine, Brush, Color, Stroke};
    // local → mundo → tela, numa matriz só (a MESMA que o render usa).
    let [a, b, c, d, e, f] = l2w.0;
    let to_screen = camera.world_to_screen_affine(window) * Affine::new([a, b, c, d, e, f]);

    let color = Color::new(HALO_RGBA);
    for s in drawing.strokes.iter().filter(|s| s.selected) {
        let path = halo_path(s.positions(), s.closed, to_screen);
        if path.is_empty() {
            continue;
        }
        // ⚠️ **`Affine::IDENTITY`, e a geometria já em px de TELA.** No Vello a espessura
        // do traço é multiplicada pelo TRANSFORM: passar o afim mundo→tela aqui e uma
        // espessura de 2.0 dá `2 × px_por_unidade_de_mundo` — centenas de pixels no zoom
        // normal. Foi o 1º corte, e o realce virou um borrão que cobria o desenho inteiro
        // (smoke do Enio, 2026-07-13). O anel do pincel (`flip_cursor`) sempre desenhou em
        // tela, com IDENTITY, exatamente por isto.
        vector_scene.inner_mut().stroke(
            &Stroke::new(HALO_PX),
            Affine::IDENTITY,
            &Brush::Solid(color),
            None,
            &path,
        );
    }
}

/// **A caixa do marquee** (W6.1) — em px de tela, como todo o chrome deste módulo.
///
/// Um marquee que não se vê é um arrasto que não faz nada: o usuário puxa a caixa no vazio
/// e a tela fica muda até soltar. Desenhada mesmo antes do slop (a caixa de 1 px é a
/// confirmação de que o gesto começou).
pub(super) fn draw_flip_marquee(
    gesture: Option<crate::flip_edit_gesture::EditGesture>,
    vector_scene: &mut VectorScene,
) {
    use ph2d_vector::{Affine, Brush, Color, Stroke};
    let Some(crate::flip_edit_gesture::EditGesture::Marquee { start, cur, .. }) = gesture else {
        return;
    };
    let (x0, y0, x1, y1) = crate::flip_edit_gesture::marquee_rect(start, cur);
    let mut path = BezPath::new();
    path.move_to(Point::new(f64::from(x0), f64::from(y0)));
    path.line_to(Point::new(f64::from(x1), f64::from(y0)));
    path.line_to(Point::new(f64::from(x1), f64::from(y1)));
    path.line_to(Point::new(f64::from(x0), f64::from(y1)));
    path.close_path();
    vector_scene.inner_mut().stroke(
        &Stroke::new(MARQUEE_PX),
        Affine::IDENTITY, // px de TELA (o transform multiplicaria a espessura — ver acima)
        &Brush::Solid(Color::new(HALO_RGBA)),
        None,
        &path,
    );
}

/// A polilinha do realce, **já em px de TELA** (o `to_screen` é aplicado aos PONTOS, não
/// entregue ao Vello como transform — ver o `stroke` acima).
///
/// Pura, e separada por isso: a invariante que ela carrega ("a espessura do realce não
/// muda com o zoom") é a que quebrou no produto, e uma função pura é o que um gate
/// consegue interrogar.
fn halo_path(pos: &[ph2d_core::Vec2], closed: bool, to_screen: ph2d_vector::Affine) -> BezPath {
    let mut path = BezPath::new();
    for (i, p) in pos.iter().enumerate() {
        let pt = to_screen * Point::new(f64::from(p.x), f64::from(p.y));
        if i == 0 {
            path.move_to(pt);
        } else {
            path.line_to(pt);
        }
    }
    // O traço FECHADO (e o contorno de uma região) fecha o realce; um traço aberto NÃO —
    // desenhar o segmento que liga as pontas mostraria uma linha que o usuário não fez, e
    // depois do BUGS #17 sabemos que "fechado" não é o caso comum.
    if closed && pos.len() >= 3 {
        path.close_path();
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_core::Vec2;
    use ph2d_vector::{Affine, Shape};

    /// 🔴 **O realce tem espessura de TELA — e a geometria dele é que carrega o zoom.**
    ///
    /// O 1º corte entregou o afim mundo→tela ao Vello como *transform* do `stroke`. No
    /// Vello o transform **multiplica a espessura**: 2 px viraram `2 × px_por_unidade` =
    /// centenas de pixels, e o realce virou um borrão cobrindo o desenho (smoke do Enio).
    ///
    /// A cura é a geometria sair daqui **já em px de tela** (e o `stroke` usar
    /// `IDENTITY`). Este gate afirma isso pelo único jeito que não mente: os pontos que
    /// saem têm de estar onde a câmera os põe NA TELA.
    ///
    /// Mutação que sangra: devolva os pontos em espaço local (tire o `to_screen *`) e a
    /// bbox deixa de acompanhar o zoom.
    #[test]
    fn the_halo_geometry_comes_out_in_screen_pixels() {
        let pts = [Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)];
        // Uma câmera qualquer: 20 px de tela por unidade de mundo, origem em (100, 50).
        let to_screen = Affine::new([20.0, 0.0, 0.0, 20.0, 100.0, 50.0]);

        let bbox = halo_path(&pts, false, to_screen).bounding_box();
        assert!(
            (bbox.x0 - 100.0).abs() < 1e-6 && (bbox.x1 - 300.0).abs() < 1e-6,
            "a geometria do realce NAO saiu em px de tela: {bbox:?} — se ela sair em \
             espaco local, o Vello multiplica a espessura pelo zoom e o realce vira um \
             borrao"
        );
    }

    /// **Um traço ABERTO não ganha o segmento de fechamento** — desenhá-lo mostraria uma
    /// linha que o usuário não fez (e, depois do BUGS #17, sabemos que o traço da mão é
    /// aberto: seria o caso COMUM).
    #[test]
    fn an_open_stroke_is_not_closed_by_the_halo() {
        let pts = [
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
        ];
        let open = halo_path(&pts, false, Affine::IDENTITY);
        let closed = halo_path(&pts, true, Affine::IDENTITY);
        assert!(
            open.elements().len() < closed.elements().len(),
            "o realce fechou um traco ABERTO — desenharia um segmento que nao existe"
        );
    }
}
