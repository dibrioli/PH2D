//! ⭐⭐⭐ **AS DICAS DO INSPECTOR** — o que saiu de um NOME e foi para o BALÃO.
//!
//! ⛔⛔ **Ordem do dono, 2026-09-21:** *«quanto aos nomes grandes precisamos reduzir, as dicas
//! devem ser passadas para o mouse Hover»*. Este ficheiro é a casa dessa regra — ela vai crescer
//! com cada nome que encolher, e um bloco de dicas dentro do `populate.rs` levou-o a `620` contra
//! um tecto de `600` logo na PRIMEIRA.
//!
//! ⚠️ **O balão mora nos CONTROLOS e não no rótulo**, e é medição: um rótulo de bloco não tem id
//! nem rect registado, logo não há onde o pendurar sem mecanismo novo — e a mão passa é sobre o
//! controlo.
//!
//! ⚠️ **Encurtar um nome não é cosmética neste painel.** Medido em 2026-09-21: a coluna do nome é
//! `min(50 %, …)` da largura, logo um nome mais largo do que METADE come a coluna do controlo. O
//! `Per-Corner Tint (vertex gradient)` deixava as quatro amostras a `35 px`; `Per-corner Tint`
//! deixa-as a **`59`** — `68 %` mais alvo, de uma string.

use ph2d_editor_core::interaction::WidgetStore;

/// Semeia o balão de cada controlo cujo nome foi encurtado.
pub(crate) fn dicas(store: &mut WidgetStore) {
    for id in [
        crate::ids::INSP_SPRITE_CORNER_TL,
        crate::ids::INSP_SPRITE_CORNER_TR,
        crate::ids::INSP_SPRITE_CORNER_BL,
        crate::ids::INSP_SPRITE_CORNER_BR,
    ] {
        store.set_tooltip(
            id,
            ph2d_i18n::tr("panel.inspector.color_tint.per_corner_tint_hint"),
        );
    }
}
