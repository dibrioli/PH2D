//! Os gates da poeira de impacto (estudo de UI viva, D2).

use super::*;
use crate::motion::{UiCharacter, UiMotion};

fn expressivo() -> UiMotion {
    let mut m = UiMotion::default();
    m.set_character(UiCharacter::Expressive);
    m.set_reduced_motion(false);
    m
}

/// ⛔⛔ **A CERCA MORA NUMA PORTA, e não em cada sítio que arma.**
///
/// O [`Role::Decoration`] diz, desde que nasceu: *"ausente em Discreto — ausente, não atenuado"*, e
/// o `reduced_motion` mata-o. ⚠️ **Perguntar em cada chamador** é o desenho que faz o quinto nascer
/// sem a cerca — a lacuna que esta casa já pagou com 36 células de física e dez chips do Painter.
///
/// ⚠️ *Ausente, não atenuado* é load-bearing e está medido aqui: em Discreto **não há uma faísca
/// curta**, não há faísca.
#[test]
fn the_fence_lives_in_the_door_and_discrete_means_absent_not_shorter() {
    let mut campo = BurstField::default();
    campo.emit(&expressivo(), [10.0, 10.0], 1);
    assert_eq!(campo.live().len(), 1, "o caso comum nao emitiu");

    for (nome, m) in [
        ("discreto", {
            let mut m = expressivo();
            m.set_character(UiCharacter::Discrete);
            m
        }),
        ("reduced motion", {
            let mut m = expressivo();
            m.set_reduced_motion(true);
            m
        }),
    ] {
        let mut c = BurstField::default();
        c.emit(&m, [10.0, 10.0], 1);
        assert!(
            c.is_empty(),
            "{nome}: emitiu uma faisca - a cerca do Role::Decoration diz AUSENTE, e uma faisca \
             curta nao e' ausencia"
        );
    }
}

/// ⭐⭐ **A POSIÇÃO É FUNÇÃO DA IDADE, não integrada passo a passo** — e é isso que a torna imune ao
/// ritmo do quadro.
///
/// ⚠️ É a mesma lei que o Painter pagou seis vezes: *o traço é fato do CAMINHO, nunca de quão fino o
/// motor amostrou o caminho*. Um quadro perdido não pode deslocar uma faísca.
///
/// ⚠️ A régua é a igualdade entre dois caminhos de tempo — um passo grande contra muitos pequenos.
///
/// ⚠️⚠️ **O QUE ELE PROVA, e o que NÃO prova.** Ele apanha uma implementação que **integre** por
/// tique (guardando velocidade e somando à posição): essa deriva entre os dois caminhos. ⛔ Ele
/// **não** é matável por uma mutação de uma linha na fórmula — trocar `vel·(1−(1−t)²)` por `vel·t`
/// continua a ser forma FECHADA da idade, e passa. Isso foi medido, não suposto.
///
/// ⇒ a propriedade é garantida sobretudo pelo **TIPO**: um [`Burst`] carrega `at`, `age` e `seed`,
/// e **nenhum estado integrado** — não há onde uma deriva se acumular. *Quando a estrutura já
/// impede o defeito, o gate defende a fronteira, não a fórmula.*
#[test]
fn a_spark_is_where_its_age_says_never_where_the_frame_rate_says() {
    let mut grosso = BurstField::default();
    let mut fino = BurstField::default();
    grosso.emit(&expressivo(), [40.0, 40.0], 7);
    fino.emit(&expressivo(), [40.0, 40.0], 7);
    grosso.tick(0.2);
    for _ in 0..20 {
        fino.tick(0.01);
    }
    let (a, b) = (grosso.live()[0], fino.live()[0]);
    assert!(
        (a.age - b.age).abs() < 1e-5,
        "as duas idades divergiram: {} contra {}",
        a.age,
        b.age
    );
    for i in 0..SPARKS {
        let (pa, oa) = a.spark(i).expect("viva");
        let (pb, ob) = b.spark(i).expect("viva");
        assert!(
            (pa[0] - pb[0]).abs() < 1e-3 && (pa[1] - pb[1]).abs() < 1e-3 && (oa - ob).abs() < 1e-4,
            "a particula {i} depende do RITMO: {pa:?}/{oa} contra {pb:?}/{ob}"
        );
    }
}

/// ⭐ **Duas faíscas no mesmo sítio não saem iguais**, e uma faísca morta não desenha nada.
///
/// ⚠️ **O controlo é a MESMA semente**: sem ele, um gerador partido que devolvesse sempre o mesmo
/// ângulo passaria a metade de cima (duas sementes diferentes dariam listas diferentes por acaso do
/// `i`), e a de baixo apanha-o.
#[test]
fn two_bursts_at_the_same_point_differ_and_a_dead_one_draws_nothing() {
    // ⚠️⚠️ **AS FAÍSCAS TÊM DE TER IDADE.** A 1.ª redacção comparou-as em `age = 0`, onde toda
    // partícula está **exactamente na origem** por construção — e o gate leu isso como *"as doze
    // coincidiram"*, acusando código correcto. *Uma fixtura sem o fenómeno lê-se como cura; esta
    // leu-se como defeito, que é o mesmo erro do outro lado.*
    let envelhecida = |seed| {
        let mut b = Burst::new([0.0, 0.0], seed);
        b.age = LIFE_S * 0.5;
        b
    };
    let a = envelhecida(1);
    let b = envelhecida(2);
    // O controlo do controlo: a meia-vida as partículas JÁ se afastaram da origem.
    assert!(
        a.spark(0).expect("viva").0 != [0.0, 0.0],
        "a fixtura ainda esta' na origem - ela nao contem o fenomeno"
    );
    let iguais = (0..SPARKS)
        .filter(|&i| a.spark(i).map(|s| s.0) == b.spark(i).map(|s| s.0))
        .count();
    assert_eq!(
        iguais, 0,
        "{iguais} particulas de duas sementes coincidiram"
    );
    // ⚠️ CONTROLO: a MESMA semente dá a MESMA faísca — senão isto media aleatoriedade, não semente.
    let c = envelhecida(1);
    for i in 0..SPARKS {
        assert_eq!(
            a.spark(i),
            c.spark(i),
            "a semente nao determina a particula {i}"
        );
    }
    // E a morte é total: nem posição, nem opacidade.
    let mut morta = Burst::new([0.0, 0.0], 1);
    morta.age = LIFE_S;
    assert!(morta.dead() && (0..SPARKS).all(|i| morta.spark(i).is_none()));
}

/// ⚠️ **A OPACIDADE cai monotonicamente e chega a zero** — uma faísca que some de repente lê-se como
/// um glitch, e uma que nunca chega a zero deixa lixo no ecrã.
#[test]
fn the_fade_is_monotonic_and_actually_reaches_zero() {
    let mut b = Burst::new([0.0, 0.0], 3);
    let mut anterior = f32::MAX;
    let mut passos = 0;
    while !b.dead() {
        let (_, o) = b.spark(0).expect("viva");
        assert!(o <= anterior, "a opacidade subiu: {o} depois de {anterior}");
        anterior = o;
        b.age += LIFE_S / 32.0;
        passos += 1;
    }
    assert!(
        passos >= 30,
        "a varredura mediu {passos} passos - fixtura curta demais"
    );
    assert!(
        anterior < 0.02,
        "a ultima opacidade foi {anterior} - a faisca some de repente"
    );
}
