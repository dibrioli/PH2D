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

/// ⭐⭐⭐ **O PAR `(controlo, dica)`** — o que saiu de um nome e onde a mão o volta a encontrar.
///
/// ⛔⛔ **O par escreve-se À MÃO e LÊ-SE do sítio da chamada, nunca se adivinha.** A 1.ª tentativa
/// derivou-o por proximidade no fonte e mapeou o `Homing` para o `INSP_PJ_SPEED` — *um balão no
/// controlo errado é pior do que balão nenhum*. ⇒ só entram aqui os pares em que o `tr(<rótulo>)`
/// é IMEDIATAMENTE seguido pelo id: **13** dos `21` rótulos com regra.
///
/// ⏳ Os outros **8** usam outra forma de chamada (entradas de texto, linhas de lista) e ficam
/// NOMEADOS no gate `nenhum_rotulo_do_inspector_carrega_uma_regra`, com a catraca a só encolher.
const DICAS: &[(ph2d_a11y::NodeId, &str)] = &[
    (
        crate::ids::INSP_ANIM_FRAME_MS_THIS,
        "panel.inspector.animation.this_frame_hint",
    ),
    // ⭐ A última REGRA que ainda vivia num nome do Inspector (2026-09-22). ⚠️ A nota da catraca
    //    dizia que este não tinha o id do controlo ao lado do rótulo — e ele tem: é o campo
    //    seguinte da mesma tupla da tabela que pinta a fileira.
    (
        crate::ids::INSP_ANIM_REPEAT,
        "panel.inspector.animation.repeat_hint",
    ),
    (
        crate::ids::INSP_LIFE_SECONDS,
        "panel.inspector.factory.lifetime_hint",
    ),
    (
        crate::ids::INSP_FACTORY_ALIVE_MAX,
        "panel.inspector.factory.max_alive_hint",
    ),
    (
        crate::ids::INSP_FACTORY_TOTAL_MAX,
        "panel.inspector.factory.max_total_hint",
    ),
    (
        crate::ids::INSP_PJ_BOUNCINESS,
        "panel.inspector.projectile.bounciness_hint",
    ),
    (
        crate::ids::INSP_PJ_GRAVITY,
        "panel.inspector.projectile.gravity_hint",
    ),
    (
        crate::ids::INSP_PJ_HOMING_ACCEL,
        "panel.inspector.projectile.homing_hint",
    ),
    (
        crate::ids::INSP_PJ_MAX_SPEED,
        "panel.inspector.projectile.max_speed_hint",
    ),
    (
        crate::ids::INSP_PJ_RANGE,
        "panel.inspector.projectile.range_hint",
    ),
    // ⚠️ O `Size X / Y` é UM nome sobre DOIS campos — a dica vai aos dois, senão ela aparece
    //    em metade da linha.
    (
        crate::ids::INSP_SLICE_SIZE[0],
        "panel.inspector.slice.size_hint",
    ),
    (
        crate::ids::INSP_SLICE_SIZE[1],
        "panel.inspector.slice.size_hint",
    ),
    (
        crate::ids::INSP_TD_ACCEL,
        "panel.inspector.topdown.acceleration_hint",
    ),
    (
        crate::ids::INSP_TD_DECEL,
        "panel.inspector.topdown.deceleration_hint",
    ),
    (
        crate::ids::INSP_TD_TURN_SPEED,
        "panel.inspector.topdown.turn_speed_hint",
    ),
];

/// Semeia o balão de cada controlo cujo nome foi encurtado.
pub(crate) fn dicas(store: &mut WidgetStore) {
    for (id, chave) in DICAS {
        store.set_tooltip(*id, ph2d_i18n::tr(chave));
    }
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
