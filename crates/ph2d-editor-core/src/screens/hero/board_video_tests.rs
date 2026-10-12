//! W18 — o vídeo pelo despacho REAL: colar um link do YouTube (Ctrl+V) faz nascer um vídeo; o clique no
//! play activa-o; activo e vivo, o rato e as teclas vão para o tocador e não para o quadro; clicar fora
//! desactiva; o botão «Video» pede o link à área de transferência; outro link continua texto.

use super::*;
use crate::screens::hero::board_video;
use ph2d_board_model::ElementId;
use crate::screens::hero::board_video_bar::{self, Ctl, layout};
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

fn click(t: &mut T, (x, y): (f32, f32)) {
    let (h, ts, c) = (&mut t.hero, &mut t.ts, &mut t.clock);
    let p = PointerButton::Primary;
    board_view::pointer(h, inp(ts, c), PointerKind::Down, p, x, y, true);
    board_view::pointer(h, inp(ts, c), PointerKind::Up, p, x, y, true);
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
    let mut ev = t.hero.handle_pointer(ptr(PointerKind::Down, x, y), &arena).to_vec();
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
    for c in [Ctl::Play, Ctl::Seek, Ctl::Mute, Ctl::Volume, Ctl::Fullscreen, Ctl::Outside] {
        let (x, y) = ctl(&t, c);
        assert!(y > vy + vh, "{c:?} em {x},{y}: à frente do vídeo {vx},{vy} {vw}×{vh}");
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
    assert_eq!(t.hero.documents.videos_mut().active, Some(id), "a barra não desactiva o vídeo");
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
    assert!(pl.live, "no ecrã inteiro o tocador está vivo: {:?}", pl.screen);
    let (_, by) = ctl(&t, Ctl::Play);
    assert!(by > y + h, "a barra à frente do vídeo");
    assert!(t.find(super::node(Item::Video)).is_none(), "as barras do quadro ficaram por baixo");
    // O preto à volta: não desactiva nem chega ao quadro.
    click(&mut t, (area.x + 2.0, area.y + 2.0));
    assert_eq!(t.hero.documents.videos_mut().active, Some(id));
    assert!(board_view::wheel(&mut t.hero, area.x + 2.0, area.y + 2.0, 1.0));
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

/// A barra nunca fica à frente do vídeo: por baixo, por cima se em baixo não couber, e nenhuma se não
/// couber em lado nenhum.
#[test]
fn the_bar_goes_under_or_over_the_video_never_on_it() {
    use crate::zones::Rect;
    let area = Rect::new(0.0, 0.0, 1000.0, 600.0);
    let time = 80.0;
    let under = layout(area, Rect::new(100.0, 100.0, 560.0, 315.0), time).unwrap();
    assert!(under.ctls.iter().all(|(_, r)| r.y >= 415.0));
    let over = layout(area, Rect::new(100.0, 250.0, 560.0, 340.0), time).unwrap();
    assert!(over.ctls.iter().all(|(_, r)| r.y + r.h <= 250.0));
    assert!(layout(area, Rect::new(0.0, 10.0, 1000.0, 580.0), time).is_none());
}
