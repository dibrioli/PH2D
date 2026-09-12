//! ⭐⭐ **O GÉMEO NEUTRO DO [`crate::sculpt3d_host`]** — o que a shell responde quando a família
//! da escultura **não foi compilada**.
//!
//! # Por que ele existe, e por que não é gatear os chamadores
//!
//! Dois predicados desta família são perguntas que o resto do app faz sem saber que ela existe:
//! *as teclas da escultura estão vivas?* e *há uma costura sob o cursor?*. O doc de cada um já **prometia por escrito** que «sem cena (ou sem a feature) ela
//! é `false`» — e o `#[cfg(feature = "sculpt3d")]` em cima deles quebrava essa promessa: sem a
//! feature o método não existia, e o chamador deixava de compilar.
//!
//! ⛔ **Gatear os dois chamadores era a cura errada, e a medição di-lo:** um deles é um campo no
//! meio de um `eprintln!` de nove guardas (`undo_route.rs`), e gateá-lo obrigava a duplicar a
//! string de formato inteira — duas cópias que divergem no primeiro guarda novo. E a outra
//! metade é pior: *toda chamada futura herdaria a obrigação de se lembrar do `#[cfg]`*, cujo
//! modo de falha só aparece numa build que a CI não corre.
//!
//! ⇒ a resposta neutra mora num sítio só, ao lado do nome do módulo que ela substitui.
//!
//! ⚠️ **O `sculpt3d_clay_on_screen` NÃO está aqui, e a ausência é medida:** o único chamador
//! dele é o `sculpt3d_host`, que cai com a feature. *Um gémeo sem chamador é código morto com
//! cara de simetria* — e o compilador diz qual é qual.
//!
//! ⚠️ **Foi uma build `--no-default-features` que os revelou** — os dois estavam ungated desde
//! antes desta linha. A build de omissão tem a feature LIGADA, então ela é cega a esta classe
//! inteira.

use crate::app_state::App;

impl App {
    /// Sem o módulo, as teclas da escultura nunca reclamam nada.
    pub(crate) fn sculpt3d_keys_live(&self) -> bool {
        false
    }

    /// Sem o módulo não há divisão de janela 3D, logo não há costura sob o cursor.
    pub(crate) fn sculpt3d_seam_cursor(&self) -> Option<winit::window::CursorIcon> {
        None
    }
}
