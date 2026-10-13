//! W19 ([ADR-0185] §W19) — **os NOSSOS controles do vídeo activo**: tocar/pausar, o tempo, a barra de
//! tempo, o mudo e o volume, o NOSSO ecrã inteiro e «Abrir no YouTube». O tocador vem sem os dele
//! (`controls=0`, `fs=0`); as regras do YouTube proíbem qualquer coisa À FRENTE do tocador, por isso a
//! barra desenha-se POR BAIXO da caixa do vídeo, fora dela (ou por cima, se em baixo não couber) — e
//! some se não couber em lado nenhum. No ecrã inteiro o vídeo enche a área do quadro com a proporção dele,
//! e a barra fica por baixo do vídeo, dentro da área.
//!
//! As ordens vão para `Videos::commands` (o app executa-as); o estado vem de `Videos::status`.
//!
//! [ADR-0185]: ../../../../../docs/architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md

use ph2d_a11y::NodeId;
use ph2d_board_embed::{OnScreen, live_allowed};
use ph2d_board_render::{Placement, PlayerStatus, VideoCommand};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, TypeToken};
use ph2d_vector::{Affine, ImageQuality, Point, VectorScene};

use super::HeroScreen;
use super::float_bar::{self, Btn};
use crate::icons::IconId;
use crate::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{paint_icon, paint_text_centered, rect_to_vello, resolve};
use crate::widget::{ButtonState, Slider, SliderOrientation, SliderState, paint_slider};
use crate::zones::Rect;

const SALT: u64 = 0xb0a2_d7ab_71de_0000;
/// Os vídeos de um diapositivo da apresentação (um id por elemento).
const SHOW_SALT: u64 = 0xb0a2_d7ab_72de_0000;

/// O id clicável do vídeo `id` na apresentação.
#[must_use]
pub fn show_node(id: ph2d_board_model::ElementId) -> NodeId {
    NodeId(SHOW_SALT ^ id.0)
}

/// Um controlo da barra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ctl {
    Play,
    Seek,
    Mute,
    Volume,
    Fullscreen,
    Outside,
}

const ALL: [Ctl; 6] = [
    Ctl::Play,
    Ctl::Seek,
    Ctl::Mute,
    Ctl::Volume,
    Ctl::Fullscreen,
    Ctl::Outside,
];

/// O id do controlo `c`.
#[must_use]
pub fn node(c: Ctl) -> NodeId {
    let i = ALL.iter().position(|x| *x == c).unwrap_or_default() as u64;
    NodeId(SALT ^ (i + 1))
}

fn ctl_of(id: NodeId) -> Option<Ctl> {
    let i = (id.0 ^ SALT).checked_sub(1)?;
    ALL.get(usize::try_from(i).ok()?).copied()
}

fn is_slider(c: Ctl) -> bool {
    matches!(c, Ctl::Seek | Ctl::Volume)
}

/// Regista os controlos (pintado ⇒ clicável). Chamado pelo `document_tabs::populate`.
pub fn populate(store: &mut WidgetStore) {
    for c in ALL {
        let state = if is_slider(c) {
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Horizontal,
            }
        } else {
            InteractiveState::Button {
                state: ButtonState::Normal,
            }
        };
        store.register(node(c), state);
    }
}

/// A dica de cada controlo no estado em que está.
fn tooltip(c: Ctl, s: PlayerStatus, full: bool) -> &'static str {
    match c {
        Ctl::Play if s.playing => "board.video.pause",
        Ctl::Play => "board.video.play",
        Ctl::Seek => "board.video.seek",
        Ctl::Mute if s.muted => "board.video.unmute",
        Ctl::Mute => "board.video.mute",
        Ctl::Volume => "board.video.volume",
        Ctl::Fullscreen if full => "board.video.exit_fullscreen",
        Ctl::Fullscreen => "board.video.fullscreen",
        Ctl::Outside => "board.video.open_outside",
    }
}

/// `m:ss` (ou `h:mm:ss`).
#[must_use]
pub fn clock(secs: f64) -> String {
    let s = secs.max(0.0) as u64;
    let (h, m, s) = (s / 3600, s / 60 % 60, s % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

fn time_label(s: PlayerStatus) -> String {
    ph2d_i18n::tr("board.video.time")
        .replace("{now}", &clock(s.time))
        .replace("{total}", &clock(s.duration))
}

/// Onde cada coisa da barra cai.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub ctls: Vec<(Ctl, Rect)>,
    pub time: Rect,
}

/// ⭐ A barra para um vídeo no rectângulo de ecrã `video`, dentro de `area`: por baixo dele, ou por cima
/// se em baixo não couber — NUNCA à frente dele; `None` se não couber (`time_w` = a largura do tempo).
#[must_use]
pub fn layout(area: Rect, video: Rect, time_w: f32) -> Option<Layout> {
    let s = float_bar::button_px();
    let (pad, gap) = (Spacing::Xs.px(), float_bar::gap());
    let vol_w = Spacing::Xl4.px();
    let seek_min = Spacing::Xl4.px();
    // play · tempo · barra · mudo · volume · ecrã inteiro · abrir fora
    let fixed = 4.0 * s + time_w + vol_w + 6.0 * gap + 2.0 * pad; // LITERAL-PX-OK: 4 botões, 6 vãos, 2 margens
    let w = video.w.max(fixed + seek_min);
    if w > area.w {
        return None;
    }
    let h = s + 2.0 * pad; // LITERAL-PX-OK: as duas margens
    let below = video.y + video.h + pad;
    let above = video.y - pad - h;
    let y = if below + h <= area.y + area.h {
        below
    } else if above >= area.y {
        above
    } else {
        return None;
    };
    let x = (video.x + video.w / 2.0 - w / 2.0)
        .max(area.x)
        .min(area.x + area.w - w);
    let seek_w = w - fixed;
    let mut cx = x + pad;
    let mut put = |width: f32| {
        let r = Rect::new(cx, y + pad, width, s);
        cx += width + gap;
        r
    };
    let play = put(s);
    let time = put(time_w);
    let seek = put(seek_w);
    let mute = put(s);
    let volume = put(vol_w);
    let full = put(s);
    let out = put(s);
    Some(Layout {
        ctls: vec![
            (Ctl::Play, play),
            (Ctl::Seek, seek),
            (Ctl::Mute, mute),
            (Ctl::Volume, volume),
            (Ctl::Fullscreen, full),
            (Ctl::Outside, out),
        ],
        time,
    })
}

/// No ecrã inteiro: o rectângulo do vídeo (a proporção `aspect` = largura/altura) e o da barra por baixo,
/// dentro de `area`.
#[must_use]
pub fn fullscreen_rect(area: Rect, aspect: f32) -> Rect {
    let bar = float_bar::button_px() + 3.0 * Spacing::Xs.px(); // LITERAL-PX-OK: as duas margens da barra e o vão até ao vídeo
    let (aw, ah) = (area.w, (area.h - bar).max(1.0));
    let (w, h) = if aw / ah > aspect {
        (ah * aspect, ah)
    } else {
        (aw, aw / aspect)
    };
    Rect::new(area.x + (aw - w) / 2.0, area.y + (ah - h) / 2.0, w, h)
}

/// O rectângulo de ecrã de um vídeo pela colocação dele (sem rodar).
fn screen_rect(pl: &Placement, el_w: f64, el_h: f64) -> Rect {
    let a = pl.to_screen * Point::new(0.0, 0.0);
    let b = pl.to_screen * Point::new(el_w, el_h);
    Rect::new(
        a.x.min(b.x) as f32,
        a.y.min(b.y) as f32,
        (b.x - a.x).abs() as f32,
        (b.y - a.y).abs() as f32,
    )
}

/// ⭐ Pinta a barra do vídeo activo (e, no ecrã inteiro, o vídeo a encher a área). `area` = a do quadro.
pub fn paint(hero: &mut HeroScreen, area: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    let theme = hero.theme;
    let Some((board, live)) = hero.documents.active_parts() else {
        return;
    };
    let vids = &mut live.render_cache.videos;
    let Some(id) = vids.active else { return };
    let Some(el) = board.doc.get(id) else { return };
    let (el_w, el_h) = (el.w, el.h);
    let full = vids.fullscreen;
    let video = if full {
        let r = fullscreen_rect(area, (el_w / el_h.max(1e-9)) as f32);
        scene.fill_rect(rect_to_vello(area), resolve(ColorToken::Bg0, theme));
        // O vídeo é o do ecrã inteiro: o rato e o tamanho do tocador seguem ESTA colocação.
        let k = f64::from(r.w) / el_w.max(1e-9);
        let size = [f64::from(r.w), f64::from(r.h)];
        let screen = OnScreen {
            size,
            angle: 0.0,
            covered: false,
            visible: size,
        };
        let pl = Placement {
            screen,
            live: live_allowed(screen),
            to_screen: Affine::translate((f64::from(r.x), f64::from(r.y))) * Affine::scale(k),
        };
        vids.on_screen.insert(id, pl);
        if let Some((img, sz)) = &vids.frame {
            let (iw, ih) = (f64::from(sz[0]), f64::from(sz[1]));
            let f = (size[0] / iw).min(size[1] / ih);
            let off = ((size[0] - iw * f) / 2.0, (size[1] - ih * f) / 2.0);
            let m = Affine::translate((f64::from(r.x) + off.0, f64::from(r.y) + off.1))
                * Affine::scale(f);
            scene.draw_recorded_image(img, m, ImageQuality::Medium, None);
        }
        r
    } else {
        match (vids.on_screen.get(&id), el.embed()) {
            // W20: a barra fica fora da CAIXA e da borda dela (o recuo do tocador é o alcance da moldura,
            // que sai da caixa no máximo isso).
            (Some(pl), Some(e)) if pl.live => {
                let (reach, inner) = ph2d_board_render::player_in_box(el, e);
                let tb = pl.to_screen * inner.inverse();
                let a = tb * Point::new(-reach, -reach);
                let b = tb * Point::new(el_w + reach, el_h + reach);
                Rect::new(
                    a.x.min(b.x) as f32,
                    a.y.min(b.y) as f32,
                    (b.x - a.x).abs() as f32,
                    (b.y - a.y).abs() as f32,
                )
            }
            _ => return,
        }
    };
    let Some(status) = vids.status else { return };
    let font = TypeToken::Sm.px();
    let label = time_label(status);
    let widest = ph2d_i18n::tr("board.video.time")
        .replace("{now}", &clock(status.duration))
        .replace("{total}", &clock(status.duration));
    let time_w = ts.prefix_width(&widest, font) + Spacing::Sm.px();
    let Some(lay) = layout(area, video, time_w) else {
        return;
    };
    float_bar::paint_panel(
        scene,
        lay.ctls.iter().map(|(_, r)| *r).chain([lay.time]),
        theme,
    );
    paint_text_centered(
        ts,
        scene,
        &label,
        lay.time,
        font,
        resolve(ColorToken::Text2, theme),
    );
    let store = &mut hero.store;
    let hit = &mut hero.hit_index;
    for (c, r) in lay.ctls {
        let id = node(c);
        store.set_tooltip(id, ph2d_i18n::tr(tooltip(c, status, full)));
        if is_slider(c) {
            let want = match c {
                Ctl::Seek if status.duration > 0.0 => (status.time / status.duration) as f32,
                Ctl::Volume if status.muted => 0.0,
                Ctl::Volume => f32::from(status.volume) / 100.0, // LITERAL-PX-OK: a escala 0..=100 da API
                _ => 0.0,
            };
            // A arrastar, o valor é o da mão; senão o do tocador.
            let dragging = store
                .slider(id)
                .is_some_and(|(s, _)| s == SliderState::Dragging);
            if !dragging {
                store.set_slider_value(id, want);
            }
            let v = store.slider(id).map_or(want, |(_, v)| v);
            hit.register(id, r);
            let mut sl = Slider::new(id, ph2d_i18n::tr(tooltip(c, status, full)))
                .accent(true)
                .visual(store.slider_visual(id));
            sl.set_value(v);
            paint_slider(&sl, r, scene, theme);
            continue;
        }
        let fg = float_bar::paint_button(
            scene,
            hit,
            store,
            theme,
            Btn {
                id,
                rect: r,
                on: false,
                tint: ColorToken::Text1,
            },
        );
        let icon = match c {
            Ctl::Play if status.playing => IconId::Pause,
            Ctl::Play => IconId::Play,
            Ctl::Mute if status.muted || status.volume == 0 => IconId::VolumeX,
            Ctl::Mute => IconId::Audio,
            Ctl::Fullscreen if full => IconId::Minimize,
            Ctl::Fullscreen => IconId::Maximize,
            _ => IconId::Open,
        };
        paint_icon(
            scene,
            icon,
            float_bar::inner(r),
            fg,
            StrokeToken::Default.px(),
        );
    }
}

/// ⭐ W19 — na apresentação: cada vídeo do diapositivo é clicável por cima da sala (o 1.º clique activa-o
/// e toca, os seguintes tocam e pausam) e o activo tem a barra, por cima de tudo. `viewport` = a janela.
pub fn paint_show(
    hero: &mut HeroScreen,
    viewport: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
) {
    if let Some((board, live)) = hero.documents.active_parts() {
        for (id, pl) in &live.render_cache.videos.on_screen {
            if let Some(el) = board.doc.get(*id) {
                let node = show_node(*id);
                // Um botão por vídeo, registado uma vez, como os botões do quadro (que o `populate`
                // regista). Medido: registá-lo a cada quadro também não perde o clique (a mutação
                // sobrevive ao teste que repinta entre o Down e o Up) — é só não refazer estado.
                if hero.store.button_state(node).is_none() {
                    hero.store.register(
                        node,
                        InteractiveState::Button {
                            state: ButtonState::Normal,
                        },
                    );
                }
                hero.hit_index.register(node, screen_rect(pl, el.w, el.h));
            }
        }
    }
    paint(hero, viewport, scene, ts);
}

/// O clique num vídeo da apresentação: activa-o, ou (já activo) toca/pausa.
fn show_click(hero: &mut HeroScreen, id: NodeId) -> bool {
    let Some((_, live)) = hero.documents.active_parts() else {
        return false;
    };
    let vids = &mut live.render_cache.videos;
    let Some(el) = vids.on_screen.keys().copied().find(|k| show_node(*k) == id) else {
        return false;
    };
    if vids.active == Some(el) {
        vids.commands.push(VideoCommand::TogglePlay);
    } else {
        vids.activate(el);
    }
    true
}

/// ⭐ Os cliques e os arrastos da barra. Corre no pré-despacho (ids derivados).
pub fn apply_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let (id, slid) = match event {
        WidgetEvent::Click(id) | WidgetEvent::DoubleClick(id) => (id, false),
        WidgetEvent::ValueChanged(id) => (id, true),
        _ => return false,
    };
    if !slid && show_click(hero, id) {
        return true;
    }
    let Some(c) = ctl_of(id) else { return false };
    let v = hero.store.slider(id).map(|(_, v)| v);
    let Some((_, live)) = hero.documents.active_parts() else {
        return true;
    };
    let vids = &mut live.render_cache.videos;
    let Some(status) = vids.status else {
        return true;
    };
    let cmd = match (c, slid, v) {
        (Ctl::Seek, true, Some(v)) => Some(VideoCommand::Seek(f64::from(v) * status.duration)),
        (Ctl::Volume, true, Some(v)) => Some(VideoCommand::Volume((v * 100.0).round() as u8)), // LITERAL-PX-OK: a escala 0..=100 da API
        (Ctl::Play, false, _) => Some(VideoCommand::TogglePlay),
        (Ctl::Mute, false, _) => Some(VideoCommand::ToggleMute),
        (Ctl::Outside, false, _) => {
            vids.fullscreen = false;
            Some(VideoCommand::OpenOutside)
        }
        (Ctl::Fullscreen, false, _) => {
            vids.fullscreen = !vids.fullscreen;
            None
        }
        _ => None,
    };
    vids.commands.extend(cmd);
    true
}
