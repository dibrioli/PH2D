//! ⚠️⚠️ **Este teste MUDOU-SE da crate para a shell em 2026-09-11 (W2/L3-B)**, e a razão é a
//! armadilha §2.5 do HOWTO: ele exercita o gesto INTEIRO, e as duas metades dele deixaram de
//! viver do mesmo lado. O `drain` é da [`ph2d_app_sculpt3d`]; a tela branca sai do
//! `image_import` e os pixels do sprite saem do `hero_intents::texture_edit`, que são folhas
//! **desta shell** com 41 e 20 consumidores de famílias diferentes.
//!
//! ⛔ **Montar o sprite à mão para o manter na crate está recusado, e a recusa é do próprio
//! autor dele**, mais abaixo: *«uma fixture que montasse o sprite à mão testaria um objeto que
//! o produto não produz»*. ⇒ o teste vai para onde as duas metades se encontram.
//!
//! ⚠️⚠️ **E ele mora em `src/`, não em `tests/it/`, porque a `shells/desktop` é um BINÁRIO.**
//! A suíte de integração dela não pode chamar função nenhuma da shell — é por isso que os ~16
//! gates da escultura que lá vivem são todos **censos de FONTE**. Um teste que precisa de
//! *correr* o produto da shell só tem um sítio: um `#[cfg(test)]` dentro dela.
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
//! gates que o teriam pego moram em `ph2d-render/tests/it/impasto_light_gpu.rs` e
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

use ph2d_app_sculpt3d::Sculpt3dScene;

/// O lado da tela que o gate assa.
///
/// ⚠️ **Menor que os 1024 do produto de propósito**, e a escolha é sobre custo,
/// não sobre cobertura: nada do que este gate afirma depende do tamanho — o
/// uniform mal-formado recusava o dispatch em qualquer extensão, e a forma
/// acender é uma propriedade por-texel. O que o tamanho compra é a leitura de
/// volta, que é o item caro (`ph2d-mesh-render::form_plane`: 1,75 ms a 512²
/// contra 6,89 a 1024²).
const EDGE: u32 = 256;

/// **A BANCADA dos dois gates** — monta o mundo do produto, põe uma tela de `bg` na mesa e assa.
///
/// ⛔⛔ **Ela nasceu de uma CATRACA** (`the_shell_only_shrinks`, 21/09): o gate da tela vazia
/// duplicava, linha a linha, a montagem do irmão — `SpriteRenderer`, `SimWorld`, `AssetDb`, o mapa
/// de atlas, a esfera e o `drain`. ⚠️ *Duas cópias de uma montagem convergem enquanto ninguém mexe
/// numa delas*, e o que difere entre os dois gates é **um número**: o fundo da tela.
///
/// Devolve o veredito, o renderizador e os bits da entidade — tudo o que as asserções leem.
fn assa_uma_tela(
    gpu: &ph2d_gpu::GpuContext,
    bg: u8,
) -> (
    ph2d_app_sculpt3d::bake::Veredito,
    SpriteRenderer,
    SimWorld,
    u64,
) {
    let mut renderer =
        SpriteRenderer::new(gpu.clone(), GameRt::FORMAT, TextureAtlas::dummy(gpu), 8);
    let mut sim = SimWorld::new();
    let asset_db = AssetDb::new();
    let mut atlas_map = BTreeMap::new();
    // ⚠️ **A MESMA porta que a cena de smoke usa** para pôr a tela na mesa — uma fixtura que
    // montasse o sprite à mão testaria um objecto que o produto não produz.
    let (_, bits) = crate::image_import::spawn_blank_canvas(
        &mut sim,
        &mut renderer,
        &asset_db,
        0,
        EDGE,
        bg,
        ph2d_core::Vec2::new(0.0, 0.0),
        100.0, // LITERAL-PX-OK: pixels por metro da fixture, nao metrica de design
        &mut atlas_map,
    )
    .expect("a tela da fixture");

    let mut scene = Sculpt3dScene::new(&gpu.device, uv_sphere(48, 72, 1.0), 1.0);
    let mut forms = BTreeMap::new();
    let mut pass = ph2d_form_donation::baked_form::PassesDaLuz::default();
    let mut next_id = 0u32;
    let veredito = ph2d_app_sculpt3d::bake::drain(
        &mut scene,
        &mut forms,
        &mut pass,
        &mut next_id,
        gpu,
        true,
        Some(bits),
        &mut sim,
        &mut renderer,
        false,
        &mut |sim: &mut SimWorld, renderer: &mut SpriteRenderer| {
            // PRECISION-READONLY: a bancada LÊ os pixels para os comparar com o antes e o depois
            // do bake, e nunca os escreve de volta — quem os escreve é o `drain`.
            crate::hero_intents::texture_edit::read_sprite_source(
                ph2d_ecs::Entity::from_bits(bits),
                sim,
                renderer,
                &asset_db,
                &atlas_map,
            )
            .map(|s| s.image)
        },
        // ⚠️ **Sem rectangulo de ecra, e e' a leitura honesta desta bancada:** ela nao tem janela
        // nem camera 2D, e a cena nunca publica canvas — logo a forma e' rasterizada na vista
        // INTEIRA, que e' exactamente o que estes dois gates mediam antes do enquadramento
        // existir. *Um `Some` inventado aqui mediria um produto que ninguem corre.*
        None,
    )
    .expect("o gesto foi pedido, entao ele responde alguma coisa");
    (veredito, renderer, sim, bits)
}

/// **Os texels do slot que o sprite assado aponta** — a outra metade partilhada.
fn texels(renderer: &mut SpriteRenderer, sim: &SimWorld, bits: u64) -> Vec<[u8; 4]> {
    let ph2d_render::SpriteSource::Individual { texture_id } = sim
        .world()
        .get::<ph2d_render::Sprite>(Entity::from_bits(bits))
        .expect("o sprite continua na cena")
        .source
    else {
        panic!("o sprite assado tem de deixar o atlas: os pixels dele agora sao base x luz");
    };
    let (w, h, rgba) = renderer
        .readback_individual(texture_id)
        .expect("o slot do sprite volta");
    assert_eq!((w, h), (EDGE, EDGE));
    rgba.as_chunks::<4>().0.to_vec()
}

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
    // Branco OPACO: a luz da forma MULTIPLICA, então sobre branco o que se vê é ela e mais nada —
    // é também o que torna a asserção de aparência simples (qualquer texel abaixo de 255 veio da
    // escultura).
    let (veredito, mut renderer, sim, bits) = assa_uma_tela(&gpu, 2);
    assert!(
        veredito.assou(),
        "o gesto recusou o bake: {}",
        veredito.frase()
    );

    // ── O que o sprite virou ────────────────────────────────────────────────
    let entity = Entity::from_bits(bits);
    let sprite = sim
        .world()
        .get::<ph2d_render::Sprite>(entity)
        .expect("o sprite continua na cena");
    assert!(
        !sprite.premultiplied,
        "o passe devolve alpha DIREITO, como recebeu"
    );
    assert!(
        sim.world().get::<BakedFormId>(entity).is_some(),
        "sem a identidade estavel o save nao sabe a quem devolver os canais"
    );
    // ⭐⭐ **E o objecto passa a ser VIRÁVEL** (report do dono, 21/09) — a secção `Live Mesh` do
    // Inspector só é pintada COM o componente, e sem esta linha o artista assava a peça dele e
    // não tinha superfície nenhuma para a virar. A lei do *«só na primeira vez»* tem gate próprio
    // na família (`assar_torna_o_objecto_viravel_e_re_assar_nao_lhe_apaga_a_pose`).
    assert!(
        sim.world().get::<ph2d_ecs::Mesh3D>(entity).is_some(),
        "assar deixou o objecto sem o `Live Mesh`: a forma 3D esta' la' e o painel nao a oferece"
    );

    // ── E o oráculo: a forma ACENDEU ────────────────────────────────────────
    let px = texels(&mut renderer, &sim, bits);
    let shaded = px.iter().filter(|p| p[0] < 250 && p[3] > 0).count();
    let total = (EDGE * EDGE) as usize;
    eprintln!("bake: {shaded} de {total} texels sairam da chapa branca");
    assert!(
        shaded * 20 > total,
        "a esfera cobre boa parte do quadro e mal escureceu {shaded} de {total} texels — \
         o gesto respondeu Ok e a tela continua uma chapa"
    );
}

/// ⭐⭐⭐⭐ **E UMA TELA VAZIA SAI VISÍVEL** — o gate que o report de 21/09 pediu, pela rota do
/// produto e com a placa a decidir.
///
/// ⚠️ **Porque é que ele não existia:** o irmão acima assa a tela `bg: 2` (branca OPACA), a que a
/// cena de smoke põe na mesa, e **nenhuma fixtura desta casa alguma vez assou uma transparente** —
/// era ali que o defeito do dono vivia. O mecanismo, a leitura errada que eu fiz do report e a lei
/// que ficou estão em [`ph2d_app_sculpt3d::albedo::veste_a_forma`].
///
/// ## O que se afirma aqui, e porquê cada metade
///
/// 1. **o gesto não recusa** (era isto que ele via);
/// 2. **o resultado tem alfa** — a única metade que prova que o problema dele acabou;
/// 3. **e ele tem SOMBRA**, senão a lei podia ter pintado um quadrado branco chapado;
/// 4. ⭐ **e os CANTOS continuam vazios**: a peça é uma bola, logo uma lei que pintasse o
///    rectângulo inteiro passaria em 1–3 e falharia aqui. *É esta metade que separa «vestir a
///    forma» de «pintar a tela de branco».*
///
/// `#[ignore]`: precisa de um adapter de GPU (não há na CI).
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn uma_tela_vazia_veste_a_peca_e_sai_visivel() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter on this machine — nothing to assert");
        return;
    };
    // ⛔ **`0` = totalmente TRANSPARENTE** — a tela do report, e a única que esta casa nunca tinha
    // assado.
    let (veredito, mut renderer, sim, bits) = assa_uma_tela(&gpu, 0);
    assert!(
        veredito.assou(),
        "uma tela vazia NAO e' recusada desde 21/09: {}",
        veredito.frase()
    );

    let px = texels(&mut renderer, &sim, bits);
    let opacos = px.iter().filter(|p| p[3] > 200).count();
    let sombreados = px.iter().filter(|p| p[3] > 200 && p[0] < 250).count();
    let total = (EDGE * EDGE) as usize;
    eprintln!("tela vazia: {opacos} opacos e {sombreados} sombreados de {total}");
    assert!(
        opacos * 20 > total,
        "a peca cobre boa parte do quadro e o resultado tem so' {opacos} de {total} texels \
         opacos — e' o objecto INVISIVEL do report"
    );
    assert!(
        sombreados * 40 > total,
        "o objecto ficou visivel mas CHAPADO ({sombreados} de {total} com sombra) — a forma tem \
         de acender o branco que ela vestiu"
    );
    // ⭐ **A metade que separa «vestir a forma» de «pintar a tela»**: uma bola nao chega aos
    // cantos, e uma lei que pintasse o rectangulo inteiro passaria em tudo acima e aqui nao.
    for (x, y) in [(0, 0), (EDGE as usize - 1, 0), (0, EDGE as usize - 1)] {
        assert_eq!(
            px[y * EDGE as usize + x][3],
            0,
            "o canto ({x},{y}) da tela vazia tem de ficar VAZIO: a peca e' uma bola"
        );
    }
}
