use super::*;
use crate::canais_campos::{campos, ler};
use ph2d_vec_scene::{VecPath, VecVertex};

const E: u64 = 9;

fn quadrado(lado: f64) -> VecPath {
    VecPath {
        verts: [[0.0, 0.0], [lado, 0.0], [lado, lado], [0.0, lado]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
        closed: true,
        ..VecPath::default()
    }
}

fn leitura(p: &VecPath) -> Vec<(FieldKey, f32)> {
    campos(p, None)
        .into_iter()
        .filter(|kk| (c::ANCORA_X..=c::FECHADO).contains(&kk.campo))
        .map(|kk| (kk, ler(p, None, kk).unwrap()))
        .collect()
}

/// A forma animada: um quadrado de lado 10 em t=0 e de lado 20 em t=1.
fn animado() -> TimelineDoc {
    let mut doc = TimelineDoc::new();
    for (t, lado) in [(0.0, 10.0), (1.0, 20.0)] {
        for (kk, v) in leitura(&quadrado(lado)) {
            doc.upsert_key(
                E,
                PropKind::Field(kk),
                RationalTime::from_seconds(t),
                AnimValue::Float(v),
                Interp::Linear,
            );
        }
    }
    doc
}

fn na_chave(doc: &TimelineDoc, t: f64, j: u16) -> [f32; 2] {
    let v = |campo| {
        let b = doc
            .binding_for(E, PropKind::Field(k(campo, 0, j)))
            .expect("trilha");
        match doc.active_clip().track(b.target).unwrap().sample(t) {
            AnimValue::Float(x) => x,
            o => panic!("{o:?}"),
        }
    };
    [v(c::ANCORA_X), v(c::ANCORA_Y)]
}

fn n_vertices(doc: &TimelineDoc) -> usize {
    doc.bindings()
        .iter()
        .filter(|b| matches!(b.prop, PropKind::Field(kk) if kk.campo == c::ANCORA_X))
        .count()
}

/// ⭐ Inserir um ponto no meio do lado de baixo, a meio da animação (lado 15): em CADA chave o
/// ponto nasce no meio do lado de baixo DESSA chave — a forma de cada chave não muda de desenho.
#[test]
fn inserir_um_ponto_numa_forma_animada_insere_em_todas_as_chaves() {
    let mut doc = animado();
    let ontem = quadrado(15.0);
    let mut hoje = ontem.clone();
    hoje.verts.insert(1, VecVertex::corner([7.5, 0.0]));
    let antes = leitura(&ontem).into_iter().collect();
    let rel = propagar(&mut doc, E, &antes, &leitura(&hoje)).expect("propagou");
    assert_eq!((rel.chaves, rel.novos, rel.apagados), (2, 1, 0));
    assert_eq!(n_vertices(&doc), 5);
    assert_eq!(
        na_chave(&doc, 0.0, 1),
        [5.0, 0.0],
        "o meio do lado na chave de lado 10"
    );
    assert_eq!(
        na_chave(&doc, 1.0, 1),
        [10.0, 0.0],
        "o meio do lado na chave de lado 20"
    );
    assert_eq!(
        na_chave(&doc, 1.0, 2),
        [20.0, 0.0],
        "o vizinho seguinte não mudou de lugar"
    );
    assert_eq!(na_chave(&doc, 0.0, 4), [0.0, 10.0], "o último é o antigo 3");
}

/// Apagar um ponto sai de TODAS as chaves, e os seguintes continuam a ser os mesmos pontos.
#[test]
fn apagar_um_ponto_numa_forma_animada_apaga_em_todas_as_chaves() {
    let mut doc = animado();
    let ontem = quadrado(15.0);
    let mut hoje = ontem.clone();
    hoje.verts.remove(2);
    let antes = leitura(&ontem).into_iter().collect();
    let rel = propagar(&mut doc, E, &antes, &leitura(&hoje)).expect("propagou");
    assert_eq!((rel.novos, rel.apagados), (0, 1));
    assert_eq!(n_vertices(&doc), 3);
    assert_eq!(
        na_chave(&doc, 1.0, 2),
        [0.0, 20.0],
        "o índice 2 é agora o antigo 3"
    );
    assert_eq!(na_chave(&doc, 0.0, 1), [10.0, 0.0]);
}

/// Sem animação na forma, ou sem mudar o nº de pontos, não há nada a propagar.
#[test]
fn sem_animacao_ou_sem_mudar_a_contagem_nada_acontece() {
    let ontem = quadrado(15.0);
    let antes = leitura(&ontem).into_iter().collect();
    let mut hoje = ontem.clone();
    hoje.verts.insert(1, VecVertex::corner([7.5, 0.0]));
    assert!(propagar(&mut TimelineDoc::new(), E, &antes, &leitura(&hoje)).is_none());
    let mut doc = animado();
    let mut mexido = ontem.clone();
    mexido.verts[0].anchor = [1.0, 1.0];
    assert!(propagar(&mut doc, E, &antes, &leitura(&mexido)).is_none());
}

/// Um segmento CURVO: o ponto novo divide-o em `u` e as alças dos vizinhos encurtam (de Casteljau).
#[test]
fn num_segmento_curvo_as_alcas_dos_vizinhos_acompanham() {
    let mut a = [0.0f32, 0.0, 0.0, 0.0, 0.0, 6.0, 0.0, 1.0];
    let mut b = [10.0f32, 0.0, 10.0, 6.0, 10.0, 0.0, 0.0, 1.0];
    let novo = dividir(&mut a, &mut b, 0.5, &[0.0; 8]);
    assert_eq!([novo[0], novo[1]], [5.0, 4.5], "o ponto da cúbica em u=0,5");
    assert_eq!(
        [a[4], a[5]],
        [0.0, 3.0],
        "a alça de saída do vizinho encolhe a metade"
    );
    assert_eq!([b[2], b[3]], [10.0, 3.0]);
}
