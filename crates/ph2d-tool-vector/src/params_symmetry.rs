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

/// **A contagem de cópias que um `SetValue` da fileira *Segments* autora.**
///
/// ⚠️ O painel já converteu o track na fronteira dele, então `v` chega no domínio do DOCUMENTO —
/// aqui não há mapa nenhum, só a **cerca do vocabulário**: um número fora de `MIN..=MAX` (ou um
/// `NaN` vindo de aritmética degenerada a montante) não pode entrar no estilo, porque o estilo é
/// o que viaja para a `SymmetrySpec` e para o ficheiro.
///
/// ⛔ **Presa e não recusada:** `SymmetrySpec::segments()` já prende na leitura, e um estilo que
/// guardasse `0` enquanto o kernel desenha `3` seria a segunda resposta a *"quantas cópias?"* —
/// exactamente o par que este módulo existe para não deixar nascer.
#[must_use]
pub fn segments_from_value(v: f64) -> u32 {
    if !v.is_finite() {
        return ph2d_symmetry::MIN_SEGMENTS;
    }
    let lo = f64::from(ph2d_symmetry::MIN_SEGMENTS);
    let hi = f64::from(ph2d_symmetry::MAX_SEGMENTS);
    v.round().clamp(lo, hi) as u32
}
