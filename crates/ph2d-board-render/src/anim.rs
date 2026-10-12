//! W16 — **uma imagem ANIMADA no ecrã** (o GIF): os quadros descodificam-se EM SEQUÊNCIA, um à frente
//! do que se mostra (a memória é a de dois quadros, qualquer que seja a duração), e recomeça no fim.
//! W18 — cada quadro é um passo no grupo de fios das imagens, não um fio por GIF. O primeiro quadro, o do export e o de enquanto o fio arranca, é o da
//! pirâmide de sempre ([`crate::image`]).
//!
//! O tempo de cada quadro é o do ficheiro, com a lei dos navegadores para os curtos ([`frame_ms`]).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use ph2d_board_model::{FileId, ImageFilter};
use ph2d_vector::StableImage;

use crate::image::{ATLAS_SIDE, Pixels, halve_to};

/// Os quadros de uma imagem animada pela ordem, cada um com a duração em milissegundos (a do
/// ficheiro). `None` = os bytes não são uma animação que o app lê. Corre no fio do tocador.
pub type Animate =
    Arc<dyn Fn(&[u8]) -> Option<Box<dyn Iterator<Item = (Pixels, u32)> + Send>> + Send + Sync>;

/// O animador do APP, instalado uma vez ao arrancar (como o descodificador).
static ANIMATOR: OnceLock<Animate> = OnceLock::new();

/// Instala o animador do app (a 2.ª chamada não muda nada).
pub fn install_animator(animate: Animate) {
    let _ = ANIMATOR.set(animate);
}

/// ⭐ A duração de um quadro no ecrã: a do ficheiro, e um quadro de **10 ms ou menos** dura **100 ms**
/// — a lei dos navegadores (Chromium `blink::ImageDecoder` e Firefox `nsGIFDecoder2`: os GIF antigos
/// gravavam 0 ou 1 centésimo a querer «o mais depressa possível», e tocá-los assim queimava a CPU e
/// não era o que o autor via).
#[must_use]
pub fn frame_ms(file_ms: u32) -> u32 {
    if file_ms <= 10 { 100 } else { file_ms }
}

type Key = (FileId, ImageFilter);
type Frame = (StableImage, [u32; 2], u32);
/// O que um passo devolve ao tocador: o quadro (ou `None` = não é animação) e o cursor de volta.
type Back = (Option<Frame>, Box<Cursor>);

/// ⭐ W18 — UM passo de um tocador: descodificar o quadro seguinte. Corre num fio do MESMO grupo que
/// descodifica as imagens ([`crate::image`]) — nenhum fio por GIF (W16 tinha um por imagem à vista, sem
/// tecto: régua `anim_measure_tests`). Cada tocador tem no máximo UM passo no ar.
pub(crate) struct Step {
    cur: Box<Cursor>,
    back: Sender<Back>,
}

impl Step {
    pub(crate) fn run(mut self) {
        let f = self.cur.next_frame();
        // O tocador foi largado: o cursor morre com o envio falhado.
        let _ = self.back.send((f, self.cur));
    }
}

/// Onde vai a leitura de um GIF: os quadros que faltam desta passagem, e quantos já deu.
struct Cursor {
    animate: Animate,
    bytes: Arc<[u8]>,
    filter: ImageFilter,
    /// O tamanho a que o desenho quer os quadros (`w << 32 | h`): reduz por metades até ele.
    want: Arc<AtomicU64>,
    frames: Option<Box<dyn Iterator<Item = (Pixels, u32)> + Send>>,
    /// Quadros desta passagem (no fim de uma de UM quadro só: não é animação).
    pass: usize,
}

impl Cursor {
    /// O quadro seguinte, reduzido ao tamanho pedido e com o filtro; no fim recomeça. `None` = não é
    /// uma animação (um quadro só, ou ilegível).
    fn next_frame(&mut self) -> Option<Frame> {
        loop {
            if self.frames.is_none() {
                self.frames = Some((self.animate)(&self.bytes)?);
                self.pass = 0;
            }
            match self.frames.as_mut()?.next() {
                Some((px, ms)) => {
                    self.pass += 1;
                    return self.prepare(px, ms);
                }
                None if self.pass <= 1 => return None,
                None => self.frames = None,
            }
        }
    }

    fn prepare(&self, mut px: Pixels, ms: u32) -> Option<Frame> {
        let w = self.want.load(Ordering::Relaxed);
        let to = [(w >> 32) as u32, w as u32];
        // Por metades enquanto a metade ainda cobre o pedido (a lei da pirâmide) — e sempre até caber
        // no atlas do Vello (o que não cabe não se desenha, em silêncio).
        loop {
            let next = px.size.map(|v| v.div_ceil(2));
            let over = px.size[0].max(px.size[1]) > ATLAS_SIDE;
            if next == px.size || (!over && (next[0] < to[0] || next[1] < to[1])) {
                break;
            }
            px = halve_to(&px, next);
        }
        if self.filter == ImageFilter::Gray {
            crate::gray::to_gray(&mut px.rgba);
        }
        let img = StableImage::from_rgba(Arc::new(px.rgba), px.size[0], px.size[1])?;
        Some((img, px.size, frame_ms(ms)))
    }
}

/// O tocador de uma imagem: o quadro mostrado, o seguinte já descodificado, e o passo no ar.
struct Player {
    tx: Sender<Back>,
    rx: Receiver<Back>,
    want: Arc<AtomicU64>,
    /// O cursor parado à espera de que o quadro adiantado se mostre.
    cursor: Option<Box<Cursor>>,
    ahead: Option<Frame>,
    shown: Option<(StableImage, [u32; 2])>,
    /// Quando o quadro mostrado acaba.
    until: Option<Instant>,
    /// Quantos quadros se mostraram.
    got: usize,
    used: u64,
}

/// Os tocadores à vista, e as imagens que se provaram NÃO animadas (um quadro só: não se volta a
/// tentar).
#[derive(Default)]
pub(crate) struct Players {
    playing: std::collections::BTreeMap<Key, Player>,
    still: std::collections::BTreeSet<Key>,
    animate: Option<Animate>,
    /// Os passos por mandar ao grupo de fios ([`Players::take_steps`]).
    outbox: Vec<Step>,
}

impl Players {
    /// Outro animador que o do app (os testes).
    pub(crate) fn set_animator(&mut self, a: Animate) {
        self.animate = Some(a);
    }

    /// Os passos que os tocadores pediram desde a última vez (quem tem o grupo de fios manda-os).
    pub(crate) fn take_steps(&mut self) -> Vec<Step> {
        std::mem::take(&mut self.outbox)
    }

    /// O quadro de `key` a mostrar em `now`, com os píxeis a `want` (o nível da pirâmide que o
    /// desenho escolheu); arranca o tocador à primeira vez. `None` = ainda nenhum (desenha-se o
    /// primeiro, da pirâmide) ou não é animada.
    pub(crate) fn frame(
        &mut self,
        key: Key,
        bytes: &Arc<[u8]>,
        want: [u32; 2],
        now: Instant,
        tick: u64,
    ) -> Option<(StableImage, [u32; 2])> {
        if self.still.contains(&key) {
            return None;
        }
        let packed = (u64::from(want[0]) << 32) | u64::from(want[1]);
        if !self.playing.contains_key(&key) {
            let animate = self.animate.clone().or_else(|| ANIMATOR.get().cloned())?;
            let (tx, rx) = channel();
            let want = Arc::new(AtomicU64::new(packed));
            let cur = Box::new(Cursor {
                animate,
                bytes: bytes.clone(),
                filter: key.1,
                want: want.clone(),
                frames: None,
                pass: 0,
            });
            self.outbox.push(Step {
                cur,
                back: tx.clone(),
            });
            self.playing.insert(
                key,
                Player {
                    tx,
                    rx,
                    want,
                    cursor: None,
                    ahead: None,
                    shown: None,
                    until: None,
                    got: 0,
                    used: 0,
                },
            );
        }
        let p = self.playing.get_mut(&key)?;
        p.used = tick;
        p.want.store(packed, Ordering::Relaxed);
        while let Ok((f, cur)) = p.rx.try_recv() {
            match f {
                Some(f) => {
                    p.ahead = Some(f);
                    p.cursor = Some(cur);
                }
                None if p.got <= 1 => {
                    self.playing.remove(&key);
                    self.still.insert(key);
                    return None;
                }
                // Ilegível a meio de uma animação: fica no último quadro.
                None => {}
            }
        }
        if p.until.is_none_or(|t| now >= t)
            && let Some((img, size, ms)) = p.ahead.take()
        {
            p.got += 1;
            p.shown = Some((img, size));
            let d = Duration::from_millis(u64::from(ms));
            // Atrasado mais de um segundo (a janela parada): recomeça a contar daqui.
            p.until = Some(match p.until {
                Some(t) if now.saturating_duration_since(t) < Duration::from_secs(1) => t + d,
                _ => now + d,
            });
            // O seguinte descodifica-se enquanto este se mostra.
            if let Some(cur) = p.cursor.take() {
                self.outbox.push(Step {
                    cur,
                    back: p.tx.clone(),
                });
            }
        }
        p.shown.clone()
    }

    /// Esquece os tocadores que ninguém desenhou desde `cut` (o passo no ar morre no envio).
    pub(crate) fn forget_before(&mut self, cut: u64) {
        self.playing.retain(|_, p| p.used >= cut);
    }

    /// Há tocadores (quem desenha só a pedido pede outro quadro).
    pub(crate) fn any(&self) -> bool {
        !self.playing.is_empty()
    }
    /// A régua: quantos quadros se mostraram, somando os tocadores.
    #[cfg(test)]
    pub(crate) fn playing_count(&self) -> usize {
        self.playing.len()
    }

    #[cfg(test)]
    pub(crate) fn shown_total(&self) -> usize {
        self.playing.values().map(|p| p.got).sum()
    }
}

#[cfg(test)]
#[path = "anim_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "anim_measure_tests.rs"]
mod measure;
