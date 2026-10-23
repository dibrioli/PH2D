//! **A família da imagem** — a `Sprite` e o que só existe por causa dela.
//!
//! # ⭐⭐ A RÉGUA DO DONO, corrida sobre esta família (2026-09-14)
//!
//! Ver a tabela inteira no cabeçalho de [`super::core`]. As duas metades — *existe um controlo
//! **sempre** visível que o escreve?* e *ele **anexa**?* — puseram **quatro** destas entradas do
//! lado das rows, e todas pela mesma razão: o snapshot da sprite **lê-as com um neutro**
//! (`SpriteGrid::SINGLE`, `SpriteCornerTint::IDENTITY`, `EMISSIVE_OFF`), logo a row é pintada em
//! **toda** sprite e a paleta escreveria exactamente o que ela já mostra.
//!
//! ⛔ **E cinco FICAM, por medição:** a §5 9-Slice, a §11 Animation e a §12 Anchors só são
//! pintadas **se o componente estiver lá** (tabela `CASES` do `inspector_presence_tests`) — ali a
//! paleta é a única porta. *A mesma régua dá respostas opostas dentro da mesma família.*
//!
//! ⚠️ **O `AnchorVisibility` é o caso subtil:** a caixa «Always show anchors» é pintada sempre que
//! o objecto **tem âncoras** — não quando ela própria está anexada. É a forma das rows de ZONA da
//! física (visíveis dado o *Sensor*), e cai do mesmo lado.
//!
//! ⚠️ A `Sprite` mora em `ph2d-render` e as outras nove em `ph2d-ecs`; aqui elas ficam
//! juntas porque o catálogo é cortado por **família**, não por crate dona. É a mesma razão
//! por que a chave é o nome canónico: o descritor fala de tipos que ele não pode importar.
//!
//! # ✅ A `Sprite` FOI partida (ADR-0164 F1 passo 6 / ADR-0166)
//!
//! Ela é o **marcador** de [`crate::ObjectKind::Image`] e tinha **20** campos congelados; hoje
//! tem **13**. Sete saíram para três componentes — [`ph2d_ecs::SpriteCornerTint`] ·
//! [`ph2d_ecs::SpriteGrid`] · [`ph2d_ecs::SpriteRegion`] —, e a razão do corte não é tamanho:
//! *enquanto o dado for campo de um componente que todo objeto-imagem tem, não há como não o
//! mostrar*. Os três estão **aqui**, e os `field_id` da `Sprite` que eles substituem **não são
//! reusados** (a tabela é append-only, e um id reusado faria um override antigo alvejar o campo
//! novo).
//!
//! ⚠️ **O nome é `SpriteGrid` e não `SpriteSheet`**, que o plano dizia: a `ph2d-ecs` já tem
//! `SpriteSheetRef` e `SpriteSheetFrame`, e as duas significam a folha HAND-PACKED — outra coisa.
//!
//! ⚠️ A `Sprite` **não deriva `Default`** (precisa de uma `source`), logo não tem
//! `insert_default` no registo — e é por isso que ela não pode ser anexada pela paleta. Ela
//! não é uma escolha do Inspector: é o que o gesto de criar uma imagem põe lá.

use crate::{
    ComponentCategory as C, ComponentDesc as D, FieldDesc, FieldKind as K, Hidden,
    ObjectKinds as O, Propagation, View, WHOLE,
};

const fn f(field_id: u16, label_key: &'static str, kind: K) -> FieldDesc {
    FieldDesc {
        field_id,
        label_key,
        kind,
        policy: Propagation::Propagate,
        is_ref: None,
        name: "",
        key: "",
        view: crate::View::Same,
    }
}

/// ⚠️ **Parcial de propósito.** A `Sprite` tem 13 campos (eram 20 até o corte do ADR-0164 F1
/// passo 6, que levou SETE para três componentes); a F0 descreve os que a §7/§4 do Inspector já editam, e a tabela cresce
/// **append-only** por procura (F3 descreve o resto quando a seção for derivada).
/// ⛔ Nunca reordene nem reuse um `field_id`.
///
/// ⚠️ **O `field_id` 5 está APOSENTADO, não livre.** Ele era o *Frame*, e o índice de célula
/// mudou-se para o [`ph2d_ecs::SpriteGrid`] com a grelha que lhe dá sentido. Reusar o 5 para
/// outro campo faria um override por-campo gravado antes do corte (F4) apontar para o campo
/// errado — a colisão passa muda porque os dois lados são `u16`.
const SPRITE: &[FieldDesc] = &[
    f(1, "component.field.sprite.1", K::Color).named("tint"),
    f(2, "component.field.sprite.2", K::Toggle).named_as("flip_h", "flip_x"),
    f(3, "component.field.sprite.3", K::Toggle).named_as("flip_v", "flip_y"),
    f(4, "component.field.sprite.4", K::Vec2).keyed("anchor"),
    // 5 — APOSENTADO (era "Frame"; ver a nota acima).
    // ⭐ CodeEditor 08 d3: o que a §Color & Tint e a §Sprite Sheet pintam e o catálogo não tinha.
    f(6, "component.field.sprite.6", K::Color).named("self_tint"),
    f(7, "component.field.sprite.7", K::Scalar).named("opacity").viewed(View::Percent),
    f(8, "component.field.sprite.8", K::Toggle).named("tint_fill"),
    f(9, "component.field.sprite.9", K::Toggle).named("centered"),
    f(10, "component.field.sprite.10", K::Vec2).named("offset"),
];

/// Os três grupos que saíram da `Sprite` (ADR-0164 F1 passo 6 / ADR-0166).
const SPRITE_GRID: &[FieldDesc] = &[
    f(1, "component.field.sprite_grid.1", K::Int).named_as("h_frames", "hframes"),
    f(2, "component.field.sprite_grid.2", K::Int).named_as("v_frames", "vframes"),
    f(3, "component.field.sprite_grid.3", K::Int).named("frame"),
];

const SPRITE_REGION: &[FieldDesc] = &[
    f(1, "component.field.sprite_region.1", K::Vec4).named_as("region", "rect"),
    f(2, "component.field.sprite_region.2", K::Toggle).named("filter_clip"),
];

// ⚠️ Um newtype de ARRAY: cada canto é um elemento (`"[0]"`), e o bloco inteiro é a linha
// «Per-corner Tint» do cartão.
const SPRITE_CORNER_TINT: &[FieldDesc] = &[
    f(1, "component.field.sprite_corner_tint.1", K::Color).named_as("top_left", "[0]"),
    f(2, "component.field.sprite_corner_tint.2", K::Color).named_as("top_right", "[1]"),
    f(3, "component.field.sprite_corner_tint.3", K::Color).named_as("bottom_left", "[2]"),
    f(4, "component.field.sprite_corner_tint.4", K::Color).named_as("bottom_right", "[3]"),
    f(5, "component.field.sprite_corner_tint.5", K::Palette).named_as("per_corner_tint", WHOLE),
];

// ⚠️ `borders` é `[left, top, right, bottom]` — cada borda é um ELEMENTO, pela ordem do array.
const SLICE_NINE: &[FieldDesc] = &[
    f(1, "component.field.slice_nine.1", K::Scalar).named_as("left", "borders[0]"),
    f(2, "component.field.slice_nine.2", K::Scalar).named_as("right", "borders[2]"),
    f(3, "component.field.slice_nine.3", K::Scalar).named_as("top", "borders[1]"),
    f(4, "component.field.slice_nine.4", K::Scalar).named_as("bottom", "borders[3]"),
    f(5, "component.field.slice_nine.5", K::Toggle).named("fill_center"),
    f(6, "component.field.slice_nine.6", K::Enum)
        .named("tile_mode")
        .viewed(crate::options!["continuous" => Continuous, "whole" => Whole]),
    f(7, "component.field.slice_nine.7", K::Vec2).named("size"),
    // A caixa liga `Sliced`: o script lê o booleano da caixa.
    f(8, "component.field.slice_nine.8", K::Enum)
        .named_as("enable_9_slice", "draw_mode")
        .viewed(View::Toggle(stringify!(Sliced), stringify!(Simple))),
];

/// ⭐ O leitor da §11 Animation (CodeEditor 08 d3). `speed` guarda q16 e o cartão mostra `1.0x`.
const SPRITE_ANIMATOR: &[FieldDesc] = &[
    f(1, "component.field.sprite_animator.1", K::Toggle).named("playing"),
    f(2, "component.field.sprite_animator.2", K::Toggle).named("autoplay"),
    f(3, "component.field.sprite_animator.3", K::Int)
        .named_as("speed", "speed_q16")
        .viewed(View::Scale(65_536)),
    f(4, "component.field.sprite_animator.4", K::Enum).named("direction_override").viewed(crate::options![
        "inherit" => null,
        "fwd" => Forward,
        "rev" => Reverse,
        "pp" => PingPong,
        "pp_rev" => PingPongReverse,
    ]),
    f(5, "component.field.sprite_animator.5", K::Enum)
        .named("loop_override")
        .viewed(crate::options!["inherit" => null, "on" => true, "off" => false]),
];

/// As duas caixas da §12 que mostram as âncoras.
const ANCHOR_VISIBILITY: &[FieldDesc] = &[
    f(1, "component.field.anchor_visibility.1", K::Toggle)
        .named_as("always_show_anchors", "in_editor"),
    f(2, "component.field.anchor_visibility.2", K::Toggle)
        .named_as("show_anchors_at_runtime", "at_runtime"),
];

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
pub const DESCS: &[D] = &[
    // A âncora é um QUADRO na hierarquia (ADR-0072-amendment-1): o filho monta-se nela.
    // ⛔ **A nota que estava aqui dizia que `AnchorMount.anchor` vira `RefKind::Object` na F1, e
    // a F4.2 REFUTOU-A:** o campo nomeia uma âncora **do PRÓPRIO PAI**, não um objeto do mundo
    // — é uma referência RELATIVA, e uma cópia profunda leva o pai junto. O nome continua a
    // resolver dentro da cópia sem que ninguém reescreva byte nenhum.
    // ⇒ **Declará-lo como referência seria pedir um remap que estragaria o que já funciona.**
    // *A estrutura da cópia apaga o caso especial.* (Ver `shells/desktop/src/instance_refs.rs`,
    // onde o censo confere declaração ↔ remapeador.)
    // ⏳ `Reference`: o valor é o NOME de uma âncora do pai (a linha «Rides Parent Anchor»).
    D::authored(
        "ph2d::ecs::AnchorMount",
        "component.anchor_mount.name",
        C::Anchors,
        O::ANY,
        &[],
    )
    .hidden(Hidden::Reference),
    // ⇒ a porta é a caixa «Always show anchors» da §12, pintada sempre que o objecto TEM âncoras (`anchor_mount_row::paint_visibility_rows`).
    D::intrinsic(
        "ph2d::ecs::AnchorVisibility",
        "component.anchor_visibility.name",
        C::Anchors,
        ANCHOR_VISIBILITY,
    )
    .coded("anchor_visibility")
    .row_of("ph2d::ecs::NamedAnchorList"),
    D::authored(
        "ph2d::ecs::NamedAnchorList",
        "component.named_anchor_list.name",
        C::Anchors,
        O::ANY,
        &[],
    )
    .hidden(Hidden::List),
    D::authored(
        "ph2d::ecs::SliceNine",
        "component.slice_nine.name",
        C::Image,
        O::IMAGE,
        SLICE_NINE,
    )
    .coded("9_slice")
    .hiding(&[
        ("tile_modes", Hidden::OwnSection),
        ("centre_tile_mode", Hidden::OwnSection),
    ]),
    D::authored(
        "ph2d::ecs::SpriteAnimations",
        "component.sprite_animations.name",
        C::Animation,
        O::IMAGE,
        &[],
    )
    .hidden(Hidden::List),
    D::authored(
        "ph2d::ecs::SpriteAnimator",
        "component.sprite_animator.name",
        C::Animation,
        O::IMAGE,
        SPRITE_ANIMATOR,
    )
    .coded("animator")
    // Sem tocador o cartão mostra `1.0x`; o `Default` do tipo guarda 0.
    .row_without_neutral("ph2d::ecs::SpriteAnimations")
    .hiding(&[
        ("current", Hidden::Reference),
        ("elapsed_ticks", Hidden::Runtime),
        ("pingpong_reverse", Hidden::Runtime),
        ("repeat_count", Hidden::Runtime),
    ]),
    // ⭐ Os TRÊS do corte (ADR-0164 F1 passo 6 / ADR-0166) — `Authored` e `O::IMAGE`: é a
    // paleta do F3 que os anexa, e só a um objeto-imagem. A ausência de cada um é o default
    // benigno que o campo tinha, então nada aparece no Inspector até o artista o pedir.
    // ⇒ a porta é as rows do degradê de cantos, pintadas em toda sprite (ausente = os quatro cantos brancos).
    D::intrinsic(
        "ph2d::ecs::SpriteCornerTint",
        "component.sprite_corner_tint.name",
        C::Image,
        SPRITE_CORNER_TINT,
    )
    .coded("corner_tint")
    .row_of("ph2d::render::Sprite"),
    // ⚠️ Categoria `Image` e não `Animation`: a grelha é um FATO da textura (como ela se
    // divide), e é a `SpriteAnimations` que a percorre. Pô-la em Animation faria um sprite com
    // folha estática — o caso comum — procurar a grelha na secção errada.
    // ⇒ a porta é a §4 Sprite Sheet, pintada em toda sprite (ausente = `SpriteGrid::SINGLE`).
    D::intrinsic(
        "ph2d::ecs::SpriteGrid",
        "component.sprite_grid.name",
        C::Image,
        SPRITE_GRID,
    )
    .coded("sprite_grid")
    .row_of("ph2d::render::Sprite"),
    // Os pixels editados desta sprite (`project_sprite_pixels.rs`): identidade de CONTEÚDO,
    // posta pelo funil de commit das oito ferramentas de imagem. O artista não a anexa.
    D::machinery(
        "ph2d::ecs::SpritePixels",
        "component.sprite_pixels.name",
        C::Image,
    ),
    // ⇒ a porta é as rows de REGIÃO do §Render Source, pintadas em toda sprite (`paint_region_rows`, sem condição).
    D::intrinsic(
        "ph2d::ecs::SpriteRegion",
        "component.sprite_region.name",
        C::Image,
        SPRITE_REGION,
    )
    .coded("region")
    // A presença é a caixa «Region»: ausente não é um rectângulo.
    .row_without_neutral("ph2d::render::Sprite"),
    // Proveniência de autoria (que folha esta sprite veio de), não índice de célula — o
    // índice vivo é o `SpriteGrid::frame`. Máquina: quem a põe é o importador.
    D::machinery(
        "ph2d::ecs::SpriteSheetFrame",
        "component.sprite_sheet_frame.name",
        C::Image,
    ),
    D::machinery(
        "ph2d::ecs::SpriteSheetRef",
        "component.sprite_sheet_ref.name",
        C::Image,
    ),
    // ⚠️ O MARCADOR de ObjectKind::Image. Sem `Default` ⇒ sem `insert_default` ⇒ a paleta
    // não a oferece; ela chega pelo gesto que cria a imagem. **Tem seção**, e das maiores —
    // é o caso que a variante `Intrinsic` existe para exprimir.
    D::intrinsic(
        "ph2d::render::Sprite",
        "component.sprite.name",
        C::Image,
        SPRITE,
    )
    .coded("sprite")
    .hiding(&[
        ("source", Hidden::Reference),
        ("size", Hidden::NoCard),
        ("anchor", Hidden::NoCard),
        ("version", Hidden::Editor),
    ]),
];

#[cfg(test)]
mod tests {
    use super::DESCS;

    /// ⭐⭐ **A imagem oferece INTENÇÕES, e nenhuma row de secção sempre-pintada.**
    ///
    /// Irmão do `the_core_family_offers_intentions_and_not_rows` e do
    /// `the_physics_family_offers_one_door_and_not_its_rows`, e existe pela mesma razão: esta
    /// família nasceu com o helper autorado e ninguém a classificou item a item. Medido em
    /// 2026-09-14: `9` oferecidas, `4` delas rows que toda sprite já mostra.
    ///
    /// ⚠️ **A lista é a afirmação.** Uma entrada nova é a pergunta de duas metades do cabeçalho.
    ///
    /// (Mutação: devolver o `SpriteGrid` a `D::authored` ⇒ RED, nomeando-o.)
    #[test]
    fn the_image_family_offers_intentions_and_not_rows() {
        const PORTAS: [&str; 5] = [
            "ph2d::ecs::AnchorMount",
            "ph2d::ecs::NamedAnchorList",
            "ph2d::ecs::SliceNine",
            "ph2d::ecs::SpriteAnimations",
            "ph2d::ecs::SpriteAnimator",
        ];
        let offered: Vec<&str> = DESCS
            .iter()
            .filter(|d| d.is_offered())
            .map(|d| d.canonical_name)
            .collect();
        assert_eq!(
            offered,
            PORTAS.to_vec(),
            "a paleta da IMAGEM tem de oferecer exatamente estas 5 intencoes.\n\
             Antes de acrescentar uma, responda as duas metades: existe um controlo SEMPRE \
             visivel que a escreve (o snapshot da sprite le-a com um neutro?), e esse controlo \
             ANEXA-a? Se sim, e' `D::intrinsic`."
        );
        assert!(
            DESCS.len() >= 12,
            "a familia encolheu para {} — o gate acima deixaria de afirmar",
            DESCS.len()
        );
    }
}
