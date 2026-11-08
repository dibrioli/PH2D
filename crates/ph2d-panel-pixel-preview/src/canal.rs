//! **O canal da imagem e o tocar da Preview** — a família publica a imagem do quadro mostrado
//! ([`publish`]) só com a janela visível; o painel lê-a ([`current`]) e só recria a imagem da GPU
//! quando o `seq` muda.
//!
//! O tocar da Preview ([`PreviewPlay`]) é a MESMA máquina do editor (`ph2d_pixel_core::play`) com
//! um estado próprio: o quadro activo do documento não anda (Trial 1.3.18, `ronda3b.preview_s0`).

use std::sync::Arc;

use ph2d_pixel_core::play::{Play, PlayOptions, PlaySpeed};
use ph2d_pixel_core::{FrameId, Sprite};

/// A imagem que a Preview mostra (RGBA de alfa reto, `w × h × 4`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewFrame {
    pub rgba: Arc<Vec<u8>>,
    pub w: u32,
    pub h: u32,
    /// Muda a cada imagem nova (a chave da imagem da GPU do painel).
    pub seq: u64,
    /// O frame mostrado (o id estável).
    pub frame: u32,
    /// A Preview está a tocar (o botão do título vira *Stop*).
    pub playing: bool,
    /// O xadrez do DOCUMENTO (P12 P: o mesmo do canvas, `Apply Zoom` incluído).
    pub checker: ph2d_pixel_core::Checker,
}

thread_local! {
    static CURRENT: std::cell::RefCell<Option<PreviewFrame>> = const { std::cell::RefCell::new(None) };
}

/// **A porta da família** — `None` com a janela fechada ou sem sprite em Draw.
pub fn publish(f: Option<PreviewFrame>) {
    CURRENT.with(|c| *c.borrow_mut() = f);
}

/// O que a Preview mostra agora.
#[must_use]
pub fn current() -> Option<PreviewFrame> {
    CURRENT.with(|c| c.borrow().clone())
}

/// O tocar da Preview e a chave da última imagem publicada.
#[derive(Debug, Default)]
pub struct PreviewPlay {
    play: Option<Play>,
    /// O documento em que o tocar começou (trocar de documento pára-o).
    doc: Option<u32>,
    /// `(documento, stamp do documento, frame)` da última imagem composta.
    chave: Option<(u32, u64, u32)>,
    seq: u64,
    /// Um pedido abriu a janela (a família aplica-o onde vê o `hero`).
    pub abrir: bool,
    /// O F7: alternar a janela.
    pub alternar: bool,
}

impl PreviewPlay {
    #[must_use]
    pub fn playing(&self) -> bool {
        self.play.is_some()
    }

    /// Shift+Enter: toca a partir do quadro activo, ou pára (e a Preview volta ao activo).
    pub fn toggle(
        &mut self,
        doc: u32,
        sprite: &Sprite,
        active: FrameId,
        opt: PlayOptions,
        speed: PlaySpeed,
    ) {
        self.abrir = true;
        if self.play.take().is_some() {
            return;
        }
        let i = sprite.frame_index(active).unwrap_or(0);
        let mut p = Play::start(sprite, i, opt);
        p.speed = speed;
        self.play = Some(p);
        self.doc = Some(doc);
    }

    pub fn stop(&mut self) {
        self.play = None;
    }

    /// Anda o tocar `dt_ms` no relógio do host (o documento `doc` é o que está em Draw).
    pub fn advance(&mut self, doc: Option<u32>, sprite: Option<&Sprite>, dt_ms: u32) {
        let (Some(p), Some(s)) = (self.play.as_mut(), sprite) else {
            return;
        };
        if doc != self.doc || p.index >= s.frames().len() {
            self.play = None;
            return;
        }
        p.advance(s, dt_ms);
        if !p.playing {
            self.play = None;
        }
    }

    /// O frame que a Preview mostra: o do tocar, ou o activo do editor.
    #[must_use]
    pub fn shown(&self, sprite: &Sprite, active: FrameId) -> FrameId {
        self.play
            .as_ref()
            .and_then(|p| sprite.frames().get(p.index))
            .map_or(active, |f| f.id)
    }

    /// Publica a imagem do quadro mostrado — composta só quando `(doc, stamp, frame)` muda — ou
    /// `None` (janela fechada / sem sprite: o tocar pára).
    pub fn publish(&mut self, visible: bool, doc: Option<(u32, &Sprite, u64, FrameId)>) {
        let Some((key, sprite, stamp, active)) = doc.filter(|_| visible) else {
            self.play = None;
            self.chave = None;
            publish(None);
            return;
        };
        if self.doc.is_some_and(|d| d != key) {
            self.play = None;
        }
        let frame = self.shown(sprite, active);
        let chave = (key, stamp, frame.0);
        let playing = self.playing();
        if self.chave == Some(chave) {
            CURRENT.with(|c| {
                if let Some(f) = c.borrow_mut().as_mut() {
                    f.playing = playing;
                    f.checker = sprite.prefs().checker;
                }
            });
            return;
        }
        self.chave = Some(chave);
        self.seq += 1;
        let mut out = Vec::new();
        ph2d_pixel_core::compose_rgba(sprite, frame, None, &mut out);
        publish(Some(PreviewFrame {
            rgba: Arc::new(out),
            w: sprite.width(),
            h: sprite.height(),
            seq: self.seq,
            frame: frame.0,
            playing,
            checker: sprite.prefs().checker,
        }));
    }
}
