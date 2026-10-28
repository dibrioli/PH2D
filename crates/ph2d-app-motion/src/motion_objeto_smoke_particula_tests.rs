//! A Fonte da cena 12 pelo quadro do PRODUTO (formas publicadas, `o_quadro`, a tomada da pilha):
//! a partícula que a Bola segue nasce, anda, e morre antes do fim do laço de 4 s.

use std::collections::BTreeMap;

use super::*;
use crate::motion_bridge::quadro::{MaoDoEditor, o_quadro};
use crate::motion_objeto::{Dono, IDENTIDADE};
use crate::motion_state::MotionState;
use ph2d_ecs::World;
use ph2d_ecs::motion_generator::{Particulas, ParticulasPedidas};

const DT: f64 = 1.0 / 60.0;
const OBJETO: u64 = 9;
/// A partícula da Bola (`ph2d_app_constraints::smoke_particula::PARTICULA`).
const PARTICULA: u32 = 1;

#[test]
fn the_fountain_particle_the_ball_rides_is_born_moves_and_dies_inside_the_loop() {
    let mut m = MotionState::new();
    for n in grafo(&mut m.doc.graph) {
        m.doc.set_owner(n, OBJETO);
    }
    m.objeto.donos.insert(
        OBJETO,
        Dono {
            entity: 0,
            visible: true,
            afim: IDENTIDADE,
        },
    );
    m.gpu_enabled = false;
    let mut w = World::new();
    w.insert_resource(ParticulasPedidas([StableId(OBJETO)].into()));
    crate::motion_particulas::publica(&mut m, &mut w);
    let mut p = ph2d_core::Playhead::new(DT);
    p.play();
    let mut viva: Vec<(u64, [f64; 2])> = Vec::new();
    let mut vistas = 0;
    for t in 0..=300u64 {
        #[expect(clippy::cast_precision_loss, reason = "tiques")]
        let s = t as f64 * DT;
        p.seek(s);
        crate::motion_externals::publish_all(&mut m, s);
        o_quadro(
            &mut m,
            None,
            &p,
            DT,
            MaoDoEditor {
                selecionado: None,
                arrasto: None,
            },
        );
        crate::motion_particulas::publica(&mut m, &mut w);
        let lidas: BTreeMap<u32, [f64; 2]> = w
            .get_resource::<Particulas>()
            .and_then(|r| r.0.get(&StableId(OBJETO)).cloned())
            .unwrap_or_default();
        vistas = vistas.max(lidas.len());
        if let Some(q) = lidas.get(&PARTICULA) {
            viva.push((t, *q));
        }
    }
    assert!(vistas >= 8, "a fonte tem poucas partículas vivas: {vistas}");
    let (nasce, morre) = (viva[0].0, viva[viva.len() - 1].0);
    eprintln!(
        "partícula {PARTICULA}: tiques {nasce}..{morre}, {:?} … {:?}",
        viva[0].1,
        viva[viva.len() - 1].1
    );
    for (t, q) in viva.iter().step_by(20) {
        eprintln!("  tique {t}: {q:?}");
    }
    assert!(nasce < 30, "nasce tarde: tique {nasce}");
    assert!(
        morre < 230 && morre - nasce > 120,
        "a vida não cabe no laço: {nasce}..{morre}"
    );
    assert_eq!(
        viva.len() as u64,
        morre - nasce + 1,
        "a partícula some e volta"
    );
    let mais_alta = viva.iter().map(|(_, q)| q[1]).fold(f64::MIN, f64::max);
    assert!(mais_alta > 1.5, "a partícula não sobe: {mais_alta}");
    for (t, q) in &viva {
        assert!(
            q[0].abs() < 4.0 && q[1] > -4.0 && q[1] < 5.0,
            "tique {t}: a partícula sai da vista {q:?}"
        );
    }
}
