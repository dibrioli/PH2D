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

use std::collections::{BTreeMap, BTreeSet};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use ph2d_board_geom::to_world;
use ph2d_board_model::{Element, FileId, Image, ImageFile, ImageFilter};
use ph2d_tokens::Theme;
use ph2d_vector::{
    Affine, ImageQuality, ImageSource, Point, Rect, Shape as _, StableImage, VectorScene,
    VelloBlend,
};

use crate::{ExportTarget, RenderCache};

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
/// W16 — o MIME de um ficheiro que pode ser ANIMADO no ecrã.
const ANIMATED_MIME: &str = "image/gif";

/// O descodificador do APP, instalado uma vez ao arrancar ([`install_decoder`]): toda [`Images`] o usa
/// quando precisa de descodificar (não quando nasce — uma cache feita antes da instalação também o vê).
static DECODER: OnceLock<Decode> = OnceLock::new();

/// Instala o descodificador do app (a 2.ª chamada não muda nada). Quem tem a porta de entrada das
/// imagens chama-o ao arrancar.
pub fn install_decoder(decode: Decode) {
    let _ = DECODER.set(decode);
}

/// W18 — o SALVADOR: as linhas que se lêem de um ficheiro que o descodificador recusou (um PNG ou JPEG
/// cortado mostra as de cima, como o navegador); a imagem continua ESTRAGADA (o aviso, a marca).
pub type Salvage = Arc<dyn Fn(&[u8]) -> Option<Pixels> + Send + Sync>;

static SALVAGER: OnceLock<Salvage> = OnceLock::new();

/// Instala o salvador do app (a 2.ª chamada não muda nada).
pub fn install_salvager(salvage: Salvage) {
    let _ = SALVAGER.set(salvage);
}

/// O maior lado de um nível que o Vello aceita: o atlas dele é um quadrado deste lado.
pub(crate) const ATLAS_SIDE: u32 = 8192;
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
    /// `bool` = salva de um ficheiro estragado (as linhas que se leram).
    Pyramid(Option<Levels>, bool),
    Level(usize, Option<(StableImage, [u32; 2])>),
}

type Done = (Key, Out);

/// ⭐ **As imagens descodificadas** do desenho, por ficheiro (os mesmos bytes = a mesma entrada, em
/// qualquer quadro). Uma por [`crate::RenderCache`].
pub struct Images {
    decode: Option<Decode>,
    salvage: Option<Salvage>,
    entries: BTreeMap<Key, Entry>,
    tx: Sender<Done>,
    rx: Receiver<Done>,
    /// Os trabalhos por fazer, partilhados pelos fios (criados à primeira vez): descodificar uma imagem
    /// e (W18) os passos dos GIF a tocar.
    jobs: Option<Sender<Task>>,
    /// W18 — a régua do W16 (um fio POR GIF), só para a medição no mesmo processo.
    #[cfg(test)]
    pub(crate) gif_thread_each: Option<BTreeMap<Key, Sender<crate::anim::Step>>>,
    frame: u64,
    /// W16 — os GIF a tocar no ecrã, e o relógio deles (o do começo do desenho).
    pub(crate) players: crate::anim::Players,
    now: Instant,
    /// Os testes param o relógio.
    clock: Option<Instant>,
    /// W17 — os ficheiros que não se lêem e já foram ditos a quem desenha ([`Images::take_failed`]).
    reported: BTreeSet<FileId>,
}

/// O que um fio do grupo faz.
enum Task {
    Decode(Job),
    Step(crate::anim::Step),
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
            salvage: None,
            entries: BTreeMap::new(),
            tx,
            rx,
            jobs: None,
            #[cfg(test)]
            gif_thread_each: None,
            frame: 0,
            players: crate::anim::Players::default(),
            now: Instant::now(),
            clock: None,
            reported: BTreeSet::new(),
        }
    }
}

#[path = "image_box.rs"]
mod boxes;
#[path = "image_pyramid.rs"]
mod pyramid;
pub(crate) use boxes::box_path;
pub use boxes::level_to_box;
use boxes::{broken_mark, placeholder};
#[cfg(test)]
use pyramid::halve;
pub(crate) use pyramid::halve_to;
use pyramid::{level_sizes, pyramid};

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
fn run(decode: &Decode, salvage: Option<&Salvage>, job: &Job) -> Out {
    let filter = job.key.1;
    let ok = |f: &dyn Fn() -> Option<Levels>| {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
            .ok()
            .flatten()
    };
    match job.at {
        None => match ok(&|| decode_filtered(decode, &job.bytes, None, filter).and_then(pyramid)) {
            Some(levels) => Out::Pyramid(Some(levels), false),
            None => {
                let saved = salvage.and_then(|s| {
                    let s: Decode = Arc::new({
                        let s = s.clone();
                        move |b: &[u8], _| s(b)
                    });
                    ok(&|| decode_filtered(&s, &job.bytes, None, filter).and_then(pyramid))
                });
                Out::Pyramid(saved, true)
            }
        },
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

    /// W18 — outro salvador que o do app.
    pub fn set_salvager(&mut self, salvage: Salvage) {
        self.salvage = Some(salvage);
    }

    /// O salvador desta cache, ou o do app.
    fn salvager(&self) -> Option<Salvage> {
        self.salvage.clone().or_else(|| SALVAGER.get().cloned())
    }

    /// W17 — os ficheiros que o descodificador não leu (o cabeçalho bom, os dados estragados) e que
    /// ainda não tinham sido ditos: cada um UMA vez, para quem desenha avisar na tela.
    pub fn take_failed(&mut self) -> Vec<FileId> {
        let mut out = Vec::new();
        for ((id, _), e) in &self.entries {
            if e.failed && self.reported.insert(*id) {
                out.push(*id);
            }
        }
        out
    }

    /// W17 — `file` (com o filtro `filter`) não se lê: desenha-se partido, não «a chegar».
    pub(crate) fn failed(&self, file: FileId, filter: ImageFilter) -> bool {
        self.entries.get(&(file, filter)).is_some_and(|e| e.failed)
    }

    /// W16 — outro animador que o do app.
    pub fn set_animator(&mut self, animate: crate::anim::Animate) {
        self.players.set_animator(animate);
    }

    /// W16 — o relógio parado em `at` (os testes); `None` = o do sistema.
    pub fn set_clock(&mut self, at: Option<Instant>) {
        self.clock = at;
    }

    /// ⭐ W16 — o quadro de uma imagem ANIMADA a mostrar agora no ecrã, aos píxeis `size` (o nível
    /// que o desenho escolheu). `None` = não é animada, ou o tocador ainda não deu o primeiro (o da
    /// pirâmide desenha-se).
    pub(crate) fn animated(
        &mut self,
        id: FileId,
        filter: ImageFilter,
        file: &ImageFile,
        size: [u32; 2],
    ) -> Option<(StableImage, [u32; 2])> {
        if file.mime() != Some(ANIMATED_MIME) {
            return None;
        }
        let shown = self
            .players
            .frame((id, filter), &file.bytes.0, size, self.now, self.frame);
        for step in self.players.take_steps() {
            #[cfg(test)]
            if let Some(each) = &mut self.gif_thread_each {
                let tx = each.entry((id, filter)).or_insert_with(|| {
                    let (tx, rx) = channel::<crate::anim::Step>();
                    std::thread::spawn(move || rx.into_iter().for_each(crate::anim::Step::run));
                    tx
                });
                let _ = tx.send(step);
                continue;
            }
            self.submit(Task::Step(step));
        }
        shown
    }

    /// O descodificador desta cache, ou o do app.
    fn decoder(&self) -> Option<Decode> {
        self.decode.clone().or_else(|| DECODER.get().cloned())
    }

    /// Há imagens a descodificar: quem desenha só a pedido pede outro quadro (a shell desenha
    /// sempre, `ControlFlow::Poll`).
    #[must_use]
    pub fn pending(&self) -> bool {
        self.players.any()
            || self
                .entries
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
            Out::Pyramid(levels, broken) => {
                e.failed |= broken;
                levels
            }
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
        if self.decoder().is_some() {
            self.submit(Task::Decode(job));
        }
    }

    /// Manda um trabalho ao grupo de fios (criado à primeira vez: metade da máquina — a outra é do
    /// desenho e do resto do app).
    fn submit(&mut self, task: Task) {
        let (decode, salvage) = (self.decoder(), self.salvager());
        let jobs = self.jobs.get_or_insert_with(|| {
            let (jt, jr) = channel::<Task>();
            let jr = Arc::new(Mutex::new(jr));
            let n = std::thread::available_parallelism().map_or(1, |n| (n.get() / 2).max(1));
            for _ in 0..n {
                let (jr, tx, decode, salvage) =
                    (jr.clone(), self.tx.clone(), decode.clone(), salvage.clone());
                std::thread::spawn(move || {
                    loop {
                        let task = match jr.lock() {
                            Ok(r) => r.recv(),
                            Err(_) => return,
                        };
                        match task {
                            Ok(Task::Decode(job)) => {
                                let Some(decode) = &decode else { continue };
                                let out = run(decode, salvage.as_ref(), &job);
                                if tx.send((job.key, out)).is_err() {
                                    return;
                                }
                            }
                            Ok(Task::Step(step)) => step.run(),
                            Err(_) => return,
                        }
                    }
                });
            }
            jt
        });
        let _ = jobs.send(task);
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
        let salvage = self.salvager();
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
        let want = e
            .levels
            .iter()
            .rposition(|l| f64::from(l.size[0]) >= need[0] && f64::from(l.size[1]) >= need[1])
            .unwrap_or(0);
        // W18 — estragada: as linhas salvas, se as há (nada se volta a descodificar).
        if e.failed {
            return self.pick(key, want, frame);
        }
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
                        let out = run(d, salvage.as_ref(), &job);
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
                    let out = run(d, salvage.as_ref(), &job);
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
            self.players.forget_before(cut);
            // W18 — a estragada não volta a descodificar: as linhas salvas ficam.
            for e in self.entries.values_mut().filter(|e| !e.failed) {
                let last = e.levels.len() - 1;
                for l in &mut e.levels[..last] {
                    if l.used < cut {
                        l.image = None;
                    }
                }
            }
        }
    }

    /// Recolhe os acabados (no início de um desenho) e acerta o relógio das animações.
    pub(crate) fn begin_frame(&mut self) {
        self.collect();
        self.now = self.clock.unwrap_or_else(Instant::now);
    }
}

/// W16 — a opacidade da imagem inteira por baixo da janela de um recorte: a medida no Excalidraw
/// (`saidas/w16`, A2: fora da janela o vermelho sai `(255, 228, 228)` sobre branco — α = 27/255).
const CROP_FADE: f32 = 27.0 / 255.0;

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
    let target = cache.export.map(|e| e.target);
    let export = target.map(ExportTarget::vector);
    let t = v * to_world(el);
    let st = &img.style;
    // Os píxeis de ecrã por píxel natural (o maior dos dois eixos da caixa rodada).
    let px = |p: [f64; 2]| t * Point::new(p[0], p[1]);
    let (o, ex, ey) = (px([0.0, 0.0]), px([el.w, 0.0]), px([0.0, el.h]));
    let (sw, sh) = ((ex - o).hypot(), (ey - o).hypot());
    let source_of = |f: &ImageFile| {
        f.mime().map(|mime| ImageSource {
            bytes: f.bytes.0.clone(),
            mime,
            size: f.size,
            gray: img.filter == ImageFilter::Gray,
        })
    };
    // O SVG embebe os bytes: nenhum píxel se descodifica (W16).
    let embed = file
        .filter(|_| target == Some(ExportTarget::Svg))
        .and_then(|f| Some((f, source_of(f)?)));
    let level = file.filter(|_| embed.is_none()).and_then(|f| {
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
    if level.is_none() && embed.is_none() && export.is_some() && !framed {
        return;
    }
    let clip = img.crop.is_some() || st.round;
    let alpha = f32::from(st.opacity.min(100)) / 100.0;
    // Com moldura e transparente: a imagem e a moldura compõem-se JUNTAS (a de uma forma: o traço não
    // deixa ver a imagem por baixo).
    let group = framed && alpha < 1.0;
    if group {
        let [x0, y0, x1, y1] = el.aabb();
        let pad = crate::image_frame::reach(el, &img.style);
        let p0 = v * Point::new(x0 - pad, y0 - pad);
        let p1 = v * Point::new(x1 + pad, y1 + pad);
        scene.push_object_layer(&Rect::from_points(p0, p1), VelloBlend::default(), alpha);
    }
    if level.is_none() && embed.is_none() {
        if export.is_none() {
            let broken = file.is_some() && cache.images.failed(img.file, img.filter);
            placeholder(scene, el, t, theme, broken);
        }
    } else {
        let a = if group { 1.0 } else { alpha };
        let layered = clip || a < 1.0;
        // W16 — um GIF no ecrã toca: o quadro de agora ao tamanho do nível escolhido (W18: também na
        // apresentação).
        let drawn = level.map(|(f, (pixels, size))| {
            let (pixels, size) = if ExportTarget::live(target) {
                cache
                    .images
                    .animated(img.file, img.filter, f, size)
                    .unwrap_or((pixels, size))
            } else {
                (pixels, size)
            };
            (f, pixels, t * level_to_box(el, img, f.size, size), size)
        });
        // W16 — no recorte, a imagem INTEIRA esbatida por baixo, FORA do corte da caixa.
        if let Some((_, pixels, m, size)) = &drawn
            && export.is_none()
            && cache.cropping == Some(el.id)
        {
            let whole = Rect::new(0.0, 0.0, f64::from(size[0]), f64::from(size[1]));
            scene.push_layer_shape(VelloBlend::default(), CROP_FADE, *m, &whole.to_path(0.1));
            scene.draw_recorded_image(pixels, *m, ImageQuality::Medium, None);
            scene.pop_layer();
        }
        if layered {
            scene.push_layer_shape(VelloBlend::default(), a, t, &box_path(el, st.round));
        }
        if let Some((f, source)) = &embed {
            scene.record_embedded_image(t * level_to_box(el, img, f.size, f.size), source);
        } else if let Some((f, pixels, m, _)) = drawn {
            scene.draw_recorded_image(&pixels, m, ImageQuality::Medium, source_of(f).as_ref());
        }
        if layered {
            scene.pop_layer();
        }
        // W18 — as linhas que se leram de uma estragada: no ecrã, a marca dela num canto.
        if export.is_none() && embed.is_none() && cache.images.failed(img.file, img.filter) {
            broken_mark(scene, el, t, theme);
        }
    }
    if framed {
        crate::image_frame::paint(scene, cache, el, &img.style, t, board);
    }
    if group {
        scene.pop_layer();
    }
}

#[cfg(test)]
#[path = "image_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "image_w15_tests.rs"]
mod w15_tests;

#[cfg(test)]
#[path = "image_w18_tests.rs"]
mod w18_tests;
