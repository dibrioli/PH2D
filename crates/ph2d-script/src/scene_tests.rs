//! Os scripts da cena, headless, sobre a VM REAL do `ScriptHost` e ficheiros REAIS no disco — o
//! caminho que o produto percorre (§3 do plano 13).

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use ph2d_ecs::{Entity, SimWorld, StableId, Transform, World};

use crate::ScriptHost;
use crate::component::LuauScript;
use crate::props::ScriptValue;
use crate::scene::ScriptInfo;

/// Um ficheiro de script único para este teste (os testes correm em paralelo).
fn script_file(tag: &str, source: &str) -> String {
    static N: AtomicU32 = AtomicU32::new(0);
    let dir: PathBuf = std::env::temp_dir().join(format!("ph2d_scene_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp");
    let p = dir.join(format!("{tag}_{}.luau", N.fetch_add(1, Ordering::Relaxed)));
    std::fs::write(&p, source).expect("escreve");
    p.to_string_lossy().into_owned()
}

fn rewrite(path: &str, source: &str) {
    std::fs::write(path, source).expect("reescreve");
}

/// Um objecto com pose, identidade e script.
fn actor(world: &mut World, sid: u64, path: &str, own: &[(&str, ScriptValue)]) -> Entity {
    let mut s = LuauScript::at(path);
    for (k, v) in own {
        s.own.insert((*k).to_owned(), v.clone());
    }
    world.spawn((Transform::IDENTITY, StableId(sid), s)).id()
}

fn x(world: &World, e: Entity) -> f32 {
    world.get::<Transform>(e).expect("pose").translation.x
}

fn frame(host: &mut ScriptHost, sim: &mut SimWorld, dt: f64) -> crate::scene::SceneReport {
    host.scene_sync(sim.world_mut());
    host.scene_tick(sim.world_mut(), dt)
}

const MOVE_BY_SPEED: &str = r#"
ph2d.property("speed", 1)
function update(self, dt)
  ph2d.set(self.id, "x", ph2d.get(self.id, "x") + self.speed * dt)
end
"#;

/// ⭐⭐⭐ **O mesmo script, os números de cada um** — o §0 do plano, headless.
#[test]
fn um_script_move_cada_objecto_pelos_numeros_dele() {
    let path = script_file("speed", MOVE_BY_SPEED);
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let lento = actor(w, 1, &path, &[]);
    let rapido = actor(w, 2, &path, &[("speed", ScriptValue::Number(4.0))]);
    let mut host = ScriptHost::new().expect("vm");
    for _ in 0..10 {
        let r = frame(&mut host, &mut sim, 0.5);
        assert!(r.failed.is_empty(), "{:?}", r.failed);
    }
    assert_eq!(x(sim.world(), lento), 5.0, "default 1 × 10 × 0,5");
    assert_eq!(x(sim.world(), rapido), 20.0, "próprio 4 × 10 × 0,5");
}

#[test]
fn o_painel_ve_as_declaracoes_sem_a_cena_correr() {
    let path = script_file("decl", MOVE_BY_SPEED);
    let mut sim = SimWorld::new();
    actor(sim.world_mut(), 1, &path, &[]);
    let mut host = ScriptHost::new().expect("vm");
    host.scene_sync(sim.world_mut());
    let Some(ScriptInfo::Ready(decls)) = host.scene_info(&path) else {
        panic!("{:?}", host.scene_info(&path));
    };
    assert_eq!(decls[0].name, "speed");
    assert_eq!(host.scene().live_count(), 0, "parado ninguém nasce");
}

/// ⭐ **Rebobinar é renascer**: o `init` volta a correr, e só então.
#[test]
fn o_init_corre_uma_vez_e_rebobinar_o_faz_correr_outra() {
    let path = script_file(
        "init",
        r#"
function init(self) self.n = 0 ; ph2d.set(self.id, "y", 100) end
function update(self, dt) self.n = self.n + 1 ; ph2d.set(self.id, "x", self.n) end
"#,
    );
    let mut sim = SimWorld::new();
    let e = actor(sim.world_mut(), 1, &path, &[]);
    let mut host = ScriptHost::new().expect("vm");
    for _ in 0..3 {
        frame(&mut host, &mut sim, 1.0 / 60.0);
    }
    assert_eq!(x(sim.world(), e), 3.0);
    assert_eq!(
        sim.world().get::<Transform>(e).expect("pose").translation.y,
        100.0,
        "a escrita do init aplica-se"
    );
    assert_eq!(host.scene_rewind(), 1);
    assert_eq!(host.scene().live_count(), 0);
    frame(&mut host, &mut sim, 1.0 / 60.0);
    assert_eq!(x(sim.world(), e), 1.0, "o self renasceu: n voltou a 0");
}

/// ⚠️ **A ordem é a da IDENTIDADE**, com TRÊS objectos em ordem arbitrária — com dois, `reverse()`
/// dá a ordem certa por acaso (a fixtura fraca do #15).
#[test]
fn quem_emite_primeiro_e_quem_tem_a_identidade_menor() {
    let path = script_file(
        "ordem",
        r#"
ph2d.property("tag", "")
function update(self, dt) ph2d.emit(self.tag) end
"#,
    );
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    actor(w, 20, &path, &[("tag", ScriptValue::Text("b".into()))]);
    actor(w, 30, &path, &[("tag", ScriptValue::Text("c".into()))]);
    actor(w, 10, &path, &[("tag", ScriptValue::Text("a".into()))]);
    let mut host = ScriptHost::new().expect("vm");
    let r = frame(&mut host, &mut sim, 0.1);
    let nomes: Vec<&str> = r.emitted.iter().map(|(_, n)| n.as_str()).collect();
    assert_eq!(nomes, ["a", "b", "c"]);
}

#[test]
fn um_sinal_sem_nome_para_o_script_e_diz_porque() {
    let path = script_file("mudo", r#"function update(self, dt) ph2d.emit("") end"#);
    let mut sim = SimWorld::new();
    let e = actor(sim.world_mut(), 1, &path, &[]);
    let mut host = ScriptHost::new().expect("vm");
    let r = frame(&mut host, &mut sim, 0.1);
    assert!(r.emitted.is_empty());
    assert!(
        host.scene_failure(e)
            .is_some_and(|m| m.contains("needs a name"))
    );
}

/// ⚠️ **Cada escrita tem DONO**: um campo que a casa não conhece pára QUEM o escreveu, e só esse.
#[test]
fn um_campo_desconhecido_para_so_quem_o_escreveu() {
    let mau = script_file(
        "z",
        r#"function update(self, dt) ph2d.set(self.id, "z", 1) end"#,
    );
    let bom = script_file("bom", MOVE_BY_SPEED);
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let e_mau = actor(w, 1, &mau, &[]);
    let e_bom = actor(w, 2, &bom, &[]);
    let mut host = ScriptHost::new().expect("vm");
    let r = frame(&mut host, &mut sim, 1.0);
    assert_eq!(r.failed.len(), 1);
    assert_eq!(r.failed[0].0, e_mau);
    assert!(
        r.failed[0].1.contains("unknown field 'z'"),
        "{}",
        r.failed[0].1
    );
    frame(&mut host, &mut sim, 1.0);
    assert_eq!(x(sim.world(), e_bom), 2.0, "o vizinho continua");
    assert_eq!(
        host.scene().failure(e_mau).map(|_| ()),
        Some(()),
        "e o mau fica parado"
    );
}

#[test]
fn um_numero_que_nao_e_numero_e_recusado() {
    let path = script_file(
        "nan",
        r#"function update(self, dt) ph2d.set(self.id, "x", 0/0) end"#,
    );
    let mut sim = SimWorld::new();
    let e = actor(sim.world_mut(), 1, &path, &[]);
    let mut host = ScriptHost::new().expect("vm");
    frame(&mut host, &mut sim, 1.0);
    assert_eq!(x(sim.world(), e), 0.0, "a pose não recebe um NaN");
    assert!(
        host.scene_failure(e)
            .is_some_and(|m| m.contains("not a number"))
    );
}

/// ⭐⭐ **Um laço sem saída não congela o app** — pára NESTE objecto, com a mensagem.
#[test]
fn um_laco_sem_saida_num_gancho_para_no_prazo() {
    let path = script_file("laco", "function update(self, dt) while true do end end");
    let mut sim = SimWorld::new();
    let e = actor(sim.world_mut(), 1, &path, &[]);
    let mut host = ScriptHost::new().expect("vm");
    let t0 = std::time::Instant::now();
    frame(&mut host, &mut sim, 0.1);
    let gasto = t0.elapsed();
    assert!(
        host.scene_failure(e)
            .is_some_and(|m| m.contains("longer than a frame")),
        "{:?}",
        host.scene_failure(e)
    );
    // ⚠️ A barra é larga de propósito (a máquina de fecho corre a load alto): o que se prova é
    // «acaba», e um laço sem prazo nunca acabaria.
    assert!(gasto < std::time::Duration::from_secs(5), "{gasto:?}");
    // E o prazo FECHA: uma chamada seguinte, fora de qualquer gancho, não herda um prazo vencido.
    let ok = script_file("depois", MOVE_BY_SPEED);
    let e2 = actor(sim.world_mut(), 2, &ok, &[]);
    frame(&mut host, &mut sim, 1.0);
    assert_eq!(x(sim.world(), e2), 1.0);
}

#[test]
fn um_laco_sem_saida_no_topo_nao_congela_a_carga() {
    let path = script_file("topo", "while true do end");
    let mut sim = SimWorld::new();
    actor(sim.world_mut(), 1, &path, &[]);
    let mut host = ScriptHost::new().expect("vm");
    host.scene_sync(sim.world_mut());
    assert!(
        matches!(host.scene_info(&path), Some(ScriptInfo::Broken(m)) if m.contains("longer than a frame")),
        "{:?}",
        host.scene_info(&path)
    );
}

/// ⚠️ **Recarregar NÃO renasce**: as funções mudam, o `self` fica.
#[test]
fn recarregar_troca_as_funcoes_e_mantem_o_self() {
    let path = script_file(
        "reload",
        r#"
function init(self) self.n = 0 end
function update(self, dt) self.n = self.n + 1 ; ph2d.set(self.id, "x", self.n) end
"#,
    );
    let mut sim = SimWorld::new();
    let e = actor(sim.world_mut(), 1, &path, &[]);
    let mut host = ScriptHost::new().expect("vm");
    frame(&mut host, &mut sim, 0.1);
    frame(&mut host, &mut sim, 0.1);
    assert_eq!(x(sim.world(), e), 2.0);
    rewrite(
        &path,
        r#"
function init(self) self.n = 1000 end
function update(self, dt) self.n = self.n + 10 ; ph2d.set(self.id, "x", self.n) end
"#,
    );
    frame(&mut host, &mut sim, 0.1);
    assert_eq!(
        x(sim.world(), e),
        12.0,
        "a lei nova (+10) sobre o self velho (n = 2), e o init NÃO correu outra vez"
    );
}

#[test]
fn gravar_a_correccao_devolve_um_script_partido_a_corrida() {
    let path = script_file("partido", r#"function update(self, dt) error("ups") end"#);
    let mut sim = SimWorld::new();
    let e = actor(sim.world_mut(), 1, &path, &[]);
    let mut host = ScriptHost::new().expect("vm");
    frame(&mut host, &mut sim, 1.0);
    assert!(host.scene_failure(e).is_some_and(|m| m.contains("ups")));
    rewrite(&path, MOVE_BY_SPEED);
    frame(&mut host, &mut sim, 1.0);
    assert!(host.scene_failure(e).is_none());
    assert_eq!(x(sim.world(), e), 1.0);
}

/// ⚠️ **O D2 na VM**: com o ficheiro a meio (erro de sintaxe) o painel lê «desconhecido», e os
/// valores próprios ficam intactos no componente.
#[test]
fn um_script_partido_nao_apaga_nada_do_componente() {
    let path = script_file("sintaxe", MOVE_BY_SPEED);
    let mut sim = SimWorld::new();
    let e = actor(
        sim.world_mut(),
        1,
        &path,
        &[("speed", ScriptValue::Number(9.0))],
    );
    let mut host = ScriptHost::new().expect("vm");
    host.scene_sync(sim.world_mut());
    rewrite(&path, "ph2d.property(\"speed\", 1\nfunction update(");
    host.scene_sync(sim.world_mut());
    let info = host.scene_info(&path).expect("conhecido");
    assert!(
        matches!(info, ScriptInfo::Broken(m) if m.starts_with("syntax error")),
        "{info:?}"
    );
    assert!(info.decls().is_none(), "desconhecido, não vazio");
    assert_eq!(
        sim.world()
            .get::<LuauScript>(e)
            .expect("cfg")
            .own
            .get("speed"),
        Some(&ScriptValue::Number(9.0))
    );
}

#[test]
fn sem_ficheiro_nada_corre_e_o_painel_sabe_porque() {
    let mut sim = SimWorld::new();
    let e = actor(sim.world_mut(), 1, "/nao/existe/bob.luau", &[]);
    let mut host = ScriptHost::new().expect("vm");
    let r = frame(&mut host, &mut sim, 1.0);
    assert_eq!(r.calls, 0);
    assert!(matches!(
        host.scene_info("/nao/existe/bob.luau"),
        Some(ScriptInfo::Missing(_))
    ));
    assert_eq!(x(sim.world(), e), 0.0);
}

/// ⚠️ **A mão do artista manda, mas só onde mexeu** (§3.5).
#[test]
fn editar_um_numero_a_meio_da_corrida_chega_so_ao_editado() {
    let path = script_file(
        "live",
        r#"
ph2d.property("a", 1)
ph2d.property("b", 1)
function init(self) self.a = 50 end
function update(self, dt) ph2d.set(self.id, "x", self.a) ; ph2d.set(self.id, "y", self.b) end
"#,
    );
    let mut sim = SimWorld::new();
    let e = actor(sim.world_mut(), 1, &path, &[]);
    let mut host = ScriptHost::new().expect("vm");
    frame(&mut host, &mut sim, 0.1);
    assert_eq!(x(sim.world(), e), 50.0, "o script ajustou o próprio a");
    sim.world_mut()
        .get_mut::<LuauScript>(e)
        .expect("cfg")
        .own
        .insert("b".into(), ScriptValue::Number(7.0));
    frame(&mut host, &mut sim, 0.1);
    let t = *sim.world().get::<Transform>(e).expect("pose");
    assert_eq!(
        (t.translation.x, t.translation.y),
        (50.0, 7.0),
        "b chegou; a não foi reposto"
    );
    sim.world_mut()
        .get_mut::<LuauScript>(e)
        .expect("cfg")
        .own
        .insert("a".into(), ScriptValue::Number(3.0));
    frame(&mut host, &mut sim, 0.1);
    assert_eq!(x(sim.world(), e), 3.0, "mexer no a é que o repõe");
}

#[test]
fn trocar_de_ficheiro_renasce() {
    let a = script_file(
        "a",
        "function init(self) self.k = 1 end function update(self) ph2d.set(self.id, 'x', self.k) end",
    );
    let b = script_file(
        "b",
        "function init(self) self.k = 2 end function update(self) ph2d.set(self.id, 'x', self.k) end",
    );
    let mut sim = SimWorld::new();
    let e = actor(sim.world_mut(), 1, &a, &[]);
    let mut host = ScriptHost::new().expect("vm");
    frame(&mut host, &mut sim, 0.1);
    assert_eq!(x(sim.world(), e), 1.0);
    sim.world_mut()
        .get_mut::<LuauScript>(e)
        .expect("cfg")
        .source = b;
    frame(&mut host, &mut sim, 0.1);
    assert_eq!(x(sim.world(), e), 2.0);
}

/// ⚠️ **Todos leem a mesma fotografia**: escrever e ler o próprio `x` no mesmo gancho lê o valor
/// do início do tique.
#[test]
fn escrever_e_ler_no_mesmo_gancho_le_a_fotografia() {
    let path = script_file(
        "foto",
        r#"
function update(self, dt)
  ph2d.set(self.id, "x", 10)
  ph2d.set(self.id, "y", ph2d.get(self.id, "x"))
end
"#,
    );
    let mut sim = SimWorld::new();
    let e = actor(sim.world_mut(), 1, &path, &[]);
    let mut host = ScriptHost::new().expect("vm");
    frame(&mut host, &mut sim, 0.1);
    let t = *sim.world().get::<Transform>(e).expect("pose");
    assert_eq!((t.translation.x, t.translation.y), (10.0, 0.0));
    frame(&mut host, &mut sim, 0.1);
    let t = *sim.world().get::<Transform>(e).expect("pose");
    assert_eq!(
        t.translation.y, 10.0,
        "no tique seguinte a fotografia já tem o 10"
    );
}

#[test]
fn uma_escrita_igual_ao_que_ja_la_esta_nao_conta() {
    let path = script_file(
        "igual",
        r#"function update(self, dt) ph2d.set(self.id, "x", 0) end"#,
    );
    let mut sim = SimWorld::new();
    actor(sim.world_mut(), 1, &path, &[]);
    let mut host = ScriptHost::new().expect("vm");
    let r = frame(&mut host, &mut sim, 0.1);
    assert_eq!(r.calls, 1);
    assert!(r.wrote.is_empty(), "o ledger só ouve quem MUDOU");
}

#[test]
fn quem_ouve_um_sinal_reage_e_quem_nao_nasceu_nao_ouve() {
    let path = script_file(
        "ouve",
        r#"
function on_signal(self, nome)
  if nome == "salta" then ph2d.set(self.id, "y", 5) ; ph2d.emit("saltei") end
end
"#,
    );
    let mut sim = SimWorld::new();
    let e = actor(sim.world_mut(), 1, &path, &[]);
    let mut host = ScriptHost::new().expect("vm");
    host.scene_sync(sim.world_mut());
    let r = host.scene_hear(sim.world_mut(), &["salta"]);
    assert_eq!(r.calls, 0, "antes do primeiro tique não há self");
    frame(&mut host, &mut sim, 0.1);
    let r = host.scene_hear(sim.world_mut(), &["outro", "salta"]);
    assert_eq!(r.calls, 2);
    assert_eq!(r.emitted, vec![(e, "saltei".to_owned())]);
    assert_eq!(r.wrote, vec![e]);
    assert_eq!(
        sim.world().get::<Transform>(e).expect("pose").translation.y,
        5.0
    );
}

/// ⚠️ **Quem perde o componente perde o vivo** — um `LuauScript` removido não deixa um `self`
/// órfão à espera de um objecto que já não é dele.
#[test]
fn tirar_o_script_esquece_o_vivo() {
    let path = script_file("tira", MOVE_BY_SPEED);
    let mut sim = SimWorld::new();
    let e = actor(sim.world_mut(), 1, &path, &[]);
    let mut host = ScriptHost::new().expect("vm");
    frame(&mut host, &mut sim, 0.1);
    assert_eq!(host.scene().live_count(), 1);
    sim.world_mut().entity_mut(e).remove::<LuauScript>();
    host.scene_sync(sim.world_mut());
    assert_eq!(host.scene().live_count(), 0);
}
