//! ⭐⭐⭐ **A família ÁUDIO** — o som que um objecto da cena faz (TOP-20 #4).
//!
//! # Porque ela é um módulo próprio
//!
//! É a lei declarada no [`super`]: **uma família, um módulo** (isolamento, DIRETRIZ §1.5.2.1). E
//! aqui ela não é uma formalidade: este app tem um subsistema de áudio inteiro — 42 efeitos, 23
//! presets, espectral, denoise, exportação — e **nada dele tinha consumidor de cena**. Estes dois
//! descritores são a porta pela qual esse subsistema entra no mundo do artista.
//!
//! ⛔ **A alternativa era pendurá-los na família LÓGICA, e seria mentir**: aquela é *«o que faz um
//! jogo acontecer sem uma linha de script»*, e um som não é uma regra — é uma saída. Um artista que
//! procure o som na secção das regras conclui que precisa de escrever uma para o ter.
//!
//! ⚠️ **A lista está ORDENADA por `canonical_name`** — há gate (`the_catalog_is_sorted_and_unique`).

use crate::{
    ComponentCategory as C, ComponentDesc, ComponentDesc as D, FieldDesc, FieldKind as K,
    ObjectKinds as O, Propagation,
};

const fn f(field_id: u16, name: &'static str, kind: K) -> FieldDesc {
    FieldDesc {
        field_id,
        name,
        kind,
        policy: Propagation::Propagate,
        is_ref: None,
    }
}

/// Os campos de uma fonte de som.
///
/// ⚠️ **O `Sound` é `Text` e não um tipo de campo próprio**, e é uma dívida NOMEADA: não existe
/// `FieldKind::File` neste catálogo, e acrescentar uma variante a um enum que todos os painéis
/// leem é uma wave própria (a `line/motion-value` mediu o preço da irmã dela: um `ParamRow::Note`
/// custava **104 sítios em 19 ficheiros**). A secção do Inspector põe um botão de procurar ao lado
/// do campo, que é a afordância que falta — e o campo continua a ser um caminho.
const SOURCE_FIELDS: &[FieldDesc] = &[
    f(0, "Sound", K::Text),
    f(1, "Volume (dB)", K::Scalar),
    f(2, "Pitch", K::Scalar),
    f(3, "Loop", K::Toggle),
    f(4, "Autoplay", K::Toggle),
    f(5, "Max Distance", K::Scalar),
    f(6, "Attenuation", K::Scalar),
    f(7, "Non-Spatialized Radius", K::Scalar),
    f(8, "Panning Strength", K::Scalar),
    f(9, "Max Polyphony", K::Int),
    f(10, "Bus", K::Enum),
];

/// Os descritores da família.
pub const DESCS: &[ComponentDesc] = &[
    // ⚠️ **`O::ANY`, e é a decisão**: as orelhas da cena são tantas vezes um objecto VAZIO — o
    // «ponto de escuta» que o artista põe onde o jogador vai estar — quanto uma sprite. Restringir
    // a `DRAWABLE` tornaria o caso canónico inexprimível.
    //
    // ⚠️ **Ele é um MARCADOR**: a posição vem do `Transform`, como tudo o resto. Dar-lhe
    // coordenadas próprias seria a segunda resposta à pergunta *«onde está isto?»*.
    D::authored(
        "ph2d::ecs::AudioListener2D",
        "Audio Listener 2D",
        C::Audio,
        O::ANY,
        &[],
    ),
    // ⚠️ **`O::ANY` pela mesma razão do relógio**: uma porta que range é uma sprite, mas o som de
    // ambiente de uma sala é um objecto vazio no meio dela.
    D::authored(
        "ph2d::ecs::AudioSource2D",
        "Audio Source 2D",
        C::Audio,
        O::ANY,
        SOURCE_FIELDS,
    ),
];
