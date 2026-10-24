//! ⭐ A cena 8 ensina o que diz: o `aventura.luau` corrido pela ponte do jogo, com a biblioteca REAL
//! (as duas imagens pelos ficheiros) — a porta abre pelo nome, o máximo sobe, a moeda some, a câmera
//! aproxima, e rebobinar devolve tudo. A vida de agora (dano e cura) é da física: provada na corrida
//! conduzida (`docs/CodeEditor/ferramentas/conduz_propriedades/`).

use super::*;
use crate::component_registry_for_tests::registo;
use crate::script_bridge::{ScriptDoors, frame, rewind};
use crate::script_image::{ImageLibrary, ImageOverride};
use ph2d_preview_drive::PreviewDrive;
use ph2d_script::ScriptHost;

struct Corrida {
    sim: SimWorld,
    host: ScriptHost,
    drive: PreviewDrive,
    db: AssetDb,
    atlas: BTreeMap<u32, AssetId>,
    pecas: Vec<Entity>,
    tick: u64,
}

fn corrida() -> Corrida {
    let (mut sim, mut cena, mut mapa) = (
        SimWorld::new(),
        VecScene::default(),
        VecEntityMap::default(),
    );
    let pecas = monta(&mut sim, &mut cena, &mut mapa);
    let db = AssetDb::new();
    let mut atlas = BTreeMap::new();
    for (i, nome) in PORTAS.iter().enumerate() {
        let id = db
            .load_png_path(&pasta_das_portas().join(nome))
            .expect("a imagem está no repo");
        atlas.insert(100 + u32::try_from(i).expect("dois"), id);
    }
    let porta = sim
        .world_mut()
        .remove_resource::<PortaPorPintar>()
        .expect("a marca")
        .0;
    sim.world_mut()
        .get_mut::<Sprite>(porta)
        .expect("sprite")
        .source = SpriteSource::Atlas { key: 100 };
    Corrida {
        sim,
        host: ScriptHost::new().expect("vm"),
        drive: PreviewDrive::default(),
        db,
        atlas,
        pecas,
        tick: 0,
    }
}

impl Corrida {
    fn corre(&mut self, segundos: f64) {
        let reg = registo();
        let (mut vec_scene, mut vec_entities) = crate::instance_docs::empty_docs();
        let tags = ph2d_tags::TagTree::new();
        for _ in 0..(segundos * 60.0) as usize {
            self.tick += 1;
            let mut portas = ScriptDoors {
                registry: &reg,
                tags: &tags,
                docs: crate::instance_docs::OwnedDocs {
                    vec_scene: &mut vec_scene,
                    vec_entities: &mut vec_entities,
                },
                tick: self.tick,
                images: ImageLibrary::of(&self.db, &self.atlas),
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
            assert!(f.failed.is_empty(), "o script parou: {:?}", f.failed);
        }
    }

    fn x(&self, i: usize) -> f32 {
        self.sim
            .world()
            .get::<Transform>(self.pecas[i])
            .expect("pose")
            .translation
            .x
    }
    fn max(&self) -> f32 {
        self.sim
            .world()
            .get::<Health>(self.pecas[0])
            .expect("vida")
            .max
    }
    fn altura(&self) -> f32 {
        self.sim
            .world()
            .get::<GameCamera>(self.pecas[4])
            .expect("câmera")
            .height_world
    }
    fn aberta(&self) -> bool {
        self.sim
            .world()
            .get::<ImageOverride>(self.pecas[3])
            .is_some()
    }
    /// O nome por que o jogo acha a porta agora, e o do documento.
    fn nomes_da_porta(&self) -> (String, String) {
        let w = self.sim.world();
        let e = self.pecas[3];
        (
            ph2d_ecs::name_in_run(w, e).expect("nome").to_owned(),
            w.get::<Name>(e).expect("nome").as_str().to_owned(),
        )
    }
    fn moeda_visivel(&self) -> bool {
        self.sim
            .world()
            .get::<Visibility>(self.pecas[2])
            .is_none_or(|v| !v.hidden)
    }
}

#[test]
fn a_porta_abre_pelo_nome_o_maximo_sobe_a_camera_aproxima_e_rebobinar_devolve() {
    let mut c = corrida();
    let nomes: Vec<_> = c
        .pecas
        .iter()
        .map(|&e| {
            c.sim
                .world()
                .get::<Name>(e)
                .expect("nome")
                .as_str()
                .to_owned()
        })
        .collect();
    assert_eq!(
        nomes,
        [
            "Hero", "Spikes", "Coin", "Door", "Camera", "Life", "Life Max"
        ]
    );

    c.corre(1.0);
    assert!(c.x(0) > AT[0][0], "o herói anda sozinho");
    assert_eq!(
        (c.max(), c.aberta(), c.altura()),
        (VIDA, false, ALTURA),
        "antes da moeda e da porta"
    );
    assert!(c.moeda_visivel());

    c.corre(2.5);
    assert!(c.x(0) > AT[2][0], "passou a moeda");
    assert_eq!(c.max(), VIDA + 50.0, "a moeda sobe o máximo");
    assert!(!c.moeda_visivel(), "e some");
    assert!(!c.aberta(), "a porta ainda não");

    c.corre(6.0);
    assert!(c.aberta(), "a porta trocou de imagem pelo nome");
    // ⭐ O 11: a câmera só aproxima se ACHAR a porta pelo nome novo — e o documento fica «Door».
    assert_eq!(
        c.nomes_da_porta(),
        ("Door Open".to_owned(), "Door".to_owned()),
        "o script renomeou a porta na corrida, e só na corrida"
    );
    assert_eq!(
        c.altura(),
        5.0,
        "a câmera achou a porta pelo nome novo e aproximou até 5 m"
    );
    assert_eq!(c.x(0), AT[3][0], "o herói entrou na porta e parou");

    rewind(&mut c.host, &mut c.sim, &mut c.drive);
    assert_eq!(
        (c.x(0), c.max(), c.aberta(), c.altura()),
        (AT[0][0], VIDA, false, ALTURA),
        "rebobinar devolve tudo"
    );
    assert!(c.moeda_visivel());
    assert_eq!(c.x(4), AT[4][0], "a câmera volta ao lugar");
    assert_eq!(
        c.nomes_da_porta().0,
        "Door",
        "rebobinar devolve o nome do artista"
    );
}
