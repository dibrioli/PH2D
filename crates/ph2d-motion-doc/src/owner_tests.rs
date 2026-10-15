//! Os gates do dono por nó (doc ObjectsAndComponents 01 §13, F5).

use super::*;
use crate::{Members, Subgraph};
use ph2d_nodegraph::graph::Edge;

const SIM: &str = include_str!("../tests/fixtures/motion_doc_sim_antes_do_dono.txt");
const HIBRIDO: &str = include_str!("../tests/fixtures/motion_doc_hibrido_antes_do_dono.txt");

fn doc_de_dois_nos() -> (MotionDoc, NodeId, NodeId) {
    let mut doc = MotionDoc::new();
    let a = doc.graph.add_node("motion.grid");
    let b = doc.graph.add_node("motion.grid");
    (doc, a, b)
}

/// ⭐ **Zero perda:** um documento gravado ANTES do dono abre sem donos e grava-se byte a byte igual.
#[test]
fn um_documento_de_antes_do_dono_abre_igual_e_sem_donos() {
    for fixture in [SIM, HIBRIDO] {
        let doc = MotionDoc::from_text(fixture).expect("a fixtura abre");
        assert!(doc.owners.is_empty());
        assert!(!doc.graph.nodes().is_empty(), "a fixtura tem nós");
        assert_eq!(doc.unowned().len(), doc.graph.nodes().len());
        assert_eq!(doc.to_text(), fixture, "grava byte a byte o que gravava");
    }
}

#[test]
fn o_dono_atravessa_o_ficheiro() {
    let mut doc = MotionDoc::from_text(SIM).unwrap();
    assert_eq!(doc.adopt_unowned(77), doc.graph.nodes().len());
    let text = doc.to_text();
    assert!(text.ends_with(&format!("o {} 77\n", doc.owners.keys().last().unwrap().0)));
    let back = MotionDoc::from_text(&text).unwrap();
    assert_eq!(back, doc);
    assert_eq!(back.owner_ids(), BTreeSet::from([77]));
}

/// A secção `[owner]` vem depois da `[subgraph]` — as duas lêem-se.
#[test]
fn o_dono_e_o_subgrafo_convivem() {
    let (mut doc, a, b) = doc_de_dois_nos();
    doc.subgraphs.push(Subgraph {
        id: 1,
        parent: None,
        x: 0.0,
        y: 0.0,
        title: "G".into(),
    });
    doc.members = Members::from([(a, 1)]);
    doc.set_owner(a, 5);
    doc.set_owner(b, 9);
    let text = doc.to_text();
    assert!(text.find("[subgraph]").unwrap() < text.find("[owner]").unwrap());
    assert_eq!(MotionDoc::from_text(&text).unwrap(), doc);
}

#[test]
fn um_dono_corrupto_e_recusado_na_fronteira() {
    let (doc, a, _) = doc_de_dois_nos();
    let base = doc.to_text();
    for bad in [
        format!("[owner]\no {} 0\n", a.0),
        "[owner]\no 999 4\n".to_string(),
        format!("[owner]\no {} 4\no {} 5\n", a.0, a.0),
        format!("[owner]\no {} 4 lixo\n", a.0),
    ] {
        assert!(
            MotionDoc::from_text(&format!("{base}{bad}")).is_err(),
            "aceitou {bad:?}"
        );
    }
}

#[test]
fn o_dono_zero_nao_e_dono() {
    let (mut doc, a, _) = doc_de_dois_nos();
    doc.set_owner(a, 0);
    assert_eq!(doc.owner_of(a), None);
}

/// ⛔ O objecto morreu: os nós DELE saem, com as arestas; os do outro ficam.
#[test]
fn tirar_um_dono_tira_os_nos_dele_e_so_os_dele() {
    let (mut doc, a, b) = doc_de_dois_nos();
    let c = doc.graph.add_node("motion.grid");
    doc.set_owner(a, 1);
    doc.set_owner(b, 1);
    doc.set_owner(c, 2);
    assert_eq!(doc.remove_owner(1), vec![a, b]);
    assert!(doc.graph.node(a).is_none() && doc.graph.node(b).is_none());
    assert_eq!(doc.nodes_of(2), vec![c]);
    assert_eq!(doc.owner_ids(), BTreeSet::from([2]));
}

#[test]
fn apagar_um_no_esquece_o_dono_dele() {
    let (mut doc, a, _) = doc_de_dois_nos();
    doc.set_owner(a, 3);
    doc.graph.remove_node(a);
    doc.forget_nodes(&[a]);
    assert_eq!(doc.owner_of(a), None);
    assert!(MotionDoc::from_text(&doc.to_text()).is_ok());
}

#[test]
fn rehome_passa_os_nos_de_um_dono_a_outro() {
    let (mut doc, a, b) = doc_de_dois_nos();
    doc.set_owner(a, 4);
    doc.set_owner(b, 6);
    assert_eq!(doc.rehome(4, 8), 1);
    assert_eq!(doc.owner_of(a), Some(8));
    assert_eq!(doc.owner_of(b), Some(6));
}

/// ⭐ A cópia de um dono leva nós, params e fios internos; o fio que vinha de FORA continua a vir do
/// mesmo nó; o original fica intacto. (Mutação: não refazer as arestas ⇒ RED.)
#[test]
fn copiar_um_dono_da_a_copia_um_grafo_proprio() {
    let mut doc = MotionDoc::new();
    let fora = doc.graph.add_node("motion.grid");
    let a = doc.graph.add_node("motion.grid");
    let b = doc.graph.add_node("motion.output");
    doc.graph.set_param(a, "rows", 7.0);
    doc.graph
        .connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .unwrap();
    doc.set_owner(fora, 9);
    doc.set_owner(a, 1);
    doc.set_owner(b, 1);
    let map = doc.copy_owner(1, 2);
    assert_eq!(map.len(), 2);
    let (a2, b2) = (map[&a], map[&b]);
    assert_eq!(doc.nodes_of(2), vec![a2, b2]);
    assert_eq!(doc.nodes_of(1), vec![a, b], "o original fica");
    assert_eq!(
        doc.graph
            .node_param_overrides(a2)
            .and_then(|p| p.get("rows")),
        Some(&7.0)
    );
    assert!(
        doc.graph
            .edges()
            .iter()
            .any(|e| e.from.0 == a2 && e.to.0 == b2)
    );
    assert!(MotionDoc::from_text(&doc.to_text()).is_ok());
}

/// ⭐ Os nós de um dono de OUTRO documento entram com os params, os fios e os params dirigidos entre
/// eles; um fio que vinha de outro dono não viaja (o nó dele não existe aqui); o que já cá estava fica.
#[test]
fn importar_um_dono_de_outro_documento_traz_o_grafo_dele() {
    let mut src = MotionDoc::new();
    let fora = src.graph.add_node("motion.grid");
    let a = src.graph.add_node("motion.grid");
    let b = src.graph.add_node("motion.output");
    src.graph.set_param(a, "rows", 7.0);
    for (de, para) in [(a, b), (fora, a)] {
        src.graph
            .connect(Edge {
                from: (de, 0),
                to: (para, 0),
                delayed: false,
            })
            .unwrap();
    }
    src.graph.drive_param(b, "size", (a, 0)).unwrap();
    src.set_owner(fora, 9);
    src.set_owner(a, 1);
    src.set_owner(b, 1);

    let mut doc = MotionDoc::new();
    let ja = doc.graph.add_node("motion.grid");
    doc.set_owner(ja, 5);
    let map = doc.import_owner(&src, 1, 5);
    let (a2, b2) = (map[&a], map[&b]);
    assert_eq!(doc.nodes_of(5), vec![ja, a2, b2]);
    assert_eq!(
        doc.graph
            .node_param_overrides(a2)
            .and_then(|p| p.get("rows")),
        Some(&7.0)
    );
    let fios: Vec<_> = doc
        .graph
        .edges()
        .iter()
        .map(|e| (e.from.0, e.to.0))
        .collect();
    assert_eq!(fios, [(a2, b2)], "só o fio interno viaja");
    assert_eq!(
        doc.graph
            .param_sources(b2)
            .and_then(|s| s.get("size"))
            .copied(),
        Some((a2, 0))
    );
}

#[test]
fn a_moldura_tem_dono_e_sai_com_ele() {
    let (mut doc, a, _) = doc_de_dois_nos();
    doc.backdrops.push(crate::Backdrop {
        id: 4,
        x: 0.0,
        y: 0.0,
        w: 10.0,
        h: 10.0,
        color: 0,
        title: "Chuva".into(),
    });
    doc.set_owner(a, 3);
    doc.adopt_unowned(3);
    assert_eq!(doc.backdrop_owners.get(&4), Some(&3));
    let text = doc.to_text();
    assert!(text.contains("ob 4 3"));
    assert_eq!(MotionDoc::from_text(&text).unwrap(), doc);
    doc.remove_owner(3);
    assert!(doc.backdrops.is_empty());
    assert!(doc.backdrop_owners.is_empty());
}

/// ⭐ «Arrumado à mão» (`oh`) atravessa o ficheiro, por ordem crescente; sem ele, nada se escreve.
#[test]
fn arrumado_a_mao_atravessa_o_ficheiro() {
    let (mut doc, a, b) = doc_de_dois_nos();
    doc.set_owner(a, 5);
    doc.set_owner(b, 9);
    let sem = doc.to_text();
    assert!(!sem.contains("oh "));
    doc.arrumados_a_mao.insert(9);
    doc.arrumados_a_mao.insert(5);
    let text = doc.to_text();
    assert!(text.ends_with("oh 5\noh 9\n"), "{text}");
    assert_eq!(MotionDoc::from_text(&text).unwrap(), doc);
}

/// Um documento só com a marca (sem donos de nó) também a grava; e um `oh` sem nós é aceite.
#[test]
fn arrumado_a_mao_sem_donos_tambem_se_grava() {
    let (mut doc, _, _) = doc_de_dois_nos();
    doc.arrumados_a_mao.insert(4);
    let text = doc.to_text();
    assert!(text.ends_with("[owner]\noh 4\n"), "{text}");
    assert_eq!(MotionDoc::from_text(&text).unwrap(), doc);
}

#[test]
fn um_oh_corrupto_e_recusado() {
    let (doc, _, _) = doc_de_dois_nos();
    let base = doc.to_text();
    for bad in [
        "[owner]\noh 0\n",
        "[owner]\noh\n",
        "[owner]\noh x\n",
        "[owner]\noh 4 5\n",
        "[owner]\noh 4\noh 4\n",
    ] {
        assert!(
            MotionDoc::from_text(&format!("{base}{bad}")).is_err(),
            "aceitou {bad:?}"
        );
    }
}

/// A cópia de um objecto arrumado à mão é arrumada à mão; o objecto que morre leva a marca.
#[test]
fn copiar_leva_a_marca_e_tirar_a_apaga() {
    let (mut doc, a, b) = doc_de_dois_nos();
    doc.set_owner(a, 1);
    doc.set_owner(b, 3);
    doc.arrumados_a_mao.insert(1);
    doc.copy_owner(1, 2);
    doc.copy_owner(3, 4);
    assert_eq!(doc.arrumados_a_mao, BTreeSet::from([1, 2]));
    doc.remove_owner(1);
    assert_eq!(doc.arrumados_a_mao, BTreeSet::from([2]));
}
