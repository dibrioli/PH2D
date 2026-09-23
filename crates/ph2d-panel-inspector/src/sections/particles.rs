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

/// ⭐⭐ **A coluna do nome desta secção — UMA, para as três espécies de linha.**
///
/// ⚠️ Ela era medida em cada pintor sobre a tabela `ROTULOS` (os números), e as duas ESCOLHAS
/// (`Emission Shape` · `Simulation Space`) nem entravam nela porque pintavam o nome POR CIMA. Com o
/// nome ao lado (2026-09-23) elas têm de estar na medida — senão a coluna salta na linha delas.
fn seccao(text_system: &mut TextSystem) -> ph2d_editor_core::property_row::Seccao {
    let mut nomes: Vec<&str> = ROTULOS.iter().map(|(chave, _, _)| chave.tr()).collect();
    nomes.push(tr("panel.inspector.particles.emission_shape"));
    nomes.push(tr("panel.inspector.particles.simulation_space"));
    ph2d_editor_core::property_row::Seccao::medida(text_system, 1, &nomes)
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
    // ⛔⛔ **O nome estava POR CIMA** (o `titulo` e o grupo a toda a largura) — a forma que o dono
    //    reprovou duas vezes. ⇒ a porta, com a coluna da SECÇÃO.
    let segments: Vec<(&str, bool, ph2d_a11y::NodeId)> = ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (rotulos.get(i).copied().unwrap_or(""), i == escolhido, id))
        .collect();
    let sec = seccao(text_system);
    ph2d_editor_core::property_row::paint_choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        titulo_txt,
        &segments,
        sec,
    )
}

/// Uma caixa.
#[allow(clippy::too_many_arguments)]
/// Uma caixa — **pela porta**, com a coluna do nome da SECÇÃO.
///
/// ⛔⛔ Ela era uma cópia local do [`ph2d_editor_core::property_row::paint_check_row`] com a
/// altura escrita à mão (`18`, que é a aresta da MARCA e não a altura da LINHA) — o report do
/// dono de 2026-09-21: *«apenas o checkbox tem sua moldura e ele próprio menores que o padrão»*.
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
    // ⚠️ A MESMA coluna que a [`num_row`] desta secção mede, e pela mesma tabela — senão o nome
    //    de uma linha de marcar cai num `x` e o da linha de número acima dela noutro.
    let seccao = seccao(text_system);
    ph2d_editor_core::property_row::paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        (id, label, on),
        seccao,
    )
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
    let seccao = seccao(text_system);
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
