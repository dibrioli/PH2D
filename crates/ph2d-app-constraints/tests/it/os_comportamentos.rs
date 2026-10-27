//! ⭐ ***Behaviours* de um clique** (`docs/BlenderConstraints/06_W5_resto.md` §3.2): Tremer, Flutuar e
//! Orbitar são o `eval` do nó do Motion de igual nome, bit a bit; o Atrair é a força do
//! `force.attractor` integrada quadro a quadro (o scrub dá o bit de tocar); o peso por distância é o
//! campo do `motion.falloff`.

use bevy_ecs::entity::Entity;
use ph2d_app_constraints::inspector::apply_all;
use ph2d_app_constraints::quadro;
use ph2d_app_constraints::relogio::Relogio;
use ph2d_constraints::Kind;
use ph2d_constraints::comportamento::{Comportamento, Onde, Perfil, Peso};
use ph2d_constraints_ecs::comportamento::{Atracao, delta, orbita, peso};
use ph2d_constraints_ecs::{Constraints, Estado, Estados};
use ph2d_ecs::{SimWorld, StableId, Transform};
use ph2d_editor_core::constraint_edits::{ConstraintFieldEdit as E, TIPO_BEHAVIOUR};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use ph2d_preview_drive::PreviewDrive;
use ph2d_timeline::TimelineDoc;

static SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("test.um_elemento"),
    name: "test.um_elemento",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame),
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

/// UM elemento em `P` — o elemento `0` do nó é o objecto.
struct Um([f32; 2]);

impl NodeOp for Um {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![self.0])));
    }
}

struct Ops(NodeRegistry, Um);

impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        if ty == SRC.id {
            Some(&self.1)
        } else {
            self.0.resolve(ty)
        }
    }
}

/// O nó `tipo` com `params`, sobre um elemento em `p`, no instante `t`: a saída inteira.
fn cozinha(tipo: &str, params: &[(&str, f32)], p: [f32; 2], t: f64) -> Stream {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_wiggle::register(&mut reg).unwrap();
    ph2d_node_motion_oscillator::register(&mut reg).unwrap();
    ph2d_node_motion_orbit::register(&mut reg).unwrap();
    ph2d_node_motion_falloff::register(&mut reg).unwrap();
    ph2d_node_force_attractor::register(&mut reg).unwrap();
    let mut g = Graph::new();
    let src = g.add_node("test.um_elemento");
    let n = g.add_node(tipo);
    g.connect(Edge {
        from: (src, 0),
        to: (n, 0),
        delayed: false,
    })
    .unwrap();
    for &(k, v) in params {
        g.set_param(n, k, v);
    }
    let ops = Ops(reg, Um(p));
    let mut cook = Cook::new();
    cook.cook(&g, &ops, n, t).unwrap()[0].as_stream().clone()
}

fn vec2(s: &Stream, c: &str) -> [f32; 2] {
    match s.get(c) {
        Some(Column::Vec2(v)) => v[0],
        _ => panic!("{c}"),
    }
}

fn escalar(s: &Stream, c: &str) -> f32 {
    match s.get(c) {
        Some(Column::Scalar(v)) => v[0],
        _ => panic!("{c}"),
    }
}

fn de(perfil: Perfil) -> Comportamento {
    Comportamento {
        semente: 3.0,
        ..perfil.numeros()
    }
}

/// ⭐ O Tremer (os dois eixos e a rotação), o Flutuar e o Orbitar dão o `eval` do nó numa instância.
#[test]
fn each_behaviour_is_its_motion_node_bit_for_bit() {
    for t in [0.0, 0.37, 1.25, 7.9] {
        let tr = de(Perfil::Tremer);
        let ruido = [("amplitude", 6.0), ("frequency", 2.0), ("seed", 3.0)];
        let xy = cozinha(
            "motion.wiggle",
            &[ruido.as_slice(), &[("channel", 4.0)]].concat(),
            [0.0; 2],
            t,
        );
        let [dx, dy, _] = delta(&tr, t);
        assert_eq!(vec2(&xy, "P"), [dx, dy], "Tremer XY em {t}");
        let rot = cozinha(
            "motion.wiggle",
            &[ruido.as_slice(), &[("channel", 2.0)]].concat(),
            [0.0; 2],
            t,
        );
        let tr_rot = Comportamento {
            onde: Onde::Rot,
            ..tr
        };
        assert_eq!(escalar(&rot, "rot"), delta(&tr_rot, t)[2], "Tremer rot em {t}");

        let fl = de(Perfil::Flutuar);
        let onda = cozinha(
            "motion.oscillator",
            &[
                ("channel", 1.0),
                ("amplitude", 12.0),
                ("frequency", 0.5),
                ("phase", 3.0),
            ],
            [0.0; 2],
            t,
        );
        assert_eq!(vec2(&onda, "P")[1], delta(&fl, t)[1], "Flutuar em {t}");

        let or = de(Perfil::Orbitar);
        let p = [40.0, -12.0];
        let volta = cozinha(
            "motion.orbit",
            &[("pivot_x", 5.0), ("pivot_y", 7.0), ("speed", 90.0)],
            p,
            t,
        );
        let (q, _) = orbita(&or, p.map(f64::from), [5.0, 7.0], t, 1.0);
        assert_eq!(vec2(&volta, "P"), q, "Orbitar em {t}");
    }
}

/// O peso é o campo do `motion.falloff` (disco, sem rotação) na posição do dono.
#[test]
fn the_weight_is_the_falloff_field_bit_for_bit() {
    for (p, curva) in [([30.0, 40.0], 2), ([10.0, -5.0], 0), ([90.0, 0.0], 3)] {
        let w = Peso {
            objecto: 0,
            raio: 80.0,
            curva,
            inverte: false,
        };
        let s = cozinha(
            "motion.falloff",
            &[
                ("radius", 80.0),
                ("curve", f32::from(curva)),
                ("center_x", 2.0),
                ("center_y", 1.0),
            ],
            p,
            0.0,
        );
        assert_eq!(
            escalar(&s, "falloff"),
            peso(&w, p.map(f64::from), [2.0, 1.0])
        );
    }
}

/// A aceleração do Atrair é a do `force.attractor` (Smooth, sem repulsão).
#[test]
fn the_attract_force_is_the_attractor_law_bit_for_bit() {
    let lei = Atracao::from(&Perfil::Atrair.numeros());
    for p in [[0.0, 0.0], [120.0, -30.0], [299.0, 0.0], [400.0, 0.0]] {
        let s = cozinha(
            "force.attractor",
            &[
                ("strength", 900.0),
                ("radius", 300.0),
                ("target_x", 10.0),
                ("target_y", 20.0),
            ],
            p,
            0.0,
        );
        assert_eq!(
            vec2(&s, "accel"),
            lei.puxao().accel(p, [10.0, 20.0], 1.0),
            "em {p:?}"
        );
    }
}

/// O dono `A` (id 2) em `(x, 0)` com o `c` e o alvo/peso `B` (id 1) em `b`.
fn mundo(x: f32, b: [f32; 2], c: Comportamento, alvo: bool) -> (SimWorld, Entity, u32) {
    let mut sim = SimWorld::new();
    let mut tb = Transform::IDENTITY;
    tb.translation.x = b[0];
    tb.translation.y = b[1];
    sim.world_mut().spawn((StableId(1), tb));
    let mut k = Constraints::default();
    let id = k.push(Kind::Comportamento(c), alvo.then_some(StableId(1)));
    let mut ta = Transform::IDENTITY;
    ta.translation.x = x;
    let a = sim.world_mut().spawn((StableId(2), ta, k)).id();
    (sim, a, id)
}

fn em(sim: &mut SimWorld, drive: &mut PreviewDrive, doc: &TimelineDoc, t: f64) -> Transform {
    let r = Relogio {
        doc,
        t,
        laco: None,
    };
    quadro(sim, &mut ph2d_vec_scene::VecScene::new(), drive, Some(r));
    let w = sim.world();
    let a = ph2d_ecs::entity_with_stable_id(w, StableId(2)).unwrap();
    *w.get::<Transform>(a).unwrap()
}

/// No objecto: a pose é a de repouso mais a saída do nó — o Flutuar no Y, sem nada mais.
#[test]
fn a_behaviour_moves_any_object_by_its_node() {
    let fl = de(Perfil::Flutuar);
    let (mut sim, _, _) = mundo(0.0, [500.0, 0.0], fl, false);
    let mut drive = PreviewDrive::default();
    let doc = TimelineDoc::new();
    for t in [0.2, 0.9] {
        let a = em(&mut sim, &mut drive, &doc, t);
        assert_eq!(a.translation.x, 0.0);
        assert_eq!(a.translation.y, delta(&fl, t)[1], "em {t}");
    }
}

/// ⭐ O Atrair puxa o dono para o alvo com estado — e arrastar a timeline para trás dá o bit de tocar.
#[test]
fn attract_scrubs_to_the_same_bit() {
    let (mut sim, _, _) = mundo(0.0, [200.0, 0.0], Perfil::Atrair.numeros(), true);
    let mut drive = PreviewDrive::default();
    let doc = TimelineDoc::new();
    let fps = doc.fps_display;
    let tocado: Vec<f32> = (0..40)
        .map(|f| em(&mut sim, &mut drive, &doc, f as f64 / fps).translation.x)
        .collect();
    assert_eq!(tocado[0], 0.0, "no quadro 0 parte do repouso");
    assert!(tocado[20] > 20.0, "puxado para o alvo: {}", tocado[20]);
    assert!(tocado[39] > 150.0, "chega ao alvo: {tocado:?}");
    for f in [12, 3, 31] {
        let x = em(&mut sim, &mut drive, &doc, f as f64 / fps).translation.x;
        assert_eq!(x.to_bits(), tocado[f].to_bits(), "quadro {f}");
    }
}

/// Sem alvo, o Atrair fica inerte com a frase do alvo.
#[test]
fn attract_without_a_target_is_inert_with_a_reason() {
    let (mut sim, a, id) = mundo(0.0, [200.0, 0.0], Perfil::Atrair.numeros(), false);
    let mut drive = PreviewDrive::default();
    let doc = TimelineDoc::new();
    let _ = em(&mut sim, &mut drive, &doc, 0.5);
    let estado = sim.world().resource::<Estados>().of(a, id);
    assert_eq!(estado, Some(Estado::SemAlvo));
}

/// ⭐ O peso por distância: perto do objecto do peso o *behaviour* vale inteiro, a meio do raio vale o
/// campo, fora dele não mexe.
#[test]
fn a_falloff_object_weighs_the_behaviour_by_distance() {
    let doc = TimelineDoc::new();
    let t = 0.4;
    for x in [0.0_f32, 30.0, 100.0] {
        let p = Peso {
            objecto: 1,
            raio: 60.0,
            curva: 0,
            inverte: false,
        };
        let fl = Comportamento {
            peso: Some(p),
            ..de(Perfil::Flutuar)
        };
        let (mut sim, _, _) = mundo(x, [0.0, 0.0], fl, false);
        let mut drive = PreviewDrive::default();
        let a = em(&mut sim, &mut drive, &doc, t);
        let w = peso(&p, [f64::from(x), 0.0], [0.0, 0.0]);
        let esperado = (f64::from(delta(&fl, t)[1]) * f64::from(w)) as f32;
        assert_eq!(a.translation.y, esperado, "em x = {x}");
        match x {
            0.0 => assert_eq!(w, 1.0),
            30.0 => assert_eq!(w, 0.5),
            _ => assert_eq!(a.translation.y, 0.0),
        }
    }
}

/// ⭐ Um clique no submenu cria o *behaviour* com os números de fábrica do perfil; o Atrair espera o
/// alvo, os outros não.
#[test]
fn one_click_applies_the_factory_profile() {
    for perfil in Perfil::ALL {
        let mut sim = SimWorld::new();
        let a = sim
            .world_mut()
            .spawn((StableId(2), Transform::IDENTITY, Constraints::default()))
            .id();
        let i = TIPO_BEHAVIOUR + perfil.tag();
        assert!(apply_all(&mut sim, &[(a.to_bits(), E::Adicionar(i))]));
        let c = sim.world().get::<Constraints>(a).unwrap();
        assert_eq!(c.items[0].kind, Kind::Comportamento(perfil.numeros()));
        assert_eq!(
            ph2d_app_constraints::inspector::armado(&sim),
            perfil == Perfil::Atrair,
            "{perfil:?}"
        );
    }
}

#[test]
fn a_behaviour_survives_save_and_open() {
    let c = Comportamento {
        onde: Onde::Rot,
        peso: Some(Peso {
            objecto: 9,
            raio: 12.0,
            curva: 3,
            inverte: true,
        }),
        ..Perfil::Orbitar.numeros()
    };
    let (sim, a, _) = mundo(0.0, [0.0; 2], c, true);
    let k = sim.world().get::<Constraints>(a).unwrap();
    let lido: Constraints = serde_json::from_str(&serde_json::to_string(k).unwrap()).unwrap();
    assert_eq!(&lido, k);
}
