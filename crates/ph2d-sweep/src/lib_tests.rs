//! Gates da folha do orçamento.

use super::*;

const EPS: f32 = 1.0e-6;

/// ⭐⭐⭐ **O ORÇAMENTO É `|v|·dt`** — a lei inteira num teste.
#[test]
fn o_orcamento_do_primeiro_passo_e_a_velocidade_vezes_o_passo() {
    let s = first_step([3.0, 4.0], 0.5, 4).expect("ha' velocidade");
    assert!((s.budget - 2.5).abs() < EPS, "budget {}", s.budget);
    assert!((len(s.dir) - 1.0).abs() < EPS, "a direccao e' unitaria");
    assert_eq!(s.steps_left, 4);
}

/// ⚠️ **Direcção ausente NÃO é direcção zero** — quem não tem para onde ir não pede nada ao mundo.
#[test]
fn sem_velocidade_nao_ha_passo() {
    assert!(first_step([0.0, 0.0], 0.5, 4).is_none());
    assert!(first_step([1.0e-9, 0.0], 0.5, 4).is_none());
}

/// ⚠️ Um `dt` que não é um passo não produz plano — ⛔ e `NaN` também não.
#[test]
fn um_dt_impossivel_nao_produz_plano() {
    for dt in [0.0_f32, -1.0, f32::NAN, f32::INFINITY] {
        assert!(first_step([1.0, 0.0], dt, 4).is_none(), "dt = {dt}");
    }
}

/// ⚠️ Um orçamento abaixo do piso não vale um passo — senão o plano gasta um a mover nada.
#[test]
fn um_orcamento_abaixo_do_piso_nao_vale_um_passo() {
    let quase = RESTO_MINIMO * 0.9;
    assert!(first_step([quase, 0.0], 1.0, 4).is_none());
    let chega = RESTO_MINIMO * 1.1;
    assert!(first_step([chega, 0.0], 1.0, 4).is_some());
}

/// ⭐⭐ **O RESTO conserva-se** — e o número vem do oráculo.
///
/// O corpus mede `andou 3,984375 + resto 6,015625 = 10,000000` num orçamento de `10` px. Aqui a
/// mesma aritmética, na unidade da casa.
#[test]
fn o_resto_e_o_orcamento_menos_o_que_coube() {
    let s = SweepStep {
        dir: [1.0, 0.0],
        budget: 10.0,
        steps_left: 4,
    };
    let r = remaining(s, 3.984_375).expect("sobra orcamento");
    assert!((r - 6.015_625).abs() < EPS, "resto {r}");
}

/// ⚠️ **O tecto fecha o plano**, e é ele que impede o ping-pong numa quina fechada.
#[test]
fn o_tecto_fecha_o_plano_mesmo_com_orcamento_de_sobra() {
    let s = SweepStep {
        dir: [1.0, 0.0],
        budget: 10.0,
        steps_left: 0,
    };
    assert!(
        remaining(s, 0.0).is_none(),
        "com o tecto a zero nao ha' passo"
    );
}

/// ⚠️ **Andar MAIS do que o orçamento não devolve um resto negativo** — um solver que devolva um
/// deslocamento com um epsilon a mais faria o plano continuar para trás.
#[test]
fn andar_mais_do_que_o_orcamento_fecha_o_plano() {
    let s = SweepStep {
        dir: [1.0, 0.0],
        budget: 10.0,
        steps_left: 4,
    };
    assert!(remaining(s, 10.5).is_none());
    // E um `moved` NEGATIVO conta como zero, nunca como orçamento a mais.
    assert!((remaining(s, -3.0).expect("sobra") - 10.0).abs() < EPS);
}

/// ⭐⭐⭐ **O ESPELHO, contra o corpus do oráculo.**
///
/// O Godot mede `erro 0,00000°` nos seis ângulos; aqui a lei é verificada pela propriedade que a
/// define — o ângulo de saída é o de entrada reflectido, e o comprimento **não muda**.
#[test]
fn o_espelho_conserva_o_comprimento_e_reflecte_o_angulo() {
    let n = [-1.0, 0.0]; // uma parede vertical, a normal contra o movimento
    for graus in [90.0_f32, 75.0, 60.0, 45.0, 30.0, 15.0] {
        let r = graus.to_radians();
        let v = [r.sin() * 12.0, r.cos() * 12.0];
        let m = mirror(v, n);
        assert!(
            (len(m) - len(v)).abs() < 1.0e-4,
            "{graus}°: o espelho mudou o comprimento ({} contra {})",
            len(m),
            len(v)
        );
        // Contra uma parede vertical o espelho inverte `x` e **preserva** `y`.
        assert!((m[0] + v[0]).abs() < 1.0e-4, "{graus}°: x nao inverteu");
        assert!((m[1] - v[1]).abs() < 1.0e-4, "{graus}°: y nao sobreviveu");
    }
}

/// ⚠️ **Reflectir duas vezes na mesma normal devolve o vector** — a involução que prova que a lei
/// é um espelho e não uma rotação qualquer.
#[test]
fn reflectir_duas_vezes_devolve_o_vector() {
    let n = normalize([1.0, 2.0]).expect("unitaria");
    let v = [3.0, -5.0];
    let ida = mirror(v, n);
    let volta = mirror(ida, n);
    assert!((volta[0] - v[0]).abs() < 1.0e-5 && (volta[1] - v[1]).abs() < 1.0e-5);
}
