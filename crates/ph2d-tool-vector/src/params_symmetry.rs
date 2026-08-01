//! **A porta única `kind → id` da SIMETRIA de desenho** (plano 25 W6.3) — irmão de
//! [`super::params`] pelo teto de 700 LOC, e o corte é por assunto: aqui mora a correspondência
//! entre um tipo de espelho e o chip que o representa; lá, as conversões de unidade dos knobs.

/// **O chip de um tipo de simetria** — a porta ÚNICA `kind → id`.
///
/// O painel a usa para PINTAR e a tool para RESOLVER o clique. Duas tabelas divergiriam no dia em
/// que o vocabulário ganhasse o quinto tipo: uma delas pintaria um chip que a outra não reconhece,
/// e ele nasceria morto sob o mouse.
#[must_use]
pub fn symmetry_kind_id(k: ph2d_symmetry::SymmetryKind) -> ph2d_a11y::NodeId {
    use ph2d_symmetry::SymmetryKind as K;
    match k {
        K::MirrorX => ph2d_editor_core::ids::VECTOR_SYM_KIND_X,
        K::MirrorY => ph2d_editor_core::ids::VECTOR_SYM_KIND_Y,
        K::Custom => ph2d_editor_core::ids::VECTOR_SYM_KIND_CUSTOM,
        K::Radial => ph2d_editor_core::ids::VECTOR_SYM_KIND_RADIAL,
    }
}
