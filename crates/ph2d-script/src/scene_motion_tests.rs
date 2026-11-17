//! Ciclo 19: a tabela `motion` do Luau (`motion.set/get/color/fire`) sobre a VM REAL. Filho do
//! `scene_tests` (os arnêses são dele).

use super::*;

const MOTION_WRITES: &str = r#"
function update(self, dt)
  motion.set("vida", 0.25)
  motion.set("ligado", true)
  motion.color("time", 1, 0.5, 0)
  motion.fire("dano")
end
"#;

/// ⭐ O que o script escreve na tabela `motion` chega ao recurso do jogo no fim do tique — e cada
/// `fire` é um disparo (a contagem sobe uma vez por tique).
#[test]
fn a_script_writes_the_motion_inputs_of_the_game() {
    use ph2d_ecs::MotionInputs;
    use ph2d_ecs::motion_inputs::{Escritor, Valor};
    let mut host = ScriptHost::new().expect("vm");
    let mut sim = SimWorld::new();
    let path = script_file("motion", MOTION_WRITES);
    actor(sim.world_mut(), 1, &path, &[]);
    let r = frame(&mut host, &mut sim, 1.0 / 60.0);
    assert!(r.failed.is_empty(), "{:?}", r.failed);
    frame(&mut host, &mut sim, 1.0 / 60.0);
    let m = sim
        .world()
        .get_resource::<MotionInputs>()
        .expect("o recurso nasceu");
    assert_eq!(
        m.valor_de("vida", &Escritor::Script),
        Some(Valor::Numero(0.25))
    );
    assert_eq!(
        m.valor_de("ligado", &Escritor::Script),
        Some(Valor::Numero(1.0))
    );
    assert_eq!(
        m.valor_de("time", &Escritor::Script),
        Some(Valor::Cor([1.0, 0.5, 0.0, 1.0]))
    );
    assert_eq!(m.disparos().collect::<Vec<_>>(), vec![("dano", vec![2])]);
}

/// Um nome vazio ou um valor que não é número PARA o objecto, com a mensagem (a lei do
/// `ph2d.emit`), e as escritas do gancho que falhou não chegam.
#[test]
fn a_blank_motion_name_stops_the_object_and_its_writes_do_not_land() {
    use ph2d_ecs::MotionInputs;
    let mut host = ScriptHost::new().expect("vm");
    let mut sim = SimWorld::new();
    let path = script_file(
        "motion_mau",
        "function update(self, dt)\n  motion.set(\"vida\", 0.5)\n  motion.set(\"\", 1)\nend\n",
    );
    actor(sim.world_mut(), 1, &path, &[]);
    let r = frame(&mut host, &mut sim, 1.0 / 60.0);
    assert_eq!(r.failed.len(), 1);
    assert!(
        r.failed[0].1.message.contains("needs a name"),
        "{}",
        r.failed[0].1
    );
    assert!(sim.world().get_resource::<MotionInputs>().is_none());
}

/// ⭐ MI-9: `motion.get` devolve o que o `value.input` lê (a 1.ª linha) do começo do tique: `nil` sem escritor,
/// a cor como tabela `color`, e o `motion.set` do próprio tique só no seguinte.
#[test]
fn motion_get_reads_what_the_graph_reads_from_the_start_of_the_tick() {
    use ph2d_ecs::MotionInputs;
    use ph2d_ecs::motion_inputs::{Escritor, Valor};
    let mut host = ScriptHost::new().expect("vm");
    let mut sim = SimWorld::new();
    let src = "function update(self, dt)\n  motion.set(\"eco\", 0.75)\n  \
               ph2d.set(self.id, \"x\", motion.get(\"eco\") or -1)\n  \
               local c = motion.get(\"cor\")\n  ph2d.set(self.id, \"y\", c and c.g or -1)\nend\n";
    let path = script_file("motion_get", src);
    let e = actor(sim.world_mut(), 1, &path, &[]);
    let mut m = MotionInputs::default();
    let lamp = Escritor::Objeto {
        nome: "Lamp".into(),
        bits: 1,
    };
    m.set("cor", lamp, Valor::Cor([1.0, 0.5, 0.0, 1.0]));
    let zed = Escritor::Objeto {
        nome: "Zed".into(),
        bits: 2,
    };
    m.set("cor", zed, Valor::Cor([0.0, 0.0, 0.0, 1.0]));
    sim.world_mut().insert_resource(m);
    let r = frame(&mut host, &mut sim, 1.0 / 60.0);
    assert!(r.failed.is_empty(), "{:?}", r.failed);
    let y = |w: &World| w.get::<Transform>(e).expect("pose").translation.y;
    assert_eq!(
        (x(sim.world(), e), y(sim.world())),
        (-1.0, 0.5),
        "nil no 1.º tique (mesmo depois do set); a cor da 1.ª linha"
    );
    frame(&mut host, &mut sim, 1.0 / 60.0);
    assert_eq!(x(sim.world(), e), 0.75, "o set do tique anterior");
}

/// ⭐ MI-5: `motion.set` aceita o par `ph2d.vec2` (chega como `Valor::Vetor`), e `motion.get` devolve-o
/// no mesmo idioma (`.x`, `.y`) no tique seguinte.
#[test]
fn motion_set_and_get_carry_a_pair_as_vec2() {
    use ph2d_ecs::MotionInputs;
    use ph2d_ecs::motion_inputs::{Escritor, Valor};
    let mut host = ScriptHost::new().expect("vm");
    let mut sim = SimWorld::new();
    let src = "function update(self, dt)\n  local p = motion.get(\"andar\")\n  \
               ph2d.set(self.id, \"x\", p and p.x or -1)\n  ph2d.set(self.id, \"y\", p and p.y or -1)\n  \
               motion.set(\"andar\", ph2d.vec2(3, -4))\nend\n";
    let e = actor(sim.world_mut(), 1, &script_file("motion_par", src), &[]);
    let r = frame(&mut host, &mut sim, 1.0 / 60.0);
    assert!(r.failed.is_empty(), "{:?}", r.failed);
    let m = sim.world().get_resource::<MotionInputs>().expect("recurso");
    assert_eq!(
        m.valor_de("andar", &Escritor::Script),
        Some(Valor::Vetor([3.0, -4.0]))
    );
    frame(&mut host, &mut sim, 1.0 / 60.0);
    let y = sim.world().get::<Transform>(e).expect("pose").translation.y;
    assert_eq!((x(sim.world(), e), y), (3.0, -4.0), "o par volta como vec2");
}
