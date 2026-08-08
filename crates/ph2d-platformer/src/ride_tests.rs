//! Os gates da PERNA (a cápsula flutuante) — `mod tests` do `ride.rs`.
//!
//! ⚠️ Módulo FILHO por `#[path]`, e não irmão: é isso que mantém `use super::*`
//! a alcançar o que não é `pub` (o precedente do `undo_delta_tests` do Painter).
use super::*;

const UP: Vec2 = [0.0, 1.0];
const G: Vec2 = [0.0, -9.81];

/// **O chão sente o peso de quem está parado, e não sente a puxada.**
#[test]
fn the_ground_feels_the_push_and_the_weight_but_never_the_pull() {
    let cfg = RideConfig::STARTING_POINT;

    // Esticada dentro do cling: a perna PUXA o personagem para baixo...
    let high = flat(cfg.float_height + 0.1);
    let motor = ride_spring(&cfg, Some(&high), [0.0, 0.0], G, UP);
    assert!(
        motor.accel[1] < 0.0,
        "a perna esticada puxa o personagem para BAIXO: {}",
        motor.accel[1]
    );
    // ...e o chão sente só o peso.
    let felt = ride_support_on_ground(Support::Spring, &cfg, Some(&high), G, UP);
    assert!(
        (felt.accel[1] - 9.81).abs() < 1.0e-4,
        "o chao sente o PESO e nada mais: {}",
        felt.accel[1]
    );

    // Comprimida: os dois concordam, ao bit.
    let low = flat(cfg.float_height - 0.05);
    let motor = ride_spring(&cfg, Some(&low), [0.0, 0.0], G, UP);
    let felt = ride_support_on_ground(Support::Spring, &cfg, Some(&low), G, UP);
    assert_eq!(
        motor.accel, felt.accel,
        "comprimida, o chao sente exatamente o que a perna faz"
    );
}

fn flat(distance: f32) -> GroundSample {
    GroundSample {
        distance,
        normal: [0.0, 1.0],
        ground_velocity: [0.0, 0.0],
        one_way: false,
    }
}

/// Sem chão ao alcance, a perna se cala — e é o que o pulo (W4) precisa para
/// existir.
#[test]
fn no_ground_means_no_motor() {
    let cfg = RideConfig::STARTING_POINT;
    assert_eq!(ride_spring(&cfg, None, [0.0, 0.0], G, UP), Motor::default());
    // Além do alcance também é ar, mesmo com o sensor tendo achado algo.
    let far = flat(cfg.float_height + cfg.cling_distance + 0.01);
    assert_eq!(
        ride_spring(&cfg, Some(&far), [0.0, 0.0], G, UP),
        Motor::default()
    );
}

/// **Na altura exata, a mola só segura o peso** — nada de empurrão extra.
///
/// É este o gate que prova a compensação da gravidade: sem ela a aceleração
/// aqui seria zero, o personagem cairia, e só pararia onde o erro da mola
/// igualasse o peso — mais baixo que a `float_height` pedida, por uma
/// quantidade que depende da gravidade do mundo.
#[test]
fn at_rest_height_the_spring_only_carries_the_weight() {
    let cfg = RideConfig::STARTING_POINT;
    let m = ride_spring(&cfg, Some(&flat(cfg.float_height)), [0.0, 0.0], G, UP);
    assert!(
        (m.accel[1] - 9.81).abs() < 1.0e-4,
        "a aceleracao tem de cancelar a gravidade exatamente: {:?}",
        m.accel
    );
    assert_eq!(m.boost, [0.0, 0.0], "parado, nada a amortecer");
}

/// Baixo demais empurra para CIMA; alto demais (dentro do cling) puxa para
/// BAIXO. A segunda metade é o que impede o personagem de subir escada
/// flutuando.
#[test]
fn the_spring_pushes_both_ways() {
    let cfg = RideConfig::STARTING_POINT;
    let low = ride_spring(&cfg, Some(&flat(cfg.float_height - 0.1)), [0.0, 0.0], G, UP);
    let high = ride_spring(&cfg, Some(&flat(cfg.float_height + 0.1)), [0.0, 0.0], G, UP);
    assert!(low.accel[1] > 9.81, "baixo demais: empurra alem do peso");
    assert!(high.accel[1] < 9.81, "alto demais: alivia e deixa descer");
}

/// ⚠️ **O TETO DO AMORTECIMENTO, medido em vez de citado.**
///
/// O boost remove `rel_v · d`. Em `d = 1` a velocidade relativa vai a zero;
/// em `d = 2` ela seria invertida inteira. O `clamp` da porta segura em
/// [`RideConfig::MAX_DAMPING`], então pedir 2.0 **não** inverte.
#[test]
fn the_damping_ceiling_is_where_the_boost_inverts() {
    let mut cfg = RideConfig::STARTING_POINT;
    let v = [0.0, -3.0]; // caindo a 3 m/s

    cfg.spring_damping = 1.0;
    let m = ride_spring(&cfg, Some(&flat(cfg.float_height)), v, G, UP);
    assert!(
        (v[1] + m.boost[1]).abs() < 1.0e-4,
        "em d=1 a velocidade relativa vai a ZERO: v={} boost={}",
        v[1],
        m.boost[1]
    );

    // Pedir o dobro do teto não pode inverter a velocidade: o clamp segura.
    cfg.spring_damping = 2.0;
    let m2 = ride_spring(&cfg, Some(&flat(cfg.float_height)), v, G, UP);
    let after = v[1] + m2.boost[1];
    assert!(
        after.abs() < 1.0e-4,
        "acima do teto o clamp segura em d=1 (nao inverte): sobrou {after}"
    );
}

/// ⚠️ **A altura de flutuação mínima é geometria** — e o ponto de partida
/// NÃO a satisfaz para a cápsula que os smokes usam.
///
/// O gate afirma as duas metades: a fórmula, e o fato desconfortável de que
/// `STARTING_POINT.float_height` deixa aquela cápsula **tangente** ao plano.
/// Sem a segunda, o número seria uma curiosidade; com ela, ele é uma dívida
/// nomeada (a semeadura pelo collider, W5).
#[test]
fn the_float_height_has_a_geometric_floor() {
    // A tabela do doc de `min_float_height`, conferida.
    let flat = RideConfig::min_float_height(0.3, 0.2, 1.0);
    assert!((flat - 0.5).abs() < 1.0e-6, "no plano: {flat}");
    let at45 = RideConfig::min_float_height(0.3, 0.2, core::f32::consts::FRAC_1_SQRT_2);
    assert!((at45 - 0.582_842_7).abs() < 1.0e-5, "a 45°: {at45}");
    let at60 = RideConfig::min_float_height(0.3, 0.2, 0.5);
    assert!((at60 - 0.7).abs() < 1.0e-6, "a 60°: {at60}");

    // ⚠️ E o ponto de partida NÃO paira sobre essa cápsula nem no plano.
    assert!(
        RideConfig::STARTING_POINT.float_height <= flat,
        "o STARTING_POINT deixa a capsula (0.3, 0.2) TANGENTE: {} vs o minimo {flat}",
        RideConfig::STARTING_POINT.float_height
    );

    // Vertical é degenerado: nenhuma altura basta.
    assert!(RideConfig::min_float_height(0.3, 0.2, 0.0).is_infinite());
}

/// ⚠️ **A mola mede tudo RELATIVO ao chão** — a metade que faz a plataforma
/// móvel funcionar sem uma linha de código de plataforma.
///
/// Um personagem parado SOBRE uma plataforma que sobe a 2 m/s sobe a 2 m/s
/// também. Se a mola amortecesse contra o mundo, ela leria isso como
/// oscilação e lutaria contra a plataforma — o personagem afundaria nela.
#[test]
fn the_spring_damps_against_the_ground_not_the_world() {
    let cfg = RideConfig::STARTING_POINT;
    let rising = GroundSample {
        distance: cfg.float_height,
        normal: [0.0, 1.0],
        ground_velocity: [0.0, 2.0],
        one_way: false,
    };
    // O corpo já viaja com a plataforma: velocidade relativa ZERO.
    let m = ride_spring(&cfg, Some(&rising), [0.0, 2.0], G, UP);
    assert_eq!(
        m.boost,
        [0.0, 0.0],
        "viajando junto com o chao nao ha' o que amortecer"
    );

    // E um corpo parado no MUNDO está descendo em relação à plataforma ⇒ a
    // mola o acelera para cima para acompanhá-la.
    let m2 = ride_spring(&cfg, Some(&rising), [0.0, 0.0], G, UP);
    assert!(
        m2.boost[1] > 0.0,
        "parado no mundo, ele esta' AFUNDANDO na plataforma: {:?}",
        m2.boost
    );
}

// ── O EIXO DO AMORTECEDOR ────────────────────────────────────────────────

/// **No plano o eixo É o `up`, bit a bit** — é esta linha que mantém tudo o
/// que a W2..W10 mediu em chão plano byte-idêntico.
#[test]
fn on_the_flat_the_damping_axis_is_up_to_the_bit() {
    assert_eq!(damping_axis([0.0, 1.0], UP), UP);
    // E o recuo da normal degenerada é o mesmo `up` — a política que a
    // `footing` já toma para um raio nascido dentro da geometria.
    assert_eq!(damping_axis([0.0, 0.0], UP), UP);
    // Fora do plano ele NORMALIZA (uma normal que chegue longa não pode
    // escalar o amortecimento).
    let a = damping_axis([0.0, 4.0], UP);
    assert!((a[1] - 1.0).abs() < 1.0e-6, "normalizado: {a:?}");
}

/// ⚠️ **O GATE DA CORREÇÃO: andar ao longo da rampa NÃO é aproximar-se dela.**
///
/// Um corpo cuja velocidade é puramente TANGENTE não está nem subindo nem
/// afundando na superfície, então a perna não tem o que amortecer. Com o
/// eixo no `up` ela amortecia `v · sen θ` — e é essa componente, invisível
/// para o freio da caminhada, que virava a subida involuntária.
///
/// **Mutação que deve sangrar:** trocar `axis` de volta por `up`.
#[test]
fn sliding_along_the_ramp_is_not_approaching_it() {
    // ⚠️ O amortecimento é FIXADO aqui, e não herdado do `STARTING_POINT`:
    // este gate julga o EIXO, e a metade de controle abaixo mede um número
    // que o default escala. Herdá-lo faria ele sangrar quando alguém mexesse
    // no default — verdade por acidente, e o gate deixaria de poder falhar
    // pelo motivo que alega.
    let cfg = RideConfig {
        spring_damping: 1.0,
        ..RideConfig::STARTING_POINT
    };
    // Rampa de 30° que sobe para a direita; a tangente é `perp_cw(n)`.
    let n: Vec2 = [-0.5, 0.866_025_4];
    let t = crate::perp_cw(n);
    let s = GroundSample {
        distance: cfg.float_height,
        normal: n,
        ground_velocity: [0.0, 0.0],
        one_way: false,
    };
    let along = [t[0] * 2.0, t[1] * 2.0];
    let m = ride_spring(&cfg, Some(&s), along, G, UP);
    assert!(
        m.boost[0].abs() < 1.0e-6 && m.boost[1].abs() < 1.0e-6,
        "andar ao longo da rampa nao pode acordar o amortecedor: {:?}",
        m.boost
    );

    // E a metade oposta: aproximar-se da superfície (ao longo da NORMAL)
    // acorda-o inteiro — senão o gate acima passaria com o amortecedor
    // deletado.
    let into = [-n[0] * 2.0, -n[1] * 2.0];
    let m2 = ride_spring(&cfg, Some(&s), into, G, UP);
    let push = m2.boost[0] * n[0] + m2.boost[1] * n[1];
    assert!(
        push > 1.9,
        "afundar na rampa TEM de ser amortecido (d = {}): {:?}",
        cfg.spring_damping,
        m2.boost
    );
}

/// **O amortecedor devolve exatamente o que o knob promete** — e no teto ele
/// zera a aproximação num tique, que é o que faz a deriva de rampa ser zero.
///
/// **Mutação que deve sangrar:** clampar o amortecimento abaixo de 1.
#[test]
fn at_the_ceiling_the_leg_kills_the_whole_approach_in_one_tick() {
    let mut cfg = RideConfig::STARTING_POINT;
    let n: Vec2 = [-0.5, 0.866_025_4];
    let s = GroundSample {
        distance: cfg.float_height,
        normal: n,
        ground_velocity: [0.0, 0.0],
        one_way: false,
    };
    let into = [-n[0] * 3.0, -n[1] * 3.0];
    cfg.spring_damping = RideConfig::MAX_DAMPING;
    let m = ride_spring(&cfg, Some(&s), into, G, UP);
    let left = (into[0] + m.boost[0]) * n[0] + (into[1] + m.boost[1]) * n[1];
    assert!(
        left.abs() < 1.0e-4,
        "no teto sobra ZERO de aproximacao: {left}"
    );
    // E metade do knob deixa metade — a linearidade que a tabela do
    // `STARTING_POINT` mede no produto.
    cfg.spring_damping = 0.5;
    let half = ride_spring(&cfg, Some(&s), into, G, UP);
    let left_half = (into[0] + half.boost[0]) * n[0] + (into[1] + half.boost[1]) * n[1];
    assert!(
        (left_half + 1.5).abs() < 1.0e-4,
        "metade do knob deixa metade da aproximacao: {left_half}"
    );
}

/// **O amortecimento que shipa é uma DECISÃO, não uma sobra** — e o gate
/// existe porque baixá-lo é uma armadilha com aparência de afinação.
///
/// ⚠️ **A decisão MUDOU de lado em 2026-08-05 e o gate mudou com ela.**
/// Enquanto o teto custava **metade do peso** do personagem, o default tinha
/// de ficar no meio e conviver com a deriva; depois da wave `gravity_hold`
/// ele custa **4 pontos** (95% → 91%), e aí a conta inverte: baixá-lo
/// devolve uma subida de `0,153 · sen θ · (1 − d)` metros a cada 10 s
/// **parado** — um personagem que anda sozinho — em troca de um quique de
/// pouso que só existe de verdade no fundo do knob (199 mm a `0,25`, 24 mm
/// a `0,50`, **zero já a partir de `0,75`**).
///
/// ⇒ Quem mover este número de volta está comprando 24 mm de quique com um
/// defeito, e é isso que este gate obriga a ler primeiro.
///
/// ⚠️ **E a troca tem um TERCEIRO eixo, medido na W26** — ver a tabela de
/// sub-passos no doc do [`RideConfig::STARTING_POINT`]. A frase acima
/// continua verdadeira **a sub-passos constantes**; quem baixa este knob
/// **e** sobe os `Sub-steps` compra o quique sem comprar a subida inteira.
#[test]
fn the_shipped_damping_is_a_decision_not_a_leftover() {
    assert_eq!(
        RideConfig::STARTING_POINT.spring_damping,
        RideConfig::MAX_DAMPING,
        "o default é o TETO desde 2026-08-05 — leia as quatro colunas antes"
    );
}
