//! Os gates da porta do layout — ver [`super`].

use super::{Split, hit, rects};
use ph2d_editor::zones::Rect as EditorRect;

/// ⭐⭐⭐ **OS QUATRO PEDAÇOS LADRILHAM A ÁREA EXACTAMENTE** — sem folga e sem sobreposição.
///
/// ⚠️ **É a lei que o `CenterSplit` desta casa pagou com um bug de produto** (report do Enio,
/// 25/08): `h · t` quase nunca é inteiro, e enquanto a porta devolvia a fracção o passe de sprites
/// recebia `422,4` e o `set_scissor_rect` ao lado `422`. A diferença era invisível parada e **um
/// movimento** num pan. Aqui a defesa é dupla: as arestas são **inteiras**, e as somas fecham.
#[test]
fn the_four_pieces_tile_the_area_exactly() {
    // ⚠️ Tamanhos ÍMPARES e origens fraccionárias de propósito — é aí que um arredondamento por
    // LARGURA (em vez de por aresta) deixa um pixel de folga no meio.
    for area in [
        EditorRect::new(0.0, 0.0, 1280.0, 720.0),
        EditorRect::new(0.0, 0.0, 1281.0, 721.0),
        EditorRect::new(10.5, 20.5, 999.0, 555.0),
        EditorRect::new(-3.25, 7.75, 640.5, 361.5),
    ] {
        let r = rects(area, Split::Quad);
        let q = r.as_slice();
        assert_eq!(q.len(), 4, "a divisão em quatro tem quatro pedaços");
        for p in q {
            assert!(
                p.x.fract() == 0.0
                    && p.y.fract() == 0.0
                    && p.w.fract() == 0.0
                    && p.h.fract() == 0.0,
                "os pixels saem INTEIROS da porta: {p:?}"
            );
            assert!(
                p.w > 0.0 && p.h > 0.0,
                "um pedaço vazio não é um viewport: {p:?}"
            );
        }
        // Ladrilham: cada aresta interior é o MESMO número para os dois vizinhos.
        assert_eq!(
            q[0].x + q[0].w,
            q[1].x,
            "sem folga entre as colunas de cima"
        );
        assert_eq!(
            q[2].x + q[2].w,
            q[3].x,
            "sem folga entre as colunas de baixo"
        );
        assert_eq!(
            q[0].y + q[0].h,
            q[2].y,
            "sem folga entre as linhas da esquerda"
        );
        assert_eq!(
            q[1].y + q[1].h,
            q[3].y,
            "sem folga entre as linhas da direita"
        );
        let soma: f32 = q.iter().map(|p| p.w * p.h).sum();
        let inteira = (area.x + area.w).round() - area.x.round();
        let alta = (area.y + area.h).round() - area.y.round();
        assert_eq!(
            soma,
            inteira * alta,
            "a soma das áreas TEM de ser a área — um pixel a menos é uma linha do fundo a aparecer \
             no meio da peça"
        );
    }
}

/// ⭐ **A vista única é a área inteira** — o estado em que a divisão não existe.
#[test]
fn a_single_viewport_is_the_whole_area() {
    let area = EditorRect::new(7.4, 3.6, 800.0, 600.0);
    let r = rects(area, Split::One);
    assert_eq!(r.as_slice().len(), 1);
    let p = r.as_slice()[0];
    assert_eq!((p.x, p.y, p.w, p.h), (7.0, 4.0, 800.0, 600.0));
}

/// ⭐⭐ **UM ponto tem UM dono** — a aresta interior não pertence aos dois.
///
/// ⚠️ Com dois donos o mesmo pixel daria uma órbita em duas câmeras, e o defeito só apareceria
/// quando a mão passasse exactamente pela linha do meio.
#[test]
fn a_point_on_the_seam_belongs_to_exactly_one_viewport() {
    let area = EditorRect::new(0.0, 0.0, 1000.0, 800.0);
    let r = rects(area, Split::Quad);
    // A costura vertical está em x = 500; a horizontal em y = 400.
    assert_eq!(
        hit(r.as_slice().iter().copied(), [499.0, 399.0]),
        Some(0),
        "cima-esquerda"
    );
    assert_eq!(
        hit(r.as_slice().iter().copied(), [500.0, 399.0]),
        Some(1),
        "a costura pertence à direita"
    );
    assert_eq!(
        hit(r.as_slice().iter().copied(), [499.0, 400.0]),
        Some(2),
        "…e à de baixo"
    );
    assert_eq!(hit(r.as_slice().iter().copied(), [500.0, 400.0]), Some(3));
    assert_eq!(
        hit(r.as_slice().iter().copied(), [-1.0, 10.0]),
        None,
        "fora da área não é de ninguém"
    );
    assert_eq!(
        hit(r.as_slice().iter().copied(), [1000.0, 10.0]),
        None,
        "a aresta EXTERIOR é de fora"
    );
}

/// ⭐ **A contagem e a disposição saem da mesma fonte.**
///
/// ⚠️ *Um `4` escrito ao lado de um `Split::Quad` é a segunda resposta à mesma pergunta* — e é a que
/// envelhece quando alguém acrescentar uma divisão em dois.
#[test]
fn the_count_and_the_named_views_agree() {
    use crate::field3d_views::Standard;
    assert_eq!(
        Split::One.count(),
        rects(EditorRect::new(0.0, 0.0, 8.0, 8.0), Split::One)
            .as_slice()
            .len()
    );
    assert_eq!(
        Split::Quad.count(),
        rects(EditorRect::new(0.0, 0.0, 8.0, 8.0), Split::Quad)
            .as_slice()
            .len()
    );
    assert_eq!(Split::Quad.named(0), Some(Standard::Top));
    assert_eq!(Split::Quad.named(1), Some(Standard::Right));
    assert_eq!(Split::Quad.named(2), Some(Standard::Front));
    assert_eq!(
        Split::Quad.named(3),
        None,
        "o quadrante do artista é o da PERSPECTIVA — é onde a mão dele já está"
    );
    assert_eq!(
        Split::One.named(0),
        None,
        "a vista única é sempre a do artista"
    );
}

/// ⭐⭐⭐ **A COSTURA: cada viewport guarda o retângulo que o LAYOUT lhe deu** (W90).
///
/// # Porque este é o gate que importa, e os de aritmética não bastam
///
/// O desenho calcula os retângulos e o **ponteiro** lê-os de volta dos viewports
/// (`field3d_smoke::viewport_at`), porque ele corre fora do quadro. São **duas** travessias, e um
/// clique roteado para o viewport errado orbita a câmera errada — sem erro, sem log, e só quando a
/// divisão está aberta.
///
/// ⚠️ *É a família da costura muda que este módulo já pagou cinco vezes* (o modificador na
/// escultura, a multi-seleção, o olho, o cadeado, o reparentar): os dois lados corretos e ninguém a
/// ligar os dois. Por isso este gate desenha **de verdade** e depois pergunta ao **ponteiro**.
#[test]
fn each_viewport_stores_the_rect_the_layout_gave_it() {
    use crate::field3d_scene::lasso_tests::{AREA, armed_with};
    use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Box {
                half: [0.4, 0.3, 0.2],
                round: 0.05,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("a peça");
    armed_with(&doc, |_| {
        let mut text = ph2d_text::TextSystem::without_system_fonts();
        crate::field3d_smoke::with_smoke(crate::field3d_smoke::toggle_split);
        let mut scene = ph2d_vector::VectorScene::new();
        crate::field3d_smoke::draw(AREA, ph2d_tokens::Theme::default(), &mut text, &mut scene);

        let esperados = rects(AREA, Split::Quad);
        crate::field3d_smoke::with_smoke(|s| {
            assert_eq!(s.vps.len(), 4, "a divisão devia ter aberto quatro vistas");
            for (i, r) in esperados.as_slice().iter().enumerate() {
                assert_eq!(
                    s.vps[i].area,
                    Some(*r),
                    "o viewport {i} guardou uma área que não é a do layout"
                );
            }
            // ⭐ E o PONTEIRO devolve o mesmo dono — a segunda travessia.
            for (i, r) in esperados.as_slice().iter().enumerate() {
                let centro = (r.x + r.w * 0.5, r.y + r.h * 0.5);
                assert_eq!(
                    crate::field3d_smoke::viewport_at(s, centro),
                    Some(i),
                    "o centro do viewport {i} foi roteado para outro"
                );
            }
        });
        // Volta à vista única para não deixar estado para o teste seguinte.
        crate::field3d_smoke::with_smoke(crate::field3d_smoke::toggle_split);
    });
}
