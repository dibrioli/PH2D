//! ⭐⭐ **A API que chega à cena** (CodeEditor item 6) — headless, sobre a VM real e scripts da
//! biblioteca do projeto. Plano e oráculo: `docs/CodeEditor/05_api_que_funciona.md` (Godot 4.7.2
//! corrido, `ferramentas/oraculo_api/saida.txt`). Cada portão nomeia o § do plano.

use ph2d_ecs::{
    ChildOf, Entity, MasterRoot, Name, RootOrder, SimWorld, Spawned, StableId, Transform, World,
};
use ph2d_input::{Key, KeyboardState, MouseButton, MouseState};

use crate::component::{LuauScript, ProjectScripts};
use crate::lane::SceneDoors;
use crate::scene::{SceneReport, ScriptInfo};
use crate::{ConsoleKind, ScriptHost};

const DT: f64 = 1.0 / 60.0;

fn script(world: &mut World, name: &str, text: &str) -> crate::ScriptId {
    let mut lib = world.get_resource_or_init::<ProjectScripts>();
    lib.by_name(name)
        .unwrap_or_else(|| lib.insert(name, text.to_owned()))
}

fn at(x: f32, y: f32) -> Transform {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    t.translation.y = y;
    t
}

/// Um objecto do DOCUMENTO com script.
fn actor(world: &mut World, sid: u64, name: &str, text: &str) -> Entity {
    let id = script(world, name, text);
    world
        .spawn((Transform::IDENTITY, StableId(sid), LuauScript::library(id)))
        .id()
}

/// Um objecto da CORRIDA (transiente) com script.
fn runtime_actor(world: &mut World, sid: u64, name: &str, text: &str) -> Entity {
    let e = actor(world, sid, name, text);
    world.entity_mut(e).insert(Spawned::default());
    e
}

fn tick(host: &mut ScriptHost, sim: &mut SimWorld) -> SceneReport {
    host.scene_sync(sim.world_mut());
    let r = host.scene_tick(sim, DT);
    for (e, d) in &r.failed {
        eprintln!("parou {e:?}: {}", d.message);
    }
    r
}

fn pose(world: &World, e: Entity) -> Transform {
    *world.get::<Transform>(e).expect("pose")
}

fn console(host: &ScriptHost) -> Vec<(String, bool)> {
    host.drain_console()
        .into_iter()
        .map(|l| (l.text, matches!(l.kind, ConsoleKind::Warning(_))))
        .collect()
}

// ── §3.1 (a) ph2d.wait ───────────────────────────────────────────────────────────────────────

/// Godot: retoma no fim do tique em que o tempo chega, contando o tique em que começou; uma espera
/// começada na passagem das retomas conta a partir do seguinte. Microssegundos inteiros: `0.05` a
/// 60 Hz são TRÊS tiques (o Godot leva quatro pelo resíduo do `double`).
#[test]
fn uma_espera_retoma_no_tique_em_que_o_tempo_chega() {
    let mut sim = SimWorld::new();
    let e = actor(
        sim.world_mut(),
        1,
        "espera.luau",
        r#"
function init(self)
  self.n = 0
  ph2d.wait(1/60)
  ph2d.set(self.id, "x", self.n)
  local passou = ph2d.wait(0.05)
  ph2d.set(self.id, "y", self.n)
  ph2d.set(self.id, "scale_x", passou)
  ph2d.wait(0.1)
  ph2d.set(self.id, "rotation", self.n)
end
function update(self, dt)
  self.n = self.n + 1
end
"#,
    );
    let mut host = ScriptHost::new().expect("vm");
    for _ in 0..12 {
        assert!(tick(&mut host, &mut sim).failed.is_empty());
    }
    let p = pose(sim.world(), e);
    assert_eq!(
        p.translation.x, 1.0,
        "wait(1/60) no init do tique 1 retoma no fim do tique 1"
    );
    assert_eq!(
        p.translation.y, 4.0,
        "wait(0.05) começado na retoma: tiques 2, 3, 4"
    );
    assert!(
        (f64::from(p.scale.x) - 0.050_001).abs() < 1e-6,
        "devolve o tempo que passou"
    );
    assert_eq!(p.rotation, 10.0, "wait(0.1) de 5 a 10");
    assert_eq!(host.scene().waiting_count(), 0);
}

/// Godot: o gancho de cada quadro continua a ser chamado enquanto uma chamada anterior espera.
#[test]
fn o_gancho_continua_a_ser_chamado_enquanto_espera() {
    let mut sim = SimWorld::new();
    let e = actor(
        sim.world_mut(),
        1,
        "continua.luau",
        r#"
function update(self, dt)
  self.calls = (self.calls or 0) + 1
  ph2d.set(self.id, "x", self.calls)
  if self.calls == 1 then
    ph2d.wait(0.1)
    ph2d.set(self.id, "y", self.calls)
  end
end
"#,
    );
    let mut host = ScriptHost::new().expect("vm");
    for _ in 0..7 {
        tick(&mut host, &mut sim);
    }
    let p = pose(sim.world(), e);
    assert_eq!(p.translation.x, 7.0, "o update correu em todo tique");
    assert_eq!(
        p.translation.y, 6.0,
        "a espera de 0,1 s acabou no tique 6, com self.calls = 6"
    );
}

#[test]
fn rebobinar_e_apagar_esquecem_as_esperas() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    actor(
        w,
        1,
        "longa.luau",
        "function init(self) ph2d.wait(10) print('nunca') end",
    );
    let quem = runtime_actor(
        w,
        2,
        "morre.luau",
        "function init(self) ph2d.wait(0.05) print('depois') end\nfunction update(self) ph2d.despawn(self.id) end",
    );
    let mut host = ScriptHost::new().expect("vm");
    let r = tick(&mut host, &mut sim);
    assert_eq!(r.despawned, vec![quem]);
    assert_eq!(host.scene().waiting_count(), 2);
    sim.world_mut().entity_mut(quem).despawn();
    for _ in 0..5 {
        tick(&mut host, &mut sim);
    }
    assert_eq!(
        host.scene().waiting_count(),
        1,
        "a espera de quem saiu foi esquecida"
    );
    host.scene_rewind();
    assert_eq!(
        host.scene().waiting_count(),
        0,
        "rebobinar esquece as esperas"
    );
    assert!(
        console(&host)
            .iter()
            .all(|(t, _)| t != "depois" && t != "nunca")
    );
}

#[test]
fn esperar_fora_de_um_gancho_diz_porque() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let id = script(w, "topo.luau", "ph2d.wait(1)");
    w.spawn((Transform::IDENTITY, StableId(1), LuauScript::library(id)));
    let mut host = ScriptHost::new().expect("vm");
    tick(&mut host, &mut sim);
    let Some(ScriptInfo::Broken(d)) = host.scene_info(id) else {
        panic!("o topo que espera tem de ser recusado");
    };
    assert!(
        d.message.contains("only inside init, update or on_signal"),
        "{}",
        d.message
    );

    let e = actor(
        sim.world_mut(),
        2,
        "cede.luau",
        "function update(self) coroutine.yield(1) end",
    );
    tick(&mut host, &mut sim);
    let d = host.scene_failure(e).expect("ceder sem ph2d.wait pára");
    assert!(d.message.contains("use ph2d.wait"), "{}", d.message);
}

// ── §3.2 (b) find_by_name ────────────────────────────────────────────────────────────────────

#[test]
fn achar_por_nome_segue_a_lista_e_nao_ve_mestres() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let pasta = w
        .spawn((Transform::IDENTITY, Name::new("Pasta"), RootOrder(1)))
        .id();
    let fundo = w
        .spawn((Transform::IDENTITY, Name::new("Alvo"), ChildOf(pasta)))
        .id();
    w.spawn((Transform::IDENTITY, Name::new("Alvo"), RootOrder(2)));
    w.spawn((
        Transform::IDENTITY,
        Name::new("Bala"),
        MasterRoot,
        RootOrder(0),
    ));
    let e = actor(
        w,
        1,
        "acha.luau",
        r#"
function update(self)
  ph2d.set(self.id, "x", ph2d.find_by_name("Alvo"))
  ph2d.set(self.id, "y", ph2d.find_by_name("Bala") and 1 or 0)
  ph2d.set(self.id, "rotation", ph2d.find_by_name("Ninguem") and 1 or 0)
  local novo = ph2d.spawn()
  ph2d.set(self.id, "scale_x", ph2d.find_by_name("Object") == novo and 1 or 0)
end
"#,
    );
    let mut host = ScriptHost::new().expect("vm");
    tick(&mut host, &mut sim);
    let p = pose(sim.world(), e);
    #[allow(clippy::cast_precision_loss)]
    let fundo_id = fundo.to_bits() as f32;
    assert_eq!(
        p.translation.x, fundo_id,
        "o primeiro em pré-ordem (Godot): o filho da Pasta"
    );
    assert_eq!(
        p.translation.y, 0.0,
        "um mestre é receita, não objecto da cena"
    );
    assert_eq!(p.rotation, 0.0, "nome inexistente = nil");
    assert_eq!(p.scale.x, 1.0, "o que nasceu neste tique já se acha");
}

// ── §3.3 (c) spawn · despawn · attach_script ─────────────────────────────────────────────────

/// Uma porta de teste: a «cópia» de um mestre é um objecto com o nome e a pose dele, e `Spawned`.
struct Copia;

impl SceneDoors for Copia {
    fn instantiate(
        &mut self,
        sim: &mut SimWorld,
        master: Entity,
        _by: Entity,
    ) -> Result<Entity, String> {
        let world = sim.world_mut();
        let name = world
            .get::<Name>(master)
            .map(|n| n.as_str().to_owned())
            .unwrap_or_default();
        let pose = *world.get::<Transform>(master).expect("pose");
        Ok(world
            .spawn((pose, Name::new(name), Spawned::default()))
            .id())
    }

    fn tick(&self) -> u64 {
        7
    }
}

#[test]
fn spawn_devolve_um_id_que_ja_se_le_e_escreve() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    w.spawn((at(5.0, 6.0), Name::new("Bala"), MasterRoot, RootOrder(0)));
    let e = actor(
        w,
        1,
        "fabrica.luau",
        r#"
function init(self)
  local b = ph2d.spawn("Bala", 3, 4)
  ph2d.set(self.id, "x", ph2d.get(b, "x"))
  ph2d.set(b, "rotation", 2)
  local c = ph2d.spawn("Bala")
  ph2d.set(self.id, "y", ph2d.get(c, "y"))
  ph2d.set(self.id, "rotation", ph2d.spawn() and 1 or 0)
end
"#,
    );
    let mut host = ScriptHost::new().expect("vm");
    host.scene_sync(sim.world_mut());
    let r = host.scene_tick_with(&mut sim, DT, &mut Copia);
    assert!(r.failed.is_empty(), "{:?}", r.failed);
    let p = pose(sim.world(), e);
    assert_eq!(
        p.translation.x, 3.0,
        "a cópia já está onde nasceu, na linha seguinte"
    );
    assert_eq!(p.translation.y, 6.0, "sem x, y fica onde o mestre está");
    assert_eq!(p.rotation, 1.0);
    let w = sim.world_mut();
    let copias: Vec<(Entity, Transform)> = w
        .query_filtered::<(Entity, &Transform), bevy_ecs::query::With<Spawned>>()
        .iter(w)
        .map(|(e, t)| (e, *t))
        .collect();
    assert_eq!(copias.len(), 3, "duas cópias e um vazio, todos da corrida");
    assert!(
        copias
            .iter()
            .any(|(_, t)| t.translation.x == 3.0 && t.rotation == 2.0),
        "o set na cópia pousou"
    );
}

#[test]
fn spawn_de_um_mestre_que_nao_existe_para_o_objecto() {
    let mut sim = SimWorld::new();
    let e = actor(
        sim.world_mut(),
        1,
        "nada.luau",
        "function init(self) ph2d.spawn('Nope') end",
    );
    let mut host = ScriptHost::new().expect("vm");
    tick(&mut host, &mut sim);
    let d = host.scene_failure(e).expect("pára");
    assert!(
        d.message.contains("no master named 'Nope'"),
        "{}",
        d.message
    );
}

#[test]
fn despawn_tira_so_o_que_nasceu_na_corrida_e_recusa_o_resto_em_voz() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let doc = w
        .spawn((Transform::IDENTITY, Name::new("Casa"), RootOrder(1)))
        .id();
    let e = actor(
        w,
        1,
        "tira.luau",
        r#"
function init(self)
  local c = ph2d.spawn()
  ph2d.set(self.id, "x", ph2d.despawn(c) and 1 or 0)
  ph2d.set(self.id, "y", ph2d.find_by_name("Object") == c and 1 or 0)
  ph2d.set(self.id, "rotation", ph2d.despawn(ph2d.find_by_name("Casa")) and 1 or 0)
  ph2d.set(self.id, "scale_x", ph2d.despawn(123456789) and 1 or 0)
end
function update(self)
  ph2d.set(self.id, "scale_y", 3)
end
"#,
    );
    let mut host = ScriptHost::new().expect("vm");
    let r = tick(&mut host, &mut sim);
    assert!(
        r.failed.is_empty(),
        "a recusa não pára o objecto: {:?}",
        r.failed
    );
    assert_eq!(r.despawned.len(), 1);
    let p = pose(sim.world(), e);
    assert_eq!(p.translation.x, 1.0, "a cópia vai");
    assert_eq!(
        p.translation.y, 1.0,
        "e ainda se acha até ao fim do quadro (Godot)"
    );
    assert_eq!(p.rotation, 0.0, "um objecto do documento fica");
    assert_eq!(p.scale.x, 0.0, "um id de ninguém é false");
    assert_eq!(p.scale.y, 3.0, "o script continuou");
    assert!(sim.world().get_entity(doc).is_ok());
    let linhas = console(&host);
    assert!(
        linhas
            .iter()
            .any(|(t, aviso)| *aviso && t.contains("'Casa' is part of the scene")),
        "{linhas:?}"
    );
}

#[test]
fn attach_script_poe_o_script_e_o_init_corre_no_tique_seguinte() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    script(
        w,
        "segue.luau",
        "function init(self) ph2d.set(self.id, 'x', 9) end",
    );
    let doc = w
        .spawn((Transform::IDENTITY, Name::new("Casa"), RootOrder(1)))
        .id();
    let e = actor(
        w,
        1,
        "poe.luau",
        r#"
function init(self)
  self.c = ph2d.spawn()
  ph2d.set(self.id, "x", ph2d.attach_script(self.c, "segue.luau") and 1 or 0)
  ph2d.set(self.id, "y", ph2d.attach_script(ph2d.find_by_name("Casa"), "segue.luau") and 1 or 0)
end
"#,
    );
    let mut host = ScriptHost::new().expect("vm");
    tick(&mut host, &mut sim);
    let p = pose(sim.world(), e);
    assert_eq!((p.translation.x, p.translation.y), (1.0, 0.0));
    assert!(
        sim.world().get::<LuauScript>(doc).is_none(),
        "o documento não ganha script"
    );
    let w = sim.world_mut();
    let copia = w
        .query_filtered::<Entity, bevy_ecs::query::With<Spawned>>()
        .iter(w)
        .next()
        .expect("a cópia");
    assert_eq!(
        pose(sim.world(), copia).translation.x,
        0.0,
        "o init dela ainda não correu"
    );
    tick(&mut host, &mut sim);
    assert_eq!(
        pose(sim.world(), copia).translation.x,
        9.0,
        "corre no tique seguinte"
    );

    let mau = actor(
        sim.world_mut(),
        2,
        "mau.luau",
        "function init(self) ph2d.attach_script(self.id, 'nao_existe.luau') end",
    );
    tick(&mut host, &mut sim);
    let d = host
        .scene_failure(mau)
        .expect("um nome que não está no projecto pára");
    assert!(
        d.message.contains("no project script named"),
        "{}",
        d.message
    );
}

// ── §3.4 (d) get de qualquer objecto, e o set que fugia ao ledger ────────────────────────────

#[test]
fn get_le_qualquer_objecto_e_um_campo_errado_para() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let pedra = w
        .spawn((at(7.0, 8.0), Name::new("Pedra"), RootOrder(1)))
        .id();
    let e = actor(
        w,
        1,
        "le.luau",
        r#"
function update(self)
  local p = ph2d.find_by_name("Pedra")
  ph2d.set(self.id, "x", ph2d.get(p, "y"))
  ph2d.set(p, "x", 1)
end
"#,
    );
    let mut host = ScriptHost::new().expect("vm");
    let r = tick(&mut host, &mut sim);
    assert_eq!(
        pose(sim.world(), e).translation.x,
        8.0,
        "lê um objecto sem script"
    );
    assert_eq!(
        r.wrote_before,
        vec![(e, Transform::IDENTITY), (pedra, at(7.0, 8.0))],
        "o «antes» de cada objecto escrito — com ou sem script — para o ledger"
    );
    let mau = actor(
        sim.world_mut(),
        2,
        "campo.luau",
        "function update(self) ph2d.get(self.id, 'z') end",
    );
    tick(&mut host, &mut sim);
    assert!(
        host.scene_failure(mau)
            .expect("pára")
            .message
            .contains("unknown field 'z'")
    );
}

// ── §3.5 (e) teclado e rato ──────────────────────────────────────────────────────────────────

const TECLAS: &str = r#"
function update(self)
  self.n = (self.n or 0) + 1
  if ph2d.key_pressed("space") then ph2d.set(self.id, "x", self.n) end
  if ph2d.key_released("space") then ph2d.set(self.id, "y", self.n) end
  if ph2d.key_down("shift") then ph2d.set(self.id, "rotation", self.n) end
  if ph2d.mouse_pressed("left") then
    local mx, my = ph2d.mouse_position()
    ph2d.set(self.id, "scale_x", mx)
    ph2d.set(self.id, "scale_y", my)
  end
end
"#;

/// A borda vive no TIQUE: um toque no mesmo quadro dá pressed e released; dois tiques num quadro
/// veem-na uma vez; um quadro sem tique não a perde.
#[test]
fn as_bordas_vivem_no_tique() {
    let mut sim = SimWorld::new();
    let e = actor(sim.world_mut(), 1, "teclas.luau", TECLAS);
    let mut host = ScriptHost::new().expect("vm");
    let space = Key(0x20);
    let mut kb = KeyboardState::new();
    let mut mouse = MouseState::default();
    // Quadro 1: um toque no espaço, o shift direito e um clique — mas NENHUM tique.
    kb.begin_frame();
    kb.handle_key_down(space);
    kb.handle_key_up(space);
    kb.handle_key_down(Key(0xF711));
    mouse.begin_frame();
    mouse.handle_button_down(MouseButton::Left);
    host.devices().feed(&kb, &mouse, Some([2.5, -1.0]));
    // Quadro 2: dois tiques.
    kb.begin_frame();
    mouse.begin_frame();
    host.devices().feed(&kb, &mouse, Some([2.5, -1.0]));
    tick(&mut host, &mut sim);
    tick(&mut host, &mut sim);
    let p = pose(sim.world(), e);
    assert_eq!(
        p.translation.x, 1.0,
        "o toque do quadro sem tique chegou ao 1.º tique, e só a ele"
    );
    assert_eq!(p.translation.y, 1.0, "o largar do toque também");
    assert_eq!(
        p.rotation, 2.0,
        "segurado vê-se em todo tique; «shift» é qualquer dos lados"
    );
    assert_eq!((p.scale.x, p.scale.y), (2.5, -1.0), "o rato no mundo");

    let mau = actor(
        sim.world_mut(),
        2,
        "tecla_errada.luau",
        "function update(self) ph2d.key_down('spcae') end",
    );
    tick(&mut host, &mut sim);
    assert!(
        host.scene_failure(mau)
            .expect("pára")
            .message
            .contains("unknown key 'spcae'")
    );
}

// ── §3.6 (f) o estado lateral ────────────────────────────────────────────────────────────────

#[test]
fn o_estado_apaga_com_nil_volta_listas_e_o_rebobinar_limpa() {
    let mut sim = SimWorld::new();
    let e = actor(
        sim.world_mut(),
        1,
        "estado.luau",
        r#"
function init(self)
  ph2d.state_set(self.id, "lista", { 10, 20, 30 })
  ph2d.state_set(self.id, "fora", true)
  ph2d.state_set(self.id, "fora", nil)
  local l = ph2d.state_get(self.id, "lista")
  ph2d.set(self.id, "x", #l)
  ph2d.set(self.id, "y", l[2])
  ph2d.set(self.id, "rotation", #ph2d.state_keys(self.id))
end
"#,
    );
    let mut host = ScriptHost::new().expect("vm");
    tick(&mut host, &mut sim);
    let p = pose(sim.world(), e);
    assert_eq!(p.translation.x, 3.0, "uma lista volta lista");
    assert_eq!(p.translation.y, 20.0);
    assert_eq!(p.rotation, 1.0, "nil apaga a chave");
    assert!(host.read_state(e.to_bits(), "lista").is_some());
    host.scene_rewind();
    assert!(
        host.read_state(e.to_bits(), "lista").is_none(),
        "rebobinar limpa o estado da corrida"
    );
}

/// Uma espera que o próprio script retomou por fora (`coroutine.running()` guardado) já não espera:
/// quando o tempo chega, nada se retoma e o objecto NÃO pára com «cannot resume dead coroutine».
#[test]
fn uma_espera_retomada_por_fora_ja_nao_espera() {
    let mut sim = SimWorld::new();
    let e = actor(
        sim.world_mut(),
        1,
        "por_fora.luau",
        r#"
function update(self)
  self.n = (self.n or 0) + 1
  if self.n == 1 then
    self.co = coroutine.running()
    ph2d.wait(0.5)
    ph2d.set(self.id, "y", 1)
  elseif self.n == 2 then
    coroutine.resume(self.co)
  end
end
"#,
    );
    let mut host = ScriptHost::new().expect("vm");
    for _ in 0..40 {
        tick(&mut host, &mut sim);
    }
    assert!(
        host.scene_failure(e).is_none(),
        "{:?}",
        host.scene_failure(e)
    );
    assert_eq!(
        pose(sim.world(), e).translation.y,
        1.0,
        "a retomada por fora correu o resto"
    );
    assert_eq!(host.scene().waiting_count(), 0);
}

/// Trocar de script a meio do quadro (`attach_script` num objecto que já corre outro, com dois
/// tiques no quadro e nenhum `sync` no meio) renasce já: o `init` do novo corre e as esperas do
/// antigo vão-se. Mutação sobrevivente do fecho de 20/10.
#[test]
fn trocar_de_script_a_meio_do_quadro_renasce() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    script(
        w,
        "b.luau",
        "function init(self) self.b = 5 end\nfunction update(self) ph2d.set(self.id, 'y', self.b or -1) end",
    );
    actor(
        w,
        1,
        "poe_b.luau",
        "function init(self) ph2d.attach_script(ph2d.find_by_name('E'), 'b.luau') end",
    );
    let alvo = runtime_actor(
        w,
        2,
        "a.luau",
        "function init(self) ph2d.wait(10) end\nfunction update(self) ph2d.set(self.id, 'x', 1) end",
    );
    w.entity_mut(alvo).insert(Name::new("E"));
    actor(w, 3, "b.luau", "");
    let mut host = ScriptHost::new().expect("vm");
    host.scene_sync(sim.world_mut());
    host.scene_tick(&mut sim, DT);
    host.scene_tick(&mut sim, DT);
    assert_eq!(
        pose(sim.world(), alvo).translation.y,
        5.0,
        "o init do b correu antes do update dele"
    );
    assert_eq!(
        host.scene().waiting_count(),
        0,
        "a espera do a foi-se com ele"
    );
}
