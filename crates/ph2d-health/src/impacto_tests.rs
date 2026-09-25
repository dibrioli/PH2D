//! Os gates da lei do impacto — ver o cabeçalho de [`super`].

use super::*;

/// ⭐⭐⭐ **Pausas não se somam — fica a maior** (dez balas no mesmo instante são um golpe só).
///
/// **Mutação que deve sangrar:** somar em vez de guardar o máximo.
#[test]
fn pausas_nao_se_somam_fica_a_maior() {
    let mut p = Pausa::new();
    p.pede(0.05);
    p.pede(0.05);
    assert_eq!(p.resta_s(), 0.05);
    p.pede(0.15);
    assert_eq!(p.resta_s(), 0.15);
    p.pede(0.05);
    assert_eq!(
        p.resta_s(),
        0.15,
        "uma pausa menor não encurta a que está a correr"
    );
    for inerte in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        p.pede(inerte);
    }
    assert_eq!(p.resta_s(), 0.15, "um pedido não finito ou ≤ 0 é inerte");
}

/// ⭐⭐ **O tempo retido é exactamente a pausa**, partido por quantos quadros for preciso, e o que
/// sobra de um quadro passa inteiro.
#[test]
fn o_tempo_retido_e_exactamente_a_pausa() {
    let mut p = Pausa::new();
    p.pede(0.05);
    let a = p.consome(0.02);
    let b = p.consome(0.02);
    let c = p.consome(0.02);
    assert_eq!(a, 0.0);
    assert_eq!(b, 0.0);
    assert!(
        (c - 0.01).abs() < 1e-12,
        "o 3.º quadro passa o que sobrou: {c}"
    );
    assert_eq!(
        p.consome(0.02),
        0.02,
        "depois da pausa o tempo passa inteiro"
    );
    assert_eq!(p.consome(f64::NAN), 0.0);
    p.pede(0.1);
    p.limpa();
    assert_eq!(p.consome(0.02), 0.02, "uma pausa esquecida não retém nada");
}

/// ⭐ **Piscar começa VISÍVEL, alterna por metades, e fora da janela está sempre visível.**
///
/// **Mutações que devem sangrar:** começar escondido · ignorar o `invencivel`.
#[test]
fn o_piscar_comeca_visivel_e_alterna() {
    let m = 0.1;
    assert!(
        pisca_visivel(0.0, true, m),
        "a 1.ª metade mostra o objecto (e o clarão)"
    );
    assert!(pisca_visivel(0.099, true, m));
    assert!(!pisca_visivel(0.1, true, m), "a 2.ª metade esconde");
    assert!(!pisca_visivel(0.199, true, m));
    assert!(pisca_visivel(0.2, true, m));
    assert!(pisca_visivel(0.15, false, m), "fora da janela não pisca");
    assert!(pisca_visivel(0.15, true, 0.0), "meio período 0 = não pisca");
    assert!(pisca_visivel(0.15, true, f64::NAN));
}
