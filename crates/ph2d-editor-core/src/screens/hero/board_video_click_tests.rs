//! O CLIQUE num vídeo, pelo despacho real: segurar e arrastar move e o clique dá play/pause (W20); segurar
//! parado não é clique e, com outra ferramenta na mão, o clique é do vídeo e o arrasto da ferramenta (W21).
//! Os ajudantes são os do módulo pai, `board_video_tests.rs`.

use super::*;

/// ⭐⭐ W20 (o dono: «segurar e arrastar move, apenas clicar dá play/pause») — pelo despacho real: o
/// arrasto a partir do play (parado), de um vídeo seleccionado e de um vídeo ACTIVO move-o e não o
/// activa nem manda nada ao tocador; o clique (sem arrasto) activa, e activo vai ao tocador no largar.
/// O oráculo move o parado e o seleccionado (A1, A3), activa no LARGAR (A2) e não move o activo (A4 —
/// aqui fazemos melhor: o tocador sem controlos do YouTube não tem gesto de arrastar que se perca).
#[test]
fn holding_and_dragging_a_video_moves_it_and_a_click_plays_it() {
    let mut t = T::new();
    assert!(paste(&mut t, LINK));
    let id = video(&t).unwrap().id;
    let out = t.at(0.02, 0.98);
    click(&mut t, out);
    let pos = |t: &T| {
        let v = video(t).unwrap();
        [v.x, v.y]
    };
    let z = t.hero.documents.active_board().unwrap().camera.zoom;
    let moved = |a: [f64; 2], b: [f64; 2]| {
        ((b[0] - a[0]) * z - 100.0).abs() < 1.0 && ((b[1] - a[1]) * z - 60.0).abs() < 1.0
    };
    for case in ["parado, pelo play", "seleccionado, pelo meio"] {
        let a = pos(&t);
        let p = on_video(&mut t, id, 0.5, 0.5);
        t.drag(p, (p.0 + 100.0, p.1 + 60.0));
        assert!(
            moved(a, pos(&t)),
            "{case}: não moveu {a:?} -> {:?}",
            pos(&t)
        );
        assert_eq!(
            t.hero.documents.videos_mut().active,
            None,
            "{case}: activou"
        );
    }
    // O clique (sem arrasto) no play activa.
    let p = on_video(&mut t, id, 0.5, 0.5);
    click(&mut t, p);
    assert_eq!(t.hero.documents.videos_mut().active, Some(id));
    t.hero.documents.videos_mut().status = Some(STATUS);
    // Activo: arrastar de dentro move-o e o tocador não recebe nada.
    t.hero.documents.videos_mut().inbox.clear();
    let a = pos(&t);
    let p = on_video(&mut t, id, 0.25, 0.25);
    t.drag(p, (p.0 + 100.0, p.1 + 60.0));
    assert!(moved(a, pos(&t)), "activo: não moveu");
    let inbox = std::mem::take(&mut t.hero.documents.videos_mut().inbox);
    assert!(
        !inbox
            .iter()
            .any(|i| matches!(i, PlayerInput::Down { .. } | PlayerInput::Up { .. })),
        "o arrasto chegou ao tocador: {inbox:?}"
    );
    assert_eq!(t.hero.documents.videos_mut().active, Some(id));
    // Activo: o clique vai ao tocador — Down e Up no ponto do clique.
    let p = on_video(&mut t, id, 0.25, 0.25);
    click(&mut t, p);
    let inbox = std::mem::take(&mut t.hero.documents.videos_mut().inbox);
    assert!(
        matches!(
            inbox.as_slice(),
            [PlayerInput::Down { .. }, PlayerInput::Up { .. }]
        ),
        "{inbox:?}"
    );
}

/// ⭐ W20 — clicar num vídeo desactivado que o tocador ainda tem dá tocar/pausar (o clique é sempre
/// play/pause); e o clique que ACTIVA um vídeo novo não manda ordem (o tocador arranca a tocar).
#[test]
fn a_click_on_the_video_the_player_holds_plays_or_pauses_it() {
    let mut t = T::new();
    let id = active(&mut t);
    assert_eq!(commands(&mut t), vec![], "activar não é uma ordem");
    let out = t.at(0.02, 0.98);
    click(&mut t, out);
    let p = on_video(&mut t, id, 0.5, 0.5);
    click(&mut t, p);
    assert_eq!(t.hero.documents.videos_mut().active, Some(id));
    assert_eq!(commands(&mut t), vec![VideoCommand::TogglePlay]);
}

/// ⭐⭐ W21 (o dono, 13/10: «corrija») — segurar parado num vídeo e largar NÃO é clique: não activa um vídeo
/// parado, não chega ao tocador de um activo, não toca/pausa o que o tocador tem. Logo abaixo do limiar é
/// clique. O oráculo (`sonda_w21.mjs` T1): o largar depois de segurar parado além do limiar não activa.
#[test]
fn a_long_still_press_on_a_video_is_not_a_click() {
    let long = board_video::HOLD_NS;
    assert_eq!(
        long,
        crate::interaction::LONG_PRESS_THRESHOLD_NS,
        "o segurar do rato da casa"
    );
    let mut t = T::new();
    assert!(paste(&mut t, LINK));
    let id = video(&t).unwrap().id;
    let out = t.at(0.02, 0.98);
    click(&mut t, out);
    // Parado: segurar no play não activa; logo abaixo do limiar, activa.
    let play = on_video(&mut t, id, 0.5, 0.5);
    hold(&mut t, play, long);
    assert_eq!(
        t.hero.documents.videos_mut().active,
        None,
        "segurou e activou"
    );
    click(&mut t, out);
    hold(&mut t, play, long - 1);
    assert_eq!(
        t.hero.documents.videos_mut().active,
        Some(id),
        "o clique abaixo do limiar"
    );
    t.hero.documents.videos_mut().status = Some(STATUS);
    t.paint();
    // Activo e vivo: segurar dentro não chega ao tocador.
    t.hero.documents.videos_mut().inbox.clear();
    let inside = on_video(&mut t, id, 0.25, 0.75);
    hold(&mut t, inside, long);
    let inbox = std::mem::take(&mut t.hero.documents.videos_mut().inbox);
    assert!(
        !inbox
            .iter()
            .any(|i| matches!(i, PlayerInput::Down { .. } | PlayerInput::Up { .. })),
        "segurou e o tocador recebeu um clique: {inbox:?}"
    );
    // Desactivado, com o tocador a segurá-lo: segurar não toca/pausa nem o activa.
    click(&mut t, out);
    let _ = commands(&mut t);
    hold(&mut t, play, long);
    assert_eq!(commands(&mut t), vec![], "segurou e tocou/pausou");
    assert_eq!(
        t.hero.documents.videos_mut().active,
        None,
        "segurou e activou"
    );
}

/// ⭐⭐ W21 (o dono, 13/10, escolheu «clique = play/pause») — com OUTRA ferramenta na mão, um clique num
/// vídeo é do vídeo: activa-o (parado) ou vai ao tocador (activo), e nada se desenha. Arrastar é da
/// ferramenta (desenha por cima, o vídeo fica onde estava) e segurar parado também (a ferramenta faz o seu
/// clique). O oráculo deixa a ferramenta fazer tudo (`sonda_w21.mjs` F_*); o dono quis outra coisa.
#[test]
fn with_another_tool_a_click_on_a_video_is_the_videos_and_a_drag_is_the_tools() {
    use ph2d_board_model::Pen;
    let tools = [
        Tool::Pen(Pen::Pen),
        Tool::Shape(ShapeType::Rectangle),
        Tool::Shape(ShapeType::Sticky),
        Tool::Connector,
        Tool::Frame,
        Tool::MindMap,
        Tool::Eraser { precise: false },
    ];
    for tool in tools {
        let mut t = T::new();
        assert!(paste(&mut t, LINK));
        let id = video(&t).unwrap().id;
        let out = t.at(0.02, 0.98);
        click(&mut t, out);
        t.hero.documents.live.editor.as_mut().unwrap().tool = tool;
        let n = t.elements().len();
        let play = on_video(&mut t, id, 0.5, 0.5);
        click(&mut t, play);
        assert_eq!(
            t.hero.documents.videos_mut().active,
            Some(id),
            "{tool:?}: o clique não activou"
        );
        assert_eq!(t.elements().len(), n, "{tool:?}: o clique desenhou");
        t.hero.documents.videos_mut().status = Some(STATUS);
        t.paint();
        t.hero.documents.videos_mut().inbox.clear();
        let inside = on_video(&mut t, id, 0.25, 0.75);
        click(&mut t, inside);
        let inbox = std::mem::take(&mut t.hero.documents.videos_mut().inbox);
        assert!(
            matches!(
                inbox.as_slice(),
                [PlayerInput::Down { .. }, PlayerInput::Up { .. }]
            ),
            "{tool:?}: activo, o clique não foi ao tocador: {inbox:?}"
        );
        assert_eq!(t.elements().len(), n, "{tool:?}: o clique desenhou");
    }
    // Arrastar desenha por cima e não move o vídeo; segurar é o clique da ferramenta.
    for tool in [
        Tool::Pen(Pen::Pen),
        Tool::Shape(ShapeType::Rectangle),
        Tool::Connector,
        Tool::Frame,
    ] {
        let mut t = T::new();
        assert!(paste(&mut t, LINK));
        let id = video(&t).unwrap().id;
        let out = t.at(0.02, 0.98);
        click(&mut t, out);
        t.hero.documents.live.editor.as_mut().unwrap().tool = tool;
        let n = t.elements().len();
        let at = |t: &T| {
            let v = video(t).unwrap();
            [v.x, v.y]
        };
        let a = at(&t);
        let p = on_video(&mut t, id, 0.25, 0.25);
        t.drag(p, (p.0 + 100.0, p.1 + 60.0));
        assert_eq!(
            t.elements().len(),
            n + 1,
            "{tool:?}: o arrasto não desenhou"
        );
        assert_eq!(at(&t), a, "{tool:?}: o arrasto moveu o vídeo");
        assert_eq!(
            t.hero.documents.videos_mut().active,
            None,
            "{tool:?}: activou"
        );
    }
    let mut t = T::new();
    assert!(paste(&mut t, LINK));
    let id = video(&t).unwrap().id;
    let out = t.at(0.02, 0.98);
    click(&mut t, out);
    t.hero.documents.live.editor.as_mut().unwrap().tool = Tool::Frame;
    let n = t.elements().len();
    let p = on_video(&mut t, id, 0.25, 0.25);
    hold(&mut t, p, board_video::HOLD_NS);
    assert_eq!(
        t.elements().len(),
        n + 1,
        "segurar com o frame não pôs um frame"
    );
    assert_eq!(
        t.hero.documents.videos_mut().active,
        None,
        "segurar activou"
    );
}

/// ⭐⭐ W21 (o dono, 13/10: «se alguma forma como notas, traços de caneta ou shapes sobrepõe parcialmente o
/// vídeo, o vídeo não toca») — um rectângulo e um traço da caneta desenhados por cima de parte do vídeo: o
/// clique no play toca-o (vivo), e o clique na parte sobreposta apanha o VÍDEO (ele está à frente, como se
/// vê), não o que ficou escondido atrás dele.
#[test]
fn shapes_over_part_of_a_video_never_stop_it_and_the_video_takes_the_click() {
    use ph2d_board_model::Pen;
    let mut t = T::new();
    assert!(paste(&mut t, LINK));
    let id = video(&t).unwrap().id;
    let out = t.at(0.02, 0.98);
    click(&mut t, out);
    let n = t.elements().len();
    let corner = on_video(&mut t, id, 0.0, 0.0);
    let inside = on_video(&mut t, id, 0.3, 0.3);
    for tool in [Tool::Shape(ShapeType::Rectangle), Tool::Pen(Pen::Pen)] {
        t.hero.documents.live.editor.as_mut().unwrap().tool = tool;
        t.drag((corner.0 - 40.0, corner.1 - 40.0), inside);
    }
    assert_eq!(t.elements().len(), n + 2, "o rectângulo e o traço");
    t.hero.documents.live.editor.as_mut().unwrap().tool = Tool::Select;
    click(&mut t, out);
    let play = on_video(&mut t, id, 0.5, 0.5);
    click(&mut t, play);
    assert_eq!(t.hero.documents.videos_mut().active, Some(id));
    assert!(live(&mut t, id), "tapado em parte e não toca");
    click(&mut t, out);
    let over = on_video(&mut t, id, 0.2, 0.2);
    click(&mut t, over);
    let sel = t
        .hero
        .documents
        .live
        .editor
        .as_ref()
        .unwrap()
        .selection()
        .clone();
    assert_eq!(
        sel,
        [id].into(),
        "o clique na parte sobreposta não apanhou o vídeo"
    );
}
