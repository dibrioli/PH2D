//! **A lei do gizmo de transformação** — a lei vive na [`ph2d_viewport3d::gizmo`].
//!
//! ⚠️ **Este alias existe por UM motivo só: não editar a árvore de outra linha.** `crate::field3d_gizmo`
//! está escrito em ficheiros `sculpt3d_*`, e a `line/app-sculpt3d` está a mover esses ficheiros
//! **hoje** — reescrever o endereço neles seria pôr duas linhas a editar o mesmo ficheiro no mesmo
//! dia, que é o caso em que a DIRETRIZ §1.5.5 manda parar.
//!
//! ⇒ **Quando a `line/app-sculpt3d` fechar, este ficheiro some** e os chamadores dela passam a
//! escrever `ph2d_viewport3d::gizmo` directamente. *Um alias com data de validade escrita é dívida
//! nomeada; um sem data é uma camada.*
//!
//! ⚠️ Os testes desta lei **viajaram com a família** (`ph2d-app-field3d`): eles exercitam-na
//! através do smoke do 3D, e é assim que ela deve ser exercitada.

pub(crate) use ph2d_viewport3d::gizmo::*;
