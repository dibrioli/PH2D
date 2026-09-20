//! ⭐⭐⭐ **O que o Inspector mostra do ABANÃO DA CÂMERA** (suplente #25) — *como* esta câmera treme.
//!
//! # ⭐⭐ O painel DIZ porque é que ela não treme, e são TRÊS razões
//!
//! | aviso | a cura |
//! |---|---|
//! | `This is not the camera in command` | ⭐ subir a prioridade, ou ligá-la — *um abanão numa câmera que não manda lê-se exactamente como um abanão partido* |
//! | `Nothing in the scene raises a shake` | anexar um *Shake Emitter* a quem explode |
//! | `It shakes while the clock plays` | carregar em Play — o `dt` é o do passo fixo, logo em pausa o abanão **congela** |
//!
//! ⛔ **Da mais ESPECÍFICA para a mais geral** — a lei da recusa dos pincéis da escultura: *dizer «o
//! relógio está parado» a quem tem a câmera errada escolhida é mandá-lo resolver a metade errada.*
//!
//! # ⭐ E há uma quarta linha que não é um aviso: o TRAUMA a correr
//!
//! Com a cena a tocar, a secção mostra `Trauma 0.42` enquanto ela treme. ⚠️ **É um FACTO e não um
//! evento** — a lição que o `projectiles_finished` do #14 pagou: uma etiqueta que pisca uma vez não
//! responde *«está a tremer AGORA?»*.

use super::tween_editor::grupo;
use super::*;
use ph2d_editor_core::shake_edits::InspectorShakeInfo;
use ph2d_editor_core::widget::{SectionFold, Unit};
use ph2d_i18n::{tr, tr_with};

/// Os rótulos dos chips dos PERFIS. ⭐ **O motor publica a CHAVE e quem pinta é que a resolve** —
/// a lei da fronteira dos motores; um motor que devolvesse a palavra seria uma 2.ª tabela de texto.
fn rotulos_dos_perfis() -> Vec<&'static str> {
    ph2d_shake::Perfil::ALL
        .iter()
        .map(|p| tr(p.label_key()))
        .collect()
}

/// Os rótulos dos chips do expoente. ⚠️ **Eles não vêm de um `ALL` de enum** — aqui a escolha é um
/// NÚMERO (`1`, `2`, `3`), e inventar um enum para três inteiros seria a segunda representação de
/// uma faixa que a lei já declara ([`ph2d_shake::EXPOENTE_MIN`]/[`ph2d_shake::EXPOENTE_MAX`]).
fn rotulos_do_expoente() -> Vec<String> {
    (ph2d_shake::EXPOENTE_MIN..=ph2d_shake::EXPOENTE_MAX)
        .map(|n| n.to_string())
        .collect()
}

/// O corpo da secção — os CONTROLOS.
#[allow(clippy::too_many_arguments)]
fn corpo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorShakeInfo,
) -> f32 {
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[
            tr("panel.inspector.shake.amplitude"),
            tr("panel.inspector.shake.frequency"),
            tr("panel.inspector.shake.decay"),
            tr("panel.inspector.shake.seed"),
            // ⭐ A ESCOLHA também — com o nome ao lado (2026-09-23) ela entra na coluna.
            tr("panel.inspector.shake.punch"),
        ],
    );
    // ⭐⭐⭐ **Os PERFIS primeiro** — eles reescrevem os quatro números que vêm a seguir, e é isso
    // que os põe em cima: *um botão que muda os campos abaixo dele lê-se; um que os muda acima,
    // não* (a mesma ordem que o `tween_editor` já paga, com a mesma frase).
    let perfis = rotulos_dos_perfis();
    let mut cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        tr("panel.inspector.shake.preset"),
        &crate::ids::INSP_SHAKE_PERFIL,
        &perfis,
        // ⚠️ **NENHUM fica aceso, e é a decisão:** um perfil não é um MODO — depois do clique ele
        // desaparece e sobram os quatro números. Acender um prometeria um estado que o componente
        // não guarda, e ele mentiria no instante em que o artista afinasse a amplitude.
        usize::MAX,
    );
    for (id, label, step, unidade) in [
        (
            crate::ids::INSP_SHAKE_AMPLITUDE,
            tr("panel.inspector.shake.amplitude"),
            0.01, // LITERAL-PX-OK: passo de scrub em METROS de vista, como o `lado` do seguidor
            Some(Unit::Meters),
        ),
        (
            crate::ids::INSP_SHAKE_FREQUENCIA,
            tr("panel.inspector.shake.frequency"),
            0.5, // LITERAL-PX-OK: passo de scrub em Hz
            None,
        ),
        (
            crate::ids::INSP_SHAKE_DECAIMENTO,
            tr("panel.inspector.shake.decay"),
            0.1, // LITERAL-PX-OK: passo de scrub em trauma/s
            None,
        ),
    ] {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            label,
            &[id],
            step,
            unidade,
            seccao,
        );
    }
    // ⭐ **A DURAÇÃO é derivada e mostra-se**, porque `decaimento` é o número que a lei come e não o
    // que o artista pensa: ele pensa *«meio segundo»*, não *«2 de trauma por segundo»*.
    if i.decaimento > 0.0 {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &tr_with(
                "panel.inspector.shake.one_full_shake_lasts_n_s",
                &[("s", &format!("{:.2}", 1.0 / i.decaimento))],
            ),
            ColorToken::Text3,
        );
    }
    let rotulos = rotulos_do_expoente();
    let refs: Vec<&str> = rotulos.iter().map(String::as_str).collect();
    // ⚠️⚠️ **`expoente − 1` e nunca `expoente`** — a faixa começa em `1`, e passar o valor cru
    // acenderia o chip errado (ou nenhum, no topo da faixa). Há gate de ida-e-volta na shell.
    cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.shake.punch"),
        &crate::ids::INSP_SHAKE_EXPOENTE,
        &refs,
        usize::from(i.expoente.saturating_sub(ph2d_shake::EXPOENTE_MIN)),
        seccao,
    );
    super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.shake.seed"),
        &[crate::ids::INSP_SHAKE_SEMENTE],
        1.0,
        None,
        seccao,
    )
}

/// ⚠️⚠️ **A LINHA QUE RESPONDE AO «não treme nada»** — da mais específica para a mais geral.
#[allow(clippy::too_many_arguments)]
fn avisos(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorShakeInfo,
) -> f32 {
    let (chave, cor) = if !i.activa {
        (
            Some("panel.inspector.shake.this_is_not_the_camera_in_command"),
            ColorToken::Warn,
        )
    } else if !i.clock_playing {
        (
            Some("panel.inspector.shake.it_shakes_while_the_clock_plays"),
            ColorToken::Text3,
        )
    } else if i.trauma > 0.0 {
        // ⭐ Não é um aviso: é o ESTADO, e é a única coluna que vem do vivo.
        return super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            y,
            &tr_with(
                "panel.inspector.shake.shaking_trauma_n",
                &[("n", &format!("{:.2}", i.trauma))],
            ),
            ColorToken::Text2,
        );
    } else {
        (None, ColorToken::Text3)
    };
    if let Some(k) = chave {
        return super::rows::aviso(scene, text_system, theme, x, w, y, tr(k), cor);
    }
    y
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_shake_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorShakeInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        ph2d_editor_core::ids::INSP_LIVE_SHAKE_SECTION,
        tr("panel.inspector.shake.camera_shake"),
    );
    paint_section_header(
        &header,
        Rect::new(x, y, w, header_h),
        scene,
        text_system,
        theme,
    );
    let Some(fold) = SectionFold::begin(
        store,
        ph2d_editor_core::ids::INSP_LIVE_SHAKE_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;
    cur_y = avisos(scene, text_system, theme, x, w, cur_y, info);
    cur_y = corpo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
    );
    fold.finish(store, scene, hit_index, cur_y)
}
