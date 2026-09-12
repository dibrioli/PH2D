//! Os gates da [`super`] — **a lei pura**, sem GPU e sem mundo.
//!
//! ⚠️ Uma `Sculpt3dScene` pede `device`, superfície e viewport; um `SimWorld` pede o registo
//! inteiro. O que decide é a [`super::plan`], que só vê **ids** — e é por isso que ela existe
//! separada do braço que a aplica.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins sculpt3d::entities::tests
//! ```

use super::{SculptEntityMap, SculptRowsSeen, plan};

fn mundo(pares: &[(u32, u64)]) -> SculptEntityMap {
    pares.iter().copied().collect()
}

fn vistas(ids: &[u32]) -> SculptRowsSeen {
    ids.iter().copied().collect()
}

/// ⭐⭐⭐ **GATE — uma peça ⟺ uma linha, nas TRÊS direcções.**
#[test]
fn a_lei_e_uma_peca_uma_linha_nas_tres_direccoes() {
    // 1. Peça nova (nunca teve linha) ⇒ linha nova.
    let p = plan(&[7, 9], &mundo(&[]), &vistas(&[]));
    assert_eq!(p.spawn, vec![7, 9]);
    assert!(p.despawn.is_empty() && p.remove_pieces.is_empty());

    // 2. Peça que sumiu (o `Delete` do canvas) ⇒ a linha vai junto.
    let p = plan(&[7], &mundo(&[(7, 700), (9, 900)]), &vistas(&[7, 9]));
    assert_eq!(p.despawn, vec![900], "a linha da peca 9 tem de sair");
    assert!(p.spawn.is_empty() && p.remove_pieces.is_empty());

    // 3. Linha que sumiu (o `Delete` da Hierarquia) ⇒ a peça vai junto.
    let p = plan(&[7, 9], &mundo(&[(7, 700)]), &vistas(&[7, 9]));
    assert_eq!(
        p.remove_pieces,
        vec![9],
        "a peca da linha apagada tem de sair"
    );
    assert!(p.spawn.is_empty() && p.despawn.is_empty());
}

/// ⭐⭐⭐ **GATE — o plano é IDEMPOTENTE: com tudo em dia ele é VAZIO.**
///
/// ⛔ É o gate do defeito mais caro desta família — *spawnar um segundo conjunto de linhas a
/// cada quadro*, que é o que acontece quando o estado guardado e o mundo discordam em silêncio.
#[test]
fn com_tudo_em_dia_o_plano_e_vazio() {
    let p = plan(&[1, 2], &mundo(&[(1, 10), (2, 20)]), &vistas(&[1, 2]));
    assert!(p.is_empty(), "nada a fazer, e mesmo assim: {p:?}");
    // A ordem das peças não muda a resposta.
    let p = plan(&[2, 1], &mundo(&[(1, 10), (2, 20)]), &vistas(&[1, 2]));
    assert!(p.is_empty(), "{p:?}");
}

/// ⭐⭐⭐ **GATE — o RESTORE não apaga a escultura.**
///
/// ⛔⛔ É o gate da decisão que este módulo tomou contra as irmãs: um Ctrl+Z respawna as
/// entidades com **bits novos**, e uma ponte que guardasse os bits leria *«a entidade sumiu»* e
/// **apagaria a peça**. Aqui o mundo é lido a cada quadro: os bits mudam, o `id` não, e o plano
/// fica **vazio**.
#[test]
fn depois_de_um_restore_com_bits_novos_o_plano_e_vazio() {
    let antes = plan(&[3], &mundo(&[(3, 30)]), &vistas(&[3]));
    assert!(antes.is_empty(), "{antes:?}");
    // O undo respawnou: mesmos ids de peça, OUTROS bits de entidade.
    let depois = plan(&[3], &mundo(&[(3, 999_999)]), &vistas(&[3]));
    assert!(
        depois.is_empty(),
        "⛔ um restore nao pode pedir accao nenhuma: {depois:?}"
    );
}

/// ⭐⭐ **GATE — sem cena e sem linhas não há nada a fazer** (o quadro normal do app inteiro).
#[test]
fn sem_peca_e_sem_linha_nao_ha_nada_a_fazer() {
    let p = plan(&[], &mundo(&[]), &vistas(&[]));
    assert!(p.is_empty(), "{p:?}");
}
