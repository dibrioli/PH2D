//! `ph2d-kanban-md` — a descrição do cartão é markdown (ADR-0195 §9): `parse` lê CommonMark
//! (+ tarefas `- [ ]` e riscado `~~`) para um `RichDoc`, `render` escreve-o de volta.
//! Lei: `render(parse(md))` é ponto fixo depois da 1.ª normalização; o que o modelo não tem
//! (tabelas, HTML, imagens) não se perde — vira texto literal.

mod inline_md;
mod parse;
mod render;

pub use parse::parse;
pub use render::render;

/// `md → doc → md`: a forma normal que se grava.
pub fn normalize(md: &str) -> String {
    render(&parse(md))
}
