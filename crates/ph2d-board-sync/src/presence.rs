//! **A presença** (ADR-0181 §6): o cursor, a selecção e a câmara de cada pessoa. Não é documento —
//! sem carimbo, o último ganha, nunca vai para o ficheiro nem para o desfazer. Os dois oráculos
//! mostram o cursor alheio aos SALTOS (~23–30 Hz, `colab_oracle` c18); aqui o receptor INTERPOLA:
//! desenha o outro um pouco no passado, entre as duas amostras que o cercam.

use std::collections::VecDeque;

use ph2d_board_model::{Camera, ElementId};
use serde::{Deserialize, Serialize};

use crate::PeerId;

/// De quanto em quanto tempo a presença sai, no máximo (ms): os ~30 Hz dos dois oráculos (c18).
pub const SEND_EVERY_MS: u64 = 33;

/// Quanto no passado se desenha o outro (ms): 1,5 intervalos de envio — há sempre uma amostra depois
/// do instante desenhado, mesmo com uma chegada atrasada meio intervalo.
pub const RENDER_DELAY_MS: u64 = SEND_EVERY_MS * 3 / 2;

/// O que uma pessoa mostra de si.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Presence {
    pub peer: PeerId,
    /// Onde está o rato no mundo do quadro (`None` = fora do quadro).
    pub cursor: Option<[f64; 2]>,
    pub selection: Vec<ElementId>,
    pub camera: Camera,
}

/// As últimas amostras de um valor no tempo de CHEGADA, para desenhar entre elas.
#[derive(Clone, Debug, Default)]
pub struct Track<const N: usize> {
    samples: VecDeque<(u64, [f64; N])>,
}

impl<const N: usize> Track<N> {
    const KEEP: usize = 8;

    /// Uma amostra que chegou em `at_ms`.
    pub fn push(&mut self, at_ms: u64, v: [f64; N]) {
        if self.samples.back().is_some_and(|(t, _)| *t > at_ms) {
            return;
        }
        self.samples.push_back((at_ms, v));
        while self.samples.len() > Self::KEEP {
            self.samples.pop_front();
        }
    }

    /// O valor a desenhar em `now_ms`: o de `now − RENDER_DELAY_MS`, interpolado; antes da 1.ª a
    /// 1.ª, depois da última a última (nunca se adivinha para a frente).
    #[must_use]
    pub fn at(&self, now_ms: u64) -> Option<[f64; N]> {
        let t = now_ms.saturating_sub(RENDER_DELAY_MS);
        let (&(t1, v1), rest) = (self.samples.back()?, &self.samples);
        if t >= t1 {
            return Some(v1);
        }
        let mut prev = rest.front().copied()?;
        if t <= prev.0 {
            return Some(prev.1);
        }
        for &(tb, vb) in rest.iter().skip(1) {
            if t <= tb {
                let (ta, va) = prev;
                let k = (t - ta) as f64 / (tb - ta).max(1) as f64;
                return Some(std::array::from_fn(|i| va[i] + (vb[i] - va[i]) * k));
            }
            prev = (tb, vb);
        }
        Some(v1)
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

/// O que se sabe de outra pessoa, para desenhar.
#[derive(Clone, Debug, Default)]
pub struct Seen {
    pub last: Option<Presence>,
    pub cursor: Track<2>,
    /// A câmara como `[centro x, centro y, ln zoom]` — o zoom interpola-se em escala logarítmica.
    pub camera: Track<3>,
}

impl Seen {
    pub fn push(&mut self, at_ms: u64, p: Presence) {
        match p.cursor {
            Some(c) => self.cursor.push(at_ms, c),
            None => self.cursor.clear(),
        }
        let c = p.camera;
        self.camera
            .push(at_ms, [c.center_x, c.center_y, c.zoom.ln()]);
        self.last = Some(p);
    }

    /// O cursor a desenhar agora.
    #[must_use]
    pub fn cursor_at(&self, now_ms: u64) -> Option<[f64; 2]> {
        self.last.as_ref()?.cursor?;
        self.cursor.at(now_ms)
    }

    /// A câmara de quem se segue, agora.
    #[must_use]
    pub fn camera_at(&self, now_ms: u64) -> Option<Camera> {
        let [x, y, z] = self.camera.at(now_ms)?;
        Some(Camera {
            center_x: x,
            center_y: y,
            zoom: z.exp(),
        })
    }
}
