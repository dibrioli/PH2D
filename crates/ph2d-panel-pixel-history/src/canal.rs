//! O canal entre a família e a janela: a vista da história, e os pedidos.

use std::cell::RefCell;

use ph2d_i18n::tr;
use ph2d_pixel_core::{History, PrettySize, StepState};

/// Uma linha: a chave i18n do nome e se está feita ou desfeita.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Row {
    pub label: &'static str,
    pub state: StepState,
}

/// O que a janela mostra.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HistoryView {
    /// Vazia = sem sprite em edição: a janela não se pinta.
    pub rows: Vec<Row>,
    pub current: usize,
    pub bytes: usize,
}

impl HistoryView {
    #[must_use]
    pub fn of(h: &History) -> Self {
        Self {
            rows: h
                .entries()
                .map(|(l, state)| Row {
                    label: l.key(),
                    state,
                })
                .collect(),
            current: h.current(),
            bytes: h.bytes(),
        }
    }
}

/// Um gesto da janela.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryRequest {
    /// Desfazer/refazer até a linha `n` ser a actual (`History::go_to`).
    GoTo(usize),
}

/// A memória como o título a mostra: `0 bytes`, `5.4K`, `1.5M` (Aseprite O11).
#[must_use]
pub fn pretty_size(bytes: usize) -> String {
    match PrettySize::of(bytes) {
        PrettySize::Bytes(n) => format!("{n} {}", tr("panel.pixel.history.bytes")),
        PrettySize::K(k) => format!("{k:.1}{}", tr("panel.pixel.history.kib")),
        PrettySize::M(m) => format!("{m:.1}{}", tr("panel.pixel.history.mib")),
    }
}

/// O título da janela: `Undo History (5.4K)`.
#[must_use]
pub fn title(bytes: usize) -> String {
    format!(
        "{} ({})",
        tr("panel.pixel_history.title"),
        pretty_size(bytes)
    )
}

#[derive(Default)]
struct Canal {
    view: HistoryView,
    requests: Vec<HistoryRequest>,
}

thread_local! {
    static CANAL: RefCell<Canal> = RefCell::new(Canal::default());
}

fn com<R>(f: impl FnOnce(&mut Canal) -> R) -> R {
    CANAL.with(|c| f(&mut c.borrow_mut()))
}

pub fn publish_view(v: HistoryView) {
    com(|c| c.view = v);
}

#[must_use]
pub fn view() -> HistoryView {
    com(|c| c.view.clone())
}

pub fn request(r: HistoryRequest) {
    com(|c| c.requests.push(r));
}

/// Toma (e limpa) os pedidos deste quadro.
#[must_use]
pub fn take_requests() -> Vec<HistoryRequest> {
    com(|c| std::mem::take(&mut c.requests))
}
