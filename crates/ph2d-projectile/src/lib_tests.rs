//! Gates da lei do projéctil — o nascimento, as acelerações, o alcance e a flecha.

use super::*;

const EPS: f32 = 1.0e-5;
const DT: f32 = 1.0 / 60.0;

fn lei() -> ProjectileLaw {
    ProjectileLaw::default()
}

fn corre(st: &mut ProjectileState, law: &ProjectileLaw, facing: f32, tiques: u32) -> Vec2 {
    let mut v = st.velocity;
    for _ in 0..tiques {
        v = advance(st, law, facing, None, [0.0, 0.0], DT);
    }
    v
}

// ─────────────────────────────────────────────────────────────────────────────
// O NASCIMENTO

/// ⭐⭐ **Ele nasce para onde o corpo está virado, com a rapidez pedida.**
#[test]
fn ele_nasce_para_onde_o_corpo_esta_virado() {
    let mut st = ProjectileState::default();
    let v = advance(&mut st, &lei(), 0.0, None, [0.0, 0.0], DT);
    assert!((v[0] - 12.0).abs() < 1.0e-3 && v[1].abs() < 1.0e-3, "{v:?}");
    let mut st = ProjectileState::default();
    let v = advance(
        &mut st,
        &lei(),
        core::f32::consts::FRAC_PI_2,
        None,
        [0.0, 0.0],
        DT,
    );
    assert!(v[0].abs() < 1.0e-3 && (v[1] - 12.0).abs() < 1.0e-3, "{v:?}");
}

/// ⚠️⚠️ **O nascimento acontece UMA vez.** Se o `facing` fosse lido a cada tique, rodar o corpo a
/// meio do voo re-lançaria a bala — e o sintoma seria uma bala que muda de direcção sozinha.
#[test]
fn o_nascimento_acontece_uma_vez_so() {
    let mut st = ProjectileState::default();
    advance(&mut st, &lei(), 0.0, None, [0.0, 0.0], DT);
    // O corpo roda 90°: a bala JÁ voa, e não pode ser relançada.
    let v = advance(
        &mut st,
        &lei(),
        core::f32::consts::FRAC_PI_2,
        None,
        [0.0, 0.0],
        DT,
    );
    assert!((v[0] - 12.0).abs() < 1.0e-3, "a bala foi relancada: {v:?}");
    assert!(st.launched);
}

// ─────────────────────────────────────────────────────────────────────────────
// AS ACELERAÇÕES

/// ⭐ **A gravidade faz o arco** — e `-y` é para baixo.
#[test]
fn a_gravidade_faz_o_arco() {
    let law = ProjectileLaw {
        gravity: 10.0,
        ..lei()
    };
    let mut st = ProjectileState::default();
    let v = corre(&mut st, &law, 0.0, 60);
    assert!((v[0] - 12.0).abs() < 1.0e-3, "o x nao e' tocado: {v:?}");
    // Um segundo a 10 m/s² ⇒ −10 m/s.
    assert!((v[1] + 10.0).abs() < 0.05, "queda {}", v[1]);
}

/// ⚠️ **Sem velocidade não há direcção de VOO**, logo a aceleração de avanço não tem onde ser
/// aplicada — inventar uma faria uma bala parada arrancar sozinha.
#[test]
fn sem_velocidade_a_aceleracao_de_avanco_nao_inventa_direccao() {
    let law = ProjectileLaw {
        initial_speed: 0.0,
        acceleration: 50.0,
        ..lei()
    };
    let mut st = ProjectileState::default();
    let v = corre(&mut st, &law, 0.0, 10);
    assert!(len(v) < EPS, "ela arrancou sozinha: {v:?}");
}

/// ⚠️ **`max_speed = 0` é SEM TECTO, nunca «parado»** — um zero lido como limite pararia toda bala
/// que não declarasse o campo.
#[test]
fn max_speed_zero_e_sem_tecto() {
    let law = ProjectileLaw {
        acceleration: 100.0,
        max_speed: 0.0,
        ..lei()
    };
    let mut st = ProjectileState::default();
    let v = corre(&mut st, &law, 0.0, 60);
    assert!(len(v) > 100.0, "o zero travou a bala: {}", len(v));
}

/// ⭐ **E com tecto ele é respeitado**, sem mudar a direcção.
#[test]
fn com_tecto_a_rapidez_para_la_e_a_direccao_fica() {
    let law = ProjectileLaw {
        acceleration: 100.0,
        max_speed: 20.0,
        ..lei()
    };
    let mut st = ProjectileState::default();
    let v = corre(&mut st, &law, 0.0, 120);
    assert!((len(v) - 20.0).abs() < EPS, "rapidez {}", len(v));
    assert!(v[1].abs() < EPS, "a direccao torceu: {v:?}");
}

/// ⭐⭐⭐ **AS ACELERAÇÕES ACUMULAM e UM integrador aplica.**
///
/// ⚠️ É a lei dos Motion Nodes desta casa. O gate mede-a pela **soma**: correr as três sozinhas e
/// somar os deltas tem de dar o mesmo que correr as três juntas.
#[test]
fn as_aceleracoes_acumulam_e_um_integrador_aplica() {
    let base = ProjectileLaw {
        initial_speed: 12.0,
        ..lei()
    };
    let so_gravidade = ProjectileLaw {
        gravity: 10.0,
        ..base
    };
    let so_avanco = ProjectileLaw {
        acceleration: 30.0,
        ..base
    };
    let ambas = ProjectileLaw {
        gravity: 10.0,
        acceleration: 30.0,
        ..base
    };
    let delta = |law: &ProjectileLaw| {
        let mut st = ProjectileState::default();
        let v = advance(&mut st, law, 0.0, None, [0.0, 0.0], DT);
        // O primeiro tique nasce E integra, então o delta é contra a velocidade de nascimento.
        [v[0] - 12.0, v[1]]
    };
    let g = delta(&so_gravidade);
    let a = delta(&so_avanco);
    let j = delta(&ambas);
    assert!(
        (j[0] - (g[0] + a[0])).abs() < EPS && (j[1] - (g[1] + a[1])).abs() < EPS,
        "as tres nao compoem: juntas {j:?} contra a soma {:?}",
        [g[0] + a[0], g[1] + a[1]]
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// O HOMING

/// ⭐⭐ **Ele persegue** — e a curva aponta para onde o alvo ESTÁ.
#[test]
fn ele_persegue_o_alvo() {
    let law = ProjectileLaw {
        homing_accel: 200.0,
        ..lei()
    };
    let mut st = ProjectileState::default();
    // A voar para a direita, com o alvo bem acima.
    let mut v = [0.0, 0.0];
    for _ in 0..30 {
        v = advance(&mut st, &law, 0.0, Some([10.0, 40.0]), [0.0, 0.0], DT);
    }
    assert!(v[1] > 5.0, "ele nao curvou para o alvo: {v:?}");
}

/// ⚠️ **`homing_accel = 0` ignora o alvo** — senão um campo de alvo esquecido no Inspector puxaria
/// toda bala da cena.
#[test]
fn sem_aceleracao_de_perseguicao_o_alvo_e_ignorado() {
    let mut st = ProjectileState::default();
    let mut v = [0.0, 0.0];
    for _ in 0..30 {
        v = advance(&mut st, &lei(), 0.0, Some([0.0, 100.0]), [0.0, 0.0], DT);
    }
    assert!(v[1].abs() < EPS, "o alvo puxou sem aceleracao: {v:?}");
}

/// ⚠️ **Um alvo em cima do projéctil não produz direcção** — e dividir por zero ali daria `NaN`
/// numa pose, que é o defeito que mata um quadro inteiro.
#[test]
fn um_alvo_em_cima_do_projectil_nao_produz_nan() {
    let law = ProjectileLaw {
        homing_accel: 200.0,
        ..lei()
    };
    let mut st = ProjectileState::default();
    let v = advance(&mut st, &law, 0.0, Some([3.0, 3.0]), [3.0, 3.0], DT);
    assert!(v[0].is_finite() && v[1].is_finite(), "{v:?}");
}

// ─────────────────────────────────────────────────────────────────────────────
// O ALCANCE E A FLECHA

/// ⭐⭐ **O alcance conta metros PERCORRIDOS, não tempo.**
///
/// ⚠️ É o que o separa do `Lifetime` do #12: *uma bala lenta e uma rápida com o mesmo tempo de vida
/// têm alcances diferentes*.
#[test]
fn o_alcance_conta_metros_percorridos() {
    let law = ProjectileLaw {
        range: 5.0,
        ..lei()
    };
    let mut st = ProjectileState::default();
    assert!(ended(&st, &law, false).is_none());
    st.travelled = 4.99;
    assert!(ended(&st, &law, false).is_none());
    st.travelled = 5.0;
    assert_eq!(ended(&st, &law, false), Some(Ended::Range));
}

/// ⚠️ **`range = 0` é SEM LIMITE** — um zero lido como alcance mataria toda bala ao nascer.
#[test]
fn alcance_zero_e_sem_limite() {
    let st = ProjectileState {
        travelled: 1.0e6,
        ..ProjectileState::default()
    };
    assert!(ended(&st, &lei(), false).is_none());
}

/// ⭐⭐ **Bater com o tecto gasto acaba o voo** — e `max_bounces = 0` acaba no PRIMEIRO toque.
#[test]
fn bater_com_o_tecto_gasto_acaba_o_voo() {
    let st = ProjectileState::default();
    assert_eq!(ended(&st, &lei(), true), Some(Ended::Bounces));
    // ⚠️ E sem tocar em nada ele continua, por mais saltos que já tenha gasto.
    assert!(ended(&st, &lei(), false).is_none());
    let law = ProjectileLaw {
        max_bounces: 2,
        ..lei()
    };
    let mut st = ProjectileState {
        bounces_used: 1,
        ..ProjectileState::default()
    };
    assert!(ended(&st, &law, true).is_none(), "ainda ha' um salto");
    st.bounces_used = 2;
    assert_eq!(ended(&st, &law, true), Some(Ended::Bounces));
}

/// ⚠️ **O alcance ganha ao toque no relatório** — uma bala que gasta o último metro contra uma
/// parede já percorreu o alcance, e «morreu de bater» seria o motivo errado.
#[test]
fn o_alcance_ganha_ao_toque_no_motivo() {
    let law = ProjectileLaw {
        range: 5.0,
        ..lei()
    };
    let st = ProjectileState {
        travelled: 5.0,
        ..ProjectileState::default()
    };
    assert_eq!(ended(&st, &law, true), Some(Ended::Range));
}

/// ⭐ **A flecha aponta para onde voa, e é INSTANTÂNEO.**
#[test]
fn a_flecha_aponta_para_onde_voa() {
    let a = facing_of([0.0, 3.0], &lei()).expect("ha' velocidade");
    assert!((a - core::f32::consts::FRAC_PI_2).abs() < EPS, "{a}");
    // ⚠️ Desligada, ela não escreve ângulo nenhum — e não é o mesmo que escrever zero.
    let law = ProjectileLaw {
        face_velocity: false,
        ..lei()
    };
    assert!(facing_of([0.0, 3.0], &law).is_none());
    // ⚠️ E sem velocidade não há ângulo: o corpo fica onde estava.
    assert!(facing_of([0.0, 0.0], &lei()).is_none());
}

/// ⚠️ Um `dt` impossível não mexe na velocidade — ⛔ e `NaN` também não.
#[test]
fn um_dt_impossivel_nao_mexe_na_velocidade() {
    for dt in [0.0_f32, -1.0, f32::NAN] {
        let mut st = ProjectileState {
            velocity: [7.0, 0.0],
            launched: true,
            ..ProjectileState::default()
        };
        let v = advance(&mut st, &lei(), 0.0, None, [0.0, 0.0], dt);
        assert!(
            (v[0] - 7.0).abs() < EPS && v[1].abs() < EPS,
            "dt {dt}: {v:?}"
        );
    }
}
