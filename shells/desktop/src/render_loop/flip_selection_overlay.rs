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
//! A pose do objeto (o gizmo), **a pose da CHAVE** (`FlipFrame::offset`, W7.2) e a câmera
//! entram numa matriz só (`world_to_screen ∘ art_to_world`) — a MESMA cadeia que o render
//! usa —, mas aplicada aos PONTOS. Sem a pose da chave, o realce fica deslocado da linha
//! por todo o offset da chave ("o traço afastado do seu mesh", smoke do Enio 2026-07-14).

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
    let Some((oid, lid, did)) = crate::flip_select::visible_drawing(doc, playhead, active_layer)
    else {
        return;
    };
    let Some(obj) = doc.object(oid) else {
        return;
    };
    let Some(drawing) = obj.drawing(did) else {
        return;
    };
    if !drawing.any_selected() {
        return;
    }

    use ph2d_vector::{Affine, Brush, Color, Stroke};
    // **A POSE DA CHAVE entra na matriz** (W7.2 fix): o render desenha o traço em
    // `art_to_world(objeto, pose)`; o realce tem de usar EXATAMENTE a mesma cadeia, senão
    // fica deslocado da linha pela pose — "o traço afastado do seu mesh" (smoke do Enio,
    // 2026-07-14). A pose sai do MESMO amostrador que o render (`offset_at_cycled` no
    // quadro atual), não de `frame_offset` — sob um ciclo os dois diferem.
    let pose = obj.layer(lid).map_or(ph2d_core::Vec2::ZERO, |l| {
        l.offset_at_cycled(obj.frame_at(playhead))
    });
    let to_screen = art_screen_affine(l2w, pose, camera.world_to_screen_affine(window));

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

/// **ARTE → TELA**, a cadeia inteira numa matriz: `câmera ∘ objeto ∘ pose_da_chave`.
///
/// É a MESMA cadeia que o render dobra (`flip_transform::art_to_world`), fechada com a
/// câmera. Pura, e separada por isso: a invariante que ela carrega — *o realce pousa
/// EXATAMENTE sobre o traço renderizado, pose inclusa* — é a que quebrou no smoke do W7.2,
/// e uma função pura é o que um gate consegue interrogar.
fn art_screen_affine(
    l2w: &Xform,
    pose: ph2d_core::Vec2,
    cam: ph2d_vector::Affine,
) -> ph2d_vector::Affine {
    let [a, b, c, d, e, f] = crate::flip_transform::art_to_world(l2w, pose).0;
    cam * ph2d_vector::Affine::new([a, b, c, d, e, f])
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

    /// 🔴 **O realce inclui a POSE DA CHAVE** (W7.2 fix) — pousa sobre o traço no lugar
    /// em que o render o desenha, não na geometria crua.
    ///
    /// O render dobra `art_to_world(objeto, pose)`; o overlay tem de dobrar o MESMO. Sem
    /// isso, o realce âmbar fica parado enquanto a arte se move com a pose — "o traço
    /// afastado do seu mesh" (smoke do Enio, 2026-07-14).
    ///
    /// Mutação que sangra: tirar a `pose` da `art_screen_affine` (voltar a `cam * l2w`).
    #[test]
    fn the_halo_carries_the_key_pose() {
        let cam = Affine::new([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]); // 2 px/unidade
        let l2w = Xform::IDENTITY; // objeto na identidade: isola a pose
        let pose = ph2d_core::Vec2::new(100.0, -40.0);

        // A âncora da arte (o ponto (0,0) da geometria) sob pose = deslocada pela pose,
        // depois pela câmera.
        let origin = art_screen_affine(&l2w, pose, cam) * Point::new(0.0, 0.0);
        assert!(
            (origin.x - 200.0).abs() < 1e-6 && (origin.y - (-80.0)).abs() < 1e-6,
            "o realce ignorou a pose da chave: {origin:?} (esperado a arte deslocada de 100,-40 \
             e escalada por 2) — o halo ficaria parado enquanto o traco anda"
        );

        // E pose neutra reduz EXATAMENTE ao caminho antigo (`cam * l2w`) — o caso comum
        // (nenhuma instancia movida) nao paga nada.
        let neutral = art_screen_affine(&l2w, ph2d_core::Vec2::ZERO, cam) * Point::new(7.0, 3.0);
        let plain = cam * Point::new(7.0, 3.0);
        assert!((neutral.x - plain.x).abs() < 1e-9 && (neutral.y - plain.y).abs() < 1e-9);
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
