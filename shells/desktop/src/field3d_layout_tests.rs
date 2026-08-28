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
        // ⚠️ **Com a costura em vários sítios** (W92): o ladrilhamento tem de valer para toda
        // posição do divisor, e não só para o meio — é ao arrastar que um arredondamento por
        // LARGURA deixaria a linha do fundo a aparecer.
        for (tx, ty) in [(0.5f32, 0.5f32), (0.25, 0.75), (0.331, 0.667), (0.75, 0.25)] {
            let r = rects(area, Split::quad().with_t(tx, ty));
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
                "a soma das áreas TEM de ser a área com t=({tx}, {ty}) — um pixel a menos é uma linha \
             do fundo a aparecer no meio da peça"
            );
        }
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
    let r = rects(area, Split::quad());
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
/// ⚠️ *Um `4` escrito ao lado de um `Split::quad()` é a segunda resposta à mesma pergunta* — e é a que
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
        Split::quad().count(),
        rects(EditorRect::new(0.0, 0.0, 8.0, 8.0), Split::quad())
            .as_slice()
            .len()
    );
    assert_eq!(Split::quad().named(0), Some(Standard::Top));
    assert_eq!(Split::quad().named(1), Some(Standard::Right));
    assert_eq!(Split::quad().named(2), Some(Standard::Front));
    assert_eq!(
        Split::quad().named(3),
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

        let esperados = rects(AREA, Split::quad());
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

/// ⭐⭐⭐ **A COSTURA AGARRA-SE, E SÓ ELA** (W92).
///
/// ⚠️ **A faixa de pega é maior do que a linha desenhada**, e isso é a lei de todo divisor de
/// janela: a pega é uma afirmação sobre o que o **dedo** alcança, não sobre o que o olho vê.
/// Apontar para uma linha de um pixel seria um gesto que só acerta por sorte.
///
/// ⭐ E o **cruzamento agarra as duas** — é o que o Blender faz, e o que a mão espera quando aponta
/// para o meio.
#[test]
fn the_seam_is_grabbable_and_nothing_else_is() {
    use super::seam_grab;
    let area = EditorRect::new(0.0, 0.0, 1000.0, 800.0);
    let split = Split::quad();
    // As costuras estão em x = 500 e y = 400.
    assert_eq!(seam_grab(area, split, [500.0, 100.0]), Some((true, false)));
    assert_eq!(seam_grab(area, split, [200.0, 400.0]), Some((false, true)));
    assert_eq!(
        seam_grab(area, split, [500.0, 400.0]),
        Some((true, true)),
        "o cruzamento agarra as DUAS costuras"
    );
    assert_eq!(
        seam_grab(area, split, [250.0, 200.0]),
        None,
        "o meio de um quadrante não é uma pega — ali o arrasto é a órbita, que é o gesto principal"
    );
    assert_eq!(
        seam_grab(area, Split::One, [500.0, 400.0]),
        None,
        "sem divisão não há costura para agarrar"
    );
    // ⚠️ A faixa tem de ser mais larga que a linha, mas não tão larga que roube a órbita.
    assert!(
        seam_grab(area, split, [497.0, 100.0]).is_some(),
        "3 px ao lado ainda agarra"
    );
    assert!(
        seam_grab(area, split, [480.0, 100.0]).is_none(),
        "20 px ao lado já é o quadrante"
    );
}

/// ⭐⭐ **A LEI PURA do divisor** — ⚠️ e ela **não** guarda o gesto: ver o gate da costura logo
/// abaixo, que é o que exercita o `advance` de verdade.
///
/// **O divisor segue o dedo em ABSOLUTO** — arrastar até ao batente e voltar não o desloca (W92).
///
/// ⚠️ **É a armadilha dos incrementos**, e este módulo já a pagou uma vez no gizmo (a âncora
/// congelada da W26): uma soma de deltas acumula o erro de **cada** trava, e quem arrasta até ao
/// limite e volta encontra a costura permanentemente deslocada da mão. *Mede-se o TOTAL contra uma
/// origem que não se mexe* — aqui, o próprio canvas.
#[test]
fn dragging_the_divider_to_the_stop_and_back_leaves_it_under_the_finger() {
    use super::t_at;
    let area = EditorRect::new(0.0, 0.0, 1000.0, 800.0);
    let mut split = Split::quad();
    // Uma varredura que ATRAVESSA os dois batentes e volta ao meio.
    for x in [500.0f32, 900.0, 950.0, 990.0, 300.0, 100.0, 10.0, 500.0] {
        let (tx, ty) = t_at(area, [x, 400.0]);
        split = split.with_t(tx, ty);
    }
    let Split::Quad { tx, .. } = split else {
        panic!("continua dividida")
    };
    assert!(
        (tx - 0.5).abs() < 1e-6,
        "depois de bater nos dois limites, a costura voltou a {tx} em vez de 0,5 — o arrasto está a \
         somar incrementos"
    );
    // ⭐ E o batente guarda um quarto para cada lado, que é a lei da casa (`CenterSplit`).
    let extremo = Split::quad().with_t(t_at(area, [990.0, 10.0]).0, t_at(area, [990.0, 10.0]).1);
    let Split::Quad { tx, ty } = extremo else {
        panic!("continua dividida")
    };
    assert!(
        (0.25..=0.75).contains(&tx) && (0.25..=0.75).contains(&ty),
        "o batente deixou t=({tx}, {ty}) fora de [0,25 · 0,75] — um quadrante pode ficar sem área"
    );
}

/// ⭐⭐⭐ **A COSTURA DO DIVISOR: o gesto REAL move a linha, e em absoluto** (W92).
///
/// # ⚠️ Porque o gate de cima não bastava
///
/// Ele prova `t_at` + `with_t`, que são a lei **pura**. Se o `advance` somasse incrementos —
/// exactamente o defeito que este módulo já pagou no gizmo (a âncora congelada da W26) — aquele
/// gate ficaria **verde**. *A causa nº 1 da semana perdida no Painter foi esta: os dois lados
/// corretos e ninguém a ligar os dois.*
///
/// Aqui o caminho é o de produção: `begin` na costura → `advance` ao longo de um arrasto que bate
/// nos **dois** limites → a linha volta a estar debaixo do dedo.
#[test]
fn the_real_gesture_moves_the_divider_and_does_not_drift() {
    use crate::field3d_input::{advance, begin};
    use crate::field3d_scene::lasso_tests::{AREA, armed_with};
    use crate::field3d_smoke::{Drag, with_smoke};
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
        with_smoke(crate::field3d_smoke::toggle_split);
        // Um quadro para os viewports guardarem as áreas — é delas que sai o canvas.
        let mut scene = ph2d_vector::VectorScene::new();
        crate::field3d_smoke::draw(AREA, ph2d_tokens::Theme::default(), &mut text, &mut scene);

        let meio = (AREA.x + AREA.w * 0.5, AREA.y + AREA.h * 0.5);
        with_smoke(|s| {
            assert!(
                begin(s, winit::event::MouseButton::Left, Drag::Orbit, false, meio),
                "o botão na costura tem de ser aceite"
            );
            assert_eq!(
                s.drag,
                Some(Drag::Divider(true, true)),
                "no cruzamento, o gesto agarra as DUAS costuras — e não a órbita"
            );
            // Um arrasto que atravessa os dois batentes e volta ao meio.
            for f in [0.9f32, 0.99, 0.05, 0.01, 0.5] {
                advance(s, AREA.x + AREA.w * f, AREA.y + AREA.h * f);
            }
            let crate::field3d_layout::Split::Quad { tx, ty } = s.split else {
                panic!("continua dividida")
            };
            assert!(
                (tx - 0.5).abs() < 1e-6 && (ty - 0.5).abs() < 1e-6,
                "depois de bater nos dois limites e voltar ao meio, a costura ficou em ({tx}, {ty}) \
                 — o arrasto está a somar incrementos em vez de medir o total"
            );
            s.drag = None;
        });
        // ⭐ E a órbita continua a ser o gesto do MEIO de um quadrante.
        with_smoke(|s| {
            let dentro = (AREA.x + AREA.w * 0.25, AREA.y + AREA.h * 0.25);
            assert!(begin(
                s,
                winit::event::MouseButton::Left,
                Drag::Orbit,
                false,
                dentro
            ));
            assert_eq!(
                s.drag,
                Some(Drag::Orbit),
                "no meio de um quadrante o arrasto tem de continuar a ser a órbita"
            );
            s.drag = None;
        });
        with_smoke(crate::field3d_smoke::toggle_split);
    });
}
