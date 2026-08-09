//! Gates da **porta única do glifo** (plano UI/UX W8b — o `IconButton`).

use super::*;
use ph2d_vec_scene::{VecPath, rectangle, star};

/// Uma estrela deitada — larga e baixa de propósito, para a lei uniforme ter o que provar.
fn wide_star() -> VecPath {
    star([3.0, -7.0], 4.0, 1.0, 5, 0.45)
}

/// **O lado maior encosta na viewbox, e a figura fica CENTRADA nela.**
///
/// ⚠️ O `24` não é um número desta wave: é a viewbox que o `paint_icon_path` do catálogo divide
/// para encaixar o glifo no botão. Normalizar noutra escala desenharia o ícone do tamanho errado
/// dentro da moldura — e o defeito seria *o ícone parece pequeno*, que ninguém atribui a uma
/// constante.
#[test]
fn the_glyph_fills_the_icon_viewbox_and_is_centred() {
    let bb = icon_face(&wide_star())
        .expect("a estrela tem figura")
        .bounding_box();
    let (w, h) = (bb.width(), bb.height());
    assert!(
        (w.max(h) - 24.0).abs() < 1e-9,
        "o lado maior nao encostou na viewbox: {w} x {h}"
    );
    // Centrada: a folga do lado menor é igual dos dois lados.
    assert!(
        (bb.x0 + bb.x1 - 24.0).abs() < 1e-9,
        "descentrada em x: {bb:?}"
    );
    assert!(
        (bb.y0 + bb.y1 - 24.0).abs() < 1e-9,
        "descentrada em y: {bb:?}"
    );
}

/// **A ESCALA É UNIFORME** — um glifo é uma figura, e esticá-la para encher a caixa mudaria o
/// desenho que o artista fez.
///
/// ⚠️ Este gate é o que separa *normalizar* de *deformar*, e a fixture tem de ser larga: numa
/// figura quadrada as duas leis dão o mesmo resultado, e o gate ficaria verde por vácuo.
#[test]
fn the_normalisation_is_uniform_so_the_drawing_keeps_its_shape() {
    let src = wide_star();
    let before = ph2d_vec_render::build_bezpath(&src.cooked()).bounding_box();
    let after = icon_face(&src)
        .expect("a estrela tem figura")
        .bounding_box();
    let (ra, rb) = (
        after.width() / after.height(),
        before.width() / before.height(),
    );
    assert!(
        (ra - rb).abs() < 1e-9,
        "a razao de aspecto mudou: {rb} -> {ra} (o glifo foi esticado)"
    );
}

/// **Sem figura, sem glifo** — e o botão desenha a moldura, que é o neutro.
#[test]
fn a_path_without_a_figure_has_no_face() {
    assert!(
        icon_face(&VecPath::default()).is_none(),
        "um caminho vazio inventou um glifo"
    );
    assert!(
        icon_face(&rectangle([1.0, 1.0], [1.0, 1.0])).is_none(),
        "um ponto (bbox de lado zero nos DOIS eixos) inventou um glifo"
    );
}

/// **Uma RETA tem glifo**, e a distinção é o ponto: ela é degenerada num eixo só.
///
/// ⚠️ Recusá-la seria confundir *não há figura* com *a figura é fina*: o lado maior dá a escala, o
/// menor centra-se, e o desenho é uma barra — que é exactamente o que o artista desenhou.
#[test]
fn a_straight_line_still_has_a_face() {
    let bb = icon_face(&rectangle([0.0, 5.0], [8.0, 5.0]))
        .expect("uma reta e' uma figura")
        .bounding_box();
    assert!(
        (bb.width() - 24.0).abs() < 1e-9,
        "a reta nao escalou: {bb:?}"
    );
    assert!(bb.height() < 1e-9, "a reta ganhou altura: {bb:?}");
}

/// Todo ponto de âncora e de controle de um caminho, para perguntar por EXTREMOS.
fn points(bp: &BezPath) -> Vec<ph2d_vector::Point> {
    use ph2d_vector::PathEl;
    let mut v = Vec::new();
    for el in bp.elements() {
        match *el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => v.push(p),
            PathEl::QuadTo(a, b) => v.extend([a, b]),
            PathEl::CurveTo(a, b, c) => v.extend([a, b, c]),
            PathEl::ClosePath => {}
        }
    }
    v
}

/// A posição de `p` na largura da própria figura — 0 na borda esquerda, 1 na direita.
///
/// ⚠️ É RELATIVA de propósito: assim o oráculo não precisa conhecer a escala nem a translação que
/// a normalização escolheu, e não vira espelho dela.
fn along_x(p: ph2d_vector::Point, bb: ph2d_vector::Rect) -> f64 {
    (p.x - bb.x0) / bb.width()
}

/// **O PONTO MAIS ALTO NO DOCUMENTO É O PONTO MAIS ALTO NA TELA** (report do Enio, 2026-08-09:
/// *"o ícone desenhado pelo usuário fica de cabeça para baixo no botão"*).
///
/// ⚠️ **É o gate que faltava, e o motivo de o defeito ter shipado está nos irmãos acima:** todos
/// medem a CAIXA — extensão, centragem, razão de aspecto, degeneração — e **uma caixa é simétrica
/// sob inversão**. Nenhum deles pode distinguir a figura da figura espelhada, então os cinco
/// ficavam verdes sobre um ícone de ponta-cabeça.
///
/// O oráculo não conhece a fórmula: ele pergunta **qual VÉRTICE** está no topo de cada lado, e
/// identifica-o pela posição relativa na largura. Se a inversão sumir, o vértice do topo passa a
/// ser outro e a posição não bate.
#[test]
fn the_glyph_is_not_upside_down() {
    let src = wide_star();
    let world = ph2d_vec_render::build_bezpath(&src.cooked());
    let glyph = icon_face(&src).expect("a estrela tem figura");
    let (wbb, gbb) = (world.bounding_box(), glyph.bounding_box());

    // No documento Y aponta para CIMA, logo o topo é o MAIOR y.
    let top_world = points(&world)
        .into_iter()
        .max_by(|a, b| a.y.total_cmp(&b.y))
        .expect("a estrela tem pontos");
    // Na viewbox do ícone Y aponta para BAIXO, logo o topo é o MENOR y.
    let top_glyph = points(&glyph)
        .into_iter()
        .min_by(|a, b| a.y.total_cmp(&b.y))
        .expect("o glifo tem pontos");

    let (a, b) = (along_x(top_world, wbb), along_x(top_glyph, gbb));
    assert!(
        (a - b).abs() < 1e-9,
        "o vertice do topo mudou de lugar ({a} no documento, {b} no glifo): o icone esta' \
         espelhado em Y — e' o desenho do artista de cabeca para baixo dentro do botao"
    );
}

/// **A PREMISSA: os dois espaços discordam sobre onde é em cima.**
///
/// ⚠️ Sem isto o gate acima afirma *"há uma inversão"* sem dizer **por quê**, e alguém que a leia
/// não distingue uma lei de um acidente da implementação. Aqui a expectativa é derivada da
/// **CÂMERA**, que é a autoridade sobre a direção de Y do documento — não de `icon_face`.
///
/// E é ele que fala no dia em que a convenção do canvas mudar: se a câmera deixar de inverter, é
/// este gate que sangra e aponta para a porta única em vez de deixar o ícone virar sozinho.
#[test]
fn the_document_is_y_up_and_the_icon_viewbox_is_y_down() {
    use ph2d_host::events::WindowSize;
    use ph2d_render::Camera2d;
    use ph2d_vector::Point;

    let cam = Camera2d::default();
    let to_screen = cam.world_to_screen_affine(WindowSize::new(800, 600));
    let origin = to_screen * Point::new(0.0, 0.0);
    let higher = to_screen * Point::new(0.0, 1.0);
    assert!(
        higher.y < origin.y,
        "premissa falsa: a camera parou de inverter Y, e entao a conversao do `icon_face` \
         passou a ser a ERRADA (origem {origin:?}, acima {higher:?})"
    );
}

/// **O CANVAS E O PAINEL LEEM O MESMO GLIFO** — o gate que carrega a fatia.
///
/// As duas metades precisam do glifo por motivos diferentes (o canvas pinta uma curva, o codegen
/// escreve texto), e é exactamente aí que nasce a divergência que só uma screenshot revela. Elas
/// percorrem [`icon_face`]; o que muda depois é só a REPRESENTAÇÃO, e este gate afirma que a
/// viagem por ela é sem perda.
///
/// ⚠️ Ele afirma **igualdade de curva**, não semelhança: o `Display` de `f64` do Rust escreve a
/// string mais curta que re-lê ao bit, então um `to_svg`/`from_svg` que perdesse precisão faria
/// isto sangrar em vez de desenhar um ícone quase-certo que ninguém compara.
#[test]
fn the_glyph_survives_the_trip_through_text() {
    let face = icon_face(&wide_star()).expect("a estrela tem figura");
    let back = BezPath::from_svg(&face.to_svg()).expect("o SVG que nos escrevemos re-le");
    assert_eq!(
        face.to_svg(),
        back.to_svg(),
        "o glifo mudou na viagem por texto — o painel desenharia outro icone"
    );
    assert_eq!(face.elements().len(), back.elements().len());
}
