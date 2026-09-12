//! **TODA PORTA DO CATÁLOGO TEM UM NOME LEGÍVEL** — o censo que fecha o report do Enio de
//! 2026-08-27 (*"vários nós como o próprio boids têm uma série de inputs sem nenhum nome"*).
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 600 para `shells/`), e o corte é por
//! PERGUNTA: o irmão [`super::dock_height_tests`] mede *cabe no dock?* — píxeis contra a altura
//! do inspector; aqui pergunta-se *este rótulo é LEGÍVEL?* — uma varredura do registry inteiro
//! que não olha para altura nenhuma. Elas partilham o `MotionState` e mais nada.

use crate::motion_state::MotionState;

/// ⛔⛔ **TODA PORTA DO CATÁLOGO TEM UM NOME LEGÍVEL** — o censo que fecha o report do Enio.
///
/// *"Vários nós como o próprio boids têm uma série de inputs sem nenhum nome, sem identificação.
/// Como o usuário entender se em nenhum lugar diz o que é?"* (2026-08-27, com foto).
///
/// O rótulo é **derivado** do nome do manifesto (`ph2d_panel_motion_graph::PortLabel`), e essa
/// escolha só é honesta se a derivação responder por **todo** o catálogo — senão um nó qualquer
/// desenha uma faixa vazia e o defeito volta, num nó só, sem ninguém dar por ela.
///
/// A régua tem três metades:
/// - **não-vazio**: uma porta sem rótulo é a fileira anónima da foto;
/// - **começa em maiúscula**: é o que separa um rótulo de um identificador cru (`target_x`);
/// - **sem `_`**: idem — se sobrar um sublinhado, a derivação não tocou naquele nome.
///
/// ⚠️ Ela vale como censo porque varre o **registry inteiro**, que é a mesma população que a
/// paleta oferece ao artista. Um nó novo entra aqui no dia em que é registado.
#[test]
fn every_port_in_the_catalogue_has_a_readable_label() {
    use ph2d_panel_motion_graph::PortLabel;
    let motion = MotionState::new();
    let mut bad: Vec<String> = Vec::new();
    let mut portas = 0usize;
    for man in motion.registry.manifests() {
        for p in man.inputs.iter().chain(man.outputs.iter()) {
            portas += 1;
            let l = PortLabel::of(p.name);
            let l = l.as_str();
            if l.is_empty()
                || l.contains('_')
                || !l
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            {
                bad.push(format!("{}::{} -> {l:?}", man.name, p.name));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "portas cujo rotulo derivado nao e' legivel: {bad:?}"
    );
    // ⚠️ **CONTROLE — sem ele um registry vazio (ou um `manifests()` que devolvesse nada) daria
    // uma lista vazia de acusacoes e o gate passaria sobre o nada.**
    assert!(
        portas > 300,
        "o censo varreu so' {portas} portas — ele mediu o registry ou mediu o vazio?"
    );
}
