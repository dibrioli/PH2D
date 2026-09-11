//! **As seis vistas nomeadas — o código mudou-se para [`ph2d_viewport3d::views`], o endereço ficou.**
//!
//! ⚠️ **Este alias existe para NÃO editar a árvore de outra linha.** `crate::field3d_views` está escrito
//! em ficheiros `sculpt3d_*`, e a `line/app-sculpt3d` está a mover esses ficheiros **hoje**:
//! reescrever o endereço neles seria pôr duas linhas a editar o mesmo ficheiro no mesmo dia, que é
//! exactamente o caso em que a DIRETRIZ §1.5.5 manda parar. O alias custa uma linha.
//!
//! ⇒ Quem chegar novo escreve `ph2d_viewport3d::views` directamente.
//!
//! # ⚠️ Os testes desta lei ficaram AQUI, e é de propósito
//!
//! `field3d_views_tests.rs` exercita a lei **através** do smoke do 3D (`crate::field3d_smoke`,
//! `crate::field3d_input`), e é assim que ela deve ser exercitada — um gate sobre uma moldura
//! provado por um gesto real vale mais do que um provado por um `Rect` escrito à mão. Ele viaja
//! para `ph2d-app-field3d` com a família, não com a moldura.

pub(crate) use ph2d_viewport3d::views::*;

// ⚠️ Ver a nota no irmão `field3d_navball.rs`: o `use super::*` do ficheiro de teste lê os aliases
// do pai, e um glob re-export não os carrega.
#[cfg(test)]
pub(crate) use ph2d_field_render::Orbit;

#[cfg(test)]
#[path = "field3d_views_tests.rs"]
mod tests;
