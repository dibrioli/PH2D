//! Os gates da lei da barra — uma regra por gate, como o cabeçalho de [`super`] as numera.

use super::*;

const DT: f32 = 1.0 / 60.0;

/// Anda `n` quadros com a vida parada em `agora`.
fn anda(mut r: Rasto, agora: f32, n: usize, atraso: f32, vel: f32) -> Rasto {
    for _ in 0..n {
        r = avanca(r, agora, DT, atraso, vel);
    }
    r
}

/// ⭐ **(1)+(2) Um golpe: o rasto segura durante o atraso e depois escorre até à vida.**
///
/// **Mutações que devem sangrar:** o golpe não recomeçar a espera · a espera não segurar.
#[test]
fn o_rasto_segura_o_atraso_e_depois_escorre() {
    let r = Rasto::nasce(1.0);
    let r = avanca(r, 0.6, DT, 0.5, 1.0);
    assert_eq!(r.valor, 1.0, "o rasto largou no quadro do golpe");
    // Meio segundo de espera: 30 quadros, e o rasto ainda está no 1.
    let r = anda(r, 0.6, 29, 0.5, 1.0);
    assert!(
        (r.valor - 1.0).abs() < 1e-6,
        "o rasto largou antes da espera: {}",
        r.valor
    );
    // Depois escorre a 1 barra/s: 0,4 de diferença em ~0,4 s.
    let r = anda(r, 0.6, 12, 0.5, 1.0);
    assert!(
        r.valor < 1.0 && r.valor > 0.6,
        "a meio da descida: {}",
        r.valor
    );
    let r = anda(r, 0.6, 60, 0.5, 1.0);
    assert_eq!(r.valor, 0.6, "o rasto não chegou à vida");
}

/// ⭐⭐ **(1) Um combo: cada golpe NOVO recomeça a espera, e o rasto segura o valor de ANTES do
/// primeiro** — é o que mostra o combo inteiro de uma vez.
#[test]
fn um_combo_segura_o_rasto_no_valor_de_antes_do_primeiro_golpe() {
    let mut r = Rasto::nasce(1.0);
    let mut vida = 1.0;
    for _ in 0..5 {
        vida -= 0.1;
        r = avanca(r, vida, DT, 0.5, 1.0);
        r = anda(r, vida, 20, 0.5, 1.0); // um golpe a cada ~0,33 s, menos que a espera
    }
    assert!(
        (r.valor - 1.0).abs() < 1e-6,
        "o rasto escorreu a meio do combo: {}",
        r.valor
    );
    // O CONTROLO: sem golpe novo, depois da espera ele escorre.
    let r = anda(r, vida, 60, 0.5, 1.0);
    assert!(r.valor < 1.0, "sem golpe novo o rasto tinha de escorrer");
}

/// ⭐ **(3) Uma cura cola o rasto à vida na hora** — nunca o deixa abaixo dela.
///
/// **Mutação que deve sangrar:** a cura não puxar o rasto.
#[test]
fn uma_cura_cola_o_rasto_a_vida() {
    let r = Rasto::nasce(0.5);
    let r = avanca(r, 0.9, DT, 0.5, 1.0);
    assert_eq!((r.valor, r.espera_s), (0.9, 0.0));
    // E a meio de uma espera, uma cura que passa o rasto também o cola.
    let r = avanca(Rasto::nasce(0.8), 0.4, DT, 0.5, 1.0);
    let r = avanca(r, 0.95, DT, 0.5, 1.0);
    assert_eq!(r.valor, 0.95);
}

/// ⭐ **(4) Nunca abaixo da vida** — nem com uma velocidade enorme nem com um `dt` enorme.
#[test]
fn o_rasto_nunca_fica_abaixo_da_vida() {
    let r = avanca(Rasto::nasce(1.0), 0.3, DT, 0.0, 1.0);
    let r = avanca(r, 0.3, 100.0, 0.0, 1000.0);
    assert_eq!(r.valor, 0.3);
}

/// ⚠️ **O relógio parado espera connosco**, e um `dt` inválido não anda nada.
///
/// **Mutação que deve sangrar:** aceitar `dt` negativo (um rebobinar a fazer escorrer).
#[test]
fn um_relogio_parado_ou_a_recuar_nao_anda_o_rasto() {
    let r = avanca(Rasto::nasce(1.0), 0.5, DT, 0.0, 1.0);
    for dt in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let s = avanca(r, 0.5, dt, 0.0, 1.0);
        assert_eq!(s.valor, r.valor, "dt = {dt} andou o rasto");
    }
}

/// ⚠️ **Uma vida que não se sabe ler desenha-se VAZIA**, e tudo fica em `0..=1`.
#[test]
fn a_fraccao_e_saneada() {
    assert_eq!(fraccao(f32::NAN), 0.0);
    assert_eq!(fraccao(-2.0), 0.0);
    assert_eq!(fraccao(7.0), 1.0);
    assert_eq!(Rasto::nasce(f32::NAN).valor, 0.0);
}

/// ⭐ **As faixas crescem da ESQUERDA**, e a do rasto nunca é mais curta que a da vida.
#[test]
fn as_faixas_crescem_da_esquerda() {
    let [fundo, rasto, vida] = faixas(2.0, 0.75, 0.5);
    assert_eq!(fundo, (0.0, 2.0));
    assert_eq!(vida, (-0.5, 1.0), "a vida a metade ocupa a metade ESQUERDA");
    assert_eq!(rasto, (-0.25, 1.5));
    // A borda esquerda é a MESMA nas três.
    for (c, w) in [fundo, rasto, vida] {
        assert!(
            (c - w * 0.5 - (-1.0)).abs() < 1e-6,
            "a borda esquerda mexeu: {c} {w}"
        );
    }
    // Um rasto abaixo da vida desenha-se como a vida.
    assert_eq!(faixas(2.0, 0.1, 0.5)[1], faixas(2.0, 0.5, 0.5)[2]);
}
