//! **O pintor do gizmo de navegação — o código mudou-se para [`ph2d_viewport3d::navball_paint`], o endereço ficou.**
//!
//! ⚠️ **Este alias existe para NÃO editar a árvore de outra linha.** `crate::field3d_navball_paint` está escrito
//! em ficheiros `sculpt3d_*`, e a `line/app-sculpt3d` está a mover esses ficheiros **hoje**:
//! reescrever o endereço neles seria pôr duas linhas a editar o mesmo ficheiro no mesmo dia, que é
//! exactamente o caso em que a DIRETRIZ §1.5.5 manda parar. O alias custa uma linha.
//!
//! ⇒ Quem chegar novo escreve `ph2d_viewport3d::navball_paint` directamente.

pub(crate) use ph2d_viewport3d::navball_paint::*;
