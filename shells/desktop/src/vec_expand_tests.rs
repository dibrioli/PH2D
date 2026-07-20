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
        OffsetSession::begin(&scene, &empty).is_none(),
        "sem seleção não há sessão"
    );
    let (scene, _h, pen, _x) = scene_with(vec![square(10.0)]);
    assert!(
        OffsetSession::begin(&scene, &pen).is_some(),
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
    let sess = OffsetSession::begin(&scene, &pen).expect("há seleção");

    let area = |scene: &VecScene| ph2d_vec_boolean::area(&scene.paths()[0]);

    // d=2 ⇒ quadrado 14 ⇒ área 196.
    sess.preview(&mut scene, &mut pen, &xf, 2.0);
    assert!((area(&scene) - 196.0).abs() < 1.0, "d=2 → 14² ({})", area(&scene));

    // d=2 DE NOVO ⇒ ainda 196 (não 18²=324, que seria offsetar o resultado anterior).
    sess.preview(&mut scene, &mut pen, &xf, 2.0);
    assert!(
        (area(&scene) - 196.0).abs() < 1.0,
        "o preview repetido compôs — a sessão não restaurou ({})",
        area(&scene)
    );

    // d=3 ⇒ 16²=256 (do ORIGINAL de 10, não do 14 do passo anterior).
    sess.preview(&mut scene, &mut pen, &xf, 3.0);
    assert!((area(&scene) - 256.0).abs() < 1.0, "d=3 → 16² ({})", area(&scene));

    // d=0 ⇒ volta ao original (offset zero = cena do grab restaurada).
    sess.preview(&mut scene, &mut pen, &xf, 0.0);
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
    let sess = OffsetSession::begin(&live, &pen_l).unwrap();
    sess.preview(&mut live, &mut pen_l, &xf, 2.5);

    assert_eq!(baked.paths().len(), live.paths().len());
    let (a, b) = (
        ph2d_vec_boolean::area(&baked.paths()[0]),
        ph2d_vec_boolean::area(&live.paths()[0]),
    );
    assert!((a - b).abs() < 1e-6, "o botão deu {a}, o arrasto deu {b}");
}
