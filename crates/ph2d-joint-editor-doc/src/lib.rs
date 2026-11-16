//! **O estado da aba Joint Editor**, sem pintura (plano `docs/JointEditor/01_plano.md`): o documento da aba
//! ([`doc::JointEditorDoc`]), o retrato do mundo ([`model`]), o desenho do gabarito em metros ([`stage`]),
//! o rato e o teclado ([`input`]) e as juntas do gabarito por fazer ([`draft`]). Saiu da
//! `ph2d-app-joint-editor` (tecto de linhas por crate); a pintura e o quadro ficaram lá.

pub mod doc;
mod doc_values;
pub mod draft;
pub mod geo;
mod grab;
mod input;
pub mod model;
// O desenho em metros mora com a fábrica (a mesma lei de onde nasce cada peça); o caminho fica.
pub use ph2d_joint_factory::stage;
mod typing;

pub use doc::{Edit, Focus, Hit, JointEditorDoc};

/// A seta do motor mostra até onde ele leva a peça neste tempo (s): o carro de fábrica, −360°/s, vira um
/// quarto de volta; acima de meia volta a seta segue em espiral, sem tecto.
pub const MOTOR_PREVIEW_S: f32 = 0.25;

/// O número como o cartão o escreve: até três casas, sem zeros à direita.
#[must_use]
pub fn format(v: f32) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".to_owned()
    } else {
        s.to_owned()
    }
}
