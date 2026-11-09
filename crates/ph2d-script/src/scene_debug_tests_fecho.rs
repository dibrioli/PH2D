//! Os portões da auditoria do fecho da Onda D (o depurador): o `exit` não pára, o estado de cada
//! fio sai com ele, as esperas do parado esperam, recarregar desarma o módulo velho, o parado
//! retomado por fora, Pause on Error de quem saiu, o *Next line* por fio e o laço numa linha.

use super::*;

fn y(m: &Mesa, i: usize) -> f32 {
    m.sim
        .world()
        .get::<Transform>(m.actors[i])
        .expect("pose")
        .translation
        .y
}

#[test]
fn depurador_um_ponto_no_exit_nao_para_e_o_exit_acaba() {
    let src = r#"
function update(self, dt) end
function exit(self)
  local a = 1
  local b = 2 -- @e2
  ph2d.set(self.id, "x", 9)
end
"#;
    let mut m = Mesa::new(src, 1);
    m.host.set_breakpoints(m.id, vec![at(src, "e2")]);
    m.frame();
    m.sim
        .world_mut()
        .entity_mut(m.actors[0])
        .remove::<LuauScript>();
    m.frame();
    assert!(!m.host.is_held(), "o exit não tem fila a quem voltar");
    assert_eq!(m.x(0), 9.0, "a 3.ª linha do exit correu");
}

#[test]
fn depurador_next_line_na_ultima_linha_nao_para_o_quadro_seguinte() {
    let mut m = Mesa::new(MOVE, 1);
    m.host.set_breakpoints(m.id, vec![at(MOVE, "write")]);
    m.frame();
    assert!(m.host.is_held());
    for _ in 0..4 {
        if !m.host.is_held() {
            break;
        }
        m.resume(true);
    }
    assert!(!m.host.is_held(), "o Next line saiu do update");
    m.host.set_breakpoints(m.id, vec![]);
    for _ in 0..3 {
        m.frame();
        assert!(
            !m.host.is_held(),
            "nenhum passo velho pára o quadro seguinte"
        );
    }
    m.host.set_breakpoints(m.id, vec![at(MOVE, "read")]);
    m.frame();
    let s = m.host.debug_stop().expect("o ponto seguinte pára");
    assert_eq!(s.why, StopKind::Breakpoint);
}

#[test]
fn depurador_as_esperas_do_parado_nao_retomam_enquanto_ele_esta_parado() {
    let src = r#"
function update(self, dt)
  self.n = (self.n or 0) + 1
  if self.n == 1 then
    ph2d.wait(0.75)
    ph2d.set(self.id, "y", 5)
  else
    local z = 1 -- @bp
  end
end
"#;
    let mut m = Mesa::new(src, 1);
    m.host.set_breakpoints(m.id, vec![at(src, "bp")]);
    m.frame();
    assert!(!m.host.is_held(), "o 1.º tique espera");
    m.frame();
    assert!(m.host.is_held(), "o 2.º pára");
    assert_eq!(
        y(&m, 0),
        0.0,
        "a espera devida não correu com o dono parado"
    );
    m.host.set_breakpoints(m.id, vec![]);
    m.resume(false);
    assert_eq!(y(&m, 0), 0.0, "ainda não");
    m.frame();
    assert_eq!(
        y(&m, 0),
        5.0,
        "solto, a espera devida retoma no tique seguinte"
    );
}

#[test]
fn depurador_recarregar_com_um_parado_desarma_o_modulo_velho() {
    let src = r#"
function update(self, dt)
  for i = 1, 2 do
    ph2d.set(self.id, "x", i) -- @loop
  end
end
"#;
    let mut m = Mesa::new(src, 1);
    m.host.set_breakpoints(m.id, vec![at(src, "loop")]);
    m.frame();
    assert!(m.host.is_held(), "parou na 1.ª volta");
    let edited = src.replace("\"x\", i)", "\"x\", i * 10)");
    assert!(
        m.sim
            .world_mut()
            .resource_mut::<ProjectScripts>()
            .set_text(m.id, &edited)
    );
    m.frame();
    assert!(m.host.is_held(), "recarregar não solta o parado");
    m.resume(false);
    assert!(
        !m.host.is_held(),
        "o módulo velho já não tem o ponto: a 2.ª volta não pára"
    );
    assert_eq!(m.x(0), 2.0, "acabou o texto velho");
    m.frame();
    assert!(m.host.is_held(), "o módulo novo pára");
    m.resume(false);
    m.resume(false);
    assert_eq!(m.x(0), 20.0, "correu o texto novo");
}

#[test]
fn depurador_um_parado_retomado_por_fora_nao_falha_no_continue() {
    let src = r#"
local saved = nil
function update(self, dt)
  if saved == nil then
    saved = coroutine.running()
    ph2d.set(self.id, "x", 1) -- @bp
  elseif saved then
    local co = saved
    saved = false
    coroutine.resume(co)
  end
end
"#;
    let mut m = Mesa::new(src, 2);
    m.host.set_breakpoints(m.id, vec![at(src, "bp")]);
    m.frame();
    assert!(m.host.is_held(), "o 1.º parou");
    m.resume(false);
    assert!(!m.host.is_held());
    assert!(
        m.host.scene_failure(m.actors[0]).is_none(),
        "{:?}",
        m.host.scene_failure(m.actors[0])
    );
    m.frame();
    assert!(m.host.scene_failure(m.actors[0]).is_none());
}

#[test]
fn depurador_apagar_o_objecto_parado_num_erro_solta_o_jogo() {
    let mut m = Mesa::new(BOOM, 1);
    m.frame();
    assert!(m.host.is_held(), "parou no erro");
    m.sim.world_mut().despawn(m.actors[0]);
    m.frame();
    assert!(!m.host.is_held(), "quem parou já não existe");
    assert!(m.host.debug_stop().is_none());
}

#[test]
fn depurador_o_ponto_de_outro_nao_cancela_o_next_line_de_quem_espera() {
    let src = r#"
local calls = 0
function update(self, dt)
  calls = calls + 1
  if calls == 1 then
    local a = 1 -- @a
    ph2d.wait(0.75) -- @w
    ph2d.set(self.id, "x", 3) -- @after
  elseif calls == 4 then
    local b = 2 -- @b
  end
end
"#;
    let mut m = Mesa::new(src, 2);
    m.host
        .set_breakpoints(m.id, vec![at(src, "a"), at(src, "b")]);
    m.frame();
    assert_eq!(m.host.debug_stop().expect("a").entity, m.actors[0]);
    m.resume(true);
    let s = m.host.debug_stop().expect("passo");
    assert_eq!(s.frame.expect("pilha").line, at(src, "w"));
    m.resume(true);
    assert!(!m.host.is_held(), "o Next line ficou à espera do ph2d.wait");
    m.frame();
    let s = m.host.debug_stop().expect("o outro bate no b");
    assert_eq!((s.entity, s.why), (m.actors[1], StopKind::Breakpoint));
    m.resume(false);
    m.frame();
    let s = m
        .host
        .debug_stop()
        .expect("o Next line do 1.º acaba depois da espera");
    assert_eq!((s.entity, s.why.clone()), (m.actors[0], StopKind::Step));
    assert_eq!(s.frame.expect("pilha").line, at(src, "after"));
}

#[test]
fn depurador_um_laco_numa_linha_para_em_cada_volta() {
    let src = r#"
function update(self, dt)
  local i = 0
  while i < 3 do i = i + 1 end -- @loop
  ph2d.set(self.id, "x", i)
end
"#;
    let mut m = Mesa::new(src, 1);
    m.host.set_breakpoints(m.id, vec![at(src, "loop")]);
    m.frame();
    let mut seen = Vec::new();
    while let Some(s) = m.host.debug_stop() {
        let f = s.frame.expect("pilha");
        assert_eq!(f.line, at(src, "loop"));
        let i = f.locals.iter().find(|v| v.name == "i").expect("i");
        seen.push(i.value.clone());
        assert!(seen.len() < 10, "{seen:?}");
        m.resume(false);
    }
    assert_eq!(
        seen,
        ["0", "1", "2", "3"],
        "cada volta pára depois do Continue"
    );
    assert_eq!(m.x(0), 3.0);
}
