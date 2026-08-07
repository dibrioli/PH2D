//! Gates das opções de vista.

use super::*;

/// **O rig é o `0`, e nenhum matcap pousa nele.**
///
/// ⚠️ É o gate do sentinela: o shader lê `matcap > 0` para escolher o caminho,
/// então um material que empacotasse em zero seria *invisível* — o artista
/// escolheria a pérola e veria a luz do documento, sem nada dizendo por quê.
#[test]
fn the_rig_is_zero_and_no_matcap_lands_there() {
    assert_eq!(ShadeRaw::pack(Shade::default()).matcap, 0);
    for i in 0..MATCAPS.len() {
        let packed = ShadeRaw::pack(Shade {
            matcap: Some(u8::try_from(i).expect("a tabela cabe num u8")),
            ..Shade::default()
        });
        assert_ne!(
            packed.matcap, 0,
            "o material {i} empacotou como \"sem matcap\""
        );
        assert_eq!(packed.matcap, i as u32 + 1);
    }
}

/// **Um índice fora da tabela é PRESO no último, nunca deixado passar.**
///
/// ⚠️ O `switch` do shader tem um braço `default`, e ele é um material legítimo
/// (a cera). Um índice inválido cairia lá e o artista veria uma resposta
/// plausível a um pedido impossível — o modo de falha que não se consegue
/// reportar.
#[test]
fn an_index_past_the_table_is_pinned_to_the_last_material() {
    let last = ShadeRaw::pack(Shade {
        matcap: Some(u8::try_from(MATCAPS.len() - 1).expect("cabe")),
        ..Shade::default()
    });
    for i in [MATCAPS.len() as u8, 200, u8::MAX] {
        assert_eq!(
            ShadeRaw::pack(Shade {
                matcap: Some(i),
                ..Shade::default()
            })
            .matcap,
            last.matcap,
            "o índice {i} escapou da tabela"
        );
    }
}

/// **A cavidade é clampada na porta.**
#[test]
fn the_cavity_is_clamped_at_the_door() {
    for (given, want) in [(-1.0, 0.0), (0.0, 0.0), (0.5, 0.5), (3.0, 1.0)] {
        assert!(
            (ShadeRaw::pack(Shade {
                cavity: given,
                ..Shade::default()
            })
            .cavity
                - want)
                .abs()
                < 1e-6
        );
    }
}

/// **O wireframe NÃO viaja no uniform** — ele é um passe, não um termo.
///
/// ⚠️ Gate de AUSÊNCIA, e ele é o que impede a próxima wave de "resolver" o
/// wireframe com um `if` no fragment: armá-lo não pode mover um byte do que o
/// shader lê, e é isso que se afirma.
#[test]
fn arming_the_wireframe_does_not_move_a_byte_of_the_uniform() {
    let off = ShadeRaw::pack(Shade::default());
    let on = ShadeRaw::pack(Shade {
        wireframe: true,
        ..Shade::default()
    });
    assert_eq!(bytemuck::bytes_of(&off), bytemuck::bytes_of(&on));
}

/// **A contagem de materiais da CPU é a do SHADER.**
///
/// ⚠️ Os números de cada material vivem no WGSL (nenhum consumidor de CPU os lê)
/// e os nomes vivem em [`MATCAPS`]. As duas listas não podem divergir em
/// TAMANHO: um nome a mais pinta um chip que cai no `default` do `switch`, um a
/// menos deixa um material inalcançável. O oráculo é o próprio fonte do shader —
/// contar os braços `case`/`default` do seletor.
#[test]
fn the_shader_has_exactly_one_arm_per_named_material() {
    let src = crate::pipeline::MESH_WGSL;
    let body = src
        .split_once("fn material(id: u32) -> Mat {")
        .expect("o seletor de material existe")
        .1
        .split_once("\n}")
        .expect("ele fecha")
        .0;
    let arms = body.matches("case ").count() + body.matches("default:").count();
    assert_eq!(
        arms,
        MATCAPS.len(),
        "o shader tem {arms} materiais e a tabela nomeia {}",
        MATCAPS.len()
    );
}
