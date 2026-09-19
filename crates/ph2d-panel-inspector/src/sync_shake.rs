//! **A semente dos campos das DUAS secções do ABANÃO** (suplente #25).
//!
//! ⛔⛔⛔ **Ela existe porque o painel mostraria os valores de FÁBRICA do `populate` sobre um objecto
//! autorado** — a sexta vez que esta casa a paga (o áudio e a câmera em 10/09, a vigia em 16/09, o
//! raio, o tween e o seguidor em 19/09).
//!
//! ⚠️⚠️ *Números plausíveis são a pior forma deste defeito:* eles não parecem partidos, e o artista
//! que escreva por cima de um deles **grava o default nos outros**.
//!
//! # ⭐ Duas secções, duas sementes, e elas não se cruzam
//!
//! A da câmera é de **assinatura** (os cinco números mudam juntos); a do emissor é de **aresta de
//! linha** (trocar de fonte troca o que o editor mostra). ⛔ Fundi-las faria a troca de fonte
//! reescrever os campos da câmera, que estão noutro objecto.
//!
//! # ⚠️ Os CHIPS não entram aqui
//!
//! O expoente e a cerca são pintados a partir do **SNAPSHOT** dentro do pintor, e o `event` decide a
//! partir do snapshot também. Semeá-los seria a SEGUNDA resposta à mesma pergunta.

use std::hash::{Hash, Hasher};

use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::shake_edits::InspectorShakeInfo;

use crate::state::InspectorState;

/// ⚠️ **Só o que SEMEIA um widget entra** — o `trauma` e o `activa` mudam a cada quadro e não
/// escrevem campo nenhum; pô-los aqui faria a semente correr sessenta vezes por segundo e apagar o
/// que o artista está a digitar.
fn assinatura(info: &InspectorShakeInfo) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    info.entity_bits.hash(&mut h);
    info.amplitude.to_bits().hash(&mut h);
    info.frequencia.to_bits().hash(&mut h);
    info.decaimento.to_bits().hash(&mut h);
    info.expoente.hash(&mut h);
    info.semente.hash(&mut h);
    h.finish()
}

/// **A semente da secção CAMERA SHAKE** — chamada pelo [`crate::sync_sections`].
pub(crate) fn sync_camera(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(info) = crate::state_components::current_inspector_shake() else {
        inspector_state.last_shake_sig = None;
        return;
    };
    let sig = assinatura(&info);
    if !entity_changed && inspector_state.last_shake_sig == Some(sig) {
        return;
    }
    inspector_state.last_shake_sig = Some(sig);

    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    #[allow(clippy::cast_precision_loss)]
    let semente = info.semente as f64;
    for (id, v) in [
        (crate::ids::INSP_SHAKE_AMPLITUDE, f64::from(info.amplitude)),
        (
            crate::ids::INSP_SHAKE_FREQUENCIA,
            f64::from(info.frequencia),
        ),
        (
            crate::ids::INSP_SHAKE_DECAIMENTO,
            f64::from(info.decaimento),
        ),
        (crate::ids::INSP_SHAKE_SEMENTE, semente),
    ] {
        if focus == Some(id) || drag == Some(id) {
            continue; // a mão do artista ganha ao instantâneo
        }
        host.store_mut().set_number_value(id, v);
    }
}

/// **A semente da secção SHAKE EMITTER** — de ARESTA DE LINHA, como a do gatilho.
pub(crate) fn sync_emitter(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(g) = crate::state_components::current_inspector_emitter() else {
        return;
    };
    // ⚠️ **O `min` é a mesma cerca das irmãs:** apagar a última fonte deixa o índice fora da lista,
    // e sem ele a semente cairia no `else` e o editor ficava a mostrar a fonte APAGADA.
    let linha = if g.rows.is_empty() {
        0
    } else {
        inspector_state.emitter_selected.min(g.rows.len() - 1)
    };
    let mudou = inspector_state.last_emitter_row != Some(linha);
    inspector_state.last_emitter_row = Some(linha);
    if !(entity_changed || mudou) {
        return;
    }
    let Some(row) = g.rows.get(linha) else {
        return;
    };
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    crate::sync_text_field::escreve_texto(host, focus, crate::ids::INSP_EMITTER_ON, &row.on);
    for (id, v) in [
        (crate::ids::INSP_EMITTER_FORCA, f64::from(row.forca)),
        (crate::ids::INSP_EMITTER_DENTRO, f64::from(row.dentro)),
        (crate::ids::INSP_EMITTER_FORA, f64::from(row.fora)),
    ] {
        if focus == Some(id) || drag == Some(id) {
            continue;
        }
        host.store_mut().set_number_value(id, v);
    }
}
