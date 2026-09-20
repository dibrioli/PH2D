//! **A FORMA DOADA** — o que uma malha 3D entrega a uma superfície 2D, nas suas duas formas.
//!
//! | módulo | o fio | quem produz | quem consome |
//! |---|---|---|---|
//! | [`baked_form`] | **ASSADO**: os canais gravados num sprite, e a luz que os relê | o gesto de assar (`ph2d-app-sculpt3d`) | a shell (persistência + reacendida por frame) |
//! | [`donated_form`] | **VIVO**: um plano de forma no tamanho do canvas do Painter | a doação (`ph2d-app-sculpt3d`) | o `painter_bridge` da shell |
//!
//! ## Por que os dois moram juntos, e por que fora da shell
//!
//! ⚠️ **A razão é a MESMA para os dois, e ela é uma AUSÊNCIA de `cfg`.** Os dois ficheiros
//! declaram-se, cada um no próprio topo, deliberadamente **fora** da feature `sculpt3d`: o
//! assado porque *«o runtime lê os canais sem o módulo 3D»* (um projeto reaberto num binário
//! sem escultura tem de continuar iluminável), o vivo porque *«o que atravessa é `Vec<f32>` e
//! um par de `u32` — nenhum tipo do módulo 3D»*. Enquanto a família morava na shell, «não ter
//! `cfg`» bastava para os manter alcançáveis dos dois lados. Quando ela saiu para uma crate
//! **opcional**, deixou de bastar: um tipo atravessa a fronteira ou não atravessa, e
//! [`baked_form::RigStamp`] é **campo de uma struct** do lado de lá.
//!
//! ⛔ **A alternativa era pô-los na crate da família, e ela reabria exactamente o defeito que
//! a ausência de `cfg` existe para impedir**: sem a feature, a `ph2d-app-sculpt3d` não é
//! compilada, e com ela iriam embora a persistência dos canais e a reacendida — a arte
//! mudaria em silêncio ao reabrir.
//!
//! ⚠️ **Ninguém aqui sabe o que é uma MALHA.** As dependências são `ph2d-gpu`, `ph2d-light`,
//! `ph2d-painter-brush` e `ph2d-render` — as quatro do desenho, nenhuma do 3D. É essa
//! ausência que faz desta crate uma folha e não uma ponte.

pub mod baked_form;
pub mod donated_form;
pub mod lei_da_luz;
