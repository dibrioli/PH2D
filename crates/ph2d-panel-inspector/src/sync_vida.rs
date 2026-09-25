//! **A semente dos campos das secções HEALTH e DAMAGE** (plano 28, W3).
//!
//! ⛔⛔ **Ela nasce no MESMO commit que a secção, e isso é a lição de 19/09**: a secção do RAIO
//! shipou sem esta metade e o painel mostrou números plausíveis (os de fábrica) sobre um objecto
//! autorado — *eles não parecem partidos, e o artista que escreva por cima de um grava o default nos
//! outros*.
//!
//! # ⚠️ De ARESTA, nunca por quadro
//!
//! Reescrever sempre apagaria o que o artista está a digitar antes de o commit chegar. A aresta é a
//! **assinatura** do instantâneo, e o foco e o arrasto ganham-lhe.
//!
//! ⚠️ **A VIDA AGORA fica de fora da assinatura**, e é deliberado: ela muda a cada golpe e não
//! semeia widget nenhum — pô-la ali faria a secção re-semear **por quadro**, o defeito que a aresta
//! existe para não ter (a mesma cerca da munição viva da arma).

use std::hash::{Hash, Hasher};

use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::vida_edits::InspectorVidaInfo;

use crate::ids;
use crate::state::InspectorState;

/// Os números a semear, com o valor do instantâneo — `None` quando o componente não está lá.
fn numeros(i: &InspectorVidaInfo) -> Vec<(ph2d_a11y::NodeId, f64)> {
    let mut out = Vec::new();
    if let Some(h) = &i.health {
        #[allow(clippy::cast_precision_loss)]
        let seed = h.seed as f64;
        out.extend([
            (ids::INSP_VIDA_MAX, f64::from(h.max)),
            (ids::INSP_VIDA_START, f64::from(h.start)),
            (ids::INSP_VIDA_INVINCIBLE, f64::from(h.invincible_s)),
            (ids::INSP_VIDA_REGEN, f64::from(h.regen)),
            (ids::INSP_VIDA_REGEN_DELAY, f64::from(h.regen_delay_s)),
            (ids::INSP_VIDA_SHIELD_START, f64::from(h.shield_start)),
            (ids::INSP_VIDA_SHIELD_MAX, f64::from(h.shield_max)),
            (
                ids::INSP_VIDA_SHIELD_DURATION,
                f64::from(h.shield_duration_s),
            ),
            (ids::INSP_VIDA_SHIELD_REGEN, f64::from(h.shield_regen)),
            (
                ids::INSP_VIDA_SHIELD_REGEN_DELAY,
                f64::from(h.shield_regen_delay_s),
            ),
            (ids::INSP_VIDA_ARMOR, f64::from(h.armor_flat)),
            (ids::INSP_VIDA_ARMOR_PCT, f64::from(h.armor_percent)),
            (ids::INSP_VIDA_DODGE, f64::from(h.dodge)),
            (ids::INSP_VIDA_SEED, seed),
        ]);
    }
    if let Some(d) = &i.damage {
        out.push((ids::INSP_DANO_AMOUNT, f64::from(d.amount)));
    }
    if let Some(b) = &i.bar {
        out.extend([
            (ids::INSP_BARRA_WIDTH, f64::from(b.width)),
            (ids::INSP_BARRA_HEIGHT, f64::from(b.height)),
            (ids::INSP_BARRA_OFFSET_X, f64::from(b.offset_x)),
            (ids::INSP_BARRA_OFFSET_Y, f64::from(b.offset_y)),
            (ids::INSP_BARRA_TRAIL_DELAY, f64::from(b.trail_delay_s)),
            (ids::INSP_BARRA_TRAIL_SPEED, f64::from(b.trail_speed)),
        ]);
    }
    out
}

/// Os textos a semear.
fn textos(i: &InspectorVidaInfo) -> Vec<(ph2d_a11y::NodeId, &str)> {
    let mut out = Vec::new();
    if let Some(h) = &i.health {
        out.extend([
            (ids::INSP_VIDA_TEAM, h.team.as_str()),
            (ids::INSP_VIDA_ON_DAMAGE, h.on_damage.as_str()),
            (ids::INSP_VIDA_ON_HEAL, h.on_heal.as_str()),
            (ids::INSP_VIDA_ON_DEATH, h.on_death.as_str()),
        ]);
    }
    if let Some(d) = &i.damage {
        out.push((ids::INSP_DANO_TEAM, d.team.as_str()));
    }
    if let Some(b) = &i.bar {
        out.push((ids::INSP_BARRA_TARGET, b.target.as_str()));
    }
    out
}

/// ⭐⭐ **As três amostras da barra têm DOIS regimes** — a lei das amostras de tinta do Sprite e
/// das cores de script:
/// - o selector aponta para ESTA amostra ⇒ o artista está a escolher, e a diferença contra o
///   documento vai ao barramento, comparada em **`u8`** (a ida-e-volta é exacta, logo o fluxo PÁRA
///   no quadro em que o commit aterra);
/// - caso contrário ⇒ a amostra é re-semeada do documento, e é assim que um `Ctrl+Z` se vê nela.
///
/// ⛔ **Corre por QUADRO e fora da aresta da assinatura**: sem este braço a cor escolhida chegava ao
/// `widget_color` e morria ali — o fio completo até ao painel e acabado nele.
fn cores(host: &mut dyn PanelHostInternal, i: &InspectorVidaInfo) {
    use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
    use ph2d_editor_core::vida_edits::VidaFieldEdit as E;
    let Some(b) = &i.bar else {
        return;
    };
    let alvo = host.store().picker_target();
    let [fill, trail, back] = ids::INSP_BARRA_CORES;
    for (id, c, edita) in [
        (fill, b.fill, E::BarFill as fn([f32; 4]) -> E),
        (trail, b.trail, E::BarTrail),
        (back, b.back, E::BarBack),
    ] {
        let gravada = crate::state_tint::tint_f32_to_u8(c);
        if alvo == Some(id) {
            if let Some(escolhida) = host.store().widget_color(id)
                && escolhida != gravada
            {
                host.bus_mut().push(EditorAction::InspectorComponentEdit {
                    entity_bits: i.entity_bits,
                    edit: ComponentEdit::Vida(edita(crate::state_tint::tint_u8_to_f32(escolhida))),
                });
            }
        } else {
            host.store_mut().set_widget_color(id, gravada);
        }
    }
}

/// ⚠️ **Só o que SEMEIA um widget entra** — ver o cabeçalho sobre a vida agora.
pub(crate) fn assinatura(i: &InspectorVidaInfo) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    i.entity_bits.hash(&mut h);
    for (id, v) in numeros(i) {
        id.0.hash(&mut h);
        v.to_bits().hash(&mut h);
    }
    for (id, t) in textos(i) {
        id.0.hash(&mut h);
        t.hash(&mut h);
    }
    h.finish()
}

/// **A semente das secções** — chamada pelo [`crate::sync_sections`].
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(info) = crate::state_components::current_inspector_vida() else {
        inspector_state.last_vida_sig = None;
        return;
    };
    cores(host, &info);
    let sig = assinatura(&info);
    if !entity_changed && inspector_state.last_vida_sig == Some(sig) {
        return;
    }
    inspector_state.last_vida_sig = Some(sig);
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    for (id, v) in numeros(&info) {
        if focus == Some(id) || drag == Some(id) {
            continue; // a mão do artista ganha ao instantâneo
        }
        host.store_mut().set_number_value(id, v);
    }
    for (id, t) in textos(&info) {
        crate::sync_text_field::escreve_texto(host, focus, id, t);
    }
}
