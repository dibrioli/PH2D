//! **O gizmo de navegação — o código mudou-se para [`ph2d_viewport3d::navball`], o endereço ficou.**
//!
//! ⚠️ **Este alias existe para NÃO editar a árvore de outra linha.** `crate::field3d_navball` está escrito
//! em ficheiros `sculpt3d_*`, e a `line/app-sculpt3d` está a mover esses ficheiros **hoje**:
//! reescrever o endereço neles seria pôr duas linhas a editar o mesmo ficheiro no mesmo dia, que é
//! exactamente o caso em que a DIRETRIZ §1.5.5 manda parar. O alias custa uma linha.
//!
//! ⇒ Quem chegar novo escreve `ph2d_viewport3d::navball` directamente.
//!
//! # ⚠️ Os testes desta lei ficaram AQUI, e é de propósito
//!
//! `field3d_navball_tests.rs` exercita a lei **através** do smoke do 3D (`crate::field3d_smoke`,
//! `crate::field3d_input`), e é assim que ela deve ser exercitada — um gate sobre uma moldura
//! provado por um gesto real vale mais do que um provado por um `Rect` escrito à mão. Ele viaja
//! para `ph2d-app-field3d` com a família, não com a moldura.

pub(crate) use ph2d_viewport3d::navball::*;

// ⚠️ **Um `use super::*` num teste vê os ALIASES do módulo pai, não só os itens públicos dele** —
// e um glob re-export não os traz. Os três abaixo estavam no topo do `field3d_navball.rs` e são o
// vocabulário que `field3d_navball_tests.rs` escreve sem qualificar. *Esta é a única coisa que um
// alias de módulo NÃO substitui, e ela só aparece com `--all-targets`: um `cargo check` cru não
// compila `cfg(test)` e passa verde sobre 44 erros.*
// ⚠️ **`#[cfg(test)]` e não `pub(crate)` nu:** fora do teste ninguém os lê, e três `unused import`
// num build normal são três vermelhos no clippy `-D warnings` do gate de fecho.
#[cfg(test)]
pub(crate) use {
    ph2d_editor::zones::Rect as EditorRect, ph2d_field_render::Orbit, ph2d_viewport3d::views::Standard,
};

#[cfg(test)]
#[path = "field3d_navball_tests.rs"]
mod tests;
