//! ⭐ **A cena 7 — a caixa com tampa** (W4.6, `docs/BlenderConstraints/04_W4_cad.md` §3.7): dez regras, o ângulo da
//! tampa com três chaves. Em todo quadro nenhuma regra se desfaz, o corpo (todo definido) não se mexe, a dobradiça
//! fica no canto e o ângulo medido nos cantos é o da chave; tocar e arrastar o tempo dão o mesmo bit.

use ph2d_app_constraints::chaves;
use ph2d_app_constraints::smoke_caixa::{self, ANGULO, Caixa};
use ph2d_ecs::canal::familia::CAD;
use ph2d_ecs::{CanaisEscritos, SimWorld, VecPathRef};
use ph2d_timeline::{TimelineState, apply_from_doc};
use ph2d_vec_constraints::{Folha, violacao};
use ph2d_vec_entities::medida::{self, ConducaoCad};
use ph2d_vec_scene::{VecPath, VecScene, VertId, Xform};

const TOL: f64 = 1e-6;

struct Cena {
    sim: SimWorld,
    scene: VecScene,
    cond: ConducaoCad,
    tl: TimelineState,
    caixa: Caixa,
}

fn cena() -> Cena {
    let mut scene = VecScene::new();
    let caixa = smoke_caixa::semear(&mut scene);
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn(VecPathRef(caixa.corpo))
        .id()
        .to_bits();
    sim.world_mut().spawn(VecPathRef(caixa.tampa));
    let mut tl = TimelineState::default();
    let fps = tl.doc.fps_display;
    let lote = smoke_caixa::chaves_do_angulo(e, &caixa.lei, caixa.angulo, fps);
    assert_eq!(lote.len(), ANGULO.len(), "a medida do ângulo tem canal");
    assert!(chaves::gravar(&mut tl, &lote, false, 0.0, true));
    Cena {
        sim,
        scene,
        cond: ConducaoCad::default(),
        tl,
        caixa,
    }
}

impl Cena {
    fn path(&self, id: u64) -> &VecPath {
        self.scene.path(id).expect("o path")
    }
    fn p(&self, id: u64, i: u32) -> [f64; 2] {
        self.path(id).vert_by_id(VertId(i)).unwrap().anchor
    }
    fn em(&mut self, q: u32) -> Vec<u64> {
        let t = f64::from(q) / self.tl.doc.fps_display;
        apply_from_doc(self.sim.world_mut(), &mut self.tl.doc, t);
        let escritas = self
            .sim
            .world_mut()
            .get_resource_mut::<CanaisEscritos>()
            .map(|mut w| w.drenar(CAD))
            .unwrap_or_default();
        medida::aplicar(&mut self.sim, &mut self.scene, escritas, &mut self.cond);
        self.cond.settle();
        [self.caixa.corpo, self.caixa.tampa]
            .iter()
            .flat_map(|&id| {
                self.path(id)
                    .verts
                    .iter()
                    .flat_map(|v| v.anchor.map(f64::to_bits))
            })
            .collect()
    }
    fn violacao(&self) -> f64 {
        violacao(
            &Folha::new([
                (self.path(self.caixa.corpo), Xform::IDENTITY),
                (self.path(self.caixa.tampa), Xform::IDENTITY),
            ]),
            self.scene.regras(),
        )
    }
    /// O ângulo da tampa medido nos cantos, em graus, a partir do topo.
    fn angulo(&self) -> f64 {
        let (h, t) = (self.p(self.caixa.tampa, 1), self.p(self.caixa.tampa, 2));
        (t[1] - h[1]).atan2(t[0] - h[0]).to_degrees()
    }
}

#[test]
fn the_lid_opens_and_scrubs_to_the_same_bit() {
    let mut a = cena();
    let corpo = a.path(a.caixa.corpo).clone();
    let tocado: Vec<Vec<u64>> = (0..=48)
        .map(|q| {
            let pose = a.em(q);
            assert!(
                a.violacao() <= TOL,
                "quadro {q}: violação {:e}",
                a.violacao()
            );
            let mexeu = a
                .path(a.caixa.corpo)
                .verts
                .iter()
                .zip(&corpo.verts)
                .map(|(x, y)| (x.anchor[0] - y.anchor[0]).hypot(x.anchor[1] - y.anchor[1]));
            assert!(
                mexeu.fold(0.0, f64::max) <= TOL,
                "quadro {q}: o corpo mexeu-se"
            );
            let (h, canto) = (a.p(a.caixa.tampa, 1), a.p(a.caixa.corpo, 4));
            assert!(
                (h[0] - canto[0]).hypot(h[1] - canto[1]) <= TOL,
                "quadro {q}: a dobradiça saiu do canto"
            );
            pose
        })
        .collect();
    for &(q, v) in &ANGULO {
        a.em(q);
        assert!(
            (a.angulo() - f64::from(v)).abs() <= 1e-4,
            "no quadro {q} a tampa abre {}° e a chave diz {v}°",
            a.angulo()
        );
    }
    assert_ne!(tocado[0], tocado[24], "controlo: o ângulo mexe a tampa");
    let mut b = cena();
    for q in [0, 40, 12, 40] {
        assert_eq!(
            b.em(q),
            tocado[q as usize],
            "arrastar até {q} ≠ tocar até {q}"
        );
    }
}
