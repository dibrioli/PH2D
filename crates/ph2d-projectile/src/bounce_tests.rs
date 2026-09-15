//! Gates do ricochete — a lei contra o corpus do oráculo.

use super::*;
use crate::{ProjectileLaw, len};

const EPS: f32 = 1.0e-5;

fn lei(bounciness: f32, max_bounces: u8) -> ProjectileLaw {
    ProjectileLaw {
        bounciness,
        max_bounces,
        ..ProjectileLaw::default()
    }
}

/// Um passo a voar para a direita contra uma parede vertical.
fn passo(budget: f32, steps_left: u8) -> SweepStep {
    SweepStep {
        dir: [1.0, 0.0],
        budget,
        steps_left,
    }
}

/// A normal de uma parede vertical à direita — ⚠️ **contra o movimento**, como a ponte a entrega.
const NORMAL: Vec2 = [-1.0, 0.0];

/// ⭐⭐⭐ **O ORÇAMENTO ATRAVESSA O RICOCHETE INTEIRO** — a cláusula 1 do corpus.
///
/// O oráculo mede `andou 3,984375 + resto 6,015625 = 10,000000`. Com `bounciness = 1`, o que sai
/// do salto tem de ser **exactamente** o resto.
#[test]
fn o_orcamento_atravessa_o_ricochete_inteiro() {
    let b = next_step(passo(10.0, 4), [10.0, 0.0], 3.984_375, NORMAL, &lei(1.0, 4))
        .expect("ha' resto e ha' tecto");
    assert!(
        (b.step.budget - 6.015_625).abs() < EPS,
        "o resto encolheu: {}",
        b.step.budget
    );
}

/// ⭐⭐⭐ **A DIRECÇÃO NOVA É O ESPELHO** — a cláusula 2, nos seis ângulos do corpus.
#[test]
fn a_direccao_nova_e_o_espelho_nos_seis_angulos_do_corpus() {
    for graus in [90.0_f32, 75.0, 60.0, 45.0, 30.0, 15.0] {
        let r = graus.to_radians();
        // `graus` é a incidência a partir de rasante: 90° é de cabeça contra a parede.
        let dir = normalize([r.sin(), r.cos()]).expect("unitaria");
        let v = [dir[0] * 12.0, dir[1] * 12.0];
        let b = next_step(
            SweepStep {
                dir,
                budget: 10.0,
                steps_left: 4,
            },
            v,
            0.0,
            NORMAL,
            &lei(1.0, 4),
        )
        .expect("bate");
        // Contra uma parede vertical o espelho inverte `x` e preserva `y`.
        assert!(
            (b.step.dir[0] + dir[0]).abs() < EPS,
            "{graus}°: x nao inverteu ({} contra {})",
            b.step.dir[0],
            -dir[0]
        );
        assert!(
            (b.step.dir[1] - dir[1]).abs() < EPS,
            "{graus}°: y nao sobreviveu"
        );
        assert!(
            (len(b.velocity) - 12.0).abs() < 1.0e-3,
            "{graus}°: a rapidez mudou num salto perfeito: {}",
            len(b.velocity)
        );
    }
}

/// ⚠️⚠️ **A `bounciness` toca nas DUAS grandezas, com o MESMO número.**
///
/// ⛔ Escalar só uma faria o resto deste tique e a rapidez do tique seguinte discordarem — e o
/// sintoma seria uma bala que anda mais do que a velocidade dela diz.
#[test]
fn a_perda_por_salto_escala_o_orcamento_e_a_velocidade_pelo_mesmo_numero() {
    let b = next_step(passo(10.0, 4), [12.0, 0.0], 0.0, NORMAL, &lei(0.5, 4)).expect("bate");
    assert!(
        (b.step.budget - 5.0).abs() < EPS,
        "orcamento {}",
        b.step.budget
    );
    assert!(
        (len(b.velocity) - 6.0).abs() < EPS,
        "rapidez {}",
        len(b.velocity)
    );
    let razao_orcamento = b.step.budget / 10.0;
    let razao_rapidez = len(b.velocity) / 12.0;
    assert!(
        (razao_orcamento - razao_rapidez).abs() < EPS,
        "as duas razoes tem de ser a MESMA: {razao_orcamento} contra {razao_rapidez}"
    );
}

/// ⭐⭐ **O TECTO fecha o plano** — a cláusula 3, e o oráculo mediu DOIS saltos num tique.
#[test]
fn o_tecto_fecha_o_plano() {
    assert!(
        next_step(passo(10.0, 0), [12.0, 0.0], 0.0, NORMAL, &lei(1.0, 4)).is_none(),
        "com o tecto a zero nao ha' salto"
    );
    let b = next_step(passo(10.0, 2), [12.0, 0.0], 0.0, NORMAL, &lei(1.0, 4)).expect("bate");
    assert_eq!(b.step.steps_left, 1, "o tecto desce um por salto");
}

/// ⚠️ **Um salto que não sobra orçamento nenhum não é um salto** — senão o plano gasta um a mover
/// nada, e o tecto esvazia-se contra uma parede em que o corpo já está encostado.
#[test]
fn um_salto_sem_orcamento_nao_acontece() {
    assert!(next_step(passo(10.0, 4), [12.0, 0.0], 10.0, NORMAL, &lei(1.0, 4)).is_none());
    // E uma `bounciness` que deixa o resto abaixo do piso também fecha o plano.
    assert!(next_step(passo(10.0, 4), [12.0, 0.0], 0.0, NORMAL, &lei(0.0, 4)).is_none());
}

/// ⚠️ **Sair da parede não é bater nela.** Com a normal a favor do movimento não há o que reflectir
/// — e reflectir mesmo assim mandaria o projéctil PARA DENTRO do sólido.
#[test]
fn nao_se_ricocheteia_numa_superficie_de_que_se_esta_a_afastar() {
    // O corpo vai para a direita e a normal aponta para a direita: ele está a sair.
    assert!(next_step(passo(10.0, 4), [12.0, 0.0], 0.0, [1.0, 0.0], &lei(1.0, 4)).is_none());
}

/// ⚠️ Uma normal degenerada não produz um salto — ela não tem direcção nenhuma.
#[test]
fn uma_normal_degenerada_nao_produz_salto() {
    assert!(next_step(passo(10.0, 4), [12.0, 0.0], 0.0, [0.0, 0.0], &lei(1.0, 4)).is_none());
}

/// ⚠️ Uma `bounciness` fora da faixa é **presa na porta**, nunca propagada — um valor negativo
/// inverteria a direcção do salto, e um acima de `1` faria a bala GANHAR energia a cada parede.
#[test]
fn uma_bounciness_fora_da_faixa_e_presa_na_porta() {
    let acima = next_step(passo(10.0, 4), [12.0, 0.0], 0.0, NORMAL, &lei(4.0, 4)).expect("bate");
    assert!(
        (acima.step.budget - 10.0).abs() < EPS,
        "acima de 1 fica em 1"
    );
    assert!((len(acima.velocity) - 12.0).abs() < EPS);
    assert!(
        next_step(passo(10.0, 4), [12.0, 0.0], 0.0, NORMAL, &lei(-2.0, 4)).is_none(),
        "abaixo de 0 fica em 0, e aí nao ha' orcamento"
    );
}
