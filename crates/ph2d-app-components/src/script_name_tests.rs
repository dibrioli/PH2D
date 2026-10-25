//! ⭐ `name` (doc CodeEditor 11): o script renomeia um objecto a correr, e é da corrida.

use super::*;
use crate::component_registry_for_tests::registo;
use crate::script_bridge::{ScriptDoors, frame, rewind};
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{StableId, Transform};
use ph2d_preview_drive::PreviewDrive;
use ph2d_script::{LuauScript, ProjectScripts, ScriptHost};

struct Cena {
    reg: ComponentRegistry,
    sim: SimWorld,
    host: ScriptHost,
    drive: PreviewDrive,
    porta: Entity,
}

fn cena(fonte: &str) -> Cena {
    let mut sim = SimWorld::new();
    let script = sim
        .world_mut()
        .get_resource_or_init::<ProjectScripts>()
        .insert("porta.luau", fonte);
    let porta = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            StableId(3),
            Name::new("Door"),
            LuauScript::library(script),
        ))
        .id();
    Cena {
        reg: registo(),
        sim,
        host: ScriptHost::new().expect("vm"),
        drive: PreviewDrive::default(),
        porta,
    }
}

impl Cena {
    /// Um quadro a correr; devolve as falhas do console.
    fn quadro(&mut self) -> String {
        let (mut vec_scene, mut vec_entities) = crate::instance_docs::empty_docs();
        let tags = ph2d_tags::TagTree::new();
        let mut portas = ScriptDoors {
            registry: &self.reg,
            tags: &tags,
            docs: crate::instance_docs::OwnedDocs {
                vec_scene: &mut vec_scene,
                vec_entities: &mut vec_entities,
            },
            tick: 1,
            images: crate::script_image::ImageLibrary::none(),
        };
        let f = frame(
            &mut self.host,
            &mut self.sim,
            &mut self.drive,
            true,
            1,
            1.0 / 60.0,
            1.0 / 60.0,
            &[],
            &mut portas,
        );
        f.failed
            .iter()
            .map(|(_, d)| d.message.clone())
            .collect::<Vec<_>>()
            .join(" | ")
    }

    fn run_name(&self) -> Option<String> {
        self.sim
            .world()
            .get::<RunName>(self.porta)
            .map(|n| n.0.clone())
    }
}

#[test]
fn o_script_renomeia_o_objecto_a_correr_e_rebobinar_devolve_o_nome_do_artista() {
    let mut c = cena(
        r#"
function init(self)
  assert(ph2d.get(self.id, "name") == "Door", "o autorado")
  assert(table.find(ph2d.properties(self.id), "name"))
  ph2d.set(self.id, "name", "Porta Aberta")
end
function update(self, dt)
  self.n = (self.n or 0) + 1
  if self.n == 2 then
    assert(ph2d.get(self.id, "name") == "Porta Aberta", "o nome da corrida")
    assert(ph2d.find_by_name("Porta Aberta") == self.id, "achado pelo nome novo")
    assert(ph2d.object("Porta Aberta") == self.id)
    assert(ph2d.find_by_name("Door") == nil, "o nome velho já não acha")
  end
end
"#,
    );
    for _ in 0..3 {
        let falhas = c.quadro();
        assert!(falhas.is_empty(), "{falhas}");
    }
    assert_eq!(c.run_name().as_deref(), Some("Porta Aberta"));
    assert_eq!(
        c.sim.world().get::<Name>(c.porta),
        Some(&Name::new("Door")),
        "o documento (a identidade) não mudou"
    );
    assert!(
        c.drive.fields.is_empty(),
        "não entra no livro da corrida (nem no Ctrl+Z)"
    );
    assert!(
        registo()
            .iter()
            .all(|e| !e.canonical_name.contains("RunName")),
        "não registado ⇒ nunca gravado"
    );
    rewind(&mut c.host, &mut c.sim, &mut c.drive);
    assert_eq!(c.run_name(), None, "rebobinar devolve o nome do artista");
}

#[test]
fn o_nome_do_artista_tira_o_override_e_um_valor_errado_diz_porque() {
    let mut c = cena(
        "function init(self)\n  ph2d.set(self.id, \"name\", \"X\")\n  ph2d.set(self.id, \"name\", \"Door\")\nend\n",
    );
    let falhas = c.quadro();
    assert!(falhas.is_empty(), "{falhas}");
    assert_eq!(
        c.run_name(),
        None,
        "voltar ao nome do artista não é override"
    );
    for (linha, espera) in [
        (
            r#"ph2d.set(self.id, "name", "")"#,
            "an object's name can't be empty",
        ),
        (r#"ph2d.set(self.id, "name", 3)"#, "'name' takes a text"),
    ] {
        let mut c = cena(&format!("function init(self)\n  {linha}\nend\n"));
        let falhas = c.quadro();
        assert!(falhas.contains(espera), "{linha}: {falhas}");
        assert_eq!(c.run_name(), None, "{linha}: nada pousou");
    }
}
