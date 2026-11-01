//! **A linha do tempo do lado da família** (P4): os pedidos do menu Frame e da linha do tempo (cada
//! acção um passo do desfazer do sprite, pelos construtores de `ph2d_pixel_core::frames`), a
//! navegação, o tocar (o relógio do host em ms, [`PixelApp::tick`]) e o papel vegetal.

use ph2d_editor_core::pixel_panel::{
    PixelCel, PixelDirection, PixelFrameAction as A, PixelFrameCol, PixelRequest, PixelTagInfo,
};
use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::play::{Play, PlayOptions};
use ph2d_pixel_core::{Direction, FrameId, LayerId, Onion, Op, Sprite};

use crate::PixelApp;

/// O nome de uma tag nova (Aseprite: «Tag»).
const NEW_TAG: &str = "Tag";

fn dir_in(d: PixelDirection) -> Direction {
    match d {
        PixelDirection::Forward => Direction::Forward,
        PixelDirection::Reverse => Direction::Reverse,
        PixelDirection::PingPong => Direction::PingPong,
        PixelDirection::PingPongReverse => Direction::PingPongReverse,
    }
}

fn dir_out(d: Direction) -> PixelDirection {
    match d {
        Direction::Forward => PixelDirection::Forward,
        Direction::Reverse => PixelDirection::Reverse,
        Direction::PingPong => PixelDirection::PingPong,
        Direction::PingPongReverse => PixelDirection::PingPongReverse,
    }
}

/// O instantâneo da linha do tempo de um sprite: colunas, cels e tags.
#[must_use]
pub fn timeline(s: &Sprite) -> (Vec<PixelFrameCol>, Vec<PixelCel>, Vec<PixelTagInfo>) {
    let cols = s
        .frames()
        .iter()
        .map(|f| PixelFrameCol {
            id: f.id.0,
            duration_ms: f.duration_ms,
        })
        .collect();
    let cels = s
        .entries()
        .map(|(&k, e)| {
            let img = s.cel(k);
            PixelCel {
                layer: k.0.0,
                frame: k.1.0,
                owner: s.owner(k).1.0,
                opacity: img.map_or(255, |c| c.opacity),
                z: e.z,
                empty: img.is_none_or(|c| c.bbox.is_empty()),
            }
        })
        .collect();
    let tags = s
        .tags()
        .iter()
        .map(|t| PixelTagInfo {
            name: t.name.clone(),
            from: t.from,
            to: t.to,
            direction: dir_out(t.direction),
            repeat: t.repeat,
            color: t.color,
        })
        .collect();
    (cols, cels, tags)
}

impl PixelApp {
    /// O papel vegetal a desenhar no sprite `key` (ligado, em Draw e parado).
    #[must_use]
    pub(crate) fn onion_view(&self, key: u32, editing: bool) -> Option<Onion> {
        let parado = self.docs.get(&key).is_some_and(|d| d.play.is_none());
        (self.onion_on && editing && parado).then_some(self.onion)
    }

    /// Anda o tocar do sprite em Draw `dt_ms` no relógio do host.
    pub fn tick(&mut self, dt_ms: u32) {
        self.formigas_ms = self.formigas_ms.wrapping_add(u64::from(dt_ms));
        let Some(doc) = self.chave.and_then(|k| self.docs.get_mut(&k)) else {
            return;
        };
        let Some(p) = doc.play.as_mut() else {
            return;
        };
        p.advance(&doc.sprite, dt_ms);
        if let Some(f) = doc.sprite.frames().get(p.index) {
            doc.frame = f.id;
        }
        if !p.playing {
            doc.play = None;
        }
    }

    /// Aplica um pedido da linha do tempo; `false` = não é dela.
    pub(crate) fn aplica_quadro(&mut self, r: &PixelRequest) -> bool {
        let Some(key) = self.chave else {
            return matches!(r, PixelRequest::FrameAction(A::Onion | A::OnionTint))
                && self.alterna_onion(r);
        };
        match r {
            PixelRequest::FrameAction(A::Onion | A::OnionTint) => self.alterna_onion(r),
            PixelRequest::FrameAction(A::Play) => {
                let t = self.tocar;
                if let Some(doc) = self.docs.get_mut(&key) {
                    match doc.play.take() {
                        // *Rewind on Stop*: parar volta ao quadro onde o tocar começou.
                        Some(p) if t.rewind => {
                            if let Some(f) = doc.sprite.frames().get(p.start) {
                                doc.frame = f.id;
                            }
                        }
                        Some(_) => {}
                        None => {
                            let i = doc.sprite.frame_index(doc.frame).unwrap_or(0);
                            let opt = PlayOptions {
                                once: t.once,
                                all: t.all,
                                subtags: t.subtags,
                            };
                            let mut p = Play::start(&doc.sprite, i, opt);
                            p.speed = velocidade(doc.speed);
                            if let Some(f) = doc.sprite.frames().get(p.index) {
                                doc.frame = f.id;
                            }
                            doc.play = Some(p);
                        }
                    }
                }
                true
            }
            PixelRequest::Playback(o) => {
                self.tocar = *o;
                if let Some(doc) = self.docs.get_mut(&key) {
                    doc.speed = o.speed;
                    if let Some(p) = doc.play.as_mut() {
                        p.speed = velocidade(o.speed);
                    }
                }
                true
            }
            PixelRequest::FrameAction(a @ (A::FirstInTag | A::LastInTag)) => {
                let ultimo = *a == A::LastInTag;
                self.vai(key, move |s, i| {
                    let (a, b) = ph2d_pixel_core::play::tag_at(s.tags(), i)
                        .map_or((0, s.frames().len().saturating_sub(1)), |t| {
                            (s.tags()[t].from as usize, s.tags()[t].to as usize)
                        });
                    Some(if ultimo { b } else { a })
                })
            }
            PixelRequest::FrameSelect(f) => {
                self.vai(key, |s, _| s.frames().iter().position(|x| x.id.0 == *f))
            }
            PixelRequest::CelSelect { layer, frame } => {
                let _ = self.aplica_camada(&PixelRequest::LayerSelect(*layer));
                self.vai(key, |s, _| s.frames().iter().position(|x| x.id.0 == *frame))
            }
            PixelRequest::FrameAction(a @ (A::First | A::Previous | A::Next | A::Last)) => {
                let a = *a;
                self.vai(key, move |s, i| {
                    let n = s.frames().len();
                    Some(match a {
                        A::First => 0,
                        A::Last => n - 1,
                        A::Previous => (i + n - 1) % n,
                        _ => (i + 1) % n,
                    })
                })
            }
            PixelRequest::FrameAction(a) => {
                self.acao(key, *a);
                true
            }
            // Com intervalo, todos os quadros dele e só eles (o 3 fica intacto em {2,4}).
            PixelRequest::FrameDuration(ms) => {
                let ids = self.docs.get(&key).map(crate::state::Doc::alvo_quadros);
                let ids = ids.unwrap_or_default();
                self.faz(key, |s, _, _| frames::set_duration(s, &ids, *ms))
            }
            PixelRequest::ConstantFrameRate(ms) => self.faz(key, |s, _, _| {
                let ids: Vec<FrameId> = s.frames().iter().map(|f| f.id).collect();
                frames::set_duration(s, &ids, *ms)
            }),
            PixelRequest::CelOpacity(o) => {
                self.faz(key, |s, f, l| frames::set_cel(s, (l, f), Some(*o), None))
            }
            PixelRequest::CelZ(z) => {
                self.faz(key, |s, f, l| frames::set_cel(s, (l, f), None, Some(*z)))
            }
            PixelRequest::TagDirection { index, direction } => {
                let d = dir_in(*direction);
                self.faz(key, |s, _, _| {
                    frames::edit_tag(s, *index, |t| t.direction = d)
                })
            }
            PixelRequest::TagRepeat { index, repeat } => self.faz(key, |s, _, _| {
                frames::edit_tag(s, *index, |t| t.repeat = *repeat)
            }),
            // A lei do diálogo do Trial: `to < from` → `to = from` (sem troca); ambos presos ao último.
            PixelRequest::TagRange { index, from, to } => self.faz(key, |s, _, _| {
                let last = u32::try_from(s.frames().len().saturating_sub(1)).unwrap_or(u32::MAX);
                let from = (*from).min(last);
                let to = (*to).max(from).min(last);
                frames::edit_tag(s, *index, |t| (t.from, t.to) = (from, to))
            }),
            PixelRequest::TagRename { index, name } if !name.trim().is_empty() => self
                .faz(key, |s, _, _| {
                    frames::edit_tag(s, *index, |t| t.name = name.trim().to_owned())
                }),
            PixelRequest::TagRename { .. } => true,
            PixelRequest::TagDelete(i) => self.faz(key, |s, _, _| frames::delete_tag(s, *i)),
            _ => false,
        }
    }

    fn alterna_onion(&mut self, r: &PixelRequest) -> bool {
        if matches!(r, PixelRequest::FrameAction(A::OnionTint)) {
            self.onion.tint = !self.onion.tint;
        } else {
            self.onion_on = !self.onion_on;
        }
        true
    }

    /// Muda o frame activo para a posição que `onde(sprite, posição de agora)` der (pára o tocar).
    fn vai(&mut self, key: u32, onde: impl FnOnce(&Sprite, usize) -> Option<usize>) -> bool {
        if let Some(doc) = self.docs.get_mut(&key) {
            doc.play = None;
            let i = doc.sprite.frame_index(doc.frame).unwrap_or(0);
            if let Some(f) = onde(&doc.sprite, i).and_then(|j| doc.sprite.frames().get(j)) {
                doc.frame = f.id;
            }
        }
        true
    }

    /// Um passo do desfazer feito pelo construtor `f(sprite, frame activo, camada activa)`.
    fn faz(&mut self, key: u32, f: impl FnOnce(&Sprite, FrameId, LayerId) -> Option<Op>) -> bool {
        let op = self
            .docs
            .get(&key)
            .and_then(|d| f(&d.sprite, d.frame, d.layer));
        if let Some(op) = op {
            let _ = self.commit(key, op);
        }
        true
    }

    fn acao(&mut self, key: u32, a: A) {
        let Some(doc) = self.docs.get_mut(&key) else {
            return;
        };
        doc.play = None;
        let (f, l) = (doc.frame, doc.layer);
        let i = doc.sprite.frame_index(f).unwrap_or(0);
        let (alvo, span, keys) = (doc.alvo_quadros(), doc.range_span(), doc.range_keys());
        let pos: Vec<usize> = (doc.range_frames().iter())
            .filter_map(|&f| doc.sprite.frame_index(f))
            .collect();
        let envelope = pos
            .iter()
            .min()
            .zip(pos.iter().max())
            .map(|(a, b)| (*a, *b));
        let picks = doc.range_frames().len();
        let s = &mut doc.sprite;
        let novo = |s: &mut Sprite, k| frames::new_frame(s, f, k);
        let (op, vai_para) = match a {
            A::NewFrame => novo(s, NewFrame::Copy).map(|(o, id)| (o, Some(id))).unzip(),
            A::NewEmptyFrame => novo(s, NewFrame::Empty)
                .map(|(o, id)| (o, Some(id)))
                .unzip(),
            A::DuplicateLinked => novo(s, NewFrame::Linked)
                .map(|(o, id)| (o, Some(id)))
                .unzip(),
            A::Delete => (frames::delete(s, &alvo), None),
            A::MoveLeft => (
                i.checked_sub(1).and_then(|j| frames::move_to(s, f, j)),
                None,
            ),
            A::MoveRight => (frames::move_to(s, f, i + 1), None),
            // Com intervalo: o envelope dele (cels e durações juntos); um quadro só = cinzento.
            A::Reverse if picks > 0 => (span.and_then(|(a, b)| frames::reverse(s, a, b)), None),
            A::Reverse => {
                let (a, b) = ph2d_pixel_core::play::tag_at(s.tags(), i)
                    .map_or((0, s.frames().len() - 1), |t| {
                        (s.tags()[t].from as usize, s.tags()[t].to as usize)
                    });
                (frames::reverse(s, a, b), None)
            }
            // Link Cels sobre o intervalo: um cel ligado por camada.
            A::LinkPrevious if picks >= 2 => (link_range(s, &keys), None),
            A::LinkPrevious => {
                let prev = i.checked_sub(1).map(|j| s.frames()[j].id);
                (prev.and_then(|p| frames::link(s, l, &[p, f])), None)
            }
            A::Unlink => (frames::unlink(s, (l, f)), None),
            A::NewTag => (frames::new_tag(s, NEW_TAG, i as u32, i as u32), None),
            // *Set Loop Section*: a tag «Loop» cobre o intervalo; sem intervalo, nada.
            A::LoopSection => (envelope.and_then(|(a, b)| loop_section(s, a, b)), None),
            _ => (None, None),
        };
        if let Some(op) = op {
            let _ = self.commit(key, op);
            if let (Some(id), Some(doc)) = (vai_para.flatten(), self.docs.get_mut(&key)) {
                doc.frame = id;
            }
        }
    }
}

/// Cel ▸ Link Cels sobre um intervalo: os cels de cada camada passam a um só ligado (um passo).
fn link_range(s: &Sprite, keys: &[ph2d_pixel_core::CelKey]) -> Option<Op> {
    let mut camadas: Vec<LayerId> = keys.iter().map(|k| k.0).collect();
    camadas.dedup();
    let ops: Vec<Op> = camadas
        .into_iter()
        .filter_map(|l| {
            let fs: Vec<FrameId> = keys.iter().filter(|k| k.0 == l).map(|k| k.1).collect();
            frames::link(s, l, &fs)
        })
        .collect();
    (!ops.is_empty()).then_some(Op::Batch(ops))
}

/// A velocidade do chrome no vocabulário do núcleo.
fn velocidade(v: ph2d_editor_core::pixel_panel::PixelSpeed) -> ph2d_pixel_core::play::PlaySpeed {
    use ph2d_editor_core::pixel_panel::PixelSpeed as V;
    use ph2d_pixel_core::play::PlaySpeed as P;
    match v {
        V::Quarter => P::Quarter,
        V::Half => P::Half,
        V::One => P::One,
        V::OneAndHalf => P::OneAndHalf,
        V::Two => P::Two,
        V::Three => P::Three,
    }
}

/// O nome da tag do *Set Loop Section* (o do Aseprite: é um nome do documento, não da UI).
const LOOP_TAG: &str = "Loop";

/// A tag «Loop» passa a `a..=b` (Forward, para sempre); nasce se não existe (o Trial: chamar de
/// novo MOVE a mesma).
fn loop_section(s: &Sprite, a: usize, b: usize) -> Option<Op> {
    let (a, b) = (u32::try_from(a).ok()?, u32::try_from(b).ok()?);
    match s.tags().iter().position(|t| t.name == LOOP_TAG) {
        Some(i) => frames::edit_tag(s, i, |t| {
            t.from = a;
            t.to = b;
        }),
        None => frames::new_tag(s, LOOP_TAG, a, b),
    }
}
