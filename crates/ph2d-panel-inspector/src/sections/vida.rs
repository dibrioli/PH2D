//! ⭐⭐⭐ **O que o Inspector mostra da VIDA e do DANO** (plano 28, W3).
//!
//! # ⭐⭐⭐ A LEITURA VIVA é a razão de a secção HEALTH existir
//!
//! Vinte campos cabiam numa tabela genérica. O que não cabe é **quanta vida ele tem AGORA** — que
//! não vem de campo nenhum: vem do `HealthNow` que a ponte publica. É ela que transforma a secção
//! numa ferramenta que se afina **a olhar** (atirar, ver `70 of 100`, subir a armadura, atirar).
//!
//! # ⭐⭐ E a QUEIXA vem antes dos números
//!
//! A ordem é a porta [`InspectorVidaInfo::queixa`], testada sem device; aqui só se escolhe a frase.
//! ⚠️ Cada secção diz a queixa que é DELA: *sem corpo* e *morto* na vida, *não fere* no dano — e
//! *sem corpo* no dano só quando não há secção de vida por cima a dizê-lo.
//!
//! # ⭐ Linhas que SOMEM conforme os números
//!
//! O atraso da regeneração só com regeneração, as quatro linhas do escudo só com escudo, a semente
//! só com esquiva, o *Overheal* só com máximo — a lei do `SignalVerb::uses_arg`: mostrar sempre
//! entrega controlos mortos.

use super::*;
use ph2d_editor_core::interaction::format_number;
use ph2d_editor_core::vida_edits::{InspectorHealthInfo, InspectorVidaInfo, VidaQueixa};
use ph2d_editor_core::widget::{SectionFold, Unit};
use ph2d_i18n::{tr, tr_with};

const CHECK_H: f32 = 18.0; // LITERAL-PX-OK: altura visual do Checkbox, igual à das irmãs

/// **A CHAVE de cada queixa** — a porta entre o enum da lei e a língua.
#[must_use]
pub(super) const fn chave_da_queixa(q: VidaQueixa) -> &'static str {
    match q {
        VidaQueixa::SemCorpo => "panel.inspector.vida.no_body",
        VidaQueixa::Morto => "panel.inspector.vida.dead",
        VidaQueixa::NaoFere => "panel.inspector.vida.hurts_nobody",
    }
}

/// Uma caixa da secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn caixa(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    id: NodeId,
    rotulo: &str,
    ligada: bool,
) -> f32 {
    let rect = Rect::new(x, y, w, CHECK_H);
    hit_index.register(id, rect);
    paint_checkbox(
        &Checkbox::new(id, rotulo)
            .visual(store.checkbox_visual(id))
            .value(if ligada {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            }),
        rect,
        scene,
        text_system,
        theme,
    );
    y + CHECK_H + ph2d_tokens::control_gap_px()
}

/// Um campo de texto (uma equipa ou um sinal). ⛔ Pela porta que já existe (`anim_rows::text_row`).
#[allow(clippy::too_many_arguments)]
pub(super) fn nome(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    id: NodeId,
    dica: &str,
) -> f32 {
    super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        id,
        TextInput::new(id, "").placeholder(dica),
    )
}

/// **Quanta vida ele tem AGORA** — ou com quanta nasce, antes do 1.º tique.
#[allow(clippy::too_many_arguments)]
fn leitura(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    h: &InspectorHealthInfo,
) -> f32 {
    let max = format_number(f64::from(h.max));
    let texto = match h.agora {
        None => tr_with(
            "panel.inspector.vida.starts_at",
            &[("start", &format_number(f64::from(h.start)))],
        ),
        Some(a) if h.max > 0.0 => tr_with(
            "panel.inspector.vida.now_of_max",
            &[("now", &format_number(a.pontos)), ("max", &max)],
        ),
        Some(a) => tr_with(
            "panel.inspector.vida.now",
            &[("now", &format_number(a.pontos))],
        ),
    };
    let mut cur_y = super::rows::aviso(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        &texto,
        ColorToken::Text1,
    );
    if let Some(a) = h.agora
        && h.tem_escudo()
    {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &tr_with(
                "panel.inspector.vida.shield_now",
                &[("shield", &format_number(a.escudo))],
            ),
            ColorToken::Text1,
        );
    }
    cur_y
}

/// Uma lista de números numa coluna de secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn numeros(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    linhas: &[(&str, NodeId, f64, Option<Unit>)],
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let mut cur_y = y;
    for &(label, id, step, unidade) in linhas {
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
    cur_y
}

/// O corpo da secção HEALTH.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn corpo_vida(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorVidaInfo,
    h: &InspectorHealthInfo,
) -> f32 {
    let mut cur_y = y;
    if let Some(q @ (VidaQueixa::SemCorpo | VidaQueixa::Morto)) = i.queixa() {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr(chave_da_queixa(q)),
            ColorToken::Danger,
        );
    }
    cur_y = leitura(scene, text_system, theme, x, w, cur_y, h);
    if !i.clock_playing {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.vida.clock_stopped"),
            ColorToken::Text3,
        );
    }
    // ⭐⭐ **A coluna do nome é da SECÇÃO**, com os nomes da secção INTEIRA — inclusive os das
    //    linhas que este quadro não pinta: *uma coluna que salta quando uma linha aparece é uma
    //    coluna por linha com outro nome*.
    let rotulos = [
        tr("panel.inspector.vida.max_0_no_max"),
        tr("panel.inspector.vida.start"),
        tr("panel.inspector.vida.invincible"),
        tr("panel.inspector.vida.regen"),
        tr("panel.inspector.vida.regen_delay"),
        tr("panel.inspector.vida.armor"),
        tr("panel.inspector.vida.armor_fraction"),
        tr("panel.inspector.vida.dodge"),
        tr("panel.inspector.vida.seed"),
        tr("panel.inspector.vida.shield_max"),
        tr("panel.inspector.vida.shield_start"),
        tr("panel.inspector.vida.shield_duration_0_forever"),
        tr("panel.inspector.vida.shield_regen"),
        tr("panel.inspector.vida.shield_regen_delay"),
    ];
    let seccao = ph2d_editor_core::property_row::Seccao::medida(text_system, 1, &rotulos);
    let s = Some(Unit::Seconds);
    let mut linhas: Vec<(&str, NodeId, f64, Option<Unit>)> = vec![
        (rotulos[0], ids::INSP_VIDA_MAX, 1.0, None), // LITERAL-PX-OK: pontos
        (rotulos[1], ids::INSP_VIDA_START, 1.0, None), // LITERAL-PX-OK: pontos
        (rotulos[2], ids::INSP_VIDA_INVINCIBLE, 0.05, s), // LITERAL-PX-OK: segundos
        (rotulos[3], ids::INSP_VIDA_REGEN, 0.5, None), // LITERAL-PX-OK: pontos/s
    ];
    if h.regen > 0.0 {
        linhas.push((rotulos[4], ids::INSP_VIDA_REGEN_DELAY, 0.05, s)); // LITERAL-PX-OK: segundos
    }
    linhas.extend([
        (rotulos[5], ids::INSP_VIDA_ARMOR, 1.0, None), // LITERAL-PX-OK: pontos
        (rotulos[6], ids::INSP_VIDA_ARMOR_PCT, 0.05, None), // LITERAL-PX-OK: fracção
        (rotulos[7], ids::INSP_VIDA_DODGE, 0.05, None), // LITERAL-PX-OK: fracção
    ]);
    if h.dodge > 0.0 {
        linhas.push((rotulos[8], ids::INSP_VIDA_SEED, 1.0, None)); // LITERAL-PX-OK: semente
    }
    linhas.extend([
        (rotulos[9], ids::INSP_VIDA_SHIELD_MAX, 1.0, None), // LITERAL-PX-OK: pontos
        (rotulos[10], ids::INSP_VIDA_SHIELD_START, 1.0, None), // LITERAL-PX-OK: pontos
    ]);
    if h.tem_escudo() {
        linhas.extend([
            (rotulos[11], ids::INSP_VIDA_SHIELD_DURATION, 0.1, s), // LITERAL-PX-OK: segundos
            (rotulos[12], ids::INSP_VIDA_SHIELD_REGEN, 0.5, None), // LITERAL-PX-OK: pontos/s
        ]);
        if h.shield_regen > 0.0 {
            linhas.push((rotulos[13], ids::INSP_VIDA_SHIELD_REGEN_DELAY, 0.05, s)); // LITERAL-PX-OK: segundos
        }
    }
    cur_y = numeros(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        &linhas,
        seccao,
    );
    if h.max > 0.0 {
        cur_y = caixa(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            ids::INSP_VIDA_OVERHEAL,
            tr("panel.inspector.vida.overheal"),
            h.overheal,
        );
    }
    if h.tem_escudo() {
        cur_y = caixa(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            ids::INSP_VIDA_SHIELD_BLOCKS,
            tr("panel.inspector.vida.shield_blocks_excess"),
            h.shield_blocks_excess,
        );
    }
    for (id, dica) in [
        (ids::INSP_VIDA_TEAM, tr("panel.inspector.vida.team_hint")),
        (
            ids::INSP_VIDA_ON_DAMAGE,
            tr("panel.inspector.vida.on_damage_hint"),
        ),
        (
            ids::INSP_VIDA_ON_HEAL,
            tr("panel.inspector.vida.on_heal_hint"),
        ),
        (
            ids::INSP_VIDA_ON_DEATH,
            tr("panel.inspector.vida.on_death_hint"),
        ),
    ] {
        cur_y = nome(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            id,
            dica,
        );
    }
    cur_y
}

/// A moldura comum às duas secções: o cabeçalho, a dobra, e o aviso da selecção múltipla.
/// Devolve `None` quando a secção está dobrada (e o `y` a seguir ao cabeçalho).
#[allow(clippy::too_many_arguments)]
pub(super) fn cabecalho(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    seccao: NodeId,
    titulo: &str,
    selected_count: usize,
) -> Result<(SectionFold, f32), f32> {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(store, seccao, titulo);
    paint_section_header(
        &header,
        Rect::new(x, y, w, header_h),
        scene,
        text_system,
        theme,
    );
    let Some(fold) = SectionFold::begin(store, seccao, x, w, y + header_h, scene, hit_index) else {
        return Err(y + header_h);
    };
    let mut cur_y = y + header_h;
    if selected_count > 1 {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.vida.editing_the_primary_selection_only"),
            ColorToken::Text3,
        );
    }
    Ok((fold, cur_y))
}

/// Pinta a secção HEALTH. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_health_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorVidaInfo,
    h: &InspectorHealthInfo,
) -> f32 {
    let (fold, cur_y) = match cabecalho(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ph2d_editor_core::ids::INSP_LIVE_HEALTH_SECTION,
        tr("panel.inspector.vida.health"),
        info.selected_count,
    ) {
        Ok(v) => v,
        Err(y) => return y,
    };
    let cur_y = corpo_vida(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
        h,
    );
    fold.finish(store, scene, hit_index, cur_y)
}
