//! As cenas de solução ÚNICA da W4.1 contra o FreeCAD 1.1.4 CORRIDO (`ferramentas/oraculo_freecad.py`, fixture com
//! cabeçalho): partindo de uma pose errada (`__partida`) ou de uma medida mudada (`__medida`), o resolvido cai onde o
//! Sketcher o põe.

use ph2d_vec_constraints::{Solucao, resolve};
use serde_json::Value;

use crate::cena::{TOL_PX, cena, xy};

const FIXTURE: &str = include_str!("../../../../docs/BlenderConstraints/fixtures/freecad_1_1_4_sketch_arrasto.json");
const TOL_FREECAD: f64 = 1e-6;

fn oraculo() -> Value {
    serde_json::from_str(FIXTURE).expect("a fixture é JSON")
}

#[test]
fn a_well_constrained_sketch_lands_where_freecad_puts_it() {
    let o = oraculo();
    let bem = o["bem"].as_object().expect("bem");
    let mut vistas = 0;
    for (nome, f) in bem.iter().filter(|(n, _)| n.starts_with("bem_")) {
        assert_eq!(f["graus_de_liberdade_livres"].as_u64(), Some(0), "{nome}: o FreeCAD conta 0 graus livres");
        let c = cena(nome);
        let partida = c.na_pose(&c.pose("partida"));
        let palpite = Solucao::da_folha(&ph2d_vec_constraints::Folha::new(
            partida.iter().map(|p| (p, ph2d_vec_scene::Xform::IDENTITY)),
        ));
        let r = resolve(&c.folha(), &c.regras, None, Some(&palpite));
        assert!(r.falta.is_empty() && r.sem_alvo.is_empty(), "{nome}: {:?} por cumprir, {:?} sem alvo", r.falta, r.sem_alvo);
        let x = c.posicoes(&r.solucao);
        let pior = crate::cena::pior(&c, &x, None);
        assert!(pior <= TOL_PX, "{nome}: viola {pior:e} px");
        for (i, (p, q)) in x.iter().zip(f["posicoes"].as_array().expect("posicoes")).enumerate() {
            let q = xy(q);
            let d = (p[0] - q[0]).hypot(p[1] - q[1]);
            assert!(d <= TOL_FREECAD, "{nome} ponto {i}: {p:?} × FreeCAD {q:?} (Δ {d:e} px)");
        }
        vistas += 1;
    }
    assert_eq!(vistas, 11, "as 11 cenas de solução única da W4.1");
}
