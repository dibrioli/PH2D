//! O gate que o relógio não pode dar: a leitura sem espera **nunca espera**.
//!
//! ⚠️ Um gate de relógio aqui seria mais um membro da família de flakes de carga, e um gate de
//! comportamento não o vê (esperar dá a MESMA resposta, só mais tarde). ⇒ a régua é o TEXTO do
//! corpo da função, sem os comentários — o molde que a casa usa para uma costura que só um relógio
//! distinguiria.

/// O corpo de `tap_sem_espera`, só as linhas de código.
fn corpo() -> Vec<&'static str> {
    let src = include_str!("tap_voo.rs");
    let ini = src.find("pub fn tap_sem_espera").expect("a funcao existe");
    let fim = src[ini..]
        .find("pub fn descarta_tap_em_voo")
        .expect("a vizinha delimita o corpo");
    src[ini..ini + fim]
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with("//"))
        .collect()
}

#[test]
fn a_leitura_sem_espera_nunca_espera_pela_placa() {
    let c = corpo();
    assert!(
        c.len() >= 20,
        "piso de populacao: o corpo tem de ter sido extraido ({} linhas)",
        c.len()
    );
    // CONTROLO: a régua vê o poll que lá está — sem isto uma extracção vazia passava.
    assert!(
        c.iter().any(|l| l.contains("PollType::Poll")),
        "o corpo acorda os callbacks com um poll NAO bloqueante"
    );
    for proibido in ["wait_indefinitely", "PollType::Wait", ".recv()"] {
        assert!(
            !c.iter().any(|l| l.contains(proibido)),
            "`{proibido}` no corpo de `tap_sem_espera`: esperar pela placa e' o defeito \
             (0,68 ms por quadro no app, doc 120 §8.6)"
        );
    }
}
