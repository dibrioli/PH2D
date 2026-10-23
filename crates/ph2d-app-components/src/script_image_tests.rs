//! ⭐ `sprite.image` (doc CodeEditor 08 d4): o script troca a imagem pelo NOME, e é da corrida.

use super::*;
use crate::component_registry_for_tests::registo;
use crate::script_bridge::{ScriptDoors, frame, rewind};
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{StableId, Transform};
use ph2d_preview_drive::PreviewDrive;
use ph2d_script::{LuauScript, ProjectScripts, ScriptHost};

fn id(n: u8) -> AssetId {
    AssetId::from_digest([n; 32])
}

const FECHADA: u8 = 1;
const ABERTA: u8 = 2;

struct Cena {
    reg: ComponentRegistry,
    sim: SimWorld,
    host: ScriptHost,
    drive: PreviewDrive,
    porta: Entity,
}

fn cena(corpo: &str) -> Cena {
    let mut sim = SimWorld::new();
    let fonte = format!("function init(self)\n{corpo}\nend\n");
    let script = sim
        .world_mut()
        .get_resource_or_init::<ProjectScripts>()
        .insert("porta.luau", &fonte);
    let porta = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            StableId(3),
            Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            SpritePixels(id(FECHADA)),
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
        let nomes: BTreeMap<AssetId, String> = [
            (id(FECHADA), "door_closed".to_owned()),
            (id(ABERTA), "door_open".to_owned()),
        ]
        .into_iter()
        .collect();
        let mut portas = ScriptDoors {
            registry: &self.reg,
            tags: &tags,
            docs: crate::instance_docs::OwnedDocs {
                vec_scene: &mut vec_scene,
                vec_entities: &mut vec_entities,
            },
            tick: 1,
            images: ImageLibrary::fixed(nomes),
        };
        let f = frame(
            &mut self.host,
            &mut self.sim,
            &mut self.drive,
            true,
            1,
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

    fn override_(&self) -> Option<ImageOverride> {
        self.sim.world().get::<ImageOverride>(self.porta).copied()
    }
}

#[test]
fn o_script_troca_a_imagem_pelo_nome_e_rebobinar_devolve_a_do_artista() {
    let mut c = cena(
        r#"
  assert(ph2d.get(self.id, "sprite.image") == "door_closed", "a autorada")
  assert(table.find(ph2d.properties(self.id), "sprite.image"))
  ph2d.set(self.id, "sprite.image", "door_open")
"#,
    );
    let falhas = c.quadro();
    assert!(falhas.is_empty(), "{falhas}");
    let o = c.override_().expect("o override pousou no fim do tique");
    assert_eq!((o.asset, o.texture_id), (id(ABERTA), 0), "por subir");
    assert!(
        c.drive.fields.is_empty(),
        "não entra no livro da corrida (nem no Ctrl+Z)"
    );
    let reg = registo();
    assert!(
        reg.iter()
            .all(|e| !e.canonical_name.contains("ImageOverride")),
        "não registado ⇒ nunca gravado"
    );
    assert_eq!(
        c.sim.world().get::<SpritePixels>(c.porta),
        Some(&SpritePixels(id(FECHADA))),
        "o documento não mudou"
    );
    rewind(&mut c.host, &mut c.sim, &mut c.drive);
    assert_eq!(c.override_(), None, "rebobinar devolve a imagem do artista");
}

#[test]
fn nil_ou_a_imagem_autorada_tiram_o_override() {
    for linha in [
        r#"ph2d.set(self.id, "sprite.image", nil)"#,
        r#"ph2d.set(self.id, "sprite.image", "door_closed")"#,
    ] {
        let mut c = cena(&format!(
            "  ph2d.set(self.id, \"sprite.image\", \"door_open\")\n  {linha}\n  \
             assert(ph2d.get(self.id, \"sprite.image\") == \"door_closed\", \"pousa no fim\")"
        ));
        let falhas = c.quadro();
        assert!(falhas.is_empty(), "{linha}: {falhas}");
        assert_eq!(c.override_(), None, "{linha}");
    }
}

#[test]
fn um_nome_que_a_biblioteca_nao_tem_ou_um_valor_errado_diz_porque() {
    for (linha, espera) in [
        (
            r#"ph2d.set(self.id, "sprite.image", "door_ajar")"#,
            "no image named 'door_ajar' in the library",
        ),
        (
            r#"ph2d.set(self.id, "sprite.image", 3)"#,
            "'sprite.image' takes a text",
        ),
        (
            r#"ph2d.get(ph2d.spawn(), "sprite.image")"#,
            "has no 'sprite'",
        ),
    ] {
        let mut c = cena(&format!("  {linha}"));
        let falhas = c.quadro();
        assert!(falhas.contains(espera), "{linha}: {falhas}");
        assert_eq!(c.override_(), None, "{linha}: nada pousou");
    }
}

/// ⭐ O extract (sem placa): por subir, a sprite desenha a autorada com a região e a grelha; subida,
/// a imagem INTEIRA, sem região nem célula; sob a pré-visualização de uma ferramenta, a ferramenta.
#[test]
fn o_extract_desenha_a_imagem_inteira_so_depois_de_subida() {
    let mut w = World::new();
    let grelha = SpriteGrid {
        hframes: 4,
        ..SpriteGrid::SINGLE
    };
    let regiao = SpriteRegion {
        rect: [0.0, 0.0, 8.0, 8.0],
        filter_clip: false,
    };
    let mut o = ImageOverride {
        asset: id(ABERTA),
        texture_id: 0,
        premultiplied: false,
        dims: (0, 0),
    };
    let e = w.spawn((grelha, regiao, o)).id();
    let autorada = cut(&w, e, false);
    assert_eq!(
        (
            autorada.image,
            autorada.region,
            autorada.grid,
            autorada.sheet_grid
        ),
        (None, Some(regiao), grelha, Some(grelha))
    );
    o.texture_id = 42;
    o.dims = (64, 32);
    w.entity_mut(e).insert(o);
    let trocada = cut(&w, e, false);
    assert_eq!(
        (
            trocada.image,
            trocada.region,
            trocada.grid,
            trocada.sheet_grid
        ),
        (Some(o), None, SpriteGrid::SINGLE, None)
    );
    assert_eq!(cut(&w, e, true), autorada, "a ferramenta ganha");
}
