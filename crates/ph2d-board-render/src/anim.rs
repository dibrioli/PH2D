//! W16 — **uma imagem ANIMADA no ecrã** (o GIF): um fio por imagem à vista descodifica os quadros EM
//! SEQUÊNCIA, um à frente do que se mostra (a memória é a de dois quadros, qualquer que seja a
//! duração), e recomeça no fim. O primeiro quadro, o do export e o de enquanto o fio arranca, é o da
//! pirâmide de sempre ([`crate::image`]).
//!
//! O tempo de cada quadro é o do ficheiro, com a lei dos navegadores para os curtos ([`frame_ms`]).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError, sync_channel};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use ph2d_board_model::{FileId, ImageFilter};
use ph2d_vector::StableImage;

use crate::image::{Pixels, halve_to};

/// Os quadros de uma imagem animada pela ordem, cada um com a duração em milissegundos (a do
/// ficheiro). `None` = os bytes não são uma animação que o app lê. Corre no fio do tocador.
pub type Animate =
    Arc<dyn Fn(&[u8]) -> Option<Box<dyn Iterator<Item = (Pixels, u32)>>> + Send + Sync>;

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

/// O tocador de uma imagem: o fio manda quadros por um canal de UM lugar (decodifica o seguinte
/// enquanto este se mostra).
struct Player {
    rx: Receiver<Frame>,
    /// O tamanho a que o desenho quer os quadros (`w << 32 | h`): o fio reduz por metades até ele.
    want: Arc<AtomicU64>,
    shown: Option<(StableImage, [u32; 2])>,
    /// Quando o quadro mostrado acaba.
    until: Option<Instant>,
    /// Quantos quadros chegaram (um só = não é animada).
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
}

impl Players {
    /// Outro animador que o do app (os testes).
    pub(crate) fn set_animator(&mut self, a: Animate) {
        self.animate = Some(a);
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
            self.playing
                .insert(key, spawn(animate, bytes.clone(), key.1, packed));
        }
        let p = self.playing.get_mut(&key)?;
        p.used = tick;
        p.want.store(packed, Ordering::Relaxed);
        while p.until.is_none_or(|t| now >= t) {
            match p.rx.try_recv() {
                Ok((img, size, ms)) => {
                    p.got += 1;
                    p.shown = Some((img, size));
                    let d = Duration::from_millis(u64::from(ms));
                    // Atrasado mais de um segundo (a janela parada): recomeça a contar daqui.
                    p.until = Some(match p.until {
                        Some(t) if now.saturating_duration_since(t) < Duration::from_secs(1) => {
                            t + d
                        }
                        _ => now + d,
                    });
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    // Um quadro só (ou ilegível): não é animada.
                    if p.got <= 1 {
                        self.playing.remove(&key);
                        self.still.insert(key);
                        return None;
                    }
                    break;
                }
            }
        }
        p.shown.clone()
    }

    /// Esquece os tocadores que ninguém desenhou desde `cut` (o fio acaba no envio seguinte).
    pub(crate) fn forget_before(&mut self, cut: u64) {
        self.playing.retain(|_, p| p.used >= cut);
    }

    /// Há tocadores (quem desenha só a pedido pede outro quadro).
    pub(crate) fn any(&self) -> bool {
        !self.playing.is_empty()
    }
}

/// O fio de um tocador: percorre os quadros, reduz cada um ao tamanho pedido, aplica o filtro, e
/// recomeça no fim. Acaba quando o desenho larga o tocador, ou se a passagem deu um quadro só.
fn spawn(animate: Animate, bytes: Arc<[u8]>, filter: ImageFilter, want: u64) -> Player {
    let (tx, rx) = sync_channel::<Frame>(1);
    let want = Arc::new(AtomicU64::new(want));
    let target = want.clone();
    std::thread::spawn(move || {
        loop {
            let Some(frames) = animate(&bytes) else {
                return;
            };
            let mut n = 0usize;
            for (mut px, ms) in frames {
                let w = target.load(Ordering::Relaxed);
                let to = [(w >> 32) as u32, w as u32];
                // Por metades enquanto a metade ainda cobre o pedido (a lei da pirâmide).
                loop {
                    let next = px.size.map(|v| v.div_ceil(2));
                    if next == px.size || next[0] < to[0] || next[1] < to[1] {
                        break;
                    }
                    px = halve_to(&px, next);
                }
                if filter == ImageFilter::Gray {
                    crate::gray::to_gray(&mut px.rgba);
                }
                let Some(img) = StableImage::from_rgba(Arc::new(px.rgba), px.size[0], px.size[1])
                else {
                    return;
                };
                n += 1;
                if tx.send((img, px.size, frame_ms(ms))).is_err() {
                    return;
                }
            }
            if n <= 1 {
                return;
            }
        }
    });
    Player {
        rx,
        want,
        shown: None,
        until: None,
        got: 0,
        used: 0,
    }
}

#[cfg(test)]
#[path = "anim_tests.rs"]
mod tests;
