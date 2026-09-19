//! ⭐⭐⭐ **O que o Inspector mostra do EMISSOR DE PARTÍCULAS** (TOP-20 #18, W3).
//!
//! # ⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! | aviso | o que se passa |
//! |---|---|
//! | `The clock is stopped` | a corrida é o relógio a andar — parado, nada nasce |
//! | `Nothing alive right now` | está a correr e não há partícula nenhuma (a emissão acabou, ou está desligada) |
//! | `It starts stopped` | o `Emitting` está desmarcado: só um sinal o acende |
//!
//! ⚠️ **Sem eles, um emissor que está exactamente como o artista pediu lê-se como partido, com
//! todos os números certos no ecrã** — a lição do projéctil e do mover de vista de cima.
//!
//! # ⭐ E as linhas SOMEM conforme o modo
//!
//! A largura/altura da forma só existem fora do `Point`, a explosividade e o `One Shot` andam
//! juntos, e os quatro sinais são um grupo à parte — é a lei do `SignalVerb::uses_arg`, e a
//! alternativa (mostrar sempre) entrega um controlo morto.

use super::*;
use ph2d_editor_core::particles_edits::{
    InspectorParticlesInfo, PARTICLES_NUMBERS, PARTICLES_TEXTS, ParticlesNumber as N,
};
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;

const CHECK_H: f32 = 18.0; // LITERAL-PX-OK: altura visual do Checkbox, igual à das irmãs

/// O rótulo, o passo e a que módulo pertence cada número — **pela ordem do modelo**.
///
/// ⚠️ A tabela é indexada pela [`PARTICLES_NUMBERS`], e há gate a atar os comprimentos: uma linha
/// nova entra num sítio só.
/// ⭐ **A UNIDADE é do CAMPO, nunca do rótulo** (`line/UIUX`, spec §7): a terceira coluna é o
/// sufixo que o `NumberInput` mostra E lê de volta. `None` = grandeza adimensional (uma fracção,
/// uma contagem, um multiplicador).
const ROTULOS: [(TextKey, f64, Option<ph2d_editor_core::widget::Unit>); 19] = [
    (TextKey::new("panel.inspector.particles.amount"), 1.0, None), // LITERAL-PX-OK: partículas (inteiro)
    (
        TextKey::new("panel.inspector.particles.lifetime_s"),
        0.1, // LITERAL-PX-OK: segundos
        Some(ph2d_editor_core::widget::Unit::Seconds),
    ), // LITERAL-PX-OK: segundos
    (
        TextKey::new("panel.inspector.particles.lifetime_randomness"),
        0.05, // LITERAL-PX-OK: fracção 0..1
        None,
    ), // LITERAL-PX-OK: fracção 0..1
    (
        TextKey::new("panel.inspector.particles.explosiveness"),
        0.05, // LITERAL-PX-OK: fracção 0..1
        None,
    ), // LITERAL-PX-OK: fracção 0..1
    (
        TextKey::new("panel.inspector.particles.preprocess_s"),
        0.1, // LITERAL-PX-OK: segundos
        Some(ph2d_editor_core::widget::Unit::Seconds),
    ), // LITERAL-PX-OK: segundos
    (
        TextKey::new("panel.inspector.particles.speed_scale"),
        0.1, // LITERAL-PX-OK: multiplicador do relógio
        None,
    ), // LITERAL-PX-OK: multiplicador do relógio
    (TextKey::new("panel.inspector.particles.seed"), 1.0, None), // LITERAL-PX-OK: semente (inteiro)
    (
        TextKey::new("panel.inspector.particles.shape_width_radius_m"),
        0.1, // LITERAL-PX-OK: metros
        Some(ph2d_editor_core::widget::Unit::Meters),
    ), // LITERAL-PX-OK: metros
    (
        TextKey::new("panel.inspector.particles.shape_height_m"),
        0.1, // LITERAL-PX-OK: metros
        Some(ph2d_editor_core::widget::Unit::Meters),
    ), // LITERAL-PX-OK: metros
    (
        TextKey::new("panel.inspector.particles.speed_m_s"),
        0.5, // LITERAL-PX-OK: metros por segundo
        Some(ph2d_editor_core::widget::Unit::MetersPerSecond),
    ), // LITERAL-PX-OK: metros por segundo
    (
        TextKey::new("panel.inspector.particles.speed_randomness"),
        0.05, // LITERAL-PX-OK: fracção 0..1
        None,
    ), // LITERAL-PX-OK: fracção 0..1
    (
        TextKey::new("panel.inspector.particles.direction_deg"),
        5.0, // LITERAL-PX-OK: graus
        Some(ph2d_editor_core::widget::Unit::Degrees),
    ), // LITERAL-PX-OK: graus
    (
        TextKey::new("panel.inspector.particles.spread_deg"),
        5.0, // LITERAL-PX-OK: graus
        Some(ph2d_editor_core::widget::Unit::Degrees),
    ), // LITERAL-PX-OK: graus
    (
        TextKey::new("panel.inspector.particles.gravity_x_m_s_u"),
        0.5, // LITERAL-PX-OK: metros por segundo²
        Some(ph2d_editor_core::widget::Unit::MetersPerSecondSquared),
    ), // LITERAL-PX-OK: metros por segundo²
    (
        TextKey::new("panel.inspector.particles.gravity_y_m_s_u"),
        0.5, // LITERAL-PX-OK: metros por segundo²
        Some(ph2d_editor_core::widget::Unit::MetersPerSecondSquared),
    ), // LITERAL-PX-OK: metros por segundo²
    (TextKey::new("panel.inspector.particles.damping"), 0.1, None), // LITERAL-PX-OK: fracção por segundo
    (
        TextKey::new("panel.inspector.particles.size_m"),
        0.05, // LITERAL-PX-OK: metros
        Some(ph2d_editor_core::widget::Unit::Meters),
    ), // LITERAL-PX-OK: metros
    (
        TextKey::new("panel.inspector.particles.size_randomness"),
        0.05, // LITERAL-PX-OK: fracção 0..1
        None,
    ), // LITERAL-PX-OK: fracção 0..1
    (
        TextKey::new("panel.inspector.particles.size_at_death"),
        0.05, // LITERAL-PX-OK: fracção do tamanho ao nascer
        None,
    ), // LITERAL-PX-OK: fracção do tamanho ao nascer
];

/// Um título de módulo — a arrumação que separa *quantas* de *como saem* e de *como são*.
fn titulo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    texto: &str,
) -> f32 {
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        texto,
        x,
        y,
        font,
        w,
        resolve(ColorToken::Text3, theme),
    );
    // ⚠️ **A cauda de um bloco tem UMA porta** — o que separa o título do que ele intitula é o
    // mesmo degrau de sempre, e escrevê-lo aqui seria a quinta resposta a essa pergunta.
    y + font + ph2d_tokens::control_gap_px()
}

/// Um segmentado. ⚠️ **A selecção vem do SNAPSHOT, nunca do store** (a lei do irmão de vista de
/// cima): ler o store faria o primeiro clique depois de trocar de objecto mandar o valor anterior.
#[allow(clippy::too_many_arguments)]
fn seg_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    titulo_txt: &str,
    ids: &[ph2d_a11y::NodeId],
    rotulos: &[&'static str],
    escolhido: usize,
) -> f32 {
    let row_y = titulo(scene, text_system, theme, x, w, y, titulo_txt);
    // ⭐⭐ **A disposição é a PORTA da casa** — ver o irmão em `sections/anim.rs` (2026-09-19). Esta
    //    cópia não chegou a cortar nada porque os rótulos dela são curtos (`Point · Sphere · Box`,
    //    `Local · World`); ⚠️ *o que a fazia passar era o CORPUS, não a lei* — e uma cópia que hoje
    //    cabe é a que corta no dia em que alguém traduzir.
    let segments: Vec<(&str, bool, ph2d_a11y::NodeId)> = ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (rotulos.get(i).copied().unwrap_or(""), i == escolhido, id))
        .collect();
    let seg_h = ph2d_editor_core::widget::panel_chrome::paint_segmented_group_adaptive(
        Rect::new(x, row_y, w, ph2d_tokens::ROW_H_PX),
        &segments,
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    row_y + seg_h + ph2d_tokens::control_gap_px()
}

/// Uma caixa.
#[allow(clippy::too_many_arguments)]
fn check_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    label: &str,
    on: bool,
) -> f32 {
    let rect = Rect::new(x, y, w, CHECK_H);
    hit_index.register(id, rect);
    paint_checkbox(
        &Checkbox::new(id, label)
            .visual(store.checkbox_visual(id))
            .value(if on {
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

/// Uma linha de número, pela ordem do modelo.
#[allow(clippy::too_many_arguments)]
fn num_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    which: N,
) -> f32 {
    let Some(i) = PARTICLES_NUMBERS.iter().position(|&n| n == which) else {
        return y;
    };
    let (chave, step, unidade) = ROTULOS[i];
    let label = chave.tr();
    // ⭐⭐ **A coluna do nome é da SECÇÃO** (`line/UIUX`, 2026-09-15): aqui ela mede-se da
    //    própria tabela, que É a lista de nomes desta secção — as 19 linhas partilham-na, e
    //    medir só a desta faria a coluna saltar de linha para linha.
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &ROTULOS.map(|(chave, _, _)| chave.tr()),
    );
    super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        label,
        &[crate::ids::INSP_PART_NUM[i]],
        step,
        unidade,
        seccao,
    )
}

#[path = "particles_avisos.rs"]
mod irmao;
pub(crate) use irmao::*;
