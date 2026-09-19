//! **A semente dos campos da secção PATH FOLLOW** (suplente #23).
//!
//! ⛔⛔⛔ **Esta linha existe porque o painel mostraria os valores de FÁBRICA do `populate` sobre um
//! objecto autorado** — a quinta vez que esta casa a paga (o áudio e a câmera em 10/09, a vigia em
//! 16/09, o raio e o tween em 19/09).
//!
//! ⚠️⚠️ *Números plausíveis são a pior forma deste defeito:* eles não parecem partidos, e o artista
//! que escreva por cima de um deles **grava o default nos outros**.
//!
//! # ⭐ E aqui só os NÚMEROS e o NOME precisam de semente
//!
//! Os quatro grupos de chips (ciclo · curva · modo · fim) e as três caixas leem a selecção do
//! **SNAPSHOT** dentro do pintor, não do store — é a lei que a §11 pagou com um report. ⇒ eles não
//! têm estado a semear, e um `set_button_state` aqui seria uma segunda resposta a *«qual está
//! aceso?»*.
//!
//! # ⚠️ De ARESTA, nunca por quadro
//!
//! Reescrever sempre apagaria o que o artista está a digitar antes de o commit da shell chegar. A
//! aresta é a **assinatura** do instantâneo, e o foco e o arrasto ganham-lhe.

use std::hash::{Hash, Hasher};

use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::path_follow_edits::InspectorPathFollowInfo;

use crate::state::InspectorState;

/// ⚠️ **Só o que SEMEIA um widget entra.**
///
/// ⚠️ **A DURAÇÃO entra**, e pela mesma razão do tween: mudar o índice do relógio troca o campo que
/// está por baixo, e sem ela o artista via o número do relógio ANTERIOR.
fn assinatura(info: &InspectorPathFollowInfo) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    info.entity_bits.hash(&mut h);
    info.caminho.hash(&mut h);
    info.relogio.hash(&mut h);
    info.duracao_us.hash(&mut h);
    info.deslocamento.to_bits().hash(&mut h);
    info.angulo.to_bits().hash(&mut h);
    info.lado.to_bits().hash(&mut h);
    h.finish()
}

/// **A semente da secção** — chamada pelo [`crate::sync_sections`].
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(info) = crate::state_components::current_inspector_path_follow() else {
        inspector_state.last_path_follow_sig = None;
        return;
    };
    let sig = assinatura(&info);
    if !entity_changed && inspector_state.last_path_follow_sig == Some(sig) {
        return;
    }
    inspector_state.last_path_follow_sig = Some(sig);

    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    // ⚠️ **O NOME da forma, pela PORTA** — e a mão do artista ganha ao instantâneo: sem a cerca do
    // foco, cada tecla digitada seria apagada pelo quadro seguinte. ⭐ É a **quarta** chamadora do
    // `escreve_texto`, que existe precisamente porque essa cerca é invisível até alguém digitar.
    crate::sync_text_field::escreve_texto(host, focus, crate::ids::INSP_PF_CAMINHO, &info.caminho);
    for (id, v) in [
        (crate::ids::INSP_PF_RELOGIO, f64::from(info.relogio)),
        (
            crate::ids::INSP_PF_DESLOCAMENTO,
            f64::from(info.deslocamento),
        ),
        (crate::ids::INSP_PF_ANGULO, f64::from(info.angulo)),
        (crate::ids::INSP_PF_LADO, f64::from(info.lado)),
    ] {
        if focus == Some(id) || drag == Some(id) {
            continue; // a mão do artista ganha ao instantâneo
        }
        host.store_mut().set_number_value(id, v);
    }
    // ⭐ **A DURAÇÃO do relógio, em SEGUNDOS** — o campo vive nesta secção e o valor vive no
    // `Timers[i]`, que é a porta com dois chamadores.
    if let Some(us) = info.duracao_us {
        let id = crate::ids::INSP_PF_DURACAO;
        if focus != Some(id) && drag != Some(id) {
            #[allow(clippy::cast_precision_loss)]
            host.store_mut().set_number_value(id, us as f64 / 1e6);
        }
    }
}
