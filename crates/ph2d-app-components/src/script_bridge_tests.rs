//! A ponte dos scripts contra o LEDGER de verdade — o `Ctrl+Z` e o rebobinar são o que ela decide,
//! então os gates andam com a `settle` a correr a cada quadro, como no produto.

use ph2d_ecs::{SimWorld, StableId, Transform};
use ph2d_preview_drive::{Driver, PreviewDrive};
use ph2d_script::{LuauScript, ProjectScripts, ScriptHost};

use super::{frame, rewind};

/// Anda `+1` por tique durante `limite` tiques, e depois PÁRA — o condutor que acaba de mexer.
const ANDA_E_PARA: &str = r#"
ph2d.property("limite", 3)
function init(self) self.n = 0 end
function update(self, dt)
  if self.n < self.limite then
    self.n = self.n + 1
    ph2d.set(self.id, "x", ph2d.get(self.id, "x") + 1)
  end
end
"#;

struct Cena {
    sim: SimWorld,
    host: ScriptHost,
    drive: PreviewDrive,
    e: ph2d_ecs::Entity,
}

fn cena(source: &str) -> Cena {
    let mut sim = SimWorld::new();
    let mut t = Transform::IDENTITY;
    t.translation.x = 10.0;
    let id = sim
        .world_mut()
        .get_resource_or_init::<ProjectScripts>()
        .insert("s.luau", source);
    let e = sim
        .world_mut()
        .spawn((t, StableId(1), LuauScript::library(id)))
        .id();
    Cena {
        sim,
        host: ScriptHost::new().expect("vm"),
        drive: PreviewDrive::default(),
        e,
    }
}

impl Cena {
    /// Um quadro do produto: a ponte, e depois a `settle` do `post_frame_undo`.
    fn quadro(&mut self, playing: bool) -> super::ScriptFrame {
        let f = frame(
            &mut self.host,
            &mut self.sim,
            &mut self.drive,
            playing,
            1,
            1.0 / 60.0,
            1.0 / 60.0,
            &[],
            &mut ph2d_script::NoDoors,
        );
        self.drive.settle();
        f
    }

    fn x(&self) -> f32 {
        self.sim
            .world()
            .get::<Transform>(self.e)
            .expect("pose")
            .translation
            .x
    }

    /// O `x` que a CAPTURA (undo + save) vê — o autorado.
    fn x_documento(&mut self) -> f32 {
        let live = self.drive.substitute_authored(&mut self.sim);
        let x = self.x();
        PreviewDrive::restore_live(&mut self.sim, &live);
        x
    }
}

#[test]
fn parado_nada_corre_e_nada_e_conduzido() {
    let mut c = cena(ANDA_E_PARA);
    let f = c.quadro(false);
    assert_eq!(f.calls, 0);
    assert_eq!(c.x(), 10.0);
    assert!(c.drive.is_empty());
    assert_eq!(c.host.scene().live_count(), 0);
}

/// ⭐⭐⭐ **A corrida vê-se e não se guarda; rebobinar devolve a pose do artista.**
#[test]
fn a_corrida_move_o_objecto_e_o_documento_fica_onde_o_artista_o_pos() {
    let mut c = cena(ANDA_E_PARA);
    c.quadro(true);
    c.quadro(true);
    assert_eq!(c.x(), 12.0, "o mundo mostra a corrida");
    assert_eq!(c.x_documento(), 10.0, "a captura vê o autorado");
    assert_eq!(rewind(&mut c.host, &mut c.sim, &mut c.drive), 1);
    assert_eq!(c.x(), 10.0, "rebobinar devolve a pose");
    assert!(c.drive.is_empty());
    c.quadro(true);
    assert_eq!(
        c.x(),
        11.0,
        "e a corrida seguinte parte do zero — o self renasceu"
    );
}

/// ⛔⛔ **A linha do meio da tabela:** um script que PAROU de mexer continua a conduzir. Sem ela a
/// `settle` promovia a pose da corrida a documento, e o rebobinar ficava sem destino.
#[test]
fn um_script_que_parou_de_mexer_nao_vira_documento() {
    let mut c = cena(ANDA_E_PARA);
    for _ in 0..10 {
        c.quadro(true);
    }
    assert_eq!(c.x(), 13.0, "andou três e parou");
    assert_eq!(
        c.x_documento(),
        10.0,
        "sete quadros parado e o documento continua o do artista"
    );
    rewind(&mut c.host, &mut c.sim, &mut c.drive);
    assert_eq!(c.x(), 10.0);
}

#[test]
fn pausado_a_meio_a_pose_fica_e_o_rebobinar_ainda_a_devolve() {
    let mut c = cena(ANDA_E_PARA);
    c.quadro(true);
    for _ in 0..5 {
        let f = c.quadro(false);
        assert_eq!(f.calls, 0);
    }
    assert_eq!(c.x(), 11.0, "congelado");
    assert_eq!(c.x_documento(), 10.0);
    rewind(&mut c.host, &mut c.sim, &mut c.drive);
    assert_eq!(c.x(), 10.0);
}

/// ⚠️ **Um arrasto feito na PAUSA é a outra mão**, e fica como o novo autorado.
#[test]
fn arrastar_na_pausa_muda_para_onde_o_rebobinar_volta() {
    let mut c = cena(ANDA_E_PARA);
    c.quadro(true);
    c.quadro(false);
    c.sim
        .world_mut()
        .get_mut::<Transform>(c.e)
        .expect("pose")
        .translation
        .x = 50.0;
    c.quadro(false);
    assert_eq!(c.x_documento(), 50.0, "o arrasto é documento");
    rewind(&mut c.host, &mut c.sim, &mut c.drive);
    assert_eq!(c.x(), 50.0, "e é para lá que se volta");
}

#[test]
fn um_script_que_nunca_mexe_nao_abre_conducao_nenhuma() {
    let mut c = cena("function update(self, dt) end");
    c.quadro(true);
    c.quadro(true);
    assert!(
        !c.drive
            .drivers_of(c.e.to_bits())
            .contains(&Driver::ScriptPose),
        "`still_driving` nunca cria uma entrada"
    );
}

#[test]
fn os_sinais_emitidos_saem_com_quem_os_emitiu() {
    let mut c = cena(r#"function update(self, dt) ph2d.emit("bip") end"#);
    let f = c.quadro(true);
    assert_eq!(f.emitted, vec![(c.e.to_bits(), "bip".to_owned())]);
}

#[test]
fn a_falha_sai_uma_vez_com_a_mensagem() {
    let mut c = cena(r#"function update(self, dt) error("partiu") end"#);
    let f = c.quadro(true);
    assert_eq!(f.failed.len(), 1);
    assert!(
        f.failed[0].1.message.contains("partiu"),
        "{}",
        f.failed[0].1
    );
    assert!(
        c.quadro(true).failed.is_empty(),
        "parado, não repete a queixa"
    );
}

#[test]
fn dois_tiques_num_quadro_sao_duas_chamadas() {
    let mut c = cena(ANDA_E_PARA);
    let f = frame(
        &mut c.host,
        &mut c.sim,
        &mut c.drive,
        true,
        2,
        1.0 / 60.0,
        1.0 / 60.0,
        &[],
        &mut ph2d_script::NoDoors,
    );
    assert_eq!(f.calls, 3, "init + dois update");
    assert_eq!(c.x(), 12.0);
}

/// ⭐ **`process` uma vez por QUADRO, com o dt do quadro** — três passos fixos num quadro são três
/// `physics` e UM `process`; parado, nenhum (GameEngineFuncional N0).
#[test]
fn tres_tiques_num_quadro_sao_um_process() {
    let mut c = cena(
        r#"
function physics(self) self.p = (self.p or 0) + 1 end
function process(self, dt)
  self.q = (self.q or 0) + 1
  ph2d.set(self.id, "x", self.q * 100 + self.p)
  ph2d.set(self.id, "y", dt)
end
"#,
    );
    for playing in [true, false] {
        frame(
            &mut c.host,
            &mut c.sim,
            &mut c.drive,
            playing,
            3,
            1.0 / 60.0,
            0.04,
            &[],
            &mut ph2d_script::NoDoors,
        );
    }
    assert_eq!(
        c.x(),
        103.0,
        "um process depois de três physics; parado, nenhum"
    );
    let y = c.sim.world().get::<Transform>(c.e).unwrap().translation.y;
    assert_eq!(y, 0.04, "o dt do quadro");
}

/// ⭐⭐ **`ph2d.spawn("Mestre")` pela porta da FÁBRICA** (CodeEditor item 6): a mesma instanciação,
/// a mesma marca — a cópia é da corrida, nasce onde o script disse, e a varredura do rebobinar
/// tira-a; o `ph2d.despawn` dela sai como uma morte para o dreno único.
#[test]
fn spawn_pela_porta_da_fabrica_e_a_copia_e_da_corrida() {
    let mut reg = ph2d_ecs::scene::ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    let (mut vec_scene, mut vec_entities) = crate::instance_docs::empty_docs();
    let mut sim = SimWorld::new();
    crate::instance_smoke::spawn_master(&mut sim);
    let id = sim
        .world_mut()
        .get_resource_or_init::<ProjectScripts>()
        .insert(
            "arma.luau",
            r#"
function init(self)
  self.c = ph2d.spawn("Ragdoll", 3, 4)
end
function update(self)
  if self.n then ph2d.despawn(self.c) end
  self.n = true
end
"#,
        );
    let arma = sim
        .world_mut()
        .spawn((Transform::IDENTITY, StableId(9), LuauScript::library(id)))
        .id();
    let mut host = ScriptHost::new().expect("vm");
    let mut drive = PreviewDrive::default();
    let mut portas = super::ScriptDoors {
        registry: &reg,
        tags: &ph2d_tags::TagTree::new(),
        docs: crate::instance_docs::OwnedDocs {
            vec_scene: &mut vec_scene,
            vec_entities: &mut vec_entities,
        },
        tick: 5,
        images: crate::script_image::ImageLibrary::none(),
    };
    let f = frame(
        &mut host,
        &mut sim,
        &mut drive,
        true,
        1,
        1.0 / 60.0,
        1.0 / 60.0,
        &[],
        &mut portas,
    );
    assert!(f.failed.is_empty(), "{:?}", f.failed);
    let w = sim.world_mut();
    let copias: Vec<(ph2d_ecs::Entity, ph2d_ecs::Spawned, Transform)> = w
        .query::<(ph2d_ecs::Entity, &ph2d_ecs::Spawned, &Transform)>()
        .iter(w)
        .map(|(e, s, t)| (e, *s, *t))
        .collect();
    assert_eq!(copias.len(), 1, "uma cópia do mestre");
    let (copia, marca, pose) = copias[0];
    assert_eq!(marca.by, 9, "posta pelo objecto do script");
    assert_eq!(marca.born_tick, 5);
    assert_eq!((pose.translation.x, pose.translation.y), (3.0, 4.0));
    assert!(
        ph2d_ecs::is_transient(sim.world(), copia),
        "não é documento"
    );
    assert!(drive.is_empty() || !drive.driven_by(Driver::ScriptPose).contains(&copia));
    let f = frame(
        &mut host,
        &mut sim,
        &mut drive,
        true,
        1,
        1.0 / 60.0,
        1.0 / 60.0,
        &[],
        &mut portas,
    );
    assert_eq!(
        f.despawned,
        vec![copia],
        "o despawn sai para o dreno das mortes"
    );
    let _ = arma;
    assert_eq!(
        crate::factory_bridge::sweep_spawned(&mut sim),
        1,
        "o rebobinar varre-a"
    );
}

/// ⛔ **Achado ao ler (20/10):** um `ph2d.set` num objecto SEM script escrevia, mas só quem tinha
/// script era declarado ao ledger — a pose da corrida virava DOCUMENTO e o rebobinar não a desfazia.
#[test]
fn set_num_objecto_sem_script_volta_ao_rebobinar() {
    let mut c = cena(
        r#"
function update(self)
  local p = ph2d.find_by_name("Pedra")
  ph2d.set(p, "x", ph2d.get(p, "x") + 1)
end
"#,
    );
    let mut t = Transform::IDENTITY;
    t.translation.x = 10.0;
    let pedra = c
        .sim
        .world_mut()
        .spawn((t, ph2d_ecs::Name::new("Pedra"), StableId(2)))
        .id();
    c.quadro(true);
    c.quadro(true);
    let x = |c: &Cena| {
        c.sim
            .world()
            .get::<Transform>(pedra)
            .expect("pose")
            .translation
            .x
    };
    assert_eq!(x(&c), 12.0, "a corrida move a pedra");
    let live = c.drive.substitute_authored(&mut c.sim);
    assert_eq!(x(&c), 10.0, "a captura vê o autorado");
    PreviewDrive::restore_live(&mut c.sim, &live);
    rewind(&mut c.host, &mut c.sim, &mut c.drive);
    assert_eq!(x(&c), 10.0, "rebobinar devolve a pedra");
}

/// ⭐ **O depurador congela o jogo**: um ponto de parada batido num quadro deixa os seguintes sem
/// tique (a pose fica), e o Continue acaba o `update` parado e o jogo volta a andar.
#[test]
fn o_ponto_de_parada_congela_e_o_continue_retoma() {
    const SEMPRE: &str = "function update(self, dt)\n  ph2d.set(self.id, \"x\", ph2d.get(self.id, \"x\") + 1)\nend\n";
    let mut c = cena(SEMPRE);
    let id = id_do_script(&c);
    // Um vizinho com OUTRO script, sem ponto de parada: o jogo inteiro congela, não só quem parou.
    let outro = c
        .sim
        .world_mut()
        .get_resource_or_init::<ProjectScripts>()
        .insert("outro.luau", format!("{SEMPRE}-- outro"));
    let vizinho = c
        .sim
        .world_mut()
        .spawn((Transform::IDENTITY, StableId(2), LuauScript::library(outro)))
        .id();
    let vx = |c: &Cena| {
        c.sim
            .world()
            .get::<Transform>(vizinho)
            .expect("pose")
            .translation
            .x
    };
    c.host.set_breakpoints(id, vec![2]);
    assert_eq!(
        super::retem(Some(&c.host), 0.25),
        0.25,
        "solto, o tempo passa"
    );
    c.quadro(true);
    assert!(c.host.is_held(), "o 1.º tique bateu na linha 2");
    assert_eq!(c.x(), 10.0, "parado ANTES da escrita");
    assert_eq!(
        super::retem(Some(&c.host), 0.25),
        0.0,
        "retido, zero tempo ao acumulador"
    );
    let relogio = crate::motores::Relogio::do_quadro(true, 2, 0.5).com_quadro(0.3);
    let r = relogio.retido_por(Some(&c.host));
    assert!(
        !r.playing && r.ticks == 0 && r.frame_dt == 0.0,
        "os motores veem a pausa"
    );
    assert_eq!(
        relogio.retido_por(None).ticks,
        2,
        "sem host, o relógio passa como veio"
    );
    let v = vx(&c);
    for _ in 0..3 {
        let f = c.quadro(true);
        assert_eq!(f.calls, 0, "retido, nenhum gancho corre");
        assert!(c.host.is_held());
        assert_eq!(c.x(), 10.0, "retido, a pose fica");
        assert_eq!(vx(&c), v, "retido, o vizinho também não anda");
    }
    c.host.set_breakpoints(id, Vec::new());
    c.host.request_continue();
    c.quadro(true);
    assert!(!c.host.is_held(), "o Continue soltou-o");
    assert_eq!(
        c.x(),
        12.0,
        "o `update` parado acabou (+1) e o tique deste quadro correu (+1)"
    );
    c.quadro(true);
    assert_eq!(c.x(), 13.0, "o jogo voltou a andar");
    assert!(vx(&c) > v, "e o vizinho também");
}

fn id_do_script(c: &Cena) -> ph2d_script::ScriptId {
    match c.sim.world().get::<LuauScript>(c.e).expect("script").source {
        ph2d_script::ScriptSource::Library(id) => id,
        _ => panic!("fixture: o script é da biblioteca"),
    }
}

/// ⭐ **Parou a meio do quadro: o resto do jogo anda só os tiques que correram**, e os sinais que a
/// ponte já leu não se perdem — ouvem-se no quadro do Continue, uma vez.
#[test]
fn depurador_parado_a_meio_do_quadro_conta_so_os_tiques_que_correram() {
    const SRC: &str = "local n = 0\nfunction update(self, dt)\n  n = n + 1\n  if n == 2 then\n    local stop = 1\n  end\nend\nfunction on_signal(self, name, de)\n  ph2d.set(self.id, \"y\", ph2d.get(self.id, \"y\") + 1)\nend\n";
    let mut c = cena(SRC);
    let id = id_do_script(&c);
    c.host.set_breakpoints(id, vec![5]);
    let y = |c: &Cena| {
        c.sim
            .world()
            .get::<Transform>(c.e)
            .expect("pose")
            .translation
            .y
    };
    let quadro = |c: &mut Cena, ticks: u32, heard: &[(&str, Option<u64>)]| {
        let f = frame(
            &mut c.host,
            &mut c.sim,
            &mut c.drive,
            true,
            ticks,
            0.5,
            2.0,
            heard,
            &mut ph2d_script::NoDoors,
        );
        c.drive.settle();
        f
    };
    let f = quadro(&mut c, 4, &[("ping", None)]);
    assert!(c.host.is_held());
    assert_eq!(
        f.ticks_run, 2,
        "o 2.º tique parou: os outros 2 não correram"
    );
    let r = crate::motores::Relogio::do_quadro(true, 4, 0.5).ate(f.ticks_run);
    assert_eq!((r.ticks, r.frame_dt), (2, 1.0), "os motores andam só esses");
    assert_eq!(y(&c), 0.0, "o sinal não se ouviu com o jogo parado");
    c.host.set_breakpoints(id, Vec::new());
    c.host.request_continue();
    quadro(&mut c, 1, &[]);
    assert!(!c.host.is_held());
    assert_eq!(
        y(&c),
        1.0,
        "o sinal guardado ouviu-se no quadro do Continue"
    );
    quadro(&mut c, 1, &[]);
    assert_eq!(y(&c), 1.0, "uma vez só");
}
