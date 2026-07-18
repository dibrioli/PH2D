//! Gates do **domínio SEGMENT** no shell (ADR-0114 §4.B). O motor (cortes → pedaços) é
//! gateado no MODELO (`ph2d-flip::segment_tests`, 11 gates com mutação provada); aqui só o
//! que o shell decide: **quem corta quem** — e as três recusas do doc do módulo.
//!
//! Cada gate nomeia a mutação que o derruba ([[reference_topic_mutation_proofs]]).

use super::*;
use ph2d_flip::{FlipDoc, FlipStroke, Hold, KeyKind, Point, Pose, Rgba};
use ph2d_vec_scene::Xform;

fn line(pts: &[(f32, f32)], width: f32) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &(x, y) in pts {
        s.push_point(Point {
            pos: Vec2::new(x, y),
            width,
            opacity: 1.0,
            color: Rgba::BLACK,
        });
    }
    s
}

/// Um objeto com uma camada "Art" (a ativa) segurando `art`, e uma 2ª camada "Other"
/// segurando `other` — as duas com chave no quadro 0.
fn obj_2layers(art: Vec<FlipStroke>, other: Vec<FlipStroke>) -> (FlipDoc, LayerId, LayerId) {
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("Flip");
    let o = doc.object_mut(oid).unwrap();
    let a = o.add_layer("Art");
    let da = o
        .insert_frame(a, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    o.drawing_mut(da).unwrap().strokes = art;
    let b = o.add_layer("Other");
    let db = o
        .insert_frame(b, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    o.drawing_mut(db).unwrap().strokes = other;
    (doc, a, b)
}

/// A camada ativa ("Art") do doc.
fn art_drawing(doc: &FlipDoc, layer: LayerId) -> &FlipDrawing {
    let o = doc.objects().first().unwrap();
    o.drawing(o.layer(layer).unwrap().drawing_at(0).unwrap())
        .unwrap()
}

/// Uma reta horizontal de 5 pontos (x = 0,10,20,30,40) na camada ativa.
fn h5() -> Vec<FlipStroke> {
    vec![line(
        &[
            (0.0, 0.0),
            (10.0, 0.0),
            (20.0, 0.0),
            (30.0, 0.0),
            (40.0, 0.0),
        ],
        2.0,
    )]
}

/// Uma vertical que cruza a `h5` em x=15 (⇒ corta o segmento 1).
fn crosser() -> FlipStroke {
    line(&[(15.0, -5.0), (15.0, 5.0)], 2.0)
}

fn cutters_of(doc: &FlipDoc, active: LayerId) -> FrameCutters {
    frame_cutters(doc.objects().first().unwrap(), 0, active)
}

/// 🔴 **O corte vem do QUADRO, não do desenho ativo** — a promessa central do modo (o
/// `§11`: *"raycast contra o BVH 2D do frame"*, e o corte é VISUAL). Um traço de OUTRA
/// camada, visível no mesmo quadro, corta este.
///
/// **Mutação que sangra:** colher só o desenho ativo (`if layer.id != active { continue }`)
/// — a linha deixa de ter corte e o clique acende ela inteira.
#[test]
fn a_stroke_on_another_visible_layer_cuts_this_one() {
    let (doc, art, _o) = obj_2layers(h5(), vec![crosser()]);
    let cutters = cutters_of(&doc, art);
    let map = cutters.piece_map(art_drawing(&doc, art), 0);
    assert_eq!(
        map,
        vec![0, 0, 1, 1, 1],
        "a vertical da outra camada parte a linha em dois pedacos"
    );
}

/// 🔴 **Camada INVISÍVEL não corta**: não está na tela, e o corte é visual.
///
/// **Mutação que sangra:** remover o `if !layer.visible { continue }` — o pedaço volta a
/// existir e o artista vê a linha partir por causa de algo que ele não enxerga.
#[test]
fn an_invisible_layer_does_not_cut() {
    let (mut doc, art, other) = obj_2layers(h5(), vec![crosser()]);
    doc.object_mut(doc.objects().first().unwrap().id)
        .unwrap()
        .layer_mut(other)
        .unwrap()
        .visible = false;
    let cutters = cutters_of(&doc, art);
    assert_eq!(
        cutters.piece_map(art_drawing(&doc, art), 0),
        vec![0; 5],
        "o que nao esta na tela nao corta"
    );
}

/// 🔴 O irmão de PRESENÇA do gate acima ([[feedback_absence_gate_needs_a_presence_sibling]]):
/// uma camada **TRAVADA corta** — você a VÊ. (A referência exclui a travada junto com a
/// invisível porque lá o conjunto que corta é o EDITÁVEL; aqui isso faria trancar uma
/// camada mudar, em silêncio, onde os pedaços do vizinho começam.)
///
/// Sem este gate, "não corta" ficaria verde com um `frame_cutters` que não colhe NADA.
#[test]
fn a_locked_but_visible_layer_still_cuts() {
    let (mut doc, art, other) = obj_2layers(h5(), vec![crosser()]);
    doc.object_mut(doc.objects().first().unwrap().id)
        .unwrap()
        .layer_mut(other)
        .unwrap()
        .locked = true;
    assert_eq!(
        cutters_of(&doc, art).piece_map(art_drawing(&doc, art), 0),
        vec![0, 0, 1, 1, 1],
        "travada e visivel: o artista a VE, entao ela corta"
    );
}

/// 🔴 **O anel invisível do balde (`hide_stroke`) não corta.** Uma região costeia a
/// arte-linha (BUGS #14: o balde ancora no EIXO da linha), então deixá-la cortar picaria a
/// linha em pedaços que o artista não desenhou.
///
/// **Mutação que sangra:** remover o `if s.hide_stroke { continue }` — a linha ganha corte.
#[test]
fn a_region_without_ink_does_not_cut() {
    let mut region = crosser();
    region.hide_stroke = true;
    let (doc, art, _o) = obj_2layers(h5(), vec![region]);
    assert_eq!(
        cutters_of(&doc, art).piece_map(art_drawing(&doc, art), 0),
        vec![0; 5],
        "anel invisivel do balde nao pica a linha"
    );
}

/// 🔴 **A POSE da chave entra no corte.** As camadas só se encontram em espaço de OBJETO;
/// a arte de cada uma é local à pose da chave dela. Sem aplicar a pose, dois desenhos que
/// se cruzam na tela não se cruzam na conta — e vice-versa.
///
/// **Mutação que sangra:** usar `(a, b)` cru em vez de `(pose.apply(a), pose.apply(b))` —
/// aqui a vertical está desenhada em x=0 e só CRUZA a linha porque a pose dela a desloca
/// para x=15; sem a pose, ela não cruza nada e o corte some.
#[test]
fn the_key_pose_places_the_cutter_where_it_is_seen() {
    let (mut doc, art, other) = obj_2layers(h5(), vec![line(&[(0.0, -5.0), (0.0, 5.0)], 2.0)]);
    let oid = doc.objects().first().unwrap().id;
    doc.object_mut(oid)
        .unwrap()
        .layer_mut(other)
        .unwrap()
        .set_frame_pose(0, Pose::from_translation(Vec2::new(15.0, 0.0)));
    assert_eq!(
        cutters_of(&doc, art).piece_map(art_drawing(&doc, art), 0),
        vec![0, 0, 1, 1, 1],
        "a vertical corta ONDE ELA APARECE (x=15), nao onde foi desenhada (x=0)"
    );
}

/// 🔴 **O clique acende o pedaço, e o lado do corte decide qual** — o seam do pick
/// (`hit_at` → `Where::Ink{i,t}`) com o motor.
///
/// **Mutação que sangra:** o `plan_down_segment` acender só o ponto do hit (voltar ao
/// domínio Point) — o pedaço vizinho fica apagado.
#[test]
fn a_click_lights_the_piece_the_cursor_fell_on() {
    let (mut doc, art, _o) = obj_2layers(h5(), vec![crosser()]);
    let cutters = cutters_of(&doc, art);
    let oid = doc.objects().first().unwrap().id;
    let did = doc
        .object(oid)
        .unwrap()
        .layer(art)
        .unwrap()
        .drawing_at(0)
        .unwrap();
    let dr = doc.object_mut(oid).unwrap().drawing_mut(did).unwrap();

    // Clique em (25, 0) = o segmento 2, depois do corte ⇒ o pedaço {2,3,4}.
    let hit = crate::flip_select_pick::hit_at(dr, Vec2::new(25.0, 0.0), 1.0, &Xform::IDENTITY);
    assert!(matches!(hit, Some((0, Where::Ink { i: 2, .. }))), "{hit:?}");
    plan_down_segment(dr, &cutters, hit, false, false);
    assert_eq!(
        dr.strokes[0].selected_point_indices(),
        vec![2, 3, 4],
        "acende o pedaco DEPOIS do corte, e so ele"
    );

    // Clique em (5, 0) = o segmento 0, antes do corte ⇒ o pedaço {0,1}.
    let hit = crate::flip_select_pick::hit_at(dr, Vec2::new(5.0, 0.0), 1.0, &Xform::IDENTITY);
    plan_down_segment(dr, &cutters, hit, false, false);
    assert_eq!(
        dr.strokes[0].selected_point_indices(),
        vec![0, 1],
        "e o outro lado do corte acende o outro pedaco"
    );
}

/// 🔴 **O marquee EXPANDE para os pedaços tocados** — o pós-processo da referência
/// (`apply_mask_as_segment_selection`). Uma caixa que pega UM ponto do pedaço acende o
/// pedaço inteiro; recortar o traço na borda da caixa é o oposto do que o modo promete.
///
/// **Mutação que sangra:** acender só os pontos dentro da caixa (o `apply_marquee_points`)
/// — a seleção para na borda.
#[test]
fn the_marquee_expands_to_the_whole_piece_it_touched() {
    let (mut doc, art, _o) = obj_2layers(h5(), vec![crosser()]);
    let cutters = cutters_of(&doc, art);
    let oid = doc.objects().first().unwrap().id;
    let did = doc
        .object(oid)
        .unwrap()
        .layer(art)
        .unwrap()
        .drawing_at(0)
        .unwrap();
    let dr = doc.object_mut(oid).unwrap().drawing_mut(did).unwrap();
    // Uma caixa que cobre SÓ o ponto 3 (x=30).
    apply_marquee_segments(
        dr,
        &cutters,
        Vec2::new(28.0, -1.0),
        Vec2::new(32.0, 1.0),
        false,
    );
    assert_eq!(
        dr.strokes[0].selected_point_indices(),
        vec![2, 3, 4],
        "tocar o ponto 3 acende o pedaco dele inteiro"
    );
}

/// Um quadrado 40×40 fechado, com tinta e preenchimento.
fn filled_square() -> FlipStroke {
    let mut s = line(&[(0.0, 0.0), (40.0, 0.0), (40.0, 40.0), (0.0, 40.0)], 2.0);
    s.closed = true;
    s.fill = Some(ph2d_flip::Fill {
        color: Rgba::new(1.0, 0.0, 0.0, 1.0),
        opacity: 1.0,
    });
    s
}

/// Acende o que `hit` mandar, e devolve os pontos acesos.
fn click_at(doc: &mut FlipDoc, art: LayerId, at: Vec2) -> Vec<usize> {
    let cutters = cutters_of(doc, art);
    let oid = doc.objects().first().unwrap().id;
    let did = doc
        .object(oid)
        .unwrap()
        .layer(art)
        .unwrap()
        .drawing_at(0)
        .unwrap();
    let dr = doc.object_mut(oid).unwrap().drawing_mut(did).unwrap();
    let hit = crate::flip_select_pick::hit_at(dr, at, 1.0, &Xform::IDENTITY);
    plan_down_segment(dr, &cutters, hit, false, false);
    dr.strokes[0].selected_point_indices()
}

/// 🔴 **O miolo de um preenchimento acende a forma INTEIRA, ignorando os cortes**: o
/// `fill` é do anel todo (a regra #4 do módulo), então quem aponta o miolo apontou a forma
/// e não uma aresta dela — não existe "um pedaço do preenchimento" para escolher.
///
/// O fixture PRECISA ter tinta: uma região `hide_stroke` não entra na lista de cortadores,
/// logo não tem corte nenhum, e "ignorar os cortes" seria impossível de errar ali (foi
/// exatamente assim que a 1ª versão deste gate ficou verde com a mutação aplicada —
/// [[reference_topic_fixture_discipline]]). Aqui o quadrado tem contorno E preenchimento, e
/// a vertical o corta em DOIS pedaços ({1,2} e {3,0}).
///
/// **Mutação que sangra:** o `piece_points` tratar `Where::Whole` como o pedaço do ponto 0
/// — o miolo acenderia `{3,0}`, meia forma escolhida arbitrariamente.
#[test]
fn clicking_the_inside_of_a_cut_filled_shape_lights_the_whole_ring() {
    let (mut doc, art, _o) = obj_2layers(
        vec![filled_square()],
        vec![line(&[(20.0, -5.0), (20.0, 45.0)], 2.0)],
    );
    // A vertical corta a base e o topo: sem o `Whole`, o miolo daria meio quadrado.
    assert_eq!(
        cutters_of(&doc, art).piece_map(art_drawing(&doc, art), 0),
        vec![1, 0, 0, 1],
        "o fixture PRECISA estar cortado, senao nao prova nada"
    );
    // (10, 20) está a 10 da aresta mais próxima — fora do alcance da tinta (5 px).
    assert_eq!(
        click_at(&mut doc, art, Vec2::new(10.0, 20.0)),
        vec![0, 1, 2, 3],
        "o miolo acende a forma inteira, nao o pedaco do ponto 0"
    );
}

/// 🔴 **Uma REGIÃO do balde (sem tinta nenhuma) é selecionável** — sem isto ela seria um
/// no-op mudo, e a cor do balde ficaria inalcançável no modo Segment.
///
/// **Mutação que sangra:** o `hit_on` recusar o fill (devolver `None` quando não há tinta)
/// — o clique no meio da cor não acende nada.
#[test]
fn a_region_without_ink_is_still_selectable() {
    let mut region = filled_square();
    region.hide_stroke = true;
    let (mut doc, art, _o) = obj_2layers(vec![region], vec![]);
    assert_eq!(
        click_at(&mut doc, art, Vec2::new(20.0, 20.0)),
        vec![0, 1, 2, 3],
        "a regiao do balde nao tem aresta onde mirar: o miolo E o gesto"
    );
}

/// 🔴 **Na borda de uma forma preenchida, a TINTA vence o preenchimento.** O `hit_on`
/// testa a tinta ANTES do fill desde o §4.B — o `stroke_at` não sente (ele é um OU), mas o
/// Segment sim: clicar na borda tem de dar a ARESTA, não a forma.
///
/// **Mutação que sangra:** voltar a ordem (fill primeiro) — a borda passa a devolver
/// `Whole` e clicar numa aresta de uma forma preenchida acende a forma toda.
#[test]
fn on_the_border_of_a_filled_shape_the_ink_wins_over_the_fill() {
    let mut shape = line(&[(0.0, 0.0), (40.0, 0.0), (40.0, 40.0), (0.0, 40.0)], 2.0);
    shape.closed = true;
    shape.fill = Some(ph2d_flip::Fill {
        color: Rgba::new(1.0, 0.0, 0.0, 1.0),
        opacity: 1.0,
    });
    let d = {
        let mut d = FlipDrawing::new();
        d.strokes = vec![shape];
        d
    };
    // (20, 0) está EM CIMA da base — e dentro do anel também (a borda é interior).
    let hit = crate::flip_select_pick::hit_at(&d, Vec2::new(20.0, 0.0), 1.0, &Xform::IDENTITY);
    assert!(
        matches!(hit, Some((0, Where::Ink { i: 0, .. }))),
        "a borda tem de dar a ARESTA, nao o preenchimento: {hit:?}"
    );
    // E o miolo continua dando a forma.
    assert_eq!(
        crate::flip_select_pick::hit_at(&d, Vec2::new(20.0, 20.0), 1.0, &Xform::IDENTITY),
        Some((0, Where::Whole)),
        "o miolo e o miolo"
    );
}

// ── O passe de HOVER (§4.C): a GUARDA ────────────────────────────────────────────
//
// O caminho POSITIVO do hover (cursor → pedaço) precisa de `gfx` (câmera/janela) e é
// coberto pelos gates de unidade (`hover_piece`, `piece_halo_path`) + o smoke. O que se
// gateia aqui é a GUARDA — a parte que apodrece: o preview não pode sobreviver a uma troca
// de modo nem competir com um arrasto. O `App` é dirigível sem janela (`crate::App::new()`).
//
// ⚠️ **O observável destes gates é `flip_segment_hover_at`, NÃO `flip_segment_hover`.** Sem
// `gfx` o caminho positivo devolve `None` de qualquer jeito (o pick precisa da câmera),
// então `flip_segment_hover` seria `None` com OU sem a guarda — um gate sobre ele ficaria
// verde com a mutação aplicada ([[feedback_a_green_gate_may_be_green_by_accident]], a
// armadilha que a mutação 2 pegou ao vivo). O que distingue "a guarda BARROU" de "a guarda
// PASSOU" é o carimbo do cursor: a guarda, ao barrar, o zera (`= None`); ao passar, ela o
// estampa (`= Some(cursor)`) antes de tentar o pick. E cada gate deixa `flip_wants_edit()`
// e o domínio de modo que SÓ a condição sob teste decida (senão outra condição limpa por
// ela e o teste não isola nada).

/// Um `App` ARMADO no modo Segment (tool Flip ativa, modo Edit) — o estado em que só a
/// condição sob teste (domínio ≠ Segment, ou gesto ativo) decide se a guarda barra.
fn app_armed_in_segment() -> crate::App {
    let mut app = crate::App::new();
    app.flip_active = true;
    app.flip_style = Some(ph2d_tool_flip::FlipStyleSnapshot {
        mode: ph2d_tool_flip::FlipMode::Edit,
        edit_domain: ph2d_tool_flip::EditDomain::Segment,
        ..Default::default()
    });
    app.last_pointer = (5.0, 5.0);
    // Um carimbo DIFERENTE do cursor atual, senão a guarda "cursor parado" curto-circuita
    // antes de a condição sob teste ser avaliada.
    app.flip_segment_hover_at = Some((1.0, 1.0));
    app
}

/// 🔴 **O hover some fora do modo Segment.** Trocar o domínio para Point/Stroke tem de
/// barrar o preview — senão um pedaço fantasma fica aceso no domínio errado.
///
/// Isola o `!is_segment`: `flip_wants_edit()` fica VERDADEIRO (armado no Edit) e só o
/// domínio muda. Mutação que sangra: tirar o termo `!is_segment` da condição de guarda — a
/// guarda passa em Point e estampa o cursor (`hover_at = Some`).
#[test]
fn the_hover_clears_when_the_domain_is_not_segment() {
    let mut app = app_armed_in_segment();
    app.flip_style = Some(ph2d_tool_flip::FlipStyleSnapshot {
        mode: ph2d_tool_flip::FlipMode::Edit,
        edit_domain: ph2d_tool_flip::EditDomain::Point, // ← só isto muda
        ..Default::default()
    });
    app.flip_segment_hover_refresh();
    assert_eq!(
        app.flip_segment_hover_at, None,
        "a guarda deixou passar fora do Segment (o cursor foi estampado)"
    );
}

/// 🔴 **O hover não é recomputado durante um GESTO** — armar/arrastar/soltar é o usuário
/// selecionando, não sondando; um preview competiria com o que ele arrasta.
///
/// Isola o `flip_edit_gesture.is_some()`: armado no Segment (as outras condições passam),
/// só o gesto barra. Mutação que sangra: tirar o termo do gesto — a guarda passa e estampa
/// o cursor.
#[test]
fn the_hover_is_suppressed_during_a_gesture() {
    let mut app = app_armed_in_segment();
    app.flip_edit_gesture = Some(crate::flip_edit_gesture::EditGesture::Click);
    app.flip_segment_hover_refresh();
    assert_eq!(
        app.flip_segment_hover_at, None,
        "a guarda deixou o hover competir com um gesto ativo (o cursor foi estampado)"
    );
}

/// 🔴 O irmão de PRESENÇA ([[feedback_absence_gate_needs_a_presence_sibling]]): armado no
/// Segment, SEM gesto, com o cursor MOVIDO, a guarda **PASSA** — estampa o cursor e segue
/// para o pick. Sem este gate, uma guarda que barrasse SEMPRE deixaria os dois de cima
/// verdes (o hover nunca competiria porque nunca existiria).
///
/// (`flip_segment_hover` fica `None` aqui — headless não tem `gfx` para o pick —, mas o
/// carimbo do cursor prova que a guarda deixou passar.)
#[test]
fn the_hover_proceeds_when_armed_and_the_cursor_moved() {
    let mut app = app_armed_in_segment();
    app.flip_segment_hover_refresh();
    assert_eq!(
        app.flip_segment_hover_at,
        Some((5.0, 5.0)),
        "a guarda barrou um hover legitimo (armado, sem gesto, cursor movido)"
    );
}
