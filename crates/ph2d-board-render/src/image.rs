//! W14 — **desenhar uma IMAGEM do quadro**: a lei do oráculo (Excalidraw 0.18.1,
//! `ferramentas/excalidraw_oracle/saidas/w14/`): a imagem natural (ou o recorte dela) estica-se à
//! caixa, o espelho vira à volta do centro, os cantos redondos são os do rectângulo
//! ([`ph2d_board_geom::corner_radius`]), a opacidade compõe a imagem inteira; sem ficheiro (ainda a
//! chegar, ou ilegível) é um cinzento com o ícone de imagem — e no export não sai (o SVG dele).
//!
//! Os píxeis vêm de uma [`Images`]: descodifica FORA do fio do desenho (o primeiro quadro mostra o
//! cinzento) e guarda uma pirâmide de metades; o desenho pede o nível com a resolução do ECRÃ — o
//! atlas do Vello é UM quadrado partilhado por todas as imagens à vista (`MAX_ATLAS_SIZE = 8192` em
//! `vello_encoding-0.10.0/src/image_cache.rs:10`), e a imagem inteira de 20 fotografias não cabia.
//!
//! W15: a pirâmide é por ficheiro E filtro (a cinzenta é outra entrada, [`crate::gray`]); um SVG não
//! se reduz por metades — cada nível rasteriza-se ao tamanho DELE (nítido a qualquer zoom, até ao
//! lado do atlas); e a MOLDURA é o contorno de um rectângulo com o estilo da imagem (a lei das formas:
//! traço, tracejado, rascunho, cantos), por cima, fora do recorte.

use std::collections::BTreeMap;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, OnceLock};

use ph2d_board_geom::{corner_radius, to_world};
use ph2d_board_model::{Element, FileId, Image, ImageFile, ImageFilter};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{
    Affine, Brush, Circle, ImageQuality, ImageSource, Point, Rect, RoundedRect, Shape as _,
    StableImage, Stroke, VectorScene, VelloBlend,
};

use crate::{RenderCache, token};

/// Os píxeis de uma imagem descodificada: RGBA de 8 bits, alfa reto, sRGB.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pixels {
    pub size: [u32; 2],
    pub rgba: Vec<u8>,
}

/// O descodificador (os bytes de um ficheiro → os píxeis). Quem desenha não descodifica: recebe-o de
/// quem tem a porta de entrada das imagens (`ph2d-app-board`). `at` = o tamanho a que se rasteriza um
/// ficheiro VECTORIAL (`image/svg+xml`); um raster ignora-o e dá o natural.
pub type Decode = Arc<dyn Fn(&[u8], Option<[u32; 2]>) -> Option<Pixels> + Send + Sync>;

/// A chave de uma pirâmide: o ficheiro e o filtro (a mesma fotografia a cores e a cinzento são duas).
type Key = (FileId, ImageFilter);

/// O MIME de um ficheiro que se rasteriza a cada tamanho.
const VECTOR_MIME: &str = "image/svg+xml";

/// O descodificador do APP, instalado uma vez ao arrancar ([`install_decoder`]): toda [`Images`] o usa
/// quando precisa de descodificar (não quando nasce — uma cache feita antes da instalação também o vê).
static DECODER: OnceLock<Decode> = OnceLock::new();

/// Instala o descodificador do app (a 2.ª chamada não muda nada). Quem tem a porta de entrada das
/// imagens chama-o ao arrancar.
pub fn install_decoder(decode: Decode) {
    let _ = DECODER.set(decode);
}

/// O maior lado de um nível que o Vello aceita: o atlas dele é um quadrado deste lado.
const ATLAS_SIDE: u32 = 8192;
/// O maior lado do nível que FICA sempre (o que se desenha enquanto o fino volta a descodificar):
/// ⚠️ política de memória, não limite — `128² × 4 B` = 64 KB por imagem.
const THUMB_SIDE: u32 = 128;
/// Quantos quadros um nível fino sobrevive sem ser desenhado.
const KEEP_FRAMES: u64 = 120;

/// A pirâmide de uma imagem: do maior (≤ [`ATLAS_SIDE`]) ao miniatura, cada nível metade do anterior.
struct Entry {
    levels: Vec<Level>,
    /// O nível mais fino pedido desde a última descodificação: ao chegar ela, os mais finos do que
    /// ele não ficam (uma fotografia de 8192² vista a 500 px não guarda os 256 MiB do 1.º nível).
    want: usize,
    /// Um raster: a pirâmide inteira a descodificar.
    pending: bool,
    failed: bool,
    /// Um SVG: cada nível pede-se sozinho ([`Level::asked`]).
    vector: bool,
}

struct Level {
    size: [u32; 2],
    image: Option<StableImage>,
    used: u64,
    asked: bool,
}

type Levels = Vec<(StableImage, [u32; 2])>;

/// O que um fio acabou: a pirâmide de um raster, ou UM nível de um SVG (o índice).
enum Out {
    Pyramid(Option<Levels>),
    Level(usize, Option<(StableImage, [u32; 2])>),
}

type Done = (Key, Out);

/// ⭐ **As imagens descodificadas** do desenho, por ficheiro (os mesmos bytes = a mesma entrada, em
/// qualquer quadro). Uma por [`crate::RenderCache`].
pub struct Images {
    decode: Option<Decode>,
    entries: BTreeMap<Key, Entry>,
    tx: Sender<Done>,
    rx: Receiver<Done>,
    /// Os trabalhos por fazer, partilhados pelos fios (criados à primeira vez).
    jobs: Option<Sender<Job>>,
    frame: u64,
}

struct Job {
    key: Key,
    bytes: Arc<[u8]>,
    /// `None` = a pirâmide de um raster; `Some` = um nível de um SVG.
    at: Option<(usize, [u32; 2])>,
}

impl Default for Images {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            decode: None,
            entries: BTreeMap::new(),
            tx,
            rx,
            jobs: None,
            frame: 0,
        }
    }
}

/// Os tamanhos da pirâmide de uma imagem `natural`: o 1.º cabe no atlas, cada um metade do anterior
/// (arredondada para cima), até um que caiba em [`THUMB_SIDE`]. Um SVG começa no lado do atlas (a
/// proporção dele), por pequeno que seja o natural: é vector.
fn level_sizes(natural: [u32; 2], vector: bool) -> Vec<[u32; 2]> {
    let mut s = natural.map(|v| v.max(1));
    if vector {
        let k = f64::from(ATLAS_SIDE) / f64::from(s[0].max(s[1]));
        s = s.map(|v| ((f64::from(v) * k).round() as u32).clamp(1, ATLAS_SIDE));
    }
    while s[0].max(s[1]) > ATLAS_SIDE {
        s = s.map(|v| v.div_ceil(2));
    }
    let mut out = vec![s];
    while s[0].max(s[1]) > THUMB_SIDE {
        s = s.map(|v| v.div_ceil(2));
        out.push(s);
    }
    out
}

/// Metade de `src` (`w × h`, RGBA reto) para `to`: a média de cada 2×2 PESADA pelo alfa (um píxel
/// transparente não escurece a borda), a última coluna/linha ímpar repetida.
fn halve(src: &[u8], [w, h]: [u32; 2], to: [u32; 2]) -> Vec<u8> {
    let (w, h) = (w as usize, h as usize);
    let mut out = vec![0u8; to[0] as usize * to[1] as usize * 4];
    for y in 0..to[1] as usize {
        for x in 0..to[0] as usize {
            let mut acc = [0u32; 4];
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let (sx, sy) = ((2 * x + dx).min(w - 1), (2 * y + dy).min(h - 1));
                let p = &src[(sy * w + sx) * 4..][..4];
                let a = u32::from(p[3]);
                for c in 0..3 {
                    acc[c] += u32::from(p[c]) * a;
                }
                acc[3] += a;
            }
            let o = &mut out[(y * to[0] as usize + x) * 4..][..4];
            for c in 0..3 {
                o[c] = (acc[c] + acc[3] / 2).checked_div(acc[3]).unwrap_or(0) as u8;
            }
            o[3] = ((acc[3] + 2) / 4) as u8;
        }
    }
    out
}

/// A pirâmide inteira de uns píxeis (o 1.º nível já reduzido ao atlas, se preciso).
fn pyramid(px: Pixels) -> Option<Levels> {
    let sizes = level_sizes(px.size, false);
    let at0 = px.size;
    if px.rgba.len() != at0[0] as usize * at0[1] as usize * 4 {
        return None;
    }
    // Cada nível é o `Arc` do anterior meio: nenhuma cópia dos píxeis.
    let (mut cur, mut at) = (Arc::new(px.rgba), at0);
    let mut out = Vec::with_capacity(sizes.len());
    for s in sizes {
        while at != s {
            let next = at.map(|v| v.div_ceil(2));
            cur = Arc::new(halve(&cur, at, next));
            at = next;
        }
        let img = StableImage::from_rgba(cur.clone(), s[0], s[1])?;
        out.push((img, s));
    }
    Some(out)
}

/// Os píxeis de `bytes` (a `at`, se é um SVG) com o filtro aplicado.
fn decode_filtered(
    decode: &Decode,
    bytes: &[u8],
    at: Option<[u32; 2]>,
    filter: ImageFilter,
) -> Option<Pixels> {
    let mut px = decode(bytes, at)?;
    match filter {
        ImageFilter::Original => {}
        ImageFilter::Gray => crate::gray::to_gray(&mut px.rgba),
    }
    Some(px)
}

/// Descodifica: a pirâmide de um raster, ou o nível `at` de um SVG. Um descodificador que entra em
/// pânico é uma imagem que falhou (o fio continua, e a imagem não fica no cinzento à espera para
/// sempre).
fn run(decode: &Decode, job: &Job) -> Out {
    let filter = job.key.1;
    let ok = |f: &dyn Fn() -> Option<Levels>| {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
            .ok()
            .flatten()
    };
    match job.at {
        None => Out::Pyramid(ok(&|| {
            decode_filtered(decode, &job.bytes, None, filter).and_then(pyramid)
        })),
        Some((k, size)) => {
            let one = ok(&|| {
                let px = decode_filtered(decode, &job.bytes, Some(size), filter)?;
                (px.size == size && px.rgba.len() == size[0] as usize * size[1] as usize * 4)
                    .then(|| StableImage::from_rgba(Arc::new(px.rgba), size[0], size[1]))
                    .flatten()
                    .map(|img| vec![(img, size)])
            });
            Out::Level(k, one.and_then(|mut v| v.pop()))
        }
    }
}

impl Images {
    /// Outro descodificador que o do app (sem nenhum as imagens ficam no cinzento).
    pub fn set_decoder(&mut self, decode: Decode) {
        self.decode = Some(decode);
    }

    /// O descodificador desta cache, ou o do app.
    fn decoder(&self) -> Option<Decode> {
        self.decode.clone().or_else(|| DECODER.get().cloned())
    }

    /// Há imagens a descodificar: quem desenha só a pedido pede outro quadro (a shell desenha
    /// sempre, `ControlFlow::Poll`).
    #[must_use]
    pub fn pending(&self) -> bool {
        self.entries
            .values()
            .any(|e| e.pending || e.levels.iter().any(|l| l.asked))
    }

    /// Recolhe o que os fios acabaram.
    fn collect(&mut self) {
        while let Ok((key, out)) = self.rx.try_recv() {
            self.adopt(key, out);
        }
    }

    fn adopt(&mut self, key: Key, out: Out) {
        let frame = self.frame;
        let Some(e) = self.entries.get_mut(&key) else {
            return;
        };
        let levels = match out {
            Out::Pyramid(levels) => levels,
            Out::Level(k, one) => {
                let Some(slot) = e.levels.get_mut(k) else {
                    return;
                };
                slot.asked = false;
                match one {
                    Some((img, size)) => {
                        *slot = Level {
                            size,
                            image: Some(img),
                            used: frame,
                            asked: false,
                        };
                    }
                    None => e.failed = true,
                }
                return;
            }
        };
        e.pending = false;
        let (want, last) = (e.want, e.levels.len().saturating_sub(1));
        e.want = usize::MAX;
        match levels {
            Some(ls) if ls.len() == e.levels.len() => {
                for (k, (slot, (img, size))) in e.levels.iter_mut().zip(ls).enumerate() {
                    if slot.image.is_none() && (k >= want || k == last) {
                        *slot = Level {
                            size,
                            image: Some(img),
                            used: frame,
                            asked: false,
                        };
                    }
                }
            }
            _ => e.failed = true,
        }
    }

    fn spawn(&mut self, job: Job) {
        let Some(decode) = self.decoder() else {
            return;
        };
        let jobs = self.jobs.get_or_insert_with(|| {
            let (jt, jr) = channel::<Job>();
            let jr = Arc::new(Mutex::new(jr));
            // Metade da máquina: a outra é do desenho e do resto do app.
            let n = std::thread::available_parallelism().map_or(1, |n| (n.get() / 2).max(1));
            for _ in 0..n {
                let (jr, tx, decode) = (jr.clone(), self.tx.clone(), decode.clone());
                std::thread::spawn(move || {
                    loop {
                        let job = match jr.lock() {
                            Ok(r) => r.recv(),
                            Err(_) => return,
                        };
                        let Ok(job) = job else { return };
                        let out = run(&decode, &job);
                        if tx.send((job.key, out)).is_err() {
                            return;
                        }
                    }
                });
            }
            jt
        });
        let _ = jobs.send(job);
    }

    /// O nível a desenhar de `file` (com o filtro `filter`) para `need` píxeis de ecrã na largura e
    /// na altura da imagem NATURAL inteira: o mais pequeno que os tem; se ainda não há, o mais
    /// próximo que houver (e o certo pede-se). `sync` = descodifica JÁ (o export não espera por
    /// quadros). `None` = nada ainda.
    pub(crate) fn level(
        &mut self,
        id: FileId,
        filter: ImageFilter,
        file: &ImageFile,
        need: [f64; 2],
        sync: bool,
    ) -> Option<(StableImage, [u32; 2])> {
        let key = (id, filter);
        let frame = self.frame;
        let decoder = self.decoder();
        let vector = file.mime() == Some(VECTOR_MIME);
        let e = self.entries.entry(key).or_insert_with(|| Entry {
            want: usize::MAX,
            levels: level_sizes(file.size, vector)
                .into_iter()
                .map(|size| Level {
                    size,
                    image: None,
                    used: 0,
                    asked: false,
                })
                .collect(),
            pending: false,
            failed: false,
            vector,
        });
        if e.failed {
            return None;
        }
        let want = e
            .levels
            .iter()
            .rposition(|l| f64::from(l.size[0]) >= need[0] && f64::from(l.size[1]) >= need[1])
            .unwrap_or(0);
        if e.levels[want].image.is_none() {
            e.want = e.want.min(want);
        }
        let bytes = &file.bytes.0;
        if e.vector {
            // O nível pedido e a miniatura (o que fica quando o fino se esquece); o export só o seu.
            let thumb = e.levels.len() - 1;
            let mut ask: Vec<(usize, [u32; 2])> = [want, thumb]
                .into_iter()
                .filter(|&k| e.levels[k].image.is_none() && !e.levels[k].asked)
                .filter(|&k| !sync || k == want)
                .map(|k| (k, e.levels[k].size))
                .collect();
            ask.dedup_by_key(|a| a.0);
            if !sync && decoder.is_some() {
                for (k, _) in &ask {
                    e.levels[*k].asked = true;
                }
            }
            for (k, size) in ask {
                let job = Job {
                    key,
                    bytes: bytes.clone(),
                    at: Some((k, size)),
                };
                match &decoder {
                    Some(d) if sync => {
                        let out = run(d, &job);
                        self.adopt(key, out);
                    }
                    Some(_) => self.spawn(job),
                    None => {}
                }
            }
        } else if e.levels[want].image.is_none() && !e.pending {
            let job = Job {
                key,
                bytes: bytes.clone(),
                at: None,
            };
            if sync {
                if let Some(d) = &decoder {
                    let out = run(d, &job);
                    self.adopt(key, out);
                }
            } else if decoder.is_some() {
                e.pending = true;
                self.spawn(job);
            }
        }
        self.pick(key, want, frame)
    }

    /// O nível `want` se há, senão o mais fino mais perto (nítido), senão o mais grosso mais perto.
    fn pick(&mut self, key: Key, want: usize, frame: u64) -> Option<(StableImage, [u32; 2])> {
        let e = self.entries.get_mut(&key)?;
        let pick = std::iter::once(want)
            .chain((0..want).rev())
            .chain(want + 1..e.levels.len())
            .find(|&k| e.levels[k].image.is_some())?;
        let l = &mut e.levels[pick];
        l.used = frame;
        Some((l.image.clone()?, l.size))
    }

    /// Fim de um quadro: recolhe os acabados e esquece os níveis finos que ninguém desenhou há
    /// [`KEEP_FRAMES`] (a miniatura fica).
    pub(crate) fn end_frame(&mut self) {
        self.collect();
        self.frame += 1;
        if self.frame.is_multiple_of(KEEP_FRAMES) {
            let cut = self.frame.saturating_sub(KEEP_FRAMES);
            for e in self.entries.values_mut() {
                let last = e.levels.len() - 1;
                for l in &mut e.levels[..last] {
                    if l.used < cut {
                        l.image = None;
                    }
                }
            }
        }
    }

    /// Recolhe os acabados (no início de um desenho).
    pub(crate) fn begin_frame(&mut self) {
        self.collect();
    }
}

/// O afim do píxel do NÍVEL (`level` de tamanho, de uma imagem natural `natural`) ao local da caixa
/// do elemento: o recorte estica-se à caixa, e o espelho vira à volta do centro.
#[must_use]
pub fn level_to_box(el: &Element, img: &Image, natural: [u32; 2], level: [u32; 2]) -> Affine {
    let [sx, sy, sw, sh] = img.source(natural);
    let flip = Affine::new([
        if img.flip[0] { -1.0 } else { 1.0 },
        0.0,
        0.0,
        if img.flip[1] { -1.0 } else { 1.0 },
        if img.flip[0] { el.w } else { 0.0 },
        if img.flip[1] { el.h } else { 0.0 },
    ]);
    flip * Affine::scale_non_uniform(el.w / sw.max(1e-9), el.h / sh.max(1e-9))
        * Affine::translate((-sx, -sy))
        * Affine::scale_non_uniform(
            f64::from(natural[0]) / f64::from(level[0].max(1)),
            f64::from(natural[1]) / f64::from(level[1].max(1)),
        )
}

/// A caixa local do elemento, com os cantos do rectângulo redondo se `round`.
fn box_path(el: &Element, round: bool) -> ph2d_vector::BezPath {
    let r = if round {
        corner_radius(el.w.min(el.h))
    } else {
        0.0
    };
    RoundedRect::new(0.0, 0.0, el.w, el.h, r).to_path(0.1)
}

/// Os píxeis por ponto de um SVG num export vectorial (o PDF mede em pontos, 1/72 pol): os 300 ppp da
/// impressão. Um raster vai inteiro (o leitor amplia o que quiser).
const PRINT_PX_PER_PT: f64 = 300.0 / 72.0;

/// Pinta a imagem `img` de `el` (`v` = mundo → ecrã) e a moldura dela. `file` = o ficheiro, se chegou;
/// `board` = o quadro (a chave do rascunho da moldura).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint(
    scene: &mut VectorScene,
    cache: &mut RenderCache,
    el: &Element,
    img: &Image,
    file: Option<&ImageFile>,
    v: Affine,
    theme: Theme,
    board: u64,
) {
    let export = cache.export.map(|e| e.vector);
    let t = v * to_world(el);
    let st = &img.style;
    // Os píxeis de ecrã por píxel natural (o maior dos dois eixos da caixa rodada).
    let px = |p: [f64; 2]| t * Point::new(p[0], p[1]);
    let (o, ex, ey) = (px([0.0, 0.0]), px([el.w, 0.0]), px([0.0, el.h]));
    let (sw, sh) = ((ex - o).hypot(), (ey - o).hypot());
    let level = file.and_then(|f| {
        let [_, _, cw, ch] = img.source(f.size);
        let vector = f.mime() == Some(VECTOR_MIME);
        let screen = [
            sw * f64::from(f.size[0]) / cw.max(1e-9),
            sh * f64::from(f.size[1]) / ch.max(1e-9),
        ];
        let need = match export {
            Some(true) if vector => screen.map(|n| n * PRINT_PX_PER_PT),
            Some(true) => [f64::from(f.size[0]), f64::from(f.size[1])],
            _ => screen,
        };
        cache
            .images
            .level(img.file, img.filter, f, need, export.is_some())
            .map(|l| (f, l))
    });
    let framed = st.stroke.is_some() && st.stroke_width > 0.0;
    // No export uma imagem sem píxeis não sai (nem o cinzento do «a chegar»); a moldura sai.
    if level.is_none() && export.is_some() && !framed {
        return;
    }
    let clip = img.crop.is_some() || st.round;
    let alpha = f32::from(st.opacity.min(100)) / 100.0;
    // Com moldura e transparente: a imagem e a moldura compõem-se JUNTAS (a de uma forma: o traço não
    // deixa ver a imagem por baixo).
    let group = framed && alpha < 1.0;
    if group {
        let [x0, y0, x1, y1] = el.aabb();
        let pad = crate::image_frame::reach(el, img);
        let p0 = v * Point::new(x0 - pad, y0 - pad);
        let p1 = v * Point::new(x1 + pad, y1 + pad);
        scene.push_object_layer(&Rect::from_points(p0, p1), VelloBlend::default(), alpha);
    }
    match level {
        None if export.is_some() => {}
        None => placeholder(scene, el, t, theme),
        Some((f, (pixels, size))) => {
            let a = if group { 1.0 } else { alpha };
            let layered = clip || a < 1.0;
            if layered {
                scene.push_layer_shape(VelloBlend::default(), a, t, &box_path(el, st.round));
            }
            let source = f.mime().map(|mime| ImageSource {
                bytes: f.bytes.0.clone(),
                mime,
                size: f.size,
                gray: img.filter == ImageFilter::Gray,
            });
            let m = t * level_to_box(el, img, f.size, size);
            scene.draw_recorded_image(&pixels, m, ImageQuality::Medium, source.as_ref());
            if layered {
                scene.pop_layer();
            }
        }
    }
    if framed {
        crate::image_frame::paint(scene, cache, el, img, t, board);
    }
    if group {
        scene.pop_layer();
    }
}

/// O lugar de uma imagem que ainda não chegou (ou que não se lê): a caixa no fundo elevado e, ao
/// meio, o ícone de imagem — a moldura, o sol e o monte (o idioma do Excalidraw, `w14_imagens.png`).
fn placeholder(scene: &mut VectorScene, el: &Element, t: Affine, theme: Theme) {
    scene.fill_path(
        &box_path(el, el.image().is_some_and(|i| i.style.round)),
        &Brush::Solid(token(ColorToken::Bg3, theme)),
        t,
    );
    let side = el.w.min(el.h) * 0.3;
    if side <= 0.0 {
        return;
    }
    let (cx, cy) = (el.w / 2.0, el.h / 2.0);
    let (x0, y0) = (cx - side / 2.0, cy - side * 0.4);
    let (x1, y1) = (cx + side / 2.0, cy + side * 0.4);
    let ink = Brush::Solid(token(ColorToken::Text2, theme));
    let frame = Rect::new(x0, y0, x1, y1).to_rounded_rect(side * 0.08);
    scene.stroke_path(&Stroke::new(side * 0.07), t, &ink, &frame);
    let sun = Circle::new((x0 + side * 0.3, y0 + side * 0.25), side * 0.09);
    scene.fill_path(&sun.to_path(0.1), &ink, t);
    let mut hill = ph2d_vector::BezPath::new();
    hill.move_to((x0 + side * 0.1, y1 - side * 0.1));
    hill.line_to((x0 + side * 0.4, y0 + side * 0.4));
    hill.line_to((x0 + side * 0.6, y1 - side * 0.3));
    hill.line_to((x0 + side * 0.72, y0 + side * 0.52));
    hill.line_to((x1 - side * 0.1, y1 - side * 0.1));
    hill.close_path();
    scene.fill_path(&hill, &ink, t);
}

#[cfg(test)]
#[path = "image_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "image_w15_tests.rs"]
mod w15_tests;
