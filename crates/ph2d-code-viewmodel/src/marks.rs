//! As marcas de um problema na calha (o que a família entrega à pintura, `ph2d-panel-code::frame`).

use std::ops::Range;

/// Gravidade de uma marca na calha (a do LSP: informação e dica são `Info`, que a barra não conta).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

/// Um problema numa linha (0-based): a calha marca a linha; `culprit` (bytes do buffer) é o
/// símbolo culpado, sublinhado no lugar exacto. `message` é a mensagem crua (a do Luau, ou a nossa);
/// `explanation`, a escrita à mão para ela (I8), quando a classe tem uma.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    pub line: usize,
    pub severity: Severity,
    pub culprit: Option<Range<usize>>,
    pub message: String,
    pub explanation: Option<Explanation>,
    /// Em quem aconteceu (os objectos do Play, «Hero, Hero 2 +1»); vazio = ninguém sabe («—»).
    pub on: String,
}

/// A explicação de um problema (I8): o título numa linha e o porquê com o que fazer. `kind` é o nome
/// estável da classe (o registo local das explicações vistas grava-o).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Explanation {
    pub kind: &'static str,
    pub title: String,
    pub why: String,
}

/// Uma linha da prévia de uma acção: o texto como fica e o que nele muda (bytes do texto).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewLine {
    pub text: String,
    pub changed: Vec<Range<usize>>,
}
