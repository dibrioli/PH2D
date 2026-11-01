//! ⭐ **As capas de IMAGEM descodificadas**, uma vez por ficheiro (a chave é o conteúdo): um fio de
//! fundo descodifica pela porta da cena (`ph2d_asset::decode_image`) e reduz por metades até à
//! largura que o cartão pode mostrar; o desenho nunca descodifica. Enquanto chega, a capa reserva o
//! lugar com a cor de espera.
//!
//! Cada linha da miniatura guarda a luminância MAIS CLARA que tem: o véu do título da capa inteira
//! escurece o que for preciso para o texto se ler a 4,5:1 sobre a imagem real ([`veil_alpha`]).

use ph2d_kanban_model::FileId;
use ph2d_vector::StableImage;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, OnceLock};

/// A largura de textura que a capa usa: o cartão (`256` lógicos) a `200 %`. Política de MEMÓRIA
/// (o atlas do Vello é um quadrado partilhado), não de qualidade — acima disto o Vello amplia
/// bilinear. Medição em `docs/TrelloClone/loop/MEDICOES.md` §W3 D2.
pub(crate) const TEXELS_W: u32 = 512;

/// A largura de textura do FUNDO de imagem do Kanban: uma janela de 1920 lógicos a 1×. Mesma política
/// de memória (≤ 2048 × 2048 × 4 = 16 MB no atlas); medição em `MEDICOES.md` §D11.
pub(crate) const BG_TEXELS_W: u32 = 2048;

/// Uma capa pronta a desenhar: a imagem ESTÁVEL (o atlas do Vello reusa-a entre quadros) e a
/// luminância máxima (linear) de cada linha.
pub(crate) struct Thumb {
    pub size: [u32; 2],
    pub image: StableImage,
    pub row_max: Vec<f32>,
}

impl std::fmt::Debug for Thumb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Thumb")
            .field("size", &self.size)
            .finish_non_exhaustive()
    }
}

/// Uma capa partilhada; duas frentes são iguais se apontam a MESMA miniatura.
#[derive(Clone, Debug)]
pub(crate) struct ThumbRef(pub Rc<Thumb>);

impl PartialEq for ThumbRef {
    fn eq(&self, o: &Self) -> bool {
        Rc::ptr_eq(&self.0, &o.0)
    }
}

enum Slot {
    Pending,
    Ready(Rc<Thumb>),
    Failed,
}

/// Um ficheiro a uma largura de textura (a capa e o fundo do mesmo ficheiro são duas miniaturas).
type Key = (FileId, u32);
type Done = (Key, Option<Thumb>);
type Job = (Key, Arc<[u8]>, Sender<Done>);

struct Cache {
    slots: BTreeMap<Key, Slot>,
    tx: Sender<Done>,
    rx: Receiver<Done>,
}

impl Default for Cache {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            slots: BTreeMap::new(),
            tx,
            rx,
        }
    }
}

thread_local! {
    static CACHE: RefCell<Cache> = RefCell::new(Cache::default());
}

/// O fio que descodifica (um só, nasce à primeira capa).
fn worker() -> &'static Mutex<Sender<Job>> {
    static W: OnceLock<Mutex<Sender<Job>>> = OnceLock::new();
    W.get_or_init(|| {
        let (tx, rx) = channel::<Job>();
        let _ = std::thread::Builder::new()
            .name("kanban-covers".into())
            .spawn(move || {
                for (key, bytes, back) in rx {
                    let _ = back.send((key, decode_to(&bytes, key.1)));
                }
            });
        Mutex::new(tx)
    })
}

/// A capa `id` se já está pronta; senão pede-a (com os bytes que `bytes` der) e devolve `None`.
pub(crate) fn get(id: FileId, bytes: impl FnOnce() -> Option<Arc<[u8]>>) -> Option<ThumbRef> {
    get_at(id, TEXELS_W, bytes)
}

/// [`get`] à largura de textura `texels_w` (o fundo: [`BG_TEXELS_W`]).
pub(crate) fn get_at(
    id: FileId,
    texels_w: u32,
    bytes: impl FnOnce() -> Option<Arc<[u8]>>,
) -> Option<ThumbRef> {
    let key = (id, texels_w);
    CACHE.with_borrow_mut(|c| {
        while let Ok((done, thumb)) = c.rx.try_recv() {
            let slot = thumb.map_or(Slot::Failed, |t| Slot::Ready(Rc::new(t)));
            c.slots.insert(done, slot);
        }
        match c.slots.get(&key) {
            Some(Slot::Ready(t)) => Some(ThumbRef(t.clone())),
            Some(Slot::Pending | Slot::Failed) => None,
            None => {
                let b = bytes()?;
                let sent = worker()
                    .lock()
                    .ok()
                    .is_some_and(|w| w.send((key, b, c.tx.clone())).is_ok());
                c.slots
                    .insert(key, if sent { Slot::Pending } else { Slot::Failed });
                None
            }
        }
    })
}

/// Ainda há capas a caminho (quem desenha a pedido pede outro quadro).
pub(crate) fn pending() -> bool {
    CACHE.with_borrow(|c| c.slots.values().any(|s| matches!(s, Slot::Pending)))
}

/// Os bytes → a miniatura (`None` = não é uma imagem que o app lê).
#[cfg(test)]
pub(crate) fn decode(bytes: &[u8]) -> Option<Thumb> {
    decode_to(bytes, TEXELS_W)
}

/// [`decode`] reduzindo até à largura `texels_w`.
pub(crate) fn decode_to(bytes: &[u8], texels_w: u32) -> Option<Thumb> {
    let asset = ph2d_asset::decode_image(bytes).ok()?;
    let (w, h, px) = asset.image_rgba8()?;
    let (mut w, mut h, mut px) = (w, h, px.into_owned());
    while w >= 2 * texels_w && h >= 2 {
        (w, h, px) = halve(w, h, &px);
    }
    let row_max = px
        .chunks_exact(w as usize * 4)
        .take(h as usize)
        .map(|row| {
            row.as_chunks::<4>()
                .0
                .iter()
                .map(luminance)
                .fold(0.0_f32, f32::max)
        })
        .collect();
    Some(Thumb {
        size: [w, h],
        image: StableImage::from_rgba(Arc::new(px), w, h)?,
        row_max,
    })
}

pub(crate) use ph2d_board_render::kanban_cover::{contrast, veil_alpha};
use ph2d_board_render::kanban_cover::{halve, pixel_luminance as luminance};
pub(crate) use ph2d_board_render::luminance as color_luminance;
