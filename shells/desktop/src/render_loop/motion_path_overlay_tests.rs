//! **O que o artista VÊ da trajetória** (ADR-0141, Fatia 3).
//!
//! Dirigem a geometria pura, não a chamada de pintura: *"esses pontos estão de fato
//! espaçados como a velocidade do objeto, em pixels de tela, nesta câmera"* é
//! respondível headless — e a resposta é o que se vê.

use super::marks;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_timeline::{MotionPath, PathAnchor, PropKind, TimelineDoc};
use ph2d_vector::{BezPath, PathEl};

fn window() -> WindowSize {
    WindowSize {
        width: 1000,
        height: 1000,
    }
}

fn camera() -> Camera2d {
    Camera2d {
        center: [10.0, 0.0],
        height_world: 40.0,
        ..Camera2d::default()
    }
}

/// Todo ponto que um caminho visita, em px de tela.
fn points(path: &BezPath) -> Vec<(f64, f64)> {
    path.elements()
        .iter()
        .filter_map(|el| match el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => Some((p.x, p.y)),
            _ => None,
        })
        .collect()
}

/// Uma reta de 20 unidades, com `keys` `(tempo, distância, interp)` — o documento e a
/// entidade que o desenho fala sobre.
fn doc_with(keys: &[(f64, f32, Interp)]) -> (TimelineDoc, u64) {
    let entity = 42_u64;
    let mut doc = TimelineDoc::new();
    doc.bind(entity, PropKind::Position);
    for &(t, s, interp) in keys {
        doc.insert_key(
            entity,
            PropKind::Position,
            RationalTime::from_seconds(t),
            AnimValue::Float(s),
            interp,
        );
    }
    doc.bindings_mut()[0].path = Some(MotionPath::new(vec![
        PathAnchor::corner([0.0, 0.0]),
        PathAnchor::corner([20.0, 0.0]),
    ]));
    (doc, entity)
}

/// **O espaçamento dos pontos É a velocidade** — a coisa inteira que o desenho existe
/// para dizer, e a razão de ser pontos em vez de uma linha.
///
/// Um ease-in-out cobre a mesma distância acelerando e freando; os pontos das pontas
/// têm de ficar JUNTOS e os do meio ESPARRAMADOS. Um oráculo que só olhasse "os pontos
/// estão sobre a curva" ficaria verde sobre um desenho que não diz nada sobre timing.
#[test]
fn the_spacing_of_the_dots_is_the_speed() {
    let (doc, e) = doc_with(&[
        (
            0.0,
            0.0,
            Interp::Bezier {
                x1: 0.8,
                y1: 0.0,
                x2: 0.2,
                y2: 1.0,
            },
        ),
        (2.0, 20.0, Interp::Linear),
    ]);
    let out = marks(true, &doc, Some(e), &camera(), window());
    // O grupo dos PONTOS: um losango por quadro, 4 vértices cada.
    let dots = &out[1].0;
    let vs = points(dots);
    assert!(vs.len() >= 4 * 24, "poucos pontos: {}", vs.len());

    // O centro de cada losango é o 1º vértice deslocado — basta a coordenada x do
    // vértice de cima, que é o centro em x.
    let centres: Vec<f64> = vs.chunks(4).map(|c| c[0].0).collect();
    let gaps: Vec<f64> = centres.windows(2).map(|w| (w[1] - w[0]).abs()).collect();
    let n = gaps.len();
    let edge = (gaps[0] + gaps[n - 1]) / 2.0;
    let middle = gaps[n / 2];
    println!("MEDIDO  vao nas pontas {edge:.2} px, no meio {middle:.2} px");
    assert!(
        middle > 3.0 * edge,
        "os pontos do meio ({middle:.2} px) não estão mais esparramados que os das \
         pontas ({edge:.2} px) — o desenho não está dizendo nada sobre velocidade"
    );
}

/// A velocidade CONSTANTE é o controle do gate acima: com uma track linear os vãos
/// têm de ficar todos iguais, senão "esparramado no meio" seria uma propriedade do
/// desenho e não do timing.
#[test]
fn a_constant_speed_track_draws_evenly_spaced_dots() {
    let (doc, e) = doc_with(&[(0.0, 0.0, Interp::Linear), (2.0, 20.0, Interp::Linear)]);
    let out = marks(true, &doc, Some(e), &camera(), window());
    let centres: Vec<f64> = points(&out[1].0).chunks(4).map(|c| c[0].0).collect();
    let gaps: Vec<f64> = centres.windows(2).map(|w| (w[1] - w[0]).abs()).collect();
    let max = gaps.iter().copied().fold(0.0f64, f64::max);
    let min = gaps.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(
        max / min < 1.05,
        "vãos variaram {:.3}x numa track de velocidade constante",
        max / min
    );
}

/// **Espaço de TELA.** Os pontos sobem pelo afim da câmera, e o tamanho de uma marca
/// NÃO — uma âncora tem o mesmo alvo de mouse em qualquer zoom. É a lei que o realce
/// do Flip pagou para aprender.
#[test]
fn the_marks_keep_their_screen_size_at_any_zoom() {
    let (doc, e) = doc_with(&[(0.0, 0.0, Interp::Linear), (2.0, 20.0, Interp::Linear)]);
    let side = |cam: &Camera2d| -> f64 {
        let out = marks(true, &doc, Some(e), cam, window());
        let vs = points(&out[2].0); // as âncoras
        (vs[1].0 - vs[0].0).abs()
    };
    let close = Camera2d {
        height_world: 4.0,
        ..camera()
    };
    let far = Camera2d {
        height_world: 400.0,
        ..camera()
    };
    let (a, b) = (side(&close), side(&far));
    assert!(
        (a - b).abs() < 1e-9,
        "a âncora mediu {a:.3} px de perto e {b:.3} px de longe: a geometria está \
         subindo pelo afim junto com os pontos"
    );
    // ...e o CONTROLE: a câmera de fato mudou alguma coisa, senão o gate acima
    // compararia dois desenhos idênticos.
    let (p_close, p_far) = (
        points(&marks(true, &doc, Some(e), &close, window())[2].0)[0].0,
        points(&marks(true, &doc, Some(e), &far, window())[2].0)[0].0,
    );
    assert!(
        (p_close - p_far).abs() > 1.0,
        "a âncora caiu no mesmo px de tela nas duas câmeras — o zoom não mudou nada"
    );
}

/// Sem seleção, sem binding Position, sem caminho e sem keys: **nada é desenhado**.
/// Uma trajetória inventada é pior que trajetória nenhuma — ela descreve um movimento
/// que o documento não tem.
#[test]
fn nothing_is_drawn_when_there_is_no_trajectory_to_show() {
    let (doc, e) = doc_with(&[(0.0, 0.0, Interp::Linear), (2.0, 20.0, Interp::Linear)]);
    assert!(
        marks(false, &doc, Some(e), &camera(), window()).is_empty(),
        "desligado"
    );
    assert!(
        marks(true, &doc, None, &camera(), window()).is_empty(),
        "sem seleção"
    );
    assert!(
        marks(true, &doc, Some(e + 1), &camera(), window()).is_empty(),
        "outro objeto selecionado"
    );

    let (mut no_path, e2) = doc_with(&[(0.0, 0.0, Interp::Linear), (2.0, 20.0, Interp::Linear)]);
    no_path.bindings_mut()[0].path = None;
    assert!(
        marks(true, &no_path, Some(e2), &camera(), window()).is_empty(),
        "binding sem caminho"
    );

    let (empty, e3) = doc_with(&[]);
    assert!(
        marks(true, &empty, Some(e3), &camera(), window()).is_empty(),
        "caminho sem key nenhuma: não há percurso, só geometria"
    );
}

/// O fio e as marcas são grupos SEPARADOS, porque são pintados com espessuras
/// diferentes — e é isso que impede a forma de competir com a leitura de tempo.
#[test]
fn the_thread_the_dots_and_the_anchors_are_three_groups() {
    let (doc, e) = doc_with(&[(0.0, 0.0, Interp::Linear), (2.0, 20.0, Interp::Linear)]);
    let out = marks(true, &doc, Some(e), &camera(), window());
    assert_eq!(out.len(), 3, "fio, pontos, âncoras");
    assert!(
        out[0].1[3] < out[1].1[3],
        "o fio é mais fraco que os pontos"
    );
    assert_eq!(out[1].1, out[2].1, "pontos e âncoras na mesma âmbar");
    // Duas âncoras, quatro vértices cada.
    assert_eq!(points(&out[2].0).len(), 8);
}

/// **A porta única, provada onde ela importa:** apertar no centro de cada quadrado
/// DESENHADO agarra exatamente aquela âncora.
///
/// Este é o gate que uma segunda derivação quebra. O modo de falha dela não é um
/// pânico nem um número errado — é a alça pintada num sítio e agarrada noutro, o dedo
/// errando, e o artista concluindo que a feature está quebrada.
#[test]
fn pressing_the_drawn_square_grabs_that_very_anchor() {
    let (doc, e) = doc_with(&[(0.0, 0.0, Interp::Linear), (2.0, 20.0, Interp::Linear)]);
    let (cam, win) = (camera(), window());
    let squares = points(&marks(true, &doc, Some(e), &cam, win)[2].0);
    // Quatro vértices por quadrado; o centro é a média do 1º e do 3º (cantos opostos).
    for (i, c) in squares.chunks(4).enumerate() {
        let (cx, cy) = ((c[0].0 + c[2].0) / 2.0, (c[0].1 + c[2].1) / 2.0);
        let hit = super::anchor_at(&doc, Some(e), &cam, win, cx as f32, cy as f32);
        assert_eq!(
            hit.map(|(_, k)| k),
            Some(i),
            "o quadrado {i} está desenhado em ({cx:.1}, {cy:.1}) e o hit-test não o \
             encontra ali — desenho e pega discordam sobre onde a âncora está"
        );
    }
}

/// O alvo tem um raio, e fora dele o clique **passa** — senão a trajetória roubaria
/// cliques do gizmo em toda a tela.
#[test]
fn the_grab_has_a_radius_and_a_far_click_passes_through() {
    let (doc, e) = doc_with(&[(0.0, 0.0, Interp::Linear), (2.0, 20.0, Interp::Linear)]);
    let (cam, win) = (camera(), window());
    let c = points(&marks(true, &doc, Some(e), &cam, win)[2].0);
    let (cx, cy) = ((c[0].0 + c[2].0) / 2.0, (c[0].1 + c[2].1) / 2.0);

    assert!(super::anchor_at(&doc, Some(e), &cam, win, cx as f32, (cy + 3.0) as f32).is_some());
    assert!(
        super::anchor_at(&doc, Some(e), &cam, win, cx as f32, (cy + 40.0) as f32).is_none(),
        "um clique a 40 px da âncora foi agarrado — a trajetória está roubando o canvas"
    );
    // E sem seleção não há o que pegar, do mesmo jeito que não há o que desenhar.
    assert!(super::anchor_at(&doc, None, &cam, win, cx as f32, cy as f32).is_none());
}

/// Com duas âncoras dentro do alvo vence a **mais próxima**, nunca a primeira da lista:
/// "a primeira" faria o dedo pegar a de trás sem nada na tela explicando por quê.
#[test]
fn the_nearest_anchor_wins_not_the_first_one_listed() {
    let (mut doc, e) = doc_with(&[(0.0, 0.0, Interp::Linear), (2.0, 20.0, Interp::Linear)]);
    // Uma câmera bem afastada põe as duas âncoras a poucos px uma da outra.
    let cam = Camera2d {
        height_world: 4000.0,
        ..camera()
    };
    let win = window();
    let pts = super::anchor_screen(&doc, Some(e), &cam, win);
    let (a, b) = (pts[0].2, pts[1].2);
    assert!(
        (a.x - b.x).abs() < 2.0 * super::HIT_R_PX,
        "a fixture não contém o fenômeno: as âncoras estão a {:.1} px, longe demais para \
         as duas caberem no alvo",
        (a.x - b.x).abs()
    );
    // Um clique colado na SEGUNDA tem de dar a segunda.
    let hit = super::anchor_at(&doc, Some(e), &cam, win, b.x as f32, b.y as f32);
    assert_eq!(hit.map(|(_, i)| i), Some(1));
    let _ = &mut doc;
}
