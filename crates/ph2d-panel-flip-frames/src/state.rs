//! O snapshot que o shell publica por frame + o estado (vazio) do painel.
//!
//! O painel é **stateless** (espelho do `ph2d-panel-flip`): o documento (`FlipDoc`),
//! o playhead e os flags de autoria vivem no shell, que publica um
//! [`FlipStripSnapshot`] ANTES do paint. As edições saem por `ToolPanelEvent` e o
//! drain do shell as aplica. Assim o painel não depende do modelo do Flip — e o
//! undo/save continuam vendo UMA fonte de verdade.

use std::cell::RefCell;

/// Uma célula da tira: uma chave real da camada ativa.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlipCell {
    /// O quadro em que a chave começa.
    pub key: i32,
    /// Quantos quadros ela ocupa (a EXPOSIÇÃO, à TVPaint). A última chave mostra
    /// `1` enquanto nada a delimita.
    pub exposure: u32,
    /// É um breakdown (inbetween gerado pelo tween) — pinta mais discreto.
    pub breakdown: bool,
    /// O desenho é compartilhado por mais de uma chave (instância/ciclo).
    pub instanced: bool,
    /// **Marcada para o MULTIFRAME** (W7): o mesmo gesto de escultura/balde age em todas
    /// as chaves marcadas. Sem o realce a feature seria invisível — o usuário marcaria
    /// quadros sem saber quais, e um gesto agiria onde ele não vê.
    pub selected: bool,
}

/// Tudo que a tira pinta, publicado pelo shell a cada frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FlipStripSnapshot {
    /// Há um objeto Flip com camada ativa? `false` ⇒ a tira pinta só o chrome.
    pub has_layer: bool,
    /// Nome da camada ativa (a tira mostra UMA camada — a que se está animando).
    pub layer_name: String,
    /// As células, em ordem de quadro.
    pub cells: Vec<FlipCell>,
    /// O quadro do playhead (pode cair no meio de uma exposição).
    pub current_frame: i32,
    /// A chave ATIVA (a que o quadro atual está mostrando), se houver.
    pub current_key: Option<i32>,
    /// O transporte está tocando?
    pub playing: bool,
    /// FPS do objeto.
    pub fps: f32,
    /// Ghost Frames ligados + quantos de cada lado.
    pub ghost: bool,
    pub ghost_before: u32,
    pub ghost_after: u32,
    /// Autokey (desenhar no rabo de um hold cria chave) + Additive (a chave nova
    /// nasce como CÓPIA, não em branco).
    pub autokey: bool,
    pub additive: bool,
    /// Quantos inbetweens o botão Tween vai gerar.
    pub tween_count: u32,
    /// Ciclo (post behavior) da camada ativa — `CycleMode as u8`.
    pub cycle: u8,
    /// **Falloff temporal** do multiframe: os vizinhos recebem menos influência que o
    /// quadro ativo. Só PINCÉIS o respeitam (o balde é discreto).
    pub falloff: bool,
}

thread_local! {
    static CURRENT: RefCell<FlipStripSnapshot> = RefCell::new(FlipStripSnapshot::default());
}

/// Publica o snapshot da tira (o shell chama 1× por frame com a tool ativa).
pub fn set_current_flip_strip(snap: FlipStripSnapshot) {
    CURRENT.with(|c| *c.borrow_mut() = snap);
}

/// O snapshot deste frame.
#[must_use]
pub fn current_flip_strip() -> FlipStripSnapshot {
    CURRENT.with(|c| c.borrow().clone())
}

/// Estado retido do painel — vazio de propósito (a verdade está no shell).
#[derive(Clone, Debug, Default)]
pub struct FlipStripState;
