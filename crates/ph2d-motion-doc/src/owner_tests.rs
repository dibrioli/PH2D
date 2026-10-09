//! Os gates do dono por nó (doc ObjectsAndComponents 01 §13, F5).

use super::*;
use crate::{Members, Subgraph};

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
