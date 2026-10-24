use super::*;
use ph2d_asset::ComponentBlob;

fn linha(id: u64, comps: &[(u64, &[u8])], pai: Option<u64>) -> Arc<EntitySnapshotRow> {
    Arc::new(EntitySnapshotRow {
        id: StableId(id),
        components: comps
            .iter()
            .map(|(t, d)| ComponentBlob {
                type_id: *t,
                data: d.to_vec(),
            })
            .collect(),
        parent: pai.map(StableId),
    })
}

fn mundo(linhas: Vec<Arc<EntitySnapshotRow>>) -> WorldSnapshot {
    WorldSnapshot {
        version: WorldSnapshot::VERSION,
        entities: linhas,
    }
}

#[test]
fn linhas_que_mudam_da_so_o_que_mudou() {
    let partilhadas: Vec<_> = (1..=1_000).map(|i| linha(i, &[(7, &[1])], None)).collect();
    let antes = mundo(partilhadas.clone());
    let mut depois = antes.clone();
    depois.set_component_blob(StableId(500), "x", Some(vec![9]));
    let posto = super::super::stable_type_id("x");
    let d = linhas_que_mudam(&antes, &depois);
    assert_eq!(
        d,
        vec![LinhaMudada::Muda {
            id: StableId(500),
            pai: None,
            componentes: vec![(posto, Some(vec![9]))],
        }]
    );
}

#[test]
fn nasce_morre_pai_e_tirado() {
    let antes = mundo(vec![
        linha(1, &[(1, &[1]), (2, &[2])], None),
        linha(2, &[(1, &[1])], None),
    ]);
    let depois = mundo(vec![
        linha(1, &[(1, &[1])], Some(3)),
        linha(3, &[(1, &[5])], None),
    ]);
    let d = linhas_que_mudam(&antes, &depois);
    assert_eq!(d.len(), 3);
    assert_eq!(
        d[0],
        LinhaMudada::Muda {
            id: StableId(1),
            pai: Some(Some(StableId(3))),
            componentes: vec![(2, None)],
        }
    );
    assert_eq!(d[1], LinhaMudada::Morre(StableId(2)));
    assert!(matches!(&d[2], LinhaMudada::Nasce(r) if r.id == StableId(3)));
}

#[test]
fn iguais_nao_mudam() {
    let a = mundo(vec![linha(1, &[(1, &[1])], None)]);
    let b = mundo(vec![linha(1, &[(1, &[1])], None)]);
    assert!(linhas_que_mudam(&a, &b).is_empty());
}
