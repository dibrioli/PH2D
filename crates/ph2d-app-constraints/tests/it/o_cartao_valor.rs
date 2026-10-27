//! ⭐ **O cartão do Follow Value** (`docs/BlenderConstraints/06_W5_resto.md` §3.1): o Follow lista as
//! propriedades do ALVO (a pose e os campos que ele publica), o Drive as do DONO, e cada peça do
//! cartão escreve o `Valor` do item.

use bevy_ecs::entity::Entity;
use ph2d_app_constraints::inspector::{apply_all, info};
use ph2d_constraints::{Canal, Kind, Mix, Mola, Prop, Valor};
use ph2d_constraints_ecs::Constraints;
use ph2d_ecs::canal::familia::OBJETO;
use ph2d_ecs::canal_constraint::{
    CANAIS, DE, DE_MAX, DE_MIN, DURACAO, EXTRAPOLA, MISTURA, MOLA_LIGADA, PARA, PARA_MAX, PARA_MIN,
    RESSALTO,
};
use ph2d_ecs::canal_objeto::OPACIDADE;
use ph2d_ecs::{CanaisLidos, FieldKey, SimWorld, StableId, Transform};
use ph2d_editor_core::constraint_edits::{ConstraintFieldEdit as E, Peca, PropriedadeDoValor as P};

/// O índice do Follow Value no menu.
const VALOR: u8 = 20;

fn opacidade() -> FieldKey {
    FieldKey::new(OBJETO, OPACIDADE, 0, 0)
}

/// O alvo (id 1) publica a opacidade; o dono (id 2) tem um Follow Value que segue o alvo.
fn mundo() -> (SimWorld, Entity, u32) {
    let mut sim = SimWorld::new();
    let alvo = sim
        .world_mut()
        .spawn((StableId(1), Transform::IDENTITY))
        .id();
    sim.world_mut()
        .get_resource_or_insert_with(CanaisLidos::default)
        .publicar(alvo.to_bits(), opacidade(), 0.5);
    let mut c = Constraints::default();
    let id = c.push(Kind::Valor(Valor::default()), Some(StableId(1)));
    let dono = sim
        .world_mut()
        .spawn((StableId(2), Transform::IDENTITY, c))
        .id();
    (sim, dono, id)
}

fn valor(sim: &SimWorld, dono: Entity, id: u32) -> Valor {
    match sim
        .world()
        .get::<Constraints>(dono)
        .unwrap()
        .item(id)
        .unwrap()
        .kind
    {
        Kind::Valor(v) => v,
        k => panic!("o item virou {k:?}"),
    }
}

/// O Follow lista as cinco da pose e o campo do alvo, com os nomes da porta única (o canal do
/// Transformation; o grupo do campo, como o dope-sheet). O Drive é sobre o dono: só a pose.
#[test]
fn the_value_card_lists_the_targets_properties() {
    let (sim, dono, _) = mundo();
    let i = info(&sim, dono.to_bits(), 1).unwrap();
    let row = &i.rows[0];
    assert_eq!(row.tipo, VALOR);
    let [follow, drive] = row.propriedades.as_slice() else {
        panic!(
            "o Follow Value tem de ter Follow e Drive: {:?}",
            row.propriedades
        );
    };
    assert_eq!((follow.campo, drive.campo), (DE, PARA));
    let nomes: Vec<&str> = follow.opcoes.iter().map(|o| o.rotulo.as_str()).collect();
    let mut esperados: Vec<&str> = CANAIS.iter().map(|k| ph2d_i18n::tr(k)).collect();
    esperados.push(ph2d_i18n::tr(opacidade().info().unwrap().rotulo));
    assert_eq!(nomes, esperados);
    let k = opacidade();
    assert_eq!(
        follow.opcoes[5].prop,
        P::Campo([k.familia, k.campo, k.i, k.j])
    );
    assert_eq!(
        drive.opcoes.len(),
        CANAIS.len(),
        "o dono não publica campos"
    );
    // De fábrica: Y do alvo → rotação do dono.
    assert_eq!(follow.valor, P::Canal(1));
    assert_eq!(drive.valor, P::Canal(2));
    assert_eq!(drive.atual, ph2d_i18n::tr(CANAIS[2]));
    // A ordem do cartão: Follow, faixa, Drive, faixa, Mix, Extrapolate, Spring, Duration, Bounce.
    let campos: Vec<u16> = row
        .pecas()
        .into_iter()
        .map(|p| match p {
            Peca::Numero(j) => row.numeros[usize::from(j)].campo,
            Peca::Escolha(j) => row.escolhas[usize::from(j)].campo,
            Peca::Propriedade(j) => row.propriedades[usize::from(j)].campo,
        })
        .collect();
    assert_eq!(
        campos,
        [
            DE,
            DE_MIN,
            DE_MAX,
            PARA,
            PARA_MIN,
            PARA_MAX,
            MISTURA,
            EXTRAPOLA,
            MOLA_LIGADA,
            DURACAO,
            RESSALTO
        ]
    );
    // A rotação do dono lê-se em graus no cartão.
    let max = row.numeros.iter().find(|n| n.campo == PARA_MAX).unwrap();
    assert!((max.valor - 90.0).abs() < 1e-3, "{}", max.valor);
}

/// Escolher a fonte e o destino, a faixa e a mola escreve o `Valor`; cada pedido que muda é UM passo
/// de desfazer (`true`), e repetir o mesmo não grava passo vazio (`false`). Sem mola, Duration e
/// Bounce saem do cartão.
#[test]
fn editing_the_value_card_writes_the_item() {
    let (mut sim, dono, id) = mundo();
    let b = dono.to_bits();
    let k = opacidade();
    let campo = P::Campo([k.familia, k.campo, k.i, k.j]);
    assert!(apply_all(&mut sim, &[(b, E::Propriedade(id, DE, campo))]));
    assert!(!apply_all(&mut sim, &[(b, E::Propriedade(id, DE, campo))]));
    assert!(apply_all(
        &mut sim,
        &[(b, E::Propriedade(id, PARA, P::Canal(0)))]
    ));
    assert!(apply_all(
        &mut sim,
        &[
            (b, E::Campo(id, DE_MIN, 0.2)),
            (b, E::Campo(id, DE_MAX, 0.8)),
            (b, E::Campo(id, PARA_MIN, -1.0)),
            (b, E::Campo(id, PARA_MAX, 2.0)),
            (b, E::Campo(id, MISTURA, 1.0)),
            (b, E::Campo(id, EXTRAPOLA, 1.0)),
        ]
    ));
    assert!(apply_all(&mut sim, &[(b, E::Campo(id, MOLA_LIGADA, 0.0))]));
    assert_eq!(valor(&sim, dono, id).mola, None);
    let row = &info(&sim, b, 1).unwrap().rows[0];
    assert!(
        row.numeros
            .iter()
            .all(|n| n.campo != DURACAO && n.campo != RESSALTO)
    );
    assert!(!apply_all(&mut sim, &[(b, E::Campo(id, DURACAO, 2.0))]));
    assert!(apply_all(&mut sim, &[(b, E::Campo(id, MOLA_LIGADA, 1.0))]));
    assert!(apply_all(&mut sim, &[(b, E::Campo(id, DURACAO, 1.5))]));
    assert!(apply_all(&mut sim, &[(b, E::Campo(id, RESSALTO, 0.25))]));
    let v = valor(&sim, dono, id);
    assert_eq!(v.fonte, Prop::Campo([k.familia, k.campo, k.i, k.j]));
    assert_eq!(v.destino, Prop::Pose(Canal::X));
    let perto = |a: f64, b: f64| (a - b).abs() < 1e-6;
    assert!(
        perto(v.de_faixa[0], 0.2) && perto(v.de_faixa[1], 0.8),
        "{:?}",
        v.de_faixa
    );
    assert!(
        perto(v.para_faixa[0], -1.0) && perto(v.para_faixa[1], 2.0),
        "{:?}",
        v.para_faixa
    );
    assert_eq!((v.mix, v.extrapola), (Mix::Add, true));
    assert_eq!(
        v.mola,
        Some(Mola {
            duracao: 1.5,
            ressalto: 0.25
        })
    );
    // O Follow mostra a escolha pelo nome do campo.
    let row = &info(&sim, b, 1).unwrap().rows[0];
    assert_eq!(row.propriedades[0].valor, campo);
    assert_eq!(
        row.propriedades[0].atual,
        ph2d_i18n::tr(k.info().unwrap().rotulo)
    );
}
