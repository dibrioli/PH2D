//! ⭐ **Um script toca as propriedades do Inspector** (doc CodeEditor 08 §3) — pela porta da cena
//! real ([`crate::script_bridge::ScriptDoors`]) e o registo do produto.

use crate::component_registry_for_tests::registo;
use crate::script_bridge::{ScriptDoors, frame, rewind};
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{Entity, SimWorld, StableId, Transform};
use ph2d_physics_ecs::Health;
use ph2d_preview_drive::PreviewDrive;
use ph2d_script::{LuauScript, ProjectScripts, ScriptHost};

struct Cena {
    reg: ComponentRegistry,
    sim: SimWorld,
    host: ScriptHost,
    drive: PreviewDrive,
    heroi: Entity,
}

fn cena(fonte: &str) -> Cena {
    let mut sim = SimWorld::new();
    let id = sim
        .world_mut()
        .get_resource_or_init::<ProjectScripts>()
        .insert("heroi.luau", fonte);
    let heroi = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            StableId(7),
            Health::default(),
            LuauScript::library(id),
        ))
        .id();
    Cena {
        reg: registo(),
        sim,
        host: ScriptHost::new().expect("vm"),
        drive: PreviewDrive::default(),
        heroi,
    }
}

impl Cena {
    fn quadro(&mut self, a_correr: bool) -> crate::script_bridge::ScriptFrame {
        let (mut vec_scene, mut vec_entities) = crate::instance_docs::empty_docs();
        let mut portas = ScriptDoors {
            registry: &self.reg,
            docs: crate::instance_docs::OwnedDocs {
                vec_scene: &mut vec_scene,
                vec_entities: &mut vec_entities,
            },
            tick: 1,
        };
        frame(
            &mut self.host,
            &mut self.sim,
            &mut self.drive,
            a_correr,
            1,
            1.0 / 60.0,
            &[],
            &mut portas,
        )
    }

    fn vida(&self) -> &Health {
        self.sim.world().get::<Health>(self.heroi).expect("vida")
    }

    fn falhas(f: &crate::script_bridge::ScriptFrame) -> String {
        f.failed
            .iter()
            .map(|(_, d)| d.message.clone())
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

const SOBE_O_MAX: &str = r#"
function update(self)
  ph2d.set(self.id, "health.max", ph2d.get(self.id, "health.max") + 10)
end
"#;

#[test]
fn o_script_escreve_e_rebobinar_devolve_o_do_artista() {
    let mut c = cena(SOBE_O_MAX);
    let autorado = c.vida().max;
    let f = c.quadro(true);
    assert!(f.failed.is_empty(), "{}", Cena::falhas(&f));
    assert_eq!(c.vida().max, autorado + 10.0, "pousou no fim do tique");
    c.quadro(true);
    assert_eq!(c.vida().max, autorado + 20.0);
    // A captura do Ctrl+Z e do gravar vê o autorado; o mundo volta ao vivo a seguir.
    let vivo = c.drive.substitute(&mut c.sim);
    assert_eq!(c.vida().max, autorado, "a corrida não é documento");
    PreviewDrive::restore(&mut c.sim, &vivo);
    assert_eq!(c.vida().max, autorado + 20.0);
    rewind(&mut c.host, &mut c.sim, &mut c.drive);
    assert_eq!(c.vida().max, autorado, "rebobinar devolve o do artista");
    assert!(c.drive.fields.is_empty());
}

#[test]
fn outra_mao_na_pausa_passa_a_ser_o_autorado() {
    let mut c = cena(SOBE_O_MAX);
    c.quadro(true);
    // O Inspector, com a corrida pausada.
    c.sim.world_mut().get_mut::<Health>(c.heroi).unwrap().max = 500.0;
    c.quadro(false);
    rewind(&mut c.host, &mut c.sim, &mut c.drive);
    assert_eq!(c.vida().max, 500.0, "o que o artista pôs na pausa fica");
}

#[test]
fn o_script_obedece_aos_limites_do_inspector() {
    let mut c = cena(
        r#"
function update(self)
  ph2d.set(self.id, "health.max", -5)
  ph2d.set(self.id, "health.armor_share", 3)
  ph2d.set(self.id, "health.team", "  red ")
end
"#,
    );
    let f = c.quadro(true);
    assert!(f.failed.is_empty(), "{}", Cena::falhas(&f));
    let h = c.vida();
    assert_eq!((h.max, h.armor_percent, h.team.as_str()), (0.0, 1.0, "red"));
}

#[test]
fn um_nome_ou_um_valor_errado_para_o_objecto_e_diz_porque() {
    for (linha, espera) in [
        (r#"ph2d.set(self.id, "health.maxx", 1)"#, "'health' has no property 'maxx'"),
        (r#"ph2d.get(self.id, "game_camera.height")"#, "has no 'game_camera'"),
        (r#"ph2d.set(self.id, "health.max", "muito")"#, "'health.max' takes a number"),
        (r#"ph2d.set(self.id, "health.seed", 1.5)"#, "takes a whole number"),
        (r#"ph2d.get(self.id, "vida.max")"#, "no component named 'vida'"),
    ] {
        let mut c = cena(&format!("function update(self)\n  {linha}\nend\n"));
        let antes = c.vida().clone();
        let f = c.quadro(true);
        let msg = Cena::falhas(&f);
        assert!(msg.contains(espera), "{linha}: {msg}");
        assert_eq!(c.vida(), &antes, "{linha}: nada pousou");
    }
}

#[test]
fn as_propriedades_de_um_objecto_sao_a_pose_e_as_dos_seus_componentes() {
    let mut c = cena(
        r#"
function init(self)
  local nomes = ph2d.properties(self.id)
  assert(nomes[1] == "x", nomes[1])
  assert(table.find(nomes, "health.max"))
  assert(table.find(nomes, "health.number_color"))
  assert(not table.find(nomes, "game_camera.height"))
  local cor = ph2d.get(self.id, "health.number_color")
  assert(cor.__kind == "color", "uma cor chega como ph2d.color")
  ph2d.set(self.id, "health.number_color", ph2d.color(1, 0, 0))
end
"#,
    );
    let f = c.quadro(true);
    assert!(f.failed.is_empty(), "{}", Cena::falhas(&f));
    assert_eq!(c.vida().numbers_color, [1.0, 0.0, 0.0, 1.0]);
}

/// ⭐ Decisão do dono nº 9: a vida de AGORA escreve-se directo E por dano/cura — os dois são pedidos à
/// ponte da física (a caixa `PedidosDeVida`), pela ordem em que o script os escreveu.
#[test]
fn a_vida_de_agora_e_o_dano_sao_pedidos_a_fisica_na_ordem_do_script() {
    use ph2d_physics_ecs::{PedidoDeVida, PedidosDeVida};
    let mut c = cena(
        r#"
function init(self)
  assert(ph2d.get(self.id, "health.now") == 100, "antes da corrida: a do nascimento")
  assert(table.find(ph2d.properties(self.id), "health.shield_now"))
  ph2d.set(self.id, "health.now", 10)
  ph2d.damage(self.id, 5)
  ph2d.heal(self.id, 2)
  ph2d.set(self.id, "health.shield_now", -3)
end
"#,
    );
    let f = c.quadro(true);
    assert!(f.failed.is_empty(), "{}", Cena::falhas(&f));
    let caixa = &c.sim.world().resource::<PedidosDeVida>().0;
    let e = c.heroi;
    assert_eq!(
        caixa,
        &[
            (e, PedidoDeVida::Define(10.0)),
            (e, PedidoDeVida::Dano(5.0)),
            (e, PedidoDeVida::Cura(2.0)),
            (e, PedidoDeVida::Escudo(0.0)),
        ]
    );
    assert!(c.drive.fields.is_empty(), "a vida de agora é da corrida: o rebobinar já a refaz");
}

#[test]
fn dano_sem_vida_ou_quantia_invalida_para_o_objecto() {
    for (linha, espera) in [
        (r#"ph2d.damage(self.id, 0)"#, "above 0"),
        (r#"ph2d.heal(self.id, -1)"#, "above 0"),
        (r#"ph2d.damage(ph2d.spawn(), 3)"#, "has no 'health'"),
        (r#"ph2d.set(self.id, "health.now", "x")"#, "'health.now' takes a number"),
    ] {
        let mut c = cena(&format!("function update(self)\n  {linha}\nend\n"));
        let msg = Cena::falhas(&c.quadro(true));
        assert!(msg.contains(espera), "{linha}: {msg}");
    }
}

/// ⭐ Um NEWTYPE (`order_in_layer` — o nome curto —, a chave `"."`) e um ELEMENTO (`9_slice.top`, `"borders[1]"`)
/// também se escrevem a correr e voltam ao do artista: o livro da corrida resolve a chave pela
/// MESMA lei do nome (`property::slot_in`), e a lei do valor do Inspector vale para o script.
#[test]
fn um_newtype_e_um_elemento_tambem_voltam_ao_do_artista() {
    let mut c = cena(
        r#"
function update(self)
  ph2d.set(self.id, "order_in_layer", 7)
  ph2d.set(self.id, "9_slice.top", -4)
  ph2d.set(self.id, "9_slice.left", 12)
end
"#,
    );
    let autorado = ph2d_ecs::SliceNine {
        borders: [1.0, 2.0, 3.0, 4.0],
        ..ph2d_ecs::SliceNine::INERT
    };
    c.sim
        .world_mut()
        .entity_mut(c.heroi)
        .insert((ph2d_ecs::OrderInLayer(3), autorado));
    let f = c.quadro(true);
    assert!(f.failed.is_empty(), "{}", Cena::falhas(&f));
    let w = c.sim.world();
    assert_eq!(w.get::<ph2d_ecs::OrderInLayer>(c.heroi).unwrap().0, 7);
    let s = w.get::<ph2d_ecs::SliceNine>(c.heroi).unwrap();
    assert_eq!(s.borders, [12.0, 0.0, 3.0, 4.0], "a borda negativa saneia-se como no cartão");
    rewind(&mut c.host, &mut c.sim, &mut c.drive);
    let w = c.sim.world();
    assert_eq!(w.get::<ph2d_ecs::OrderInLayer>(c.heroi).unwrap().0, 3);
    assert_eq!(w.get::<ph2d_ecs::SliceNine>(c.heroi).unwrap().borders, autorado.borders);
}
