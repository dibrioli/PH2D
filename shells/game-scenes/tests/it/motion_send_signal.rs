//! ⭐ MI-8 «Send signal» de ponta a ponta (CPU): um envelope que cai abaixo do nível uma vez manda
//! UM sinal, com o nome autorado e o objecto Motion dono do nó como origem (`quem()`), pelo mesmo
//! caminho do quadro: a bomba marcha a tomada, `collect_signals` decide a borda, `from_motion` publica.

use ph2d_app_motion::motion_bridge::signals::{collect_signals, signal_nodes};
use ph2d_app_motion::motion_objeto::{Dono, IDENTIDADE};
use ph2d_app_motion::motion_state::MotionState;
use ph2d_node_send_signal as send;
use ph2d_nodegraph::cook::TimeScopes;
use ph2d_nodegraph::graph::{Edge, NodeId};
use ph2d_runtime::{EntityBits, Signal, SignalOutbox, SignalReader};

const FPS: f64 = 60.0;
const SID: u64 = 77;
const BITS: u64 = 4242;

/// `grid 1×1 → beat → adsr → Send signal (Falls below 0.5) → drive → output`; o nó é do objecto `SID`.
fn grafo(motion: &mut MotionState, bits: u64) -> NodeId {
    let g = &mut motion.doc.graph;
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 1.0);
    let beat = g.add_node("pulse.beat");
    g.set_param(beat, "period", 30.0);
    let env = g.add_node("pulse.adsr");
    for (k, v) in [
        ("attack", 0.05),
        ("decay", 0.1),
        ("sustain", 0.8),
        ("hold", 0.2),
        ("release", 0.3),
    ] {
        g.set_param(env, k, v);
    }
    let s = g.add_node(send::MANIFEST.name);
    g.set_text_param(s, send::NAME_KEY, "caiu");
    g.set_param(s, send::WHEN, send::WHEN_FALLS as f32);
    g.set_param(s, send::LEVEL, 0.5);
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");
    let fios = [
        (grid, 0, beat, 0),
        (beat, 0, env, 0),
        (env, 0, s, 0),
        (grid, 0, drive, 0),
        (s, 0, drive, 1),
        (drive, 0, out, 0),
    ];
    for (a, ap, b, bp) in fios {
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed: false,
        })
        .expect("fio");
    }
    for n in [beat, env] {
        g.connect(Edge {
            from: (n, 0),
            to: (n, 1),
            delayed: true,
        })
        .expect("pre");
    }
    g.validate(&motion.registry).expect("grafo válido");
    motion.doc.set_owner(s, SID);
    let dono = Dono {
        entity: bits,
        visible: true,
        afim: IDENTIDADE,
    };
    motion.objeto.donos.insert(SID, dono);
    motion.sinks = vec![out];
    s
}

/// Por qual porta a bomba marcha (as duas do `dispatch`): a da CPU (sinks) ou a HÍBRIDA (o prefixo
/// de CPU até às fronteiras do plano da placa).
#[derive(Clone, Copy)]
enum Rota {
    Cpu,
    Hibrida,
}

/// O relógio do quadro, pela lei REAL (`Playhead::crosses_forward`, a pergunta do shell antes de
/// `collect_signals`).
#[derive(Clone, Copy, Debug)]
enum Relogio {
    Toca,
    Parado,
    Salta,
}

/// `segundos` de jogo pela `rota`, com a lei do `relogio`. Devolve o que a outbox recebeu.
fn corre(segundos: f64, rota: Rota, relogio: Relogio) -> Vec<Signal> {
    let outbox = outbox(segundos, rota, relogio, BITS);
    let mut leitor = SignalReader::new();
    outbox.read(&mut leitor).cloned().collect()
}

/// O barramento depois de `segundos` de jogo, com o nó dono do objecto Motion de `bits`.
fn outbox(segundos: f64, rota: Rota, relogio: Relogio, bits: u64) -> SignalOutbox {
    let mut motion = MotionState::new();
    grafo(&mut motion, bits);
    motion.signal_taps = signal_nodes(&motion.doc.graph);
    motion.pump.set_taps(&motion.signal_taps);
    motion.pump.clear_tap_fires();
    let plano = ph2d_gpu_cook::plan(
        &motion.doc.graph,
        &motion.registry,
        &motion.registry,
        motion.sinks[0],
    );
    let fronteiras: Vec<NodeId> = plano.boundaries.iter().map(|(n, _)| *n).collect();
    let mut playhead = ph2d_core::Playhead::new(1.0 / FPS);
    match relogio {
        Relogio::Parado => playhead.pause(),
        Relogio::Toca | Relogio::Salta => playhead.play(),
    }
    let scopes = TimeScopes::new();
    let mut outbox = SignalOutbox::new();
    for tick in 0..=((segundos * FPS) as u64) {
        let (g, r) = (&motion.doc.graph, &motion.registry);
        match rota {
            Rota::Cpu => motion.pump.advance_or_scrub_scoped(
                g,
                r,
                &motion.sinks,
                tick,
                |t| t as f64 / FPS,
                motion.default_uv_rect,
                motion.default_size,
                &scopes,
            ),
            Rota::Hibrida => motion.pump.advance_or_scrub_to_nodes_scoped(
                g,
                r,
                &fronteiras,
                tick,
                |t| t as f64 / FPS,
                &scopes,
            ),
        };
        if playhead.crosses_forward(matches!(relogio, Relogio::Salta)) {
            collect_signals(&mut motion);
        }
        motion.pump.clear_tap_fires();
        for sig in motion.signals_out.drain(..) {
            outbox.publish(Signal::from_motion(
                &sig.name, sig.source, sig.tick, sig.rows,
            ));
        }
    }
    outbox
}

/// ⭐ Uma queda abaixo do nível = UM sinal, «caiu», com o objecto Motion como origem — pelas DUAS
/// portas de marcha; relógio parado e salto no play (a lei real) = nada.
#[test]
fn send_signal_falls_below_once_and_speaks_for_the_motion_object() {
    for rota in [Rota::Cpu, Rota::Hibrida] {
        let sinais = corre(2.0, rota, Relogio::Toca);
        assert_eq!(sinais.len(), 1, "uma queda = um sinal: {sinais:?}");
        assert_eq!(&*sinais[0].name, "caiu");
        assert_eq!(
            sinais[0].origin.quem(),
            Some(EntityBits(BITS)),
            "a origem é o objecto Motion"
        );
        assert!(
            corre(2.0, rota, Relogio::Parado).is_empty(),
            "relógio parado: nada"
        );
        assert!(
            corre(2.0, rota, Relogio::Salta).is_empty(),
            "um salto no play: nada"
        );
    }
}

/// Uma cena com o objecto «Motion» (dono do nó) e a «Door» visível; devolve `(sim, motion, door)`.
fn cena_com_porta() -> (ph2d_ecs::SimWorld, ph2d_ecs::Entity, ph2d_ecs::Entity) {
    use ph2d_ecs::{Name, StableId, Transform, Visibility};
    let mut sim = ph2d_ecs::SimWorld::new();
    let w = sim.world_mut();
    let motion = w
        .spawn((Transform::IDENTITY, StableId(SID), Name::new("Motion")))
        .id();
    let door = w
        .spawn((
            Transform::IDENTITY,
            StableId(SID + 1),
            Name::new("Door"),
            Visibility::visible(),
        ))
        .id();
    (sim, motion, door)
}

/// ⭐ **Uma regra «When signal caiu» reage ao nó** (a tabela de acções do quadro): a linha mora no
/// objecto Motion com «From: Myself» e esconde a «Door» — só passa porque a origem é o Motion.
/// Controlo: relógio parado (o nó não manda) ⇒ a porta fica.
#[test]
fn a_rule_on_the_motion_object_hides_the_door_when_the_node_sends() {
    use ph2d_app_components::signal_actions_bridge::{apply, lido};
    use ph2d_ecs::{SignalAction, SignalActions, SignalFrom, SignalVerb, Visibility};
    for (relogio, escondida) in [(Relogio::Toca, true), (Relogio::Parado, false)] {
        let (mut sim, motion, door) = cena_com_porta();
        sim.world_mut()
            .entity_mut(motion)
            .insert(SignalActions(vec![SignalAction {
                on: "caiu".into(),
                target: "Door".into(),
                verb: SignalVerb::Hide,
                from: SignalFrom::Myself,
                ..SignalAction::default()
            }]));
        let outbox = outbox(2.0, Rota::Cpu, relogio, motion.to_bits());
        let mut leitor = SignalReader::new();
        let lidos: Vec<_> = outbox.read(&mut leitor).map(lido).collect();
        let disparos: Vec<ph2d_ecs::Disparo<'_>> = lidos
            .iter()
            .map(|(nome, quem, outro)| ph2d_ecs::Disparo {
                nome,
                quem: *quem,
                outro: *outro,
            })
            .collect();
        let tags = ph2d_tags::TagTree::new();
        let efeitos = ph2d_ecs::resolve_signal_actions(sim.world_mut(), &tags, &disparos);
        let mut drive = ph2d_preview_drive::PreviewDrive::default();
        let r = apply(&mut sim, &efeitos, 2.0, &mut drive, &mut |_, _, _| false);
        assert_eq!(r.applied, usize::from(escondida), "{relogio:?}: {r:?}");
        assert_eq!(
            sim.world().get::<Visibility>(door).map(|v| v.hidden),
            Some(escondida),
            "{relogio:?}"
        );
    }
}

/// ⭐ **Um script Luau ouve o nó**: `on_signal(self, signal, from)` recebe `signal = "caiu"` e `from` =
/// o objecto Motion (o mesmo id de `ph2d.find_by_name`), pelos motores do quadro (`motores::correm`).
/// Controlo: relógio parado ⇒ o gancho não corre.
#[test]
fn a_luau_on_signal_hears_the_node_with_the_motion_object_as_from() {
    use ph2d_app_components::motores;
    use ph2d_ecs::Transform;
    const SRC: &str = r#"
function on_signal(self, signal, from)
  if signal == "caiu" then ph2d.set(self.id, "y", 1) end
  if from == ph2d.find_by_name("Motion") then ph2d.set(self.id, "x", 1) end
end
"#;
    for (relogio, ouviu) in [(Relogio::Toca, 1.0), (Relogio::Parado, 0.0)] {
        let (mut sim, motion, door) = cena_com_porta();
        let id = sim
            .world_mut()
            .get_resource_or_init::<ph2d_script::ProjectScripts>()
            .insert("ouvinte.luau", SRC.to_owned());
        sim.world_mut()
            .entity_mut(door)
            .insert(ph2d_script::LuauScript::library(id));
        let mut outbox = outbox(2.0, Rota::Cpu, relogio, motion.to_bits());
        let reg = crate::comp::registo();
        let tags = ph2d_tags::TagTree::new();
        let (mut vec, mut mapa) = ph2d_app_components::instance_docs::empty_docs();
        let mut portas = ph2d_app_components::script_bridge::ScriptDoors {
            registry: &reg,
            tags: &tags,
            docs: ph2d_app_components::instance_docs::OwnedDocs {
                vec_scene: &mut vec,
                vec_entities: &mut mapa,
            },
            tick: 1,
            images: ph2d_app_components::script_image::ImageLibrary::none(),
            physics: None,
        };
        let mut host = Some(ph2d_script::ScriptHost::new().expect("vm"));
        let (mut r_script, mut r_part, mut r_arma) = (
            SignalReader::new(),
            SignalReader::new(),
            SignalReader::new(),
        );
        motores::correm(
            &mut sim,
            &mut host,
            &mut ph2d_app_components::particles_bridge::ParticlesState::new(),
            &mut ph2d_app_components::health_bar_bridge::HealthBarsState::new(),
            &ph2d_ecs::sort_key::SortScratch::default(),
            &mut ph2d_preview_drive::PreviewDrive::default(),
            &mut outbox,
            motores::Leitores {
                script: &mut r_script,
                particles: &mut r_part,
                weapon: &mut r_arma,
            },
            &motores::Relogio::do_quadro(true, 1, 1.0 / FPS),
            &std::collections::BTreeMap::new(),
            &mut portas,
        );
        let t = sim.world().get::<Transform>(door).expect("pose");
        assert_eq!(t.translation.y, ouviu, "{relogio:?}: signal = o nome");
        assert_eq!(
            t.translation.x, ouviu,
            "{relogio:?}: from = o objecto Motion"
        );
    }
}

/// ⭐ O cartão MOTION (MI-8) lê os «Send signal» DO objecto, com o nome e as vezes desta corrida;
/// parado, outro objecto ou sem nome = nada. **Mutação que deve sangrar:** tirar o filtro do dono.
#[test]
fn o_cartao_motion_le_os_envios_do_objecto_nesta_corrida() {
    use ph2d_app_motion::motion_objeto::origem::envios_do_objeto;
    let mut motion = MotionState::new();
    let s = grafo(&mut motion, BITS);
    motion.envios.corrida(true);
    let l = send::Leitura {
        v: 0.0,
        when: send::WHEN_PULSE,
        level: 0.5,
        pulsos: 1,
    };
    motion.envios.passo(s, 1, &l);
    motion.envios.passo(s, 2, &l);
    let info = envios_do_objeto(&motion, BITS, true).expect("em Play há cartão");
    assert_eq!(info.entity_bits, BITS);
    let lido: Vec<_> = info
        .sent
        .iter()
        .map(|e| (e.signal.as_str(), e.times))
        .collect();
    assert_eq!(lido, [("caiu", 2)]);
    assert!(
        envios_do_objeto(&motion, BITS, false).is_none(),
        "parado: sem cartão"
    );
    assert!(
        envios_do_objeto(&motion, BITS + 1, true).is_none(),
        "outro objecto"
    );
    motion.envios.corrida(false);
    motion.envios.corrida(true);
    let zero = envios_do_objeto(&motion, BITS, true).expect("cartão");
    assert_eq!(zero.sent[0].times, 0, "o Play seguinte começa do zero");
    motion.doc.graph.set_text_param(s, send::NAME_KEY, "  ");
    assert!(
        envios_do_objeto(&motion, BITS, true).is_none(),
        "sem nome não envia"
    );
}
