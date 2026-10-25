//! ⭐ **O ciclo de vida, uma lei por portão** (GameEngineFuncional N0, `scene_life.rs`) — mesmos
//! ajudantes do pai (`scene_api_tests.rs`).

use super::*;
use crate::NoDoors;

/// Quantas linhas do console dizem exactamente `text`.
fn printed(host: &ScriptHost, text: &str) -> usize {
    console(host).iter().filter(|(t, _)| t == text).count()
}

fn named(world: &mut World, e: Entity, name: &str) -> Entity {
    world.entity_mut(e).insert(Name::new(name));
    e
}

/// `ready` corre no MESMO tique do `init`, depois de TODOS os `init` desse tique: o `ready` de A
/// (identidade 1, corre primeiro) vê o que o `init` de B (identidade 2) fez. Uma vez por vida, e de
/// novo depois do rebobinar.
#[test]
fn o_ready_ve_o_init_dos_outros_no_mesmo_tique() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let a = actor(
        w,
        1,
        "a.luau",
        r#"
function ready(self)
  local b = ph2d.find_by_name("B")
  ph2d.set(self.id, "x", ph2d.state_get(b, "pronto") or -1)
  self.r = (self.r or 0) + 1
  ph2d.set(self.id, "y", self.r)
end
"#,
    );
    let b = actor(
        w,
        2,
        "b.luau",
        r#"function init(self) ph2d.state_set(self.id, "pronto", 7) end"#,
    );
    named(w, b, "B");
    let mut host = ScriptHost::new().expect("vm");
    tick(&mut host, &mut sim);
    assert_eq!(
        pose(sim.world(), a).translation.x,
        7.0,
        "o ready viu o init de B"
    );
    assert_eq!(pose(sim.world(), a).translation.y, 1.0, "no mesmo tique");
    tick(&mut host, &mut sim);
    tick(&mut host, &mut sim);
    assert_eq!(pose(sim.world(), a).translation.y, 1.0, "uma vez por vida");
    host.scene_rewind();
    sim.world_mut()
        .get_mut::<Transform>(a)
        .unwrap()
        .translation
        .y = 0.0;
    tick(&mut host, &mut sim);
    assert_eq!(
        pose(sim.world(), a).translation.y,
        1.0,
        "renasce: o ready de novo"
    );
}

/// `physics(self, dt)` corre a cada passo fixo, ANTES do `update`, com o mesmo `dt`.
#[test]
fn o_physics_corre_antes_do_update_com_o_mesmo_dt() {
    let mut sim = SimWorld::new();
    let e = actor(
        sim.world_mut(),
        1,
        "p.luau",
        r#"
function physics(self, dt)
  self.log = (self.log or "") .. "p"
  self.pdt = dt
end
function update(self, dt)
  self.log = self.log .. "u"
  self.n = (self.n or 0) + 1
  ph2d.set(self.id, "x", self.log == string.rep("pu", self.n) and 1 or 0)
  ph2d.set(self.id, "y", self.pdt == dt and 1 or 0)
end
"#,
    );
    let mut host = ScriptHost::new().expect("vm");
    for _ in 0..3 {
        let r = tick(&mut host, &mut sim);
        assert!(r.failed.is_empty(), "{:?}", r.failed);
    }
    assert_eq!(
        pose(sim.world(), e).translation.x,
        1.0,
        "physics antes de update"
    );
    assert_eq!(pose(sim.world(), e).translation.y, 1.0, "o mesmo dt");
}

/// `process(self, dt)` corre quando o QUADRO pede (com o dt dele), nunca nos passos fixos; e só em
/// quem já nasceu.
#[test]
fn o_process_corre_por_quadro_e_nao_por_passo() {
    let mut sim = SimWorld::new();
    let e = actor(
        sim.world_mut(),
        1,
        "q.luau",
        r#"
function process(self, dt)
  self.q = (self.q or 0) + 1
  ph2d.set(self.id, "y", self.q)
  ph2d.set(self.id, "x", dt)
end
"#,
    );
    let mut host = ScriptHost::new().expect("vm");
    host.scene_sync(sim.world_mut());
    assert_eq!(
        host.scene_process(&mut sim, 0.5).calls,
        0,
        "ainda não nasceu"
    );
    tick(&mut host, &mut sim);
    tick(&mut host, &mut sim);
    assert_eq!(
        pose(sim.world(), e).translation.y,
        0.0,
        "os passos não o chamam"
    );
    let r = host.scene_process(&mut sim, 0.03);
    assert_eq!(r.calls, 1);
    assert_eq!(pose(sim.world(), e).translation.y, 1.0);
    assert_eq!(pose(sim.world(), e).translation.x, 0.03, "o dt do quadro");
}

/// `exit` no `ph2d.despawn`: uma vez, no fim da passagem (o objecto ainda existe e a escrita
/// pousa), e nunca mais — nem quando o objecto sai do mundo.
#[test]
fn o_exit_do_despawn_corre_uma_vez() {
    let mut sim = SimWorld::new();
    let e = runtime_actor(
        sim.world_mut(),
        1,
        "d.luau",
        r#"
function update(self)
  ph2d.despawn(self.id)
end
function exit(self)
  print("exit")
  ph2d.set(self.id, "x", 5)
end
"#,
    );
    let mut host = ScriptHost::new().expect("vm");
    let r = tick(&mut host, &mut sim);
    assert_eq!(r.despawned, vec![e]);
    assert_eq!(
        pose(sim.world(), e).translation.x,
        5.0,
        "a escrita do exit pousa"
    );
    tick(&mut host, &mut sim);
    sim.world_mut().entity_mut(e).despawn();
    tick(&mut host, &mut sim);
    host.scene_stop(&mut sim);
    assert_eq!(printed(&host, "exit"), 1);
}

/// `exit` quando o objecto some do mundo (uma morte) e quando muda de script entre quadros: no
/// tique seguinte, uma vez, antes do `init` do script novo.
#[test]
fn o_exit_da_morte_e_da_troca_entre_quadros() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let morto = actor(w, 1, "m.luau", r#"function exit(self) print("exit m") end"#);
    let troca = actor(w, 2, "t.luau", r#"function exit(self) print("exit t") end"#);
    let novo = script(w, "n.luau", r#"function init(self) print("init n") end"#);
    let mut host = ScriptHost::new().expect("vm");
    tick(&mut host, &mut sim);
    sim.world_mut().entity_mut(morto).despawn();
    sim.world_mut()
        .entity_mut(troca)
        .insert(LuauScript::library(novo));
    tick(&mut host, &mut sim);
    tick(&mut host, &mut sim);
    let linhas: Vec<String> = console(&host).into_iter().map(|(t, _)| t).collect();
    assert_eq!(
        linhas,
        ["exit m", "exit t", "init n"],
        "uma vez cada, exit antes do init"
    );
}

/// `exit` na troca a MEIO do quadro (`ph2d.attach_script`, dois tiques sem `sync`): logo antes do
/// `init` do novo.
#[test]
fn o_exit_da_troca_a_meio_do_quadro() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    script(
        w,
        "b.luau",
        r#"function init(self) if self.id == ph2d.find_by_name("E") then print("init b") end end"#,
    );
    actor(
        w,
        1,
        "poe_b.luau",
        "function init(self) ph2d.attach_script(ph2d.find_by_name('E'), 'b.luau') end",
    );
    let alvo = runtime_actor(w, 2, "a.luau", r#"function exit(self) print("exit a") end"#);
    named(w, alvo, "E");
    actor(w, 3, "b.luau", ""); // o b carregado antes da troca
    let mut host = ScriptHost::new().expect("vm");
    host.scene_sync(sim.world_mut());
    host.scene_tick(&mut sim, DT);
    host.scene_tick(&mut sim, DT);
    host.scene_sync(sim.world_mut());
    host.scene_tick(&mut sim, DT);
    let linhas: Vec<String> = console(&host).into_iter().map(|(t, _)| t).collect();
    assert_eq!(linhas, ["exit a", "init b"]);
}

/// O Stop: cada vivo recebe o `exit` uma vez, pela identidade; o que ele escreve NÃO pousa, e criar
/// é recusado. Parar outra vez não o repete.
#[test]
fn o_stop_corre_o_exit_e_deita_fora_as_escritas() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let e = actor(
        w,
        2,
        "s.luau",
        r#"
function exit(self)
  print("exit " .. tostring(ph2d.get(self.id, "x")))
  ph2d.set(self.id, "x", 99)
end
"#,
    );
    actor(
        w,
        1,
        "s1.luau",
        r#"function exit(self) print("exit 1") ph2d.spawn() end"#,
    );
    w.get_mut::<Transform>(e).unwrap().translation.x = 3.0;
    let mut host = ScriptHost::new().expect("vm");
    tick(&mut host, &mut sim);
    let antes = sim.world().entities().len();
    assert_eq!(host.scene_stop(&mut sim), 2);
    assert_eq!(host.scene_stop(&mut sim), 0);
    let linhas = console(&host);
    let textos: Vec<&str> = linhas.iter().map(|(t, _)| t.as_str()).collect();
    assert_eq!(&textos[..1], ["exit 1"], "pela identidade");
    assert!(
        textos
            .iter()
            .any(|t| t.contains("not while the scene stops")),
        "{textos:?}"
    );
    assert_eq!(
        textos.iter().filter(|t| **t == "exit 3").count(),
        1,
        "{textos:?}"
    );
    assert_eq!(
        pose(sim.world(), e).translation.x,
        3.0,
        "a escrita do Stop não pousa"
    );
    assert_eq!(sim.world().entities().len(), antes, "nada nasceu no Stop");
}

/// `on_signal(self, nome, de)`: `de` é quem emitiu (o id que o script conhece), `nil` sem emissor.
#[test]
fn o_on_signal_recebe_o_emissor() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let emissor = actor(
        w,
        1,
        "e.luau",
        r#"function update(self) ph2d.emit("ping") end"#,
    );
    named(w, emissor, "E");
    let ouvinte = actor(
        w,
        2,
        "o.luau",
        r#"
function on_signal(self, signal, from)
  if from == nil then
    ph2d.set(self.id, "y", 1)
  elseif from == ph2d.find_by_name("E") then
    ph2d.set(self.id, "x", 1)
  end
end
"#,
    );
    let mut host = ScriptHost::new().expect("vm");
    let r = tick(&mut host, &mut sim);
    let ouvidos: Vec<(&str, Option<u64>)> = r
        .emitted
        .iter()
        .map(|(e, n)| (n.as_str(), Some(e.to_bits())))
        .collect();
    assert_eq!(ouvidos, [("ping", Some(emissor.to_bits()))]);
    host.scene_hear_with(&mut sim, &ouvidos, &mut NoDoors);
    assert_eq!(
        pose(sim.world(), ouvinte).translation.x,
        1.0,
        "de = o emissor"
    );
    host.scene_hear(&mut sim, &["marker"]);
    assert_eq!(
        pose(sim.world(), ouvinte).translation.y,
        1.0,
        "sem emissor: nil"
    );
}

/// `ph2d.connect(obj, nome, fn)`: `fn(self, de)` só corre para ESSE emissor e esse nome, e a ligação
/// morre com o `exit` do emissor.
#[test]
fn o_connect_so_ouve_o_emissor_ligado_e_morre_no_exit() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let a = runtime_actor(
        w,
        1,
        "a.luau",
        r#"function on_signal(self, signal) if signal == "go" then ph2d.despawn(self.id) end end"#,
    );
    named(w, a, "A");
    let b = actor(w, 2, "b.luau", "");
    let l = actor(
        w,
        3,
        "l.luau",
        r#"
function ready(self)
  ph2d.connect(ph2d.object("A"), "hit", function(me, from)
    me.n = (me.n or 0) + 1
    ph2d.set(me.id, "x", me.n)
    ph2d.set(me.id, "y", from == ph2d.object("A") and 1 or 0)
  end)
end
"#,
    );
    let (ab, bb) = (Some(a.to_bits()), Some(b.to_bits()));
    let mut host = ScriptHost::new().expect("vm");
    tick(&mut host, &mut sim);
    let x = |sim: &SimWorld| pose(sim.world(), l).translation.x;
    host.scene_hear_with(&mut sim, &[("hit", ab)], &mut NoDoors);
    assert_eq!(x(&sim), 1.0, "o emissor ligado");
    assert_eq!(pose(sim.world(), l).translation.y, 1.0, "de = o emissor");
    host.scene_hear_with(
        &mut sim,
        &[("hit", bb), ("hit", None), ("miss", ab)],
        &mut NoDoors,
    );
    assert_eq!(
        x(&sim),
        1.0,
        "outro emissor, sem emissor ou outro nome: nada"
    );
    host.scene_hear_with(&mut sim, &[("go", None)], &mut NoDoors);
    host.scene_hear_with(&mut sim, &[("hit", ab)], &mut NoDoors);
    assert_eq!(x(&sim), 1.0, "o exit do emissor levou a ligação");
}
