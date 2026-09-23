//! ⭐⭐⭐ **O gate da migração v128 → v129** (as TAGS, TOP-20 #9) — e ele guarda os **BYTES**, nunca
//! o tipo congelado.
//!
//! ⚠️ **A razão é a mesma do `the_frozen_v95_bytes_still_load`:** o [`ProjectFileV128`] referencia os
//! tipos VIVOS nos campos que não mudaram, o que o mantém curto e o faz apodrecer em silêncio se um
//! deles mudar de forma — os dois lados mudariam juntos e nada ficaria vermelho. Aqui os bytes são
//! montados **campo a campo, sem passar pelo tipo**, e é isso que torna o gate capaz de ver a deriva.

use crate::project_migrate::{ProjectFileV128, SignalActionSplit, migrate_v128_to_v129};
use ph2d_ecs::StableId;
use ph2d_ecs::scene::{EntitySnapshotRow, WorldSnapshot};

/// Os bytes de um `SignalActions` **v128** com uma linha — os mesmos do gate do `ph2d-ecs`:
/// `[("botao", "Parede", Hide, "")]`, sem o `target_by` que a W2 apendou.
const ACCOES_V128: &[u8] = &[
    0x01, 0x05, b'b', b'o', b't', b'a', b'o', 0x06, b'P', b'a', b'r', b'e', b'd', b'e', 0x03, 0x00,
];

/// **Um ficheiro v128 com um objecto que tem uma tabela de acções.**
///
/// ⚠️ Montado como TUPLO, campo a campo, na ordem exacta da v128 — o formato do ficheiro é
/// `(versão, corpo)`, e os `Vec` vazios codificam igual seja qual for o tipo dos elementos.
fn v128_bytes() -> Vec<u8> {
    v128_bytes_com(vec![ph2d_asset::ComponentBlob {
        type_id: ph2d_ecs::scene::stable_type_id("ph2d::ecs::SignalActions"),
        data: ACCOES_V128.to_vec(),
    }])
}

/// **Um ficheiro v128 com UM objecto que carrega `blobs`** — a mesma montagem campo a campo, para
/// os gates que precisam de outro componente no objecto.
fn v128_bytes_com(blobs: Vec<ph2d_asset::ComponentBlob>) -> Vec<u8> {
    let world = WorldSnapshot {
        version: WorldSnapshot::VERSION,
        entities: vec![std::sync::Arc::new(EntitySnapshotRow {
            id: StableId(1),
            components: blobs,
            parent: None,
        })],
    };
    let body = (
        (
            world,
            ph2d_vec_scene::VecScene::new(),
            ph2d_flip::FlipDoc::new(),
            ph2d_guides::GuideSet::default(),
            ph2d_ui_state::StateSets::default(),
            // A biblioteca: os catálogos (bytes) e as lápides.
            (Vec::<u8>::new(), Vec::<[u8; 32]>::new()),
        ),
        Vec::<u8>::new(),                                     // assets
        Vec::<u8>::new(),                                     // painted
        String::new(),                                        // motion
        Vec::<u8>::new(),                                     // timeline
        ph2d_physics_ecs::PhysicsSettings::default(),         // physics
        Vec::<u8>::new(),                                     // tokens
        crate::project_settings::collect(Default::default()), // settings
        Vec::<u8>::new(),                                     // sculpt
        Vec::<u8>::new(),                                     // baked_forms
        ph2d_physics_ecs::TapeWire::default(),                // player_tape
        Vec::<u8>::new(),                                     // sprite_pixels
        7u64,                                                 // stable_id_counter
        ph2d_input::InputMap::default(),                      // input_map
        Vec::<u8>::new(),                                     // pattern_art
    );
    postcard::to_allocvec(&(128u32, body)).expect("serializa um v128")
}

/// ⭐⭐⭐ **Um v128 lê-se pelo tipo congelado, e cada tabela de acções sai com o alvo POR NOME.**
///
/// **Mutações que devem sangrar:** a travessia a não reescrever o blob (o tipo vivo não o lê) · a
/// migração a escrever `Tagged` · a árvore de tags a não nascer vazia.
#[test]
fn a_frozen_v128_file_migrates_its_signal_actions() {
    let bytes = v128_bytes();
    let (_, antigo): (u32, ProjectFileV128) =
        postcard::from_bytes(&bytes).expect("um v128 le-se pelo tipo congelado");
    let m = migrate_v128_to_v129(antigo);
    assert_eq!(
        m.actions,
        SignalActionSplit {
            tables: 1,
            unreadable: 0
        }
    );
    assert!(
        m.file.state.tags.is_empty(),
        "um ficheiro v128 nao tem arvore de tags a declarar"
    );
    let data = &m.file.state.world.entities[0].components[0].data;
    let accoes: ph2d_ecs::SignalActions =
        postcard::from_bytes(data).expect("o blob reescrito le-se com o tipo VIVO");
    assert_eq!(accoes.0.len(), 1);
    assert_eq!(accoes.0[0].on, "botao");
    assert_eq!(accoes.0[0].target, "Parede");
    assert_eq!(accoes.0[0].verb, ph2d_ecs::SignalVerb::Hide);
    assert_eq!(accoes.0[0].target_by, ph2d_ecs::SignalTarget::Named);
}

/// ⛔ **O CONTROLO do degrau: o tipo VIVO não lê um v128.**
///
/// ⚠️ Ele não estoura necessariamente — o campo novo cai no MEIO do fluxo e os campos seguintes
/// deslizam —, e é por isso que a asserção é sobre um VALOR conhecido (`stable_id_counter`) e não
/// sobre um `Err`: *um ficheiro que abre com os campos trocados é o modo de falha que o degrau
/// existe para impedir.*
#[test]
fn the_live_type_cannot_read_a_v128() {
    let bytes = v128_bytes();
    let vivo = postcard::from_bytes::<(u32, crate::project::ProjectFile)>(&bytes);
    assert!(
        vivo.is_err() || vivo.is_ok_and(|(_, f)| f.stable_id_counter != 7),
        "o tipo VIVO leu um v128 inteiro e correcto — entao o degrau nao seria preciso"
    );
}

/// ⛔⛔ **Um v128 com uma CÂMERA DO JOGO abre o mundo inteiro, e a câmera nasce com o dolly a `0`**
/// (auditoria 26, §2.2).
///
/// O `dolly` foi apendado à `GameCamera` no degrau `167 → 168`, e o postcard é posicional: o blob
/// de CINCO campos que um v128 gravou não decodifica com o tipo vivo, e o `snapshot_to_world` pára
/// na primeira linha que falha — *o ficheiro abria sem mundo nenhum*. Os bytes vêm de um TUPLO
/// montado à mão, nunca do tipo congelado, pela razão do cabeçalho.
///
/// **Mutações que devem sangrar:** a travessia a não chamar a migração da câmera (o restauro falha
/// e a contagem lê `0`) · o dolly a nascer noutro valor.
#[test]
fn a_frozen_v128_camera_migrates_and_the_world_restores() {
    let camera_v128 =
        postcard::to_allocvec(&(12.5_f32, [1.0_f32, -2.0_f32], 3_i32, true, 0xFF_u32))
            .expect("os cinco campos da GameCamera v128");
    let bytes = v128_bytes_com(vec![ph2d_asset::ComponentBlob {
        type_id: ph2d_ecs::scene::stable_type_id("ph2d::ecs::GameCamera"),
        data: camera_v128.clone(),
    }]);
    // O CONTROLO: o tipo vivo não lê os cinco campos — sem isso o gate não contém o fenómeno.
    assert!(
        postcard::from_bytes::<ph2d_ecs::GameCamera>(&camera_v128).is_err(),
        "o tipo vivo leu uma GameCamera v128 — o degrau da câmera deixou de ser preciso"
    );
    let (_, antigo): (u32, ProjectFileV128) =
        postcard::from_bytes(&bytes).expect("um v128 le-se pelo tipo congelado");
    let m = migrate_v128_to_v129(antigo);
    assert_eq!(
        m.cameras,
        SignalActionSplit {
            tables: 1,
            unreadable: 0
        }
    );
    let mut world = bevy_ecs::world::World::new();
    let reg = crate::init::build_component_registry();
    let spawned = ph2d_ecs::scene::snapshot_to_world(&mut world, &m.file.state.world, &reg)
        .expect("o mundo migrado restaura-se");
    assert_eq!(spawned.len(), 1, "o objecto da câmera voltou");
    let cam = world
        .get::<ph2d_ecs::GameCamera>(spawned[0])
        .expect("a câmera voltou com o objecto");
    assert_eq!(cam.height_world, 12.5);
    assert_eq!(cam.offset, [1.0, -2.0]);
    assert_eq!(cam.priority, 3);
    assert!(cam.active);
    assert_eq!(cam.cull_mask, 0xFF);
    assert_eq!(
        cam.dolly, 0.0,
        "o dolly de um ficheiro velho é a identidade"
    );
}
