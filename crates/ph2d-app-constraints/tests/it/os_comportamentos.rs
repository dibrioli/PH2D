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
        // Fora da grelha: uma fase inteira de ciclos seria invisível.
        semente: 3.25,
        ..perfil.numeros()
    }
}

/// ⭐ O Tremer (os dois eixos e a rotação), o Flutuar e o Orbitar dão o `eval` do nó numa instância.
#[test]
fn each_behaviour_is_its_motion_node_bit_for_bit() {
    for t in [0.0, 0.37, 1.25, 7.9] {
        let tr = de(Perfil::Tremer);
        let ruido = [
            ("amplitude", tr.amplitude as f32),
            ("frequency", tr.frequencia as f32),
            ("seed", tr.semente as f32),
        ];
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
        assert_eq!(
            escalar(&rot, "rot"),
            delta(&tr_rot, t)[2],
            "Tremer rot em {t}"
        );

        let fl = de(Perfil::Flutuar);
        let onda = cozinha(
            "motion.oscillator",
            &[
                ("channel", 1.0),
                ("amplitude", 0.25),
                ("frequency", 0.5),
                ("phase", 3.25),
            ],
            [0.0; 2],
            t,
        );
        assert_eq!(vec2(&onda, "P")[1], delta(&fl, t)[1], "Flutuar em {t}");

        let or = Comportamento {
            angulo: 33.0,
            ..de(Perfil::Orbitar)
        };
        let volta = cozinha(
            "motion.orbit",
            &[
                ("pivot_x", 5.0),
                ("pivot_y", 7.0),
                ("angle", 33.0),
                ("speed", or.velocidade as f32),
            ],
            [5.0 + or.raio as f32, 7.0],
            t,
        );
        let (q, _) = orbita(&or, [5.0, 7.0], t);
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
    for p in [
        [0.0, 0.0],
        [3.0, -1.5],
        [15.9, 20.0],
        [40.0, 0.0],
        [10.0, 31.0],
    ] {
        let s = cozinha(
            "force.attractor",
            &[
                ("strength", 400.0),
                ("radius", 30.0),
                ("curve", 0.0),
                ("peak", 1.0),
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
    let r = Relogio { doc, t, laco: None };
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
    let (mut sim, _, _) = mundo(0.0, [4.0, 0.0], Perfil::Atrair.numeros(), true);
    let mut drive = PreviewDrive::default();
    let doc = TimelineDoc::new();
    let fps = doc.fps_display;
    let tocado: Vec<f32> = (0..40)
        .map(|f| em(&mut sim, &mut drive, &doc, f as f64 / fps).translation.x)
        .collect();
    assert_eq!(tocado[0], 0.0, "no quadro 0 parte do repouso");
    assert!(tocado[39] > 3.9, "chega ao alvo: {tocado:?}");
    assert!(tocado.iter().all(|x| *x < 4.05), "sem passar dele: {tocado:?}");
    for f in [12, 3, 31] {
        let x = em(&mut sim, &mut drive, &doc, f as f64 / fps).translation.x;
        assert_eq!(x.to_bits(), tocado[f].to_bits(), "quadro {f}");
    }
}

/// Sem alvo, o Atrair fica inerte com a frase do alvo.
#[test]
fn attract_without_a_target_is_inert_with_a_reason() {
    let (mut sim, a, id) = mundo(0.0, [4.0, 0.0], Perfil::Atrair.numeros(), false);
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
            perfil.mostra_alvo(),
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

/// ⭐ «Weight By» → clique no objecto: o peso grava-se pelo `StableId` (o raio de fábrica); «Clear»
/// tira-o. O dono não serve; o vazio desiste.
#[test]
fn picking_the_weight_object_writes_it_and_clear_removes_it() {
    use ph2d_app_constraints::inspector::{Escolha, armado, escolher_alvo, info};
    let (mut sim, a, id) = mundo(0.0, [50.0, 0.0], de(Perfil::Tremer), false);
    let b = ph2d_ecs::entity_with_stable_id(sim.world(), StableId(1)).unwrap();
    let dono = a.to_bits();
    let linha = |sim: &SimWorld| info(sim, dono, 1).unwrap().rows[0].peso.clone().unwrap();
    assert_eq!(linha(&sim).nome, None);
    apply_all(&mut sim, &[(dono, E::EscolherPeso(id))]);
    assert!(armado(&sim) && linha(&sim).escolhendo);
    assert_eq!(
        escolher_alvo(&mut sim, &[dono], [0.0; 2]),
        Escolha::Recusado
    );
    assert_eq!(
        escolher_alvo(&mut sim, &[dono, b.to_bits()], [0.0; 2]),
        Escolha::Gravado
    );
    assert!(!armado(&sim));
    let Kind::Comportamento(c) = sim.world().get::<Constraints>(a).unwrap().items[0].kind else {
        panic!("tipo");
    };
    assert_eq!(c.peso.map(|p| p.objecto), Some(1));
    assert_eq!(c.peso.map(|p| p.raio), Some(2.0));
    assert!(linha(&sim).nome.is_some());
    assert!(apply_all(&mut sim, &[(dono, E::LargarPeso(id))]));
    assert_eq!(linha(&sim).nome, None);
    apply_all(&mut sim, &[(dono, E::EscolherPeso(id))]);
    assert_eq!(escolher_alvo(&mut sim, &[], [0.0; 2]), Escolha::Desistiu);
    assert!(!armado(&sim));
}

/// O passo do Atrair é o semi-implícito do `sim.step` — velocidade, atrito linear, posição com a
/// velocidade NOVA —, transcrito à parte para o fixar.
#[test]
fn attract_steps_like_the_sim_step() {
    use ph2d_constraints_ecs::{Dinamica, Simulador};
    let lei = Atracao::from(&Perfil::Atrair.numeros());
    let fps = 24.0;
    let dt = (1.0 / fps) as f32;
    let alvo = [4.0_f64, 1.0];
    let (mut v, mut q) = ([0.0_f32; 2], [0.0_f32; 2]);
    let mut sim: Simulador<2> = Simulador::default();
    // De fábrica, 2 sub-passos: `2·√(400/1)·dt = 1,67` e `k·dt = 1,67` arredondam a 2.
    let h = dt / 2.0;
    for f in 1..30 {
        let k = lei.arrasto as f32;
        for _ in 0..2 {
            let a = lei.puxao().accel(q, alvo.map(|x| x as f32), 1.0);
            for c in 0..2 {
                v[c] += (a[c] - v[c] * k) * h;
                q[c] += v[c] * h;
            }
        }
        let got = sim.corre(f, lei, fps, &mut |_| alvo);
        assert_eq!(got, q.map(f64::from), "quadro {f}");
    }
    let _ = lei.angular();
}

/// ⭐ Report do dono 27/10: *«quando o planeta se move o raio da órbita aumenta»*. A órbita é RELATIVA
/// ao centro (como filho dele): o planeta anda e a lua fica ao mesmo raio.
#[test]
fn the_orbit_keeps_its_radius_when_the_centre_moves() {
    let or = Perfil::Orbitar.numeros();
    let (mut sim, _, _) = mundo(1.5, [0.0, 0.0], or, true);
    let mut drive = PreviewDrive::default();
    let doc = TimelineDoc::new();
    for (k, x) in [0.0_f32, 2.0, 5.0, -3.0].into_iter().enumerate() {
        let b = ph2d_ecs::entity_with_stable_id(sim.world(), StableId(1)).unwrap();
        sim.world_mut().get_mut::<Transform>(b).unwrap().translation.x = x;
        let l = em(&mut sim, &mut drive, &doc, 0.3 * k as f64).translation;
        let r = f64::from(l.x - x).hypot(f64::from(l.y));
        assert!((r - or.raio).abs() < or.raio * 1e-3, "centro em {x}: raio {r}");
    }
}

/// Escolher o centro apanha o raio e o ângulo de onde a lua está: ligar não a faz saltar.
#[test]
fn picking_the_orbit_centre_does_not_move_the_moon() {
    use ph2d_app_constraints::inspector::{Escolha, escolher_alvo};
    let mut sim = SimWorld::new();
    let mut tp = Transform::IDENTITY;
    tp.translation.x = 1.0;
    let p = sim.world_mut().spawn((StableId(1), tp)).id();
    let mut tl = Transform::IDENTITY;
    tl.translation.x = 1.6;
    tl.translation.y = 2.3;
    let a = sim
        .world_mut()
        .spawn((StableId(2), tl, Constraints::default()))
        .id();
    let i = TIPO_BEHAVIOUR + Perfil::Orbitar.tag();
    apply_all(&mut sim, &[(a.to_bits(), E::Adicionar(i))]);
    assert_eq!(escolher_alvo(&mut sim, &[p.to_bits()], [0.0; 2]), Escolha::Gravado);
    let Kind::Comportamento(c) = sim.world().get::<Constraints>(a).unwrap().items[0].kind else {
        panic!("tipo");
    };
    assert!((c.raio - 0.6_f64.hypot(2.3)).abs() < 1e-6, "{}", c.raio);
    let mut drive = PreviewDrive::default();
    let doc = TimelineDoc::new();
    let l = em(&mut sim, &mut drive, &doc, 0.0).translation;
    assert!((l.x - 1.6).abs() < 3e-3 && (l.y - 2.3).abs() < 3e-3, "saltou para {l:?}");
}

/// ⭐ Report do dono 27/10: *«a limalha não corre atrás, parece que corre na frente»*. Na curva do
/// íman da cena 10, a limalha segue ATRÁS (como o Harmonic do Blender, oráculo 27/10: 0,68 m atrás,
/// nunca à frente) e nunca se perde.
#[test]
fn attracted_filings_run_behind_a_moving_magnet() {
    let (mut sim, _, _) = mundo(-4.5, [-4.5, -0.8], Perfil::Atrair.numeros(), true);
    let ima = ph2d_ecs::entity_with_stable_id(sim.world(), StableId(1)).unwrap();
    let mut tl = ph2d_timeline::TimelineState::default();
    let mut ph = ph2d_core::Playhead::default();
    let fps = tl.doc.fps_display;
    for i in ph2d_app_constraints::smoke_recreio::chaves_do_ima(ima.to_bits(), fps) {
        ph2d_timeline::apply_intent(&mut tl, &mut ph, i);
    }
    let mut drive = PreviewDrive::default();
    let (mut frente, mut atras, mut antes) = (0.0_f32, 0.0_f32, -4.5_f32);
    for f in 0..180 {
        let t = f64::from(f) / fps;
        let x = ph2d_timeline::world_pose_at(sim.world(), &tl.doc, ima.to_bits(), t)
            .unwrap()
            .translation
            .x;
        sim.world_mut().get_mut::<Transform>(ima).unwrap().translation.x = x;
        let l = em(&mut sim, &mut drive, &tl.doc, t).translation;
        let sentido = (x - antes).signum();
        antes = x;
        if sentido != 0.0 && f > 6 {
            frente = frente.max((l.x - x) * sentido);
            atras = atras.max((x - l.x) * sentido);
        }
    }
    assert!(frente < 0.05, "à frente do íman: {frente}");
    assert!(atras > 0.2 && atras < 1.0, "atrás dele, perto: {atras}");
}
