//! **O que o artista VÊ da trajetória** (ADR-0141, Fatia 3).
//!
//! Dirigem a geometria pura, não a chamada de pintura: *"esses pontos estão de fato
//! espaçados como a velocidade do objeto, em pixels de tela, nesta câmera"* é
//! respondível headless — e a resposta é o que se vê.

use super::{MarkRole, MotionPathGrab, OverlayMark, marks};
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

/// O primeiro grupo de um dado PAPEL — os grupos se acham por função, não por posição
/// (a fita virou N bandas, então índice fixo mentiria).
fn only(out: &[OverlayMark], role: MarkRole) -> &OverlayMark {
    out.iter()
        .find(|m| m.role == role)
        .unwrap_or_else(|| panic!("nenhum grupo de papel {role:?}"))
}

/// Todos os grupos de um papel (a fita tem vários — um por banda de cor).
fn all(out: &[OverlayMark], role: MarkRole) -> Vec<&OverlayMark> {
    out.iter().filter(|m| m.role == role).collect()
}

/// O índice de âncora que um grab nomeia, se for uma âncora (não uma alça).
fn anchor_of(g: Option<MotionPathGrab>) -> Option<usize> {
    match g {
        Some(MotionPathGrab::Anchor { i, .. }) => Some(i),
        _ => None,
    }
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
    let dots = &only(&out, MarkRole::Dot).path;
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
    let centres: Vec<f64> = points(&only(&out, MarkRole::Dot).path)
        .chunks(4)
        .map(|c| c[0].0)
        .collect();
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
        let vs = points(&only(&out, MarkRole::Anchor).path); // as âncoras
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
    let anchor_x = |cam: &Camera2d| {
        points(&only(&marks(true, &doc, Some(e), cam, window()), MarkRole::Anchor).path)[0].0
    };
    let (p_close, p_far) = (anchor_x(&close), anchor_x(&far));
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

/// A fita, os pontos e as âncoras são grupos SEPARADOS, por papel. O GLOW é o mais
/// fraco (halo, nunca disputa), e pontos e âncoras compartilham a âmbar da identidade.
#[test]
fn the_overlay_groups_are_tagged_by_role() {
    let (doc, e) = doc_with(&[(0.0, 0.0, Interp::Linear), (2.0, 20.0, Interp::Linear)]);
    let out = marks(true, &doc, Some(e), &camera(), window());
    // O glow, a fita, os pontos e as âncoras existem cada um por seu papel.
    let glow = only(&out, MarkRole::Glow);
    assert!(
        !all(&out, MarkRole::Ribbon).is_empty(),
        "a fita de velocidade não foi desenhada"
    );
    let dots = only(&out, MarkRole::Dot);
    let anchors = only(&out, MarkRole::Anchor);
    assert!(
        glow.rgba[3] < dots.rgba[3],
        "o glow ({}) não é mais fraco que os pontos ({}) — ele é halo, não leitura",
        glow.rgba[3],
        dots.rgba[3]
    );
    assert_eq!(dots.rgba, anchors.rgba, "pontos e âncoras na mesma âmbar");
    // Duas âncoras, quatro vértices cada.
    assert_eq!(points(&anchors.path).len(), 8);
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
    let squares = points(&only(&marks(true, &doc, Some(e), &cam, win), MarkRole::Anchor).path);
    // Quatro vértices por quadrado; o centro é a média do 1º e do 3º (cantos opostos).
    for (i, c) in squares.chunks(4).enumerate() {
        let (cx, cy) = ((c[0].0 + c[2].0) / 2.0, (c[0].1 + c[2].1) / 2.0);
        let hit = super::motion_path_hit(&doc, Some(e), &cam, win, cx as f32, cy as f32);
        assert_eq!(
            anchor_of(hit),
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
    let c = points(&only(&marks(true, &doc, Some(e), &cam, win), MarkRole::Anchor).path);
    let (cx, cy) = ((c[0].0 + c[2].0) / 2.0, (c[0].1 + c[2].1) / 2.0);

    assert!(
        super::motion_path_hit(&doc, Some(e), &cam, win, cx as f32, (cy + 3.0) as f32).is_some()
    );
    assert!(
        super::motion_path_hit(&doc, Some(e), &cam, win, cx as f32, (cy + 40.0) as f32).is_none(),
        "um clique a 40 px da âncora foi agarrado — a trajetória está roubando o canvas"
    );
    // E sem seleção não há o que pegar, do mesmo jeito que não há o que desenhar.
    assert!(super::motion_path_hit(&doc, None, &cam, win, cx as f32, cy as f32).is_none());
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
    let hit = super::motion_path_hit(&doc, Some(e), &cam, win, b.x as f32, b.y as f32);
    assert_eq!(anchor_of(hit), Some(1));
    let _ = &mut doc;
}

/// Um caminho MOLDADO: a âncora do meio tem alças de tangente longas, para que elas
/// sejam desenhadas e agarráveis (uma quina de handles zero não tem o que puxar).
fn doc_shaped() -> (TimelineDoc, u64) {
    let entity = 55_u64;
    let mut doc = TimelineDoc::new();
    let target = doc.bind(entity, PropKind::Position);
    for (t, p) in [
        (0.0_f64, [0.0_f32, 0.0]),
        (1.0, [10.0, 0.0]),
        (2.0, [20.0, 0.0]),
    ] {
        doc.add_path_key(target, RationalTime::from_seconds(t), p);
    }
    // Molda a do meio: alças bem longas em px de tela (câmera de 40 u de altura ⇒ 25
    // px/u, então 4 u = 100 px, muito além do TANGENT_MIN_PX de 14).
    doc.move_path_tangent(target, 1, true, [10.0 + 4.0, 4.0]);
    (doc, entity)
}

/// **A alça de tangente é DESENHADA e a ponta DESENHADA é agarrável** — o gesto que o
/// smoke pediu (*"ainda não temos alças de manipulação"*).
///
/// Prova a porta única do jeito que importa: apertar no centro do círculo que o desenho
/// pinta agarra AQUELA alça de tangente, não a âncora nem nada. Uma segunda derivação
/// pintaria a ponta num sítio e a agarraria noutro.
#[test]
fn a_shaped_anchor_draws_a_grabbable_tangent_handle() {
    let (doc, e) = doc_shaped();
    let (cam, win) = (camera(), window());
    let groups = marks(true, &doc, Some(e), &cam, win);
    // Com alças, os papéis de linha e de ponta de tangente aparecem.
    assert!(
        !all(&groups, MarkRole::TangentLine).is_empty(),
        "a âncora moldada não desenhou as linhas de tangente"
    );
    let tips_group = only(&groups, MarkRole::TangentTip);

    let tips = super::tangent_screen(&doc, Some(e), &cam, win);
    assert!(
        !tips.is_empty(),
        "a âncora moldada não ofereceu alça nenhuma"
    );

    // ⚠️ O centro de cada círculo DESENHADO (grupo 4, as pontas) tem de coincidir com uma
    // ponta que o hit-test conhece. Ler o DESENHO, não só a porta de hit, é o que fecha o
    // buraco "pintado num sítio, agarrado noutro": mover o `push_circle` para a âncora
    // deixa o hit intacto e só este oráculo sangra.
    let tip_pts = points(&tips_group.path);
    // push_circle desenha 1 move + TIP_SEGS line_to = 13 vértices por ponta.
    let per = 13;
    assert_eq!(tip_pts.len() % per, 0, "geometria de ponta inesperada");
    let drawn: Vec<(f64, f64)> = tip_pts
        .chunks(per)
        .map(|c| {
            let (sx, sy): (f64, f64) = c
                .iter()
                .fold((0.0, 0.0), |(ax, ay), p| (ax + p.0, ay + p.1));
            (sx / c.len() as f64, sy / c.len() as f64)
        })
        .collect();
    for (target, i, out, _anchor_pt, tip) in &tips {
        // O círculo desenhado para esta ponta.
        assert!(
            drawn
                .iter()
                .any(|(dx, dy)| (dx - tip.x).hypot(dy - tip.y) < 0.5),
            "o hit conhece uma ponta em ({:.1}, {:.1}) mas nenhum círculo foi DESENHADO ali",
            tip.x,
            tip.y
        );
        // E apertar no centro desenhado agarra AQUELA alça (a porta única, pelos dois lados).
        let hit = super::motion_path_hit(&doc, Some(e), &cam, win, tip.x as f32, tip.y as f32);
        assert_eq!(
            hit,
            Some(MotionPathGrab::Tangent {
                target: *target,
                i: *i,
                out: *out
            }),
            "a ponta em ({:.1}, {:.1}) não é a agarrada ali",
            tip.x,
            tip.y
        );
    }
}

/// Uma alça CURTA demais (a ponta a menos de TANGENT_MIN_PX da âncora) não é oferecida —
/// senão ela roubaria o alvo de mouse da própria âncora. É o mesmo pudor do grip de fade.
#[test]
fn a_tiny_tangent_is_not_offered_so_it_does_not_steal_the_anchor() {
    let (doc, e) = doc_shaped();
    // Câmera muito afastada: 4 u de alça viram ~2,5 px, abaixo do mínimo de 14.
    let far = Camera2d {
        height_world: 4000.0,
        ..camera()
    };
    let win = window();
    assert!(
        super::tangent_screen(&doc, Some(e), &far, win).is_empty(),
        "uma alça de ~2 px foi oferecida — vai competir com o quadrado da âncora"
    );
    // ...e a âncora continua agarrável (é o que o mínimo protege).
    let a = super::anchor_screen(&doc, Some(e), &far, win)[1].2;
    assert!(
        super::motion_path_hit(&doc, Some(e), &far, win, a.x as f32, a.y as f32).is_some(),
        "com a alça de fora, o clique na âncora tem de pegar a âncora"
    );
}

/// **A curva é mais QUENTE onde o objeto é mais RÁPIDO** — a fita de velocidade, a coisa
/// que a torna bela E legível de um relance. Um ease-in-out acelera no meio e freia nas
/// pontas, então a banda do meio tem de sair mais quente (mais perto do ouro branco) que
/// as das pontas. O calor é lido do canal verde da rampa (0,28 brasa → 0,95 quente),
/// monótono, então "mais quente" é comparável.
#[test]
fn the_ribbon_is_hotter_where_the_object_is_faster() {
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
    // (x do meio do segmento, calor=canal verde) por todas as bandas da fita.
    let mut samples: Vec<(f64, f32)> = Vec::new();
    for m in all(&out, MarkRole::Ribbon) {
        for seg in points(&m.path).chunks(2) {
            if seg.len() == 2 {
                samples.push(((seg[0].0 + seg[1].0) / 2.0, m.rgba[1]));
            }
        }
    }
    assert!(
        samples.len() >= 20,
        "poucos segmentos de fita: {}",
        samples.len()
    );
    samples.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let n = samples.len();
    let ends = (samples[0].1 + samples[n - 1].1) / 2.0;
    let middle = samples[n / 2].1;
    println!("MEDIDO  calor nas pontas {ends:.2}, no meio {middle:.2}");
    assert!(
        middle > ends + 0.1,
        "a fita não está mais quente no meio ({middle:.2}, rápido) que nas pontas \
         ({ends:.2}, lento) — a cor não está dizendo a velocidade"
    );
}

/// O CONTROLE: velocidade constante ⇒ fita UNIFORME. Sem ele, "quente no meio" poderia
/// ser uma propriedade do desenho e não do timing.
#[test]
fn a_constant_speed_track_paints_a_uniform_ribbon() {
    let (doc, e) = doc_with(&[(0.0, 0.0, Interp::Linear), (2.0, 20.0, Interp::Linear)]);
    let out = marks(true, &doc, Some(e), &camera(), window());
    let bands = all(&out, MarkRole::Ribbon);
    assert!(!bands.is_empty(), "sem fita");
    let greens: Vec<f32> = bands.iter().map(|m| m.rgba[1]).collect();
    let max = greens.iter().copied().fold(0.0f32, f32::max);
    let min = greens.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        max - min < 0.01,
        "a fita de uma velocidade constante variou de cor ({min:.2}..{max:.2}) — o ritmo \
         uniforme não deu uma fita uniforme"
    );
}
