//! Gates da legenda presa ao dono (onda 4, item 8).

use super::*;
use crate::motion_objeto::Dono;

fn dono(afim: [f32; 6], visible: bool) -> Dono {
    Dono {
        entity: 1,
        visible,
        afim,
    }
}

fn ficha(world: [f32; 2], no: Option<NodeId>) -> Caption {
    Caption {
        no,
        ..Caption::new(world, "x")
    }
}

/// ⭐ **Dono em (400, 150) rodado 20° ⇒ a ficha vai para `afim × world`; dono na identidade ⇒ a de
/// hoje, bit a bit** — dois donos ao mesmo tempo, cada ficha com o seu, e a ficha sem nó intacta.
/// (Mutação: `no_mundo_de` sem a afim ⇒ a 1.ª ficha fica no espaço do grafo — RED.)
#[test]
fn a_caption_follows_the_origin_of_the_owner_of_its_node() {
    let mut m = MotionState::new();
    let (a, b) = (
        m.doc.graph.add_node("field.box"),
        m.doc.graph.add_node("field.box"),
    );
    m.doc.set_owner(a, 7);
    m.doc.set_owner(b, 8);
    let (s, c) = 20f32.to_radians().sin_cos();
    m.objeto
        .donos
        .insert(7, dono([c, s, -s, c, 400.0, 150.0], true));
    m.objeto.donos.insert(8, dono(IDENTIDADE, true));
    let w = [2.0, -1.5];
    let fichas = [
        ficha(w, Some(a)),
        ficha([-0.0, 3.0], Some(b)),
        ficha(w, None),
    ];
    let r = no_mundo_de(&fichas, &m);
    let esperado = [400.0 + c * w[0] - s * w[1], 150.0 + s * w[0] + c * w[1]];
    assert!(
        (r[0].world[0] - esperado[0]).abs() < 1e-3 && (r[0].world[1] - esperado[1]).abs() < 1e-3,
        "{:?} ≠ {esperado:?}",
        r[0].world
    );
    assert_eq!(
        r[1].world.map(f32::to_bits),
        [-0.0f32, 3.0].map(f32::to_bits)
    );
    assert_eq!(r[2], fichas[2]);
}

/// O olho do dono esconde as fichas dele com o que elas explicam.
#[test]
fn the_eye_of_the_owner_hides_its_captions() {
    let mut m = MotionState::new();
    let a = m.doc.graph.add_node("field.box");
    m.doc.set_owner(a, 7);
    m.objeto.donos.insert(7, dono(IDENTIDADE, false));
    assert!(no_mundo_de(&[ficha([0.0, 0.0], Some(a))], &m).is_empty());
}

/// ⭐ **A cena montada prende a legenda ao 1.º sink dela** — é por ele que o dono se resolve depois
/// da migração. (Mutação: `monta_cena` sem `ancora_em` ⇒ `no == None` — RED.)
#[test]
fn a_mounted_scene_anchors_its_captions_to_its_first_sink() {
    let mut m = MotionState::new();
    let (sinks, fichas) = super::super::monta("114", &mut m.doc, &m.registry);
    assert!(!fichas.is_empty(), "controlo: a cena =114 publica legenda");
    assert!(fichas.iter().all(|f| f.no == sinks.first().copied()));
    let mut nomeada = [ficha([0.0, 0.0], Some(NodeId(99)))];
    ancora_em(&mut nomeada, &sinks);
    assert_eq!(
        nomeada[0].no,
        Some(NodeId(99)),
        "a ficha que já nomeia um nó fica com ele"
    );
}
