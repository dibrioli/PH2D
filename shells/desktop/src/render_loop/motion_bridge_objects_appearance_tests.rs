//! **A APARÊNCIA de um sprite como FONTE** — qual loja responde por cada variante do
//! `SpriteSource` (doc 89 folha 14, a última célula e a cerca 7 da §4).
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 600 para `shells/`), e o corte é por
//! PERGUNTA: o irmão mede *o que o grafo recebe quando nomeia X*; este mede *de onde a imagem
//! de X vem*.

/// ⭐ **UM SPRITE KTX2 ASSADO DEIXA DE SER FONTE INVISÍVEL** (doc 89 folha 14, a última célula
/// e a cerca 7 da §4).
///
/// ⚠️ **A cerca era ENCANAMENTO com cara de decisão.** Ela dizia *"um sprite KTX2 nomeado é
/// fonte invisível, por decisão declarada"* — e a razão escrita ao lado do `None` era
/// *"resolve through `renderer.cooked_texture_id`, **not in hand**"*. O chamador tinha-o na
/// linha ao lado (é de lá que sai o `atlas()`). *Uma decisão cuja razão é «não tenho isto
/// aqui» é um adiamento, e o que a dissolve é passar aquilo.*
#[test]
fn a_cooked_ktx2_sprite_is_no_longer_an_invisible_source() {
    use ph2d_render::SpriteSource;
    let atlas_uv = |_k: u32| [0.1, 0.2, 0.3, 0.4];
    let logical = ph2d_asset::LogicalTextureId::from_source_bytes(b"uma-textura");

    // A loja CONHECE o id lógico ⇒ o rect INTEIRO e o id que ela deu.
    let known = |id: ph2d_asset::LogicalTextureId| (id == logical).then_some(42);
    let got = super::appearance::appearance_of(
        SpriteSource::CookedTexture {
            logical_id: logical,
        },
        &atlas_uv,
        &known,
    );
    assert_eq!(
        got,
        Some(([0.0, 0.0, 1.0, 1.0], 42)),
        "um KTX2 resolvido e' o rect inteiro (a textura e' dela propria, nao uma celula \
         de um atlas partilhado) com o id da loja"
    );

    // ⚠️ E a cerca 6 da MESMA folha continua de pé: id desconhecido ⇒ `None` ⇒ o nó não
    // emite nada — *não adivinha e não falha*.
    let empty = |_: ph2d_asset::LogicalTextureId| None;
    assert_eq!(
        super::appearance::appearance_of(
            SpriteSource::CookedTexture {
                logical_id: logical,
            },
            &atlas_uv,
            &empty,
        ),
        None,
        "artefacto por carregar continua a nao emitir nada, e nao a adivinhar"
    );
}

/// **E as duas variantes que já funcionavam não se mexeram** — o controle que impede o gate
/// acima de provar só a metade nova.
#[test]
fn the_two_sources_that_already_worked_are_untouched() {
    use ph2d_render::{RenderInstance, SpriteSource};
    let atlas_uv = |_k: u32| [0.1, 0.2, 0.3, 0.4];
    let never = |_: ph2d_asset::LogicalTextureId| None;
    assert_eq!(
        super::appearance::appearance_of(SpriteSource::Atlas { key: 3 }, &atlas_uv, &never),
        Some(([0.1, 0.2, 0.3, 0.4], RenderInstance::ATLAS_TEXTURE_ID)),
        "o atlas continua a dar a celula empacotada"
    );
    assert_eq!(
        super::appearance::appearance_of(
            SpriteSource::Individual { texture_id: 7 },
            &atlas_uv,
            &never
        ),
        Some(([0.0, 0.0, 1.0, 1.0], 7)),
        "e a individual, o rect inteiro com o handle dela"
    );
}
