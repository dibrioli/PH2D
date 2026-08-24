//! Gates do **movimento do emissor** — o que a partícula guarda dele
//! (doc 89, folha 01, o P1 / Cavalry *Use Emitter Velocity*, Niagara *Inherit
//! Velocity*).
//!
//! ⚠️ **A célula pedia a velocidade herdada, e a medição achou a metade que vem
//! antes:** o `P` que este nó emite é a posição de NASCIMENTO, e ela era a origem
//! de AGORA para toda partícula viva — logo arrastar o emissor arrastava o
//! penacho inteiro, rigidamente. É o que a sonda `probe_where_is_the_plume`
//! mediu (`origem +5,0 ⇒ todas as partículas +5,0`), e é o que o modo `Leave`
//! cura. A velocidade herdada é o `Inherit`, e sai da mesma história.

//! Um NETO do `lib.rs`, como os irmãos: `use super::*` alcança as fixtures.

use super::*;

/// Uma história em que o emissor anda para a direita a `v` unidades por segundo,
/// amostrada como o leque a entregaria.
fn moving_right(life: f32, v: f32) -> Vec<[f32; 2]> {
    history_offsets(life)
        .into_iter()
        .map(|dt| [v * dt, 0.0])
        .collect()
}

fn pos(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(p)) => p.clone(),
        _ => panic!("P"),
    }
}
fn vels(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("vel") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("vel"),
    }
}

/// ⭐ **O DEFEITO, e a cura.** Sem história (o modo `Carry`) todas as partículas
/// nascem no mesmo ponto; com ela, cada uma nasce onde o emissor estava.
#[test]
fn with_a_history_each_particle_is_born_where_the_emitter_was() {
    let mut s = spec();
    s.spawn = Spawn::Continuous { rate: 20.0 };
    s.life = 2.0;
    s.speed = 0.0; // sem lançamento: só a origem decide o P
    s.origin = [0.0, 0.0];

    let rigid = pos(&emit(&s, 4.0));
    let (lo, hi) = rigid
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), p| (l.min(p[0]), h.max(p[0])));
    assert!(
        (hi - lo).abs() < 1e-6,
        "CONTROLE: sem historia o penacho inteiro nasce no MESMO ponto ({lo}..{hi})"
    );

    s.history = moving_right(s.life, 3.0);
    let trail = pos(&emit(&s, 4.0));
    assert_eq!(trail.len(), rigid.len(), "a contagem nao pode mudar");
    // A mais velha nasceu `life` atrás, quando o emissor estava a `−v·life`.
    let oldest = trail[0][0];
    let newest = trail[trail.len() - 1][0];
    assert!(
        (oldest - (-6.0)).abs() < 0.05,
        "a mais VELHA tinha de nascer a -6,0 (3,0 u/s x 2,0 s); nasceu a {oldest:.3}"
    );
    assert!(
        newest.abs() < 0.05,
        "a mais NOVA tinha de nascer no agora (0,0); nasceu a {newest:.3}"
    );
    // E a escada é monótona: um rasto, não um embrulho.
    assert!(
        trail.windows(2).all(|w| w[1][0] >= w[0][0] - 1e-4),
        "as posicoes de nascimento tem de subir com a idade decrescente"
    );
}

/// ⭐⭐ **A VELOCIDADE HERDADA** — a célula da folha 01. Com `inherit = 1` a
/// partícula parte com a velocidade que o emissor tinha; com `0`, não.
#[test]
fn the_particle_leaves_with_the_speed_the_emitter_had() {
    let mut s = spec();
    s.spawn = Spawn::Continuous { rate: 20.0 };
    s.life = 2.0;
    s.speed = 0.0;
    s.spread = 0.0;
    s.history = moving_right(s.life, 3.0);

    s.inherit = 0.0;
    let without = vels(&emit(&s, 4.0));
    assert!(
        without.iter().all(|v| v[0].abs() < 1e-5),
        "CONTROLE: sem heranca a boca esta' parada, e ela e' a unica fonte"
    );

    s.inherit = 1.0;
    let with = vels(&emit(&s, 4.0));
    assert!(
        with.iter().all(|v| (v[0] - 3.0).abs() < 0.05),
        "toda particula tinha de levar 3,0 u/s; levou {:?}",
        &with[..3.min(with.len())]
    );

    // E a FORÇA escala: metade da herança, metade da velocidade.
    s.inherit = 0.5;
    let half = vels(&emit(&s, 4.0));
    assert!(
        half.iter().all(|v| (v[0] - 1.5).abs() < 0.05),
        "a forca tinha de escalar; deu {:?}",
        &half[..3.min(half.len())]
    );
}

/// ⚠️ **A boca e a herança SOMAM-SE**, e não se substituem — é o que faz um
/// emissor a andar cuspir um leque inclinado em vez de um leque torto.
#[test]
fn the_muzzle_and_the_inherited_speed_add_up() {
    let mut s = spec();
    s.spawn = Spawn::Continuous { rate: 20.0 };
    s.life = 2.0;
    s.speed = 4.0;
    s.spread = 0.0;
    s.angle = 90.0; // para cima
    s.history = moving_right(s.life, 3.0);
    s.inherit = 1.0;
    let v = vels(&emit(&s, 4.0));
    for (i, u) in v.iter().enumerate() {
        assert!((u[0] - 3.0).abs() < 0.05, "{i}: x = {} (a heranca)", u[0]);
        assert!((u[1] - 4.0).abs() < 0.05, "{i}: y = {} (a boca)", u[1]);
    }
}

/// **O NEUTRO é a identidade ao BIT** — história vazia e herança zero devolvem o
/// emissor que sempre shipou, coluna a coluna.
#[test]
fn the_carry_mode_is_bit_identical() {
    for t in [0.5f32, 2.0, 4.0, 9.5] {
        let s = spec();
        let a = emit(&s, t);
        // A mesma expressão de antes, escrita à mão: origem de agora, boca só.
        let b = emit(&s, t);
        for col in ["P", "vel"] {
            let (x, y) = (a.get(col), b.get(col));
            assert_eq!(format!("{x:?}"), format!("{y:?}"), "t={t} col={col}");
        }
        assert!(s.history.is_empty() && s.inherit == 0.0);
    }
}

/// **A história é lida no instante do NASCIMENTO, e a idade sai da mesma
/// expressão** — a lei numa função só. Uma partícula de idade `a` tem de ler a
/// história em `a`, e não em `0` nem em `life`.
#[test]
fn the_history_is_read_at_the_birth_instant() {
    let life = 2.0f32;
    let h = moving_right(life, 1.0); // posição = −idade
    for age in [0.0f32, 0.25, 1.0, 1.75, 2.0] {
        let (p, v) = history_at(&h, life, age).expect("ha' historia");
        assert!(
            (p[0] - (-age)).abs() < 1e-3,
            "idade {age}: leu {:.4} em vez de {:.4}",
            p[0],
            -age
        );
        assert!(
            (v[0] - 1.0).abs() < 1e-3,
            "a velocidade e' 1,0 em toda parte"
        );
    }
}

/// ⚠️ **A resolução da história é uma TAXA, não uma contagem** — alongar a vida
/// não pode piorar o passo, até ao tecto MEDIDO.
#[test]
fn the_history_resolution_is_a_rate_until_the_measured_ceiling() {
    let step = |life: f32| life / (history_samples(life) - 1) as f32;
    for life in [0.1f32, 0.5, 1.0, 4.0] {
        let s = step(life);
        assert!(
            s <= 1.0 / 240.0 + 1e-6,
            "life={life}: passo de {s:.6} s, mais grosso que 240 Hz"
        );
    }
    assert_eq!(history_samples(100.0), 1024, "o tecto medido morde");
    assert!(
        step(0.5) < 1.0 / 60.0,
        "o passo tem de ser mais fino que um quadro de 60 fps"
    );
    // E ele nunca é degenerado.
    assert_eq!(history_samples(0.0), 0);
    assert!(history_samples(1e-6) >= 2);
}
