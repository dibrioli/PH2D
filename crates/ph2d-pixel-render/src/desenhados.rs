//! **Os Pixel Sprites que o último quadro desenhou** (saído de `ph2d-app-pixel::draw`, 09/11): o
//! clique e a caixa do gizmo perguntam AQUI, à mesma geometria que a tinta usou (nunca uma segunda
//! conta). A família escreve; a composição pergunta.

use std::cell::RefCell;

/// Um Pixel Sprite como o último quadro o desenhou.
#[derive(Debug, Clone, Copy)]
pub struct Drawn {
    pub bits: u64,
    pub to_screen: ph2d_vector::Affine,
    pub size: (u32, u32),
    /// A origem (o pivô, píxeis do sprite) com que `to_screen` foi feito.
    pub pivo: (f64, f64),
}

thread_local! {
    static DRAWN: RefCell<Vec<Drawn>> = const { RefCell::new(Vec::new()) };
    /// A arte do EDITOR já desenhada nas faixas deste quadro (`Some`) — o desenho do chrome não a repete.
    static PENDING: RefCell<Option<Vec<Drawn>>> = const { RefCell::new(None) };
}

/// Os Pixel Sprites sob o ponto de ecrã `p` (dentro do canvas), do de cima para o de baixo.
#[must_use]
pub fn pick_at(p: (f32, f32)) -> Vec<u64> {
    DRAWN.with(|d| {
        d.borrow()
            .iter()
            .rev()
            .filter(|s| {
                crate::cell_at(s.to_screen, (f64::from(p.0), f64::from(p.1))).is_some_and(
                    |(x, y)| x >= 0 && y >= 0 && (x as u32) < s.size.0 && (y as u32) < s.size.1,
                )
            })
            .map(|s| s.bits)
            .collect()
    })
}

/// A caixa do gizmo de objecto do Pixel Sprite `bits` no espaço LOCAL (metros, antes da escala,
/// Y para cima): `(centro relativo à origem = o pivô, meia-extensão)`. `None` = não foi desenhado
/// no último quadro.
#[must_use]
pub fn caixa_local(bits: u64, ppm: f32) -> Option<([f32; 2], [f32; 2])> {
    let ppm = ppm.max(f32::MIN_POSITIVE);
    DRAWN.with(|d| {
        d.borrow().iter().find(|s| s.bits == bits).map(|s| {
            let (hw, hh) = (s.size.0 as f32 * 0.5, s.size.1 as f32 * 0.5);
            let (px, py) = (s.pivo.0 as f32, s.pivo.1 as f32);
            ([(hw - px) / ppm, (py - hh) / ppm], [hw / ppm, hh / ppm])
        })
    })
}

/// Os `bits` de todo Pixel Sprite desenhado no último quadro (o escondido não entra).
#[must_use]
pub fn desenhados() -> Vec<u64> {
    DRAWN.with(|d| d.borrow().iter().map(|s| s.bits).collect())
}

/// Início do quadro: nenhuma arte nas faixas ainda (chamado antes de codificar as faixas).
pub fn begin_frame() {
    PENDING.with(|p| *p.borrow_mut() = None);
}

/// A arte do editor desenhada numa faixa: o chrome do quadro não a repete.
pub fn junta_pendentes(drawn: Vec<Drawn>) {
    PENDING.with(|p| p.borrow_mut().get_or_insert_with(Vec::new).extend(drawn));
}

/// A arte já desenhada nas faixas deste quadro (`None` = nenhuma faixa a levou).
#[must_use]
pub fn tira_pendentes() -> Option<Vec<Drawn>> {
    PENDING.with(|p| p.borrow_mut().take())
}

/// O que o quadro desenhou passa a ser o que o clique pergunta.
pub fn regista(drawn: Vec<Drawn>) {
    DRAWN.with(|d| *d.borrow_mut() = drawn);
}
