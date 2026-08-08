//! **O GESTO DE ASSAR, ponta a ponta, num device de verdade.**
//!
//! Módulo irmão do [`super`] (`#[path]`, `cfg(test)`), ao lado do
//! [`super::light_measure`]: lá a medição das DUAS luzes sobre a mesma forma,
//! aqui o gesto inteiro — a tela na mesa, a malha na cena, o `drain`, e o sprite
//! que sai aceso.
//!
//! ⚠️ **O sufixo `_tests` é load-bearing**, e foi um gate que o exigiu: o
//! `texture_edit_chokepoint` proíbe `readback_individual` fora da porta de
//! alpha-mode dos image tools, e isenta os `*_tests.rs` porque *um módulo que
//! não shipa não pode largar o `premultiplied` no documento de ninguém*. Este
//! aqui lê o slot de volta para julgar PIXELS, que é observar o device — não
//! editar um sprite.
//!
//! ## Por que ele existe, e por que ele é caro
//!
//! Ele nasceu de um **PANIC** que o Enio encontrou apertando `Shift+B`. A causa
//! não era do bake: o `Globals` do `impasto_light.wgsl` tinha ganhado um campo e
//! deixado um `pad` para trás, então o uniform media 240 bytes contra os 224 do
//! Rust e o wgpu **recusava todo dispatch** da luz de impasto na GPU. Os seis
//! gates que o teriam pego moram em `ph2d-render/tests/impasto_light_gpu.rs` e
//! são `#[ignore]` — eles precisam de adapter, e a varredura de GPU daquela wave
//! rodou as crates do módulo 3D e **não** a do renderizador.
//!
//! ⚠️ **A cura de fundo é o gate SEM device** (`the_wgsl_globals_measures_exactly_the_rust_globals`,
//! na `ph2d-render`): uma incompatibilidade de ABI entre duas declarações do
//! mesmo buffer é aritmética, e aritmética não se pergunta à placa de vídeo. Este
//! aqui é a outra metade, e a que responde a pergunta que o Enio de fato fez —
//! ***o gesto funciona?*** — pelo caminho que ele usa: o `drain` do produto, com
//! o mundo, o renderizador e o mapa de atlas reais.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --release --bins sculpt3d::bake::gesture -- --ignored --nocapture
//! ```

use std::collections::BTreeMap;

use ph2d_asset::AssetDb;
use ph2d_ecs::{BakedForm as BakedFormId, Entity, SimWorld};
use ph2d_mesh::shapes::uv_sphere;
use ph2d_render::{GameRt, SpriteRenderer, TextureAtlas};

use super::super::Sculpt3dScene;

/// O lado da tela que o gate assa.
///
/// ⚠️ **Menor que os 1024 do produto de propósito**, e a escolha é sobre custo,
/// não sobre cobertura: nada do que este gate afirma depende do tamanho — o
/// uniform mal-formado recusava o dispatch em qualquer extensão, e a forma
/// acender é uma propriedade por-texel. O que o tamanho compra é a leitura de
/// volta, que é o item caro (`ph2d-mesh-render::form_plane`: 1,75 ms a 512²
/// contra 6,89 a 1024²).
const EDGE: u32 = 256;

/// **O gesto inteiro: a forma da escultura acende um sprite da cena.**
///
/// ⚠️ **O oráculo é a APARÊNCIA, e sem ele o gate seria verde sobre um bake que
/// não fez nada.** Um `drain` que devolvesse `Ok` e escrevesse a fonte de volta
/// passaria por qualquer asserção de estado — o sprite viraria `Individual`,
/// ganharia identidade, e continuaria uma chapa branca. Então o que se afirma é
/// que os PIXELS mudaram: a tela nasce branca opaca e a esfera tem de deixar
/// sombra nela.
///
/// `#[ignore]`: precisa de um adapter de GPU (não há na CI).
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_bake_gesture_lights_the_selected_sprite() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter on this machine — nothing to assert");
        return;
    };
    let mut renderer =
        SpriteRenderer::new(gpu.clone(), GameRt::FORMAT, TextureAtlas::dummy(&gpu), 8);
    let mut sim = SimWorld::new();
    let asset_db = AssetDb::new();
    let mut atlas_map = BTreeMap::new();
    // A MESMA porta que a cena de smoke usa para pôr a tela na mesa — uma
    // fixture que montasse o sprite à mão testaria um objeto que o produto não
    // produz.
    let (_, bits) = crate::image_import::spawn_blank_canvas(
        &mut sim,
        &mut renderer,
        &asset_db,
        0,
        EDGE,
        // Branco OPACO: a luz da forma MULTIPLICA, então sobre branco o que se vê
        // é ela e mais nada. É também o que torna a asserção de aparência simples
        // — qualquer texel abaixo de 255 veio da escultura.
        2,
        ph2d_core::Vec2::new(0.0, 0.0),
        100.0, // LITERAL-PX-OK: pixels por metro da fixture, nao metrica de design
        &mut atlas_map,
    )
    .expect("a tela branca da fixture");

    let mut scene = Sculpt3dScene::new(&gpu.device, uv_sphere(48, 72, 1.0), 1.0);
    let mut forms = BTreeMap::new();
    let mut pass = None;
    let mut next_id = 0u32;
    let line = super::drain(
        &mut scene,
        &mut forms,
        &mut pass,
        &mut next_id,
        &gpu,
        true,
        Some(bits),
        &mut sim,
        &mut renderer,
        &asset_db,
        &atlas_map,
    )
    .expect("o gesto foi pedido, entao ele responde alguma coisa");
    assert!(line.contains("ASSADO"), "o gesto recusou o bake: {line}");

    // ── O que o sprite virou ────────────────────────────────────────────────
    let entity = Entity::from_bits(bits);
    let sprite = sim
        .world()
        .get::<ph2d_render::Sprite>(entity)
        .expect("o sprite continua na cena");
    let ph2d_render::SpriteSource::Individual { texture_id } = sprite.source else {
        panic!("o sprite assado tem de deixar o atlas: os pixels dele agora sao base x luz");
    };
    assert!(
        !sprite.premultiplied,
        "o passe devolve alpha DIREITO, como recebeu"
    );
    assert!(
        sim.world().get::<BakedFormId>(entity).is_some(),
        "sem a identidade estavel o save nao sabe a quem devolver os canais"
    );

    // ── E o oráculo: a forma ACENDEU ────────────────────────────────────────
    let (w, h, rgba) = renderer
        .readback_individual(texture_id)
        .expect("o slot do sprite volta");
    assert_eq!((w, h), (EDGE, EDGE));
    let shaded = rgba
        .chunks_exact(4)
        .filter(|px| px[0] < 250 && px[3] > 0)
        .count();
    let total = (EDGE * EDGE) as usize;
    eprintln!("bake: {shaded} de {total} texels sairam da chapa branca");
    assert!(
        shaded * 20 > total,
        "a esfera cobre boa parte do quadro e mal escureceu {shaded} de {total} texels — \
         o gesto respondeu Ok e a tela continua uma chapa"
    );
}
