//! ⭐ **O CONTRATO entre a família e a pintura** — o que `ph2d-app-code` entrega em cada quadro. A
//! pintura não conhece a gramática, o Luau nem a cena: só isto.

use crate::style::Ink;
use ph2d_code_buffer::Buffer;
use std::ops::Range;

/// Gravidade de uma marca na calha.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Warning,
    Error,
}

/// Um problema numa linha (0-based): a calha marca a linha; `culprit` (bytes do buffer) é o
/// símbolo culpado, sublinhado no lugar exacto.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    pub line: usize,
    pub severity: Severity,
    pub culprit: Option<Range<usize>>,
    pub message: String,
}

/// Uma dobra: as linhas `start+1 ..= end` (0-based) escondidas quando `folded`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoldMark {
    pub start: usize,
    pub end: usize,
    pub folded: bool,
}

/// O selo da barra de estado (o resultado do último salvar).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Badge {
    /// «Reloaded · N objects».
    Reloaded { objects: usize },
    /// O texto não compila: a versão anterior continua; `line` 1-based.
    Refused { line: Option<u32> },
    /// Gravado, e nenhum objecto corre este script (não há recarga a relatar).
    Saved,
}

/// O assistente de código (o `luau-lsp`) quando NÃO está pronto — a barra di-lo; pronto, cala-se.
/// Em todos os casos o editor continua a editar, realçar e salvar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Assist {
    Starting,
    /// Morreu depois de ter respondido: está a ser reposto sozinho.
    Restarting,
    /// O programa não está instalado onde o editor o procura.
    Missing,
    /// Não arrancou, ou morreu outra vez antes de responder: só o comando «Restart» tenta de novo.
    Stopped,
}

/// Tudo o que um quadro do editor pinta.
pub struct CodeFrame<'a> {
    pub buffer: &'a Buffer,
    /// Os trechos de cor de uma faixa de bytes (a pintura só pede as linhas visíveis).
    pub spans: &'a mut dyn FnMut(Range<usize>) -> Vec<(Range<usize>, Ink)>,
    pub problems: &'a [Problem],
    /// As dobras que COMEÇAM numa faixa de linhas (a pintura só pede as das linhas visíveis, por
    /// bloco contíguo — o custo é o da vista, não o do ficheiro).
    pub folds: &'a mut dyn FnMut(Range<usize>) -> Vec<FoldMark>,
    /// As dobras FECHADAS do texto inteiro: decidem que linhas se escondem (só as `folded` contam).
    pub closed: &'a [FoldMark],
    /// O par do parêntese/chave junto ao cursor primário (os dois bytes).
    pub bracket: Option<(usize, usize)>,
    /// As ocorrências da busca (bytes), e qual é a corrente.
    pub matches: &'a [Range<usize>],
    pub current_match: Option<usize>,
    /// A composição em curso (IME / tecla morta), pintada sublinhada no cursor primário.
    pub preedit: Option<&'a str>,
    /// O cursor pisca: `false` = esta fase não o pinta.
    pub caret_on: bool,
    /// Largura de uma tabulação, em colunas.
    pub tab: usize,
    pub badge: Option<&'a Badge>,
    /// O último Ctrl+S aplicou o script mas o projecto ainda não tem ficheiro (o selo di-lo).
    pub project_unsaved: bool,
    /// `None` = o assistente está pronto (ou desligado de propósito): a barra não fala dele.
    pub assist: Option<Assist>,
}
