//! ⭐ **O DIAGNÓSTICO de um script** — o que correu mal, em que LINHA dele, com a mensagem inteira e
//! a pilha quando o Luau a dá (CodeEditor Onda 0, T1).
//!
//! O formato que isto lê foi MEDIDO no mlua 0.12.1 / luau0-src 0.21.0+luau736 e está congelado em
//! `tests/it/diagnostico_do_luau_medido.rs`: `<nome>:<linha>: <mensagem>` à frente, e a pilha
//! acrescentada pelo mlua depois de `\nstack traceback:\n` (ou no campo de um `CallbackError`).
//!
//! ⚠️ **Só a linha DESTE script conta**: o nome do chunk é o do script (`set_name("=<nome>")`), e
//! uma posição de outro chunk nunca é atribuída a ele — cai-se para a primeira moldura da pilha que
//! seja deste script, ou `None`.

use crate::props::DeclError;

/// **De que espécie é a falha.**
#[derive(Clone, Debug, PartialEq)]
pub enum DiagnosticKind {
    /// O Luau não conseguiu ler o texto.
    Syntax,
    /// Uma declaração (`ph2d.property`) foi recusada — o porquê tipado.
    Decl(DeclError),
    /// O script falhou a correr.
    Runtime,
    /// Uma chamada passou do prazo de um quadro ([`crate::HOOK_BUDGET`]).
    Budget,
}

/// ⭐ **Porque um script não carregou ou parou.**
#[derive(Clone, Debug, PartialEq)]
pub struct ScriptDiagnostic {
    pub kind: DiagnosticKind,
    /// A linha (1-based) NESTE script, quando a falha tem uma.
    pub line: Option<u32>,
    /// A mensagem inteira (todas as linhas), sem o prefixo `<nome>:<linha>: ` deste script e sem a
    /// pilha. Quem a mostra decide quanto mostra.
    pub message: String,
    /// A pilha tal como o Luau a escreveu (`stack traceback:\n\t…`), quando há.
    pub traceback: Option<String>,
}

/// O separador que o mlua põe entre a mensagem e a pilha (medido).
const TRACEBACK: &str = "\nstack traceback:\n";

impl ScriptDiagnostic {
    /// Uma falha que o HOST detectou depois da chamada (sem posição no texto).
    #[must_use]
    pub fn runtime(message: impl Into<String>) -> Self {
        Self {
            kind: DiagnosticKind::Runtime,
            line: None,
            message: message.into(),
            traceback: None,
        }
    }

    /// ⭐ **Lê um erro do mlua** de um script cujo chunk se chama `chunk`.
    #[must_use]
    pub fn from_error(e: &mlua::Error, chunk: &str) -> Self {
        if let Some(b) = e.downcast_ref::<BudgetExceeded>() {
            let traceback = (!b.traceback.is_empty()).then(|| b.traceback.clone());
            return Self {
                kind: DiagnosticKind::Budget,
                line: traceback.as_deref().and_then(|t| frame_line(t, chunk)),
                message: b.to_string(),
                traceback,
            };
        }
        if let mlua::Error::SyntaxError { message, .. } = e {
            let (line, message) = split_position(message, chunk);
            return Self {
                kind: DiagnosticKind::Syntax,
                line,
                message: message.to_owned(),
                traceback: None,
            };
        }
        let (text, callback_tb) = parts(e);
        let (body, text_tb) = match text.rfind(TRACEBACK) {
            Some(i) => (&text[..i], Some(text[i + 1..].to_owned())),
            None => (text.as_str(), None),
        };
        let traceback = callback_tb.or(text_tb);
        let (line, message) = split_position(body, chunk);
        Self {
            kind: DiagnosticKind::Runtime,
            line: line.or_else(|| traceback.as_deref().and_then(|t| frame_line(t, chunk))),
            message: message.to_owned(),
            traceback,
        }
    }
}

/// ⚠️ Sem localização: é o texto do terminal e dos registos. O cartão compõe o seu pelo i18n.
/// `{:#}` acrescenta a pilha.
impl std::fmt::Display for ScriptDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.kind == DiagnosticKind::Syntax {
            f.write_str("syntax error: ")?;
        }
        if let Some(n) = self.line {
            write!(f, "line {n}: ")?;
        }
        f.write_str(&self.message)?;
        if f.alternate()
            && let Some(tb) = &self.traceback
        {
            write!(f, "\n{tb}")?;
        }
        Ok(())
    }
}

/// O texto mais interno e a pilha do `CallbackError` mais externo (a que tem as molduras do script).
fn parts(e: &mlua::Error) -> (String, Option<String>) {
    match e {
        mlua::Error::CallbackError { traceback, cause } => {
            (parts(cause).0, Some(traceback.clone()))
        }
        mlua::Error::RuntimeError(s) => (s.clone(), None),
        other => (other.to_string(), None),
    }
}

/// `"<chunk>:<n>: resto"` ⇒ `(Some(n), "resto")`; qualquer outra forma fica inteira e sem linha.
fn split_position<'a>(msg: &'a str, chunk: &str) -> (Option<u32>, &'a str) {
    let Some((n, rest)) = position(msg, chunk) else {
        return (None, msg);
    };
    match rest.strip_prefix(": ") {
        Some(rest) => (Some(n), rest),
        None => (None, msg),
    }
}

/// A primeira moldura da pilha que é DESTE script (`\t<chunk>:<n>: in …`).
fn frame_line(traceback: &str, chunk: &str) -> Option<u32> {
    traceback.lines().find_map(|l| {
        let (n, rest) = position(l.trim_start(), chunk)?;
        (rest.is_empty() || rest.starts_with(':')).then_some(n)
    })
}

/// `"<chunk>:<dígitos>…"` ⇒ a linha e o que vem depois dos dígitos.
fn position<'a>(s: &'a str, chunk: &str) -> Option<(u32, &'a str)> {
    if chunk.is_empty() {
        return None;
    }
    let rest = s.strip_prefix(chunk)?.strip_prefix(':')?;
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    let n = rest[..end].parse().ok()?;
    Some((n, &rest[end..]))
}

/// **O prazo de um quadro estourou** — o erro TIPADO que a interrupção levanta, com a pilha
/// colhida dentro dela (fora dela já não há pilha: medido, o erro de uma interrupção volta sem).
#[derive(Debug)]
pub(crate) struct BudgetExceeded {
    pub(crate) traceback: String,
}

impl std::fmt::Display for BudgetExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the script ran for longer than a frame ({} ms) — is there a loop that never ends?",
            crate::scene::HOOK_BUDGET.as_secs_f64() * 1000.0
        )
    }
}

impl std::error::Error for BudgetExceeded {}
