//! W18 — o vídeo pelo despacho REAL: colar um link do YouTube (Ctrl+V) faz nascer um vídeo; o clique no
//! play activa-o; activo e vivo, o rato e as teclas vão para o tocador e não para o quadro; clicar fora
//! desactiva; o botão «Video» pede o link à área de transferência; outro link continua texto.

use super::*;
use crate::screens::hero::board_video;
use crate::screens::hero::board_video_bar::{self, Ctl, layout};
use ph2d_board_model::ElementId;
use ph2d_board_render::{PlayerInput, PlayerStatus, VideoCommand};

const LINK: &str = "https://www.youtube.com/watch?v=aqz-KE-bpKQ";

fn paste(t: &mut T, text: &str) -> bool {
    let out = board_keys::key(
        &mut t.hero,
        &mut t.ts,
        BoardKey::Char('v'),
        CTRL,
        true,
        None,
        Some(text.into()),
    );
    board_keys::key(
        &mut t.hero,
        &mut t.ts,
        BoardKey::Char('v'),
        CTRL,
        false,
        None,
        None,
    );
    t.paint();
    out.consumed
}

fn video(t: &T) -> Option<Element> {
    t.elements().into_iter().find(|e| e.embed().is_some())
}

/// O ponto de ecrã do local `(fx, fy)` (fracções da caixa) do vídeo, pelo que o desenho registou.
fn on_video(t: &mut T, id: ElementId, fx: f64, fy: f64) -> (f32, f32) {
    let el = video(t).unwrap();
    let p = t.hero.documents.videos_mut().on_screen[&id].to_screen
        * ph2d_vector::Point::new(el.w * fx, el.h * fy);
    (p.x as f32, p.y as f32)
}

/// Um clique rápido (o largar `QUICK_NS` depois do carregar) pelo despacho real.
fn click(t: &mut T, at: (f32, f32)) {
    hold(t, at, QUICK_NS);
}

/// O tempo entre o carregar e o largar de um clique dos testes.
const QUICK_NS: u128 = 80_000_000;

/// Carregar em `(x, y)`, ficar parado `ns` e largar, pelo despacho real (o relógio do `Input`).
fn hold(t: &mut T, (x, y): (f32, f32), ns: u128) {
    let p = PointerButton::Primary;
    board_view::pointer(
        &mut t.hero,
        inp(&mut t.ts, &mut t.clock),
        PointerKind::Down,
        p,
        x,
        y,
        true,
    );
    t.clock += ns;
    let up = Input {
        text: &mut t.ts,
        mods: Modifiers::default(),
        now_ns: t.clock,
    };
    board_view::pointer(&mut t.hero, up, PointerKind::Up, p, x, y, true);
    t.paint();
}

/// ⭐⭐ O caminho inteiro do vídeo no quadro.
#[test]
fn a_pasted_link_becomes_a_video_that_plays_on_its_play_and_takes_the_mouse_and_keys() {
    let mut t = T::new();
    assert!(paste(&mut t, LINK), "o link colado é do vídeo");
    let v = video(&t).expect("nasceu um vídeo");
    assert_eq!(v.embed().unwrap().link, LINK);
    assert_eq!([v.w, v.h], [560.0, 315.0], "o tamanho do oráculo");
    let id = v.id;
    assert!(
        t.hero.documents.videos_mut().on_screen.contains_key(&id),
        "o desenho registou-o"
    );
    // Activar: o clique no play.
    let play = on_video(&mut t, id, 0.5, 0.5);
    click(&mut t, play);
    assert_eq!(t.hero.documents.videos_mut().active, Some(id));
    let pl = t.hero.documents.videos_mut().on_screen[&id];
    assert!(
        pl.live,
        "inteiro à vista, descoberto, ≥ 200 px: vivo ({:?})",
        pl.screen
    );
    // Activo: o clique dentro vai para o tocador (o vídeo não se mexe).
    t.hero.documents.videos_mut().inbox.clear();
    let inside = on_video(&mut t, id, 0.25, 0.75);
    click(&mut t, inside);
    let inbox = std::mem::take(&mut t.hero.documents.videos_mut().inbox);
    assert!(
        matches!(inbox.first(), Some(PlayerInput::Down { button: 0, .. })),
        "{inbox:?}"
    );
    assert!(
        inbox.iter().any(|i| matches!(i, PlayerInput::Up { .. })),
        "{inbox:?}"
    );
    let [x, y] = match inbox[0] {
        PlayerInput::Down { x, y, .. } => [x, y],
        _ => unreachable!(),
    };
    assert!(
        (x - pl.screen.size[0] * 0.25).abs() < 1.0 && (y - pl.screen.size[1] * 0.75).abs() < 1.0,
        "{x},{y}"
    );
    assert_eq!(
        [video(&t).unwrap().x, video(&t).unwrap().y],
        [v.x, v.y],
        "o quadro não arrastou"
    );
    // As teclas (W19): o espaço é dos NOSSOS controles (o tocador vem com `disablekb=1`).
    t.hero.documents.videos_mut().status = Some(STATUS);
    assert!(t.key(BoardKey::Space, Modifiers::default(), None));
    let cmds = std::mem::take(&mut t.hero.documents.videos_mut().commands);
    assert_eq!(cmds, vec![VideoCommand::TogglePlay]);
    // Fora: desactiva.
    let out = t.at(0.02, 0.98);
    click(&mut t, out);
    assert_eq!(t.hero.documents.videos_mut().active, None);
}

/// Um link que NÃO é um vídeo do YouTube continua o colar de texto (nenhum vídeo nasce).
#[test]
fn another_link_stays_text() {
    let mut t = T::new();
    let _ = paste(&mut t, "https://example.com/page");
    assert!(video(&t).is_none());
}

/// O botão «Video» está na barra curta, e o clique pede o link ao app.
#[test]
fn the_video_button_asks_the_app_for_the_clipboard_link() {
    let mut t = T::new();
    assert!(!t.hero.documents.take_video_ask());
    t.click_bar(Item::Video);
    assert!(t.hero.documents.take_video_ask());
    // E o app, com um link, põe o vídeo pelo mesmo caminho do colar.
    assert!(board_video::insert_link(&mut t.hero, LINK, None));
    t.paint();
    assert!(video(&t).is_some());
}

/// O que o app põe a cada quadro com o tocador a tocar (aos 10 s de 100).
const STATUS: PlayerStatus = PlayerStatus {
    time: 10.0,
    duration: 100.0,
    playing: true,
    volume: 80,
    muted: false,
};

/// Um vídeo colado, activado pelo play, com o estado do tocador (como o app o deixa).
fn active(t: &mut T) -> ElementId {
    assert!(paste(t, LINK));
    let id = video(t).unwrap().id;
    let play = on_video(t, id, 0.5, 0.5);
    click(t, play);
    t.hero.documents.videos_mut().status = Some(STATUS);
    t.paint();
    id
}

/// Down + Up pelo despacho real em `(x, y)`; os eventos vão ao ecrã.
fn press(t: &mut T, (x, y): (f32, f32)) {
    let arena = Bump::new();
    let mut ev = t
        .hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(t.hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    for e in ev {
        t.hero.apply_event(e);
    }
    t.paint();
}

fn ctl(t: &T, c: Ctl) -> (f32, f32) {
    t.find(board_video_bar::node(c))
        .unwrap_or_else(|| panic!("{c:?} não está pintado"))
}

/// Clica no controlo `c` da NOSSA barra pelo despacho real.
fn tap(t: &mut T, c: Ctl) {
    let at = ctl(t, c);
    press(t, at);
}

fn commands(t: &mut T) -> Vec<VideoCommand> {
    std::mem::take(&mut t.hero.documents.videos_mut().commands)
}

/// O rectângulo de ecrã do vídeo, pela colocação que o desenho registou.
fn video_rect(t: &mut T, id: ElementId) -> [f32; 4] {
    let (a, b) = (on_video(t, id, 0.0, 0.0), on_video(t, id, 1.0, 1.0));
    [a.0, a.1, b.0 - a.0, b.1 - a.1]
}

/// ⭐⭐ W19 — os NOSSOS controles: pintados POR BAIXO do vídeo (nada à frente dele — as regras do
/// YouTube), e cada um, pelo despacho real, dá a ordem dele ao tocador.
#[test]
fn our_controls_sit_under_the_video_and_each_gives_its_order() {
    let mut t = T::new();
    let id = active(&mut t);
    let [vx, vy, vw, vh] = video_rect(&mut t, id);
    for c in [
        Ctl::Play,
        Ctl::Seek,
        Ctl::Mute,
        Ctl::Volume,
        Ctl::Fullscreen,
        Ctl::Outside,
    ] {
        let (x, y) = ctl(&t, c);
        assert!(
            y > vy + vh,
            "{c:?} em {x},{y}: à frente do vídeo {vx},{vy} {vw}×{vh}"
        );
    }
    for (c, want) in [
        (Ctl::Play, VideoCommand::TogglePlay),
        (Ctl::Mute, VideoCommand::ToggleMute),
        (Ctl::Outside, VideoCommand::OpenOutside),
    ] {
        let at = ctl(&t, c);
        press(&mut t, at);
        assert_eq!(commands(&mut t), vec![want], "{c:?}");
    }
    // A barra de tempo: o clique a meio salta para o meio do vídeo.
    let (x, y) = ctl(&t, Ctl::Seek);
    press(&mut t, (x, y));
    let got = commands(&mut t);
    assert!(
        matches!(got.as_slice(), [VideoCommand::Seek(s)] if (s - 50.0).abs() < 3.0),
        "{got:?}"
    );
    assert_eq!(
        t.hero.documents.videos_mut().active,
        Some(id),
        "a barra não desactiva o vídeo"
    );
}

/// ⭐⭐ W19 — o NOSSO ecrã inteiro: o vídeo enche a área do quadro com a proporção dele (16:9, nunca
/// esticado), a barra fica por baixo dele, as barras do quadro somem, o rato no preto e a roda não chegam
/// ao quadro, e o `Esc` sai.
#[test]
fn our_fullscreen_fills_the_board_keeping_the_shape_and_esc_leaves() {
    let mut t = T::new();
    let id = active(&mut t);
    let at = ctl(&t, Ctl::Fullscreen);
    press(&mut t, at);
    assert!(board_video::fullscreen(&t.hero));
    t.paint();
    let [x, y, w, h] = video_rect(&mut t, id);
    let area = t.hero.last_canvas;
    assert!((w / h - 16.0 / 9.0).abs() < 1e-3, "esticado: {w}×{h}");
    assert!(
        (w - area.w).abs() < 1.0 || h > area.h * 0.8,
        "não encheu a área {area:?}: {w}×{h}"
    );
    assert!(x >= area.x && y >= area.y && x + w <= area.x + area.w + 0.5);
    let pl = t.hero.documents.videos_mut().on_screen[&id];
    assert!(
        pl.live,
        "no ecrã inteiro o tocador está vivo: {:?}",
        pl.screen
    );
    let (_, by) = ctl(&t, Ctl::Play);
    assert!(by > y + h, "a barra à frente do vídeo");
    assert!(
        t.find(super::node(Item::Video)).is_none(),
        "as barras do quadro ficaram por baixo"
    );
    // O preto à volta: não desactiva nem chega ao quadro.
    click(&mut t, (area.x + 2.0, area.y + 2.0));
    assert_eq!(t.hero.documents.videos_mut().active, Some(id));
    assert!(board_view::wheel(
        &mut t.hero,
        area.x + 2.0,
        area.y + 2.0,
        1.0
    ));
    assert!(t.key(BoardKey::Escape, Modifiers::default(), None));
    assert!(!board_video::fullscreen(&t.hero));
}

/// W19 — os atalhos do YouTube são dos nossos controles.
#[test]
fn the_youtube_shortcuts_drive_our_controls() {
    let mut t = T::new();
    let _ = active(&mut t);
    let none = Modifiers::default();
    for (k, want) in [
        (BoardKey::Char('k'), VideoCommand::TogglePlay),
        (BoardKey::Right, VideoCommand::Seek(15.0)),
        (BoardKey::Left, VideoCommand::Seek(5.0)),
        (BoardKey::Char('l'), VideoCommand::Seek(20.0)),
        (BoardKey::Char('j'), VideoCommand::Seek(0.0)),
        (BoardKey::Up, VideoCommand::Volume(85)),
        (BoardKey::Down, VideoCommand::Volume(75)),
        (BoardKey::Char('m'), VideoCommand::ToggleMute),
    ] {
        assert!(t.key(k, none, None), "{k:?}");
        assert_eq!(commands(&mut t), vec![want], "{k:?}");
    }
    assert!(t.key(BoardKey::Char('f'), none, None));
    assert!(board_video::fullscreen(&t.hero));
}

/// Arrasto com `button` no quadro, pela porta da shell (`t.drag` é só o principal).
fn drag_with(t: &mut T, button: PointerButton, a: (f32, f32), b: (f32, f32)) {
    let (h, ts, c) = (&mut t.hero, &mut t.ts, &mut t.clock);
    board_view::pointer(h, inp(ts, c), PointerKind::Down, button, a.0, a.1, true);
    board_view::pointer_move(h, inp(ts, c), (a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    board_view::pointer_move(h, inp(ts, c), b.0, b.1);
    board_view::pointer(h, inp(ts, c), PointerKind::Up, button, b.0, b.1, true);
    t.paint();
}

fn space(t: &mut T, pressed: bool) {
    let text = pressed.then_some(" ");
    let none = Modifiers::default();
    board_keys::key(
        &mut t.hero,
        &mut t.ts,
        BoardKey::Space,
        none,
        pressed,
        text,
        None,
    );
}

fn camera(t: &T) -> [f64; 3] {
    let c = t.hero.documents.active_board().unwrap().camera;
    [c.center_x, c.center_y, c.zoom]
}

fn live(t: &mut T, id: ElementId) -> bool {
    t.hero
        .documents
        .videos_mut()
        .on_screen
        .get(&id)
        .is_some_and(|p| p.live)
}

/// ⭐⭐ W20 (o dono, 12/10: «Pan pausa o vídeo») — nenhum dos pans (o botão do meio, que é também o dos
/// dois dedos; o Espaço; a mão), começado no vazio ou SOBRE o vídeo, chega ao vídeo: ele fica activo e
/// vivo, nenhuma ordem vai ao tocador, e a vista anda. O oráculo (`sonda_w20.mjs`, P_*): o vídeo toca
/// depois de todos.
#[test]
fn a_pan_never_reaches_the_video() {
    let mut t = T::new();
    let id = active(&mut t);
    let over = on_video(&mut t, id, 0.25, 0.25);
    let empty = t.at(0.05, 0.1);
    for (name, at) in [("vazio", empty), ("sobre", over)] {
        let to = (at.0 + 40.0, at.1 + 20.0);
        for pan in ["meio", "espaco", "mao"] {
            t.hero.documents.videos_mut().inbox.clear();
            let before = camera(&t);
            match pan {
                "meio" => drag_with(&mut t, PointerButton::Middle, at, to),
                "espaco" => {
                    space(&mut t, true);
                    t.drag(at, to);
                    space(&mut t, false);
                }
                _ => {
                    board_view::set_tool(&mut t.hero, ph2d_board_edit::Tool::Hand);
                    t.drag(at, to);
                    board_view::set_tool(&mut t.hero, ph2d_board_edit::Tool::Select);
                }
            }
            // A vista volta (o próximo gesto começa no mesmo sítio do vídeo).
            let after = camera(&t);
            t.hero
                .documents
                .active_board_mut()
                .unwrap()
                .camera
                .pan_by_screen(-40.0, -20.0);
            t.paint();
            assert_ne!(before, after, "{pan} {name}: a vista não andou");
            assert_eq!(
                t.hero.documents.videos_mut().active,
                Some(id),
                "{pan} {name}: desactivou"
            );
            assert!(live(&mut t, id), "{pan} {name}: deixou de estar vivo");
            assert_eq!(commands(&mut t), vec![], "{pan} {name}: ordem ao tocador");
            let inbox = std::mem::take(&mut t.hero.documents.videos_mut().inbox);
            assert!(
                !inbox.iter().any(|i| matches!(i, PlayerInput::Down { .. })),
                "{pan} {name}: o carregar foi ao tocador {inbox:?}"
            );
        }
    }
}

/// ⭐ W20 — clicar fora tira o RATO ao vídeo, não o som: desactivado, ele continua vivo (o oráculo,
/// P_clique_fora: o tempo anda 4,26 → 6,27 s depois do clique fora).
#[test]
fn clicking_outside_takes_the_mouse_not_the_picture() {
    let mut t = T::new();
    let id = active(&mut t);
    let out = t.at(0.02, 0.98);
    click(&mut t, out);
    assert_eq!(t.hero.documents.videos_mut().active, None);
    assert!(live(&mut t, id), "desactivado, o vídeo pausou");
    // As teclas já não são dele: o Espaço volta a ser do quadro.
    t.hero.documents.videos_mut().status = Some(STATUS);
    space(&mut t, true);
    space(&mut t, false);
    assert_eq!(commands(&mut t), vec![]);
}

/// ⭐⭐ W20 (o dono: «só deve pausar se o quadro estiver fora da área visível») — com 150 px à vista o
/// vídeo continua vivo; todo fora, não; de volta, vivo outra vez. (A regra do YouTube é o tamanho do
/// TOCADOR, ≥ 200×200 — não o pedaço à vista; o oráculo toca com 150 px e todo fora.)
#[test]
fn a_video_plays_until_it_is_wholly_out_of_view() {
    let mut t = T::new();
    let id = active(&mut t);
    let area = t.hero.last_canvas;
    let [vx, _, vw, _] = video_rect(&mut t, id);
    let dx = area.x + 150.0 - (vx + vw);
    let pan = |t: &mut T, dx: f32| {
        let cam = &mut t.hero.documents.active_board_mut().unwrap().camera;
        cam.pan_by_screen(f64::from(dx), 0.0);
        t.paint();
    };
    pan(&mut t, dx);
    let [vx, _, vw, _] = video_rect(&mut t, id);
    assert!((vx + vw - area.x - 150.0).abs() < 1.0, "{vx} {vw}");
    assert!(live(&mut t, id), "150 px à vista: pausou");
    pan(&mut t, -200.0);
    assert!(!live(&mut t, id), "todo fora da vista: continua vivo");
    pan(&mut t, 200.0 - dx);
    assert!(live(&mut t, id), "de volta: não voltou");
}

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
    assert_eq!(t.hero.documents.videos_mut().active, None, "segurou e activou");
    click(&mut t, out);
    hold(&mut t, play, long - 1);
    assert_eq!(t.hero.documents.videos_mut().active, Some(id), "o clique abaixo do limiar");
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
    assert_eq!(t.hero.documents.videos_mut().active, None, "segurou e activou");
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
        assert_eq!(t.elements().len(), n + 1, "{tool:?}: o arrasto não desenhou");
        assert_eq!(at(&t), a, "{tool:?}: o arrasto moveu o vídeo");
        assert_eq!(t.hero.documents.videos_mut().active, None, "{tool:?}: activou");
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
    assert_eq!(t.elements().len(), n + 1, "segurar com o frame não pôs um frame");
    assert_eq!(t.hero.documents.videos_mut().active, None, "segurar activou");
}

/// ⭐⭐ W20 (o dono: «a janela de vídeo deve ter as opções de decoração como um retângulo: borda (stroke),
/// cor da borda, etc.») — um vídeo seleccionado tem na barra os grupos do rectângulo (o fundo, a cor e a
/// espessura da borda, o traço, os cantos, a opacidade); cada clique fica no vídeo, UM passo de desfazer;
/// com borda o tocador recua para dentro dela (o desenho: nada por cima do vídeo).
#[test]
fn the_style_bar_decorates_a_selected_video_like_a_rectangle() {
    let mut t = T::new();
    assert!(paste(&mut t, LINK));
    let id = video(&t).unwrap().id;
    board_view::select(&mut t.hero, [id]);
    t.paint();
    for it in [
        Item::Fill(Some(0)),
        Item::Stroke(None),
        Item::Width(0),
        Item::Dash(Dash::Dashed),
        Item::Round(false),
        Item::Opacity(0),
    ] {
        assert!(t.find(node(it)).is_some(), "{it:?} na barra do vídeo");
    }
    let bare = t.hero.documents.videos_mut().on_screen[&id].screen.size;
    t.click_bar(Item::Stroke(Some(Some(BLACK))));
    let thick = WIDTHS.len() - 1;
    t.click_bar(Item::Width(thick));
    t.click_bar(Item::Dash(Dash::Dotted));
    t.click_bar(Item::Fill(Some(0)));
    t.click_bar(Item::Round(false));
    let st = video(&t).unwrap().style().clone();
    assert_eq!(st.stroke, Some(palette(BLACK)));
    assert_eq!(
        (st.stroke_width, st.dash, st.round),
        (WIDTHS[thick], Dash::Dotted, false)
    );
    assert!(st.fill.is_some(), "o fundo");
    let z = t.hero.documents.active_board().unwrap().camera.zoom;
    let framed = t.hero.documents.videos_mut().on_screen[&id].screen.size;
    let inset = 2.0 * WIDTHS[thick] * z;
    assert!(
        (bare[0] - framed[0] - inset).abs() < 1e-6 && (bare[1] - framed[1] - inset).abs() < 1e-6,
        "o tocador recua pela borda: {bare:?} -> {framed:?}"
    );
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    assert!(
        video(&t).unwrap().style().round,
        "o desfazer volta um passo"
    );
    // Activo, os NOSSOS controles ficam por baixo da caixa E da borda (a borda sai dela até ao alcance).
    // Uma borda GROSSA (mais que a folga da barra a qualquer zoom do teste): com a da barra, a folga dela
    // bastava e o teste não distinguia (mutação M14 sobreviveu).
    const THICK: f64 = 24.0;
    {
        let b = t.hero.documents.active_board_mut().unwrap();
        let mut el = b.doc.get(id).unwrap().clone();
        el.style_mut().stroke_width = THICK;
        let _ = ph2d_board_model::BoardOp::Put(el).apply(&mut b.doc);
    }
    t.paint();
    let p = on_video(&mut t, id, 0.5, 0.5);
    click(&mut t, p);
    t.hero.documents.videos_mut().status = Some(STATUS);
    t.paint();
    let [_, vy, _, vh] = video_rect(&mut t, id);
    let border_bottom = vy + vh + (1.5 * THICK * z) as f32;
    let (_, by) = ctl(&t, Ctl::Play);
    assert!(
        by > border_bottom,
        "a barra em {by}, a borda até {border_bottom}"
    );
    let r = t
        .hero
        .hit_index
        .rect_for(board_video_bar::node(Ctl::Play))
        .unwrap();
    assert!(
        r.y >= border_bottom,
        "a barra por cima da borda: {r:?} (a borda até {border_bottom})"
    );
}

/// ⭐ W20 («troca na mesma janela», o dono) — a sugestão do «Mais vídeos» troca o vídeo da janela (o link
/// canónico; a caixa fica) num passo de desfazer: o `Ctrl+Z` volta ao vídeo de antes.
#[test]
fn a_suggestion_swaps_the_video_in_its_window_and_undo_brings_it_back() {
    let mut t = T::new();
    assert!(paste(&mut t, LINK));
    let before = video(&t).unwrap();
    let other =
        ph2d_board_embed::youtube("https://www.youtube.com/watch?v=7CfDEnHxPWA&feature=endscreen")
            .unwrap();
    assert!(board_video::navigate(&mut t.hero, before.id, other.clone()));
    t.paint();
    let after = video(&t).unwrap();
    assert_eq!(after.embed().unwrap().video, other);
    assert_eq!(
        after.embed().unwrap().link,
        ph2d_board_embed::canonical_link(&other)
    );
    assert_eq!(
        [after.x, after.y, after.w, after.h],
        [before.x, before.y, before.w, before.h]
    );
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    assert_eq!(
        video(&t).unwrap().embed(),
        before.embed(),
        "o desfazer não voltou ao de antes"
    );
}

/// ⭐⭐ W20 (o dono: «além do CTRL+Z, setas <> de navegação dos vídeos tocados, e botão reset para limpar a
/// navegação e voltar ao vídeo original») — pelo despacho real sobre a NOSSA barra: sem navegação os três não
/// existem; depois de duas sugestões, ‹ volta, › avança (cada um um passo de desfazer, e o `Ctrl+Z` anda no
/// MESMO trilho); uma sugestão depois de recuar corta o que estava à frente (como um navegador); o «voltar ao
/// original» devolve o 1.º vídeo e os três somem.
#[test]
fn the_arrows_walk_the_watched_videos_and_reset_brings_the_original_back() {
    let mut t = T::new();
    let id = active(&mut t);
    let video_now = |t: &T| video(t).unwrap().embed().unwrap().video.clone();
    let original = video_now(&t);
    for c in [Ctl::Back, Ctl::Forward, Ctl::Reset] {
        assert!(
            t.find(board_video_bar::node(c)).is_none(),
            "{c:?} sem navegação"
        );
    }
    let yt = |s: &str| ph2d_board_embed::youtube(&format!("https://youtu.be/{s}")).unwrap();
    let (a, b, c) = (yt("7CfDEnHxPWA"), yt("dQw4w9WgXcQ"), yt("kJQP7kiw5Fk"));
    assert!(board_video::navigate(&mut t.hero, id, a.clone()));
    assert!(board_video::navigate(&mut t.hero, id, b.clone()));
    t.paint();
    assert_eq!(video_now(&t), b);
    tap(&mut t, Ctl::Back);
    assert_eq!(video_now(&t), a, "‹");
    tap(&mut t, Ctl::Back);
    assert_eq!(video_now(&t), original, "‹ até ao original");
    tap(&mut t, Ctl::Back);
    assert_eq!(video_now(&t), original, "‹ no início não faz nada");
    tap(&mut t, Ctl::Forward);
    assert_eq!(video_now(&t), a, "›");
    // O `Ctrl+Z` anda no mesmo trilho: desfaz o › e as setas seguem-no.
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    t.paint();
    assert_eq!(video_now(&t), original);
    tap(&mut t, Ctl::Forward);
    assert_eq!(video_now(&t), a);
    // Uma sugestão depois de recuar corta o que estava à frente (o `b`).
    assert!(board_video::navigate(&mut t.hero, id, c.clone()));
    t.paint();
    tap(&mut t, Ctl::Forward);
    assert_eq!(video_now(&t), c, "› no fim não faz nada");
    tap(&mut t, Ctl::Back);
    assert_eq!(video_now(&t), a, "o `b` saiu do trilho");
    tap(&mut t, Ctl::Reset);
    assert_eq!(video_now(&t), original, "voltar ao original");
    for c in [Ctl::Back, Ctl::Forward, Ctl::Reset] {
        assert!(
            t.find(board_video_bar::node(c)).is_none(),
            "{c:?} depois de limpar"
        );
    }
}

/// A barra nunca fica à frente do vídeo: por baixo, por cima se em baixo não couber, e nenhuma se não
/// couber em lado nenhum.
#[test]
fn the_bar_goes_under_or_over_the_video_never_on_it() {
    use crate::zones::Rect;
    let area = Rect::new(0.0, 0.0, 1000.0, 600.0);
    let time = 80.0;
    let under = layout(area, Rect::new(100.0, 100.0, 560.0, 315.0), time, false).unwrap();
    assert!(under.ctls.iter().all(|(_, r)| r.y >= 415.0));
    let over = layout(area, Rect::new(100.0, 250.0, 560.0, 340.0), time, false).unwrap();
    assert!(over.ctls.iter().all(|(_, r)| r.y + r.h <= 250.0));
    assert!(layout(area, Rect::new(0.0, 10.0, 1000.0, 580.0), time, false).is_none());
}
