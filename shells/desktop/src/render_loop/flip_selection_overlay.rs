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

/// Raio do PONTO no domínio Point (W8), em px de tela — a âncora que se clica.
const POINT_DOT_PX: f64 = 3.0; // LITERAL-PX-OK: chrome de overlay, raio de tela
/// Raio do ponto NÃO-selecionado (menor: presença, não destaque).
const POINT_DIM_PX: f64 = 2.0; // LITERAL-PX-OK: chrome de overlay, raio de tela

/// O ponto não-selecionado sobre linha CLARA: quase-preto (chrome, não arte).
const POINT_DIM_DARK: [f32; 4] = [0.06, 0.07, 0.10, 0.9]; // LITERAL-COLOR-OK: overlay de selecao
/// O ponto não-selecionado sobre linha ESCURA: quase-branco.
const POINT_DIM_LIGHT: [f32; 4] = [0.92, 0.93, 0.97, 0.9]; // LITERAL-COLOR-OK: overlay de selecao

/// O limiar de luminância em que o contraste WCAG contra o branco e contra o preto
/// EMPATA (`(1.05)/(Y+0.05) == (Y+0.05)/0.05` ⇒ `Y ≈ 0.179`). Acima dele, o escuro
/// contrasta mais; abaixo, o claro. É a fronteira canônica do "texto preto ou branco?",
/// e o mesmo raciocínio vale para um ponto sobre a linha.
const CONTRAST_FLIP_Y: f32 = 0.179; // LITERAL-COLOR-OK: limiar WCAG, nao cor de design

/// Luminância relativa (Rec.709/WCAG) de uma cor LINEAR — que é como o Flip guarda as
/// cores de ponto (`srgb8_to_linear` na autoria). Polinomial (HR-5).
fn relative_luminance(c: ph2d_flip::Rgba) -> f32 {
    0.2126 * c.0[0] + 0.7152 * c.0[1] + 0.0722 * c.0[2]
}

/// **A cor do ponto NÃO-selecionado, CONTRASTANDO com a linha** (smoke do Enio,
/// 2026-07-15: *"em linhas muito claras não são visíveis"*): escuro sobre linha clara,
/// claro sobre linha escura. Recomputada por frame a partir da cor VIVA do ponto —
/// recolorir o traço (painel, per-point) muda o ponto junto, por construção.
fn dim_dot_rgba(line: ph2d_flip::Rgba) -> [f32; 4] {
    if relative_luminance(line) > CONTRAST_FLIP_Y {
        POINT_DIM_DARK
    } else {
        POINT_DIM_LIGHT
    }
}

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
    point_domain: bool,
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
    // No domínio POINT as âncoras aparecem SEMPRE (dim; selecionadas em acento) — é a
    // linguagem do modo (GP): sem os pontos na tela não há o que mirar. No Stroke, sem
    // seleção não há nada a realçar.
    if !point_domain && !drawing.any_selected() {
        return;
    }

    use ph2d_vector::{Affine, Brush, Color, Stroke};
    // **A POSE DA CHAVE entra na matriz** (W7.2 fix): o render desenha o traço em
    // `art_to_world(objeto, pose)`; o realce tem de usar EXATAMENTE a mesma cadeia, senão
    // fica deslocado da linha pela pose — "o traço afastado do seu mesh" (smoke do Enio,
    // 2026-07-14). A pose sai do MESMO amostrador que o render (`offset_at_cycled` no
    // quadro atual), não de `frame_offset` — sob um ciclo os dois diferem.
    let pose = obj.layer(lid).map_or(ph2d_flip::Pose::IDENTITY, |l| {
        l.pose_at_cycled(obj.frame_at(playhead))
    });
    let to_screen = art_screen_affine(l2w, pose, camera.world_to_screen_affine(window));

    // ── Domínio POINT (W8): âncoras como PONTOS — dim nas não-selecionadas, acento nas
    // selecionadas. Sem halo de traço: com um ponto aceso o `any()` acenderia o traço
    // inteiro e o realce mentiria sobre O QUE está selecionado.
    if point_domain {
        let hot = Color::new(HALO_RGBA);
        for s in &drawing.strokes {
            let colors = s.colors();
            for (i, p) in s.positions().iter().enumerate() {
                let c = to_screen * Point::new(f64::from(p.x), f64::from(p.y));
                let selected = s.point_selected(i);
                // O ponto dim contrasta com a COR DO PONTO da linha (por ponto — o
                // per-point color do Edit pode variar dentro do mesmo traço).
                let (r, col) = if selected {
                    (POINT_DOT_PX, hot)
                } else {
                    let line = colors.get(i).copied().unwrap_or(ph2d_flip::Rgba::WHITE);
                    (POINT_DIM_PX, Color::new(dim_dot_rgba(line)))
                };
                vector_scene.inner_mut().fill(
                    ph2d_vector::Fill::NonZero,
                    Affine::IDENTITY, // px de TELA (como todo o chrome deste módulo)
                    &Brush::Solid(col),
                    None,
                    &ph2d_vector::Circle::new(c, r),
                );
            }
        }
        return;
    }

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
    pose: ph2d_flip::Pose,
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
        let pose = ph2d_flip::Pose::from_translation(ph2d_core::Vec2::new(100.0, -40.0));

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
        let neutral =
            art_screen_affine(&l2w, ph2d_flip::Pose::IDENTITY, cam) * Point::new(7.0, 3.0);
        let plain = cam * Point::new(7.0, 3.0);
        assert!((neutral.x - plain.x).abs() < 1e-9 && (neutral.y - plain.y).abs() < 1e-9);
    }

    /// 🔴 **O ponto não-selecionado CONTRASTA com a cor da linha** (smoke do Enio,
    /// 2026-07-15: dots brancos somem em linha clara): linha clara ⇒ ponto escuro;
    /// linha escura ⇒ ponto claro. O limiar é a fronteira WCAG onde o contraste contra
    /// branco e contra preto empata (Y ≈ 0.179) — e as BORDAS dele são gateadas dos
    /// dois lados (`feedback_gate_the_edges_of_the_domain`).
    ///
    /// Mutação que sangra: devolver sempre o claro (o bug original), ou inverter o
    /// limiar.
    #[test]
    fn the_dim_dot_contrasts_with_the_line_colour() {
        use ph2d_flip::Rgba;
        // Linha branca (o caso do smoke): o ponto tem de ser ESCURO.
        assert_eq!(dim_dot_rgba(Rgba::new(1.0, 1.0, 1.0, 1.0)), POINT_DIM_DARK);
        // Linha preta: claro.
        assert_eq!(dim_dot_rgba(Rgba::new(0.0, 0.0, 0.0, 1.0)), POINT_DIM_LIGHT);
        // Cor saturada ESCURA em luminância (vermelho puro: Y = 0.2126): logo acima do
        // limiar ⇒ escuro; azul puro (Y = 0.0722): bem abaixo ⇒ claro. É o que pega a
        // troca r/g/b por uma média ingênua.
        assert_eq!(dim_dot_rgba(Rgba::new(1.0, 0.0, 0.0, 1.0)), POINT_DIM_DARK);
        assert_eq!(dim_dot_rgba(Rgba::new(0.0, 0.0, 1.0, 1.0)), POINT_DIM_LIGHT);
        // As BORDAS do limiar, dos dois lados (cinza linear Y == c).
        let just_above = CONTRAST_FLIP_Y + 1e-4;
        let just_below = CONTRAST_FLIP_Y - 1e-4;
        assert_eq!(
            dim_dot_rgba(Rgba::new(just_above, just_above, just_above, 1.0)),
            POINT_DIM_DARK
        );
        assert_eq!(
            dim_dot_rgba(Rgba::new(just_below, just_below, just_below, 1.0)),
            POINT_DIM_LIGHT
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
