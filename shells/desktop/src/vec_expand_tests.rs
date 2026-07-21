//! Testes de [`crate::vec_expand`] — arquivo irmão.
//!
//! O motor está gateado na `ph2d-vec-boolean`. O que se prova AQUI é o que só existe na
//! shell: quais paths o comando pega, em que z o resultado fica, o que sobra da original, e
//! que o gesto inteiro custa **UM** Ctrl+Z.

use super::*;
use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecPath, VecVertex, WidthProfile};

fn square(s: f64) -> VecPath {
    VecPath {
        verts: [[0.0, 0.0], [s, 0.0], [s, s], [0.0, s]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

/// Cena com os `paths` dados, todos selecionados. Sem poses (`VecXforms` vazio): a fronteira
/// de pose já é exercida pela booleana irmã, e aqui o que se mede é o efeito no DOCUMENTO.
fn scene_with(paths: Vec<VecPath>) -> (VecScene, History, PenTool, VecXforms) {
    let mut scene = VecScene::new();
    let ids: Vec<VecPathId> = paths.into_iter().map(|p| scene.push_path(p)).collect();
    let mut pen = PenTool::default();
    pen.select_many(&ids);
    (scene, History::default(), pen, VecXforms::default())
}

fn stroked(mut p: VecPath) -> VecPath {
    p.stroke = Some(StrokeSpec::new(Rgba8::new(200, 30, 30, 255), 2.0));
    p
}

fn filled(mut p: VecPath) -> VecPath {
    p.fill = Some(Paint::Solid(Rgba8::new(30, 30, 200, 255)));
    p
}

const MITER: Expand = Expand::Offset {
    join: LineJoin::Miter,
    side: ph2d_vec_scene::OffsetSide::Both,
};

/// **O id do botão diz o comando.** Um id de outra seção não é nosso — senão o dreno
/// atenderia cliques alheios e roubaria o Union.
#[test]
fn the_button_ids_map_to_their_commands() {
    assert!(matches!(
        expand_for_id(ph2d_editor::ids::VECTOR_EXPAND_OFFSET_PATH),
        Some(Expand::Offset { .. })
    ));
    assert_eq!(
        expand_for_id(ph2d_editor::ids::VECTOR_EXPAND_OUTLINE_STROKE),
        Some(Expand::OutlineStroke)
    );
    assert_eq!(expand_for_id(ph2d_editor::ids::VECTOR_BOOL_UNION), None);
    assert_eq!(expand_for_id(ph2d_editor::ids::VECTOR_MODE_PEN), None);
}

/// **Offsetar N formas offseta as N** — por-path, não N-ário. Uma versão que fundisse a
/// seleção devolveria UMA forma, e o artista perderia duas.
#[test]
fn offsetting_three_shapes_offsets_all_three() {
    let (mut scene, mut hist, mut pen, xf) = scene_with(vec![square(10.0); 3]);
    apply_vec_expand(&mut scene, &mut hist, &mut pen, &xf, MITER, 1.0);
    assert_eq!(scene.paths().len(), 3, "as tres continuam existindo");
    for p in scene.paths() {
        let a = ph2d_vec_boolean::area(p);
        assert!((a - 144.0).abs() < 1.0, "area {a} (10+2·1 ao quadrado)");
    }
    assert_eq!(pen.selected_paths().len(), 3, "as tres ficam selecionadas");
}

/// **UM passo de undo para o gesto inteiro**, mesmo mexendo em três formas. Um passo por
/// forma faria "desfazer o Expand" custar tantos Ctrl+Z quantos objetos — e nada pareceria
/// errado.
#[test]
fn the_whole_gesture_is_one_undo_step() {
    let (mut scene, mut hist, mut pen, xf) = scene_with(vec![square(10.0); 3]);
    apply_vec_expand(&mut scene, &mut hist, &mut pen, &xf, MITER, 1.0);
    let restored = hist.undo(&scene).expect("ha um passo a desfazer");
    assert_eq!(restored.paths().len(), 3);
    for p in restored.paths() {
        let a = ph2d_vec_boolean::area(p);
        assert!((a - 100.0).abs() < 1e-6, "voltou ao original? area {a}");
    }
    assert!(
        hist.undo(&restored).is_none(),
        "havia um SEGUNDO passo — o gesto gravou mais de um"
    );
}

/// **Outline Stroke numa forma com preenchimento deixa DOIS objetos**: o miolo (agora sem
/// traço) e o contorno assado por cima. Fundir os dois num só perderia uma das duas cores —
/// e elas são diferentes justamente por serem coisas diferentes.
#[test]
fn outlining_a_filled_shape_keeps_the_fill_as_its_own_object() {
    let (mut scene, mut hist, mut pen, xf) = scene_with(vec![filled(stroked(square(10.0)))]);
    apply_vec_expand(
        &mut scene,
        &mut hist,
        &mut pen,
        &xf,
        Expand::OutlineStroke,
        0.0,
    );
    assert_eq!(scene.paths().len(), 2, "o miolo + o contorno");
    let base = &scene.paths()[0];
    assert!(base.fill.is_some(), "o miolo manteve o preenchimento");
    assert_eq!(base.stroke, None, "…e perdeu o traço, que agora é forma");
    let ring = &scene.paths()[1];
    assert_eq!(
        ring.fill,
        Some(Paint::Solid(Rgba8::new(200, 30, 30, 255))),
        "o contorno é preenchido com a COR do traço"
    );
    // O anel: um quadrado de 10 com caneta de 2 mede ~4·10·2 = 80 de tinta.
    let a = ph2d_vec_boolean::area(ring);
    assert!((60.0..100.0).contains(&a), "area do anel {a}");
}

/// …e **sem preenchimento não sobra miolo**: a forma original era só o traço, então
/// convertê-lo a consome. Deixar um path invisível de área zero seria lixo para o artista
/// caçar na Hierarquia.
#[test]
fn outlining_a_stroke_only_shape_consumes_the_original() {
    let (mut scene, mut hist, mut pen, xf) = scene_with(vec![stroked(square(10.0))]);
    apply_vec_expand(
        &mut scene,
        &mut hist,
        &mut pen,
        &xf,
        Expand::OutlineStroke,
        0.0,
    );
    assert_eq!(scene.paths().len(), 1, "só o contorno assado");
    assert_eq!(scene.paths()[0].stroke, None);
}

/// **Uma seleção onde nada pode ser convertido não gasta um passo de undo.** Sem isto, o
/// artista clicaria Outline Stroke sobre formas sem traço, veria a tela igual, e o Ctrl+Z
/// seguinte desfaria uma edição ANTERIOR que ele não pediu para desfazer.
#[test]
fn a_gesture_that_changes_nothing_records_nothing() {
    let (mut scene, mut hist, mut pen, xf) = scene_with(vec![square(10.0), square(4.0)]);
    let before = scene.paths().len();
    apply_vec_expand(
        &mut scene,
        &mut hist,
        &mut pen,
        &xf,
        Expand::OutlineStroke,
        0.0,
    );
    assert_eq!(scene.paths().len(), before);
    assert!(
        hist.undo(&scene).is_none(),
        "gravou um passo de undo para uma edição que não aconteceu"
    );
}

/// **O resultado fica no z da original**, não salta para a frente de tudo. Uma forma que
/// pula de camada ao ser offsetada reordena a arte por um motivo invisível.
#[test]
fn the_result_keeps_the_z_of_the_shape_it_replaced() {
    let mut scene = VecScene::new();
    let back = scene.push_path(square(4.0)); // z=0, NÃO selecionada
    let mid = scene.push_path(square(10.0)); // z=1, a que será offsetada
    let front = scene.push_path(square(4.0)); // z=2, NÃO selecionada
    let mut pen = PenTool::default();
    pen.select_many(&[mid]);
    let mut hist = History::default();
    apply_vec_expand(
        &mut scene,
        &mut hist,
        &mut pen,
        &VecXforms::default(),
        MITER,
        1.0,
    );
    assert_eq!(scene.paths().len(), 3);
    assert_eq!(scene.paths()[0].id, back, "o de trás não se mexeu");
    assert_eq!(scene.paths()[2].id, front, "o da frente não se mexeu");
    let a = ph2d_vec_boolean::area(&scene.paths()[1]);
    assert!(
        (a - 144.0).abs() < 1.0,
        "o do MEIO é o offsetado (area {a})"
    );
}

/// **O Power Stroke chega ao documento pelo mesmo caminho dos outros dois** — mesmo dreno,
/// mesmo undo, mesma regra do miolo. O que ele produz está gateado no motor; o que se prova
/// AQUI é que o comando existe na shell e assa a seleção.
#[test]
fn the_power_stroke_command_bakes_the_selection() {
    let taper = WidthProfile {
        start: 0.25,
        mid: 2.0,
        end: 0.25,
        position: 0.5,
    };
    let (mut scene, mut hist, mut pen, xf) = scene_with(vec![stroked(VecPath {
        verts: vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::corner([20.0, 0.0]),
        ],
        closed: false,
        ..VecPath::default()
    })]);
    apply_vec_expand(
        &mut scene,
        &mut hist,
        &mut pen,
        &xf,
        Expand::PowerStroke { profile: taper },
        0.0,
    );
    assert_eq!(scene.paths().len(), 1, "o traço virou UMA forma");
    assert_eq!(scene.paths()[0].stroke, None, "…e ela É o traço");
    assert!(
        hist.undo(&scene).is_some(),
        "o gesto tem de custar um passo de undo"
    );
}

/// **Um perfil UNIFORME não gasta um passo de undo** — o motor recusa (é o Outline Stroke), e
/// o comando não pode registar uma edição que não aconteceu.
#[test]
fn a_uniform_power_stroke_records_nothing() {
    let (mut scene, mut hist, mut pen, xf) = scene_with(vec![stroked(square(10.0))]);
    apply_vec_expand(
        &mut scene,
        &mut hist,
        &mut pen,
        &xf,
        Expand::PowerStroke {
            profile: WidthProfile::UNIFORM,
        },
        0.0,
    );
    assert!(hist.undo(&scene).is_none());
}

// ───────────────────────── Offset Path ao vivo (o arrasto) ─────────────────────────

/// A sessão viva só existe se há forma selecionada — arrastar o slider sem seleção não tem o
/// que prever (e o botão também não faria nada).
#[test]
fn an_offset_session_needs_a_selection() {
    let mut scene = VecScene::new();
    scene.push_path(square(10.0));
    let empty = PenTool::default();
    assert!(
        OffsetSession::begin(&scene, &empty, &VecXforms::default()).is_none(),
        "sem seleção não há sessão"
    );
    let (scene, _h, pen, x) = scene_with(vec![square(10.0)]);
    assert!(
        OffsetSession::begin(&scene, &pen, &x).is_some(),
        "com seleção há sessão"
    );
}

/// **O preview ao vivo NÃO É CUMULATIVO** — a propriedade que justifica a sessão inteira. O
/// offset é destrutivo, então re-offsetar o resultado do frame anterior somaria; a sessão
/// RESTAURA a cena do grab a cada frame e offseta do original. Arrastar o slider de 2 para 3 e
/// de volta a 0 tem de dar sempre o offset ABSOLUTO, nunca a soma dos passos.
#[test]
fn the_live_offset_is_not_cumulative() {
    let (mut scene, _hist, mut pen, xf) = scene_with(vec![square(10.0)]);
    let mut sess = OffsetSession::begin(&scene, &pen, &xf).expect("há seleção");

    let area = |scene: &VecScene| ph2d_vec_boolean::area(&scene.paths()[0]);

    // d=2 ⇒ quadrado 14 ⇒ área 196.
    sess.preview(&mut scene, &mut pen, 2.0);
    assert!(
        (area(&scene) - 196.0).abs() < 1.0,
        "d=2 → 14² ({})",
        area(&scene)
    );

    // d=2 DE NOVO ⇒ ainda 196 (não 18²=324, que seria offsetar o resultado anterior).
    sess.preview(&mut scene, &mut pen, 2.0);
    assert!(
        (area(&scene) - 196.0).abs() < 1.0,
        "o preview repetido compôs — a sessão não restaurou ({})",
        area(&scene)
    );

    // d=3 ⇒ 16²=256 (do ORIGINAL de 10, não do 14 do passo anterior).
    sess.preview(&mut scene, &mut pen, 3.0);
    assert!(
        (area(&scene) - 256.0).abs() < 1.0,
        "d=3 → 16² ({})",
        area(&scene)
    );

    // d=0 ⇒ volta ao original (offset zero = cena do grab restaurada).
    sess.preview(&mut scene, &mut pen, 0.0);
    assert!(
        (area(&scene) - 100.0).abs() < 1e-6,
        "d=0 devia devolver o original de 10² ({})",
        area(&scene)
    );
    assert_eq!(scene.paths().len(), 1, "e uma forma só");
}

/// **O preview usa a MESMA porta que o botão** — o que o artista vê arrastando é byte-a-byte o
/// que o botão assaria no mesmo `d`. Os dois chamam `expand_selection`; um 2º caminho
/// divergiria no dia em que o offset ganhasse um detalhe.
#[test]
fn the_live_preview_matches_the_committed_offset() {
    // Caminho do botão.
    let (mut baked, mut hist, mut pen_b, xf) = scene_with(vec![square(10.0)]);
    apply_vec_expand(&mut baked, &mut hist, &mut pen_b, &xf, MITER, 2.5);

    // Caminho ao vivo, ao mesmo `d`.
    let (mut live, _h, mut pen_l, _x) = scene_with(vec![square(10.0)]);
    let mut sess = OffsetSession::begin(&live, &pen_l, &xf).unwrap();
    sess.preview(&mut live, &mut pen_l, 2.5);

    assert_eq!(baked.paths().len(), live.paths().len());
    let (a, b) = (
        ph2d_vec_boolean::area(&baked.paths()[0]),
        ph2d_vec_boolean::area(&live.paths()[0]),
    );
    assert!((a - b).abs() < 1e-6, "o botão deu {a}, o arrasto deu {b}");
}

// ─────────────────── O arrasto sob o FRAME real (sync + settle) ───────────────────

/// Um frame do produto durante o arrasto: preview → `sync` → `settle_origins` com o
/// `drawing` que a `render_loop` passa (os `live_paths` da sessão — o preview é GESTO).
/// Devolve o CENTRO DESENHADO do path do preview: bbox da geometria × xform vivo — o
/// que a tela mostra, não o que o documento guarda.
fn drag_frame(
    sess: &mut OffsetSession,
    scene: &mut VecScene,
    pen: &mut PenTool,
    sim: &mut ph2d_ecs::SimWorld,
    map: &mut crate::vec_entities::VecEntityMap,
    d: f64,
) -> [f64; 2] {
    sess.preview(scene, pen, d);
    crate::vec_entities::sync(sim, scene, map);
    let drawing: Vec<VecPathId> = sess.live_paths().to_vec();
    crate::vec_transform::settle_origins(sim, scene, map, &drawing);
    let live = crate::vec_transform::build(sim, map);
    let pid = scene.paths().last().expect("o preview deixou um path").id;
    let (lo, hi) = scene.path_bbox(pid).expect("bbox");
    xform_of(&live, pid).apply([(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5])
}

/// Fixture posada: quadrado LOCAL centrado (como um path já assentado) cuja entidade
/// carrega a pose `(4, 0)` — a rosquinha do smoke mora aí. Devolve tudo que o
/// [`drag_frame`] precisa.
#[allow(clippy::type_complexity)]
fn posed_scene() -> (
    VecScene,
    PenTool,
    ph2d_ecs::SimWorld,
    crate::vec_entities::VecEntityMap,
    VecXforms,
) {
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut map = crate::vec_entities::VecEntityMap::new();
    let mut scene = VecScene::new();
    let id = scene.push_path(ph2d_vec_scene::rectangle([-1.0, -1.0], [1.0, 1.0]));
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform {
                translation: ph2d_core::Vec2::new(4.0, 0.0),
                ..ph2d_ecs::Transform::IDENTITY
            },
            ph2d_ecs::Name::new("S"),
            ph2d_ecs::VecPathRef(id),
        ))
        .id();
    map.insert(id, e.to_bits());
    let mut pen = PenTool::default();
    pen.select_many(&[id]);
    let xf = crate::vec_transform::build(&sim, &map);
    (scene, pen, sim, map, xf)
}

/// **O preview vivo desenha NO MESMO LUGAR todo frame.** O `clone_from(&pre)` restaura o
/// `next_id` da cena, então o resultado renasce com o MESMO id — e a MESMA entidade — a
/// cada frame do arrasto. Se o `settle_origins` assentar essa entidade no frame 1
/// (`Transform` = centro), o frame 2 insere geometria de MUNDO sob uma entidade que já
/// carrega o centro: mundo × centro = translação DOBRADA — o *"pula para o canto direito"*
/// (Enio, 2026-07-20), e cada re-grab compunha mais um centro. O preview é um GESTO: o
/// settle o pula, e o lugar desenhado é estável.
///
/// ⚠️ Os dois frames usam `d` DIFERENTE (0.3 → 0.4) de propósito: o memo do preview
/// (`OffsetSession::last`) pula um frame de `d` IGUAL, então um teste com o mesmo `d`
/// testaria o memo, não a costura settle↔re-derive que ESTE gate guarda. O offset cresce
/// simétrico em torno do centro, então o centro fica em `(4,0)` para qualquer `d` — o que
/// muda entre os frames é o TAMANHO, e é a re-inserção sob o mesmo id (com o settle
/// pulando-a como gesto) que tem de manter o centro parado.
#[test]
fn the_live_preview_draws_in_the_same_place_every_frame() {
    let (mut scene, mut pen, mut sim, mut map, xf) = posed_scene();
    let mut sess = OffsetSession::begin(&scene, &pen, &xf).expect("há seleção");
    let c1 = drag_frame(&mut sess, &mut scene, &mut pen, &mut sim, &mut map, 0.3);
    let c2 = drag_frame(&mut sess, &mut scene, &mut pen, &mut sim, &mut map, 0.4);
    assert!(
        (c1[0] - 4.0).abs() < 1e-3 && c1[1].abs() < 1e-3,
        "frame 1 desenhou fora da pose da fonte: {c1:?}"
    );
    assert!(
        (c2[0] - 4.0).abs() < 1e-3 && c2[1].abs() < 1e-3,
        "o frame seguinte (d novo, re-derivado sob o mesmo id) desenhou em {c2:?} — o \
         settle assentou o preview e a translação dobrou"
    );
}

/// **Um frame cujo `d` e knobs não mudaram é MEMOIZADO — não re-clona a cena nem re-offseta.**
/// O preview roda todo frame enquanto o slider está agarrado, mesmo com o mouse parado; o
/// `clone_from(&pre)` custa O(cena TODA), então segurar o slider re-copiava a cena inteira e
/// re-rodava o sweep à toa (a queda de FPS ao pausar o arrasto numa cena grande, 2026-07-20).
/// O `preview` devolve `true` quando re-derivou, `false` quando pulou.
#[test]
fn an_unchanged_offset_frame_is_memoized() {
    let (mut scene, _h, mut pen, xf) = scene_with(vec![square(10.0)]);
    let mut sess = OffsetSession::begin(&scene, &pen, &xf).expect("há seleção");

    assert!(sess.preview(&mut scene, &mut pen, 2.0), "1º frame re-deriva");
    let area = ph2d_vec_boolean::area(&scene.paths()[0]);
    assert!(
        !sess.preview(&mut scene, &mut pen, 2.0),
        "o MESMO d tem de ser memoizado (pulado)"
    );
    assert!(
        (ph2d_vec_boolean::area(&scene.paths()[0]) - area).abs() < 1e-9,
        "o frame memoizado deixou a cena EXATAMENTE como estava"
    );
    assert!(
        sess.preview(&mut scene, &mut pen, 3.0),
        "um d NOVO re-deriva"
    );
    assert!(
        !sess.preview(&mut scene, &mut pen, 3.0),
        "…e o memo volta a valer no novo d"
    );
}

/// **Voltar o slider ao centro NO MEIO do arrasto mostra a forma NO LUGAR dela.** As
/// entidades das fontes morrem no 1º churn, então restaurar a fonte crua (geometria LOCAL,
/// entidade nova na IDENTIDADE) desenharia na ORIGEM — o "pula de lugar" original. Em
/// `|d| ~ 0` pós-churn a fonte vira cópia assada em MUNDO: a pose viaja na geometria.
#[test]
fn returning_to_zero_mid_drag_shows_the_shape_at_its_pose() {
    let (mut scene, mut pen, mut sim, mut map, xf) = posed_scene();
    let mut sess = OffsetSession::begin(&scene, &pen, &xf).expect("há seleção");
    drag_frame(&mut sess, &mut scene, &mut pen, &mut sim, &mut map, 0.5);
    let c = drag_frame(&mut sess, &mut scene, &mut pen, &mut sim, &mut map, 0.0);
    assert_eq!(scene.paths().len(), 1, "d=0 mostra UMA forma");
    assert!(
        (c[0] - 4.0).abs() < 1e-3 && c[1].abs() < 1e-3,
        "d=0 pós-churn desenhou em {c:?} — a fonte encalhada perdeu a pose (origem?)"
    );
}

/// **Um `d` que ANIQUILA a forma a some da cena — não a encalha.** `results` vazio num `d`
/// grande é a forma que encolheu até sumir (resposta honesta); a fonte encalhada ficaria
/// LOCAL com entidade nova na identidade e desenharia na ORIGEM. E o `d` seguinte a
/// restaura do `pre`, como qualquer frame.
#[test]
fn an_annihilating_d_removes_the_source_from_the_scene() {
    let (mut scene, _h, mut pen, xf) = scene_with(vec![square(2.0)]);
    let mut sess = OffsetSession::begin(&scene, &pen, &xf).expect("há seleção");
    sess.preview(&mut scene, &mut pen, -100.0);
    assert!(
        scene.paths().is_empty(),
        "a forma aniquilada tem de sumir DESTE frame"
    );
    assert!(sess.live_paths().is_empty(), "nada vivo");
    assert!(pen.selected_paths().is_empty(), "seleção sem id morto");
    sess.preview(&mut scene, &mut pen, 0.5);
    assert_eq!(scene.paths().len(), 1, "o d seguinte restaura do pre");
}

/// **Agarrar o slider e soltar sem mover não muda NEM UM ID.** Todo grab começa com o
/// slider recentrado (`d = 0`): antes de qualquer churn isso é restauração pura — sem esta
/// zona morta, o simples toque no slider re-identificava a forma (id/nome/entidade novos)
/// e custava um passo de undo que o artista não pediu.
#[test]
fn grabbing_without_moving_changes_nothing() {
    let (mut scene, _h, mut pen, xf) = scene_with(vec![square(3.0)]);
    let before: Vec<VecPathId> = scene.paths().iter().map(|p| p.id).collect();
    let mut sess = OffsetSession::begin(&scene, &pen, &xf).expect("há seleção");
    sess.preview(&mut scene, &mut pen, 0.0);
    let after: Vec<VecPathId> = scene.paths().iter().map(|p| p.id).collect();
    assert_eq!(before, after, "d=0 no grab tem de ser restauração pura");
    assert!(sess.live_paths().is_empty(), "nada churnou, nada vivo");
}

// ───────────────────── A janela de RETUNE (Join/Side pós-release) ─────────────────────

/// **Clicar Round DEPOIS de soltar o slider re-offseta na hora — sem mover a forma.** O
/// pedido do Enio (2026-07-20): a quina se escolhe VENDO o resultado. A janela re-roda o
/// preview ao `d` comitado; o oráculo do lugar é o mesmo do arrasto (bbox × xform vivo), e
/// o de FORMA é a área: Round come as quatro quinas do Miter (`(4−π)·d²`).
///
/// ⚠️ O reset de identidade dentro do `apply` é load-bearing: o release ASSENTOU as
/// entidades do resultado (`Transform` = centro), e o preview re-insere geometria de MUNDO
/// sob os MESMOS ids — sem o reset, o retune desenharia a forma com a pose DOBRADA (a
/// doença exata do "pula pro canto direito", `9c0446df`).
#[test]
fn changing_the_join_after_release_retunes_the_committed_offset() {
    let (mut scene, mut pen, mut sim, mut map, xf) = posed_scene();
    // O estado dos chips é thread-local — pina Miter/Both e restaura no fim (outros
    // gates deste arquivo medem áreas computadas para Miter).
    ph2d_panel_vector::set_expand_join(0);
    ph2d_panel_vector::set_expand_side(2);

    let mut sess = OffsetSession::begin(&scene, &pen, &xf).expect("há seleção");
    drag_frame(&mut sess, &mut scene, &mut pen, &mut sim, &mut map, 0.5);
    // O release: preview final; a sessão morre e o settle ASSENTA o resultado.
    sess.preview(&mut scene, &mut pen, 0.5);
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    crate::vec_transform::settle_origins(&mut sim, &mut scene, &map, &[]);
    let miter_area = ph2d_vec_boolean::area(&scene.paths()[0]);

    let mut win = OffsetRetune::after_release(sess, 0.5).expect("churnou — há janela");
    assert_eq!(
        win.step(7, expand_knobs()),
        RetuneStep::Keep,
        "frame 1 aprende"
    );

    // O artista clica Round:
    ph2d_panel_vector::set_expand_join(1);
    let knobs = expand_knobs();
    assert_eq!(
        win.step(7, knobs),
        RetuneStep::Retune,
        "knob mudou = retune"
    );
    win.apply(&mut scene, &mut pen, &mut sim, &map, knobs);
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    crate::vec_transform::settle_origins(&mut sim, &mut scene, &map, &[]);

    let round_area = ph2d_vec_boolean::area(&scene.paths()[0]);
    assert!(
        round_area < miter_area - 0.05,
        "a quina não mudou (miter {miter_area}, round {round_area})"
    );
    let live = crate::vec_transform::build(&sim, &map);
    let pid = scene.paths()[0].id;
    let (lo, hi) = scene.path_bbox(pid).expect("bbox");
    let c = xform_of(&live, pid).apply([(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5]);
    assert!(
        (c[0] - 4.0).abs() < 1e-3 && c[1].abs() < 1e-3,
        "o retune moveu a forma (pose dobrada?): {c:?}"
    );
    ph2d_panel_vector::set_expand_join(0);
}

/// **A janela morre quando o undo ANDA — pra qualquer lado.** Depois do nosso passo,
/// qualquer profundidade diferente significa outra edição (mover, apagar) ou um Ctrl+Z:
/// retunar por cima restauraria a cena do grab e engoliria o que o artista fez.
#[test]
fn the_retune_window_dies_when_the_undo_moves() {
    let make = || {
        let (mut scene, _h, mut pen, xf) = scene_with(vec![square(2.0)]);
        let mut sess = OffsetSession::begin(&scene, &pen, &xf).expect("há seleção");
        sess.preview(&mut scene, &mut pen, 0.5);
        OffsetRetune::after_release(sess, 0.5).expect("churnou")
    };
    let k = expand_knobs();

    let mut win = make();
    assert_eq!(
        win.step(7, k),
        RetuneStep::Keep,
        "frame 1 aprende a profundidade"
    );
    assert_eq!(
        win.step(7, k),
        RetuneStep::Keep,
        "nada mudou = nada a fazer"
    );
    assert_eq!(
        win.step(8, k),
        RetuneStep::Dead,
        "edição alheia fecha a janela"
    );

    let mut win = make();
    assert_eq!(win.step(7, k), RetuneStep::Keep);
    assert_eq!(win.step(6, k), RetuneStep::Dead, "Ctrl+Z também fecha");
}

/// **Um arrasto que nunca churnou não abre janela** — não há resultado a retunar, e uma
/// janela sobre nada faria um clique perdido num chip re-rodar a cena inteira.
#[test]
fn a_drag_that_never_churned_opens_no_retune_window() {
    let (mut scene, _h, mut pen, xf) = scene_with(vec![square(2.0)]);
    let mut sess = OffsetSession::begin(&scene, &pen, &xf).expect("há seleção");
    sess.preview(&mut scene, &mut pen, 0.0);
    assert!(
        OffsetRetune::after_release(sess, 0.0).is_none(),
        "grab-e-solta sem mover não deixa janela armada"
    );
}
