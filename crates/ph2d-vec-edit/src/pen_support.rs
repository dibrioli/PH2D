#![forbid(unsafe_code)]
//! ph2d-vec-edit — pen STYLE + edit HISTORY (sibling of `lib.rs` for the HR-18 file-LOC cap).
//! `PenStyle` is the pen's stroke/fill/cap/join/dash config; `History` is the snapshot-based
//! undo/redo stack over `VecScene`. Both are `pub` and re-exported by `lib.rs`, so `crate::PenStyle`
//! / `crate::History` keep resolving for `shape.rs` and the tests. No `PenTool` coupling.

use ph2d_vec_scene::{LineCap, LineJoin, Rgba8, StrokeSpec, VecScene};

/// Cor do traço do Pen (claro, sobre o canvas escuro).
const PEN_STROKE: Rgba8 = Rgba8::new(240, 240, 245, 255);

/// Preenchimento leve aplicado ao fechar o path.
const PEN_FILL: Rgba8 = Rgba8::new(90, 150, 230, 120);

/// Teto de passos de undo (memória limitada; snapshots são baratos p/ cenas de
/// desenho, mas não infinitos).
const HISTORY_CAP: usize = 256;

/// Estilo aplicado a paths **recém-criados** pelo Pen. O shell sincroniza a partir
/// da tool (`ph2d-tool-vector`): traço, largura em px, e o fill usado ao fechar.
/// Default = as cores de scaffold da Fase 1.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PenStyle {
    /// Cor do traço dos paths desenhados.
    pub stroke: Rgba8,
    /// Largura do traço em **pixels de tela** (o shell multiplica por `px_to_world`).
    pub stroke_w_px: f64,
    /// Preenchimento aplicado ao FECHAR um path.
    pub fill: Rgba8,
    /// Ponta / junção do traço.
    pub cap: LineCap,
    pub join: LineJoin,
    /// Dash/vão como **múltiplos da largura**: `Some((dash, gap))` ⇒ traço e vão
    /// de `dash·width`/`gap·width`; `None` = contínuo. O render multiplica pela
    /// largura do path.
    pub dash: Option<(f64, f64)>,
}

impl Default for PenStyle {
    fn default() -> Self {
        Self {
            stroke: PEN_STROKE,
            stroke_w_px: 3.0,
            fill: PEN_FILL,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            dash: None,
        }
    }
}

impl PenStyle {
    /// `StrokeSpec` do traço para `width` world-units (aplica cap/join/dash).
    #[must_use]
    pub fn stroke_spec(&self, width: f64) -> StrokeSpec {
        StrokeSpec {
            color: self.stroke,
            width,
            cap: self.cap,
            join: self.join,
            dash: self.dash,
        }
    }
}

/// Undo/redo por **snapshot** da `VecScene` inteira (ADR-0108 Fase 2). Barato: a
/// cena é `Clone`. Uso: `begin` no início de uma interação (Down), `commit_if_changed`
/// no fim (Up) → só vira passo se a cena mudou de fato; ou `push_undo(pre)` direto
/// numa operação atômica (booleana/delete/load).
#[derive(Default)]
pub struct History {
    undo: Vec<VecScene>,
    redo: Vec<VecScene>,
    pending: Option<VecScene>,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot tentativo do estado ATUAL, antes de uma interação que pode mutar.
    pub fn begin(&mut self, scene: &VecScene) {
        self.pending = Some(scene.clone());
    }

    /// Fecha a interação: se a cena difere do snapshot de `begin`, vira um passo de
    /// undo (e limpa o redo). Se nada mudou, descarta o snapshot (não polui o histórico).
    pub fn commit_if_changed(&mut self, scene: &VecScene) {
        if let Some(pre) = self.pending.take()
            && &pre != scene
        {
            self.push_undo(pre);
        }
    }

    /// Descarta o snapshot pendente de `begin` SEM virar passo de undo. Usado quando
    /// uma interação é cancelada e o efeito colateral (ex.: um shape descartado que
    /// já consumiu um `next_id`) não deve poluir o histórico com um passo espúrio.
    pub fn cancel(&mut self) {
        self.pending = None;
    }

    /// Empurra um estado-pré direto pro undo (operação atômica que já sabe que mutou).
    pub fn push_undo(&mut self, pre: VecScene) {
        if self.undo.len() >= HISTORY_CAP {
            self.undo.remove(0);
        }
        self.undo.push(pre);
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Desfaz: devolve o estado anterior; empurra o `current` pro redo.
    pub fn undo(&mut self, current: &VecScene) -> Option<VecScene> {
        let prev = self.undo.pop()?;
        self.redo.push(current.clone());
        Some(prev)
    }

    /// Refaz: devolve o próximo estado; empurra o `current` de volta pro undo.
    pub fn redo(&mut self, current: &VecScene) -> Option<VecScene> {
        let next = self.redo.pop()?;
        self.undo.push(current.clone());
        Some(next)
    }
}
