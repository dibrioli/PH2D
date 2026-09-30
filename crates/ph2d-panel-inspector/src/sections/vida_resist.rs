//! ⭐⭐⭐ **A tabela de RESISTÊNCIAS da secção HEALTH** (plano 28, W6).
//!
//! # A forma é a da máquina de estados, e de propósito
//!
//! **Uma lista, `+ Add` / `x Remove`, e UM editor para a linha aberta** — os três campos de cada
//! linha desenhados sempre custariam `3 × 8 = 24` controlos numa coluna que mostra ~30 linhas. ⭐ A
//! lista e os botões são OS da máquina de estados ([`super::statemachine::lista`] e
//! [`super::statemachine::botoes`]), e não uma cópia: ⛔ a casa tem QUATRO cópias daquelas duas
//! funções (gatilho · cérebro · emissor · vigia), e uma quinta seria a próxima a divergir.
//!
//! ⚠️ **Irmão do [`super::vida`] por RESPONSABILIDADE** — uma tabela é outro assunto que vinte
//! números, e aquele ficheiro vive a `473` de um tecto de `600`.
//!
//! # ⭐⭐ O que a lista DIZ que os campos sozinhos não diriam
//!
//! Cada linha lê-se como a LEI a lê: `fogo  immune` · `gelo  ×2` · `fogo  heals ×1` — e uma linha
//! com um tipo que outra ACIMA já tem pinta-se em WARN, porque a lei lê só a primeira (a porta da
//! repetida é a mesma [`ph2d_physics_ecs::kind_key`] da lei, vinda no instantâneo).

use super::*;
use ph2d_editor_core::interaction::format_number;
use ph2d_editor_core::vida_edits::{InspectorHealthInfo, InspectorResistanceRow};
use ph2d_i18n::{tr, tr_with};

/// **O que uma linha É, como a lei a lê** — `fogo  immune` · `gelo  ×2` · `fogo  heals ×1`.
///
/// ⚠️ **Imune é a taxa `0` SEM absorver** — com o «absorve» ligado a taxa é quanto CURA, e `0`
/// ali é uma cura de nada (o oráculo: `absorve_cheio` não faz nada), que se lê como «heals ×0».
fn resumo(r: &InspectorResistanceRow) -> String {
    let kind = if r.kind.trim().is_empty() {
        tr("panel.inspector.vida.resist_untyped").to_string()
    } else {
        r.kind.clone()
    };
    let rate = format_number(f64::from(r.rate));
    if r.absorbs {
        tr_with(
            "panel.inspector.vida.resist_heals",
            &[("kind", &kind), ("rate", &rate)],
        )
    } else if r.rate <= 0.0 {
        tr_with("panel.inspector.vida.resist_immune", &[("kind", &kind)])
    } else {
        tr_with(
            "panel.inspector.vida.resist_times",
            &[("kind", &kind), ("rate", &rate)],
        )
    }
}

/// **A tabela inteira** — o título, a lista (ou a frase de vazia), os dois botões e o editor da
/// linha aberta. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn corpo_resistencias(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    h: &InspectorHealthInfo,
    aberta: Option<usize>,
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let mut cur_y = super::rows::aviso(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        tr("panel.inspector.vida.resistances"),
        ColorToken::Text2,
    );
    if h.resistances.is_empty() {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.vida.no_resistances"),
            ColorToken::Text3,
        );
    } else {
        let linhas: Vec<(String, bool)> = h
            .resistances
            .iter()
            .map(|r| (resumo(r), r.repetida))
            .collect();
        cur_y = super::statemachine::lista(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            &linhas,
            &ids::INSP_VIDA_RESIST_ROW,
            aberta.unwrap_or(0),
        );
    }
    cur_y = super::statemachine::botoes(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        (
            ids::INSP_VIDA_RESIST_ADD,
            tr("panel.inspector.vida.add_resistance"),
        ),
        (
            ids::INSP_VIDA_RESIST_REMOVE,
            tr("panel.inspector.vida.x_remove_resistance"),
        ),
        // ⚠️ **O `+` DESAPARECE no tecto** e não fica cinzento a mentir — a lei das irmãs.
        h.resistances.len() < ids::INSP_VIDA_RESIST_ROW.len(),
        !h.resistances.is_empty(),
    );
    let Some(r) = aberta.and_then(|k| h.resistances.get(k)) else {
        return cur_y;
    };
    cur_y = super::vida::nome(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.vida.resist_kind"),
        ids::INSP_VIDA_RESIST_KIND,
        tr("panel.inspector.vida.resist_kind_hint"),
        seccao,
    );
    cur_y = super::vida::numeros(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        &[(
            tr("panel.inspector.vida.resist_rate"),
            ids::INSP_VIDA_RESIST_RATE,
            0.05, // LITERAL-PX-OK: multiplicador
            None,
        )],
        seccao,
    );
    cur_y = super::vida::caixa(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        ids::INSP_VIDA_RESIST_ABSORBS,
        tr("panel.inspector.vida.resist_absorbs"),
        r.absorbs,
        seccao,
    );
    if r.repetida {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.vida.resist_repeated"),
            ColorToken::Warn,
        );
    }
    cur_y
}
