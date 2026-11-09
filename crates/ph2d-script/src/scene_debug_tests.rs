//! ⭐ O depurador sobre a VM REAL do `ScriptHost` (plano 13 Onda D): parar numa linha, Next line,
//! a fila dos que batem no mesmo ponto, Pause on Error, valores ao vivo só do vigiado.

use ph2d_ecs::{Entity, SimWorld, StableId, Transform, World};
use ph2d_script_debug::StopKind;

use crate::component::{LuauScript, ProjectScripts};
use crate::{NoDoors, ScriptHost, ScriptId};

const DT: f64 = 0.5;

/// A linha (1-based) do marcador `-- @nome` em `src`.
fn at(src: &str, mark: &str) -> u32 {
    let tag = format!("-- @{mark}");
    let i = src
        .lines()
        .position(|l| l.contains(&tag))
        .unwrap_or_else(|| panic!("marcador {mark}"));
    u32::try_from(i + 1).expect("linha")
}

struct Mesa {
    host: ScriptHost,
    sim: SimWorld,
    id: ScriptId,
    actors: Vec<Entity>,
}

impl Mesa {
    fn new(src: &str, n: u64) -> Self {
        let mut sim = SimWorld::new();
        let w = sim.world_mut();
        let id = w
            .get_resource_or_init::<ProjectScripts>()
            .insert("dbg.luau", src.to_owned());
        let actors = (1..=n)
            .map(|sid| {
                w.spawn((Transform::IDENTITY, StableId(sid), LuauScript::library(id)))
                    .id()
            })
            .collect();
        let mut host = ScriptHost::new().expect("vm");
        host.scene_sync(sim.world_mut());
        Self {
            host,
            sim,
            id,
            actors,
        }
    }

    fn frame(&mut self) {
        self.host.scene_sync(self.sim.world_mut());
        if !self.host.is_held() {
            self.host.scene_tick(&mut self.sim, DT);
        }
        self.host.debug_end_frame();
    }

    fn resume(&mut self, step: bool) {
        if step {
            self.host.request_next_line();
        } else {
            self.host.request_continue();
        }
        assert!(
            self.host
                .debug_frame_with(&mut self.sim, &mut NoDoors)
                .is_some(),
            "o pedido executou"
        );
    }

    fn x(&self, i: usize) -> f32 {
        x_of(self.sim.world(), self.actors[i])
    }
}

fn x_of(world: &World, e: Entity) -> f32 {
    world.get::<Transform>(e).expect("pose").translation.x
}

const MOVE: &str = r#"
function update(self, dt)
  local x = ph2d.get(self.id, "x") -- @read
  local step = dt * 2 -- @step
  ph2d.set(self.id, "x", x + step) -- @write
end
function never(self)
  return 1 -- @never
end
"#;

#[test]
fn depurador_para_na_linha_com_as_variaveis_e_continue_acaba_o_quadro() {
    let mut m = Mesa::new(MOVE, 1);
    m.host.set_breakpoints(m.id, vec![at(MOVE, "write")]);
    m.frame();
    assert!(m.host.is_held(), "parou");
    let stop = m.host.debug_stop().expect("paragem");
    assert_eq!(stop.why, StopKind::Breakpoint);
    assert_eq!(stop.entity, m.actors[0]);
    let f = stop.frame.expect("pilha");
    assert_eq!(f.line, at(MOVE, "write"));
    assert_eq!(f.function, "update");
    let v = |n: &str| {
        f.locals
            .iter()
            .find(|v| v.name == n)
            .map(|v| v.value.clone())
    };
    assert_eq!(v("x").as_deref(), Some("0"));
    assert_eq!(v("step").as_deref(), Some("1"));
    assert!(f.locals.iter().any(|v| v.name == "self"), "{:?}", f.locals);
    assert_eq!(m.x(0), 0.0, "a escrita do parado ainda não pousou");
    // Parado, o quadro seguinte não corre (quem chama congela o jogo).
    m.frame();
    assert_eq!(m.x(0), 0.0);
    m.resume(false);
    assert!(!m.host.is_held(), "seguiu");
    assert_eq!(m.x(0), 1.0, "o resto do update correu e a escrita pousou");
    // Próximo quadro: pára outra vez no mesmo ponto.
    m.frame();
    assert!(m.host.is_held());
}

#[test]
fn depurador_os_que_batem_no_mesmo_ponto_ficam_em_fila() {
    let mut m = Mesa::new(MOVE, 3);
    m.host.set_breakpoints(m.id, vec![at(MOVE, "write")]);
    m.frame();
    let s = m.host.debug_stop().expect("paragem");
    assert_eq!((s.entity, s.waiting.len()), (m.actors[0], 2));
    m.resume(false);
    let s = m.host.debug_stop().expect("o 2.º");
    assert_eq!((s.entity, s.waiting.len()), (m.actors[1], 1));
    m.resume(false);
    m.resume(false);
    assert!(!m.host.is_held());
    assert_eq!((m.x(0), m.x(1), m.x(2)), (1.0, 1.0, 1.0));
}

#[test]
fn depurador_tirar_o_ponto_com_tres_em_fila_um_continue_solta_todos() {
    let mut m = Mesa::new(MOVE, 3);
    m.host.set_breakpoints(m.id, vec![at(MOVE, "write")]);
    m.frame();
    assert_eq!(m.host.debug_stop().expect("paragem").waiting.len(), 2);
    m.host.set_breakpoints(m.id, vec![]);
    // O mostrado continua o mesmo; os da fila já não contam.
    let s = m.host.debug_stop().expect("ainda parado");
    assert_eq!((s.entity, s.waiting.len()), (m.actors[0], 0));
    assert!(m.host.is_held(), "só o Continue os solta");
    m.resume(false);
    assert!(!m.host.is_held(), "os da fila seguiram sozinhos");
    assert!(m.host.debug_stop().is_none());
    assert_eq!((m.x(0), m.x(1), m.x(2)), (1.0, 1.0, 1.0));
}

#[test]
fn depurador_paragem_de_next_line_nao_se_solta_sozinha() {
    let mut m = Mesa::new(MOVE, 2);
    m.host.set_breakpoints(m.id, vec![at(MOVE, "read")]);
    m.frame();
    m.resume(true);
    let s = m.host.debug_stop().expect("Next line parou");
    assert_eq!((s.why.clone(), s.entity), (StopKind::Step, m.actors[0]));
    m.host.set_breakpoints(m.id, vec![]);
    let s = m.host.debug_stop().expect("ainda o passo");
    assert_eq!(
        (s.why, s.entity, s.waiting.len()),
        (StopKind::Step, m.actors[0], 0)
    );
    // Mais um Next line: o 2.º (ponto tirado) segue sozinho; a paragem do passo fica.
    m.resume(true);
    let s = m.host.debug_stop().expect("o passo não se solta");
    assert_eq!((s.why, s.entity), (StopKind::Step, m.actors[0]));
    assert_eq!(m.x(1), 1.0, "o da fila acabou o update");
    m.resume(false);
    assert!(!m.host.is_held());
    assert_eq!((m.x(0), m.x(1)), (1.0, 1.0));
}

#[test]
fn depurador_next_line_para_na_linha_seguinte() {
    let mut m = Mesa::new(MOVE, 1);
    m.host.set_breakpoints(m.id, vec![at(MOVE, "read")]);
    m.frame();
    let s = m.host.debug_stop().expect("paragem");
    assert_eq!(s.frame.expect("pilha").line, at(MOVE, "read"));
    m.resume(true);
    let s = m.host.debug_stop().expect("Next line parou");
    assert_eq!(s.why, StopKind::Step);
    let f = s.frame.expect("pilha");
    assert_eq!(f.line, at(MOVE, "step"));
    assert!(f.locals.iter().any(|v| v.name == "x"), "x já existe");
    m.resume(true);
    assert_eq!(
        m.host
            .debug_stop()
            .expect("outra")
            .frame
            .expect("pilha")
            .line,
        at(MOVE, "write")
    );
    // Da última linha o Next line pára no `end` (o retorno mora nela); mais um e o update acaba.
    m.resume(true);
    let end = m
        .host
        .debug_stop()
        .expect("no end")
        .frame
        .expect("pilha")
        .line;
    assert_eq!(end, at(MOVE, "write") + 1);
    m.resume(true);
    assert!(!m.host.is_held());
    assert_eq!(m.x(0), 1.0);
}

#[test]
fn depurador_ponto_numa_funcao_que_nao_corre_nao_para() {
    let mut m = Mesa::new(MOVE, 2);
    m.host.set_breakpoints(m.id, vec![at(MOVE, "never")]);
    assert_eq!(m.host.armed_lines(m.id), &[at(MOVE, "never")]);
    for _ in 0..3 {
        m.frame();
    }
    assert!(!m.host.is_held());
    assert_eq!(m.x(0), 3.0);
    // Tirar os pontos: nada armado.
    m.host.set_breakpoints(m.id, vec![]);
    assert!(m.host.armed_lines(m.id).is_empty());
}

#[test]
fn depurador_uma_linha_sem_codigo_arma_na_seguinte() {
    let src = "\nfunction update(self, dt)\n\n  ph2d.set(self.id, \"x\", 5) -- @w\nend\n";
    let mut m = Mesa::new(src, 1);
    m.host.set_breakpoints(m.id, vec![at(src, "w") - 1]);
    assert_eq!(m.host.armed_lines(m.id), &[at(src, "w")]);
}

const BOOM: &str = r#"
function update(self, dt)
  local hp = 3
  local target = nil
  local n = target.health -- @boom
end
"#;

#[test]
fn depurador_pause_on_error_para_no_erro_com_as_variaveis() {
    let mut m = Mesa::new(BOOM, 1);
    assert!(m.host.pause_on_error(), "ligado por omissão");
    m.frame();
    let s = m.host.debug_stop().expect("parou no erro");
    let StopKind::Error(msg) = &s.why else {
        panic!("{:?}", s.why)
    };
    assert!(msg.contains("health") || msg.contains("nil"), "{msg}");
    let f = s.frame.expect("pilha");
    assert_eq!(f.line, at(BOOM, "boom"));
    assert!(
        f.locals.iter().any(|v| v.name == "hp" && v.value == "3"),
        "{:?}",
        f.locals
    );
    // Plano 14: descer ao `self` (a 1.ª local) não pisa as seguintes no fio parado pelo erro.
    assert!(
        f.locals.iter().any(|v| v.name == "dt" && v.value == "0.5"),
        "{:?}",
        f.locals
    );
    m.resume(false);
    assert!(!m.host.is_held(), "o Continue larga o erro");
    m.frame();
    assert!(!m.host.is_held(), "o objecto já parou: não volta a parar");
}

#[test]
fn depurador_sem_pause_on_error_o_erro_nao_para_o_jogo() {
    let mut m = Mesa::new(BOOM, 1);
    m.host.set_pause_on_error(false);
    m.frame();
    assert!(!m.host.is_held());
}

const LIVE: &str = r#"
function update(self, dt)
  local x = ph2d.get(self.id, "x") -- @x
  local speed = 4 -- @speed
  print("tick") -- @print
  ph2d.set(self.id, "x", x + speed) -- @set
end
"#;

#[test]
fn depurador_valores_ao_vivo_so_do_script_vigiado_e_de_um_objecto() {
    let mut m = Mesa::new(LIVE, 2);
    m.frame();
    assert!(m.host.live_values(m.id).is_empty(), "nada vigiado");
    m.host.set_live_watch(Some(m.id), None);
    m.frame();
    m.frame();
    assert_eq!(
        m.host.live_object(),
        Some(m.actors[0]),
        "o 1.º pela identidade"
    );
    let live = m.host.live_values(m.id);
    let speed = live.get(&at(LIVE, "speed")).expect("linha do speed");
    assert_eq!(speed.hits, 1, "um objecto, um quadro");
    assert!(
        speed
            .values
            .iter()
            .any(|v| v.name == "speed" && v.value == "4"),
        "{speed:?}"
    );
    let x = live.get(&at(LIVE, "x")).expect("linha do x");
    // O mostrado é o ÚLTIMO quadro: o x lido nele já tinha 2 passos de 4.
    assert!(
        x.values.iter().any(|v| v.name == "x" && v.value == "8"),
        "{x:?}"
    );
    assert_eq!(live.get(&at(LIVE, "print")).expect("print").prints, 1);
    // Outro script não tem valores.
    assert!(m.host.live_values(ScriptId(m.id.0 + 99)).is_empty());
    // Seleccionar o 2.º troca de quem são.
    m.host.set_live_watch(Some(m.id), Some(m.actors[1]));
    m.frame();
    assert_eq!(m.host.live_object(), Some(m.actors[1]));
    // Fechar a aba: nada.
    m.host.set_live_watch(None, None);
    m.frame();
    assert!(m.host.live_values(m.id).is_empty());
}

const LOOPS: &str = r#"
function update(self, dt)
  local x = ph2d.get(self.id, "x") -- @x
  self.odd = not self.odd
  if self.odd then
    self.t = 1 -- @odd
  end
  for i = 1, 3 do
    self.n = i -- @loop
  end
end
"#;

/// Smoke do dono 09/11: «×2» piscava no fim das linhas — era o quadro com DOIS passos fixos. O `×N` é
/// de uma corrida (o laço), e uma linha que não correu num quadro guarda o último valor.
#[test]
fn depurador_o_vezes_e_de_uma_corrida_e_a_linha_que_nao_correu_nao_some() {
    let mut m = Mesa::new(LOOPS, 1);
    m.host.set_live_watch(Some(m.id), None);
    m.frame();
    let hits = |m: &Mesa, k| m.host.live_values(m.id).get(&at(LOOPS, k)).map(|x| x.hits);
    // Um quadro com dois passos fixos: o `odd` corre num deles.
    m.host.scene_sync(m.sim.world_mut());
    m.host.scene_tick(&mut m.sim, DT);
    m.host.scene_tick(&mut m.sim, DT);
    m.host.debug_end_frame();
    assert_eq!(hits(&m, "x"), Some(1), "dois passos não são «×2»");
    assert_eq!(hits(&m, "loop"), Some(3), "o laço de uma corrida é «×3»");
    assert_eq!(hits(&m, "odd"), Some(1));
    // O quadro seguinte (um passo) não entra no `if`: a linha dele fica com o último valor.
    m.frame();
    assert_eq!(hits(&m, "odd"), Some(1), "a linha que não correu não some");
    assert_eq!(hits(&m, "x"), Some(1));
    // O texto muda e recarrega: as linhas do texto velho saem (as novas voltam a correr).
    let novo = LOOPS.replace("self.t = 1", "self.t = 2");
    m.sim
        .world_mut()
        .resource_mut::<ProjectScripts>()
        .set_text(m.id, &novo);
    m.host.scene_sync(m.sim.world_mut());
    assert_eq!(
        hits(&m, "odd"),
        None,
        "recarregar limpa os valores do texto velho"
    );
    let _ = m.host.drain_reloads();
    // Rebobinar é renascer: os valores da corrida velha saem.
    m.host.scene_rewind();
    assert!(
        m.host.live_values(m.id).is_empty(),
        "rebobinar limpa os valores"
    );
}

const TWO_HOOKS: &str = r#"
function physics(self, dt)
  local a = 1 -- @phys
end
function update(self, dt)
  ph2d.set(self.id, "x", 7) -- @upd
end
"#;

#[test]
fn depurador_os_ganchos_seguintes_esperam_a_vez_do_parado() {
    let mut m = Mesa::new(TWO_HOOKS, 1);
    m.host.set_breakpoints(m.id, vec![at(TWO_HOOKS, "phys")]);
    m.frame();
    assert!(m.host.is_held());
    assert_eq!(m.x(0), 0.0, "o update não correu antes de o physics acabar");
    m.resume(false);
    assert_eq!(m.x(0), 7.0, "o update correu depois do Continue");
    assert!(!m.host.is_held());
}

#[test]
fn depurador_recarregar_o_script_rearma_o_ponto() {
    let mut m = Mesa::new(MOVE, 1);
    m.host.set_breakpoints(m.id, vec![at(MOVE, "write")]);
    let edited = MOVE.replace("dt * 2", "dt * 4");
    assert!(
        m.sim
            .world_mut()
            .resource_mut::<ProjectScripts>()
            .set_text(m.id, &edited)
    );
    m.frame();
    assert!(m.host.is_held(), "o módulo novo também pára");
    m.resume(false);
    assert_eq!(m.x(0), 2.0, "correu o texto novo");
}

#[test]
fn depurador_um_ponto_dentro_de_uma_funcao_c_nao_para_nem_parte() {
    let src = r#"
function update(self, dt)
  local t = {3, 1, 2}
  table.sort(t, function(a, b)
    return a < b -- @cmp
  end)
  ph2d.set(self.id, "x", t[1])
end
"#;
    let mut m = Mesa::new(src, 1);
    m.host.set_breakpoints(m.id, vec![at(src, "cmp")]);
    m.frame();
    assert!(!m.host.is_held(), "dentro do table.sort não se cede");
    assert!(m.host.take_blocked_stops() > 0, "e a aba fica a saber");
    assert_eq!(m.x(0), 1.0, "o script correu inteiro");
}

#[test]
fn depurador_rebobinar_larga_quem_estava_parado() {
    let mut m = Mesa::new(MOVE, 1);
    m.host.set_breakpoints(m.id, vec![at(MOVE, "write")]);
    m.frame();
    assert!(m.host.is_held());
    m.host.scene_rewind();
    assert!(!m.host.is_held());
}

#[path = "scene_debug_tests_fecho.rs"]
mod fecho;

/// Plano 14: a paragem dá o TIPO de cada valor e desce às tabelas (nomes por ordem, depois `[i]`),
/// até `Var::MAX_DEPTH` níveis; quem espera vem pelo nome (a entidade), não só a contagem.
#[test]
fn depurador_a_paragem_da_o_tipo_e_os_campos_das_tabelas_aninhadas() {
    use crate::ValueKind;
    const SRC: &str = r#"
function update(self, dt)
  self.pos = { x = 3, y = 4, deep = { a = { b = { c = 1 } } } }
  self.trail = { 10, 20 }
  self.label = "hi"
  self.alive = true
  local none = nil -- @stop
end
"#;
    let mut m = Mesa::new(SRC, 2);
    m.host.set_breakpoints(m.id, vec![at(SRC, "stop")]);
    m.frame();
    let stop = m.host.debug_stop().expect("paragem");
    assert_eq!(stop.waiting, vec![m.actors[1]], "o 2.º espera, pelo nome");
    let f = stop.frame.expect("pilha");
    let field = |n: &str| f.this.iter().find(|v| v.name == n).expect(n).clone();
    let pos = field("pos");
    assert_eq!((pos.kind, pos.field_count), (ValueKind::Table, 3));
    let names: Vec<&str> = pos.fields.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(names, ["deep", "x", "y"]);
    assert_eq!(pos.fields[1].kind, ValueKind::Number);
    // self.pos é o nível 1; deep 2; a 3; b já não desce (só a contagem).
    let a = &pos.fields[0].fields[0];
    assert_eq!(a.name, "a");
    let b = &a.fields[0];
    assert_eq!((b.kind, b.fields.len(), b.field_count), (ValueKind::Table, 0, 1));
    let trail = field("trail");
    let idx: Vec<(&str, &str)> = trail
        .fields
        .iter()
        .map(|v| (v.name.as_str(), v.value.as_str()))
        .collect();
    assert_eq!(idx, [("[1]", "10"), ("[2]", "20")]);
    assert_eq!(field("label").kind, ValueKind::String);
    assert_eq!(field("alive").kind, ValueKind::Boolean);
    let dt = f.locals.iter().find(|v| v.name == "dt").expect("dt");
    assert_eq!(dt.kind, ValueKind::Number);
}
