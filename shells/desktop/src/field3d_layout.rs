//! **As quatro viewports e as costuras — o código mudou-se para [`ph2d_viewport3d::layout`], o endereço ficou.**
//!
//! ⚠️ **Este alias existe para NÃO editar a árvore de outra linha.** `crate::field3d_layout` está escrito
//! em ficheiros `sculpt3d_*`, e a `line/app-sculpt3d` está a mover esses ficheiros **hoje**:
//! reescrever o endereço neles seria pôr duas linhas a editar o mesmo ficheiro no mesmo dia, que é
//! exactamente o caso em que a DIRETRIZ §1.5.5 manda parar. O alias custa uma linha.
//!
//! ⇒ Quem chegar novo escreve `ph2d_viewport3d::layout` directamente.
//!
//! # ⚠️ Os testes desta lei ficaram AQUI, e é de propósito
//!
//! `field3d_layout_tests.rs` exercita a lei **através** do smoke do 3D (`crate::field3d_smoke`,
//! `crate::field3d_input`), e é assim que ela deve ser exercitada — um gate sobre uma moldura
//! provado por um gesto real vale mais do que um provado por um `Rect` escrito à mão. Ele viaja
//! para `ph2d-app-field3d` com a família, não com a moldura.

pub(crate) use ph2d_viewport3d::layout::*;

#[cfg(test)]
#[path = "field3d_layout_tests.rs"]
mod tests;
