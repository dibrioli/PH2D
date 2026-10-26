//! ⭐ **Cada operação que mexe em vértices guarda o id de quem sobrevive e dá id novo a quem nasce**
//! (W4.2, `docs/BlenderConstraints/04_W4_cad.md` §2). Um caso por sítio desta crate; os do `vec-edit` e
//! da booleana vivem lá (`each_vertex_mutation_site_keeps_or_assigns_ids_*`).

use ph2d_vec_scene::{
    FlipAxis, Rotate90, VecPath, VecPathId, VecScene, VecVertex, VecXforms, VertId,
    dissolve_vertex, split_segment,
};

fn cena(n: usize, closed: bool) -> (VecScene, VecPathId) {
    let mut c = VecScene::new();
    let verts = (0..n)
        .map(|i| {
            let a = i as f64 * std::f64::consts::TAU / n as f64;
            VecVertex::corner([10.0 * a.cos(), 10.0 * a.sin() + i as f64 * 0.37])
        })
        .collect();
    let id = c.push_path(VecPath {
        verts,
        closed,
        ..VecPath::default()
    });
    (c, id)
}

fn ids(p: &VecPath) -> Vec<u32> {
    p.verts.iter().map(|v| v.id.0).collect()
}

fn sem_repetidos(p: &VecPath) {
    let mut v = ids(p);
    v.sort_unstable();
    let n = v.len();
    v.dedup();
    assert_eq!(v.len(), n, "ids repetidos: {:?}", ids(p));
    assert!(!v.contains(&0), "vértice sem id");
}

#[test]
fn each_vertex_mutation_site_keeps_or_assigns_ids() {
    // Inserir no segmento: os outros ficam, o novo ganha o seguinte.
    let (mut c, id) = cena(6, true);
    split_segment(c.path_mut(id).unwrap(), 1, 0.5).unwrap();
    c.normaliza_ids();
    assert_eq!(ids(c.path(id).unwrap()), [1, 2, 7, 3, 4, 5, 6]);

    // Apagar (dissolver): o id some, a ordem dos outros fica.
    let (mut c, id) = cena(6, true);
    assert!(dissolve_vertex(&mut c.path_mut(id).unwrap().verts, 2, true));
    c.normaliza_ids();
    assert_eq!(ids(c.path(id).unwrap()), [1, 2, 4, 5, 6]);

    // Inverter: os mesmos ids na ordem inversa.
    let (mut c, id) = cena(5, false);
    assert!(c.reverse_path(id));
    assert_eq!(ids(c.path(id).unwrap()), [5, 4, 3, 2, 1]);

    // Espelhar, rodar, suavizar: nada muda de id.
    let (mut c, id) = cena(5, true);
    assert!(c.flip_path(id, FlipAxis::Horizontal));
    assert!(c.rotate_path(id, Rotate90::Cw));
    assert!(c.smooth_path(id));
    assert_eq!(ids(c.path(id).unwrap()), [1, 2, 3, 4, 5]);

    // Simplificar: quem fica, fica com o seu id e pela ordem.
    let (mut c, id) = cena(12, true);
    assert!(c.simplify_path(id));
    let ficou = ids(c.path(id).unwrap());
    assert!(
        ficou.len() < 12 && ficou.windows(2).all(|w| w[0] < w[1]),
        "{ficou:?}"
    );

    // Partir um contorno FECHADO: abre, e a cópia da costura ganha id novo no assentamento.
    let (mut c, id) = cena(4, true);
    assert!(c.cut_path_at_vertex(id, 2).unwrap().opened);
    c.normaliza_ids();
    let p = c.path(id).unwrap();
    assert_eq!(
        ids(p),
        [3, 4, 1, 2, 5],
        "a costura duplicada é a 2.ª ocorrência"
    );
    sem_repetidos(p);

    // Partir um contorno ABERTO: a 2.ª metade é um path NOVO que leva os ids dela.
    let (mut c, id) = cena(5, false);
    let nova = c.cut_path_at_vertex(id, 2).unwrap().new_path.unwrap();
    c.normaliza_ids();
    assert_eq!(ids(c.path(id).unwrap()), [1, 2, 3]);
    assert_eq!(ids(c.path(nova).unwrap()), [3, 4, 5]);

    // Juntar B a A: os de A ficam; os de B que colidem ganham ids novos.
    let (mut c, a) = cena(3, false);
    let b = c.push_path(VecPath {
        verts: [[30.0, 0.0], [40.0, 0.0], [50.0, 5.0]]
            .map(VecVertex::corner)
            .to_vec(),
        ..VecPath::default()
    });
    let juntos = c.join_paths(a, b, &VecXforms::new(), 1e-6).expect("junta");
    c.normaliza_ids();
    let p = c.path(juntos).unwrap();
    sem_repetidos(p);
    let de_a: Vec<u32> = ids(p).into_iter().filter(|i| *i <= 3).collect();
    assert_eq!(de_a.len(), 3, "os três de A sobrevivem: {:?}", ids(p));

    // Duplicar: o clone é OUTRO path, com o seu espaço de ids (os mesmos números).
    let (mut c, id) = cena(4, true);
    let dup = c.duplicate_path(id, 1.0, 1.0).unwrap();
    assert_eq!(ids(c.path(dup).unwrap()), ids(c.path(id).unwrap()));
}

/// ⭐ Uma forma RE-GERADA com a mesma topologia (forma viva, envelope, conector) volta com os mesmos ids
/// — senão cada quadro com entrada seria um passo de undo espúrio —, e uma inserção não rouba o id de
/// quem foi deslocado.
#[test]
fn a_regenerated_shape_inherits_its_ids_and_an_insert_does_not_steal_one() {
    let (mut c, id) = cena(5, true);
    let antes = c.clone();
    let novos: Vec<VecVertex> = c
        .path(id)
        .unwrap()
        .verts
        .iter()
        .map(|v| VecVertex::corner([v.anchor[0] * 2.0, v.anchor[1]]))
        .collect();
    c.path_mut(id).unwrap().replace_cooked(VecPath {
        verts: novos,
        closed: true,
        ..VecPath::default()
    });
    assert!(
        c.path(id)
            .unwrap()
            .verts
            .iter()
            .all(|v| v.id == VertId::NOVO)
    );
    c.normaliza_ids_desde(Some(&antes));
    assert_eq!(ids(c.path(id).unwrap()), [1, 2, 3, 4, 5]);
    assert_eq!(c.path(id).unwrap().next_vert, 6, "nada consumido");

    let antes = c.clone();
    c.path_mut(id)
        .unwrap()
        .verts
        .insert(0, VecVertex::corner([-50.0, 0.0]));
    c.normaliza_ids_desde(Some(&antes));
    assert_eq!(ids(c.path(id).unwrap()), [6, 1, 2, 3, 4, 5]);

    // ⛔ CONTROLO: sem a fotografia anterior, a forma re-gerada ganharia ids NOVOS.
    let (mut c, id) = cena(3, true);
    let v: Vec<VecVertex> = c
        .path(id)
        .unwrap()
        .verts
        .iter()
        .map(|v| VecVertex::corner(v.anchor))
        .collect();
    c.path_mut(id).unwrap().verts = v;
    c.normaliza_ids_desde(None);
    assert_eq!(ids(c.path(id).unwrap()), [4, 5, 6]);
}
