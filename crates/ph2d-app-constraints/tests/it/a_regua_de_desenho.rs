//! ⭐ **A cena 6 — a medida animada** (W4.5, `docs/BlenderConstraints/04_W4_cad.md` §4–§5): a altura do dente do
//! meio tem três chaves e a pose resolve-se a partir do desenho AUTORADO em cada quadro ⇒ tocar e arrastar o tempo
//! dão o mesmo bit, e nenhum quadro desfaz uma regra. As chaves entram pela porta do cartão (`chaves::gravar`).

use ph2d_app_constraints::chaves;
use ph2d_app_constraints::smoke_regua::{self, MEDIDA, Regua};
use ph2d_ecs::canal::familia::CAD;
use ph2d_ecs::{CanaisEscritos, SimWorld, VecPathRef};
use ph2d_timeline::{TimelineState, apply_from_doc};
use ph2d_vec_constraints::{Folha, violacao};
use ph2d_vec_entities::medida::{self, ConducaoCad};
use ph2d_vec_scene::{VecPath, VecScene, VertId, Xform};

/// O cumprimento exigido, em unidades do mundo: `1e-4` px até 100 px por unidade.
const TOL: f64 = 1e-6;

struct Cena {
    sim: SimWorld,
    scene: VecScene,
    cond: ConducaoCad,
    tl: TimelineState,
    regua: Regua,
}

fn cena() -> Cena {
    let mut scene = VecScene::new();
    let regua = smoke_regua::semear(&mut scene);
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(VecPathRef(regua.path)).id().to_bits();
    let mut tl = TimelineState::default();
    let fps = tl.doc.fps_display;
    let lote = smoke_regua::chaves_da_medida(e, regua.medida, fps);
    assert!(
        chaves::gravar(&mut tl, &lote, false, 0.0, true),
        "as três chaves, num passo"
    );
    Cena {
        sim,
        scene,
        cond: ConducaoCad::default(),
        tl,
        regua,
    }
}

impl Cena {
    fn path(&self) -> &VecPath {
        self.scene.path(self.regua.path).expect("a régua")
    }

    /// Vai ao quadro `q` como a shell: o apply da timeline, a família CAD drena, o fim do quadro assenta.
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
        self.path()
            .verts
            .iter()
            .flat_map(|v| v.anchor.map(f64::to_bits))
            .collect()
    }

    fn violacao(&self) -> f64 {
        violacao(
            &Folha::new([(self.path(), Xform::IDENTITY)]),
            self.scene.regras(),
        )
    }

    fn y(&self, i: u32) -> f64 {
        self.path().vert_by_id(VertId(i + 1)).unwrap().anchor[1]
    }

    /// A altura do dente do meio, medida nos cantos (não na regra).
    fn altura(&self) -> f64 {
        let meio = 4 * (smoke_regua::DENTES / 2);
        self.y(meio + 1) - self.y(meio)
    }

    /// O pé do dente do meio (o chão das folgas): a cena ensina que o dente estica para CIMA.
    fn pe(&self) -> f64 {
        self.y(4 * (smoke_regua::DENTES / 2))
    }
}

#[test]
fn an_animated_dimension_plays_and_scrubs_to_the_same_bit() {
    let mut a = cena();
    let autorado = a.path().clone();
    let tocado: Vec<Vec<u64>> = (0..=48)
        .map(|q| {
            let pose = a.em(q);
            assert!(
                a.pe().abs() <= TOL,
                "quadro {q}: o pé afundou para {}",
                a.pe()
            );
            assert!(
                a.violacao() <= TOL,
                "quadro {q}: violação {:e}",
                a.violacao()
            );
            pose
        })
        .collect();
    for &(q, v) in &MEDIDA {
        a.em(q);
        assert!(
            (a.altura() - f64::from(v)).abs() <= TOL,
            "no quadro {q} o dente mede {} e a chave diz {v}",
            a.altura()
        );
    }
    assert_ne!(tocado[0], tocado[24], "controlo: a medida mexe o desenho");
    assert_eq!(
        a.cond.path(a.regua.path),
        Some(&autorado),
        "o autorado é o desenho de antes da curva"
    );
    let mut b = cena();
    for q in [0, 40, 12, 40] {
        assert_eq!(
            b.em(q),
            tocado[q as usize],
            "arrastar até {q} ≠ tocar até {q}"
        );
        assert!(b.violacao() <= TOL);
    }
    // O canto preso não saiu do lugar.
    assert_eq!(a.path().verts[0].anchor, autorado.verts[0].anchor);
}
