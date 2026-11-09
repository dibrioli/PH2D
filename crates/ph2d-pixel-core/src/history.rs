//! O histórico de um sprite: pilhas de inversos com tecto em BYTES (uma contagem é um
//! multiplicador, não um tecto — ADR-0117). Cada passo leva o NOME do que o artista fez (P12 H,
//! Aseprite O11–O14): a lista é linear (R2) e o tecto larga o mais velho com o nome dele (R3).

use crate::op::{Op, OpError, apply};
use crate::sprite::{FrameId, LayerId, Sprite};

static NEXT_REVISION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// A revisão do conteúdo gravado: um valor do PROCESSO (nunca se repete entre documentos), novo a
/// cada passo aplicado, desfeito ou refeito. Não é conteúdo: dois históricos iguais comparam-se
/// iguais seja qual for a revisão.
#[derive(Debug, Clone, Copy)]
struct Revision(u64);

impl Revision {
    fn fresh() -> Self {
        Self(NEXT_REVISION.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

impl PartialEq for Revision {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl Eq for Revision {}

/// O tecto por omissão: 64 MiB de RAM por sprite — o do MVP web (`docs/Pixel Art/app/engine/ops.js`,
/// `DEFAULT_CAP_BYTES`). O recurso é a memória do processo; o mais velho cai primeiro.
pub const DEFAULT_CAP_BYTES: usize = 64 * 1024 * 1024;

/// O nome de um passo: uma chave do `ph2d-i18n` (`pixel.history.*`), dada por quem grava.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Label(&'static str);

impl Label {
    /// A primeira linha da lista, sem op.
    pub const INITIAL: Self = Self("pixel.history.initial");

    #[must_use]
    pub const fn new(key: &'static str) -> Self {
        Self(key)
    }

    #[must_use]
    pub const fn key(self) -> &'static str {
        self.0
    }

    /// O nome por omissão de um op sem nome dado: a família do op.
    #[must_use]
    pub const fn of(op: &Op) -> Self {
        Self(match op {
            Op::Patch(_) => "pixel.history.draw",
            Op::Palette { .. } => "pixel.history.palette",
            Op::Remap { .. } => "pixel.history.remap",
            Op::Whole(_) => "pixel.history.sprite",
            Op::Layers(_) => "pixel.history.layers",
            Op::Frames(_) => "pixel.history.frames",
            Op::Mask(_) => "pixel.history.selection",
            Op::Canvas(_) => "pixel.history.canvas",
            Op::Tiles(_) => "pixel.history.tiles",
            Op::Slices(_) => "pixel.history.slices",
            Op::PivotSlice(_) => "pixel.history.pivot",
            Op::LayerNormal { .. } => "pixel.history.layer_normal",
            Op::UserData { .. } => "pixel.history.user_data",
            Op::Batch(_) => "pixel.history.transform",
        })
    }
}

/// Uma linha da lista: feita (até à actual) ou desfeita (por baixo, esmaecida).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    Done,
    Undone,
}

/// O cel que um passo mudou, para «ir ao quadro/camada mudados» (O14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Touched {
    pub layer: LayerId,
    pub frame: FrameId,
}

fn touched_of(op: &Op) -> Option<Touched> {
    match op {
        Op::Patch(p) => Some(Touched {
            layer: p.layer,
            frame: p.frame,
        }),
        Op::Batch(v) => v.iter().find_map(touched_of),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Step {
    op: Op,
    label: Label,
}

/// Desfazer/refazer de um sprite. Um gesto do artista = um op.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History {
    undo: Vec<Step>,
    redo: Vec<Step>,
    bytes: usize,
    cap_bytes: usize,
    revision: Revision,
}

impl Default for History {
    fn default() -> Self {
        Self::with_cap(DEFAULT_CAP_BYTES)
    }
}

impl History {
    #[must_use]
    pub fn with_cap(cap_bytes: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            bytes: 0,
            cap_bytes,
            revision: Revision::fresh(),
        }
    }

    /// Muda em todo passo que mexe no sprite por esta porta (commit, undo, redo, `go_to`) e em
    /// [`Self::touch`]; um passo recusado ou nada a desfazer/refazer deixa-a. A chave da cache do
    /// codec ([`crate::codec::EntryCache`]).
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision.0
    }

    /// Uma mudança gravável que não é passo do desfazer.
    pub fn touch(&mut self) {
        self.revision = Revision::fresh();
    }

    /// [`Self::commit_labeled`] com o nome da família do op.
    pub fn commit(&mut self, sprite: &mut Sprite, op: Op) -> Result<(), OpError> {
        let label = Label::of(&op);
        self.commit_labeled(sprite, op, label)
    }

    /// Aplica `op`, guarda o inverso com o nome; o refazer esvazia-se (lista linear, R2).
    pub fn commit_labeled(
        &mut self,
        sprite: &mut Sprite,
        op: Op,
        label: Label,
    ) -> Result<(), OpError> {
        let inverse = apply(sprite, op)?;
        self.touch();
        for s in self.redo.drain(..) {
            self.bytes -= s.op.bytes();
        }
        self.bytes += inverse.bytes();
        self.undo.push(Step { op: inverse, label });
        while self.bytes > self.cap_bytes && self.undo.len() > 1 {
            let dropped = self.undo.remove(0);
            self.bytes -= dropped.op.bytes();
        }
        Ok(())
    }

    /// Desfaz o último passo. `false` = nada a desfazer.
    pub fn undo(&mut self, sprite: &mut Sprite) -> bool {
        let feito = Self::swap(sprite, &mut self.undo, &mut self.redo, &mut self.bytes);
        if feito {
            self.touch();
        }
        feito
    }

    /// Refaz o último passo desfeito. `false` = nada a refazer.
    pub fn redo(&mut self, sprite: &mut Sprite) -> bool {
        let feito = Self::swap(sprite, &mut self.redo, &mut self.undo, &mut self.bytes);
        if feito {
            self.touch();
        }
        feito
    }

    fn swap(
        sprite: &mut Sprite,
        from: &mut Vec<Step>,
        to: &mut Vec<Step>,
        bytes: &mut usize,
    ) -> bool {
        let Some(Step { op, label }) = from.pop() else {
            return false;
        };
        *bytes -= op.bytes();
        match apply(sprite, op) {
            Ok(inv) => {
                *bytes += inv.bytes();
                to.push(Step { op: inv, label });
                true
            }
            // Um inverso que o próprio `apply` produziu não falha; se falhar o documento não mudou.
            Err(_) => false,
        }
    }

    /// As linhas da lista: `Initial State`, os feitos (o último é o actual), os desfeitos.
    pub fn entries(&self) -> impl Iterator<Item = (Label, StepState)> + '_ {
        std::iter::once(Label::INITIAL)
            .chain(self.undo.iter().map(|s| s.label))
            .map(|l| (l, StepState::Done))
            .chain(self.redo.iter().rev().map(|s| (s.label, StepState::Undone)))
    }

    /// Quantas linhas há, com o `Initial State`.
    #[must_use]
    pub fn len(&self) -> usize {
        1 + self.undo.len() + self.redo.len()
    }

    /// Nunca vazia: há sempre o `Initial State`.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }

    /// A linha actual (0 = `Initial State`).
    #[must_use]
    pub fn current(&self) -> usize {
        self.undo.len()
    }

    /// O cel que a linha `n` mudou, se o op o diz (`Patch`); o `Initial State` não mudou nada.
    #[must_use]
    pub fn touched(&self, n: usize) -> Option<Touched> {
        let k = self.undo.len();
        let step = match n {
            0 => return None,
            n if n <= k => &self.undo[n - 1],
            n => self.redo.get(self.redo.len().checked_sub(n - k)?)?,
        };
        touched_of(&step.op)
    }

    /// Desfaz ou refaz até a linha `n` ser a actual: igual a `|n − current|` passos. `false` = `n`
    /// fora da lista (nada muda) ou um passo falhou.
    pub fn go_to(&mut self, sprite: &mut Sprite, n: usize) -> bool {
        if n >= self.len() {
            return false;
        }
        while self.current() > n {
            if !self.undo(sprite) {
                return false;
            }
        }
        while self.current() < n {
            if !self.redo(sprite) {
                return false;
            }
        }
        true
    }

    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    #[must_use]
    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }
    #[must_use]
    pub fn bytes(&self) -> usize {
        self.bytes
    }
}

/// A memória no título, à Aseprite (`base::get_pretty_memory_size`): menos de 1000 = bytes;
/// menos de 1000×1000 = KiB com uma casa (`5.4K`); acima, MiB (`1.5M`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrettySize {
    Bytes(usize),
    K(f64),
    M(f64),
}

impl PrettySize {
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn of(bytes: usize) -> Self {
        if bytes < 1000 {
            Self::Bytes(bytes)
        } else if bytes < 1000 * 1000 {
            Self::K(bytes as f64 / 1024.0)
        } else {
            Self::M(bytes as f64 / (1024.0 * 1024.0))
        }
    }
}
