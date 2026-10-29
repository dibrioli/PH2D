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

/// Metade em cada lado (média de cada quadrado 2×2; a coluna/linha ímpar final sai).
fn halve(w: u32, h: u32, px: &[u8]) -> (u32, u32, Vec<u8>) {
    let (nw, nh) = (w / 2, h / 2);
    let mut out = Vec::with_capacity(nw as usize * nh as usize * 4);
    let at =
        |x: u32, y: u32, c: usize| u32::from(px[(y as usize * w as usize + x as usize) * 4 + c]);
    for y in 0..nh {
        for x in 0..nw {
            for c in 0..4 {
                let s = at(2 * x, 2 * y, c)
                    + at(2 * x + 1, 2 * y, c)
                    + at(2 * x, 2 * y + 1, c)
                    + at(2 * x + 1, 2 * y + 1, c);
                #[allow(clippy::cast_possible_truncation)]
                out.push(((s + 2) / 4) as u8);
            }
        }
    }
    (nw, nh, out)
}

fn lin(c: u8) -> f32 {
    let c = f32::from(c) / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// A luminância relativa (WCAG) de um píxel; o transparente conta como BRANCO (o pior fundo).
fn luminance(p: &[u8; 4]) -> f32 {
    let l = 0.2126 * lin(p[0]) + 0.7152 * lin(p[1]) + 0.0722 * lin(p[2]);
    let a = f32::from(p[3]) / 255.0;
    l * a + (1.0 - a)
}

/// A luminância relativa de uma cor de token.
pub(crate) fn color_luminance(c: ph2d_vector::Color) -> f32 {
    let [r, g, b, _] = c.to_rgba8().to_u8_array();
    luminance(&[r, g, b, 255])
}

/// O contraste WCAG entre duas luminâncias.
pub(crate) fn contrast(a: f32, b: f32) -> f32 {
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// O contraste que o texto tem de ter sobre a imagem (WCAG AA, texto normal).
pub(crate) const MIN_CONTRAST: f32 = 4.5;

/// ⭐ O alfa do véu (cor de luminância `veil`) para o texto de luminância `ink` se ler a
/// [`MIN_CONTRAST`] sobre a linha mais clara `brightest` da faixa do título — composto em luz
/// LINEAR (o Vello compõe em sRGB, que dá MAIS escuro: a conta é do lado seguro). Nunca menos que
/// `floor` (o véu da maquete).
pub(crate) fn veil_alpha(brightest: f32, ink: f32, veil: f32, floor: f32) -> f32 {
    let goal = (ink + 0.05) / MIN_CONTRAST - 0.05;
    let need = if brightest <= goal || brightest <= veil {
        0.0
    } else {
        (brightest - goal) / (brightest - veil)
    };
    need.clamp(floor, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halving_averages_each_two_by_two() {
        let px = [
            0, 0, 0, 255, 255, 255, 255, 255, //
            255, 255, 255, 255, 0, 0, 0, 255,
        ];
        let (w, h, out) = halve(2, 2, &px);
        assert_eq!((w, h), (1, 1));
        assert_eq!(out, vec![128, 128, 128, 255]);
    }

    #[test]
    fn the_veil_brings_white_text_to_four_and_a_half_on_a_white_image() {
        let a = veil_alpha(1.0, 0.8, 0.0, 0.3);
        let under = 1.0 * (1.0 - a);
        assert!((contrast(0.8, under) - MIN_CONTRAST).abs() < 1e-3, "{a}");
        assert!((veil_alpha(0.0, 0.8, 0.0, 0.3) - 0.3).abs() < f32::EPSILON);
    }
}
