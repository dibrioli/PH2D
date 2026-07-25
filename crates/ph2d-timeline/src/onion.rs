//! **As configurações do onion da timeline** (ADR-0142) — dados puros, para o shell (que
//! desenha os fantasmas), o [`TimelineState`](crate::TimelineState) (que os guarda), o
//! `apply_intent` (que os edita) e o painel (que os controla) falarem UMA língua.
//!
//! É estado de VISTA, não de documento: nasce desligado e NÃO é serializado (a resposta a
//! *"o que a tela mostra"* não muda sozinha após um load — a classe do toggle Physics).
//! O motor de fantasmas (`RenderInstance`, silhueta) mora no shell; aqui só os números.

/// **O que os fantasmas vizinhos SÃO** (ADR-0142 §4). O onion serve os dois fluxos de um
/// timeline de keyframes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnionMode {
    /// Fantasmas a `t ± k` QUADROS — mostra o espaçamento dos inbetweens (o ritmo).
    Frames,
    /// Fantasmas nas KEYFRAMES vizinhas — o pose-a-pose, o modelo do animador. O default.
    Keys,
}

/// Como os fantasmas do onion da timeline se parecem e quantos são.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OnionSettings {
    /// Desligado por default: um onion que se arma sozinho é cena que já mudou ao olhar.
    pub enabled: bool,
    /// Quantos quadros/keyframes ANTES do playhead ganham fantasma.
    pub frames_before: u32,
    /// Quantos DEPOIS.
    pub frames_after: u32,
    /// A opacidade do fantasma mais PRÓXIMO; os mais distantes desvanecem a partir dela.
    pub opacity: f32,
    /// A cor (RGB) de um fantasma do passado — frio, o vocabulário do Flip.
    pub color_before: [f32; 3],
    /// A cor de um fantasma do futuro — o azul do Flip.
    pub color_after: [f32; 3],
    /// Quadros por segundo, para converter `frames_before/after` em tempo de clip
    /// (modo `Frames`).
    pub fps: f64,
    /// Fantasmas por QUADRO ou por KEYFRAME (ADR-0142 §4). Default `Keys`.
    pub mode: OnionMode,
}

impl Default for OnionSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            frames_before: 2,
            frames_after: 2,
            opacity: 0.5,
            // Os defaults do `ph2d_flip::OnionSettings` (ADR-0142 §3): um vocabulário de
            // fantasma no app inteiro (passado verde, futuro azul).
            color_before: [0.145, 0.420, 0.137],
            color_after: [0.125, 0.082, 0.529],
            fps: 24.0,
            // Pose-a-pose é o modelo do animador num timeline de keyframes (ADR-0142 §4).
            mode: OnionMode::Keys,
        }
    }
}
