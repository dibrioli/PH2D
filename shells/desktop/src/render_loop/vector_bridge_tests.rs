//! **Os gates do restyle do traço** — o seam onde o Style da tool vira `StrokeSpec` no
//! documento.
//!
//! Os dois fatos que precisam morder, e que um teste de unidade da tool sozinha não dá:
//!
//! 1. **O Head Size / o Head Round chegam ao `StrokeSpec` de TODOS os selecionados.** É o
//!    ponto da feature (calibrar o diagrama inteiro de uma vez); um `for` sobre o primeiro da
//!    seleção seria uma UI que promete e não cumpre.
//! 2. **Quem DETECTA a mudança e quem a GRAVA falam da mesma ficha.** `differs_from` é o
//!    portão do `will_change`: um campo ausente dele faz o painel mexer no número e **nada**
//!    mudar na tela (nem `RECOLOR_PRE`, logo nem undo) — e o compilador fica calado.

use super::{StrokeStyle, restyle_selected_strokes};
use ph2d_vec_scene::{
    LineCap, LineJoin, Marker, Rgba8, StrokeSpec, VecPath, VecPathId, VecScene, VecVertex,
};

/// A cor do traço de teste (a mesma no `StrokeSpec` e na ficha — senão todo caso "sem
/// mudança" já nasceria diferente).
const INK: Rgba8 = Rgba8::new(240, 240, 245, 255);
/// A largura do traço de teste, em mundo.
const W: f64 = 0.05;

/// A ficha default (sem ponta), o ponto de partida de cada caso.
fn style() -> StrokeStyle {
    StrokeStyle {
        color: INK,
        cap: LineCap::Butt,
        join: LineJoin::Miter,
        dash: None,
        marker_start: Marker::None,
        marker_end: Marker::None,
        marker_scale: 1.0,
        marker_round: 0.0,
    }
}

/// O `StrokeSpec` que a ficha default produz (o construtor da crate de geometria é a fonte:
/// escrever o literal aqui seria uma segunda verdade sobre o que é um traço default).
fn spec() -> StrokeSpec {
    StrokeSpec::new(INK, W)
}

/// Um caminho traçado (dois pontos), pronto para entrar na cena.
fn stroked_path() -> VecPath {
    VecPath {
        verts: vec![VecVertex::corner([0.0, 0.0]), VecVertex::corner([1.0, 0.0])],
        stroke: Some(spec()),
        ..VecPath::default()
    }
}

/// Uma cena com `n` caminhos traçados, todos idênticos (o `push_path` atribui os ids).
fn scene_with(n: usize) -> (VecScene, Vec<VecPathId>) {
    let mut scene = VecScene::new();
    let ids = (0..n).map(|_| scene.push_path(stroked_path())).collect();
    (scene, ids)
}

/// **O Head Size e o Head Round chegam a TODOS os selecionados** — não só ao primeiro.
///
/// Este é o gate do pedido do Enio: selecionar o diagrama e afinar a cabeça das setas de uma
/// vez. Um `for` que parasse no primeiro passaria num teste de "o valor chegou"; só a
/// varredura de TODA a seleção o derruba.
#[test]
fn head_size_and_round_reach_every_selected_path() {
    let (mut scene, ids) = scene_with(3);
    let s = StrokeStyle {
        marker_end: Marker::Triangle,
        marker_scale: 2.5,
        marker_round: 0.75,
        ..style()
    };
    let n = restyle_selected_strokes(&mut scene, &ids, &s, None);
    assert_eq!(n, 3, "os tres tracos selecionados tinham de ser reescritos");
    for id in &ids {
        let spec = scene
            .paths()
            .iter()
            .find(|p| p.id == *id)
            .and_then(|p| p.stroke)
            .expect("o caminho tem traco");
        assert!(
            (spec.marker_scale - 2.5).abs() < 1e-12,
            "o Head Size nao chegou ao caminho {id:?}: {}",
            spec.marker_scale
        );
        assert!(
            (spec.marker_round - 0.75).abs() < 1e-12,
            "o Head Round nao chegou ao caminho {id:?}: {}",
            spec.marker_round
        );
        assert_eq!(spec.marker_end, Marker::Triangle);
    }
}

/// A largura **não** acompanha a tool fora do arrasto do slider: uma escolha de cor (ou de
/// Head Size) não pode reengrossar a linha. Com `Some(w)`, acompanha.
#[test]
fn the_width_only_follows_the_tool_while_the_slider_is_dragged() {
    let (mut scene, ids) = scene_with(1);

    restyle_selected_strokes(&mut scene, &ids, &style(), None);
    let kept = scene.paths()[0].stroke.expect("traco").width;
    assert!(
        (kept - W).abs() < f64::EPSILON,
        "sem arrasto do slider a largura tem de ser PRESERVADA: {kept} != {W}"
    );

    restyle_selected_strokes(&mut scene, &ids, &style(), Some(0.5));
    let dragged = scene.paths()[0].stroke.expect("traco").width;
    assert!(
        (dragged - 0.5).abs() < f64::EPSILON,
        "arrastando, ela segue"
    );
}

/// **O detector e o gravador falam da MESMA ficha.** `differs_from` é o portão do
/// `will_change`: se o Head Size (ou o Head Round) não estivesse nele, o `restyle` nem
/// rodaria — o número mudaria no painel e a seta continuaria igual na tela, sem sequer um
/// passo de undo. O teste percorre CADA campo da ficha, então um campo novo que alguém
/// esqueça no `differs_from` cai aqui.
#[test]
fn every_field_of_the_style_is_seen_by_the_change_detector() {
    let base = style();
    let spec = spec();
    assert!(
        !base.differs_from(&spec),
        "a ficha default e o traco default TEM de ser a mesma coisa (senao todo frame reescreve)"
    );
    let mutations = [
        (
            "color",
            StrokeStyle {
                color: Rgba8::new(1, 2, 3, 255),
                ..base
            },
        ),
        (
            "cap",
            StrokeStyle {
                cap: LineCap::Round,
                ..base
            },
        ),
        (
            "join",
            StrokeStyle {
                join: LineJoin::Bevel,
                ..base
            },
        ),
        (
            "dash",
            StrokeStyle {
                dash: Some((2.0, 1.0)),
                ..base
            },
        ),
        (
            "marker_start",
            StrokeStyle {
                marker_start: Marker::Triangle,
                ..base
            },
        ),
        (
            "marker_end",
            StrokeStyle {
                marker_end: Marker::Triangle,
                ..base
            },
        ),
        (
            "marker_scale",
            StrokeStyle {
                marker_scale: 2.0,
                ..base
            },
        ),
        (
            "marker_round",
            StrokeStyle {
                marker_round: 1.0,
                ..base
            },
        ),
    ];
    for (field, m) in mutations {
        assert!(
            m.differs_from(&spec),
            "mudar `{field}` na ficha NAO foi visto pelo detector — o painel mexeria no \
             controle e nada mudaria na tela (nem undo)"
        );
        // E o gravador realmente escreve o campo (o outro lado do mesmo pacto).
        let written = m.onto(spec.width);
        assert!(
            m.differs_from(&spec) && !m.differs_from(&written),
            "`{field}` foi detectado mas NAO foi gravado pelo `onto`"
        );
    }
}

/// Um caminho **sem traço** (só preenchimento) não ganha um do nada: a UI não inventa
/// geometria, e o contador não o conta.
#[test]
fn a_path_without_a_stroke_is_left_alone() {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: vec![VecVertex::corner([0.0, 0.0])],
        stroke: None,
        ..VecPath::default()
    });
    let n = restyle_selected_strokes(&mut scene, &[id], &style(), None);
    assert_eq!(n, 0);
    assert!(scene.paths()[0].stroke.is_none());
}
