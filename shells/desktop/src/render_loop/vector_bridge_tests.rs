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
        align: ph2d_vec_scene::StrokeAlign::Centre,
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
            .and_then(|p| p.stroke.clone())
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

/// A largura só é reescrita quando o chamador a ENTREGA: com `None` ela é preservada (é isso que
/// impede uma escolha de cor — ou de Head Size — de reengrossar a linha), com `Some(w)` ela segue.
///
/// ⚠️ Este gate prova a **canalização**, não a política. Ele chamava-se *"…while the slider is
/// dragged"* e o nome afirmava uma lei que era do CHAMADOR e estava errada: quem decide o `Some`
/// é o `vector_bridge`, e ele perguntava se o slider estava em arrasto — o que deixava a caixa
/// numérica ao lado muda sobre a forma selecionada. Quem pina a política é o
/// `the_width_reaches_the_selection_when_it_is_authored_not_when_a_slider_is_dragged`.
#[test]
fn the_width_is_rewritten_only_when_the_caller_hands_one() {
    let (mut scene, ids) = scene_with(1);

    restyle_selected_strokes(&mut scene, &ids, &style(), None);
    let kept = scene.paths()[0].stroke.as_ref().expect("traco").width;
    assert!(
        (kept - W).abs() < f64::EPSILON,
        "sem largura entregue ela tem de ser PRESERVADA: {kept} != {W}"
    );

    restyle_selected_strokes(&mut scene, &ids, &style(), Some(0.5));
    let handed = scene.paths()[0].stroke.as_ref().expect("traco").width;
    assert!((handed - 0.5).abs() < f64::EPSILON, "entregue, ela segue");
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

/// **A LEI: o que o apply ESCREVE, a seleção LÊ de volta.**
///
/// ⚠️ Este é o gate da wave, e o oráculo é um ROUND-TRIP, não uma lista de campos. Um gate que
/// enumerasse os campos nasceria incompleto no dia em que um entrasse — que é literalmente o que
/// aconteceu: a semente adotava só as PONTAS, e cor/largura/cap/join/dash/align ficavam com o
/// último valor autorado (report do Enio, 2026-08-01).
///
/// E não era só display: o `take_apply_to_selected` REESCREVE a seleção com o que a tool tem,
/// então tocar um controle empurrava o estilo velho inteiro para cima da forma recém-selecionada.
#[test]
fn what_the_apply_writes_the_selection_reads_back() {
    use ph2d_tool_vector::VectorTool;
    use ph2d_vec_scene::{Marker, StrokeAlign};

    // Uma ficha DELIBERADAMENTE diferente do default em todo campo — sem isso o round-trip
    // passa por acidente sobre o valor de fábrica.
    let ficha = StrokeStyle {
        color: Rgba8::new(11, 22, 33, 200),
        cap: LineCap::Square,
        join: LineJoin::Bevel,
        align: StrokeAlign::Outer,
        dash: Some((2.5, 1.5)),
        marker_start: Marker::Triangle,
        marker_end: Marker::DiamondOpen,
        marker_scale: 1.75,
        marker_round: 0.5,
    };
    let width_world = 0.4;
    let spec = ficha.onto(width_world);

    let mut tool = VectorTool::default();
    // O documento fala MUNDO, a tool fala px de TELA — a conversão é do chamador.
    let world_to_px = 50.0;
    tool.adopt_stroke(&spec, width_world * world_to_px);

    // …e a volta: a ficha que a tool produziria tem de ser a MESMA.
    let back = StrokeStyle {
        color: super::style::rgba(tool.stroke_rgba()),
        cap: tool.cap().into(),
        join: tool.join().into(),
        align: tool.stroke_align(),
        dash: (tool.dash() > 0.0).then_some((tool.dash(), tool.gap())),
        marker_start: tool.marker_start(),
        marker_end: tool.marker_end(),
        marker_scale: tool.marker_scale(),
        marker_round: tool.marker_round(),
    };
    assert!(
        !back.differs_from(&spec),
        "a adocao perdeu um campo da ficha"
    );
    assert!(
        (tool.stroke_width_px() - width_world * world_to_px).abs() < 1e-9,
        "a largura nao atravessou a fronteira de unidade"
    );
}

/// **Adotar é LER — nunca arma a reescrita da seleção.**
///
/// ⚠️ Sem isto o produto entraria num laço: selecionar arma, o apply reescreve a seleção com o
/// que ela já é, e cada frame vira um passo de undo. É a mesma lei que o `adopt_markers` já
/// declarava, agora com gate.
#[test]
fn adopting_a_style_does_not_arm_the_restyle() {
    use ph2d_tool_vector::VectorTool;
    let mut tool = VectorTool::default();
    assert!(!tool.take_apply_to_selected(), "o default nao arma");
    tool.adopt_stroke(&style().onto(0.2), 10.0);
    tool.adopt_fill([1, 2, 3, 4]);
    assert!(
        !tool.take_apply_to_selected(),
        "adotar armou a reescrita da selecao"
    );
}

/// **Selecionar um caminho re-semeia o STORE, não só a tool.**
///
/// ⚠️ As duas metades são perguntas diferentes, e sem a segunda metade do painel continua a
/// mentir. As rows de Cap/Join/Align pintam do SNAPSHOT (que segue a tool), mas Width, as duas
/// Opacity, Dash e Gap pintam do **STORE** — `store.slider(...)` com o snapshot só como
/// *fallback*, e o fallback nunca dispara porque o `populate` já registrou o widget. Adotar na
/// tool sem semear o store deixaria a barra de Width parada no valor anterior.
///
/// Este gate dirige a PORTA REAL (`seed_style_from_selection`), não o helper privado.
#[test]
fn selecting_a_path_reseeds_the_store_not_only_the_tool() {
    use ph2d_editor::panel::Panel;
    use ph2d_tool_vector::{VectorTool, params};
    use ph2d_vec_edit::PenTool;
    use ph2d_vec_scene::{StrokeSpec, VecPath, VecVertex};

    // O store REGISTRADO — o fluxo do produto; num vazio o `set` não alcança widget nenhum.
    let mut store = ph2d_editor::interaction::WidgetStore::default();
    <ph2d_panel_vector::VectorPanel as Panel>::populate(&mut store);

    // Um caminho com largura BEM diferente do default, para o número não passar por acidente.
    let world_to_px = 50.0;
    let width_world = 1.5; // 75 px de tela — fora da faixa que o default ocupa
    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath {
        verts: vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::corner([1.0, 0.0]),
            VecVertex::corner([1.0, 1.0]),
        ],
        closed: true,
        stroke: Some(StrokeSpec::new(Rgba8::new(9, 9, 9, 255), width_world)),
        ..VecPath::default()
    });

    let mut tool = VectorTool::default();
    let mut pen = PenTool::default();
    pen.select(Some(id));
    super::style::seed_style_from_selection(&mut tool, &mut store, &pen, &scene, world_to_px);

    assert!(
        (tool.stroke_width_px() - width_world * world_to_px).abs() < 1e-9,
        "a tool nao adotou a largura"
    );
    let track = store
        .slider(ph2d_editor::ids::VECTOR_WIDTH)
        .map(|(_, v)| v)
        .expect("o slider de Width existe no store");
    assert!(
        (f64::from(track) - f64::from(params::px_to_slider(width_world * world_to_px))).abs()
            < 1e-6,
        "o STORE ficou com o valor anterior: a barra de Width mostraria a largura errada \
         (track {track})"
    );
}
