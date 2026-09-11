//! **A área de desenho visível — o código mudou-se, o endereço ficou.**
//!
//! A lei vive agora em [`ph2d_app_host::canvas_area`] (W2). Ela é uma função de sete linhas sobre
//! um `HeroScreen` e um `Rect`, sem uma linha de `App` — e tem **dois** clientes de linhas
//! diferentes (o enquadramento do módulo 3D e o do *Edit Prefab*), o que é precisamente a razão
//! por que ela não pode viver dentro de nenhum dos dois.
//!
//! ⚠️ O doc dela guarda a história de por que ela mudou de nome duas vezes: *«uma porta com o nome
//! de um dos seus clientes convida à segunda cópia»*.

pub(crate) use ph2d_app_host::canvas_area::visible;
