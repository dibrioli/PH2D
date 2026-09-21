//! Os gates da travessia — ver o `//!` do irmão para porque ela existe.

use super::{a_guardar, a_usar};
use ph2d_mesh_render::{Framing, ViewRegion};

/// ⭐⭐⭐ **A IDA-E-VOLTA, ao bit** — é ela que torna a duplicação declarada dos dois tipos
/// honesta em vez de uma segunda resposta a envelhecer.
///
/// ⚠️ **O `size` do argumento é deliberadamente ABSURDO** (`7×3`, que não é o aspecto nenhum dos
/// recortes): se a volta o lesse, o aspecto voltaria `2,333` e o gate reprovava. *Um argumento que
/// o caminho certo nunca toca é o controlo mais barato que existe.*
#[test]
fn o_enquadramento_vai_e_volta_ao_bit() {
    for f in [
        Framing {
            aspect: 1.777_777_8,
            region: ViewRegion {
                origin: [0.15, 0.13],
                size: [0.325, 0.42],
            },
        },
        Framing {
            aspect: 0.5,
            region: ViewRegion {
                origin: [-0.112_5, -0.1],
                size: [0.437_5, 0.688],
            },
        },
        Framing {
            aspect: 2.333_333_3,
            region: ViewRegion::FULL,
        },
    ] {
        let volta = a_usar(Some(a_guardar(f)), (7, 3));
        assert_eq!(volta, f, "a travessia perdeu bits em {f:?}");
    }
}

/// ⚠️ **A AUSÊNCIA é uma resposta e não um valor de fábrica disfarçado:** um documento anterior a
/// esta wave não traz recorte nenhum, e o que ele quer dizer é *«a vista inteira»* — que é o que
/// aquele ficheiro produzia.
#[test]
fn sem_recorte_o_alvo_e_a_vista_inteira() {
    let f = a_usar(None, (1024, 512));
    assert_eq!(f.region, ViewRegion::FULL);
    assert!(f.region.is_full());
    assert!(
        (f.aspect - 2.0).abs() < 1e-6,
        "sem recorte o aspecto e' o do ALVO: {}",
        f.aspect
    );
}

/// ⭐ **O CONTROLO da metade de cima:** com recorte, o aspecto vem do RECORTE e o tamanho do alvo
/// não é lido. Sem esta metade, uma volta que derivasse o aspecto do `size` passaria a ida-e-volta
/// em toda fixtura cujo sprite fosse, por acaso, do aspecto da vista.
#[test]
fn com_recorte_o_aspecto_e_o_da_vista_e_nao_o_do_alvo() {
    let guardado = a_guardar(Framing {
        aspect: 1.777_777_8,
        region: ViewRegion {
            origin: [0.2, 0.1],
            size: [0.3, 0.4],
        },
    });
    // Um alvo QUADRADO, que é o caso comum de um sprite — e cujo aspecto (`1`) não pode aparecer.
    let f = a_usar(Some(guardado), (512, 512));
    assert!(
        (f.aspect - 1.777_777_8).abs() < 1e-7,
        "o aspecto veio do alvo em vez da vista: {}",
        f.aspect
    );
}
