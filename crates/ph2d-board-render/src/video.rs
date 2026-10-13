//! W18 ([ADR-0185]) — **desenhar um VÍDEO do quadro**: o quadro do tocador (o navegador da pessoa,
//! projectado) quando o tocador o tem e as regras do YouTube o deixam vivo ([`ph2d_board_embed::live_allowed`]:
//! descoberto, sem rodar, tocador ≥ 200×200 px, algum pedaço à vista); senão o último quadro que se viu, ou a miniatura
//! (cortada a encher a caixa), ou o lugar vazio — sempre com o símbolo de play quando não está vivo.
//! Quem toca, busca a miniatura e encaminha o rato é o app (`ph2d-app-board`); aqui só se desenha e se
//! diz ONDE cada vídeo ficou ([`Videos::on_screen`]).
//!
//! [ADR-0185]: ../../../docs/architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Mutex, OnceLock};

use ph2d_board_embed::{OnScreen, live_allowed};
use ph2d_board_geom::{corner_radius, to_world};
use ph2d_board_model::{BoardDoc, Element, ElementId, Embed, Video};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{
    Affine, Brush, Circle, ImageQuality, Point, Rect, RoundedRect, Shape as _, StableImage,
    VectorScene, VelloBlend,
};

use crate::{RenderCache, token};

/// A miniatura de um vídeo, para qualquer desenho do processo (ecrã, apresentação, export).
#[derive(Clone)]
pub enum Thumb {
    Ready(StableImage, [u32; 2]),
    /// A rede não a deu (o lugar vazio, e o app avisa).
    Missing,
}

static THUMBS: OnceLock<Mutex<BTreeMap<String, Thumb>>> = OnceLock::new();

fn thumbs() -> std::sync::MutexGuard<'static, BTreeMap<String, Thumb>> {
    THUMBS
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// A chave de um vídeo (a da miniatura).
#[must_use]
pub fn video_key(v: &Video) -> String {
    match v {
        Video::YouTube { id, .. } => format!("youtube:{id}"),
    }
}

/// O app põe a miniatura (ou a falta dela) de um vídeo.
pub fn set_thumbnail(key: String, t: Thumb) {
    thumbs().insert(key, t);
}

/// Um gesto que o editor manda ao tocador, em píxeis do TOCADOR (`0..tamanho do vídeo no ecrã`).
#[derive(Clone, Debug, PartialEq)]
pub enum PlayerInput {
    /// `button`: 0 esquerdo, 1 do meio, 2 direito.
    Down {
        x: f64,
        y: f64,
        button: u8,
        clicks: u32,
    },
    Up {
        x: f64,
        y: f64,
        button: u8,
        clicks: u32,
    },
    Move {
        x: f64,
        y: f64,
    },
}

/// O estado do tocador do vídeo activo, para os NOSSOS controles (o app põe-no a cada quadro).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerStatus {
    /// Segundos.
    pub time: f64,
    pub duration: f64,
    /// A tocar (ou a encher): o botão mostra «pausa».
    pub playing: bool,
    /// `0..=100`.
    pub volume: u8,
    pub muted: bool,
}

/// Uma ordem dos NOSSOS controles para o tocador (o app executa-a).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VideoCommand {
    TogglePlay,
    /// Salta para estes segundos.
    Seek(f64),
    /// `0..=100` (e tira o mudo).
    Volume(u8),
    ToggleMute,
    /// «Abrir no YouTube»: o link no navegador do sistema, no tempo em que está (e pausa aqui).
    OpenOutside,
}

/// Onde um vídeo ficou no ecrã neste desenho.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub screen: OnScreen,
    /// O tocador está vivo (activo E as regras deixam).
    pub live: bool,
    /// Local (`0..w`, `0..h` do elemento) → ecrã.
    pub to_screen: Affine,
}

/// ⭐ O estado do vídeo no desenho (um por [`RenderCache`]): o activado, o quadro do tocador, o que o
/// editor lhe manda e onde cada vídeo ficou.
#[derive(Default)]
pub struct Videos {
    /// O vídeo ACTIVADO: o rato dentro dele e as teclas são do tocador.
    pub active: Option<ElementId>,
    /// O vídeo que o tocador tem (um de cada vez). W20: fica depois de desactivado e continua a tocar
    /// enquanto as regras o deixam vivo — desactivar tira o rato, não a imagem (o oráculo).
    pub player: Option<ElementId>,
    /// O quadro do tocador mais novo (o app põe-no).
    pub frame: Option<(StableImage, [u32; 2])>,
    /// O que o editor mandou ao tocador (o app esvazia).
    pub inbox: Vec<PlayerInput>,
    /// Onde cada vídeo à vista ficou no último desenho do ecrã.
    pub on_screen: BTreeMap<ElementId, Placement>,
    /// As miniaturas que faltam (o app busca-as).
    pub wanted: BTreeSet<String>,
    /// Os vídeos à vista: a chave de cada um (para o app saber o vídeo do activado).
    pub videos: BTreeMap<ElementId, Video>,
    /// O estado do tocador activo (`None` = ainda não há tocador).
    pub status: Option<PlayerStatus>,
    /// As ordens dos nossos controles (o app esvazia).
    pub commands: Vec<VideoCommand>,
    /// O NOSSO ecrã inteiro: o vídeo activo enche a área do quadro, com a proporção dele.
    pub fullscreen: bool,
    /// A apresentação está a desenhar (o modo limpo do export): os vídeos dela registam-se e tocam.
    pub presenting: bool,
}

impl Videos {
    /// Activa `id`: o tocador passa a ter este vídeo (o quadro do anterior esquece-se) e o rato e as teclas
    /// passam a ser dele.
    pub fn activate(&mut self, id: ElementId) {
        if self.player != Some(id) {
            self.frame = None;
        }
        self.active = Some(id);
        self.player = Some(id);
    }

    /// O ponto de ecrã `p` em píxeis do tocador de `id` (`None` = fora dele, ou não está à vista).
    #[must_use]
    pub fn player_point(&self, id: ElementId, el: &Element, p: [f64; 2]) -> Option<[f64; 2]> {
        let pl = self.on_screen.get(&id)?;
        let l = pl.to_screen.inverse() * Point::new(p[0], p[1]);
        let inside = l.x >= 0.0 && l.y >= 0.0 && l.x <= el.w && l.y <= el.h;
        inside.then(|| {
            [
                l.x / el.w.max(1e-9) * pl.screen.size[0],
                l.y / el.h.max(1e-9) * pl.screen.size[1],
            ]
        })
    }

    /// `p` (ecrã) cai no símbolo de play de `id`.
    #[must_use]
    pub fn on_play(&self, id: ElementId, el: &Element, p: [f64; 2]) -> bool {
        let Some(pl) = self.on_screen.get(&id) else {
            return false;
        };
        let l = pl.to_screen.inverse() * Point::new(p[0], p[1]);
        let r = play_radius(el);
        (l.x - el.w / 2.0).hypot(l.y - el.h / 2.0) <= r
    }

    pub(crate) fn begin_frame(&mut self) {
        self.on_screen.clear();
        self.videos.clear();
    }
}

fn play_radius(el: &Element) -> f64 {
    el.w.min(el.h) * 0.12
}

/// Algum elemento desenhado DEPOIS de `el` (por cima, os frames não contam) toca a caixa dele.
fn covered(doc: &BoardDoc, el: &Element) -> bool {
    let [x0, y0, x1, y1] = el.aabb();
    doc.live().any(|o| {
        if o.id == el.id || o.as_frame().is_some() || o.z <= el.z {
            return false;
        }
        let [a0, b0, a1, b1] = o.aabb();
        a0 < x1 && a1 > x0 && b0 < y1 && b1 > y0
    })
}

/// ⭐ W20 — a caixa e o TOCADOR dentro dela: sem borda são a mesma; com borda o tocador recua para dentro
/// dela pelo alcance da moldura (o Excalidraw recua o iframe pela largura do traço, `saidas/w20` E2) — o
/// traço, que é centrado no contorno, nunca fica por cima do vídeo (as regras do YouTube). Devolve o
/// recuo (unidades da caixa) e o afim do local do tocador, normalizado a `0..el.w × 0..el.h`, à caixa.
#[must_use]
pub fn player_in_box(el: &Element, e: &Embed) -> (f64, Affine) {
    let st = &e.style;
    let framed = st.stroke.is_some() && st.stroke_width > 0.0;
    let inset = if framed {
        crate::image_frame::reach(el, st)
    } else {
        0.0
    };
    let (w, h) = (el.w.max(f64::EPSILON), el.h.max(f64::EPSILON));
    let k = [
        (el.w - 2.0 * inset).max(0.0) / w,
        (el.h - 2.0 * inset).max(0.0) / h,
    ];
    (
        inset,
        Affine::translate((inset, inset)) * Affine::scale_non_uniform(k[0], k[1]),
    )
}

/// Pinta o vídeo `e` de `el` (`v` = mundo → ecrã; `area` = a vista). W20: com o estilo de um rectângulo —
/// o fundo POR TRÁS do tocador (nunca o tapa), o tocador recuado ([`player_in_box`]) e a moldura das
/// formas no contorno; a opacidade é do conjunto.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint(
    scene: &mut VectorScene,
    cache: &mut RenderCache,
    doc: &BoardDoc,
    el: &Element,
    e: &Embed,
    v: Affine,
    area: [f64; 4],
    theme: Theme,
    board: u64,
) {
    let tb = v * to_world(el);
    let (inset, inner) = player_in_box(el, e);
    let t = tb * inner;
    let corners =
        [[0.0, 0.0], [el.w, 0.0], [0.0, el.h], [el.w, el.h]].map(|[x, y]| t * Point::new(x, y));
    // A parte à vista: a caixa de ecrã dos cantos cortada pela vista (sem rodar é o retângulo exacto; rodado
    // o tocador não está vivo de qualquer forma).
    let [x, y, w, h] = area;
    let span = |a: fn(&Point) -> f64, lo: f64, hi: f64| {
        let (mn, mx) = corners
            .iter()
            .map(a)
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(m, n), v| {
                (m.min(v), n.max(v))
            });
        (mx.min(hi) - mn.max(lo)).max(0.0)
    };
    let screen = OnScreen {
        size: [
            (corners[1] - corners[0]).hypot(),
            (corners[2] - corners[0]).hypot(),
        ],
        angle: el.angle,
        covered: covered(doc, el),
        visible: [span(|c| c.x, x, x + w), span(|c| c.y, y, y + h)],
    };
    let on_screen = cache.export.is_none() || cache.videos.presenting;
    let active = on_screen && cache.videos.player == Some(el.id);
    let live = active && live_allowed(screen);
    if on_screen {
        cache.videos.on_screen.insert(
            el.id,
            Placement {
                screen,
                live,
                to_screen: t,
            },
        );
        cache.videos.videos.insert(el.id, e.video.clone());
    }
    let st = &e.style;
    let alpha = f32::from(st.opacity.min(100)) / 100.0;
    if alpha < 1.0 {
        let [x0, y0, x1, y1] = el.aabb();
        let pad = crate::image_frame::reach(el, st);
        let p0 = v * Point::new(x0 - pad, y0 - pad);
        let p1 = v * Point::new(x1 + pad, y1 + pad);
        scene.push_object_layer(&Rect::from_points(p0, p1), VelloBlend::default(), alpha);
    }
    let outer = crate::image::box_path(el, st.round);
    scene.push_layer_shape(VelloBlend::default(), 1.0, tb, &outer);
    let bg = st
        .fill
        .map_or_else(|| token(ColorToken::Bg3, theme), crate::doc_color);
    scene.fill_path(&outer, &Brush::Solid(bg), tb);
    // O tocador recortado no SEU rectângulo (os cantos concêntricos aos da caixa: o traço, metade para
    // dentro, fica a ≥ meio traço dele).
    if inset > 0.0 {
        let r = if st.round {
            corner_radius(el.w.min(el.h))
        } else {
            0.0
        };
        let inner_box = RoundedRect::new(inset, inset, el.w - inset, el.h - inset, r).to_path(0.1);
        scene.push_layer_shape(VelloBlend::default(), 1.0, tb, &inner_box);
    }
    // O quadro do tocador (vivo, ou o último que se viu ao deixar de o estar); senão a miniatura.
    let frame = cache.videos.frame.clone().filter(|_| active);
    let key = video_key(&e.video);
    let thumb = thumbs().get(&key).cloned();
    match (frame, thumb) {
        (Some((img, size)), _) => fit(scene, &img, size, el, t),
        (None, Some(Thumb::Ready(img, size))) => cover(scene, &img, size, el, t),
        (None, Some(Thumb::Missing)) => {}
        (None, None) => {
            cache.videos.wanted.insert(key);
        }
    }
    if inset > 0.0 {
        scene.pop_layer();
    }
    scene.pop_layer();
    if !live {
        play_mark(scene, el, t, theme);
    }
    crate::image_frame::paint(scene, cache, el, st, tb, board);
    if alpha < 1.0 {
        scene.pop_layer();
    }
}

/// O quadro do tocador INTEIRO na caixa, com faixas se a proporção for outra (como o tocador faz) — nunca
/// esticado (W19: o ecrã inteiro do tocador mandava 4:3 para uma caixa 16:9). Um quadro de outro tamanho
/// (o tocador ainda não seguiu o zoom) escala-se por igual.
fn fit(scene: &mut VectorScene, img: &StableImage, size: [u32; 2], el: &Element, t: Affine) {
    let (iw, ih) = (f64::from(size[0]), f64::from(size[1]));
    let k = (el.w / iw).min(el.h / ih);
    let off = [(el.w - iw * k) / 2.0, (el.h - ih * k) / 2.0];
    let m = t * Affine::translate((off[0], off[1])) * Affine::scale(k);
    scene.draw_recorded_image(img, m, ImageQuality::Medium, None);
}

/// A miniatura a ENCHER a caixa, cortada ao meio (as faixas pretas do `hqdefault` 4:3 saem num 16:9).
fn cover(scene: &mut VectorScene, img: &StableImage, size: [u32; 2], el: &Element, t: Affine) {
    let (iw, ih) = (f64::from(size[0]), f64::from(size[1]));
    let k = (el.w / iw).max(el.h / ih);
    let off = [(el.w - iw * k) / 2.0, (el.h - ih * k) / 2.0];
    let m = t * Affine::translate((off[0], off[1])) * Affine::scale(k);
    scene.draw_recorded_image(img, m, ImageQuality::Medium, None);
}

/// O símbolo de play: o círculo do perigo (o vermelho de «vídeo») com o triângulo, ao meio.
fn play_mark(scene: &mut VectorScene, el: &Element, t: Affine, theme: Theme) {
    let r = play_radius(el);
    if r <= 0.0 {
        return;
    }
    let c = Point::new(el.w / 2.0, el.h / 2.0);
    scene.fill_path(
        &Circle::new(c, r).to_path(0.1),
        &Brush::Solid(token(ColorToken::Danger, theme)),
        t,
    );
    let mut tri = ph2d_vector::BezPath::new();
    tri.move_to((c.x - r * 0.35, c.y - r * 0.45));
    tri.line_to((c.x + r * 0.5, c.y));
    tri.line_to((c.x - r * 0.35, c.y + r * 0.45));
    tri.close_path();
    scene.fill_path(&tri, &Brush::Solid(token(ColorToken::Text1, theme)), t);
}

#[cfg(test)]
#[path = "video_tests.rs"]
mod tests;
