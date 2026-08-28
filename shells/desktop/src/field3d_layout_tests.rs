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

/// ⭐⭐⭐ **TODA VISTA PARADA ASSENTA — não só a activa** (W90b, report do Enio 27/08).
///
/// > *«apenas a janela activa fica com o objecto desenhado liso, as demais ficam no modo de baixa
/// > resolução»*
///
/// # O defeito, e porque nenhum gate desta wave o via
///
/// Cada viewport decide o traçado seguinte com [`crate::field3d_preview::next_trace`], que compara o
/// **pedido anterior** com o estado de agora. O passe perguntava pelo pedido do viewport **ACTIVO** e
/// comparava-o com a câmera **deste** ⇒ para toda vista não-activa *«a câmera mudou?»* era **sempre
/// sim**, e ela ficava presa no quadro de movimento (grosso, sem anti-serrilhado) para sempre.
///
/// ⚠️ **Os gates da W90 mediam a GEOMETRIA** (os retângulos ladrilham, a costura tem um dono, cada
/// viewport guarda a sua área) e passaram todos: o defeito não estava em onde as vistas ficam, mas
/// em **com quem cada uma se compara**. *Uma divisão certa pode alimentar quatro laços errados.*
///
/// ⇒ a régua é o **estado em que cada vista PÁRA**: com a cena quieta, nenhuma pode continuar a
/// pedir um quadro de movimento.
#[test]
fn every_still_viewport_settles_not_only_the_active_one() {
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
        // ⚠️ **Espera LIMITADA, e o limite é generoso de propósito**: os traçados correm noutras
        // threads, e o que se afirma é a CONVERGÊNCIA, não um tempo. Ou as quatro assentam, ou o
        // produto está partido — não há um terceiro resultado que a máquina lenta produza.
        let mut assentaram = false;
        for _ in 0..600 {
            let mut scene = ph2d_vector::VectorScene::new();
            crate::field3d_smoke::draw(AREA, ph2d_tokens::Theme::default(), &mut text, &mut scene);
            let pronto = crate::field3d_smoke::with_smoke(|s| {
                s.vps.iter().all(|v| v.probe_resting_state().is_some())
            })
            .unwrap_or(false);
            if pronto {
                assentaram = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(
            assentaram,
            "as quatro vistas não chegaram a um quadro pronto em 3 s"
        );
        crate::field3d_smoke::with_smoke(|s| {
            for (i, v) in s.vps.iter().enumerate() {
                let grosso = v
                    .probe_resting_state()
                    .expect("a vista está parada com um quadro");
                assert!(
                    !grosso,
                    "o viewport {i} parou a pedir um quadro de MOVIMENTO com a cena quieta — ele \
                     nunca sobe os degraus do assentar, e o artista vê-o em baixa resolução para \
                     sempre (activo = {})",
                    s.active
                );
            }
        });
        crate::field3d_smoke::with_smoke(crate::field3d_smoke::toggle_split);
    });
}

/// ⭐⭐⭐ **A VISTA ACTIVA CHEGA PRIMEIRO** (W90c) — a prioridade que o relógio exigiu.
///
/// # O número que a manda existir
///
/// `the_price_of_four_views.rs`, máquina calma, uma edição a `1280×720`: uma vista de área inteira
/// custa `156,2 ms`, uma de um quarto `64,5`, e **quatro ao mesmo tempo `253,7`** — `3,93×` uma
/// sozinha. ⇒ elas **não ganham nada** por correrem juntas (cada uma já satura a máquina com o
/// `rayon`, então só se fatiam), e sem prioridade a vista onde a mão do artista está espera pelas
/// outras três: `254 ms` em vez de `64`.
///
/// ⚠️ **A afirmação é de ORDEM, não de relógio** — e é por isso que ela não é um gate de tempo
/// (aqueles reprovam sob fan-out sem nada mudar). Com a guarda, uma vista não-activa **nem começa**
/// enquanto a activa tem traçado em voo ⇒ a imagem da activa aparece pelo menos um quadro antes de
/// qualquer outra, por construção.
#[test]
fn the_active_viewport_gets_its_image_first() {
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
        // ⚠️ **Do FRIO**: nenhuma vista tem imagem, que é o instante em que a ordem se decide.
        crate::field3d_smoke::with_smoke(|s| {
            for v in &mut s.vps {
                v.probe_forget_frame();
            }
        });
        let mut primeiro = [usize::MAX; 4];
        for tick in 0..600 {
            let mut scene = ph2d_vector::VectorScene::new();
            crate::field3d_smoke::draw(AREA, ph2d_tokens::Theme::default(), &mut text, &mut scene);
            let tem = crate::field3d_smoke::with_smoke(|s| {
                let mut t = [false; 4];
                for (i, v) in s.vps.iter().enumerate() {
                    t[i] = v.probe_has_frame();
                }
                t
            })
            .unwrap_or([false; 4]);
            for i in 0..4 {
                if tem[i] && primeiro[i] == usize::MAX {
                    primeiro[i] = tick;
                }
            }
            if primeiro.iter().all(|t| *t != usize::MAX) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let activa = crate::field3d_smoke::with_smoke(|s| s.active).expect("armado");
        assert!(
            primeiro.iter().all(|t| *t != usize::MAX),
            "nem todas as vistas chegaram a ter imagem: {primeiro:?}"
        );
        for i in 0..4 {
            if i == activa {
                continue;
            }
            assert!(
                primeiro[activa] < primeiro[i],
                "a vista {i} recebeu imagem no tique {} e a ACTIVA ({activa}) só no {} — sem \
                 prioridade a vista onde a mão está espera pelas outras três (254 ms contra 64)",
                primeiro[i],
                primeiro[activa]
            );
        }
        crate::field3d_smoke::with_smoke(crate::field3d_smoke::toggle_split);
    });
}
