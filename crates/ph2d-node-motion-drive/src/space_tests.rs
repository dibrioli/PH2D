//! Gates do **ESPAÇO** do `motion.drive` (doc 89, folha 06, célula 41 — o *Transform Space*
//! do C4D).
//!
//! ⚠️ Arquivo próprio por teto de LOC (HR-18): o `tests.rs` já mede mais de 500 linhas.
//!
//! ## Por que isto é capacidade e não ergonomia
//!
//! O `value.math` tem **dezassete** operações e **nenhuma trigonométrica**, então não existe
//! cadeia de nós capaz de virar a coluna `rot` numa direcção. Sem este param o espaço do
//! elemento é **inexprimível**.
//!
//! ⚠️ **A enumeração que o prova NÃO mora aqui**, e a razão é o ADR-0075: uma crate-nó não
//! pode depender de outra, muito menos do `registry-init` (seria um ciclo). Ela vive em
//! `ph2d-node-registry-init/tests/no_value_op_is_trigonometric.rs`, que é a única casa de onde
//! o catálogo inteiro é visível.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};

/// Uma fileira de três peças na origem, cada uma virada para um lado diferente.
fn turned() -> Stream {
    Stream::new(3)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; 3]))
        .with("rot", Column::Scalar(vec![0.0, 90.0, 180.0]))
}

fn pos(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("P"),
    }
}

/// ⭐⭐ **A ENTREGA: cada peça anda para onde ELA aponta.** Empurrar `X` por `1` no espaço do
/// elemento leva a de `0°` para `+x`, a de `90°` para `+y` e a de `180°` para `−x`.
#[test]
fn each_element_moves_along_its_own_axis() {
    let out = pos(&channel::drive_channel(
        &turned(),
        0, // X
        &[1.0],
        1.0,
        combine::Combine::Add,
        true,
    ));
    let want = [[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0]];
    for (i, (got, w)) in out.iter().zip(&want).enumerate() {
        assert!(
            (got[0] - w[0]).abs() < 1e-3 && (got[1] - w[1]).abs() < 1e-3,
            "peca {i}: {got:?} contra {w:?}"
        );
    }
    // ⚠️ O CONTROLE: no espaço do MUNDO as três andam para o mesmo lado. Sem esta metade o
    // gate acima passaria com um espaço que não fizesse nada de especial.
    let world = pos(&channel::drive_channel(
        &turned(),
        0,
        &[1.0],
        1.0,
        combine::Combine::Add,
        false,
    ));
    assert!(
        world
            .iter()
            .all(|p| (p[0] - 1.0).abs() < 1e-6 && p[1].abs() < 1e-6),
        "no mundo as tres andam para +x: {world:?}"
    );
}

/// **O `Y` local é PERPENDICULAR ao `X` local**, e é o que faz «para o lado» querer dizer
/// alguma coisa. Numa peça a `0°` ele é `+y`; numa a `90°`, `−x`.
#[test]
fn the_local_y_is_the_quarter_turn_of_the_local_x() {
    let out = pos(&channel::drive_channel(
        &turned(),
        1, // Y
        &[1.0],
        1.0,
        combine::Combine::Add,
        true,
    ));
    let want = [[0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]];
    for (i, (got, w)) in out.iter().zip(&want).enumerate() {
        assert!(
            (got[0] - w[0]).abs() < 1e-3 && (got[1] - w[1]).abs() < 1e-3,
            "peca {i}: {got:?} contra {w:?}"
        );
    }
}

/// ⭐ **UMA CENA QUE NÃO GIRA NADA É BYTE-IDÊNTICA.** Com `rot = 0` o eixo local é `(1, 0)`
/// EXACTO — a aproximação da casa bate com a trigonometria real nos quartos de volta —, então
/// ligar o espaço do elemento não move um bit. É o que protege todo documento salvo.
#[test]
fn without_rotation_the_element_space_is_the_world_space_bit_for_bit() {
    let flat = Stream::new(4).with(
        "P",
        Column::Vec2(vec![[1.0, 2.0], [-3.0, 0.5], [0.0, 0.0], [9.0, -9.0]]),
    );
    for mode in [
        combine::Combine::Add,
        combine::Combine::Set,
        combine::Combine::Multiply,
        combine::Combine::Subtract,
    ] {
        for ch in [0, 1] {
            let w = pos(&channel::drive_channel(&flat, ch, &[2.5], 1.0, mode, false));
            let e = pos(&channel::drive_channel(&flat, ch, &[2.5], 1.0, mode, true));
            assert_eq!(w, e, "canal {ch}, modo {}", mode as u8);
        }
    }
}

/// ⚠️ **O MODO continua a querer dizer o que dizia.** O que se projecta é o DELTA que o drive
/// teria aplicado, então um `Multiply` ainda multiplica a componente — o resultado é que ele
/// anda torto em vez de recto, e não que ele vira um `Add`.
#[test]
fn the_mode_still_decides_the_magnitude_the_space_only_turns_it() {
    let s = Stream::new(1)
        .with("P", Column::Vec2(vec![[4.0, 0.0]]))
        .with("rot", Column::Scalar(vec![90.0]));
    // Mundo: `x` vai de 4 para 8 (×2). Elemento: o mesmo delta (+4) cai em `+y`.
    let w = pos(&channel::drive_channel(
        &s,
        0,
        &[2.0],
        1.0,
        combine::Combine::Multiply,
        false,
    ));
    let e = pos(&channel::drive_channel(
        &s,
        0,
        &[2.0],
        1.0,
        combine::Combine::Multiply,
        true,
    ));
    assert!((w[0][0] - 8.0).abs() < 1e-4, "mundo: {:?}", w[0]);
    assert!(
        (e[0][0] - 4.0).abs() < 1e-3 && (e[0][1] - 4.0).abs() < 1e-3,
        "elemento: o mesmo delta, virado: {:?}",
        e[0]
    );
}

/// **Uma coluna `rot` AUSENTE cai no espaço do mundo**, e não em lixo — a identidade é `0`,
/// como no binding do device.
#[test]
fn a_stream_without_rotation_falls_on_the_world_axis() {
    let bare = Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 1.0]]));
    assert_eq!(
        pos(&channel::drive_channel(
            &bare,
            0,
            &[3.0],
            1.0,
            combine::Combine::Add,
            true
        )),
        pos(&channel::drive_channel(
            &bare,
            0,
            &[3.0],
            1.0,
            combine::Combine::Add,
            false
        )),
    );
}

/// ⚠️ **O WGSL é uma string e não vê consts do Rust** — este gate cruza a aproximação dos dois
/// lados pelos literais dela, que é a mesma rede que o `AXIS_SEED_OFFSET` do `motion.noise` usa.
#[test]
fn the_wgsl_carries_the_same_parabolic_sine_as_the_rust() {
    for needle in [
        "0.225 * (pp * abs(pp) - pp) + pp",
        "drive_local_axis",
        "rot_deg / 360.0",
    ] {
        assert!(
            crate::kernel::DRIVE_LIB.contains(needle),
            "o WGSL tem de conter `{needle}`"
        );
    }
    // E o param entrou na lista do kernel — sem ele o device lê um `params.space` que não existe.
    assert!(
        crate::kernel::DRIVE_PARAMS.contains(&"space"),
        "o kernel declara `space`"
    );
}
