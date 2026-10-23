//! Os portões das mutações que sobreviveram no fecho de 20/10 — mesmos ajudantes do pai.

use super::*;

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

/// O inclinar do cartão Transform (Skew X/Y) é pose do script, em RADIANOS como o `Transform` o
/// guarda: o `set` chega ao `Transform` (com a guarda do Inspector perto do quarto de volta) e o
/// `get` do tique seguinte devolve-o; a ajuda do `PoseField` nomeia todo campo da lista.
#[test]
fn o_inclinar_e_pose_do_script_em_radianos() {
    let mut sim = SimWorld::new();
    let e = actor(
        sim.world_mut(),
        1,
        "inclina.luau",
        r#"
function update(self)
  self.n = (self.n or 0) + 1
  if self.n == 1 then
    ph2d.set(self.id, "skew_x", 0.3)
    ph2d.set(self.id, "skew_y", 10)
  else
    ph2d.set(self.id, "x", ph2d.get(self.id, "skew_x"))
    ph2d.set(self.id, "y", ph2d.get(self.id, "skew_y"))
  end
end
"#,
    );
    let mut host = ScriptHost::new().expect("vm");
    let r = tick(&mut host, &mut sim);
    assert!(r.failed.is_empty(), "{:?}", r.failed);
    let t = pose(sim.world(), e);
    assert_eq!(t.skew_x, 0.3, "o set chega ao Transform, em radianos");
    assert_eq!(t.skew_y, Transform::SKEW_LIMIT, "a guarda do Inspector");
    tick(&mut host, &mut sim);
    let t = pose(sim.world(), e);
    assert_eq!(t.translation.x, 0.3, "o get devolve o que o set pôs");
    assert_eq!(t.translation.y, Transform::SKEW_LIMIT);
    let ajuda = crate::api_help::API_HELP
        .iter()
        .find(|h| h.key == "type.PoseField")
        .expect("página do PoseField");
    for f in crate::io::POSE_FIELDS {
        assert!(
            ajuda.more.contains(&format!("`{f}`")),
            "a ajuda não diz `{f}`"
        );
        assert!(crate::io::POSE_FIELD_UNION.contains(&format!("\"{f}\"")));
    }
}
