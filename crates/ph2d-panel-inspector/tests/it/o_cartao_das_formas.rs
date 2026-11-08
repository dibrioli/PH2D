//! **A lista «Shapes» do cartão Shape** (W3-B, `docs/GameEngineFuncional/componentes/f2_contorno_e_formas_plano.md`):
//! as linhas pintadas na ordem do snapshot, e o clique REAL em cada uma chega ao barramento com o efeito certo.

use ph2d_editor_core::action_bus::{EditorAction, SelectModifier};
use ph2d_editor_core::screens::hero::{InspectorPhysicsInfo, PhysicsFieldEdit as E};
use ph2d_panel_inspector::ids::*;

use crate::o_cartao_da_fisica::{achado, cena, clica, pinta, sai};
use crate::seam_physics::with_body;

const PAREDE: u64 = 0x51;
const TETO: u64 = 0x52;

/// Um chão com duas peças, como o «Floor» da cena 120 (a sua forma, «Invisible Wall» e mais uma).
fn chao() -> InspectorPhysicsInfo {
    let b = with_body();
    InspectorPhysicsInfo {
        part_count: 2,
        shapes: vec![
            (b.entity_bits, "Floor".into(), 1),
            (PAREDE, "Invisible Wall".into(), 1),
            (TETO, "Roof".into(), 0),
        ],
        ..b
    }
}

#[test]
fn um_corpo_com_duas_pecas_pinta_tres_linhas_na_ordem_da_arvore() {
    let (mut host, mut st) = cena(chao(), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    let ys: Vec<f32> = (0..3)
        .map(|i| achado(&r, INSP_PHYS_SHAPE_ROW[i]).expect("linha pintada").y)
        .collect();
    assert!(ys[0] < ys[1] && ys[1] < ys[2], "fora de ordem: {ys:?}");
    assert!(achado(&r, INSP_PHYS_SHAPE_ROW[3]).is_none(), "linha a mais");
    assert!(
        achado(&r, INSP_PHYS_SHAPE_DEL[0]).is_none(),
        "o corpo não se remove daqui"
    );
    assert!(
        achado(&r, INSP_PHYS_SHAPE_DEL[1]).is_some()
            && achado(&r, INSP_PHYS_SHAPE_DEL[2]).is_some()
    );
    assert!(achado(&r, INSP_PHYS_ADD_PART).is_some(), "+ Add shape");
    sai();
}

#[test]
fn uma_forma_so_nao_pinta_linhas_mas_oferece_add_shape() {
    let b = with_body();
    let so = InspectorPhysicsInfo {
        shapes: vec![(b.entity_bits, "Floor".into(), 1)],
        ..b
    };
    let (mut host, mut st) = cena(so, [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    assert!(achado(&r, INSP_PHYS_SHAPE_ROW[0]).is_none());
    assert!(achado(&r, INSP_PHYS_ADD_PART).is_some());
    sai();
}

#[test]
fn clicar_numa_linha_seleciona_aquela_peca() {
    let (mut host, mut st) = cena(chao(), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    for (i, alvo) in [(1, PAREDE), (2, TETO)] {
        let a = clica(
            &mut host,
            &mut st,
            achado(&r, INSP_PHYS_SHAPE_ROW[i]).unwrap(),
        );
        assert!(
            a.iter().any(|a| matches!(
                a,
                EditorAction::SelectSprite { entity_bits, modifier: SelectModifier::Replace }
                    if *entity_bits == alvo
            )),
            "linha {i}: {a:?}"
        );
    }
    sai();
}

#[test]
fn add_shape_e_remove_shape_chegam_ao_barramento() {
    let (mut host, mut st) = cena(chao(), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    let a = clica(&mut host, &mut st, achado(&r, INSP_PHYS_ADD_PART).unwrap());
    assert!(
        a.iter().any(|a| matches!(
            a,
            EditorAction::InspectorPhysicsEdit {
                edit: E::AddPart,
                ..
            }
        )),
        "{a:?}"
    );
    let a = clica(
        &mut host,
        &mut st,
        achado(&r, INSP_PHYS_SHAPE_DEL[2]).unwrap(),
    );
    assert!(
        a.iter().any(|a| matches!(
            a,
            EditorAction::InspectorPhysicsEdit {
                edit: E::RemovePart(TETO),
                ..
            }
        )),
        "{a:?}"
    );
    sai();
}
