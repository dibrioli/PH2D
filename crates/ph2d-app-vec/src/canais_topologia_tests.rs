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
    na_chave_i(doc, t, 0, j)
}

fn na_chave_i(doc: &TimelineDoc, t: f64, i: u16, j: u16) -> [f32; 2] {
    let v = |campo| {
        let b = doc
            .binding_for(E, PropKind::Field(k(campo, i, j)))
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

/// Duas formas num caminho: o quadrado `a` na origem e o quadrado `b` deslocado de 100 em x.
fn dois(a: f64, b: f64) -> VecPath {
    let mut p = quadrado(a);
    let sub = quadrado(b);
    p.subpaths.push(ph2d_vec_scene::Contour {
        verts: sub
            .verts
            .iter()
            .map(|v| VecVertex::corner([v.anchor[0] + 100.0, v.anchor[1]]))
            .collect(),
        closed: true,
    });
    p
}

/// O caminho `dois` animado: `(10, 50)` em t=0 e `(20, 100)` em t=1.
fn dois_animado() -> TimelineDoc {
    let mut doc = TimelineDoc::new();
    for (t, (a, b)) in [(0.0, (10.0, 50.0)), (1.0, (20.0, 100.0))] {
        for (kk, v) in leitura(&dois(a, b)) {
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

fn n_trilhas_do_contorno(doc: &TimelineDoc, i: u16) -> usize {
    doc.bindings()
        .iter()
        .filter(|b| matches!(b.prop, PropKind::Field(kk) if kk.familia == VETOR && kk.i == i))
        .count()
}

/// ⭐ Apagar o contorno 0 (lei do dono 20/10): em CADA chave o novo `i=0` tem os valores do antigo
/// `i=1`, e não sobra trilha de `i=1`.
#[test]
fn apagar_o_contorno_0_leva_as_chaves_do_1() {
    let mut doc = dois_animado();
    let ontem = dois(15.0, 75.0);
    let mut hoje = ontem.clone();
    assert!(hoje.remove_contour(0));
    let antes: BTreeMap<FieldKey, f32> = leitura(&ontem).into_iter().collect();
    assert!(mudou(&antes, &leitura(&hoje)), "a pergunta que abre o undo");
    propagar(&mut doc, E, &antes, &leitura(&hoje)).expect("propagou");
    assert_eq!(
        na_chave_i(&doc, 0.0, 0, 1),
        [150.0, 0.0],
        "o antigo 1 na chave de lado 50"
    );
    assert_eq!(
        na_chave_i(&doc, 1.0, 0, 2),
        [200.0, 100.0],
        "o antigo 1 na chave de lado 100"
    );
    assert_eq!(
        n_trilhas_do_contorno(&doc, 1),
        0,
        "as trilhas do i que deixou de existir saem"
    );
    assert_eq!(n_vertices(&doc), 4);
}

/// Inserir um contorno no índice 0: os antigos passam a 1 e 2 com os seus valores; o novo entra com
/// o valor de hoje em todas as chaves.
#[test]
fn inserir_um_contorno_nao_mexe_nos_outros() {
    let mut doc = dois_animado();
    let ontem = dois(15.0, 75.0);
    let mut hoje = ontem.clone();
    let novo = [[-50.0, -50.0], [-40.0, -50.0], [-45.0, -40.0]];
    let velho = ph2d_vec_scene::Contour {
        verts: std::mem::replace(
            &mut hoje.verts,
            novo.into_iter().map(VecVertex::corner).collect(),
        ),
        closed: true,
    };
    hoje.subpaths.insert(0, velho);
    let antes: BTreeMap<FieldKey, f32> = leitura(&ontem).into_iter().collect();
    assert!(mudou(&antes, &leitura(&hoje)));
    let rel = propagar(&mut doc, E, &antes, &leitura(&hoje)).expect("propagou");
    assert_eq!((rel.novos, rel.apagados), (1, 0), "um contorno novo");
    assert_eq!(
        na_chave_i(&doc, 0.0, 1, 2),
        [10.0, 10.0],
        "o antigo 0 na chave de lado 10"
    );
    assert_eq!(na_chave_i(&doc, 1.0, 1, 2), [20.0, 20.0]);
    assert_eq!(na_chave_i(&doc, 0.0, 2, 1), [150.0, 0.0], "o antigo 1");
    assert_eq!(na_chave_i(&doc, 1.0, 2, 1), [200.0, 0.0]);
    for t in [0.0, 1.0] {
        assert_eq!(
            na_chave_i(&doc, t, 0, 1),
            [-40.0, -50.0],
            "o novo vale o de hoje"
        );
    }
    assert_eq!(n_vertices(&doc), 3 + 4 + 4);
}

/// Trocar a ordem dos dois contornos (mesmo nº) leva as chaves de cada um para o seu novo `i`.
#[test]
fn reordenar_contornos_leva_as_chaves() {
    let mut doc = dois_animado();
    let ontem = dois(15.0, 75.0);
    let mut hoje = ontem.clone();
    std::mem::swap(&mut hoje.verts, &mut hoje.subpaths[0].verts);
    let antes: BTreeMap<FieldKey, f32> = leitura(&ontem).into_iter().collect();
    assert!(mudou(&antes, &leitura(&hoje)));
    propagar(&mut doc, E, &antes, &leitura(&hoje)).expect("propagou");
    assert_eq!(
        na_chave_i(&doc, 0.0, 0, 1),
        [150.0, 0.0],
        "o antigo 1 é agora o 0"
    );
    assert_eq!(
        na_chave_i(&doc, 1.0, 1, 1),
        [20.0, 0.0],
        "o antigo 0 é agora o 1"
    );
    assert_eq!(n_vertices(&doc), 8);
}

/// Mexer num ponto de um contorno sem mudar o nº de nada não é topologia.
#[test]
fn mexer_um_ponto_de_uma_forma_de_dois_contornos_nao_e_topologia() {
    let ontem = dois(15.0, 75.0);
    let mut hoje = ontem.clone();
    hoje.subpaths[0].verts[0].anchor = [101.0, 1.0];
    let antes: BTreeMap<FieldKey, f32> = leitura(&ontem).into_iter().collect();
    assert!(!mudou(&antes, &leitura(&hoje)));
    assert!(propagar(&mut dois_animado(), E, &antes, &leitura(&hoje)).is_none());
}
