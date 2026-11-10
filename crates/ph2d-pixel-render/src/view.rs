//! O sprite, o xadrez, a grelha e o mapa ecrã↔célula.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::checker::Checker;
use ph2d_pixel_core::{
    FrameId, LayerId, Onion, Purpose, Rect, Sprite, Stroke, compose_window_em,
    compose_with_onion_em, instante_do_quadro,
};
use ph2d_vector::{
    Affine, BezPath, Color, ImageQuality, Point, StableImage, Stroke as Pen, VectorScene,
};

/// O que a RAM guarda de frames já compostos para o tocar não recompor (o recurso é a RAM; além
/// disto um frame recompõe-se, ~65 ms em 1024² × 20 camadas nas faixas — `mede_compose`).
const FRAME_CACHE_BYTES: usize = 256 << 20;

/// A imagem composta de um sprite, refeita só quando a versão (`stamp`), o frame ou o papel vegetal
/// mudam (um id estável por imagem: o atlas do Vello reusa-a entre quadros).
#[derive(Default)]
pub struct SpriteImage {
    key: Option<(u64, FrameId, Option<Onion>)>,
    image: Option<StableImage>,
    checker: Checker,
    scratch: Vec<u8>,
    /// O retângulo do traço em curso na última composição (o próximo quadro do MESMO traço só
    /// recompõe a união dele com o novo).
    pending_rect: Option<ph2d_pixel_core::Rect>,
    /// O traço acabou e o documento ficou nesta versão: ela só difere da imagem na janela dele.
    settled: Option<u64>,
    /// Os frames desta versão já compostos (sem papel vegetal): navegar e tocar não recompõem.
    frames: BTreeMap<FrameId, (Arc<Vec<u8>>, StableImage)>,
    frames_stamp: Option<u64>,
    frames_bytes: usize,
    /// A camada activa, para o `Current layer only` do papel vegetal.
    ghost_layer: Option<LayerId>,
    /// O xadrez das preferências do documento.
    doc_checker: crate::prefs_view::DocChecker,
    /// O relógio do Play que os ciclos de cor somam ao instante do quadro (voltas inteiras).
    relogio: (u64, u64),
}

impl SpriteImage {
    /// Recompõe se `stamp` ou `frame` mudaram. `pending` é o traço em curso.
    pub fn refresh(
        &mut self,
        sprite: &Sprite,
        frame: FrameId,
        pending: Option<&Stroke>,
        stamp: u64,
    ) {
        self.refresh_with(sprite, frame, pending, stamp, None);
    }

    /// A imagem já é a desta versão, deste frame e deste papel vegetal: refazê-la não mudaria nada.
    #[must_use]
    pub fn is_current(&self, frame: FrameId, stamp: u64, onion: Option<&Onion>) -> bool {
        self.image.is_some() && self.key == Some((stamp, frame, onion.copied()))
    }

    /// O Play andou `dt_ms`: os ciclos de cor rodam também num sprite de UM quadro. O `t` de cada
    /// quadro é `instante + voltas inteiras da animação tocadas` (o mesmo índice por quadro dentro
    /// de uma volta); só refaz a imagem quando a volta muda. Chamar só com ciclos no documento.
    pub fn avanca_ciclo(&mut self, sprite: &Sprite, dt_ms: u32) {
        let total: u64 = sprite
            .frames()
            .iter()
            .map(|f| u64::from(f.duration_ms))
            .sum();
        let tocado = self.relogio.0.wrapping_add(u64::from(dt_ms));
        let voltas = tocado - tocado % total.max(1);
        if voltas != self.relogio.1 {
            (self.key, self.frames_stamp) = (None, None);
        }
        self.relogio = (tocado, voltas);
    }

    /// A camada activa (o `Current layer only` do papel vegetal); mudá-la refaz a imagem.
    pub fn set_ghost_layer(&mut self, layer: Option<LayerId>) {
        if self.ghost_layer != layer {
            self.ghost_layer = layer;
            self.key = None;
        }
    }

    /// O xadrez das preferências `c` do documento por baixo do sprite `w × h`.
    pub fn draw_doc_checker(
        &mut self,
        scene: &mut VectorScene,
        size: (u32, u32),
        c: &ph2d_pixel_core::Checker,
        to_screen: Affine,
    ) {
        self.doc_checker.draw_under(scene, size, c, to_screen);
    }

    /// O [`Self::refresh`] com o papel vegetal (`None` = desligado).
    pub fn refresh_with(
        &mut self,
        sprite: &Sprite,
        frame: FrameId,
        pending: Option<&Stroke>,
        stamp: u64,
        onion: Option<&Onion>,
    ) {
        let key = (stamp, frame, onion.copied());
        if self.key == Some(key) && self.image.is_some() {
            return;
        }
        if self.frames_stamp != Some(stamp) {
            self.frames.clear();
            self.frames_bytes = 0;
            self.frames_stamp = Some(stamp);
        }
        let same_view = self.key.is_some_and(|(_, f, o)| f == frame && o == key.2);
        let (w, h) = (sprite.width(), sprite.height());
        let cheio = w as usize * h as usize * 4;
        let cur = pending.map(Stroke::rect);
        let assentou = self.settled.take() == Some(stamp);
        if pending.is_none()
            && onion.is_none()
            && !(assentou && same_view)
            && let Some((px, img)) = self.frames.get(&frame)
        {
            self.scratch.clone_from(px);
            self.image = Some(img.clone());
            (self.pending_rect, self.key) = (None, Some(key));
            return;
        }
        // Durante um traço o documento não muda: só a janela do traço (a de antes ∪ a de agora, a
        // forma pode encolher) se recompõe. Medido: 512² × 20 camadas cheio ≫ a janela de um pincel.
        let pronto = same_view && self.scratch.len() == cheio && self.image.is_some();
        match (self.pending_rect, cur) {
            (Some(a), None) if assentou && pronto => {
                let o = onion.map(|o| (o, self.ghost_layer));
                region_into(sprite, frame, None, a, o, &mut self.scratch, self.relogio.1);
            }
            (Some(a), Some(b)) if pronto => {
                let o = onion.map(|o| (o, self.ghost_layer));
                let r = a.union(b);
                region_into(
                    sprite,
                    frame,
                    pending,
                    r,
                    o,
                    &mut self.scratch,
                    self.relogio.1,
                );
            }
            _ => {
                let o = onion.map(|o| (o, self.ghost_layer));
                self.scratch = compose_bands_em(sprite, frame, pending, o, self.relogio.1);
            }
        }
        self.pending_rect = cur;
        let px = Arc::new(self.scratch.clone());
        self.image = StableImage::from_rgba(Arc::clone(&px), w, h);
        if pending.is_none()
            && onion.is_none()
            && let Some(img) = &self.image
            && self.frames_bytes + cheio <= FRAME_CACHE_BYTES
        {
            self.frames_bytes += cheio;
            self.frames.insert(frame, (px, img.clone()));
        }
        self.key = Some(key);
    }

    /// O traço em curso acabou (gravado ou largado) e o documento está na versão `stamp`: a próxima
    /// composição dessa versão só refaz a janela do traço. Qualquer outra versão recompõe tudo.
    pub fn settle(&mut self, stamp: u64) {
        self.settled = Some(stamp);
    }

    /// O RGBA da célula `(x, y)` na imagem composta, para `w` células de largura.
    #[must_use]
    pub fn rgba_at(&self, w: u32, x: u32, y: u32) -> Option<[u8; 4]> {
        let i = (y as usize * w as usize + x as usize) * 4;
        self.scratch.get(i..i + 4)?.try_into().ok()
    }

    /// Desenha o sprite (nearest).
    pub fn draw(&self, scene: &mut VectorScene, to_screen: Affine) {
        if let Some(img) = &self.image {
            scene.draw_stable_image_transformed(img, to_screen, ImageQuality::Low);
        }
    }

    /// O xadrez por baixo de um sprite `w × h` (uma imagem de uma célula por casa, esticada).
    pub fn draw_checker_under(
        &mut self,
        scene: &mut VectorScene,
        w: u32,
        h: u32,
        to_screen: Affine,
    ) {
        self.checker.draw_under(scene, w, h, to_screen);
    }
}

/// A janela `region` composta (com o papel vegetal, se houver), com os ciclos de cor no instante do
/// quadro ([`instante_do_quadro`]): função do documento e do quadro, logo o `key` da imagem e a
/// cache por quadro continuam a valer. O Pixel Sprite na cena passa por aqui com o quadro que mostra.
fn window(
    sprite: &Sprite,
    frame: FrameId,
    pending: Option<&Stroke>,
    region: Rect,
    onion: Option<(&Onion, Option<LayerId>)>,
    voltas: u64,
) -> Vec<u8> {
    let t = instante_do_quadro(sprite, frame) + voltas;
    match onion {
        Some((o, l)) => {
            compose_with_onion_em(sprite, frame, pending, region, o, (None, l), Some(t))
        }
        None => compose_window_em(sprite, frame, pending, region, Purpose::View, t),
    }
}

/// Recompõe `region` dentro de `out` (o sprite inteiro).
fn region_into(
    sprite: &Sprite,
    frame: FrameId,
    pending: Option<&Stroke>,
    region: Rect,
    onion: Option<(&Onion, Option<LayerId>)>,
    out: &mut [u8],
    voltas: u64,
) {
    let region = region.intersect(sprite.bounds());
    if region.is_empty() {
        return;
    }
    let buf = window(sprite, frame, pending, region, onion, voltas);
    let (w, rw) = (sprite.width() as usize, region.w as usize * 4);
    for (y, row) in buf.chunks_exact(rw).enumerate() {
        let o = ((region.y as usize + y) * w + region.x as usize) * 4;
        out[o..o + rw].copy_from_slice(row);
    }
}

/// A composição cheia em faixas de linhas, uma por núcleo da fatia da linha (metade dos da
/// máquina). Medido (`mede_compose`, load ~6): 512² × 20 camadas 123 ms num núcleo, 16,7 ms em 8.
/// `onion` leva a camada activa (o `Current layer only`).
#[must_use]
pub fn compose_bands(
    sprite: &Sprite,
    frame: FrameId,
    pending: Option<&Stroke>,
    onion: Option<(&Onion, Option<LayerId>)>,
) -> Vec<u8> {
    compose_bands_em(sprite, frame, pending, onion, 0)
}

/// O [`compose_bands`] com as `voltas` do Play somadas ao instante dos ciclos de cor.
#[must_use]
pub fn compose_bands_em(
    sprite: &Sprite,
    frame: FrameId,
    pending: Option<&Stroke>,
    onion: Option<(&Onion, Option<LayerId>)>,
    voltas: u64,
) -> Vec<u8> {
    let (w, h) = (sprite.width(), sprite.height());
    let n = std::thread::available_parallelism().map_or(1, |n| (n.get() / 2).max(1)) as u32;
    let alto = h.div_ceil(n.min(h));
    let bands: Vec<Rect> = (0..h.div_ceil(alto))
        .map(|k| Rect::new(0, (k * alto) as i32, w, alto.min(h - k * alto)))
        .collect();
    if bands.len() == 1 {
        return window(sprite, frame, pending, bands[0], onion, voltas);
    }
    let parts: Vec<Vec<u8>> = std::thread::scope(|sc| {
        let hs: Vec<_> = bands
            .iter()
            .map(|&r| sc.spawn(move || window(sprite, frame, pending, r, onion, voltas)))
            .collect();
        hs.into_iter()
            .map(|h| h.join().unwrap_or_else(|e| std::panic::resume_unwind(e)))
            .collect()
    });
    parts.concat()
}

/// ⭐ **A porta ÚNICA imagem → mundo** (P14 §4: «onde está a origem?»): o ponto `pivo` (píxeis do
/// sprite, Y para baixo — `ph2d_pixel_cena::pivo_px`, o centro sem pivô) fica na posição do
/// `Transform`; 1 célula = `1 / ppm` metros, y da imagem para baixo e do mundo para cima. Desenho,
/// clique, onion, Tiled Mode, chrome e miniatura passam todos por aqui.
#[must_use]
pub fn image_to_world(
    pivo: (f64, f64),
    ppm: f64,
    translation: (f64, f64),
    rotation: f64,
    scale: (f64, f64),
    skew: (f64, f64),
) -> Affine {
    let ppm = ppm.max(f64::MIN_POSITIVE);
    let img_to_local =
        Affine::scale_non_uniform(1.0 / ppm, -1.0 / ppm) * Affine::translate((-pivo.0, -pivo.1));
    let local_to_world = Affine::translate(translation)
        * Affine::rotate(rotation)
        * Affine::skew(skew.0.tan(), skew.1.tan())
        * Affine::scale_non_uniform(scale.0, scale.1);
    local_to_world * img_to_local
}

/// A célula sob o ponto de ecrã `p`; `None` se o afim não se inverte (escala zero).
#[must_use]
pub fn cell_at(to_screen: Affine, p: (f64, f64)) -> Option<(i32, i32)> {
    if to_screen.determinant().abs() < 1e-12 {
        return None;
    }
    let q = to_screen.inverse() * Point::new(p.0, p.1);
    let (x, y) = (q.x.floor(), q.y.floor());
    if !(x.is_finite() && y.is_finite())
        || x.abs() > f64::from(i32::MAX)
        || y.abs() > f64::from(i32::MAX)
    {
        return None;
    }
    Some((x as i32, y as i32))
}

/// Píxeis de ecrã por célula (a raiz do determinante: o lado médio de uma célula).
#[must_use]
pub fn px_per_cell(to_screen: Affine) -> f64 {
    to_screen.determinant().abs().sqrt()
}

fn line(path: &mut BezPath, to_screen: Affine, a: (f64, f64), b: (f64, f64)) {
    path.move_to(to_screen * Point::new(a.0, a.1));
    path.line_to(to_screen * Point::new(b.0, b.1));
}

/// A grelha dos tiles de uma camada de tilemap, alinhada à fase `(fx, fy)` do mapa.
pub fn draw_tile_grid(
    scene: &mut VectorScene,
    (w, h): (u32, u32),
    (tw, th): (u32, u32),
    (fx, fy): (i32, i32),
    to_screen: Affine,
    color: Color,
    width_px: f64,
) {
    let mut path = BezPath::new();
    let (fw, fh) = (f64::from(w), f64::from(h));
    let primeira = |f: i32, t: u32| f.rem_euclid(t.max(1) as i32) as u32;
    for x in (primeira(fx, tw)..w).step_by(tw.max(1) as usize) {
        let x = f64::from(x);
        line(&mut path, to_screen, (x, 0.0), (x, fh));
    }
    for y in (primeira(fy, th)..h).step_by(th.max(1) as usize) {
        let y = f64::from(y);
        line(&mut path, to_screen, (0.0, y), (fw, y));
    }
    scene
        .inner_mut()
        .stroke(&Pen::new(width_px), Affine::IDENTITY, color, None, &path);
}

/// A grelha de pixels (linhas entre células) de um sprite `w × h`, `width_px` de espessura de ecrã.
pub fn draw_pixel_grid(
    scene: &mut VectorScene,
    w: u32,
    h: u32,
    to_screen: Affine,
    color: Color,
    width_px: f64,
) {
    let mut path = BezPath::new();
    let (fw, fh) = (f64::from(w), f64::from(h));
    for x in 1..w {
        line(
            &mut path,
            to_screen,
            (f64::from(x), 0.0),
            (f64::from(x), fh),
        );
    }
    for y in 1..h {
        line(
            &mut path,
            to_screen,
            (0.0, f64::from(y)),
            (fw, f64::from(y)),
        );
    }
    scene
        .inner_mut()
        .stroke(&Pen::new(width_px), Affine::IDENTITY, color, None, &path);
}

/// Polígonos fechados em coordenadas do sprite: a grelha e o realce de iso e hex.
pub fn draw_polygons(
    scene: &mut VectorScene,
    poligonos: &[Vec<(f64, f64)>],
    to_screen: Affine,
    color: Color,
    width_px: f64,
) {
    let mut path = BezPath::new();
    for p in poligonos {
        let mut pontos = p.iter().map(|&(x, y)| to_screen * Point::new(x, y));
        let Some(a) = pontos.next() else { continue };
        path.move_to(a);
        for b in pontos {
            path.line_to(b);
        }
        path.close_path();
    }
    scene
        .inner_mut()
        .stroke(&Pen::new(width_px), Affine::IDENTITY, color, None, &path);
}

/// A borda do canvas.
pub fn draw_border(
    scene: &mut VectorScene,
    w: u32,
    h: u32,
    to_screen: Affine,
    color: Color,
    width_px: f64,
) {
    let (fw, fh) = (f64::from(w), f64::from(h));
    let mut path = BezPath::new();
    path.move_to(to_screen * Point::new(0.0, 0.0));
    path.line_to(to_screen * Point::new(fw, 0.0));
    path.line_to(to_screen * Point::new(fw, fh));
    path.line_to(to_screen * Point::new(0.0, fh));
    path.close_path();
    stroke_halo(scene, &path, width_px, color);
}

/// O contorno da célula `(x, y)` — onde o lápis vai pintar.
pub fn draw_cell_cursor(
    scene: &mut VectorScene,
    cell: (i32, i32),
    to_screen: Affine,
    color: Color,
    width_px: f64,
) {
    let (x, y) = (f64::from(cell.0), f64::from(cell.1));
    let mut path = BezPath::new();
    path.move_to(to_screen * Point::new(x, y));
    path.line_to(to_screen * Point::new(x + 1.0, y));
    path.line_to(to_screen * Point::new(x + 1.0, y + 1.0));
    path.line_to(to_screen * Point::new(x, y + 1.0));
    path.close_path();
    stroke_halo(scene, &path, width_px, color);
}

/// Os eixos da simetria sobre um canvas `w × h`: `axis2` em MEIAS células (o do espelho H é
/// vertical, o do V horizontal); as diagonais passam pelo centro e cortam-se na borda.
#[allow(clippy::too_many_arguments)]
pub fn draw_symmetry_axes(
    scene: &mut VectorScene,
    (w, h): (u32, u32),
    to_screen: Affine,
    axis2: (i32, i32),
    (sym_h, sym_v, d1, d2): (bool, bool, bool, bool),
    color: Color,
    width_px: f64,
) {
    let (fw, fh) = (f64::from(w), f64::from(h));
    let (cx, cy) = (f64::from(axis2.0) / 2.0, f64::from(axis2.1) / 2.0);
    let mut path = BezPath::new();
    let mut seg = |a: Point, b: Point| {
        path.move_to(to_screen * a);
        path.line_to(to_screen * b);
    };
    if sym_h {
        seg(Point::new(cx, 0.0), Point::new(cx, fh));
    }
    if sym_v {
        seg(Point::new(0.0, cy), Point::new(fw, cy));
    }
    if d1 {
        let (lo, hi) = ((-cx).max(-cy), (fw - cx).min(fh - cy));
        if lo < hi {
            seg(Point::new(cx + lo, cy + lo), Point::new(cx + hi, cy + hi));
        }
    }
    if d2 {
        let (lo, hi) = ((-cx).max(cy - fh), (fw - cx).min(cy));
        if lo < hi {
            seg(Point::new(cx + lo, cy - lo), Point::new(cx + hi, cy - hi));
        }
    }
    stroke_halo(scene, &path, width_px, color);
}

/// A cor de dentro do cursor de célula: clara em QUALQUER tema, porque o fundo é a arte, não o
/// chrome (a mesma razão do xadrez).
pub const CURSOR_CORE: Color = Color::new([1.0, 1.0, 1.0, 1.0]); // LITERAL-COLOR-OK: núcleo claro do cursor sobre a arte (invariante ao tema)
/// O anel escuro por fora de todo traço do Draw: com o núcleo claro, um dos dois contrasta com
/// qualquer cor por baixo (dono 25/10: o cursor sumia sobre uma cor parecida).
pub(crate) const HALO: Color = Color::new([0.0, 0.0, 0.0, 0.85]); // LITERAL-COLOR-OK: anel de contraste sobre a arte (invariante ao tema)
/// Quanto o anel sai de cada lado do núcleo, em píxeis de ECRÃ.
pub(crate) const HALO_PX: f64 = 1.0; // LITERAL-PX-OK: espessura do anel de contraste, em píxeis de ecrã

/// O traço com o anel escuro por baixo e o núcleo `core` por cima. As arestas das células caem em
/// píxeis INTEIROS do ecrã (o zoom do Draw alinha-as), e só uma largura PAR centrada numa aresta
/// cobre píxeis inteiros — com 1 px o núcleo espalhava-se a 50 % por dois píxeis e virava cinzento
/// sobre o anel. ⇒ núcleo `2 × width_px`, anel `HALO_PX` de cada lado.
pub(crate) fn stroke_halo(scene: &mut VectorScene, path: &BezPath, width_px: f64, core: Color) {
    let nucleo = 2.0 * width_px;
    let s = scene.inner_mut();
    let anel = Pen::new(nucleo + 2.0 * HALO_PX);
    s.stroke(&anel, Affine::IDENTITY, HALO, None, path);
    s.stroke(&Pen::new(nucleo), Affine::IDENTITY, core, None, path);
}

#[cfg(test)]
mod tests {
    use super::SpriteImage;
    use ph2d_pixel_core::layers;
    use ph2d_pixel_core::{NewSprite, Paint, Sprite, Stroke, apply, compose_rgba};

    /// ⭐ A janela do traço e a do «assentar» dão os MESMOS bytes que a composição cheia — senão
    /// o artista veria um retângulo velho que só aparece enquanto edita.
    #[test]
    fn the_stroke_window_and_the_settle_give_the_bytes_of_the_full_compose() {
        let mut s = Sprite::new(NewSprite::default()).expect("s");
        let base = s.layers()[0].id;
        let (op, top) = layers::new_layer(&mut s, base).expect("l");
        apply(&mut s, op).expect("l");
        let op = layers::edit(&s, top, |l| {
            l.opacity = 140;
            l.blend = ph2d_pixel_core::blend::BlendMode::Screen;
        })
        .expect("e");
        apply(&mut s, op).expect("e");
        let f = s.frames()[0].id;
        let mut img = SpriteImage::default();
        img.refresh(&s, f, None, 1);
        let mut st = Stroke::begin(
            &s,
            top,
            f,
            s.value_of_rgba([200, 40, 90, 255]),
            Paint::default(),
            (2, 2),
        );
        let full = |s: &Sprite, p: Option<&Stroke>| {
            let mut out = Vec::new();
            compose_rgba(s, f, p, &mut out);
            out
        };
        let mut stamp = 2;
        for (x, y) in [(9, 4), (20, 25), (3, 30)] {
            st.to(&s, x, y);
            img.refresh(&s, f, Some(&st), stamp);
            assert_eq!(img.scratch, full(&s, Some(&st)), "o traço em curso");
            stamp += 1;
        }
        let op = st.op(&s).expect("op");
        apply(&mut s, op).expect("commit");
        img.settle(stamp);
        img.refresh(&s, f, None, stamp);
        assert_eq!(img.scratch, full(&s, None), "o traço gravado");
    }

    /// As faixas, o papel vegetal e a imagem guardada de cada frame dão os bytes da composição
    /// de uma vez; uma edição (outra versão) nunca mostra a imagem guardada de antes.
    #[test]
    fn bands_onion_and_the_frame_cache_give_the_bytes_of_the_single_compose() {
        use ph2d_pixel_core::frames::{self, NewFrame};
        use ph2d_pixel_core::{Onion, Op, Patch, Rect, compose_with_onion};
        let mut s = Sprite::new(NewSprite {
            width: 37,
            height: 29,
            ..NewSprite::default()
        })
        .expect("s");
        let l = s.layers()[0].id;
        let f0 = s.frames()[0].id;
        let r = s.bounds();
        let px: Vec<u8> = (0..r.area())
            .flat_map(|k| [(k * 7) as u8, (k * 3) as u8, 90, (k * 11) as u8])
            .collect();
        apply(
            &mut s,
            Op::Patch(Patch {
                layer: l,
                frame: f0,
                bbox: Some(r),
                rect: r,
                pixels: px,
            }),
        )
        .expect("p");
        let (op, f1) = frames::new_frame(&mut s, f0, NewFrame::Empty).expect("f");
        apply(&mut s, op).expect("f");
        let one = Rect::new(3, 3, 1, 1);
        apply(
            &mut s,
            Op::Patch(Patch {
                layer: l,
                frame: f1,
                bbox: Some(one),
                rect: one,
                pixels: vec![9, 9, 9, 255],
            }),
        )
        .expect("p");
        let full = |s: &Sprite, f| {
            let mut out = Vec::new();
            compose_rgba(s, f, None, &mut out);
            out
        };
        let mut img = SpriteImage::default();
        img.refresh(&s, f0, None, 1);
        assert_eq!(img.scratch, full(&s, f0), "faixas, frame 0");
        img.refresh(&s, f1, None, 1);
        assert_eq!(img.scratch, full(&s, f1), "faixas, frame 1");
        img.refresh(&s, f0, None, 1);
        assert_eq!(img.scratch, full(&s, f0), "a imagem guardada do frame 0");
        let o = Onion {
            tint: true,
            ..Onion::default()
        };
        img.refresh_with(&s, f1, None, 1, Some(&o));
        assert_eq!(
            img.scratch,
            compose_with_onion(&s, f1, None, s.bounds(), &o, None),
            "papel vegetal"
        );
        apply(
            &mut s,
            Op::Patch(Patch {
                layer: l,
                frame: f0,
                bbox: Some(r),
                rect: one,
                pixels: vec![1, 2, 3, 255],
            }),
        )
        .expect("p");
        img.refresh(&s, f0, None, 2);
        assert_eq!(img.scratch, full(&s, f0), "outra versão recompõe");
    }
}
