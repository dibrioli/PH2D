//! ⭐ **O ESPELHO DA FERRAMENTA ACTIVA** — o `&'static str` que o chrome lê.
//!
//! O `ToolId` vivo é uma `String` de runtime; `ImageEditState::active_tool_id` é
//! `Option<&'static str>`. A ponte entre os dois é esta função, e ela **não escolhe**: procura o
//! manifesto do registry cujo `id` bate, e devolve **o `&'static` dele**.
//!
//! ⛔⛔ **Aqui esteve um `match` de UM literal** (`"painter" => Some("painter"), _ => None`), mais um
//! filtro por `image_edit.mode_on`. Era verdade enquanto o único leitor era o trilho do Painter, e
//! deixou de ser em 2026-08-30: os toggles de `vector`/`motion`/`flip` leem este campo para escolher
//! entre **activar** e **cancelar**, e com o espelho cego eles liam sempre *«não está activa»* — o
//! segundo clique reactivava em vez de desligar.
//!
//! ⚠️ **E o filtro `mode_on` saiu**: ele pertence a quem pergunta pelo Painter
//! (`offers::rail_shows_painter_tools` já o exige), e aqui apagava a resposta para toda ferramenta
//! que não é de imagem.
//!
//! ⚠️ **Este ficheiro existe para o espelho ser GATEÁVEL.** Enquanto ele era três linhas dentro de
//! um closure no `render_loop`, **nada no repo o media** — um `grep` por `active_tool_id` em
//! `shells/desktop/` devolvia **um** ficheiro, o próprio.

/// O `&'static str` do manifesto cuja `id` é `live`, ou `None`.
///
/// `None` quando não há ferramenta activa, quando o registry ainda não foi instalado, ou quando a
/// ferramenta viva não tem manifesto — os três casos em que o chrome não tem o que espelhar.
#[must_use]
pub(crate) fn intern_active_tool(live: Option<&str>) -> Option<&'static str> {
    let live = live?;
    ph2d_editor::installed_registry()?
        .manifests()
        .iter()
        .find(|m| m.id == live)
        .map(|m| m.id)
}

#[cfg(test)]
#[path = "active_tool_mirror_tests.rs"]
mod tests;
