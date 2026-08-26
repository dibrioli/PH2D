//! Gates do **ESTADO ANGULAR** (doc 89, folha 13 — *POP Spin* / *POP Drag Spin*).
//!
//! A lei tem quatro metades: sem a coluna `spin` nada acontece (e *nada* aqui é bit-a-bit),
//! o giro integra no `rot`, o arrasto angular tira do giro pela mesma forma de primeira ordem
//! do irmão linear, e uma peça divergida repõe-se em vez de envenenar a zona.

use super::*;

/// Um estado de `n` peças com `spin` autorado, e opcionalmente um `rot` de partida.
fn spinning(spins: &[f32], rot: Option<&[f32]>) -> Stream {
    let n = spins.len();
    let mut s = Stream::new(n);
    s.set("P", Column::Vec2(vec![[0.0, 0.0]; n]));
    s.set("vel", Column::Vec2(vec![[0.0, 0.0]; n]));
    s.set("sim_t", Column::Scalar(vec![0.0; n]));
    s.set("spin", Column::Scalar(spins.to_vec()));
    if let Some(r) = rot {
        s.set("rot", Column::Scalar(r.to_vec()));
    }
    s
}

fn scalar(s: &Stream, name: &str) -> Option<Vec<f32>> {
    match s.get(name) {
        Some(Column::Scalar(v)) => Some(v.clone()),
        _ => None,
    }
}

/// ⭐ **O CONTROLE.** Um estado que nunca autorou `spin` sai exactamente como saía — e a
/// afirmação é sobre as COLUNAS, não só sobre os valores: cunhar um `rot` que não existia
/// mudaria o que todo nó a jusante vê, e seria uma regressão silenciosa em toda cena antiga.
#[test]
fn a_state_without_a_spin_column_is_the_step_that_always_shipped() {
    let mut s = Stream::new(3);
    s.set("P", Column::Vec2(vec![[1.0, 2.0]; 3]));
    s.set("vel", Column::Vec2(vec![[0.5, 0.0]; 3]));
    s.set("sim_t", Column::Scalar(vec![0.0; 3]));
    for angular in [0.0f32, 0.5, 1.0] {
        let out = step(&s, 0.02, 1.0, 0.0, 0.0, angular);
        assert!(
            out.get("spin").is_none(),
            "angular {angular}: a coluna `spin` NAO pode nascer aqui"
        );
        assert!(
            out.get("rot").is_none(),
            "angular {angular}: nem a `rot` -- ninguem a autorou"
        );
    }
    // E um `rot` que EXISTE sem `spin` passa intacto: girar é o que o `spin` faz, e sem ele
    // o ângulo é um dado como outro qualquer.
    let mut with_rot = s.clone();
    with_rot.set("rot", Column::Scalar(vec![10.0, 20.0, 30.0]));
    let out = step(&with_rot, 0.02, 1.0, 0.0, 0.0, 0.4);
    assert_eq!(scalar(&out, "rot"), Some(vec![10.0, 20.0, 30.0]));
}

/// O giro integra no ângulo: `rot += spin · dt`, e um `dt` de zero não move nada.
///
/// ⚠️ **O `dt` desta suíte fica ABAIXO do [`MAX_DT`]**, e a 1.ª versão dela não ficava: eu
/// pedi `dt = 0,25` e li `12,7` onde esperava `32,5`. O nó **clampa** o passo em `0,03` — a
/// declaração está no doc-comment dele —, então eu tinha escrito a expectativa de um
/// integrador sem tecto e acusado o que o tinha. *Uma expectativa que ignora um limite
/// declarado mede o limite, não a lei.*
#[test]
fn the_spin_integrates_into_the_angle() {
    let dt = 0.02f32;
    let out = step(
        &spinning(&[90.0, -40.0], Some(&[10.0, 0.0])),
        dt,
        1.0,
        0.0,
        0.0,
        1.0,
    );
    assert_eq!(
        scalar(&out, "rot"),
        Some(vec![10.0 + 90.0 * dt, -40.0 * dt])
    );
    assert_eq!(
        scalar(&out, "spin"),
        Some(vec![90.0, -40.0]),
        "sem arrasto o giro nao muda"
    );
    // `sim_t == playhead` ⇒ `dt = 0` ⇒ nada anda, e o ângulo é o autorado.
    let still = step(&spinning(&[90.0], Some(&[7.0])), 0.0, 1.0, 0.0, 0.0, 1.0);
    assert_eq!(scalar(&still, "rot"), Some(vec![7.0]));
}

/// ⚠️ **Sem `rot` autorado o ângulo parte de ZERO** — a mesma ausência que a `identity` do
/// binding do device declara. As duas metades leem a falta da mesma maneira, ou a paridade
/// CPU/GPU seria uma coincidência.
#[test]
fn a_missing_angle_starts_at_zero_on_both_paths() {
    let out = step(&spinning(&[180.0], None), 0.02, 1.0, 0.0, 0.0, 1.0);
    assert_eq!(scalar(&out, "rot"), Some(vec![180.0 * 0.02]));
}

/// ⭐ **E o TECTO do passo vale para o giro também.** O `MAX_DT` existe para um quadro
/// perdido não teleportar uma peça; se a metade angular o ignorasse, o mesmo quadro perdido
/// dava-lhe uma volta inteira enquanto a posição mal se mexia — e as duas metades da mesma
/// peça descreveriam movimentos de relógios diferentes.
#[test]
fn the_step_ceiling_binds_the_angle_as_it_binds_the_position() {
    let huge = step(&spinning(&[360.0], Some(&[0.0])), 10.0, 1.0, 0.0, 0.0, 1.0);
    let rot = scalar(&huge, "rot").expect("sai");
    assert!(
        (rot[0] - 360.0 * MAX_DT).abs() < 1e-4,
        "um quadro perdido de 10 s tem de girar {} e nao 3600: {rot:?}",
        360.0 * MAX_DT
    );
}

/// O arrasto angular tira do giro pela MESMA forma de primeira ordem do amortecimento linear
/// — e em `1,0` o factor é exactamente `1`, então uma peça sem arrasto gira com os mesmos bits.
#[test]
fn the_angular_drag_is_the_first_order_twin_of_the_linear_one() {
    let dt = 0.02f32;
    // `keep = 1 - (1 - 0,5)·0,02 = 0,99`.
    let out = step(&spinning(&[100.0], Some(&[0.0])), dt, 1.0, 0.0, 0.0, 0.5);
    let spin = scalar(&out, "spin").expect("a coluna sai");
    assert!((spin[0] - 99.0).abs() < 1e-4, "spin {spin:?}");
    // O ângulo usa o valor NOVO (semi-implícito, como o irmão linear usa a velocidade nova).
    let rot = scalar(&out, "rot").expect("a coluna sai");
    assert!((rot[0] - 99.0 * dt).abs() < 1e-4, "rot {rot:?}");
    // ⭐ E em `1,0`, bit a bit.
    let undamped = step(&spinning(&[100.0], Some(&[0.0])), dt, 1.0, 0.0, 0.0, 1.0);
    assert_eq!(
        scalar(&undamped, "spin").map(|v| v[0].to_bits()),
        Some(100.0f32.to_bits())
    );
}

/// A rede: uma peça que divergiu repõe-se em vez de congelar (ou envenenar com `NaN`) a zona
/// inteira — a mesma decisão que a metade linear toma, e pelo mesmo motivo.
#[test]
fn a_diverged_element_keeps_the_value_it_had() {
    let dt = 0.02f32;
    let out = step(
        &spinning(&[f32::INFINITY, 30.0], Some(&[5.0, 5.0])),
        dt,
        1.0,
        0.0,
        0.0,
        1.0,
    );
    let (spin, rot) = (
        scalar(&out, "spin").expect("sai"),
        scalar(&out, "rot").expect("sai"),
    );
    assert_eq!(spin[0], f32::INFINITY, "o valor divergido fica onde estava");
    assert_eq!(rot[0], 5.0, "e o angulo dele nao anda");
    assert_eq!(
        rot[1],
        5.0 + 30.0 * dt,
        "a peca sa' ao lado anda normalmente"
    );
}

/// A porta pura, medida directamente — é ela que o WGSL porta termo a termo.
#[test]
fn the_one_door_is_the_pair_that_the_kernel_ports() {
    assert_eq!(spin_step(100.0, 0.0, 0.02, 1.0), (100.0, 2.0));
    let (s, r) = spin_step(100.0, 10.0, 0.02, 0.5);
    assert!((s - 99.0).abs() < 1e-4 && (r - (10.0 + 99.0 * 0.02)).abs() < 1e-4);
}
