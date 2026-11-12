//! O censo conta cada linha da tabela no papel certo, por nome exato, e os scripts Luau.

use ph2d_ecs::World;
use ph2d_ecs::camera_shake::{ShakeEmitter, ShakeSource};
use ph2d_ecs::counter_watch::{CounterWatch, CounterWatchRow};
use ph2d_ecs::factory::Factory;
use ph2d_ecs::hud::UiButton;
use ph2d_ecs::lifetime::Lifetime;
use ph2d_ecs::particle_emitter::ParticleEmitter;
use ph2d_ecs::signal_actions::{SignalAction, SignalActions};
use ph2d_ecs::signal_on_action::{ActionTriggerRow, SignalOnAction};
use ph2d_ecs::sprite_anim::{AnimationTag, SpriteAnimations};
use ph2d_ecs::state_machine::{MachineState, StateMachine, Transition};
use ph2d_ecs::timer::{Timer, Timers};
use ph2d_ecs::weapon::WeaponFire;
use ph2d_ecs::{FrameMarker, SpriteAnimMarkers};
use ph2d_physics_ecs::{
    AnimRule, AnimRules, AnimWhen, Health, NavAgent, NavLink, RaySignals, SignalOnHit,
    SignalOnLeave,
};
use ph2d_script::{LuauScript, ProjectScripts};
use ph2d_script_doc::ScriptLibrary;
use ph2d_signal_census::{CAMPOS, Fora, Hint, Role, SignalCensus};

/// O censo só do mundo (sem documentos de fora).
fn census(w: &World) -> SignalCensus {
    ph2d_signal_census::census(w, &Fora::default())
}

/// O nome distinto de cada linha da tabela: `Componente.campo`.
fn n(comp: &str, field: &str) -> String {
    format!("{comp}.{field}")
}

/// Um mundo com TODOS os componentes da tabela, cada campo com o nome da sua linha.
fn mundo_com_tudo() -> World {
    let mut w = World::new();
    w.spawn(SignalActions(vec![SignalAction {
        on: n("SignalActions", "on"),
        ..Default::default()
    }]));
    w.spawn(WeaponFire {
        on_signal: n("WeaponFire", "on_signal"),
        reload_on: n("WeaponFire", "reload_on"),
        on_fire: n("WeaponFire", "on_fire"),
        on_empty: n("WeaponFire", "on_empty"),
        on_reloaded: n("WeaponFire", "on_reloaded"),
        ..Default::default()
    });
    w.spawn(Factory {
        on_signal: n("Factory", "on_signal"),
        on_spawned: n("Factory", "on_spawned"),
        on_exhausted: n("Factory", "on_exhausted"),
        ..Default::default()
    });
    w.spawn(StateMachine {
        states: vec![MachineState {
            on_enter: n("StateMachine", "states.on_enter"),
            on_exit: n("StateMachine", "states.on_exit"),
            ..Default::default()
        }],
        transitions: vec![Transition {
            on: n("StateMachine", "transitions.on"),
            ..Default::default()
        }],
        ..Default::default()
    });
    w.spawn(ShakeEmitter(vec![ShakeSource {
        on: n("ShakeEmitter", "on"),
        ..Default::default()
    }]));
    w.spawn(ParticleEmitter {
        start_on: n("ParticleEmitter", "start_on"),
        stop_on: n("ParticleEmitter", "stop_on"),
        restart_on: n("ParticleEmitter", "restart_on"),
        finished_signal: n("ParticleEmitter", "finished_signal"),
        ..Default::default()
    });
    w.spawn(AnimRules(vec![AnimRule {
        when: AnimWhen::Signal(n("AnimRules", "when.Signal")),
        play: "idle".into(),
    }]));
    w.spawn(SignalOnAction(vec![ActionTriggerRow {
        signal: n("SignalOnAction", "signal"),
        ..Default::default()
    }]));
    w.spawn(CounterWatch(vec![CounterWatchRow {
        signal: n("CounterWatch", "signal"),
        ..Default::default()
    }]));
    w.spawn(Timers(vec![Timer {
        signal: n("Timers", "signal"),
        ..Default::default()
    }]));
    w.spawn(Lifetime {
        on_death: n("Lifetime", "on_death"),
        ..Default::default()
    });
    w.spawn(Health {
        on_damage: n("Health", "on_damage"),
        on_heal: n("Health", "on_heal"),
        on_death: n("Health", "on_death"),
        ..Default::default()
    });
    w.spawn(NavAgent {
        on_arrived: n("NavAgent", "on_arrived"),
        on_no_path: n("NavAgent", "on_no_path"),
        on_stuck: n("NavAgent", "on_stuck"),
        ..Default::default()
    });
    w.spawn(NavLink {
        on_crossed: n("NavLink", "on_crossed"),
        ..Default::default()
    });
    w.spawn(RaySignals {
        on_enter: n("RaySignals", "on_enter"),
        on_exit: n("RaySignals", "on_exit"),
    });
    w.spawn(SignalOnHit(n("SignalOnHit", "0")));
    w.spawn(SignalOnLeave(n("SignalOnLeave", "0")));
    let mut tag = AnimationTag::new("walk", 0, 1);
    tag.signal_on_finish = n("SpriteAnimations", "signal_on_finish");
    tag.signal_on_loop = n("SpriteAnimations", "signal_on_loop");
    let mut anims = SpriteAnimations::new();
    anims.0.push(tag);
    w.spawn(anims);
    w.spawn(SpriteAnimMarkers(vec![FrameMarker {
        tag: "walk".into(),
        step: 0,
        signal: n("SpriteAnimMarkers", "signal"),
    }]));
    w.spawn(UiButton {
        signal: n("UiButton", "signal"),
        ..Default::default()
    });
    w
}

#[test]
fn cada_linha_da_tabela_e_contada_no_papel_dela() {
    let c = census(&mundo_com_tudo());
    // Controlo positivo: um nome por linha, nem mais nem menos.
    assert_eq!(
        c.names.len(),
        CAMPOS.len(),
        "nomes achados: {:?}",
        c.names.keys()
    );
    assert!(CAMPOS.len() >= 37, "a tabela encolheu: {}", CAMPOS.len());
    for campo in CAMPOS {
        let e = c.ends(&n(campo.component, campo.field));
        let (emit, listen) = match campo.role {
            Role::Emit => (1, 0),
            Role::Listen => (0, 1),
        };
        assert_eq!(
            (e.emit, e.listen),
            (emit, listen),
            "{}.{}",
            campo.component,
            campo.field
        );
    }
}

/// A suspeita (d) do handoff OUTLINE_E_F3 §11: «N listen» contaria a própria caixa. Cada linha
/// sozinha com o seu nome: a caixa dela tem de dizer `Nobody`, nunca `Count(1)` de si mesma.
#[test]
fn nenhuma_caixa_e_a_sua_propria_outra_ponta() {
    let c = census(&mundo_com_tudo());
    for campo in CAMPOS {
        let nome = n(campo.component, campo.field);
        assert_eq!(
            c.hint(&nome, campo.role),
            Hint::Nobody,
            "{nome} conta-se a si mesma"
        );
    }
}

#[test]
fn letra_trocada_e_ninguem_e_o_proprio_campo_nao_se_conta() {
    let mut w = World::new();
    w.spawn(SignalOnHit("coin_taken".into()));
    w.spawn(SignalOnHit("coin_takn".into()));
    w.spawn(SignalActions(vec![SignalAction {
        on: "coin_taken".into(),
        ..Default::default()
    }]));
    let c = census(&w);
    assert_eq!(c.hint("coin_taken", Role::Emit), Hint::Count(1));
    assert_eq!(c.hint("coin_takn", Role::Emit), Hint::Nobody);
    assert_eq!(c.hint("Coin_Taken", Role::Emit), Hint::Nobody);
    // A caixa é guardada como foi escrita: maiúsculas casam com maiúsculas.
    let mut m = World::new();
    m.spawn(SignalOnHit("Door".into()));
    m.spawn(SignalActions(vec![SignalAction {
        on: "Door".into(),
        ..Default::default()
    }]));
    assert_eq!(census(&m).hint("Door", Role::Emit), Hint::Count(1));
    // O ouvinte vê os DOIS emissores do nome exato, e nenhum de si mesmo.
    assert_eq!(c.hint("coin_taken", Role::Listen), Hint::Count(1));
    // Um emissor sozinho não é a sua própria outra ponta.
    let mut s = World::new();
    s.spawn(SignalOnHit("solo".into()));
    assert_eq!(census(&s).hint("solo", Role::Emit), Hint::Nobody);
}

fn com_script(text: &str) -> World {
    let mut w = World::new();
    let mut lib = ScriptLibrary::new();
    let id = lib.insert("s", text);
    w.insert_resource(ProjectScripts::new(lib));
    w.spawn(LuauScript::library(id));
    w
}

#[test]
fn scripts_luau_literais_contam_e_o_resto_e_a_script_may() {
    let c = census(&com_script(
        "ph2d.emit(\"door_open\")\nph2d.connect(other, 'bell', function(self) end)\n",
    ));
    assert_eq!(c.ends("door_open").emit, 1);
    assert_eq!(c.ends("bell").listen, 1);
    assert!(!c.dynamic_emit && !c.dynamic_listen);

    let d = census(&com_script("local n = \"x\"\nph2d.emit(n)\n"));
    assert!(d.dynamic_emit);
    assert_eq!(d.hint("anything", Role::Listen), Hint::ScriptMay);
    assert_eq!(d.hint("anything", Role::Emit), Hint::Nobody);

    let o = census(&com_script("function on_signal(self, s, from) end"));
    assert_eq!(o.hint("anything", Role::Emit), Hint::ScriptMay);
}

/// O custo por quadro numa cena de 200 objectos com sinais (cada um com 9 dos componentes da
/// tabela) e um script. Medido à mão: `cargo test --profile smoke -p ph2d-signal-census
/// mede_o_custo -- --ignored --nocapture` (o mínimo de 200 corridas).
#[test]
#[ignore = "medição, não lei"]
fn mede_o_custo_do_censo_por_quadro() {
    let mut w = com_script("ph2d.emit(\"a\")\nph2d.connect(o, \"b\", f)\n");
    for i in 0..200 {
        w.spawn((
            SignalOnHit(format!("hit_{i}")),
            SignalActions(vec![SignalAction {
                on: format!("hit_{}", i / 2),
                ..Default::default()
            }]),
            Health {
                on_death: format!("died_{i}"),
                ..Default::default()
            },
            Timers(vec![Timer {
                signal: "tick".into(),
                ..Default::default()
            }]),
            RaySignals {
                on_enter: "seen".into(),
                on_exit: String::new(),
            },
            WeaponFire::default(),
            ParticleEmitter::default(),
            StateMachine::default(),
            SpriteAnimMarkers::default(),
        ));
    }
    // O mesmo mundo sem objectos (só as consultas), intercalado com o cheio.
    let vazio = com_script("");
    let (mut cheio, mut so_consultas) = (std::time::Duration::MAX, std::time::Duration::MAX);
    let mut digital = std::time::Duration::MAX;
    for _ in 0..200 {
        let t = std::time::Instant::now();
        let c = std::hint::black_box(census(&w));
        cheio = cheio.min(t.elapsed());
        assert!(c.names.len() > 400);
        let t = std::time::Instant::now();
        std::hint::black_box(census(&vazio));
        so_consultas = so_consultas.min(t.elapsed());
        let t = std::time::Instant::now();
        std::hint::black_box(ph2d_signal_census::fingerprint(&w, &Fora::default()));
        digital = digital.min(t.elapsed());
    }
    println!(
        "censo de 200 objectos: mínimo {cheio:?}; mundo vazio: {so_consultas:?}; impressão digital: {digital:?}"
    );
}

#[test]
fn a_porta_do_quadro_so_publica_quando_um_nome_muda() {
    let mut w = World::new();
    let e = w.spawn(SignalOnHit("a".into())).id();
    let mut vezes = 0;
    ph2d_signal_census::when_changed(&w, &Fora::default(), |_| vezes += 1);
    ph2d_signal_census::when_changed(&w, &Fora::default(), |_| vezes += 1);
    assert_eq!(vezes, 1, "o mesmo mundo não republica");
    w.entity_mut(e).insert(SignalOnHit("b".into()));
    ph2d_signal_census::when_changed(&w, &Fora::default(), |c| {
        vezes += 1;
        assert_eq!(c.ends("b").emit, 1);
    });
    assert_eq!(vezes, 2, "um nome novo republica");
}
