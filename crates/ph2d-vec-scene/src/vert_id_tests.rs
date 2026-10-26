use super::*;
use crate::{Rgba8, VecScene, with_wire_layout};

fn dez() -> Vec<VecVertex> {
    (0..10)
        .map(|i| VecVertex::corner([f64::from(i), f64::from(i * i)]))
        .collect()
}

/// Os espelhos `VecVertexV24`/`VecPathV24` e o `Serialize` derivado andam na MESMA ordem — todos os
/// valores distintos, para que dois campos do mesmo tipo trocados se vejam.
#[test]
fn the_live_layout_round_trips() {
    let mut v = VecVertex::smooth([1.0, 2.0], [3.0, 4.0], [5.0, 6.0]);
    v.corner_radius = -0.5;
    v.id = VertId(41);
    let p = VecPath {
        id: 7,
        verts: vec![v, VecVertex::corner([9.0, 8.0])],
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(1, 2, 3, 4))),
        stroke: Some(StrokeSpec::new(Rgba8::new(5, 6, 7, 8), 0.25)),
        subpaths: vec![Contour::new_closed(vec![VecVertex::corner([-1.0, -2.0])])],
        fill_rule: FillRule::EvenOdd,
        next_vert: 99,
        ..VecPath::default()
    };
    let bytes = postcard::to_allocvec(&p).unwrap();
    let back: VecPath = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(back, p);
}

#[test]
fn normaliza_numbers_in_order_and_is_idempotent() {
    let mut p = VecPath {
        verts: dez(),
        subpaths: vec![Contour::new_closed(vec![VecVertex::corner([0.5, 0.5]); 2])],
        ..VecPath::default()
    };
    assert!(p.normaliza_ids());
    let ids: Vec<u32> = p
        .verts
        .iter()
        .chain(&p.subpaths[0].verts)
        .map(|v| v.id.0)
        .collect();
    assert_eq!(ids, (1..=12).collect::<Vec<_>>());
    assert_eq!(p.next_vert, 13);
    let antes = p.clone();
    assert!(!p.normaliza_ids(), "a 2.ª passagem não muda nada");
    assert_eq!(p, antes);
}

/// Um vértice DUPLICADO por uma operação (mesmo id duas vezes): o 1.º fica, o 2.º ganha id novo — e
/// um id vindo de fora acima do contador sobe-o, para o contador nunca o reusar.
#[test]
fn a_repeated_id_keeps_the_first_and_a_foreign_id_raises_the_counter() {
    let mut p = VecPath {
        verts: dez(),
        ..VecPath::default()
    };
    p.normaliza_ids();
    p.verts.insert(4, p.verts[3]);
    p.verts[9].id = VertId(50);
    p.normaliza_ids();
    assert_eq!(p.verts[3].id, VertId(4), "o 1.º fica");
    assert_eq!(
        p.verts[4].id,
        VertId(51),
        "o duplicado ganha o seguinte ao 50"
    );
    assert_eq!(p.verts[9].id, VertId(50));
    assert_eq!(p.next_vert, 52);
}

/// ⭐ Apagar o 3.º de 10: os outros guardam o id, e o id do 5.º continua a achar o MESMO ponto (pelo
/// índice, o 5.º passa a ser o antigo 6.º).
#[test]
fn a_vertex_keeps_its_id_when_a_neighbour_is_deleted() {
    let mut cena = VecScene::new();
    let pid = cena.push_path(VecPath {
        verts: dez(),
        ..VecPath::default()
    });
    let p = cena.path_mut(pid).unwrap();
    let ids: Vec<VertId> = p.verts.iter().map(|v| v.id).collect();
    let quinto = (ids[4], p.verts[4].anchor);
    p.verts.remove(2);
    cena.normaliza_ids();
    let p = cena.path(pid).unwrap();
    let depois: Vec<VertId> = p.verts.iter().map(|v| v.id).collect();
    let mut esperado = ids.clone();
    esperado.remove(2);
    assert_eq!(depois, esperado);
    assert_eq!(p.vert_by_id(quinto.0).unwrap().anchor, quinto.1);
    assert_ne!(
        p.verts[4].anchor, quinto.1,
        "⛔ CONTROLO: o índice 4 já é outro ponto"
    );
    assert!(p.vert_by_id(ids[2]).is_none(), "o apagado sumiu");
    assert!(p.vert_by_id(VertId::NOVO).is_none());
}

/// Um id apagado NÃO volta: inserir depois de apagar o último dá um id novo, nunca o antigo.
#[test]
fn a_deleted_id_is_never_reused() {
    let mut p = VecPath {
        verts: dez(),
        ..VecPath::default()
    };
    p.normaliza_ids();
    let ultimo = p.verts.pop().unwrap().id;
    p.verts.push(VecVertex::corner([0.0, 0.0]));
    p.normaliza_ids();
    assert_ne!(p.verts[9].id, ultimo);
    assert_eq!(p.verts[9].id, VertId(11));
}

/// O âmbito v23 lê o vértice sem id e o path sem contador — e fecha.
#[test]
fn the_v23_scope_reads_the_geometry_without_ids_and_closes() {
    #[derive(serde::Serialize)]
    struct Antigo {
        anchor: [f64; 2],
        in_handle: [f64; 2],
        out_handle: [f64; 2],
        kind: VertexKind,
        corner_radius: f64,
    }
    let bytes = postcard::to_allocvec(&Antigo {
        anchor: [1.0, 2.0],
        in_handle: [3.0, 4.0],
        out_handle: [5.0, 6.0],
        kind: VertexKind::Symmetric,
        corner_radius: 1.5,
    })
    .unwrap();
    let v: VecVertex = with_wire_layout(23, || postcard::from_bytes(&bytes)).unwrap();
    assert_eq!(v.id, VertId::NOVO);
    assert_eq!((v.kind, v.corner_radius), (VertexKind::Symmetric, 1.5));
    assert!(!reading_geometry_v23(), "o âmbito fechou");
}
