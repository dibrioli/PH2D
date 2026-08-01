//! Os gates do alinhamento VIVO.
//!
//! O kernel (`ph2d_vec_boolean::align`) já prova ONDE a tinta cai. O que só se pode afirmar aqui
//! é a costura: que o alinhamento **compõe** com a derivada de outro produtor em vez de a apagar,
//! e que quem não o usa fica **byte-intocado**.

use super::*;
use ph2d_vec_scene::{Paint, Rgba8, StrokeAlign, StrokeSpec, VecVertex};

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex::corner([x, y])
}

fn square(align: StrokeAlign) -> VecPath {
    VecPath {
        verts: vec![v(-2.0, -2.0), v(2.0, -2.0), v(2.0, 2.0), v(-2.0, 2.0)],
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(200, 200, 200, 255))),
        stroke: Some(StrokeSpec {
            align,
            ..StrokeSpec::new(Rgba8::new(10, 20, 30, 255), 0.4)
        }),
        ..VecPath::default()
    }
}

/// **Uma forma centrada não entra no mapa** — e é isso que torna o mundo de quem nunca escolheu
/// Inner/Outer byte-idêntico ao de antes desta feature.
///
/// ⚠️ *Ausente* e *presente-com-a-fonte* não são a mesma coisa: uma entrada, mesmo igual, faz o
/// `dispatch` desenhar pela rota da derivada (que já tem a pose ASSADA) em vez da rota da fonte
/// (que a aplica pela câmera) — e aplicar a pose duas vezes foi bug real desta linha.
#[test]
fn a_centred_stroke_never_enters_the_live_map() {
    let mut scene = VecScene::default();
    scene.push_path(square(StrokeAlign::Centre));
    let mut live = LiveGeometry::new();
    AlignLive::default().recook(&scene, &VecXforms::default(), &mut live);
    assert!(
        live.is_empty(),
        "o mapa nasceu com {} entrada(s)",
        live.len()
    );
}

/// **Um traço alinhado vira PREENCHIMENTO + FAIXA.**
#[test]
fn an_aligned_stroke_becomes_a_fill_and_a_band() {
    let mut scene = VecScene::default();
    let id = scene.push_path(square(StrokeAlign::Inner));
    let mut live = LiveGeometry::new();
    AlignLive::default().recook(&scene, &VecXforms::default(), &mut live);

    let items = live.get(&id).expect("a forma alinhada entra no mapa");
    assert!(items.len() >= 2, "saiu com {} item(ns)", items.len());
    assert!(
        items.iter().all(|p| p.stroke.is_none()),
        "nenhum item pode sair com traço: a faixa É a tinta"
    );
    // O 1º item carrega o preenchimento da forma — descartá-lo esvaziaria o miolo.
    assert_eq!(
        items[0].fill,
        Some(Paint::Solid(Rgba8::new(200, 200, 200, 255)))
    );
    // …e as peças da faixa vestem a cor do TRAÇO.
    assert!(
        items[1..]
            .iter()
            .all(|p| p.fill == Some(Paint::Solid(Rgba8::new(10, 20, 30, 255)))),
        "a faixa tem de vestir a cor do traço"
    );
}

/// **O alinhamento COMPÕE com a derivada de outro produtor em vez de a apagar.**
///
/// ⚠️ É o gate que justifica este módulo existir separado. Os cinco produtores irmãos são
/// mutuamente exclusivos e por isso o `render_loop` os funde com `extend`; o alinhamento **não é**
/// — ele é um campo do `StrokeSpec` e convive com um offset vivo. Fundido por `extend` ele
/// apagaria o offset em silêncio, e a forma perderia metade do que o artista pediu.
///
/// A fixture põe no mapa uma geometria que a FONTE não tem (um triângulo onde a cena guarda um
/// quadrado): se o alinhamento re-derivasse da fonte, o resultado teria 4 vértices.
#[test]
fn the_alignment_composes_with_another_producers_geometry() {
    let mut scene = VecScene::default();
    let id = scene.push_path(square(StrokeAlign::Inner));

    // "Já derivado por outro produtor": um TRIÂNGULO, ainda traçado com o mesmo alinhamento.
    let derived = VecPath {
        verts: vec![v(-3.0, -3.0), v(3.0, -3.0), v(0.0, 3.0)],
        ..square(StrokeAlign::Inner)
    };
    let mut live = LiveGeometry::new();
    live.insert(id, vec![derived]);
    AlignLive::default().recook(&scene, &VecXforms::default(), &mut live);

    let items = live.get(&id).expect("a entrada continua no mapa");
    assert_eq!(
        items[0].verts.len(),
        3,
        "o alinhamento re-derivou da FONTE e apagou a geometria do outro produtor"
    );
    assert!(items.len() >= 2, "e a faixa tem de ter sido acrescentada");
}

/// **Uma linha ABERTA passa verbatim** — sem interior não há dentro nem fora, e inventar uma
/// faixa ali seria a UI decidindo o que a geometria não diz.
#[test]
fn an_open_path_passes_through_untouched() {
    let mut scene = VecScene::default();
    let mut open = square(StrokeAlign::Outer);
    open.closed = false;
    scene.push_path(open);
    let mut live = LiveGeometry::new();
    AlignLive::default().recook(&scene, &VecXforms::default(), &mut live);
    assert!(
        live.is_empty(),
        "um caminho aberto não tem alinhamento a executar"
    );
}

/// **O memo não sobrevive a voltar para Centre.**
///
/// ⚠️ Sem o `retain` a resposta velha ficaria guardada e seria re-servida se a forma voltasse a
/// Inner com OUTRA geometria — a falha é silenciosa e desenha a forma de antes.
#[test]
fn the_memo_forgets_a_shape_that_went_back_to_centre() {
    let mut scene = VecScene::default();
    let id = scene.push_path(square(StrokeAlign::Inner));
    let mut al = AlignLive::default();
    let mut live = LiveGeometry::new();
    al.recook(&scene, &VecXforms::default(), &mut live);
    assert_eq!(al.memo.len(), 1);

    if let Some(p) = scene.path_mut(id)
        && let Some(s) = p.stroke.as_mut()
    {
        s.align = StrokeAlign::Centre;
    }
    let mut live2 = LiveGeometry::new();
    al.recook(&scene, &VecXforms::default(), &mut live2);
    assert!(
        al.memo.is_empty(),
        "o memo guardou uma forma que não alinha"
    );
    assert!(live2.is_empty());
}
