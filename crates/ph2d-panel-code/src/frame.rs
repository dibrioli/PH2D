//! ⭐ **O CONTRATO entre a família e a pintura** — o que `ph2d-app-code` entrega em cada quadro. A
//! pintura não conhece a gramática, o Luau nem a cena: só isto.

use crate::style::Ink;
use ph2d_code_buffer::Buffer;
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
}

/// A explicação de um problema (I8): o título numa linha e o porquê com o que fazer. `kind` é o nome
/// estável da classe (o registo local das explicações vistas grava-o).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Explanation {
    pub kind: &'static str,
    pub title: String,
    pub why: String,
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

/// As linhas que a lista de completação mostra de uma vez: o `completion_lines` do `CodeEdit` do
/// Godot 4.7.2 (oráculo corrido em 16/10, `docs/CodeEditor/03_onda2.md` I4).
pub const COMPLETION_ROWS: usize = 7;
/// A largura máxima da lista, em colunas da mono: o `completion_max_width` do mesmo oráculo.
pub const COMPLETION_MAX_CHARS: usize = 50;

/// Uma linha da lista de completação.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionRow {
    pub label: String,
    /// As letras que casaram com o que se escreveu (bytes do rótulo).
    pub matched: Vec<Range<usize>>,
    pub ink: Ink,
    /// O tipo, à direita (cortado se não couber).
    pub detail: Option<String>,
}

/// A lista de completação aberta: a janela das linhas à vista.
pub struct CompletionView<'a> {
    /// O byte onde começa a palavra: a lista alinha-se por ele, por baixo da linha (por cima se não
    /// couber).
    pub anchor: usize,
    /// As linhas `top .. top + rows.len()` da lista inteira.
    pub rows: &'a [CompletionRow],
    pub total: usize,
    pub top: usize,
    /// O índice (na lista inteira) da seleccionada.
    pub selected: usize,
    /// A largura que a lista inteira pede, em colunas (fixa enquanto se rola).
    pub width_chars: usize,
    /// A documentação da seleccionada (a caixa ao lado).
    pub doc: Option<&'a str>,
    /// A prévia da acção seleccionada (I7; no lugar da documentação): as linhas como ficam.
    pub preview: &'a [PreviewLine],
    pub kind: ListKind,
}

/// O que a janelinha lista: a completação, as acções de contexto do Alt+Enter (I7) ou os sítios de um
/// nome do Shift+F12 (I10). As duas últimas nascem com a 1.ª linha em foco neutro, não no acento.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListKind {
    Completion,
    Actions,
    References,
}

/// Uma linha da prévia de uma acção: o texto como fica e o que nele muda (bytes do texto).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewLine {
    pub text: String,
    pub changed: Vec<Range<usize>>,
}

/// O balão do rato, por baixo de `word` (bytes do buffer): o que o assistente diz da palavra (I5) e
/// os problemas (I8), uma secção cada.
pub struct HoverView<'a> {
    pub word: Range<usize>,
    pub sections: Vec<HoverSection<'a>>,
    /// As correcções do problema (I8b), um botão cada por baixo do texto.
    pub fixes: Vec<&'a str>,
    /// O botão sob o rato.
    pub hot: Option<usize>,
    /// Quanto do realce de cada botão está presente (o eixo do hover; vazio = assente).
    pub live: &'a [f32],
}

/// Uma secção do balão: o título (marcado) e o texto, em Markdown.
pub struct HoverSection<'a> {
    pub title: Option<&'a str>,
    pub text: &'a str,
}

/// A ajuda de assinatura da chamada no cursor primário (I5).
pub struct SignatureView<'a> {
    pub label: &'a str,
    /// O parâmetro activo, em bytes do `label` (marcado).
    pub active: Option<Range<usize>>,
    /// A documentação do parâmetro activo e a da função, em Markdown.
    pub doc: Option<&'a str>,
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
    pub completion: Option<CompletionView<'a>>,
    pub hover: Option<HoverView<'a>>,
    pub signature: Option<SignatureView<'a>>,
    /// As ocorrências do nome sob o cursor (I10), realçadas.
    pub occurrences: &'a [Range<usize>],
    pub rename: Option<RenameView<'a>>,
    /// As dicas de tipo à mostra (I9), por ordem de `at`.
    pub inlays: &'a [Inlay],
    /// Os buracos NÃO activos do modelo inserido (I12), marcados.
    pub stops: &'a [Range<usize>],
    /// O script tem `--!strict` (I14): a barra di-lo.
    pub strict: bool,
    /// O nome que o Ctrl+clique alcança, com o Ctrl carregado e o rato sobre ele: sublinhado.
    pub link: Option<Range<usize>>,
    /// Um arrasto entre painéis sobre o texto (item 8): onde cairia, e o balão das teclas.
    pub dropping: Option<crate::DropView<'a>>,
}

/// Uma dica de tipo (I9): o rótulo (`: number`) mostrado em `at` sem ocupar bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inlay {
    pub at: usize,
    pub label: String,
}

/// A caixinha do rename (I10), por baixo de `word` (bytes do buffer).
pub struct RenameView<'a> {
    pub word: Range<usize>,
    /// O nome novo; `None` = não há o que renomear aqui e a caixinha só diz porquê.
    pub field: Option<&'a Buffer>,
    /// A ajuda por baixo do campo.
    pub hint: &'a str,
    /// Porque o nome não serve (no lugar da ajuda, a vermelho).
    pub error: Option<&'a str>,
}
